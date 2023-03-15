use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use log::info;
use tokio::net::TcpStream;
use tokio::sync::Mutex as AsyncMutex;
use tokio_tungstenite::tungstenite::{self, Message as WSMessage};
use tokio_tungstenite::{connect_async as connect_ws_async, MaybeTlsStream, WebSocketStream};

use crate::config::{
    sync_server_connect_endpoint, HEARTBEAT_INTERVAL_SECONDS, HEARTBEAT_TIMEOUT_SECONDS,
};
use crate::events::{
    add_pool_node_event, add_pool_user_event, init_pool_event, remove_pool_node_event,
    remove_pool_user_event,
};
use crate::ipc::{IPCInitPool, IPCPoolNode};
use crate::sspb::ss_message::{
    self, AddNodeData, AddUserData, Data as SSMessageData, InitPoolData, Op as SSMessageOp,
    RemoveNodeData, RemoveUserData, SdpResponseData, SuccessResponseData,
};
use crate::sspb::SsMessage as SSMessage;
use crate::STORE_MANAGER;
use prost::Message as ProstMessage;

use super::pool_conn::PoolConn;
use super::pool_node_position::PoolNodePosition;
use super::pool_state::PoolState;

pub struct SyncServerClient {
    pool_state: Arc<PoolState>,
    pool_conn: Arc<PoolConn>,

    ws_write: AsyncMutex<Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WSMessage>>>,
    heartbeat_timeout: AtomicBool,
}

impl SyncServerClient {
    pub(super) fn init(pool_state: Arc<PoolState>, pool_conn: Arc<PoolConn>) -> Arc<Self> {
        let sync_server_client = Arc::new(SyncServerClient {
            pool_state,
            pool_conn,
            ws_write: AsyncMutex::new(None),
            heartbeat_timeout: AtomicBool::new(true),
        });

        sync_server_client.start_sync_server_client();

        let sync_server_client_clone = sync_server_client.clone();
        tokio::spawn(async move {
            sync_server_client_clone.report_node_loop().await;
        });

        sync_server_client
    }

    pub(super) async fn close(&self) {
        if !self.pool_state.close() {
            return;
        }
        info!("WS CLOSE");
        let mut ws = self.ws_write.lock().await;
        if let Some(ws) = &mut *ws {
            let _ = ws.send(WSMessage::Close(None)).await;
            let _ = ws.close().await;
        }
    }

    fn init_pool(&self, init_pool_data: InitPoolData) {
        let pool_info = match init_pool_data.pool_info {
            Some(pool_info) => pool_info,
            None => return,
        };

        log::info!("nodeID: {}", self.pool_state.node_id);
        log::info!("userID: {}", self.pool_state.user.user_id);

        STORE_MANAGER.update_pool(self.pool_state.pool_id.clone(), pool_info.clone());

        let init_nodes = {
            let mut init_nodes = Vec::with_capacity(init_pool_data.init_nodes.len());
            let mut active_nodes = self.pool_state.active_nodes.write();

            for add_node_data in init_pool_data.init_nodes {
                init_nodes.push(IPCPoolNode {
                    node_id: add_node_data.node_id.clone(),
                    user_id: add_node_data.user_id,
                });
                active_nodes.insert(add_node_data.node_id, add_node_data.path);
            }

            init_nodes
        };

        init_pool_event(IPCInitPool {
            node_id: self.pool_state.node_id.clone(),
            pool_info,
            init_nodes,
        });
    }

    fn add_node(&self, add_node_data: AddNodeData) {
        self.pool_state
            .update_active_node_path(&add_node_data.node_id, add_node_data.path);

        add_pool_node_event(
            &self.pool_state.pool_id,
            IPCPoolNode {
                node_id: add_node_data.node_id,
                user_id: add_node_data.user_id,
            },
        );
    }

    fn remove_node(&self, remove_node_data: RemoveNodeData) {
        self.pool_state
            .remove_node(&remove_node_data.node_id, remove_node_data.promoted_nodes);

        remove_pool_node_event(&self.pool_state.pool_id, remove_node_data.node_id);
    }

    fn add_user(&self, add_user_data: AddUserData) {
        let user_info = match add_user_data.user_info {
            Some(user_info) => user_info,
            None => return,
        };

        STORE_MANAGER.add_pool_user(&self.pool_state.pool_id, user_info.clone());

        add_pool_user_event(&self.pool_state.pool_id, user_info);
    }

    fn remove_user(&self, remove_user_data: RemoveUserData) {
        STORE_MANAGER.remove_pool_user(&self.pool_state.pool_id, &remove_user_data.user_id);

        remove_pool_user_event(&self.pool_state.pool_id, remove_user_data.user_id);
    }

    async fn handle_ws_http_error(self: &Arc<SyncServerClient>) {
        info!("WS ERROR");
        self.close().await;
    }

