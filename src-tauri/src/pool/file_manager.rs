use std::{
    collections::{HashMap, VecDeque},
    fs::{remove_file, File},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering, AtomicBool},
        Arc, Weak,
    },
    time::{Instant, SystemTime},
};

use anyhow::{anyhow, Result};
use arc_swap::ArcSwapOption;
use flume::{Receiver, Sender};
use log::info;
use parking_lot::{Mutex, RwLock};

use crate::{
    config::{
        CHUNKS_MISSING_POLLING_INTERVAL, CHUNK_SIZE, MAX_CHUNKS_MISSING_RETRY,
        MAX_POLL_COUNT_BEFORE_SEND, MAX_TEMP_FILE_SIZE,
    },
    pool::chunk::{chunk_ranges::create_full_chunk_range, chunk_util::total_size_to_total_chunks},
    poolpb::{pool_message::FileRequestData, PoolChunkMessage, PoolFileInfo},
    store::file_store::{FileStore, TempFile},
    STORE_MANAGER,
};

use super::{
    chunk::{
        chunk_ranges::{ChunkRanges, ChunkRangesUtil},
        chunk_util::chunk_number_to_cache_chunk_number,
    },
    pool_net::{PoolNet, SendChunkInfo},
    pool_state::PoolState,
};

pub const FILE_PROGRESS_STATUS_DOWNLOADING: usize = 0;
pub const FILE_PROGRESS_STATUS_RETRYING: usize = 1;

pub struct FileDownloadStatus {
    status: AtomicUsize,
    progress: AtomicUsize, // 0 - 100
}

struct FileRequest {
    file_id: String,
    requesting_node_id: String,
    requested_chunks: ChunkRanges,
    promised_chunks: HashMap<u64, ChunkRanges>,
    start_chunk_number: u64,
    next_chunk_number: u64,
    chunk_missing_range_number: usize,
    new: bool,
    wrapped: bool,
}

struct ChunkSender {
    file_info: PoolFileInfo,
    total_chunks: u64,
    full_chunk_range: ChunkRanges,

    broadcasting: AtomicBool,
    path: PathBuf,

    file_requests: Mutex<VecDeque<FileRequest>>,
    file_manager_ref: Weak<FileManager>,
}

pub(super) struct FileDownload {
    file_info: PoolFileInfo,
    full_chunk_range: ChunkRanges,

    path: PathBuf,
    is_temp: bool,

    total_chunks: u64,
    chunks_downloaded: u64,
    chunks_downloaded_ranges: ChunkRanges,
}

pub(super) struct FileManager {
    pool_state: Arc<PoolState>,
    pub(super) pool_net_ref: ArcSwapOption<PoolNet>,

    file_downloads: Mutex<HashMap<String, FileDownload>>, // file_id -> file_download
    chunk_handlers: RwLock<HashMap<String, Sender<PoolChunkMessage>>>, // file_id -> chunk_handler
    chunk_senders: RwLock<HashMap<String, Arc<ChunkSender>>>, // file_id -> chunk_sender

    send_chunk_tx: Sender<SendChunkInfo>,
}

impl FileManager {
    pub(super) fn init(
        pool_state: Arc<PoolState>,
        send_chunk_tx: Sender<SendChunkInfo>,
    ) -> Arc<Self> {
        let file_manager = Arc::new(FileManager {
            pool_state,
            file_downloads: Mutex::new(HashMap::new()),
            chunk_handlers: RwLock::new(HashMap::new()),
            chunk_senders: RwLock::new(HashMap::new()),
            send_chunk_tx,
            pool_net_ref: ArcSwapOption::empty(),
        });

        if let Some(file_offers) =
            STORE_MANAGER.file_offers_with_path(&file_manager.pool_state.pool_id)
        {
            let mut chunk_senders = file_manager.chunk_senders.write();
            for (path, file_info) in file_offers {
                chunk_senders.insert(
                    file_info.file_id.clone(),
                    ChunkSender::new(file_info, path, Arc::downgrade(&file_manager), false),
                );
            }
        }

        file_manager
    }

    pub(super) fn clean(&self) {
        {
            let chunk_senders = self.chunk_senders.write();
            for chunk_sender in chunk_senders.values() {
                chunk_sender.clean();
            }
        }

        {
            let mut chunk_handlers = self.chunk_handlers.write();
            chunk_handlers.clear();
        }

        {
            let mut file_downloads = self.file_downloads.lock();
            file_downloads.clear();
        }

        self.pool_net_ref.store(None);
    }

