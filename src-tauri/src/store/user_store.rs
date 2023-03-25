use std::{collections::HashMap, time::{UNIX_EPOCH, SystemTime}};

use serde::{Deserialize, Serialize};

use crate::sspb::{PoolDeviceInfo, PoolInfo, PoolUserInfo};

use super::store_manager::StoreManager;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct PoolData {
    pub pool_info: PoolInfo,
    pub last_modified: u64,
    // Additional data such as file offers, etc.
}

pub struct BasicUserInfo {
    pub user_id: String,
    pub display_name: String,
    pub device: PoolDeviceInfo,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStore {
    pub(super) registered: bool,
    pub(super) user_info: PoolUserInfo,
    pub(super) device: PoolDeviceInfo,
    pub(super) pools: HashMap<String, PoolData>, // pool_id -> pool_data
}

impl UserStore {
    pub fn sorted_pools(&self) -> Vec<PoolInfo> {
        let mut pools: Vec<PoolData> = self.pools.values().cloned().collect();
        pools.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        return pools.into_iter().map(|pool| pool.pool_info).collect()
    }
}

impl StoreManager {
    pub fn new_profile(&self, user_info: PoolUserInfo, device: PoolDeviceInfo) {
        let mut user_store = self.user_store.lock();
        user_store.registered = true;
        user_store.user_info = user_info;
        user_store.device = device;
        user_store.update();
    }

    pub fn is_registered(&self) -> bool {
        let user_store = self.user_store.lock();
        user_store.registered
    }

    pub fn update_pool(&self, pool_info: PoolInfo) {
        let mut user_store = self.user_store.lock();
        user_store.pools.insert(pool_info.pool_id.clone(), PoolData {
            pool_info,
            last_modified: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        });
        user_store.update();
    }

    pub fn remove_pool(&self, pool_id: &String) {
        let mut user_store = self.user_store.lock();
        user_store.pools.remove(pool_id);
        user_store.update();
    }

    pub fn add_pool_user(&self, pool_id: &String, user_info: PoolUserInfo) {
        let mut user_store = self.user_store.lock();
        if let Some(pool) = user_store.pools.get_mut(pool_id) {
            for user in pool.pool_info.users.iter_mut() {
                if user.user_id == user_info.user_id {
                    *user = user_info;
                    user_store.update();
                    return;
                }
            }

            pool.pool_info.users.push(user_info);
            user_store.update();
        }
    }

    pub fn remove_pool_user(&self, pool_id: &String, user_id: &String) {
        let mut user_store = self.user_store.lock();
        if let Some(pool) = user_store.pools.get_mut(pool_id) {
            for i in 0..pool.pool_info.users.len() {
                if &pool.pool_info.users[i].user_id == user_id {
                    pool.pool_info.users.remove(i);
                    user_store.update();
                    return;
                }
            }
        }
    }

    pub fn add_pool_device(&self, pool_id: &String, user_id: &String, device: &PoolDeviceInfo) {
        let mut user_store = self.user_store.lock();
        if let Some(pool) = user_store.pools.get_mut(pool_id) {
            for user in pool.pool_info.users.iter_mut() {
                if &user.user_id == user_id {
                    for device in user.devices.iter() {
                        if device.device_id == device.device_id {
                            return;
                        }
                    }

                    user.devices.push(device.clone());
                    user_store.update();
                    return;
                }
                break;
            }
        }
    }

    pub fn basic_user_info(&self) -> BasicUserInfo {
        let user_store = self.user_store.lock();
        BasicUserInfo {
            user_id: user_store.user_info.user_id.clone(),
            display_name: user_store.user_info.display_name.clone(),
            device: user_store.device.clone(),
        }
    }

    pub fn device_id(&self) -> String {
        let user_store = self.user_store.lock();
        user_store.device.device_id.clone()
    }

    pub fn _set_display_name(&self, display_name: String) {
        let mut user_store = self.user_store.lock();
        user_store.user_info.display_name = display_name;
    }

    pub fn _display_name(&self) -> String {
        let user_store = self.user_store.lock();
        user_store.user_info.display_name.clone()
    }

    pub fn _user_id(&self) -> String {
        let user_store = self.user_store.lock();
        user_store.user_info.user_id.clone()
    }
}