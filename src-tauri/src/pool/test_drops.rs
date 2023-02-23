use super::{
    cache_manager::CacheManager, file_manager::FileManager, pool_conn::PoolConn, pool_net::PoolNet,
    pool_state::PoolState, sync_server_client::SyncServerClient,
};

impl Drop for PoolState {
    fn drop(&mut self) {
        log::warn!("POOL STATE DROPPED");
    }
}
impl Drop for PoolConn {
    fn drop(&mut self) {
        log::warn!("POOL CONN DROPPED");
    }
}
impl Drop for PoolNet {
    fn drop(&mut self) {
        log::warn!("POOL NET DROPPED");
    }
}
impl Drop for SyncServerClient {
    fn drop(&mut self) {
        log::warn!("SYNC SERVER CLIENT DROPPED");
    }
}
impl Drop for CacheManager {
    fn drop(&mut self) {
        log::warn!("CACHE MANAGER DROPPED");
    }
}
impl Drop for FileManager {
    fn drop(&mut self) {
        log::warn!("FILE MANAGER DROPPED");
    }
}
