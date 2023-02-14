use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Weak,
    },
    time::{Duration, Instant},
};

use anyhow::anyhow;
use arc_swap::{ArcSwapOption};
use bytes::Bytes;
use flume::{Receiver, Sender};
use log::info;
use parking_lot::{RwLock};
use prost::Message;
use tokio::sync::{broadcast::Sender as BroadcastSender, Mutex as AsyncMutex};
use webrtc::{
    api::APIBuilder,
    data_channel::{
        data_channel_init::RTCDataChannelInit, data_channel_state::RTCDataChannelState,
        OnCloseHdlrFn, OnMessageHdlrFn, OnOpenHdlrFn, RTCDataChannel,
    },
    ice_transport::ice_server::RTCIceServer,
    peer_connection::{
        configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription, RTCPeerConnection,
    },
    sctp::stream::OnBufferedAmountLowFn,
};

use crate::{
    config::{
        BUFFERED_AMOUNT_LOW_THRESHOLD, DC_INIT_BUFFER_MAX_FILL_RATE_TIMEOUT,
        DC_INIT_BUFFER_MIN_FILL_RATE_TIMEOUT, DC_REFILL_RATE_CHUNK_AMOUNT,
        MAX_DC_BUFFER_CHUNK_AMOUNT, MAX_DC_BUFFER_SIZE,
    },
    poolpb::{PoolMessagePackage},
    sspb::ss_message,
};

use super::{
    message_util::{
        message_checks::MessageChecks, message_package_bundle::MessagePackageBundle,
    },
    pool_net::PoolNet,
    pool_node_position::PoolPanelNodeIDs,
    pool_state::PoolState,
};

struct InitChunksBuffer {
    buffer: VecDeque<Bytes>,
    buffer_rate_limiter: Option<Receiver<()>>, // receiver listens to drop
}

struct ChunksBuffer {
    signal_chunks_send_tx: BroadcastSender<()>,

    init_buffer: Arc<AsyncMutex<Option<InitChunksBuffer>>>,

    last_max_buffer_time: Arc<AtomicU64>,
}
struct PoolNodeConnection {
    // position: usize, // BUG: position not updated and also not needed?
    connection: Arc<RTCPeerConnection>,
    main_data_channel: Arc<RTCDataChannel>,
    chunks_data_channel: Arc<RTCDataChannel>,
    chunks_buffer: Arc<ChunksBuffer>,

    _closed_tx: Sender<()>,
    closed_rx: Receiver<()>,
}

pub struct PoolConn {
    pool_state: Arc<PoolState>,
    pub(super) pool_net_ref: ArcSwapOption<Weak<PoolNet>>,

    node_connections: RwLock<HashMap<String, PoolNodeConnection>>,
    min_time_to_send_deque_size: Arc<AtomicU64>,

    is_fully_connected: AtomicBool,

    report_node_chan: Sender<ss_message::ReportNodeData>,
    pub(super) report_node_recv: Receiver<ss_message::ReportNodeData>,
}

impl PoolConn {
    pub(super) fn init(pool_state: Arc<PoolState>) -> Arc<Self> {
        let (report_node_chan, report_node_recv) = flume::unbounded::<ss_message::ReportNodeData>();

        let pool_conn: Arc<PoolConn> = Arc::new(PoolConn {
            pool_state,
            pool_net_ref: ArcSwapOption::empty(),
            node_connections: RwLock::new(HashMap::with_capacity(12)), // MAX 6 connections, so double just in case
            min_time_to_send_deque_size: Arc::new(AtomicU64::new(0)),
            is_fully_connected: AtomicBool::new(false),
            report_node_chan,
            report_node_recv,
        });

        pool_conn
    }

    pub(super) async fn clean(&self) {
        let node_connections: Vec<(String, PoolNodeConnection)> = {
            let mut node_connections = self.node_connections.write();
            node_connections.drain().collect()
        };

        for (_, node_connection) in node_connections {
            self.close_node_connection(node_connection).await;
        }
    }

    pub(super) fn is_fully_connected(&self) -> bool {
        self.is_fully_connected.load(Ordering::Relaxed)
    }

    fn update_is_fully_connected(&self) {
        let node_connections = self.node_connections.read();
        for (_, c) in node_connections.iter() {
            if c.main_data_channel.ready_state() == RTCDataChannelState::Connecting {
                self.is_fully_connected.store(false, Ordering::Relaxed);
                return;
            }
        }
        self.is_fully_connected.store(true, Ordering::Relaxed);
    }

