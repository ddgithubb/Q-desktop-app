pub mod pool;
pub mod config;

pub mod store;
pub mod db;

pub mod poolpb {
    include!(concat!(env!("OUT_DIR"), "/pool.v1.rs"));
}

pub mod sspb {
    include!(concat!(env!("OUT_DIR"), "/sync_server.v1.rs"));
}

use db::messages_db::MessagesDB;
use lazy_static::lazy_static;

use crate::{store::store_manager::StoreManager, pool::pool_manager::PoolManager};

lazy_static! {
    pub static ref POOL_MANAGER: PoolManager = PoolManager::init();
    pub static ref STORE_MANAGER: StoreManager = StoreManager::init();
    pub static ref MESSAGES_DB: MessagesDB = MessagesDB::init();
}