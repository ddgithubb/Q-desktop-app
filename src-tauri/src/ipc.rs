use serde::Serialize;

use crate::{
    poolpb::{PoolFileInfo, PoolFileSeeders, PoolMessage},
    sspb::{PoolDeviceInfo, PoolInfo, PoolUserInfo},
};

#[derive(Clone, Serialize)]
pub struct IPCFileDownloadProgress {
    pub file_id: String,
    pub progress: usize,
}

#[derive(Default, Clone, Serialize)]
pub struct IPCStateUpdate {
    pub file_downloads_progress: Vec<IPCFileDownloadProgress>,
}

#[derive(Clone, Serialize)]
pub struct IPCInitProfile {
    pub user_info: PoolUserInfo,
    pub device: PoolDeviceInfo,
}

#[derive(Clone, Serialize)]
pub struct IPCPoolNode {
    pub node_id: String,
    pub user_id: String,
}

#[derive(Clone, Serialize)]
pub struct IPCInitPool {
    pub pool_info: PoolInfo,
    pub init_nodes: Vec<IPCPoolNode>,
}

#[derive(Clone, Serialize)]
pub struct IPCReconnectPool {
    pub pool_id: String,
}

#[derive(Clone, Serialize)]
pub struct IPCAddPoolNode {
    pub pool_id: String,
    pub node: IPCPoolNode,
}

#[derive(Clone, Serialize)]
pub struct IPCRemovePoolNode {
    pub pool_id: String,
    pub node_id: String,
}

#[derive(Clone, Serialize)]
pub struct IPCUpdatePoolUser {
    pub pool_id: String,
    pub user_info: PoolUserInfo,
}

#[derive(Clone, Serialize)]
pub struct IPCRemovePoolUser {
    pub pool_id: String,
    pub user_id: String,
}

#[derive(Clone, Serialize)]
pub struct IPCAddPoolFileOffers {
    pub pool_id: String,
    pub node_id: String,
    pub file_offers: Vec<PoolFileInfo>,
}

#[derive(Clone, Serialize)]
pub struct IPCRemovePoolFileOffer {
    pub pool_id: String,
    pub node_id: String,
    pub file_id: String,
}

#[derive(Clone, Serialize)]
pub struct IPCInitPoolFileSeeders {
    pub pool_id: String,
    pub file_seeders: Vec<PoolFileSeeders>,
}

#[derive(Clone, Serialize)]
pub struct IPCAddPoolFileDownload {
    pub pool_id: String,
    pub file_info: PoolFileInfo,
}

#[derive(Clone, Serialize)]
pub struct IPCCompletePoolFileDownload {
    pub pool_id: String,
    pub file_id: String,
    pub success: bool,
}

#[derive(Clone, Serialize)]
pub struct IPCInitPoolMessages {
    pub pool_id: String,
    pub messages: Vec<PoolMessage>,
}

#[derive(Clone, Serialize)]
pub struct IPCAppendPoolMessage {
    pub pool_id: String,
    pub message: PoolMessage,
}