    pub(super) async fn generate_offer(
        self: &Arc<Self>,
        target_node_id: String,
    ) -> anyhow::Result<String> {
        let node_connection = self.create_connection(&target_node_id).await?;
        let connection = node_connection.connection.clone();
        let closed_rx = node_connection.closed_rx.clone();

        self.replace_node_connection(target_node_id, node_connection)
            .await;

        let desc = connection.create_offer(None).await?;
        connection.set_local_description(desc).await?;

        let mut gathering_complete_chan = connection.gathering_complete_promise().await;
        tokio::select! {
            _ = closed_rx.recv_async() => { return Err(anyhow::anyhow!("node connection closed")) },
            _ = gathering_complete_chan.recv() => {}
        }

        let local_desc = connection
            .local_description()
            .await
            .ok_or(anyhow::anyhow!("no local description"))?;
        let local_desc_str = serde_json::to_string(&local_desc)?;

        anyhow::Ok(local_desc_str)
    }

    pub(super) async fn answer_offer(
        self: &Arc<Self>,
        target_node_id: String,
        sdp: String,
    ) -> anyhow::Result<String> {
        let node_connection = self.create_connection(&target_node_id).await?;
        let connection = node_connection.connection.clone();
        let closed_rx = node_connection.closed_rx.clone();

        self.replace_node_connection(target_node_id, node_connection)
            .await;

        let offer = serde_json::from_str::<RTCSessionDescription>(&sdp)?;
        connection.set_remote_description(offer).await?;

        let desc = connection.create_answer(None).await?;
        connection.set_local_description(desc).await?;

        let mut gathering_complete_chan = connection.gathering_complete_promise().await;
        tokio::select! {
            _ = closed_rx.recv_async() => { return Err(anyhow::anyhow!("node connection closed")) },
            _ = gathering_complete_chan.recv() => {}
        }

        let local_desc = connection
            .local_description()
            .await
            .ok_or(anyhow::anyhow!("no local description"))?;
        let local_desc_str = serde_json::to_string(&local_desc)?;

        anyhow::Ok(local_desc_str)
    }

