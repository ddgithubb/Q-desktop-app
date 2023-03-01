use std::{fs::{File, rename}, ops::{Deref, DerefMut}, io::{Write, Read}, path::PathBuf};

use serde::{Serialize, Deserialize};

use crate::config::PRODUCTION_MODE;

use super::store_manager::StoreManager;

const DISABLE_STORE: bool = !PRODUCTION_MODE;

pub struct Store<T> {
    name: String,
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
    pub fn new(name: String) -> Self {
        let mut store = Store {
            name,
            store_path: PathBuf::new(),
            store_data: T::default(),
        };

        store.load();
        store
    }

    pub fn load(&mut self) {
        if DISABLE_STORE {
            return;
        } else {
            unreachable!();
        }

        let (mut store_file, store_path) = Self::open_store_file(&self.name, false).unwrap();
        let mut b = Vec::new();
        store_file.read_to_end(&mut b).unwrap();
        self.store_path = store_path;
        if let Ok(data) = serde_json::from_slice(&b) {
            self.store_data = data;
        }
    }

    pub fn update(&mut self) {
        if DISABLE_STORE {
            return;
        } else {
            unreachable!();
        }

        let (mut tmp_store_file, tmp_path) = Self::open_store_file(&self.name, true).unwrap();
        tmp_store_file.set_len(0).unwrap();
        let b = serde_json::to_vec_pretty(&self.store_data).unwrap();
        tmp_store_file.write_all(&b).unwrap();
        rename(tmp_path, &self.store_path).unwrap();
    }

    pub fn open_store_file(name: &String, is_temp: bool) -> Option<(File, PathBuf)> {
        let mut path = match StoreManager::app_data_dir() {
            Some(path) => path,
            None => return None,
        };
        if is_temp {
            path.push(format!("{}.store.tmp", name));
        } else {
            path.push(format!("{}.store.json", name));
        }
        match File::options().write(true).read(true).create(true).open(path.clone()) {
            Ok(file) => Some((file, path)),
            _ => None,
        }
    }
}