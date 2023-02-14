use std::{collections::HashMap, sync::Arc};

use log::info;
use parking_lot::RwLock;

use super::{
    pool_net::PoolNet, pool_state::PoolState,
    sync_server_client::{SyncServerClient}, pool_conn::PoolConn,
};

struct Pool {
    pool_state: Arc<PoolState>,
    pool_conn: Arc<PoolConn>,
    pool_net: Arc<PoolNet>,
    sync_server_client: Arc<SyncServerClient>,
}

impl Pool {
    pub(self) fn init(pool_id: String) -> Self {
        let pool_state = Arc::new(PoolState::new(pool_id));
        let pool_conn = PoolConn::init(pool_state.clone());
        let pool_net = PoolNet::init(pool_state.clone(), pool_conn.clone());
        let sync_server_client = SyncServerClient::init(pool_state.clone(), pool_conn.clone());

        pool_conn.pool_net_ref.store(Some(Arc::new(Arc::downgrade(&pool_net.clone()))));

        Pool {
            pool_state,
            pool_conn,
            pool_net,
            sync_server_client,
        }
    }

    pub(self) fn clean(self) {
        tokio::spawn(async move {
            self.pool_net.clean().await;
            self.pool_conn.clean().await;
        });
    }
}

pub struct PoolManager {
    active_pools: RwLock<HashMap<String, Pool>>,
}

impl PoolManager {
    pub fn init() -> Self {
        info!("Initializing Pool Manager...");

        PoolManager { active_pools: RwLock::new(HashMap::new()) }
    }

    pub fn connect_to_pool(self: Arc<PoolManager>, pool_id: String) {
        let pool: Pool = Pool::init(pool_id.clone());

        self.clone()
            .set_pool_close_reconnect_handler(pool_id.clone(), pool.pool_state.clone());

        {
            let mut active_pools = self.active_pools.write();
            let _ = active_pools.insert(pool_id, pool);
        }
    }

    pub async fn disconnect_from_pool(&self, pool_id: String) {
        let mut active_pools = self.active_pools.write();
        if let Some(pool) = active_pools.remove(&pool_id) {
            pool.pool_state.set_disconnect();
            pool.sync_server_client.close().await;
        }
    }

    fn set_pool_close_reconnect_handler(
        self: Arc<PoolManager>,
        pool_id: String,
        pool_state: Arc<PoolState>,
    ) {
        tokio::spawn(async move {
            pool_state.close_signal().await;
            {
                let mut active_pools = self.active_pools.write();
                let pool = active_pools.remove(&pool_id);
                if let Some(pool) = pool {
                    pool.clean();
                }
            }
            if pool_state.reconnect() {
                self.connect_to_pool(pool_id);
            }
        });
    }
}
