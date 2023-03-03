use std::{
    collections::{HashMap, VecDeque},
    fs::{create_dir, read_dir, remove_file},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH}, io::Read,
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, http::ResponseBuilder};
use tauri::http::{
    Request as HttpRequest,
    Response as HttpResponse,
};

use crate::{
    config::{FILE_ID_LENGTH, MAX_TEMP_FILES_SIZE_PER_POOL, MAX_TEMP_FILE_SIZE},
    poolpb::PoolFileInfo, STORE_MANAGER, POOL_MANAGER,
};

use super::store_manager::StoreManager;

#[derive(Debug)]
pub struct TempFile {
    pub file_id: String,
    pub file_size: u64,
    pub created: SystemTime,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct TempFileQueue {
    size: u64,
    queue: VecDeque<TempFile>,
}

#[derive(PartialEq)]
pub enum FilePathError {
    NoRecord,
    NotExist,
    OtherError,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FilePathInfo {
    pool_id: String,
    normalized_path: String,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileStore {
    file_offers: HashMap<String, HashMap<String, PoolFileInfo>>, // pool_id -> normalized_path.to_str() -> file_info

    #[serde(skip)]
    file_paths: HashMap<String, FilePathInfo>, // file_id -> file_path_info
    #[serde(skip)]
    temp_file_queues: HashMap<String, TempFileQueue>, // pool_id -> temp_file_queue
}

impl StoreManager {
    pub fn file_offers(&self, pool_id: &String) -> Vec<PoolFileInfo> {
        let file_store = self.file_store.lock();
        if let Some(pool_offers) = file_store.file_offers.get(pool_id) {
            return pool_offers.values().cloned().collect();
        }
        Vec::new()
    }

    pub fn file_offers_with_path(&self, pool_id: &String) -> Vec<(PathBuf, PoolFileInfo)> {
        let file_store = self.file_store.lock();
        if let Some(pool_offers) = file_store.file_offers.get(pool_id) {
            let mut offers = Vec::with_capacity(pool_offers.len());
            for (path, file_info) in pool_offers.iter() {
                offers.push((PathBuf::from(path.clone()), file_info.clone()))
            }

            return offers;
        }
        Vec::new()
    }

    pub fn add_file_offer(&self, pool_id: &String, file_info: PoolFileInfo, path: PathBuf) -> bool {
        if let Some(normalized_path) = FileStore::normalize_path(path) {
            if let Some(normalized_path) = normalized_path.to_str() {
                let mut file_store = self.file_store.lock();

                let file_id = file_info.file_id.clone();
                if let Some(pool_offers) = file_store.file_offers.get_mut(pool_id) {
                    if pool_offers.contains_key(normalized_path) {
                        return false;
                    } else {
                        pool_offers.insert(normalized_path.to_string(), file_info);
                    }
                } else {
                    let mut pool_offers = HashMap::new();
                    pool_offers.insert(normalized_path.to_string(), file_info);
                    file_store.file_offers.insert(pool_id.clone(), pool_offers);
                }

                file_store.file_paths.insert(
                    file_id,
                    FilePathInfo {
                        pool_id: pool_id.clone(),
                        normalized_path: normalized_path.to_string(),
                    },
                );

                file_store.update();
                return true;
            }
        }
        false
    }

    pub fn remove_file_offer(&self, file_id: &String) -> bool {
        let mut file_store = self.file_store.lock();
        if let Some(file_path) = file_store.file_paths.remove(file_id) {
            if let Some(pool_offers) = file_store.file_offers.get_mut(&file_path.pool_id) {
                if let Some(_) = pool_offers.remove(&file_path.normalized_path) {
                    file_store.update();
                    return true;
                }
            }
        }
        false
    }

    pub fn file_path(&self, file_id: &String) -> Result<(PathBuf, bool), FilePathError> {
        let file_store = self.file_store.lock();
        if let Some(file_path) = file_store.file_paths.get(file_id) {
            let path = PathBuf::from(file_path.normalized_path.clone());
            if path.exists() {
                // Quick attempt to figure out if it's a temp file
                let is_temp = match path.file_name() {
                    Some(file_name) => {
                        match file_name.to_str() {
                            Some(file_name) => file_name == file_id,
                            None => return Err(FilePathError::OtherError),
                        }
                    },
                    None => return Err(FilePathError::OtherError),
                };

                return Ok((path, is_temp));
            }
            return Err(FilePathError::NotExist)
        }
        Err(FilePathError::NoRecord)
    }

    pub fn add_temp_file(&self, pool_id: &String, temp_file: TempFile) -> Option<Vec<TempFile>> {
        if temp_file.file_size > MAX_TEMP_FILE_SIZE {
            return None;
        }

        let mut file_store = self.file_store.lock();
        if !file_store.temp_file_queues.contains_key(pool_id) {
            let temp_file_queue = TempFileQueue {
                size: 0,
                queue: VecDeque::new(),
            };

            file_store
                .temp_file_queues
                .insert(pool_id.clone(), temp_file_queue);
        }

        let temp_file_queue = file_store.temp_file_queues.get_mut(pool_id).unwrap();
        temp_file_queue.size += temp_file.file_size;
        temp_file_queue.queue.push_back(temp_file);

        let mut removed_temp_files: Vec<TempFile> = Vec::new();
        while temp_file_queue.size > MAX_TEMP_FILES_SIZE_PER_POOL {
            let removed_temp_file = temp_file_queue.queue.pop_front().unwrap();
            let _ = remove_file(removed_temp_file.path.clone());
            temp_file_queue.size -= removed_temp_file.file_size;
            removed_temp_files.push(removed_temp_file);
        }

        if removed_temp_files.is_empty() {
            None
        } else {
            Some(removed_temp_files)
        }
    }
}

impl FileStore {
    pub(super) fn init(&mut self) {
        self.validate_file_offers();
        let _ = self.init_temp_file_queues();
    }

    fn init_temp_file_queues(&mut self) -> std::io::Result<()> {
        if let Some(temp_folder_path) = Self::temp_folder_path() {
            for entry in read_dir(temp_folder_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(pool_id) = path.file_name() {
                        if let Some(pool_id) = pool_id.to_str() {
                            let _ = self.init_temp_file_queue(pool_id.to_string(), path);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn init_temp_file_queue(&mut self, pool_id: String, path: PathBuf) -> std::io::Result<()> {
        let mut size = 0;
        let mut queue: Vec<TempFile> = Vec::new();

        for entry in read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let file_id = match path.file_name() {
                    Some(file_name) => {
                        if let Some(file_name) = file_name.to_str() {
                            if file_name.len() != FILE_ID_LENGTH {
                                continue;
                            }
                            file_name.to_string()
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                };

                if let Ok(metadata) = path.metadata() {
                    if metadata.len() == 0 {
                        continue;
                    }

                    // Remove if old?

                    if let Ok(created) = metadata.created() {
                        let file_size = metadata.len();
                        size += file_size;
                        queue.push(TempFile {
                            file_id,
                            file_size,
                            created,
                            path,
                        });
                    }
                }
            }
        }

        queue.sort_by(|a, b| a.created.cmp(&b.created));
        self.temp_file_queues.insert(
            pool_id,
            TempFileQueue {
                size,
                queue: VecDeque::from(queue),
            },
        );

        Ok(())
    }

    pub(super) fn validate_file_offers(&mut self) {
        let mut file_paths: HashMap<String, FilePathInfo> = HashMap::new();
        for (pool_id, pool_offers) in self.file_offers.iter_mut() {
            pool_offers.retain(|normalized_path, file_info| {
                let path = PathBuf::from(normalized_path);
                if let Ok(metadata) = path.metadata() {
                    if metadata.len() == file_info.total_size {
                        file_paths.insert(
                            file_info.file_id.clone(),
                            FilePathInfo {
                                pool_id: pool_id.clone(),
                                normalized_path: normalized_path.clone(),
                            },
                        );
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            });
        }
        self.file_paths = file_paths;
    }

    pub fn register_media_protocol(_app: &AppHandle, req: &HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let response = ResponseBuilder::new();
        let path = req.uri().strip_prefix("media://localhost/").unwrap();
        let path: Vec<&str> = path.split("%2F").collect();
        let pool_id = path[0].to_string();
        let file_id = path[1].to_string();

        let file_path = match STORE_MANAGER.file_path(&file_id) {
            Ok((file_path, _)) => file_path,
            Err(err) => {
                if err == FilePathError::NotExist {
                    let pool_id = pool_id.clone();
                    let file_id = file_id.clone();
                    tokio::spawn(async move {
                        POOL_MANAGER.retract_file_offer(&pool_id, file_id).await;
                    });
                }

                match Self::temp_file_path(pool_id, file_id) {
                    Some(path) => path,
                    _ => return response.mimetype("text/plain").status(404).body(Vec::new()),
                }
            },
        };

        let mut file = match std::fs::File::open(file_path) {
            Ok(file) => file,
            Err(_) => return response.mimetype("text/plain").status(404).body(Vec::new()),
        };

        let mut buf = Vec::new();
        match file.read_to_end(&mut buf) {
            Ok(_) => {},
            _ => return response.mimetype("text/plain").status(404).body(Vec::new()),
        }

        response.body(buf)
    }

    pub fn normalize_path<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
        std::fs::canonicalize(path).ok()
    }

    pub fn create_temp_folder_path() -> bool {
        if let Some(path) = Self::temp_folder_path() {
            let _ = create_dir(path.clone());
            if path.exists() {
                return true;
            }
        }
        false
    }

    fn temp_folder_path() -> Option<PathBuf> {
        match StoreManager::app_data_dir() {
            Some(mut path) => {
                path.push("temp");
                Some(path)
            }
            None => return None,
        }
    }

    pub fn temp_file_path(pool_id: String, file_id: String) -> Option<PathBuf> {
        match Self::temp_folder_path() {
            Some(mut path) => {
                path.push(pool_id);

                if !path.exists() {
                    let _ = create_dir(path.clone());
                    if !path.exists() {
                        return None;
                    }
                }

                path.push(file_id);
                Some(path)
            }
            None => return None,
        }
    }

    pub fn check_path_could_exist(path: &PathBuf) -> bool {
        match path.try_exists() {
            Ok(true) | Err(_) => true,
            _ => false,
        }
    }

    pub fn create_valid_file_path(path: &mut PathBuf, file_name: &String) {
        path.push(file_name.clone());

        while Self::check_path_could_exist(path) {
            path.pop();
            path.push(format!(
                "{}-{}",
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
                file_name.clone(),
            ));
        }
    }
}
