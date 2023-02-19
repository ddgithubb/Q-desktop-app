use std::{fs::{File, rename}, ops::{Deref, DerefMut}, io::{BufReader, BufWriter}, path::PathBuf};

use serde::{Serialize, Deserialize};

use super::store_manager::StoreManager;

pub struct Store<T> {
    name: String,
    store_file: File,
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
    pub fn new(name: String) -> Option<Self> {
        let (store_file, store_path) = match Self::open_store_file(&name, false) {
            Some(store) => store,
            None => return None,
        };

        let mut store = Store {
            name,
            store_file,
            store_path,
            store_data: T::default(),
        };

        store.load();
        Some(store)
    }

    pub fn load(&mut self) -> bool {
        let reader = BufReader::new(&mut self.store_file);
        if let Ok(data) = serde_json::from_reader::<_, T>(reader) {
            self.store_data = data;
            return true;
        }
        false
    }

    pub fn update(&mut self) -> bool {
        let (tmp_store_file, tmp_path) = match Self::open_store_file(&self.name, true) {
            Some(store) => store,
            None => return false,
        };
        let writer = BufWriter::new(tmp_store_file);
        if let Ok(_) = serde_json::to_writer_pretty(writer, &self.store_data) {
            return rename(tmp_path, &self.store_path).is_ok()
        }
        false
    }

    fn open_store_file(name: &String, is_temp: bool) -> Option<(File, PathBuf)> {
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