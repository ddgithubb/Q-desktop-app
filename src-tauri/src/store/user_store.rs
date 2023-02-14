use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::sspb::{PoolDeviceInfo, PoolInfo, PoolUserInfo};

use super::store_manager::StoreManager;

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStore {
    pub(super) user_id: String,
    pub(super) device: PoolDeviceInfo,
    pub(super) pools: HashMap<String, PoolInfo>, // pool_id -> pool_info
}

pub struct BasicUserInfo {
    pub user_id: String,
    pub device: PoolDeviceInfo,
}

impl StoreManager {
    pub fn init_profile(&self, user_id: String, device: PoolDeviceInfo) {
        let mut user_store = self.user_store.lock();
        user_store.user_id = user_id;
        user_store.device = device;
        user_store.update();
    }

    pub fn init_pool(&self, pool_id: String, pool_info: PoolInfo) {
        let mut user_store = self.user_store.lock();
        user_store.pools.insert(pool_id, pool_info);
        user_store.update();
    }

    pub fn update_pool_user(&self, pool_id: &String, user_info: PoolUserInfo) {
        let mut user_store = self.user_store.lock();
        if let Some(pool) = user_store.pools.get_mut(pool_id) {
            for user in pool.users.iter_mut() {
                if user.user_id == user_info.user_id {
                    *user = user_info;
                    return;
                }
            }

            pool.users.push(user_info);
            user_store.update();
        }
    }

    pub fn remove_pool_user(&self, pool_id: &String, user_id: &String) {
        let mut user_store = self.user_store.lock();
        if let Some(pool) = user_store.pools.get_mut(pool_id) {
            for i in 0..pool.users.len() {
                if &pool.users[i].user_id == user_id {
                    pool.users.remove(i);
                    user_store.update();
                    return;
                }
            }
        }
    }

    pub fn user_info(&self) -> BasicUserInfo {
        let user_store = self.user_store.lock();
        BasicUserInfo {
            user_id: user_store.user_id.clone(),
            device: user_store.device.clone(),
        }
    }

    pub fn _device_name(&self) -> String {
        let user_store = self.user_store.lock();
        user_store.device.device_name.clone()
    }
}