    pub(super) fn add_chunk_sender(self: &Arc<Self>, file_info: PoolFileInfo, path: PathBuf, intend_to_broadcast: bool) {
        let mut chunk_senders = self.chunk_senders.write();
        chunk_senders.insert(
            file_info.file_id.clone(),
            ChunkSender::new(file_info, path, Arc::downgrade(self), intend_to_broadcast),
        );
    }

    pub(super) fn remove_chunk_sender(&self, file_id: &String) {
        let mut chunk_senders = self.chunk_senders.write();
        chunk_senders.remove(file_id);
    }

    pub(super) fn handle_file_chunk(&self, chunk_msg: PoolChunkMessage) {
        let chunk_handlers = self.chunk_handlers.read();
        if let Some(chunk_handler) = &chunk_handlers.get(&chunk_msg.file_id) {
            let _ = chunk_handler.send(chunk_msg);
        }
    }

    pub(super) async fn promise_file_chunks(
        &self,
        requesting_node_id: String,
        file_request_data: &mut FileRequestData,
        partner_int_path: u32,
    ) -> bool {
        let downloading_promised_chunks = {
            let file_downloads = self.file_downloads.lock();
            if let Some(file_download) = file_downloads.get(&file_request_data.file_id) {
                let promised_chunks = file_request_data.requested_chunks.promise_valid_chunks(
                    &file_download.chunks_downloaded_ranges,
                    &mut file_request_data.promised_chunks,
                    partner_int_path,
                );

                if promised_chunks.is_empty() {
                    None
                } else {
                    Some(promised_chunks)
                }
            } else {
                None
            }
        };

        let chunk_senders = self.chunk_senders.read();
        let chunk_sender = match chunk_senders.get(&file_request_data.file_id) {
            Some(chunk_sender) => chunk_sender,
            None => return false,
        };

        let promised_chunks = match downloading_promised_chunks {
            Some(downloading_promised_chunks) => downloading_promised_chunks,
            None => file_request_data.requested_chunks.promise_valid_chunks(
                &chunk_sender.full_chunk_range,
                &mut file_request_data.promised_chunks,
                partner_int_path,
            ),
        };

        if promised_chunks.is_empty() {
            return false;
        }

        file_request_data.requested_chunks =
            file_request_data.requested_chunks.diff(&promised_chunks);

        let file_request_data = FileRequestData {
            file_id: file_request_data.file_id.clone(),
            requested_chunks: promised_chunks,
            promised_chunks: Vec::new(),
            request_from_origin: false,
        };

        chunk_sender.add_request(requesting_node_id, file_request_data);

        true
    }

    pub(super) fn request_file(
        &self,
        requesting_node_id: String,
        file_request_data: FileRequestData,
    ) {
        let chunk_senders = self.chunk_senders.read();
        if let Some(chunk_sender) = chunk_senders.get(&file_request_data.file_id) {
            chunk_sender.add_request(requesting_node_id, file_request_data);
        }
    }

    pub(super) fn retract_file_request(&self, file_id: &String) {
        let chunk_senders = self.chunk_senders.read();
        if let Some(chunk_sender) = chunk_senders.get(file_id) {
            chunk_sender.retract_request(file_id);
        }
    }
    
    pub(super) fn broadcast_file(&self, file_id: String) {
        let chunk_senders = self.chunk_senders.read();
        if let Some(chunk_sender) = chunk_senders.get(&file_id) {
            chunk_sender.broadcast_file(file_id);
        }
    }

