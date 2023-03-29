use std::{fs::{File, rename, create_dir}, ops::{Deref, DerefMut}, io::{Write, Read}, path::PathBuf};

use serde::{Serialize, Deserialize};

use crate::config::PRODUCTION_MODE;

use super::store_manager::StoreManager;

const DISABLE_STORE: bool = !PRODUCTION_MODE;
// const DISABLE_STORE: bool = false;

pub enum StoreDataType {
    JSON,
    Binary,
}

pub struct Store<T> {
    name: String,
    data_type: StoreDataType,
    store_path: PathBuf,
    store_data: T,
}

impl<T> Deref for Store<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.store_data
    }
}

impl<T> DerefMut for Store<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.store_data
    }
}

impl<'a, T: Default + Serialize + for<'de> Deserialize<'de>> Store<T> {
    pub fn open(name: String, data_type: StoreDataType) -> Self {
        let mut store = Store {
            name,
            data_type,
            store_path: PathBuf::new(),
            store_data: T::default(),
        };

        store.load();
        store
    }

    pub fn load(&mut self) {
        if DISABLE_STORE {
            return;
        }

        let (mut store_file, store_path) = self.open_store_file(false).unwrap();
        let mut b = Vec::new();
        store_file.read_to_end(&mut b).unwrap();
        self.store_path = store_path;

        let data = match self.data_type {
            StoreDataType::JSON => {
                serde_json::from_slice(&b).ok()
            },
            StoreDataType::Binary => {
                rmp_serde::decode::from_slice(&b).ok()
            }
        };

        if let Some(data) = data {
            self.store_data = data;
        }
    }

    pub fn update(&mut self) {
        if DISABLE_STORE {
            return;
        }

        let (mut tmp_store_file, tmp_path) = self.open_store_file(true).unwrap();
        tmp_store_file.set_len(0).unwrap();

        let b = match self.data_type {
            StoreDataType::JSON => {
                serde_json::to_vec_pretty(&self.store_data).unwrap()
            },
            StoreDataType::Binary => {
                rmp_serde::encode::to_vec(&self.store_data).unwrap()
            }
        };

        tmp_store_file.write_all(&b).unwrap();
        rename(tmp_path, &self.store_path).unwrap();
    }

    pub fn open_store_file(&self, is_temp: bool) -> Option<(File, PathBuf)> {
        let mut path = match StoreManager::app_data_dir() {
            Some(path) => path,
            None => return None,
        };

        path.push("store");
        match self.data_type {
            StoreDataType::Binary => {
                path.push("binary");
            },
            _ => {},
        }

        let _ = create_dir(path.clone());

        if is_temp {
            path.push(format!("{}.store.tmp", self.name));
        } else {
            match self.data_type {
                StoreDataType::JSON => {
                    path.push(format!("{}.store.json", self.name));
                },
                StoreDataType::Binary => {
                    path.push(format!("{}.store", self.name));
                }
            }
        }
        match File::options().write(true).read(true).create(true).open(path.clone()) {
            Ok(file) => Some((file, path)),
            _ => None,
        }
    }
}