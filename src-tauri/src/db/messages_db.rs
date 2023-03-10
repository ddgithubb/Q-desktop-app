use std::{
    collections::{HashMap, HashSet},
    fs::{create_dir, File},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use bytes::{Buf, Bytes};
use parking_lot::{Mutex, MutexGuard};
use prost::Message;

use crate::{
    config::{LATEST_MESSAGES_SIZE, MESSAGES_DB_CHUNK_SIZE, PRODUCTION_MODE},
    ipc::IPCPoolMessageHistory,
    poolpb::PoolMessage,
    store::store_manager::StoreManager, STORE_MANAGER,
};

// const TEMP_MESSAGES: bool = !PRODUCTION_MODE;
const TEMP_MESSAGES: bool = false;

pub struct MessagesDB {
    pool_messages: Mutex<HashMap<String, MessagesDBInternal>>, // pool_id -> internal
    
    max_messages_render: usize,
}

impl MessagesDB {
    pub fn init() -> Self {
        let db_path = Self::db_path().unwrap();
        let _ = create_dir(db_path);

        MessagesDB {
            pool_messages: Mutex::new(HashMap::new()),
            max_messages_render: STORE_MANAGER.max_messages_render(),
        }
    }

    pub fn last_messages(&self, pool_id: &String, size: usize) -> Vec<PoolMessage> {
        let mut pool_messages = self.pool_messages.lock();
        let internal = self.get_messages_internal(pool_id, &mut pool_messages);
        internal.last_messages(size)
    }

    pub fn messages_history_chunk_by_id(
        &self,
        pool_id: &String,
        msg_id: &String,
    ) -> IPCPoolMessageHistory {
        let mut pool_messages = self.pool_messages.lock();
        if let Some(internal) = pool_messages.get_mut(pool_id) {
            return internal.messages_history_chunk_by_id(msg_id, self.max_messages_render);
        }
        unreachable!()
    }

    pub fn messages_history_chunk(
        &self,
        pool_id: &String,
        chunk_number: u64,
    ) -> IPCPoolMessageHistory {
        let mut pool_messages = self.pool_messages.lock();
        if let Some(internal) = pool_messages.get_mut(pool_id) {
            return internal.messages_history_chunk(chunk_number);
        }
        unreachable!()
    }

    // Precondition: message is already filtired
    pub fn append_message(&self, pool_id: &String, msg: PoolMessage) {
        let mut pool_messages = self.pool_messages.lock();
        let internal = self.get_messages_internal(pool_id, &mut pool_messages);
        internal.append_message(msg.clone());
    }

    // Filters and adds latest messages
    pub fn add_latest_messages(&self, pool_id: &String, latest_msgs: Vec<PoolMessage>) {
        let mut pool_messages = self.pool_messages.lock();
        let internal = self.get_messages_internal(pool_id, &mut pool_messages);

        let last_messages = internal.last_messages(LATEST_MESSAGES_SIZE + 50);
        let mut existing_messages = HashSet::with_capacity(last_messages.len());
        for msg in last_messages {
            existing_messages.insert(msg.msg_id);
        }

        for msg in latest_msgs {
            if !existing_messages.contains(&msg.msg_id) {
                internal.append_message(msg);
            }
        }
    }

    fn get_messages_internal<'a>(
        &self,
        pool_id: &String,
        pool_messages: &'a mut MutexGuard<HashMap<String, MessagesDBInternal>>,
    ) -> &'a mut MessagesDBInternal {
        if !pool_messages.contains_key(pool_id) {
            pool_messages.insert(pool_id.clone(), MessagesDBInternal::init(pool_id.clone()));
        }

        pool_messages.get_mut(pool_id).unwrap()
    }

    fn db_path() -> Option<PathBuf> {
        match StoreManager::app_data_dir() {
            Some(mut path) => {
                path.push("db");
                Some(path)
            }
            None => None,
        }
    }
}

struct MessagesDBInternal {
    messages_file: File,

    current_chunk_number: u64,
    current_chunk_size: u64,
}

impl MessagesDBInternal {
    fn init(pool_id: String) -> Self {
        let messages_file = Self::open_messages_file(pool_id).unwrap();
        let messages_file_metadata = messages_file.metadata().unwrap();
        let messages_file_size = messages_file_metadata.len();

        let current_chunk_number = messages_file_size / MESSAGES_DB_CHUNK_SIZE;
        let current_chunk_size = messages_file_size % MESSAGES_DB_CHUNK_SIZE;

        let mut internal = MessagesDBInternal {
            messages_file,
            current_chunk_number,
            current_chunk_size,
        };

        internal.check_corrupt();

        internal
    }

    fn append_message(&mut self, msg: PoolMessage) {
        let mut buf = msg.encode_length_delimited_to_vec();
        self.pre_message_chunk_append(buf.len());
        self.messages_file.seek(SeekFrom::End(0)).unwrap();
        self.messages_file.write_all(&mut buf).unwrap();
    }

    fn last_messages(&mut self, size: usize) -> Vec<PoolMessage> {
        let mut msgs = Vec::with_capacity(size);
        let mut chunk_number = self.current_chunk_number;
        loop {
            let mut chunk = self.process_chunk(chunk_number);
            println!("chunk number {} chunk len {}", chunk_number, chunk.len());
            chunk.append(&mut msgs);
            msgs = chunk;

            if msgs.len() >= size {
                msgs = msgs.split_off(msgs.len() - size);
                break;
            }

            if chunk_number == 0 {
                break;
            }
            chunk_number -= 1;
        }

        msgs
    }