    // Returns Ok(true) if new file download has been initialized
    // Returns Ok(false) if download already exists
    // Returns Err(_) if file download cannot be initialized
    pub(super) fn init_file_download(
        self: &Arc<Self>,
        file_info: PoolFileInfo,
        dir_path: Option<PathBuf>,
    ) -> Result<bool> {
        if self.has_file_download(&file_info.file_id) {
            return Ok(false);
        }

        let (path, is_temp) = match dir_path {
            Some(mut path) => {
                FileStore::create_valid_file_path(&mut path, &file_info.file_name);
                (path, false)
            }
            None => {
                if file_info.total_size > MAX_TEMP_FILE_SIZE {
                    return Err(anyhow!("temp file too big"));
                }

                let path = match FileStore::temp_file_path(
                    self.pool_state.pool_id.clone(),
                    file_info.file_id.clone(),
                ) {
                    Some(path) => path,
                    _ => return Err(anyhow!("cannot store temp files")),
                };

                (path, true)
            }
        };

        let file_handle = File::options()
            .write(true)
            .create(true)
            .open(path.clone())
            .ok();

        let file_handle = match file_handle {
            Some(file_handle) => file_handle,
            _ => return Err(anyhow!("cannot store temp files")),
        };

        let total_chunks = total_size_to_total_chunks(file_info.total_size);

        let file_download = FileDownload {
            file_info: file_info.clone(),
            full_chunk_range: create_full_chunk_range(file_info.total_size),
            path: path.clone(),
            is_temp,
            total_chunks,
            chunks_downloaded: 0,
            chunks_downloaded_ranges: ChunkRanges::new(),
        };

        let mut file_downloads = self.file_downloads.lock();
        file_downloads.insert(file_info.file_id.clone(), file_download);

        drop(file_downloads);

        let file_download_status = Arc::new(FileDownloadStatus {
            status: AtomicUsize::new(FILE_PROGRESS_STATUS_DOWNLOADING),
            progress: AtomicUsize::new(0),
        });

        let (handle_chunk_tx, handle_chunk_rx) = flume::unbounded::<PoolChunkMessage>();

        let mut chunk_handlers = self.chunk_handlers.write();
        chunk_handlers.insert(file_info.file_id.clone(), handle_chunk_tx);
        drop(chunk_handlers);

        self.add_chunk_sender(file_info.clone(), path, false);

        info!(
            "Downloading file {}, at {:?}",
            file_info.file_id,
            Instant::now()
        );

        let file_manager_clone = self.clone();
        std::thread::spawn(move || {
            file_manager_clone.chunk_handler_loop(
                file_info,
                file_handle,
                handle_chunk_rx,
                file_download_status,
            );
        });

        Ok(true)
    }

