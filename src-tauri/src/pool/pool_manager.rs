use std::{collections::HashMap, path::PathBuf, sync::Arc};

use log::info;
use tokio::sync::RwLock as AsyncRwLock;

use crate::{events::reconnect_pool_event, poolpb::PoolFileInfo};

use super::{
    pool_conn::PoolConn, pool_net::PoolNet, pool_state::PoolState,
    sync_server_client::SyncServerClient,
};

struct Pool {
    pool_state: Arc<PoolState>,
    pool_conn: Arc<PoolConn>,
    pool_net: Arc<PoolNet>,
    sync_server_client: Arc<SyncServerClient>,
}

impl Pool {
    pub(self) fn init(pool_id: String) -> Self {
        let pool_state = Arc::new(PoolState::init(pool_id));
        let pool_conn = PoolConn::init(pool_state.clone());
        let pool_net = PoolNet::init(pool_state.clone(), pool_conn.clone());
        let sync_server_client = SyncServerClient::init(pool_state.clone(), pool_conn.clone());

        pool_conn
            .pool_net_ref
            .store(Some(pool_net.clone()));

        Pool {
            pool_state,
            pool_conn,
            pool_net,
            sync_server_client,
        }
    }

    pub(self) fn clean(self) {
        tokio::spawn(async move {
            self.pool_state.set_disconnect();
            self.sync_server_client.close().await;
            self.pool_net.clean().await;
            self.pool_conn.clean().await;
        });
    }
}

pub struct PoolManager {
    active_pools: AsyncRwLock<HashMap<String, Pool>>,
}

impl PoolManager {
    pub fn init() -> Self {
        info!("Initializing Pool Manager...");

        PoolManager {
            active_pools: AsyncRwLock::new(HashMap::new()),
        }
    }

    pub async fn connect_to_pool(&self, pool_id: String) {
        let pool: Pool = Pool::init(pool_id.clone());

        let mut active_pools = self.active_pools.write().await;
        if let Some(existing_pool) = active_pools.insert(pool_id, pool) {
            reconnect_pool_event(&existing_pool.pool_state.pool_id);
            existing_pool.clean();
        }
    }

    pub async fn disconnect_from_pool(&self, pool_id: String) {
        let mut active_pools = self.active_pools.write().await;
        if let Some(pool) = active_pools.remove(&pool_id) {
            pool.clean();
        }
    }

    pub async fn send_text_message(&self, pool_id: &String, text: String) {
        let active_pools = self.active_pools.read().await;
        if let Some(pool) = active_pools.get(pool_id) {
            pool.pool_net.send_text_message(text).await;
        }
    }

    pub async fn add_file_offer(&self, pool_id: &String, file_path: String) {
        let active_pools = self.active_pools.read().await;
        if let Some(pool) = active_pools.get(pool_id) {
            let file_path = PathBuf::from(file_path);

            if let Some(file_offer) =
                PoolNet::generate_file_offer(file_path.clone(), pool.pool_state.node_id.clone())
            {
                pool.pool_net.send_file_offer(file_offer, file_path).await;
            }
        }
    }

    pub async fn add_image_offer(&self, pool_id: &String, file_path: String) {
        let active_pools = self.active_pools.read().await;
        if let Some(pool) = active_pools.get(pool_id) {
            let path = PathBuf::from(file_path);

            if let Some(file_offer) =
                PoolNet::generate_file_offer(path.clone(), pool.pool_state.node_id.clone())
            {
                pool.pool_net.send_image_offer(file_offer, path).await;
            }
        }
    }

    pub async fn download_file(&self, pool_id: &String, file_info: PoolFileInfo, dir_path: String) {
        let active_pools = self.active_pools.read().await;
        if let Some(pool) = active_pools.get(pool_id) {
            let dir_path = PathBuf::from(dir_path);

            if let Ok(metadata) = dir_path.metadata() {
                if !metadata.is_dir() {
                    return;
                }

                return pool.pool_net.download_file(file_info, dir_path).await;
            }
        }
    }

    pub async fn retract_file_offer(&self, pool_id: &String, file_id: String) {
        let active_pools = self.active_pools.read().await;
        if let Some(pool) = active_pools.get(pool_id) {
            pool.pool_net.send_retract_file_offer(file_id).await;
        }
    }

    pub async fn remove_file_download(&self, pool_id: &String, file_id: String) {
        let active_pools = self.active_pools.read().await;
        if let Some(pool) = active_pools.get(pool_id) {
            pool.pool_net.send_retract_file_request(file_id).await;
        }
    }
}
