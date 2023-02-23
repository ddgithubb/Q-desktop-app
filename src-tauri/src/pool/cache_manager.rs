use std::cmp::{max, min};
use std::collections::{HashMap, VecDeque};
use std::fs::{remove_file, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem;
use std::path::PathBuf;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Arc;

use flume::{Receiver, Sender};
use parking_lot::Mutex;

use crate::config::{
    CACHE_CHUNK_BUFFER_AMOUNT, CACHE_CHUNK_SIZE, CHUNK_SIZE, MAX_CACHE_CHUNKS_AMOUNT,
};
use crate::poolpb::pool_message::FileRequestData;
use crate::poolpb::{PoolChunkMessage, PoolChunkRange};
use crate::store::file_store::FileStore;

use super::chunk::chunk_ranges::{ChunkRanges, ChunkRangesUtil};
use super::chunk::chunk_util::{
    cache_chunk_number_to_chunk_number, cache_chunk_number_to_partner_int_path,
    chunk_number_to_cache_chunk_number,
};
use super::pool_net::SendChunkInfo;
use super::pool_state::PoolState;

struct PromisedCacheRequestInfo {
    requesting_node_id: String,
    requested_chunk_ranges: ChunkRanges,
}
struct CacheChunk {
    file_id: String,
    cache_chunk_number: u64,
    promised_requests: HashMap<String, ChunkRanges>,
    // promised_requests: Vec<PromisedCacheRequestInfo>,

    // Any writes/reads have to acquire this lock
    active_lock: Arc<Mutex<()>>,
    chunk_ranges: ChunkRanges,
}

struct CacheChunks {
    // file_id -> cache_chunk_number -> queue_pos
    cache_chunks_pos: HashMap<String, HashMap<u64, usize>>,
    cache: [Option<CacheChunk>; MAX_CACHE_CHUNKS_AMOUNT],

    promised_queue: VecDeque<usize>,
}

pub(super) struct CacheManager {
    pool_state: Arc<PoolState>,

    send_chunk_tx: Sender<SendChunkInfo>,

    cache_file_chunk_tx: Sender<PoolChunkMessage>,

    promised_cache_chunks_head: AtomicIsize,

    wake_promised_cache_loop: Sender<()>,

    cache_chunks: Mutex<CacheChunks>,
    cache_file_path: PathBuf,
}

impl CacheManager {
    pub(super) fn init(
        pool_state: Arc<PoolState>,
        send_chunk_tx: Sender<SendChunkInfo>,
    ) -> Option<Arc<Self>> {
        let (writer_file_handle, cache_file_path) = match FileStore::create_cache_file_handle(
            pool_state.pool_id.clone(),
            pool_state.instant_seed.clone()
        ) {
            Some(file) => file,
            None => return None,
        };

        let (reader_file_handle, _) = match FileStore::create_cache_file_handle(
            pool_state.pool_id.clone(),
            pool_state.instant_seed.clone(),
        ) {
            Some(file) => file,
            None => return None,
        };

        let (cache_file_chunk, cache_file_chunk_recv) =
            flume::bounded::<PoolChunkMessage>(CACHE_CHUNK_BUFFER_AMOUNT);

        let (wake_promised_cache_loop_tx, wake_promised_cache_loop_rx) = flume::bounded::<()>(1);

        const INIT_CACHE_CHUNK: Option<CacheChunk> = None;
        let cache_manager = Arc::new(CacheManager {
            pool_state: pool_state,
            send_chunk_tx,
            cache_file_chunk_tx: cache_file_chunk,
            promised_cache_chunks_head: AtomicIsize::new(-1),
            wake_promised_cache_loop: wake_promised_cache_loop_tx,
            cache_chunks: Mutex::new(CacheChunks {
                cache_chunks_pos: HashMap::new(),
                cache: [INIT_CACHE_CHUNK; MAX_CACHE_CHUNKS_AMOUNT],
                promised_queue: VecDeque::with_capacity(MAX_CACHE_CHUNKS_AMOUNT),
            }),
            cache_file_path,
        });

        let cache_manager_clone = cache_manager.clone();
        std::thread::spawn(move || {
            cache_manager_clone.cache_file_chunk_loop(writer_file_handle, cache_file_chunk_recv);
        });

        let cache_manager_clone = cache_manager.clone();
        std::thread::spawn(move || {
            cache_manager_clone
                .promised_chunks_loop(reader_file_handle, wake_promised_cache_loop_rx);
        });

        Some(cache_manager)
    }

    pub(super) fn clean(&self) {
        let _ = remove_file(self.cache_file_path.clone());
    }

    pub(super) fn promise_cache_chunks(
        &self,
        requesting_node_id: String,
        file_request_data: &mut FileRequestData,
        partner_int_path: u32,
    ) -> bool {
        let mut cache_chunks = self.cache_chunks.lock();

        let promised_chunks_start = file_request_data.promised_chunks.len();
        for chunk_range in &file_request_data.requested_chunks {
            for i in chunk_number_to_cache_chunk_number(chunk_range.start)
                ..=chunk_number_to_cache_chunk_number(chunk_range.end)
            {
                if cache_chunk_number_to_partner_int_path(i) == partner_int_path {
                    let pos = match cache_chunks
                        .cache_chunks_pos
                        .get(&file_request_data.file_id)
                    {
                        Some(cache_chunks_pos) => match cache_chunks_pos.get(&i) {
                            Some(pos) => *pos,
                            None => continue,
                        },
                        None => continue,
                    };

                    let cache_chunk = match &mut cache_chunks.cache[pos] {
                        Some(cache_chunk) => cache_chunk,
                        None => continue,
                    };

                    let queue = cache_chunk.promised_requests.is_empty();

                    for cache_chunk_range in &cache_chunk.chunk_ranges {
                        let chunk_range = PoolChunkRange {
                            start: max(chunk_range.start, cache_chunk_range.start),
                            end: min(chunk_range.end, cache_chunk_range.end),
                        };

                        if chunk_range.start <= chunk_range.end {
                            if let Some(chunk_ranges) =
                                cache_chunk.promised_requests.get_mut(&requesting_node_id)
                            {
                                chunk_ranges.add(&chunk_range);
                                file_request_data.promised_chunks.push(chunk_range);
                            } else {
                                file_request_data.promised_chunks.push(chunk_range.clone());
                                cache_chunk
                                    .promised_requests
                                    .insert(requesting_node_id.clone(), vec![chunk_range]);
                            }
                        }
                    }

                    if queue && !cache_chunk.promised_requests.is_empty() {
                        cache_chunks.promised_queue.push_back(pos);
                    }
                }
            }
        }

        drop(cache_chunks);

        let promised_chunks = &file_request_data.promised_chunks[promised_chunks_start..];
        if promised_chunks.is_empty() {
            return false;
        }

        file_request_data.requested_chunks =
            file_request_data.requested_chunks.diff(&promised_chunks);

        let _ = self.wake_promised_cache_loop.try_send(());
        true
    }

    pub(super) fn cache_file_chunk(&self, chunk_msg: PoolChunkMessage) {
        let _ = self.cache_file_chunk_tx.try_send(chunk_msg);
    }

    fn cache_file_chunk_loop(
        self: Arc<Self>,
        mut writer_file_handle: File,
        cache_file_chunk_recv: Receiver<PoolChunkMessage>,
    ) {
        let mut writer_head: usize = 0;
        let close_chan_rx = self.pool_state.close_chan_rx();

        loop {
            let close = flume::select::Selector::new()
                .recv(&close_chan_rx, |_| true)
                .recv(&cache_file_chunk_recv, |chunk_msg| {
                    if let Ok(chunk_msg) = chunk_msg {
                        writer_head = self.handle_cache_file_chunk(
                            &mut writer_file_handle,
                            writer_head,
                            chunk_msg,
                        );
                    }
                    false
                })
                .wait();

            if close {
                return;
            }
        }
    }

    fn handle_cache_file_chunk(
        &self,
        file_handle: &mut File,
        mut writer_head: usize,
        chunk_msg: PoolChunkMessage,
    ) -> usize {
        let cache_chunk_number = chunk_number_to_cache_chunk_number(chunk_msg.chunk_number);

        let (cache_chunk_pos, new_chunk, active_lock) = {
            let mut cache_chunks = self.cache_chunks.lock();

            let mut existing_cache_chunk_pos: Option<usize> = None;
            if let Some(cache_chunk_map) = cache_chunks.cache_chunks_pos.get(&chunk_msg.file_id) {
                if let Some(pos) = cache_chunk_map.get(&cache_chunk_number) {
                    existing_cache_chunk_pos = Some(*pos);
                }
            }

            let (cache_chunk_pos, new_chunk) = match existing_cache_chunk_pos {
                Some(cache_chunk_pos) => match &cache_chunks.cache[cache_chunk_pos] {
                    Some(cache_chunk)
                        if !cache_chunk.chunk_ranges.has_chunk(chunk_msg.chunk_number) =>
                    {
                        (cache_chunk_pos, false)
                    }
                    _ => return writer_head,
                },
                None if cache_chunks.promised_queue.len() >= MAX_CACHE_CHUNKS_AMOUNT - 1 => {
                    let active_read = self.promised_cache_chunks_head.load(Ordering::SeqCst);
                    if active_read >= 0 {
                        (active_read as usize, true)
                    } else {
                        return writer_head;
                    }
                }
                None => {
                    let mut cache_chunk_pos = writer_head;
                    loop {
                        let cache_chunk = match &cache_chunks.cache[cache_chunk_pos] {
                            Some(cache_chunk) => cache_chunk,
                            None => break,
                        };

                        if cache_chunk.promised_requests.len() == 0 {
                            break;
                        }

                        cache_chunk_pos = (cache_chunk_pos + 1) % MAX_CACHE_CHUNKS_AMOUNT;
                    }
                    writer_head = (cache_chunk_pos + 1) % MAX_CACHE_CHUNKS_AMOUNT;
                    (cache_chunk_pos, true)
                }
            };

            if new_chunk {
                if let Some(cache_chunk) = cache_chunks.cache[cache_chunk_pos].take() {
                    if let Some(cache_chunk_map) =
                        cache_chunks.cache_chunks_pos.get_mut(&cache_chunk.file_id)
                    {
                        cache_chunk_map.remove(&cache_chunk.cache_chunk_number);
                        if cache_chunk_map.is_empty() {
                            cache_chunks.cache_chunks_pos.remove(&cache_chunk.file_id);
                        }
                    }
                }

                let new_cache_chunk = CacheChunk {
                    file_id: chunk_msg.file_id,
                    cache_chunk_number,
                    promised_requests: HashMap::new(),
                    // active_state: Arc::new(AtomicUsize::new(WRITE_ACTIVE_STATE)),
                    active_lock: Arc::new(Mutex::new(())),
                    chunk_ranges: Vec::with_capacity(1),
                };

                let cache_chunk_map = cache_chunks
                    .cache_chunks_pos
                    .entry(new_cache_chunk.file_id.clone())
                    .or_insert(HashMap::new());
                cache_chunk_map.insert(cache_chunk_number, cache_chunk_pos);
                cache_chunks.cache[cache_chunk_pos] = Some(new_cache_chunk);
            }

            let active_lock = match &mut cache_chunks.cache[cache_chunk_pos] {
                Some(cache_chunk) => {
                    cache_chunk.chunk_ranges.add_chunk(chunk_msg.chunk_number);
                    cache_chunk.active_lock.clone()
                }
                None => return writer_head,
            };

            (cache_chunk_pos, new_chunk, active_lock)
        };

        let _active_lock = active_lock.lock();

        // NOTE: existing data is overwritten, literal sequential reading could result in reading old data

        let offset = (cache_chunk_pos * CACHE_CHUNK_SIZE) as u64
            + (CHUNK_SIZE as u64
                * (chunk_msg.chunk_number
                    - cache_chunk_number_to_chunk_number(cache_chunk_number)));

        let chunk_diff = CHUNK_SIZE as isize - chunk_msg.chunk.len() as isize;

        let write_ok = if chunk_diff >= 0 {
            if let Ok(_) = file_handle.seek(SeekFrom::Start(offset)) {
                if file_handle.write_all(&*chunk_msg.chunk).is_ok() {
                    true
                } else {
                    if chunk_diff != 0 {
                        let fill_buf: Vec<u8> = vec![0u8; chunk_diff as usize];
                        file_handle.write_all(&fill_buf).is_ok()
                    } else {
                        true
                    }
                }
            } else {
                false
            }
        } else {
            false
        };

        if !write_ok {
            let mut cache_chunks = self.cache_chunks.lock();
            if new_chunk {
                if let Some(cache_chunk) = cache_chunks.cache[cache_chunk_pos].take() {
                    if let Some(cache_chunk_map) =
                        cache_chunks.cache_chunks_pos.get_mut(&cache_chunk.file_id)
                    {
                        cache_chunk_map.remove(&cache_chunk.cache_chunk_number);
                        if cache_chunk_map.is_empty() {
                            cache_chunks.cache_chunks_pos.remove(&cache_chunk.file_id);
                        }
                    }
                }
                writer_head = cache_chunk_pos
            } else {
                if let Some(cache_chunk) = &mut cache_chunks.cache[cache_chunk_pos] {
                    cache_chunk.chunk_ranges =
                        cache_chunk.chunk_ranges.diff(&vec![PoolChunkRange {
                            start: chunk_msg.chunk_number,
                            end: chunk_msg.chunk_number,
                        }]);
                }
            }
        }

        writer_head
    }

    fn promised_chunks_loop(
        &self,
        mut file_handle: File,
        wake_promised_cache_loop_rx: Receiver<()>,
    ) {
        let close_chan_rx = self.pool_state.close_chan_rx();

        loop {
            loop {
                let (
                    file_id,
                    cache_chunk_pos,
                    chunk_ranges,
                    mut promised_requests_map,
                    active_lock,
                ) = {
                    let mut cache_chunks = self.cache_chunks.lock();
                    let cache_chunk_pos = match cache_chunks.promised_queue.pop_front() {
                        Some(cache_chunk_pos) => cache_chunk_pos,
                        None => break,
                    };

                    let cache_chunk = match &mut cache_chunks.cache[cache_chunk_pos] {
                        Some(cache_chunk) => cache_chunk,
                        None => continue,
                    };

                    self.promised_cache_chunks_head
                        .store(cache_chunk_pos as isize, Ordering::SeqCst);

                    (
                        cache_chunk.file_id.clone(),
                        cache_chunk_pos,
                        cache_chunk.chunk_ranges.clone(),
                        mem::take(&mut cache_chunk.promised_requests),
                        cache_chunk.active_lock.clone(),
                    )
                };

                let mut promised_requests: Vec<PromisedCacheRequestInfo> =
                    Vec::with_capacity(promised_requests_map.len());
                for (requesting_node_id, requested_chunk_ranges) in promised_requests_map.drain() {
                    if self.pool_state.is_node_active(&requesting_node_id) {
                        promised_requests.push(PromisedCacheRequestInfo {
                            requesting_node_id,
                            requested_chunk_ranges,
                        });
                    }
                }

                if promised_requests.len() == 0 {
                    continue;
                }

                let init_offset = (cache_chunk_pos * CACHE_CHUNK_SIZE) as u64;
                let chunk_number_offset = cache_chunk_number_to_chunk_number(
                    chunk_number_to_cache_chunk_number(chunk_ranges[0].start),
                );

                let _active_lock = active_lock.lock();

                for chunk_range in chunk_ranges.iter() {
                    for chunk_number in chunk_range.start..=chunk_range.end {
                        let mut dest_node_ids = Vec::with_capacity(promised_requests.len());
                        let mut send_to_self = false;
                        for request in &promised_requests {
                            for request_range in &request.requested_chunk_ranges {
                                if request_range.has_chunk(chunk_number) {
                                    if request.requesting_node_id == self.pool_state.node_id {
                                        send_to_self = true;
                                    } else {
                                        dest_node_ids.push(request.requesting_node_id.clone());
                                    }
                                    break;
                                }
                            }
                        }

                        if !send_to_self && dest_node_ids.is_empty() {
                            continue;
                        }

                        let mut buf = vec![0u8; CHUNK_SIZE];
                        let read_ok = if let Ok(_) = file_handle.seek(SeekFrom::Start(
                            init_offset
                                + (CHUNK_SIZE as u64 * (chunk_number - chunk_number_offset)),
                        )) {
                            file_handle.read_exact(&mut buf).is_ok()
                        } else {
                            false
                        };

                        if !read_ok {
                            continue;
                        }

                        let send_chunk_info = SendChunkInfo::create(
                            file_id.clone(),
                            chunk_number,
                            buf,
                            Some(dest_node_ids),
                            send_to_self,
                        );

                        if self.send_chunk_tx.send(send_chunk_info).is_err() {
                            return;
                        }
                    }
                }
            }

            self.promised_cache_chunks_head.store(-1, Ordering::SeqCst);

            let close = flume::select::Selector::new()
                .recv(&close_chan_rx, |_| true)
                .recv(&wake_promised_cache_loop_rx, |_| false)
                .wait();

            if close {
                return;
            }
        }
    }
}