    fn chunk_sender_loop(self: Arc<Self>, file_id: String, mut broadcast: bool) {
        let chunk_sender = {
            let chunk_senders = self.chunk_senders.read();
            match chunk_senders.get(&file_id) {
                Some(chunk_sender) => chunk_sender.clone(),
                None => {
                    self.retract_file_offer(file_id);
                    return;
                }
            }
        };

        let mut file_handle = match File::options().read(true).open(chunk_sender.path.clone()) {
            Ok(file_handle) => file_handle,
            Err(_) => {
                self.retract_file_offer(file_id);
                return;
            }
        };

        let total_chunks = chunk_sender.total_chunks;
        let last_chunk = total_chunks - 1;
        let last_chunk_size = {
            let last_chunk_size =
                (chunk_sender.file_info.total_size % (CHUNK_SIZE as u64)) as usize;
            if last_chunk_size == 0 {
                CHUNK_SIZE
            } else {
                last_chunk_size
            }
        };

        let mut chunk_number: u64 = 0;
        loop {
            let dest_node_ids: Option<Vec<String>> = if !broadcast {
                let mut file_requests = chunk_sender.file_requests.lock();
                let file_requests_len = file_requests.len();

                if file_requests.len() == 0 {
                    return;
                }

                let wrap = chunk_number >= total_chunks;
                if wrap {
                    chunk_number = 0;
                }

                let mut dest_node_ids: Vec<String> = Vec::with_capacity(file_requests_len);
                let mut min_next_chunk_number = total_chunks + 1;
                for i in (0..file_requests_len).rev() {
                    let req = &mut file_requests[i];

                    if !self
                        .pool_state
                        .check_is_node_active(&req.requesting_node_id)
                    {
                        file_requests.remove(i);
                        continue;
                    }

                    if req.new {
                        req.start_chunk_number = chunk_number;
                        req.next_chunk_number = chunk_number;
                        req.new = false;
                    }

                    if wrap {
                        req.next_chunk_number = 0;
                        req.chunk_missing_range_number = 0;
                        req.wrapped = true;
                    }

                    if req.next_chunk_number > chunk_number {
                        if req.next_chunk_number < min_next_chunk_number {
                            dest_node_ids.clear();
                            dest_node_ids.push(req.requesting_node_id.clone());
                            min_next_chunk_number = req.next_chunk_number;
                        } else if req.next_chunk_number == min_next_chunk_number {
                            dest_node_ids.push(req.requesting_node_id.clone());
                        }
                        continue;
                    } else {
                        req.next_chunk_number = chunk_number;
                    }

                    while req.next_chunk_number < total_chunks {
                        if req.next_chunk_number
                            < req.requested_chunks[req.chunk_missing_range_number].start
                        {
                            req.next_chunk_number =
                                req.requested_chunks[req.chunk_missing_range_number].start;
                        } else if req.next_chunk_number
                            > req.requested_chunks[req.chunk_missing_range_number].end
                        {
                            let chunks_missing_len = req.requested_chunks.len();
                            loop {
                                req.chunk_missing_range_number += 1;
                                if req.chunk_missing_range_number >= chunks_missing_len {
                                    req.next_chunk_number = total_chunks;
                                    break;
                                }

                                if req.next_chunk_number
                                    <= req.requested_chunks[req.chunk_missing_range_number].end
                                {
                                    if req.next_chunk_number
                                        < req.requested_chunks[req.chunk_missing_range_number].start
                                    {
                                        req.next_chunk_number =
                                            req.requested_chunks[req.chunk_missing_range_number].start;
                                    }
                                    break;
                                }
                            }
                        }

                        if let Some(promised_chunk_ranges) = req
                            .promised_chunks
                            .get(&chunk_number_to_cache_chunk_number(req.next_chunk_number))
                        {
                            if let Some(promised_chunk_range) =
                                promised_chunk_ranges.find_chunk_range(req.next_chunk_number)
                            {
                                req.next_chunk_number = promised_chunk_range.end + 1;
                                continue;
                            }
                        }

                        break;
                    }

                    if req.wrapped && req.next_chunk_number >= req.start_chunk_number {
                        file_requests.remove(i);
                        continue;
                    }

                    if req.next_chunk_number < min_next_chunk_number {
                        dest_node_ids.clear();
                        dest_node_ids.push(req.requesting_node_id.clone());
                        min_next_chunk_number = req.next_chunk_number;
                    } else if req.next_chunk_number == min_next_chunk_number {
                        dest_node_ids.push(req.requesting_node_id.clone());
                    }
                }

                if file_requests.is_empty() {
                    continue;
                }

                chunk_number = min_next_chunk_number;
                if chunk_number >= total_chunks {
                    continue;
                }

                Some(dest_node_ids)
            } else {
                if chunk_number >= total_chunks {
                    chunk_number = 0;
                    broadcast = false;
                    chunk_sender.broadcasting.store(false, Ordering::SeqCst);
                    continue;   
                }

                None
            };

            let mut buf = if chunk_number == last_chunk {
                vec![0u8; last_chunk_size]
            } else {
                vec![0u8; CHUNK_SIZE]
            };

            let read_ok = if let Ok(_) =
                file_handle.seek(SeekFrom::Start(chunk_number * CHUNK_SIZE as u64))
            {
                file_handle.read_exact(&mut buf).is_ok()
            } else {
                false
            };

            if !read_ok {
                self.retract_file_offer(file_id);
                return;
            }

            let send_chunk_info = SendChunkInfo::create(
                file_id.clone(),
                chunk_number,
                buf,
                dest_node_ids,
                false,
            );

            if self.send_chunk_tx.send(send_chunk_info).is_err() {
                return;
            }

            chunk_number += 1;
        }
    }