    pub(super) fn start_sync_server_client(self: &Arc<SyncServerClient>) {
        let self_clone = self.clone();
        tokio::spawn(async move {
            self_clone.sync_server_loop().await;
        });
    }

    async fn sync_server_loop(self: Arc<Self>) {
        let device_id = STORE_MANAGER.device_id();
        let url = url::Url::parse(
            sync_server_connect_endpoint(self.pool_state.pool_id.as_str(), device_id.clone())
                .as_str(),
        )
        .unwrap();

        let ws_conn = match connect_ws_async(url).await {
            Ok((ws_conn, _)) => ws_conn,
            Err(_) => {
                self.handle_ws_http_error().await;
                return;
            }
        };

        info!("WS OPEN");
        let (ws_write, mut ws_read) = ws_conn.split();
        {
            let mut lock = self.ws_write.lock().await;
            *lock = Some(ws_write);
        }

        let auth_token = STORE_MANAGER.auth_token();
        self.send_ws_conn(auth_token.into_bytes()).await;

        self.clone().start_heartbeat_interval();

        let mut auth_err = true;
        use tungstenite::Error;
        loop {
            match ws_read.next().await {
                Some(Ok(WSMessage::Binary(buf))) => {
                    if let Ok(ss_msg) = SSMessage::decode(&*buf) {
                        if ss_msg.op() == SSMessageOp::Close {
                            if auth_err {
                                log::warn!(
                                    "sync_server_loop : AUTH_ERR with token {}",
                                    STORE_MANAGER.auth_token()
                                );
                                self.pool_state.set_auth_error()
                            }
                            self.close().await;
                            continue;
                        }

                        if ss_msg.op() == SSMessageOp::Heartbeat {
                            auth_err = false;
                            self.heartbeat_timeout.store(false, Ordering::SeqCst);
                            continue;
                        }

                        // debug!("WS MESSAGE {:?}", ss_msg);

                        let ss_client_clone = self.clone();
                        tokio::spawn(async move {
                            ss_client_clone.handle_ss_message(ss_msg).await;
                        });
                    }
                }
                Some(Err(Error::Http(_) | Error::HttpFormat(_) | Error::Url(_))) => {
                    self.handle_ws_http_error().await;
                    break;
                }
                _ => {
                    self.close().await;
                    break;
                }
            };
        }
    }

    async fn handle_ss_message(&self, ss_msg: SSMessage) {
        let mut res_ss_msg: SSMessage = SSMessage {
            op: ss_msg.op,
            key: ss_msg.key,
            data: None,
        };
        if let Some(op) = ss_message::Op::from_i32(ss_msg.op) {
            match op {
                SSMessageOp::Close => {
                    unreachable!()
                }
                SSMessageOp::Heartbeat => {
                    unreachable!()
                }
                SSMessageOp::UpdateNodePosition => {
                    if let Some(SSMessageData::UpdateNodePositionData(update_node_position_data)) =
                        ss_msg.data
                    {
                        log::info!("New Node Position: {:?}", update_node_position_data);
                        let only_node = self.pool_state.set_node_position(
                            PoolNodePosition::from_update_node_position_data(
                                update_node_position_data,
                            ),
                        );

                        if only_node {
                            self.pool_conn.update_is_fully_connected()
                        }
                    }
                }
                SSMessageOp::ConnectNode => {
                    if let Some(SSMessageData::ConnectNodeData(connect_node_data)) = ss_msg.data {
                        let mut sdp_response_data = SdpResponseData::default();
                        if let Ok(sdp) = self
                            .pool_conn
                            .generate_offer(connect_node_data.node_id)
                            .await
                        {
                            sdp_response_data.success = true;
                            sdp_response_data.sdp = sdp;
                        }
                        res_ss_msg.set_op(SSMessageOp::SendOffer);
                        res_ss_msg.data = Some(SSMessageData::SdpResponseData(sdp_response_data));
                    }
                }
                SSMessageOp::DisconnectNode => {
                    if let Some(SSMessageData::DisconnectNodeData(disconnect_node_data)) =
                        ss_msg.data
                    {
                        self.pool_conn
                            .disconnect_node(disconnect_node_data.node_id)
                            .await;
                    }
                }
                SSMessageOp::ReportNode => {}
                SSMessageOp::SendOffer => {
                    if let Some(SSMessageData::SdpOfferData(sdp_offer_data)) = ss_msg.data {
                        let mut sdp_response_data = SdpResponseData::default();
                        if let Ok(sdp) = self
                            .pool_conn
                            .answer_offer(sdp_offer_data.from_node_id, sdp_offer_data.sdp)
                            .await
                        {
                            sdp_response_data.success = true;
                            sdp_response_data.sdp = sdp;
                        }
                        res_ss_msg.set_op(SSMessageOp::AnswerOffer);
                        res_ss_msg.data = Some(SSMessageData::SdpResponseData(sdp_response_data));
                    }
                }
                SSMessageOp::AnswerOffer => {
                    if let Some(SSMessageData::SdpOfferData(sdp_offer_data)) = ss_msg.data {
                        let mut success_response_data = SuccessResponseData::default();
                        if let Ok(()) = self
                            .pool_conn
                            .connect_node(sdp_offer_data.from_node_id, sdp_offer_data.sdp)
                            .await
                        {
                            success_response_data.success = true;
                        }
                        res_ss_msg.set_op(SSMessageOp::ConnectNode);
                        res_ss_msg.data =
                            Some(SSMessageData::SuccessResponseData(success_response_data));
                    }
                }
                SSMessageOp::VerifyNodeConnected => {
                    if let Some(SSMessageData::VerifyNodeConnectedData(
                        verify_node_connected_data,
                    )) = ss_msg.data
                    {
                        let mut success_response_data = SuccessResponseData::default();
                        success_response_data.success = self
                            .pool_conn
                            .clone()
                            .verify_connection(verify_node_connected_data.node_id)
                            .await;
                        res_ss_msg.data =
                            Some(SSMessageData::SuccessResponseData(success_response_data));
                    }
                }
                SSMessageOp::InitPool => {
                    if let Some(SSMessageData::InitPoolData(init_pool_data)) = ss_msg.data {
                        self.init_pool(init_pool_data);
                    }
                }
                SSMessageOp::AddNode => {
                    if let Some(SSMessageData::AddNodeData(add_node_data)) = ss_msg.data {
                        self.add_node(add_node_data);
                    }
                }
                SSMessageOp::RemoveNode => {
                    if let Some(SSMessageData::RemoveNodeData(remove_node_data)) = ss_msg.data {
                        self.remove_node(remove_node_data);
                    }
                }
                SSMessageOp::AddUser => {
                    if let Some(SSMessageData::AddUserData(add_user_data)) = ss_msg.data {
                        self.add_user(add_user_data);
                    }
                }
                SSMessageOp::RemoveUser => {
                    if let Some(SSMessageData::RemoveUserData(remove_user_data)) = ss_msg.data {
                        self.remove_user(remove_user_data);
                    }
                }
            }
            self.send_ws_message(res_ss_msg).await;
        }
    }