    pub(super) async fn connect_node(
        &self,
        target_node_id: String,
        sdp: String,
    ) -> anyhow::Result<()> {
        let (connection, closed_rx) = {
            let node_connections = self.node_connections.read();
            let node_connection = node_connections
                .get(&target_node_id)
                .ok_or(anyhow::anyhow!("no node connection found"))?;
            (
                node_connection.connection.clone(),
                node_connection.closed_rx.clone(),
            )
        };

        let (open_tx, open_rx) = flume::bounded::<()>(0);
        connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            let open_chan = open_tx.clone();
            Box::pin(async move {
                if s == RTCPeerConnectionState::Connected {
                    let _ = open_chan.send(());
                }
            })
        }));

        let answer = serde_json::from_str::<RTCSessionDescription>(&sdp)?;
        connection.set_remote_description(answer).await?;

        tokio::select! {
            _ = closed_rx.recv_async() => { return Err(anyhow::anyhow!("node connection closed")) },
            _ = open_rx.recv_async() => {}
        }

        anyhow::Ok(())
    }

    async fn replace_node_connection(
        &self,
        target_node_id: String,
        new_node_connection: PoolNodeConnection,
    ) {
        let existing_node_connection = {
            let mut node_connections = self.node_connections.write();
            node_connections.insert(target_node_id, new_node_connection)
        };

        if let Some(existing_node_connection) = existing_node_connection {
            self.close_node_connection(existing_node_connection).await;
        }
    }

    pub(super) async fn disconnect_node(&self, target_node_id: String) {
        let existing_node_connection = {
            let mut node_connections = self.node_connections.write();
            node_connections.remove(&target_node_id)
        };

        if let Some(existing_node_connection) = existing_node_connection {
            self.close_node_connection(existing_node_connection).await;
        }
    }

    pub(super) async fn verify_connection(&self, target_node_id: String) -> bool {
        let node_connections = self.node_connections.read();
        if let Some(node_connection) = node_connections.get(&target_node_id) {
            return node_connection.connection.connection_state()
                == RTCPeerConnectionState::Connected;
        }
        return false;
    }

    // Closes and consumes node_connection
    async fn close_node_connection(&self, node_connection: PoolNodeConnection) {
        let _ = node_connection.connection.close().await;
        // drop(node_connection._closed_tx); implied
    }

    pub(super) async fn distribute_message(
        &self,
        mut msg_pkg_bundle: MessagePackageBundle,
    ) {
        let src = msg_pkg_bundle.take_src();

        // let src_path = match self.pool_state.get_active_node_path(&src.node_id) {
        //     Some(src_path) => src_path,
        //     None => msg_pkg_bundle.get_src_path(),
        // };
        let dests = &msg_pkg_bundle.msg_pkg.dests;
        let has_dests = dests.len() != 0;

        let (partner_int_path, has_partner_int_path) = match msg_pkg_bundle.msg_pkg.partner_int_path
        {
            Some(partner_int_path) => (partner_int_path as usize, true),
            None => (0, false),
        };
        let from_node_id = &msg_pkg_bundle.from_node_id;

        let node_position = self.pool_state.node_position.load();
        let my_partner_int = node_position.partner_int;
        let my_panel_number = node_position.panel_number;

        let restrict_to_own_panel = has_partner_int_path
            && src.node_id != self.pool_state.node_id
            && partner_int_path != my_partner_int;

        if has_dests {
            for i in 0..3 {
                if i != my_partner_int {
                    if let Some(node_id) =
                        &node_position.parent_cluster_node_ids[my_panel_number][i]
                    {
                        if !has_partner_int_path
                            || (i == partner_int_path && node_id != from_node_id)
                        {
                            self.send_data_channel(node_id, &msg_pkg_bundle).await;
                            if has_partner_int_path {
                                return;
                            }
                            break;
                        }
                    }
                }
            }

            let mut parent_cluster_panel_switches = [false; 3];
            let mut child_cluster_panel_switches = [false; 2];

            // Prevents right to send to any other panel if it's not source node and the message does not correspond to the partnerInt
            'dest_loop: for dest in dests {
                // if dest.visited {
                //     continue;
                // }

                let dest_node_id = &dest.node_id;

                for i in 0..3 {
                    if restrict_to_own_panel && i != my_panel_number {
                        continue;
                    }

                    for j in 0..3 {
                        if let Some(node_id) = &node_position.parent_cluster_node_ids[i][j] {
                            if node_id != &self.pool_state.node_id && node_id == dest_node_id {
                                if i == my_panel_number && j != my_partner_int {
                                    // Sets boundaries to when it's allowed to send to its own panel
                                    if !has_partner_int_path
                                        || my_partner_int == partner_int_path
                                        || partner_int_path == j
                                        || node_position.parent_cluster_node_ids[my_panel_number]
                                            [partner_int_path]
                                            .is_none()
                                    {
                                        self.send_data_channel(node_id, &msg_pkg_bundle).await;
                                    }
                                } else {
                                    parent_cluster_panel_switches[i] = true;
                                }

                                continue 'dest_loop;
                            }
                        }
                    }
                }

                if restrict_to_own_panel {
                    continue;
                }

                let dest_path = match self.pool_state.active_node_path(dest_node_id) {
                    Some(dest_path) => dest_path,
                    None => continue,
                };

                let mut matches = 0;
                if node_position.path.len() <= dest_path.len() {
                    for i in 0..node_position.path.len() {
                        if node_position.path[i] == dest_path[i] {
                            matches += 1;
                        } else {
                            matches = 0;
                            break;
                        }
                    }
                }

                if matches == 0 {
                    if node_position.center_cluster {
                        parent_cluster_panel_switches[dest_path[0] as usize] = true;
                    } else {
                        parent_cluster_panel_switches[2] = true;
                    }
                } else {
                    if matches >= dest_path.len() {
                        continue;
                    }
                    child_cluster_panel_switches[dest_path[matches] as usize] = true;
                }
            }

            if restrict_to_own_panel {
                return;
            }

            let (send_to_parent, send_to_child) =
                self.direction_of_message(&node_position.path, &src.path);

            if send_to_parent {
                for i in 0..3 {
                    if i != my_panel_number && parent_cluster_panel_switches[i] {
                        self.send_to_panel(
                            &node_position.parent_cluster_node_ids[i],
                            &msg_pkg_bundle,
                        )
                        .await;
                    }
                }
            }

            if send_to_child {
                for i in 0..2 {
                    if child_cluster_panel_switches[i] {
                        self.send_to_panel(
                            &node_position.child_cluster_node_ids[i],
                            &msg_pkg_bundle,
                        )
                        .await;
                    }
                }
            }
        } else {
            for i in 0..3 {
                if i != my_partner_int {
                    if let Some(node_id) =
                        &node_position.parent_cluster_node_ids[my_panel_number][i]
                    {
                        if node_id != from_node_id
                            && (!has_partner_int_path
                                || my_partner_int == partner_int_path
                                || partner_int_path == i
                                || node_position.parent_cluster_node_ids[my_panel_number]
                                    [partner_int_path]
                                    .is_none())
                        {
                            self.send_data_channel(node_id, &msg_pkg_bundle).await;
                        }
                    }
                }
            }

            if restrict_to_own_panel {
                return;
            }

            let (send_to_parent, send_to_child) =
                self.direction_of_message(&node_position.path, &src.path);

            if send_to_parent {
                for i in 0..3 {
                    if i != my_panel_number {
                        self.send_to_panel(
                            &node_position.parent_cluster_node_ids[i],
                            &msg_pkg_bundle,
                        )
                        .await;
                    }
                }
            }

            if send_to_child {
                for i in 0..2 {
                    self.send_to_panel(&node_position.child_cluster_node_ids[i], &msg_pkg_bundle)
                        .await;
                }
            }
        }
    }

    async fn send_to_panel(
        &self,
        panel: &PoolPanelNodeIDs,
        msg_pkg_bundle: &MessagePackageBundle,
    ) {
        let mut has_partner_int_path = false;
        if let Some(partner_int_path) = msg_pkg_bundle.msg_pkg.partner_int_path {
            if let Some(node_id) = &panel[partner_int_path as usize] {
                if self.send_data_channel(node_id, msg_pkg_bundle).await {
                    return;
                }
            }
            has_partner_int_path = true;
        }

        for opt_node_id in panel {
            if let Some(node_id) = &opt_node_id {
                if self.send_data_channel(node_id, msg_pkg_bundle).await && has_partner_int_path {
                    return;
                }
            }
        }
    }

    pub(super) async fn send_data_channel(
        &self,
        node_id: &String,
        msg_pkg_bundle: &MessagePackageBundle,
    ) -> bool {
        if node_id.is_empty() {
            return false;
        }
        if msg_pkg_bundle.has_chunks() {
            let (chunks_dc, chunks_buffer, closed_rx) = {
                let node_connections = self.node_connections.read();
                match node_connections.get(node_id) {
                    Some(nc) => (
                        nc.chunks_data_channel.clone(),
                        nc.chunks_buffer.clone(),
                        nc.closed_rx.clone(),
                    ),
                    None => return false,
                }
            };

            if chunks_dc.ready_state() == RTCDataChannelState::Open {
                if chunks_dc.buffered_amount().await >= MAX_DC_BUFFER_SIZE {
                    let mut signal_chunks_send_rx = chunks_buffer.signal_chunks_send_tx.subscribe();
                    chunks_buffer.last_max_buffer_time.store(
                        Instant::now()
                            .duration_since(self.pool_state.instant_seed)
                            .as_millis() as u64,
                        Ordering::SeqCst,
                    );
                    tokio::select! {
                        _ = closed_rx.recv_async() => { return false },
                        _ = signal_chunks_send_rx.recv() => {},
                    };
                }

                let _ = chunks_dc.send(&msg_pkg_bundle.encoded_msg_pkg).await;
                return true;
            } else if chunks_dc.ready_state() == RTCDataChannelState::Connecting {
                let buffer_rate_limiter = {
                    let mut init_buffer_lock = chunks_buffer.init_buffer.lock().await;
                    let init_buffer = match &mut *init_buffer_lock {
                        Some(init_buffer) => init_buffer,
                        None => {
                            let _ = chunks_dc.send(&msg_pkg_bundle.encoded_msg_pkg).await;
                            return true;
                        }
                    };

                    if let Some(buffer_rate_limiter) = &init_buffer.buffer_rate_limiter {
                        buffer_rate_limiter.clone()
                    } else {
                        if init_buffer.buffer.len() % DC_REFILL_RATE_CHUNK_AMOUNT == 0 {
                            let (buffer_rate_limiter_sender, buffer_rate_limiter) =
                                flume::bounded::<()>(0);
                            let min_time_to_send_deque_size =
                                self.min_time_to_send_deque_size.clone();

                            tokio::spawn(async move {
                                tokio::time::sleep(Duration::from_millis(std::cmp::min(
                                    std::cmp::max(
                                        min_time_to_send_deque_size.load(Ordering::Relaxed),
                                        DC_INIT_BUFFER_MIN_FILL_RATE_TIMEOUT,
                                    ),
                                    DC_INIT_BUFFER_MAX_FILL_RATE_TIMEOUT,
                                )))
                                .await;
                                let _ = buffer_rate_limiter_sender.try_send(());
                            });

                            init_buffer.buffer_rate_limiter = Some(buffer_rate_limiter.clone());
                            buffer_rate_limiter
                        } else {
                            init_buffer
                                .buffer
                                .push_back(msg_pkg_bundle.encoded_msg_pkg.slice(..));
                            return true;
                        }
                    }
                };

                let mut signal_chunks_send_rx = chunks_buffer.signal_chunks_send_tx.subscribe();

                tokio::select! {
                    _ = signal_chunks_send_rx.recv() => {
                        let _ = chunks_dc.send(&msg_pkg_bundle.encoded_msg_pkg).await;
                        return true;
                    },
                    _ = buffer_rate_limiter.recv_async() => {}
                };

                let mut init_buffer_lock = chunks_buffer.init_buffer.lock().await;
                let init_buffer = match &mut *init_buffer_lock {
                    Some(init_buffer) => init_buffer,
                    None => {
                        let _ = chunks_dc.send(&msg_pkg_bundle.encoded_msg_pkg).await;
                        return true;
                    }
                };

                if init_buffer.buffer.len() == MAX_DC_BUFFER_CHUNK_AMOUNT {
                    let _ = init_buffer.buffer.drain(..DC_REFILL_RATE_CHUNK_AMOUNT);
                }

                init_buffer
                    .buffer
                    .push_back(msg_pkg_bundle.encoded_msg_pkg.slice(..));
                init_buffer.buffer_rate_limiter = None;

                return true;
            }
        } else {
            let main_dc = {
                let node_connections = self.node_connections.read();
                match node_connections.get(node_id) {
                    Some(nc) if nc.main_data_channel.ready_state() == RTCDataChannelState::Open => {
                        Some(nc.main_data_channel.clone())
                    }
                    _ => None,
                }
            };

            if let Some(main_dc) = main_dc {
                let _ = main_dc.send(&msg_pkg_bundle.encoded_msg_pkg).await;
                return true;
            }
        }
        return false;
    }

    fn main_dc_on_open(self_clone: Weak<Self>, node_id: String) -> OnOpenHdlrFn {
        Box::new(move || {
            Box::pin(async move {
                info!("DC MAIN OPEN FOR {}", node_id);
                if let Some(self_clone) = self_clone.upgrade() {
                    self_clone.update_is_fully_connected();
                    if let Some(pool_net) = (&*self_clone.pool_net_ref.load()).as_ref().unwrap().upgrade() {
                        if !self_clone.pool_state.is_latest() {
                            pool_net.send_latest_request(&node_id).await;
                            pool_net.send_node_info_data().await;
                        } else {
                            pool_net.send_missed_messages().await;
                        }
                    }
                }
            })
        })
    }

    fn main_dc_on_message(&self, pool_net: Arc<Weak<PoolNet>>, node_id: String) -> OnMessageHdlrFn {
        let self_node_id = self.pool_state.node_id.clone();
        Box::new(move |dc_msg| {
            let self_node_id = self_node_id.clone();
            let node_id = node_id.clone();
            let pool_net = pool_net.clone();
            Box::pin(async move {
                if let Some(pool_net) = pool_net.upgrade() {
                    if dc_msg.data.len() == 0 || dc_msg.is_string {
                        // invalid message
                        return;
                    }
                    if let Ok(msg_pkg) = PoolMessagePackage::decode(dc_msg.data.slice(..)) {
                        if let Some(src) = &msg_pkg.src {
                            if src.node_id == self_node_id {
                                return;
                            }
                        }

                        if msg_pkg.msg.is_some() {
                            if !msg_pkg.is_valid_message() {
                                return;
                            }

                            pool_net
                                .handle_message(
                                    MessagePackageBundle {
                                        msg_pkg: msg_pkg,
                                        encoded_msg_pkg: dc_msg.data,
                                        from_node_id: node_id,
                                    },
                                )
                                .await;
                        } else if msg_pkg.direct_msg.is_some() {
                            if !msg_pkg.is_valid_direct_message() {
                                return;
                            }

                            pool_net
                                .handle_direct_message(
                                    MessagePackageBundle {
                                        msg_pkg: msg_pkg,
                                        encoded_msg_pkg: dc_msg.data,
                                        from_node_id: node_id,
                                    },
                                )
                                .await;
                        }
                        
                    }
                }
            })
        })
    }

    fn main_dc_on_close(self_clone: Weak<Self>, node_id: String) -> OnCloseHdlrFn {
        Box::new(move || {
            let self_clone = self_clone.clone();
            let node_id = node_id.clone();
            Box::pin(async move {
                if let Some(self_clone) = self_clone.upgrade() {
                    info!("DC MAIN CLOSED FOR {}", node_id);
                    self_clone.update_is_fully_connected();
                    let _ = self_clone
                        .report_node_chan
                        .send_async(ss_message::ReportNodeData {
                            node_id: node_id,
                            report_code: ss_message::ReportCode::DisconnectReport.into(),
                        })
                        .await;
                }
            })
        })
    }

    fn chunks_dc_on_open(
        &self,
        chunks_dc: Arc<RTCDataChannel>,
        chunks_buffer: Arc<ChunksBuffer>,
    ) -> OnOpenHdlrFn {
        Box::new(move || {
            Box::pin(async move {
                let mut init_buffer = chunks_buffer.init_buffer.lock().await;
                if let Some(mut init_buffer) = init_buffer.take() {
                    loop {
                        if let Some(chunk) = init_buffer.buffer.pop_front() {
                            let _ = chunks_dc.send(&chunk).await;
                            continue;
                        }
                        break;
                    }
                }
                chunks_buffer
                    .last_max_buffer_time
                    .store(0, Ordering::SeqCst); // to prevent inaccurate diff measures
                let _ = chunks_buffer.signal_chunks_send_tx.send(());
            })
        })
    }

    fn chunks_dc_on_message(
        &self,
        pool_net: Arc<Weak<PoolNet>>,
        node_id: String,
    ) -> OnMessageHdlrFn {
        let self_node_id = self.pool_state.node_id.clone();
        Box::new(move |dc_msg| {
            let self_node_id = self_node_id.clone();
            let node_id = node_id.clone();
            let pool_net = pool_net.clone();
            Box::pin(async move {
                if let Some(pool_net) = pool_net.upgrade() {
                    if dc_msg.data.len() == 0 || dc_msg.is_string {
                        // invalid message
                        return;
                    }
                    if let Ok(msg_pkg) = PoolMessagePackage::decode(dc_msg.data.slice(..)) {
                        if let Some(src) = &msg_pkg.src {
                            if src.node_id == self_node_id {
                                return;
                            }
                        }

                        if !msg_pkg.is_valid_chunk() {
                            return;
                        }

                        pool_net
                            .handle_chunk(
                                MessagePackageBundle {
                                    msg_pkg: msg_pkg,
                                    encoded_msg_pkg: dc_msg.data,
                                    from_node_id: node_id,
                                },
                            )
                            .await;
                    }
                }
            })
        })
    }

    fn chunks_dc_on_buffered_amount_low(
        &self,
        signal_chunks_send_tx: BroadcastSender<()>,
        last_max_buffer_time: Arc<AtomicU64>,
    ) -> OnBufferedAmountLowFn {
        let instant_seed = self.pool_state.instant_seed;
        let min_time_to_send_deque_size = self.min_time_to_send_deque_size.clone();

        Box::new(move || {
            let signal_chunks_send_tx = signal_chunks_send_tx.clone();
            let last_max_buffer_time = last_max_buffer_time.clone();
            let min_time_to_send_deque_size = min_time_to_send_deque_size.clone();

            Box::pin(async move {
                let last_time = last_max_buffer_time.load(Ordering::SeqCst);

                if last_time > 0 {
                    let diff_time =
                        Instant::now().duration_since(instant_seed).as_millis() as u64 - last_time;
                    let min_time = min_time_to_send_deque_size.load(Ordering::Relaxed);

                    if diff_time < min_time || min_time == 0 {
                        min_time_to_send_deque_size.store(diff_time, Ordering::Relaxed);
                    }
                }

                let _ = signal_chunks_send_tx.send(());
            })
        })
    }

    async fn create_connection(
        self: &Arc<Self>,
        node_id: &String,
    ) -> anyhow::Result<PoolNodeConnection> {
        let pool_net = match self.pool_net_ref.load_full() {
            Some(pool_net) => pool_net,
            None => return Err(anyhow!("pool_net doesn't exist")),
        };

        let pc = Arc::new(Self::create_peer_connection().await?);

        let main_dc_options = Some(RTCDataChannelInit {
            ordered: Some(false),
            negotiated: Some(0),
            ..Default::default()
        });

        let chunks_dc_options = Some(RTCDataChannelInit {
            ordered: Some(false),
            negotiated: Some(1),
            ..Default::default()
        });

        let main_dc = pc.create_data_channel("main", main_dc_options).await?;
        let chunks_dc = pc.create_data_channel("chunks", chunks_dc_options).await?;

        // let position = self
        //     .pool_state
        //     .get_position_of_node(node_id)
        //     .ok_or(anyhow::anyhow!("node doesn't exist"))?;

        let (_closed_tx, closed_rx) = flume::bounded::<()>(0);

        let (signal_chunks_send_tx, _) = tokio::sync::broadcast::channel::<()>(1);

        let last_max_buffer_time = Arc::new(AtomicU64::new(0));

        let chunks_buffer = Arc::new(ChunksBuffer {
            signal_chunks_send_tx,
            init_buffer: Arc::new(AsyncMutex::new(Some(InitChunksBuffer {
                buffer: VecDeque::with_capacity(MAX_DC_BUFFER_CHUNK_AMOUNT),
                buffer_rate_limiter: None,
            }))),
            last_max_buffer_time: last_max_buffer_time.clone(),
        });

        let node_connection = PoolNodeConnection {
            // position: position,
            connection: pc,
            main_data_channel: main_dc,
            chunks_data_channel: chunks_dc,
            chunks_buffer: chunks_buffer,
            _closed_tx,
            closed_rx,
        };

        // Set main DC functions

        node_connection
            .main_data_channel
            .on_open(Self::main_dc_on_open(
                Arc::downgrade(self),
                node_id.clone(),
            ));

        node_connection
            .main_data_channel
            .on_message(Self::main_dc_on_message(
                self,
                pool_net.clone(),
                node_id.clone(),
            ));

        node_connection
            .main_data_channel
            .on_close(Self::main_dc_on_close(
                Arc::downgrade(self),
                node_id.clone(),
            ));

        // Set chunks DC functions

        node_connection
            .chunks_data_channel
            .on_open(Self::chunks_dc_on_open(
                self,
                node_connection.chunks_data_channel.clone(),
                node_connection.chunks_buffer.clone(),
            ));

        node_connection
            .chunks_data_channel
            .on_message(Self::chunks_dc_on_message(
                self,
                pool_net.clone(),
                node_id.clone(),
            ));

        node_connection
            .chunks_data_channel
            .set_buffered_amount_low_threshold(BUFFERED_AMOUNT_LOW_THRESHOLD)
            .await;

        node_connection
            .chunks_data_channel
            .on_buffered_amount_low(Self::chunks_dc_on_buffered_amount_low(
                self,
                node_connection.chunks_buffer.signal_chunks_send_tx.clone(),
                last_max_buffer_time,
            ))
            .await;

        anyhow::Ok(node_connection)
    }

    async fn create_peer_connection() -> Result<RTCPeerConnection, webrtc::Error> {
        let api = APIBuilder::new().build();

        let ice_servers = vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_string()],
            ..Default::default()
        }];

        let config = RTCConfiguration {
            ice_servers,
            ..Default::default()
        };

        api.new_peer_connection(config).await
    }

    fn direction_of_message(&self, my_path: &Vec<u32>, src_path: &Vec<u32>) -> (bool, bool) {
        let mut send_to_parent = false;
        let mut send_to_child = false;

        if my_path.len() < src_path.len() {
            for i in 0..my_path.len() {
                if my_path[i] != src_path[i] {
                    send_to_parent = false;
                    send_to_child = true;
                    break;
                } else {
                    send_to_parent = true;
                    send_to_child = false;
                }
            }
        } else if my_path.len() == src_path.len() {
            let mut send = true;
            for i in 0..my_path.len() {
                if my_path[i] != src_path[i] {
                    send = false;
                    break;
                }
            }
            send_to_parent = send;
            send_to_child = send;
        }

        (send_to_parent, send_to_child)
    }
}