    pub(super) fn chunk_handler_loop(
        self: Arc<Self>,
        file_info: PoolFileInfo,
        mut file_handle: File,
        handle_chunk_rx: Receiver<PoolChunkMessage>,
        file_download_status: Arc<FileDownloadStatus>,
    ) {
        let mut cached_seeders: Vec<String> =
            match self.pool_state.sorted_file_seeders(&file_info.file_id) {
                Some(seeders) => seeders,
                None => return,
            };

        self.request_chunks_missing(
            file_info.file_id.clone(),
            cached_seeders[0].clone(),
            create_full_chunk_range(file_info.total_size),
            false,
        );

        let mut is_done = false;
        let mut is_missing = false;
        let mut last_progress = 0;

        let mut retry_count = 0;
        let mut last_request_sent_count = MAX_POLL_COUNT_BEFORE_SEND;
        let mut seeder_index = 0;

        loop {
            let chunk_msg = match handle_chunk_rx.recv_timeout(CHUNKS_MISSING_POLLING_INTERVAL) {
                Ok(chunk_msg) => chunk_msg,
                Err(flume::RecvTimeoutError::Disconnected) => return,
                Err(flume::RecvTimeoutError::Timeout) => {
                    let chunks_missing = {
                        let file_downloads = self.file_downloads.lock();
                        let file_download = match file_downloads.get(&file_info.file_id) {
                            Some(file_download) => file_download,
                            None => return,
                        };

                        // Note chunks_missing should never be empty or else it would've finished downloading
                        file_download
                            .full_chunk_range
                            .diff(&file_download.chunks_downloaded_ranges)
                    };

                    if last_request_sent_count == MAX_POLL_COUNT_BEFORE_SEND {
                        last_request_sent_count = 0;
                    } else {
                        last_request_sent_count += 1;
                        continue;
                    }

                    if retry_count < MAX_CHUNKS_MISSING_RETRY {
                        if is_missing {
                            retry_count += 1;
                        } else {
                            retry_count = 0;
                        }
                    } else {
                        if seeder_index == cached_seeders.len() {
                            cached_seeders =
                                match self.pool_state.sorted_file_seeders(&file_info.file_id) {
                                    Some(seeders) => seeders,
                                    None => {
                                        self.complete_file_download(&file_info.file_id, false);
                                        return;
                                    }
                                };

                            seeder_index = 0;
                        } else {
                            seeder_index += 1;
                        }

                        retry_count = 0;
                    }

                    if !is_missing {
                        file_download_status
                            .status
                            .store(FILE_PROGRESS_STATUS_RETRYING, Ordering::Relaxed);
                    }

                    is_missing = true;

                    self.request_chunks_missing(
                        file_info.file_id.clone(),
                        cached_seeders[seeder_index].clone(),
                        chunks_missing,
                        retry_count == MAX_CHUNKS_MISSING_RETRY,
                    );
                    continue;
                }
            };

            let mut file_downloads = self.file_downloads.lock();
            let mut file_download = match file_downloads.get_mut(&file_info.file_id) {
                Some(file_download) => file_download,
                None => return,
            };

            if file_download
                .chunks_downloaded_ranges
                .has_chunk(chunk_msg.chunk_number)
            {
                continue;
            }

            if is_missing {
                file_download_status
                    .status
                    .store(FILE_PROGRESS_STATUS_DOWNLOADING, Ordering::Relaxed);
                is_missing = false;
            }

            file_download
                .chunks_downloaded_ranges
                .add_chunk(chunk_msg.chunk_number);
            file_download.chunks_downloaded += 1;

            let progress =
                ((file_download.chunks_downloaded * 100) / file_download.total_chunks) as usize;
            if last_progress != progress {
                last_progress = progress;
                file_download_status
                    .progress
                    .store(progress, Ordering::Relaxed);
            }

            if file_download.chunks_downloaded == file_download.total_chunks {
                is_done = true;
            }

            let chunk = if chunk_msg.chunk_number == file_download.total_chunks - 1 {
                let end = (file_info.total_size % (CHUNK_SIZE as u64)) as usize;
                if end != 0 {
                    &chunk_msg.chunk[..end]
                } else {
                    &chunk_msg.chunk
                }
            } else {
                &chunk_msg.chunk
            };

            drop(file_downloads);

            let offset = chunk_msg.chunk_number * (CHUNK_SIZE as u64);

            let write_ok = if let Ok(_) = file_handle.seek(SeekFrom::Start(offset)) {
                file_handle.write_all(chunk).is_ok()
            } else {
                false
            };

            if !write_ok || is_done {
                self.complete_file_download(&file_info.file_id, !write_ok);
                return;
            }
        }
    }

    fn complete_file_download(&self, file_id: &String, fail_override: bool) {
        let mut file_downloads = self.file_downloads.lock();
        let file_download = match file_downloads.remove(file_id) {
            Some(file_download) => file_download,
            None => return,
        };

        let success =
            !fail_override && file_download.chunks_downloaded == file_download.total_chunks;
        info!(
            "Completed file {}, at {:?}. Success: {}",
            file_id,
            Instant::now(),
            success
        );

        if success {
            if file_download.is_temp {
                self.add_temp_file(TempFile {
                    file_id: file_download.file_info.file_id.clone(),
                    file_size: file_download.file_info.total_size,
                    created: SystemTime::now(),
                    path: file_download.path.clone(),
                });
            }

            self.seed_file(&file_download.path, file_download.file_info);
        } else {
            let _ = remove_file(file_download.path);
        }

        drop(file_downloads);

        let mut chunk_handlers = self.chunk_handlers.write();
        chunk_handlers.remove(file_id);
        drop(chunk_handlers);

        // Fire complete file status (including success data)
        todo!()
    }

