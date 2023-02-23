use std::{
    io::Cursor,
    mem,
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use base64::Engine;
use bytes::Bytes;
use flume::Receiver;

use image::GenericImageView;
use nanoid::nanoid;
use parking_lot::Mutex;

use crate::{
    config::{
        FILE_ID_LENGTH, MAX_SEND_CHUNK_BUFFER_LENGTH, MAX_TEMP_FILE_SIZE, MESSAGE_ID_LENGTH,
        PREVIEW_IMAGE_DIMENSION,
    },
    events::complete_pool_file_download_event,
    poolpb::{
        pool_direct_message::{
            Data as PoolDirectMessageData, DirectType as PoolDirectMessageType, LatestReplyData,
        },
        pool_message::{
            media_offer_data::MediaData, Data as PoolMessageData, FileRequestData, MediaOfferData,
            NodeInfoData, RetractFileOfferData, RetractFileRequestData, TextData,
            Type as PoolMessageType,
        },
        PoolChunkMessage, PoolDirectMessage, PoolFileInfo, PoolImageData, PoolMediaType,
        PoolMessage, PoolMessagePackage, PoolMessagePackageDestinationInfo,
        PoolMessagePackageSourceInfo,
    },
    store::file_store::FileStore,
    MESSAGES_DB, STORE_MANAGER,
};

use super::{
    cache_manager::CacheManager,
    chunk::{
        chunk_ranges::{create_full_chunk_range, ChunkRanges, ChunkRangesUtil},
        chunk_util::chunk_number_to_partner_int_path,
    },
    file_manager::FileManager,
    message_util::{
        message_package_bundle::MessagePackageBundle, received_message_queue::ReceivedMessageQueue,
    },
    pool_conn::PoolConn,
    pool_state::PoolState,
};

pub(super) struct SendChunkInfo {
    chunk_msg: PoolChunkMessage,
    dest_node_ids: Option<Vec<String>>,
    send_to_self: bool,
}

impl SendChunkInfo {
    pub(super) fn create(
        file_id: String,
        chunk_number: u64,
        chunk: Vec<u8>,
        dest_node_ids: Option<Vec<String>>,
        send_to_self: bool,
    ) -> Self {
        SendChunkInfo {
            chunk_msg: PoolChunkMessage {
                file_id,
                chunk_number,
                chunk: Bytes::from(chunk),
            },
            dest_node_ids,
            send_to_self,
        }
    }
}

pub(super) struct PoolNet {
    pool_state: Arc<PoolState>,
    pool_conn: Arc<PoolConn>,

    file_manager: Arc<FileManager>,
    cache_manager: Option<Arc<CacheManager>>,

    missed_messages: Mutex<Vec<MessagePackageBundle>>,
    received_messages: Mutex<ReceivedMessageQueue>,
}

impl PoolNet {
    pub(super) fn init(pool_state: Arc<PoolState>, pool_conn: Arc<PoolConn>) -> Arc<Self> {
        let (send_chunk_tx, send_chunk_rx) =
            flume::bounded::<SendChunkInfo>(MAX_SEND_CHUNK_BUFFER_LENGTH);

        let file_manager = FileManager::init(pool_state.clone(), send_chunk_tx.clone());
        let cache_manager = CacheManager::init(pool_state.clone(), send_chunk_tx.clone());

        let pool_net: Arc<PoolNet> = Arc::new(PoolNet {
            pool_state,
            pool_conn,
            file_manager,
            cache_manager,
            missed_messages: Mutex::new(Vec::new()),
            received_messages: Mutex::new(ReceivedMessageQueue::new()),
        });

        let pool_net_clone = pool_net.clone();
        tokio::spawn(async move {
            pool_net_clone.send_chunk_loop(send_chunk_rx).await;
        });

        pool_net
            .file_manager
            .pool_net_ref
            .store(Some(pool_net.clone()));

        pool_net
    }

    pub(super) async fn clean(&self) {
        self.file_manager.clean();
        if let Some(cache_manager) = &self.cache_manager {
            cache_manager.clean();
        }
    }

    pub(super) async fn send_latest_request(&self, target_node_id: &String) {
        self.send_direct_message(PoolDirectMessageType::LatestRequest, None, target_node_id)
            .await;
    }

    pub(super) async fn send_latest_reply(&self, target_node_id: &String) {
        let latest_reply_data = LatestReplyData {
            latest_messages: MESSAGES_DB.latest_messages(&self.pool_state.pool_id),
            file_seeders: self.pool_state.collect_file_seeders(),
        };

        self.send_direct_message(
            PoolDirectMessageType::LatestReply,
            Some(PoolDirectMessageData::LatestReplyData(latest_reply_data)),
            target_node_id,
        )
        .await;
    }

    pub(super) async fn send_missed_messages(&self) {
        let missed_messages = {
            let mut missed_messages = self.missed_messages.lock();

            if missed_messages.is_empty() {
                return;
            }

            if self.pool_conn.is_fully_connected() {
                mem::take(&mut *missed_messages)
            } else {
                missed_messages.clone()
            }
        };

        for msg in missed_messages {
            self.pool_conn.distribute_message(msg).await;
        }
    }

    pub(super) async fn send_node_info_data(&self) {
        let file_offers = STORE_MANAGER.file_offers(&self.pool_state.pool_id);

        let node_info_data = NodeInfoData { file_offers };

        self.send_message(
            PoolMessageType::NodeInfo,
            Some(PoolMessageData::NodeInfoData(node_info_data)),
            None,
            None,
        )
        .await;
    }

    pub(super) async fn send_text_message(&self, text: String) {
        let text_data = TextData { text };

        self.send_message(
            PoolMessageType::Text,
            Some(PoolMessageData::TextData(text_data)),
            None,
            None,
        )
        .await
    }

    pub(super) async fn send_file_offer(&self, file_offer: PoolFileInfo, path: PathBuf) {
        if !STORE_MANAGER.add_file_offer(&self.pool_state.pool_id, file_offer.clone(), path.clone())
        {
            return;
        }

        self.file_manager
            .add_chunk_sender(file_offer.clone(), path, false);

        self.send_message(
            PoolMessageType::FileOffer,
            Some(PoolMessageData::FileOfferData(file_offer)),
            None,
            None,
        )
        .await;
    }

    pub(super) async fn send_image_offer(&self, file_offer: PoolFileInfo, path: PathBuf) {
        if file_offer.total_size > MAX_TEMP_FILE_SIZE {
            self.send_file_offer(file_offer, path).await;
            return;
        }

        let image = match image::open(path.clone()) {
            Ok(image) => image,
            _ => return,
        };

        let (width, height) = image.dimensions();

        let image = if width < height {
            image.resize(
                PREVIEW_IMAGE_DIMENSION,
                height,
                image::imageops::FilterType::Nearest,
            )
        } else {
            image.resize(
                width,
                PREVIEW_IMAGE_DIMENSION,
                image::imageops::FilterType::Nearest,
            )
        };

        let mut preview_img_buf = Vec::new();
        match image.write_to(
            &mut Cursor::new(&mut preview_img_buf),
            image::ImageOutputFormat::Png,
        ) {
            Ok(_) => {}
            _ => return,
        };

        let preview_image_base64 = format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(preview_img_buf)
        );

        if !STORE_MANAGER.add_file_offer(&self.pool_state.pool_id, file_offer.clone(), path.clone())
        {
            return;
        }

        self.file_manager
            .add_chunk_sender(file_offer.clone(), path, true);

        let file_id = file_offer.file_id.clone();

        let media_offer_data = MediaOfferData {
            file_info: Some(file_offer),
            media_type: PoolMediaType::Image.into(),
            media_data: Some(MediaData::ImageData(PoolImageData {
                width,
                height,
                preview_image_base64,
            })),
        };

        self.send_message(
            PoolMessageType::MediaOffer,
            Some(PoolMessageData::MediaOfferData(media_offer_data)),
            None,
            None,
        )
        .await;

        self.file_manager.broadcast_file(file_id);
    }

    pub(super) async fn download_file(&self, file_info: PoolFileInfo, mut dir_path: PathBuf) {
        if let Some(existing_path) = STORE_MANAGER.check_existing_file(&file_info.file_id) {
            let pool_id = self.pool_state.pool_id.clone();
            tokio::spawn(async move {
                FileStore::create_valid_file_path(&mut dir_path, &file_info.file_name);
                let success = tokio::fs::copy(existing_path, dir_path).await.is_ok();

                complete_pool_file_download_event(&pool_id, file_info.file_id, success);
            });
            return;
        }

        if !self.pool_state.is_available_file(&file_info.file_id) {
            return;
        }

        let file_id = file_info.file_id.clone();
        let full_chunk_range = create_full_chunk_range(file_info.total_size);

        if let Ok(request_node_id) = self
            .file_manager
            .init_file_download(file_info, Some(dir_path))
        {
            self.send_file_request(file_id, request_node_id, full_chunk_range, false)
                .await;
        }
    }

    pub(super) async fn send_file_request(
        &self,
        file_id: String,
        request_node_id: String,
        requested_chunks: ChunkRanges,
        request_from_origin: bool,
    ) {
        let file_request_data = FileRequestData {
            file_id,
            requested_chunks,
            promised_chunks: Vec::new(),
            request_from_origin,
        };

        for i in 0..3 {
            self.send_message(
                PoolMessageType::FileRequest,
                Some(PoolMessageData::FileRequestData(file_request_data.clone())),
                Some(vec![request_node_id.clone()]),
                Some(i),
            )
            .await;
        }
    }

    pub(super) async fn send_retract_file_offer(&self, file_id: String) {
        if !STORE_MANAGER.remove_file_offer(&file_id) {
            return;
        }

        self.file_manager.remove_chunk_sender(&file_id);

        self.send_message(
            PoolMessageType::RetractFileOffer,
            Some(PoolMessageData::RetractFileOfferData(
                RetractFileOfferData { file_id },
            )),
            None,
            None,
        )
        .await;
    }

    pub(super) async fn send_retract_file_request(&self, file_id: String) {
        let requested_node_id = match self.file_manager.download_requested_node_id(&file_id) {
            Some(requested_node_id) => requested_node_id,
            None => return,
        };

        self.file_manager.complete_file_download(&file_id, false);

        self.send_message(
            PoolMessageType::RetractFileRequest,
            Some(PoolMessageData::RetractFileRequestData(
                RetractFileRequestData { file_id },
            )),
            Some(vec![requested_node_id]),
            None,
        )
        .await;
    }

    pub(super) async fn send_direct_message(
        &self,
        msg_type: PoolDirectMessageType,
        msg_data: Option<PoolDirectMessageData>,
        target_node_id: &String,
    ) {
        let mut msg_pkg = self.create_message_package(None, None);
        msg_pkg.direct_msg = Some(PoolDirectMessage {
            r#type: msg_type.into(),
            data: msg_data,
        });

        self.pool_conn
            .send_data_channel(
                target_node_id,
                &MessagePackageBundle::create(msg_pkg, String::new()),
            )
            .await;
    }

    async fn send_message(
        &self,
        msg_type: PoolMessageType,
        msg_data: Option<PoolMessageData>,
        dest_node_ids: Option<Vec<String>>,
        partner_int_path: Option<u32>,
    ) {
        let created = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(created) => created,
            Err(_) => return,
        };

        let mut msg_pkg = self.create_message_package(dest_node_ids, partner_int_path);
        msg_pkg.msg = Some(PoolMessage {
            msg_id: nanoid!(MESSAGE_ID_LENGTH),
            r#type: msg_type.into(),
            user_id: self.pool_state.user.user_id.clone(),
            created: created.as_millis() as u64,
            data: msg_data,
        });

        self.handle_message(MessagePackageBundle::create(
            msg_pkg,
            self.pool_state.node_id.clone(),
        ))
        .await;
    }

    fn add_message(&self, msg: PoolMessage) {
        MESSAGES_DB.add_message(&self.pool_state.pool_id, msg);
    }

    fn add_missed_message(&self, msg_pkg_bundle: &MessagePackageBundle) {
        if !self.pool_conn.is_fully_connected() {
            let mut missed_messages = self.missed_messages.lock();
            missed_messages.push(msg_pkg_bundle.clone());
        }
    }

    fn update_latest(&self, latest_reply_data: LatestReplyData) {
        if self.pool_state.is_latest() {
            // Maybe use diff algorithm to get any extra data?
            return;
        }

        self.pool_state.set_latest();

        log::debug!("update_latest {:?}", latest_reply_data);

        {
            let mut received_messages = self.received_messages.lock();
            for msg in &latest_reply_data.latest_messages {
                received_messages.append_message(&msg.msg_id);
            }
        }

        MESSAGES_DB
            .init_latest_messages(&self.pool_state.pool_id, latest_reply_data.latest_messages);

        self.pool_state
            .init_file_seeders(latest_reply_data.file_seeders);
    }

    fn update_node_info(&self, target_node_id: &String, node_info_data: NodeInfoData) {
        self.pool_state
            .add_file_offers(target_node_id, node_info_data.file_offers);
    }

    // Promise chunks and returns true if promised
    fn promise_chunks(
        &self,
        requesting_node_id: String,
        file_request_data: &mut FileRequestData,
        partner_int_path: u32,
    ) -> bool {
        if file_request_data.request_from_origin || file_request_data.requested_chunks.is_empty() {
            return false;
        }

        file_request_data.requested_chunks.compact();

        if !self
            .file_manager
            .promise_file_chunks(
                requesting_node_id.clone(),
                file_request_data,
                partner_int_path,
            )
        {
            if let Some(cache_manager) = &self.cache_manager {
                return cache_manager
                    .promise_cache_chunks(requesting_node_id, file_request_data, partner_int_path);
            }
            false
        } else {
            true
        }
    }

    async fn send_chunk_loop(&self, send_chunk_rx: Receiver<SendChunkInfo>) {
        loop {
            tokio::select! {
                _ = self.pool_state.close_signal() => {
                    return;
                },
                Ok(send_chunk_info) = send_chunk_rx.recv_async() => {
                    self.send_chunk(send_chunk_info).await;
                }
            }
        }
    }

    async fn send_chunk(&self, chunk_info: SendChunkInfo) {
        if chunk_info.send_to_self {
            self.file_manager
                .handle_file_chunk(chunk_info.chunk_msg.clone());
        }
        
        // log::debug!("send_chunk {}", chunk_info.chunk_msg.chunk_number);

        let partner_int_path = chunk_number_to_partner_int_path(chunk_info.chunk_msg.chunk_number);
        let mut msg_pkg =
            self.create_message_package(chunk_info.dest_node_ids, Some(partner_int_path));
        msg_pkg.chunk_msg = Some(chunk_info.chunk_msg);

        self.pool_conn
            .distribute_message(MessagePackageBundle::create(
                msg_pkg,
                self.pool_state.node_id.clone(),
            ))
            .await;
    }

    pub(super) async fn handle_chunk(&self, mut msg_pkg_bundle: MessagePackageBundle) {
        if msg_pkg_bundle.msg_pkg.dests.is_empty() {
            self.file_manager
                .handle_file_chunk(msg_pkg_bundle.take_chunk_msg());
        } else {
            if msg_pkg_bundle.check_and_update_is_dest(&self.pool_state.node_id) {
                self.file_manager
                    .handle_file_chunk(msg_pkg_bundle.take_chunk_msg());

                if msg_pkg_bundle.msg_pkg.dests.is_empty() {
                    return;
                }
            } else if let Some(cache_manager) = &self.cache_manager {
                if let Some(partner_int_path) = msg_pkg_bundle.msg_pkg.partner_int_path {
                    if partner_int_path == self.pool_state.partner_int() as u32 {
                        let chunk_msg = msg_pkg_bundle.take_chunk_msg();

                        if self.pool_state.is_available_file(&chunk_msg.file_id) {
                            cache_manager.cache_file_chunk(chunk_msg);
                        }
                    }
                }
            } else {
                // shouldn't happen much
            }
        }

        self.pool_conn.distribute_message(msg_pkg_bundle).await;
    }

    pub(super) async fn handle_direct_message(&self, mut msg_pkg_bundle: MessagePackageBundle) {
        let direct_msg = msg_pkg_bundle.take_direct_msg(); // should never panic or else logic error
        let src_node_id = msg_pkg_bundle.src_node_id(); // should never panic or else logic error

        match direct_msg.r#type() {
            PoolDirectMessageType::LatestRequest => {
                self.send_latest_reply(&src_node_id).await;
            }
            PoolDirectMessageType::LatestReply => {
                let latest_reply_data = match direct_msg.data {
                    Some(PoolDirectMessageData::LatestReplyData(latest_reply_data)) => {
                        latest_reply_data
                    }
                    _ => return,
                };

                self.update_latest(latest_reply_data);
            }
        }
    }

    pub(super) async fn handle_message(&self, mut msg_pkg_bundle: MessagePackageBundle) {
        // log::debug!("UNPROCESSED handle_message {:?}", msg_pkg_bundle.msg_pkg);

        let mut msg = msg_pkg_bundle.take_msg(); // should never panic or else logic error
        let src_node_id = msg_pkg_bundle.src_node_id(); // should never panic or else logic error

        {
            let mut received_messages = self.received_messages.lock();
            if !received_messages.append_message(&msg.msg_id) {
                return;
            }
        }

        log::debug!("handle_message {:?} {:?}", msg_pkg_bundle.msg_pkg, msg);

        if msg_pkg_bundle.msg_pkg.dests.len() != 0 {
            if msg_pkg_bundle.check_and_update_is_dest(&self.pool_state.node_id) {
                match msg.r#type() {
                    PoolMessageType::FileRequest => {
                        let file_request_data = match msg.data {
                            Some(PoolMessageData::FileRequestData(file_request_data)) => {
                                file_request_data
                            }
                            _ => return,
                        };

                        self.file_manager
                            .request_file(src_node_id.clone(), file_request_data);
                    }
                    PoolMessageType::RetractFileRequest => {
                        let retract_file_request_data = match msg.data {
                            Some(PoolMessageData::RetractFileRequestData(
                                retract_file_request_data,
                            )) => retract_file_request_data,
                            _ => return,
                        };

                        self.file_manager
                            .retract_file_request(&retract_file_request_data.file_id);
                    }
                    _ => return,
                }

                if msg_pkg_bundle.msg_pkg.dests.is_empty() {
                    return;
                }
            } else {
                let mut modified = false;
                match msg.r#type() {
                    PoolMessageType::FileRequest => {
                        let file_request_data = match &mut msg.data {
                            Some(PoolMessageData::FileRequestData(file_request_data)) => {
                                file_request_data
                            }
                            _ => return,
                        };

                        if let Some(partner_int_path) = msg_pkg_bundle.msg_pkg.partner_int_path {
                            if partner_int_path as usize == self.pool_state.partner_int()
                                || src_node_id == self.pool_state.node_id
                            {
                                modified = self
                                    .promise_chunks(
                                        src_node_id,
                                        file_request_data,
                                        partner_int_path,
                                    );
                            }
                        }
                    }
                    _ => (),
                }
                if modified {
                    log::info!("Modified Messages {:?}", msg);

                    msg_pkg_bundle.msg_pkg.msg = Some(msg);
                    msg_pkg_bundle = MessagePackageBundle::create(
                        msg_pkg_bundle.msg_pkg,
                        msg_pkg_bundle.from_node_id.clone(),
                    );
                }
            }
        } else {
            match msg.r#type() {
                PoolMessageType::NodeInfo => {
                    let node_info_data = match msg.data {
                        Some(PoolMessageData::NodeInfoData(node_info_data)) => node_info_data,
                        _ => return,
                    };

                    if src_node_id != self.pool_state.node_id {
                        self.update_node_info(&src_node_id, node_info_data);
                    }
                }
                PoolMessageType::Text => {
                    let _ = match &msg.data {
                        Some(PoolMessageData::TextData(text_data)) => text_data,
                        _ => return,
                    };

                    self.add_message(msg);
                }
                PoolMessageType::FileOffer => {
                    let file_info = match &msg.data {
                        Some(PoolMessageData::FileOfferData(file_offer_data)) => file_offer_data,
                        _ => return,
                    };

                    let is_original = src_node_id == file_info.origin_node_id;

                    self.pool_state.add_file_offer(&src_node_id, &file_info);

                    if is_original {
                        self.add_message(msg);
                    }
                }
                PoolMessageType::MediaOffer => {
                    let media_offer_data = match &msg.data {
                        Some(PoolMessageData::MediaOfferData(media_offer_data)) => media_offer_data,
                        _ => return,
                    };

                    match media_offer_data.media_type() {
                        PoolMediaType::Image => {
                            let _ = match &media_offer_data.media_data {
                                Some(MediaData::ImageData(image_data)) => image_data,
                                _ => return,
                            };

                            let file_info = match &media_offer_data.file_info {
                                Some(file_info) => file_info,
                                _ => return,
                            };

                            if file_info.total_size > MAX_TEMP_FILE_SIZE {
                                return;
                            }

                            if src_node_id != self.pool_state.node_id {
                                let _ = self
                                    .file_manager
                                    .init_file_download(file_info.clone(), None);
                            }

                            self.pool_state.add_file_offer(&src_node_id, file_info);
                            self.add_message(msg);
                        }
                    }
                }
                PoolMessageType::RetractFileOffer => {
                    let retract_file_offer_data = match &msg.data {
                        Some(PoolMessageData::RetractFileOfferData(retract_file_offer_data)) => {
                            retract_file_offer_data
                        }
                        _ => return,
                    };

                    self.pool_state
                        .remove_file_offer(&src_node_id, &retract_file_offer_data.file_id);
                }
                _ => return,
            }
        }

        self.add_missed_message(&msg_pkg_bundle);

        self.pool_conn.distribute_message(msg_pkg_bundle).await;
    }

    pub(super) fn create_message_package(
        &self,
        dest_node_ids: Option<Vec<String>>,
        partner_int_path: Option<u32>,
    ) -> PoolMessagePackage {
        let dests = if let Some(dest_node_ids) = dest_node_ids {
            let mut dests = Vec::with_capacity(dest_node_ids.len());

            for node_id in dest_node_ids {
                dests.push(PoolMessagePackageDestinationInfo { node_id });
            }

            dests
        } else {
            Vec::new()
        };

        PoolMessagePackage {
            src: Some(PoolMessagePackageSourceInfo {
                node_id: self.pool_state.node_id.clone(),
                path: self.pool_state.node_position_path(),
            }),
            dests,
            partner_int_path,
            msg: None,
            chunk_msg: None,
            direct_msg: None,
        }
    }

    pub(super) fn generate_file_offer(path: PathBuf, node_id: String) -> Option<PoolFileInfo> {
        if let Ok(metadata) = path.metadata() {
            if !metadata.is_file() {
                return None;
            }

            let file_name = match path.file_name() {
                Some(file_name) => {
                    if let Some(file_name) = file_name.to_str() {
                        file_name.to_string()
                    } else {
                        return None;
                    }
                }
                None => return None,
            };

            return Some(PoolFileInfo {
                file_id: nanoid::nanoid!(FILE_ID_LENGTH),
                file_name,
                total_size: metadata.len(),
                origin_node_id: node_id,
            });
        }
        return None;
    }
}