    fn start_heartbeat_interval(self: Arc<Self>) {
        let heartbeat_msg: SSMessage = SSMessage {
            op: SSMessageOp::Heartbeat.into(),
            key: String::from(""),
            data: None,
        };
        let heartbeat_buf: Vec<u8> = SyncServerClient::encode_ss_message(heartbeat_msg);
        tokio::spawn(async move {
            loop {
                self.heartbeat_timeout.store(true, Ordering::SeqCst);

                // debug!("SEND WS HEARTBEAT");
                if !self.send_ws_conn(heartbeat_buf.clone()).await {
                    break;
                }

                tokio::select! {
                    _ = self.pool_state.close_signal() => {
                        break;
                    },
                    _ = tokio::time::sleep(Duration::from_secs(HEARTBEAT_TIMEOUT_SECONDS)) => {},
                }

                if self.heartbeat_timeout.load(Ordering::SeqCst) {
                    self.close().await;
                    break;
                }

                tokio::select! {
                    _ = self.pool_state.close_signal() => {
                        break;
                    },
                    _ = tokio::time::sleep(Duration::from_secs(HEARTBEAT_INTERVAL_SECONDS - HEARTBEAT_TIMEOUT_SECONDS,)) => {},
                }
            }
        });
    }

    async fn report_node_loop(&self) {
        loop {
            tokio::select! {
                _ = self.pool_state.close_signal() => {
                    return;
                },
                Ok(report_node_data) = self.pool_conn.report_node_recv.recv_async() => {
                    self.send_ws_message(SSMessage {
                        op: SSMessageOp::ReportNode.into(),
                        key: String::from(""),
                        data: Some(SSMessageData::ReportNodeData(report_node_data))
                    }).await;
                },
            }
        }
    }

    async fn send_ws_message(&self, ss_msg: SSMessage) -> bool {
        self.send_ws_conn(SyncServerClient::encode_ss_message(ss_msg))
            .await
    }

    async fn send_ws_conn(&self, buf: Vec<u8>) -> bool {
        let mut ws = self.ws_write.lock().await;
        if let Some(ws) = &mut *ws {
            match ws.send(WSMessage::Binary(buf)).await {
                Ok(_) => return true,
                Err(_) => return false,
            }
        }
        return false;
    }

    fn encode_ss_message(ss_msg: SSMessage) -> Vec<u8> {
        ss_msg.encode_to_vec()
    }
}