    fn add_temp_file(&self, temp_file: TempFile) {
        if let Some(mut removed_temp_files) =
            STORE_MANAGER.add_temp_file(&self.pool_state.pool_id, temp_file)
        {
            for temp_file in removed_temp_files.drain(..) {
                self.retract_file_offer(temp_file.file_id);
            }
        }
    }

    pub(super) fn has_file_download(&self, file_id: &String) -> bool {
        self.chunk_handlers.read().contains_key(file_id)
    }

    fn request_chunks_missing(
        &self,
        file_id: String,
        request_node_id: String,
        requested_chunks: ChunkRanges,
        request_from_origin: bool,
    ) {
        if let Some(pool_net) = self.pool_net_ref.load_full() {
            tokio::spawn(async move {
                pool_net
                    .send_file_request(
                        file_id,
                        request_node_id,
                        requested_chunks,
                        request_from_origin,
                    )
                    .await;
            });
        }
    }

    fn seed_file(&self, path: &PathBuf, file_info: PoolFileInfo) {
        let path_str = match path.to_str() {
            Some(path_str) => path_str.to_string(),
            None => return,
        };

        if let Some(pool_net) = self.pool_net_ref.load_full() {
            tokio::spawn(async move {
                pool_net.send_file_offer(path_str, file_info).await;
            });
        }
    }

    fn retract_file_offer(&self, file_id: String) {
        if let Some(pool_net) = self.pool_net_ref.load_full() {
            tokio::spawn(async move {
                pool_net.send_retract_file_offer(&file_id).await;
            });
        }
    }
}

impl ChunkSender {
    pub fn new(
        file_info: PoolFileInfo,
        path: PathBuf,
        file_manager_ref: Weak<FileManager>,
        intend_to_broadcast: bool,
    ) -> Arc<Self> {
        Arc::new(ChunkSender {
            total_chunks: total_size_to_total_chunks(file_info.total_size),
            full_chunk_range: create_full_chunk_range(file_info.total_size),
            file_info,
            broadcasting: AtomicBool::new(intend_to_broadcast), 
            path,
            file_requests: Mutex::new(VecDeque::new()),
            file_manager_ref,
        })
    }

    pub fn clean(&self) {
        let mut file_requests = self.file_requests.lock();
        file_requests.clear();
    }

    pub fn add_request(&self, requesting_node_id: String, mut file_request_data: FileRequestData) {
        let mut file_requests = self.file_requests.lock();

        let mut existing_file_request: Option<&mut FileRequest> = None;
        for i in 0..file_requests.len() {
            if file_requests[i].file_id == file_request_data.file_id {
                existing_file_request = Some(&mut file_requests[i]);
                break;
            }
        }

        if let Some(file_request) = existing_file_request {
            file_request_data
                .promised_chunks
                .map_promised(&mut file_request.promised_chunks);
        } else if !file_request_data.requested_chunks.is_empty() {
            file_request_data.requested_chunks.compact();

            let mut promised_chunks = HashMap::new();
            file_request_data
                .promised_chunks
                .map_promised(&mut promised_chunks);

            file_requests.push_back(FileRequest {
                file_id: file_request_data.file_id,
                requesting_node_id,
                requested_chunks: file_request_data.requested_chunks,
                promised_chunks,
                start_chunk_number: 0,
                next_chunk_number: 0,
                chunk_missing_range_number: 0,
                new: true,
                wrapped: false,
            });

            if file_requests.len() == 1 && !self.broadcasting.load(Ordering::SeqCst) {
                let file_id = file_requests[0].file_id.clone(); 
                if let Some(file_manager) = self.file_manager_ref.upgrade() {
                    std::thread::spawn(move || {
                        file_manager.chunk_sender_loop(file_id, false);
                    });
                }
            }
        }
    }

    pub fn broadcast_file(&self, file_id: String) {
        if let Some(file_manager) = self.file_manager_ref.upgrade() {
            std::thread::spawn(move || {
                file_manager.chunk_sender_loop(file_id, true);
            });
        }
    }

    pub fn retract_request(&self, file_id: &String) {
        let mut file_requests = self.file_requests.lock();

        for i in 0..file_requests.len() {
            if &file_requests[i].file_id == file_id {
                file_requests.remove(i);
                break;
            }
        }
    }
}
