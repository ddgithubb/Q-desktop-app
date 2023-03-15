use serde::{Deserialize, Serialize};

use crate::config::{LATEST_MESSAGES_SIZE, MESSAGE_VIEWPORT_SIZE, MIN_MESSAGE_HIEGHT};

use super::store_manager::StoreManager;

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingStore {
    #[serde(skip)]
    monitor_height: u32,
    #[serde(skip)]
    max_messages_render: usize,
}

impl StoreManager {
    pub fn set_monitor_height(&self, height: u32) {
        let mut setting_store = self.setting_store.lock();
        setting_store.monitor_height = height;
        setting_store.max_messages_render = std::cmp::min(
            LATEST_MESSAGES_SIZE,
            ((height / MIN_MESSAGE_HIEGHT) * MESSAGE_VIEWPORT_SIZE) as usize,
        );
        
        log::debug!(
            "set_monitor_height : max_messages_render {}",
            setting_store.max_messages_render
        )
    }

    pub fn max_messages_render(&self) -> usize {
        let setting_store = self.setting_store.lock();
        setting_store.max_messages_render
    }
}