    fn messages_history_chunk_by_id(&mut self, msg_id: &String, min_messages: usize) -> IPCPoolMessageHistory {
        let mut messages = Vec::new();
        let mut chunk_number = self.current_chunk_number;
        let mut chunk_lens = Vec::with_capacity(2);
        'outer_loop: loop {
            let mut chunk = self.process_chunk(chunk_number);
            let chunk_len = chunk.len();

            for i in 0..chunk_len {
                if &chunk[i].msg_id == msg_id {
                    if i > chunk_len / 2 {
                        if chunk_number < self.current_chunk_number {
                            messages = self.process_chunk(chunk_number + 1);
                            chunk_lens.push(chunk.len());
                            chunk_lens.push(messages.len());
                            chunk.append(&mut messages);
                            messages = chunk;
                        } else {
                            chunk_lens.push(chunk.len());
                            messages = chunk;
                        }
                    } else {
                        if chunk_number > 0 {
                            chunk_number -= 1;
                            messages = self.process_chunk(chunk_number);
                            chunk_lens.push(messages.len());
                            chunk_lens.push(chunk.len());
                            messages.append(&mut chunk);
                        } else {
                            chunk_lens.push(chunk.len());
                            messages = chunk;
                        }
                    }

                    while messages.len() < min_messages {
                        if chunk_number == 0 {
                            break;
                        }
                        chunk_number -= 1;

                        let mut chunk = self.process_chunk(chunk_number);
                        chunk_lens.insert(0, chunk.len());
                        chunk.append(&mut messages);
                        messages = chunk;
                    }

                    break 'outer_loop;
                }
            }
            if chunk_number == 0 {
                break;
            }
            chunk_number -= 1;
        }

        IPCPoolMessageHistory {
            messages,
            chunk_lens,
            chunk_number,
            is_latest: chunk_number == self.current_chunk_number,
        }
    }

    fn messages_history_chunk(&mut self, chunk_number: u64) -> IPCPoolMessageHistory {
        if chunk_number > self.current_chunk_number {
            return IPCPoolMessageHistory {
                messages: Vec::new(),
                chunk_lens: Vec::new(),
                chunk_number,
                is_latest: false,
            };
        }

        let chunk = self.process_chunk(chunk_number);
        let chunk_len = chunk.len();
        IPCPoolMessageHistory {
            messages: chunk,
            chunk_lens: vec![chunk_len],
            chunk_number,
            is_latest: chunk_number == self.current_chunk_number,
        }
    }

    fn check_corrupt(&mut self) {
        let mut buf = vec![0u8; self.current_chunk_size as usize];
        self.messages_file
            .seek(SeekFrom::Start(
                self.current_chunk_number * MESSAGES_DB_CHUNK_SIZE,
            ))
            .unwrap();
        self.messages_file.read_exact(&mut buf).unwrap();

        let mut buf = Bytes::from(buf);
        while buf.has_remaining() {
            let remaining = buf.remaining();
            if PoolMessage::decode_length_delimited(&mut buf).is_err() {
                self.current_chunk_size -= remaining as u64;
                self.messages_file
                    .set_len(self.messages_file_size())
                    .unwrap();
                break;
            }
        }
    }

    fn process_chunk(&mut self, chunk_number: u64) -> Vec<PoolMessage> {
        let chunk_size = if chunk_number == self.current_chunk_number {
            self.current_chunk_size
        } else {
            MESSAGES_DB_CHUNK_SIZE
        };

        if chunk_size == 0 {
            return Vec::new();
        }

        let mut buf = vec![0u8; chunk_size as usize];
        self.messages_file
            .seek(SeekFrom::Start(chunk_number * MESSAGES_DB_CHUNK_SIZE))
            .unwrap();
        let _ = self.messages_file.read_exact(&mut buf).unwrap();

        let mut buf = Bytes::from(buf);
        let mut msgs = Vec::new();
        while buf.has_remaining() {
            match PoolMessage::decode_length_delimited(&mut buf) {
                Ok(msg) if !msg.msg_id.is_empty() => msgs.push(msg),
                _ => break,
            }
        }

        msgs
    }

    fn pre_message_chunk_append(&mut self, msg_buf_len: usize) {
        if self.current_chunk_size + msg_buf_len as u64 <= MESSAGES_DB_CHUNK_SIZE {
            self.current_chunk_size += msg_buf_len as u64;
            return;
        }

        let padding_len = MESSAGES_DB_CHUNK_SIZE - self.current_chunk_size;
        let mut padding = vec![0u8; padding_len as usize];
        self.messages_file.seek(SeekFrom::End(0)).unwrap();
        self.messages_file.write_all(&mut padding).unwrap();
        self.current_chunk_number += 1;
        self.current_chunk_size = msg_buf_len as u64;
    }

    fn messages_file_size(&self) -> u64 {
        (self.current_chunk_number * MESSAGES_DB_CHUNK_SIZE) + self.current_chunk_size
    }

    fn open_messages_file(pool_id: String) -> Option<File> {
        let mut path = match MessagesDB::db_path() {
            Some(db_path) => db_path,
            None => return None,
        };

        if TEMP_MESSAGES {
            path.push(format!(
                "{}-{}.msgs.db",
                pool_id,
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_micros()
            ));
        } else {
            path.push(format!("{}.msgs.db", pool_id));
        }

        File::options()
            .write(true)
            .read(true)
            .create(true)
            .open(path)
            .ok()
    }
}
