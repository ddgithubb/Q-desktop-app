use std::{fs::{create_dir}, path::PathBuf};

use log::info;
use parking_lot::Mutex;

use crate::{GLOBAL_APP_HANDLE};

use super::{file_store::FileStore, store::Store, user_store::UserStore};

pub const USER_STORE_NAME: &'static str = "user";
pub const FILE_STORE_NAME: &'static str = "file";

pub struct StoreManager {
    pub(super) user_store: Mutex<Store<UserStore>>,
    pub(super) file_store: Mutex<Store<FileStore>>,
}

impl StoreManager {
    pub fn init() -> Self {
        info!("Initializing Store Manager...");

        if !Self::create_app_data_dir() || !FileStore::create_temp_folder_path() {
            // reinitialize with auth data
            // only for corrupted data files, not for errors like this
            panic!()
        }

        let user_store: Store<UserStore> = Store::new(USER_STORE_NAME.to_string());
        let mut file_store: Store<FileStore> = Store::new(FILE_STORE_NAME.to_string());

        file_store.init();

        StoreManager {
            user_store: Mutex::new(user_store),
            file_store: Mutex::new(file_store),
        }
    }

    pub fn app_data_dir() -> Option<PathBuf> {
        match &*GLOBAL_APP_HANDLE.load() {
            Some(app_handle) => app_handle.path_resolver().app_data_dir(),
            None => None,
        }
    }

    fn create_app_data_dir() -> bool {
        if let Some(path) = Self::app_data_dir() {
            let _ = create_dir(path.clone());
            if path.exists() {
                return true;
            }
        }
        false
    }
}
