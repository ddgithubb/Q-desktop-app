use crate::{
    events::{
        add_pool_file_offers_event, init_pool_file_seeders_event, remove_pool_file_offer_event, reconnect_pool_event,
    },
    poolpb::{PoolFileInfo, PoolFileSeeders},
    sspb::PoolBasicNode,
    store::user_store::BasicUserInfo,
    STORE_MANAGER,
};

use super::pool_node_position::PoolNodePosition;
use arc_swap::{ArcSwap, ArcSwapOption};
use flume::{Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};

pub(super) struct FileSeeders {
    pub(super) file_info: PoolFileInfo,
    pub(super) seeders: HashSet<String>, // node_ids
}

pub(super) struct AvailableFiles {
    pub(super) file_seeders: HashMap<String, FileSeeders>, // file_id -> file_seeders
    pub(super) file_offers: HashMap<String, HashSet<String>>, // node_id -> file_ids
}

pub(super) struct PoolState {
    pub(super) pool_id: String,
    pub(super) instant_seed: Instant,

    pub(super) user: BasicUserInfo,

    pub(super) node_id: String,
    pub(super) node_position: ArcSwap<PoolNodePosition>,

    reconnect: AtomicBool,
    auth_error: AtomicBool,
    closed: AtomicBool,
    close_chan_tx: ArcSwapOption<Sender<()>>,
    close_chan_rx: Receiver<()>,

    latest: AtomicBool,
    _is_only_node: AtomicBool,

    pub(super) active_nodes: RwLock<HashMap<String, Vec<u32>>>,
    available_files: Mutex<AvailableFiles>,
}

impl PoolState {
    pub(super) fn init(pool_id: String) -> Self {
        let (close_chan_tx, close_chan_rx) = flume::bounded::<()>(1);
        let user = STORE_MANAGER.basic_user_info();
        let node_id = user.device.device_id.clone();
        let pool_state = PoolState {
            pool_id,
            instant_seed: Instant::now(),
            user,
            node_id,
            node_position: ArcSwap::new(Arc::new(Default::default())),
            reconnect: AtomicBool::new(true),
            auth_error: AtomicBool::new(false),
            closed: AtomicBool::new(false),
            close_chan_tx: ArcSwapOption::new(Some(Arc::new(close_chan_tx))),
            close_chan_rx,
            latest: AtomicBool::new(false),
            _is_only_node: AtomicBool::new(false),
            active_nodes: RwLock::new(HashMap::new()),
            available_files: Mutex::new(AvailableFiles::new()),
        };

        pool_state.add_file_offers(
            &pool_state.node_id,
            STORE_MANAGER.file_offers(&pool_state.pool_id),
        );

        pool_state
    }

    pub(super) fn set_disconnect(&self) {
        self.reconnect.store(false, Ordering::SeqCst);
    }

    pub(super) fn set_auth_error(&self) {
        self.auth_error.store(true, Ordering::SeqCst);
    }

    pub(super) fn reconnect(&self) -> bool {
        self.reconnect.load(Ordering::SeqCst)
    }

    pub(super) fn close(&self) -> bool {
        if !self.is_closed() {
            self.closed.store(true, Ordering::SeqCst);
            let close_chan_tx = self.close_chan_tx.swap(None);
            if let Some(_close_chan_tx) = close_chan_tx {
                if self.reconnect() {
                    let auth_error = self.auth_error.load(Ordering::SeqCst);
                    let pool_id = self.pool_id.clone(); 
                    reconnect_pool_event(&pool_id, auth_error);
                }
                // let _ = _close_chan_tx.try_send(()); // will be dropped anyways
                return true;
            }
        }
        return false;
    }

    pub(super) fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    pub(super) async fn close_signal(&self) {
        let _ = self.close_chan_rx.recv_async().await;
    }

    pub(super) fn close_chan_rx(&self) -> Receiver<()> {
        self.close_chan_rx.clone()
    }

    pub(super) fn _is_neighbouring_node(&self, node_id: &String) -> bool {
        let node_position = self.node_position.load();
        for i in 0..3 {
            if let Some(id) =
                &node_position.parent_cluster_node_ids[node_position.panel_number as usize][i]
            {
                if id == node_id {
                    return true;
                }
            }
        }
        return false;
    }

    pub(super) fn set_node_position(&self, node_position: PoolNodePosition) -> bool {
        let mut only_node = true;
        if node_position.center_cluster {
            'outer_loop: for panel in &node_position.parent_cluster_node_ids {
                for node_id in panel {
                    if let Some(node_id) = node_id {
                        if node_id != &self.node_id {
                            only_node = false;
                            break 'outer_loop;
                        }
                    }
                }
            }
            self._is_only_node.store(only_node, Ordering::SeqCst);

            if only_node {
                self.set_latest();
            }
        }

        self.node_position.store(Arc::new(node_position));
        only_node
    }

    pub(super) fn node_position_path(&self) -> Vec<u32> {
        let node_position = self.node_position.load();
        node_position.path.clone()
    }

    pub(super) fn partner_int(&self) -> usize {
        let node_position = self.node_position.load();
        node_position.partner_int
    }

    pub(super) fn set_latest(&self) {
        self.latest.store(true, Ordering::SeqCst);
    }

    pub(super) fn is_latest(&self) -> bool {
        self.latest.load(Ordering::SeqCst)
    }

    pub(super) fn update_active_node_path(&self, node_id: &String, path: Vec<u32>) {
        let mut w = self.active_nodes.write();
        w.insert(node_id.clone(), path);
    }

    pub(super) fn active_node_path(&self, node_id: &String) -> Option<Vec<u32>> {
        let r = self.active_nodes.read();
        r.get(node_id).cloned()
    }

    pub(super) fn is_node_active(&self, node_id: &String) -> bool {
        let r = self.active_nodes.read();
        r.contains_key(node_id)
    }

    pub(super) fn remove_node(&self, node_id: &String, promoted_nodes: Vec<PoolBasicNode>) {
        {
            let mut active_nodes = self.active_nodes.write();
            active_nodes.remove(node_id);

            for basic_node in promoted_nodes {
                active_nodes.insert(basic_node.node_id, basic_node.path);
            }
        }

        let mut available_files = self.available_files.lock();
        if let Some(file_offers) = available_files.file_offers.remove(node_id) {
            for file_id in file_offers {
                if let Some(file_seeders) = available_files.file_seeders.get_mut(&file_id) {
                    file_seeders.seeders.remove(node_id);

                    if file_seeders.seeders.is_empty() {
                        available_files.file_seeders.remove(&file_id);
                    }
                }
            }
        }
    }

    pub(super) fn add_file_offer(&self, seeder_node_id: &String, file_info: &PoolFileInfo) {
        let success = {
            let mut available_files = self.available_files.lock();
            available_files.add_seeder(seeder_node_id, file_info)
        };

        if !success {
            return;
        }

        log::debug!("add_file_offer {} {:?}", seeder_node_id, file_info);

        add_pool_file_offers_event(
            &self.pool_id,
            seeder_node_id.clone(),
            vec![file_info.clone()],
        );
    }

    pub(super) fn add_file_offers(&self, seeder_node_id: &String, file_offers: Vec<PoolFileInfo>) {
        if file_offers.is_empty() {
            return;
        }

        let file_offers = {
            let mut added_file_offers = Vec::with_capacity(file_offers.len());
            let mut available_files = self.available_files.lock();
            for file_offer in file_offers {
                if available_files.add_seeder(seeder_node_id, &file_offer) {
                    added_file_offers.push(file_offer);
                }
            }
            added_file_offers
        };

        if file_offers.is_empty() {
            return;
        }

        log::debug!("add file offers {} {:?}", seeder_node_id, file_offers);

        add_pool_file_offers_event(&self.pool_id, seeder_node_id.clone(), file_offers);
    }

    pub(super) fn remove_file_offer(&self, seeder_node_id: &String, file_id: &String) {
        let success = {
            let mut available_files = self.available_files.lock();
            available_files.remove_seeder(seeder_node_id, file_id)
        };

        if !success {
            return;
        }

        remove_pool_file_offer_event(&self.pool_id, seeder_node_id.clone(), file_id.clone());
    }

    pub(super) fn init_file_seeders(&self, file_seeders: Vec<PoolFileSeeders>) {
        if file_seeders.is_empty() {
            return;
        }

        let file_seeders: Vec<PoolFileSeeders> = {
            let mut available_files = self.available_files.lock();
            for file in file_seeders {
                let file_info = match &file.file_info {
                    Some(file_info) => file_info,
                    None => continue,
                };

                for seeder_id in file.seeder_node_ids {
                    available_files.add_seeder(&seeder_id, file_info);
                }
            }

            available_files.collect_file_seeders()
        };

        if file_seeders.is_empty() {
            return;
        }

        init_pool_file_seeders_event(&self.pool_id, file_seeders);
    }

    pub(super) fn collect_file_seeders(&self) -> Vec<PoolFileSeeders> {
        let available_files = self.available_files.lock();
        available_files.collect_file_seeders()
    }

    pub(super) fn is_available_file(&self, file_id: &String) -> bool {
        let available_files = self.available_files.lock();
        available_files.file_seeders.contains_key(file_id)
    }

    pub(super) fn sorted_file_seeders(&self, file_id: &String) -> Option<Vec<String>> {
        let mut seeders: Vec<String> = {
            let available_files = self.available_files.lock();
            match available_files.file_seeders.get(file_id) {
                Some(file_seeders) => {
                    if file_seeders.seeders.is_empty() {
                        return None;
                    }

                    file_seeders.seeders.iter().cloned().collect()
                }
                None => return None,
            }
        };

        let my_path = self.node_position_path();
        let active_nodes_path = self.active_nodes.read();
        seeders.sort_by(|a, b| {
            match (active_nodes_path.get(a), active_nodes_path.get(b)) {
                (Some(lsp_a), Some(lsp_b)) => Self::distance_between(&my_path, lsp_a)
                    .cmp(&Self::distance_between(&my_path, lsp_b)),
                _ => std::cmp::Ordering::Equal, // loically impossible
            }
        });

        Some(seeders)
    }

    fn distance_between(path1: &Vec<u32>, path2: &Vec<u32>) -> usize {
        let mut matches = 0;
        let min_len = std::cmp::min(path1.len(), path2.len());
        while matches < min_len {
            if path1[matches] != path2[matches] {
                break;
            }
            matches += 1;
        }
        (path1.len() - matches) + (path2.len() - matches)
    }
}

impl AvailableFiles {
    fn new() -> Self {
        AvailableFiles {
            file_seeders: HashMap::new(),
            file_offers: HashMap::new(),
        }
    }

    fn collect_file_seeders(&self) -> Vec<PoolFileSeeders> {
        self.file_seeders
            .values()
            .map(|file| PoolFileSeeders {
                file_info: Some(file.file_info.clone()),
                seeder_node_ids: file.seeders.iter().cloned().collect(),
            })
            .collect()
    }

    fn add_seeder(&mut self, seeder_node_id: &String, file_info: &PoolFileInfo) -> bool {
        if let Some(file_seeders) = self.file_seeders.get_mut(&file_info.file_id) {
            if file_seeders.file_info.file_name != file_info.file_name
                || file_seeders.file_info.total_size != file_info.total_size
            {
                return false;
            }

            if !file_seeders.seeders.insert(seeder_node_id.clone()) {
                return false;
            }
        } else {
            let mut seeders = HashSet::new();
            seeders.insert(seeder_node_id.clone());

            self.file_seeders.insert(
                file_info.file_id.clone(),
                FileSeeders {
                    file_info: file_info.clone(),
                    seeders,
                },
            );
        }

        if let Some(file_offers) = self.file_offers.get_mut(seeder_node_id) {
            file_offers.insert(file_info.file_id.clone());
        } else {
            let mut file_ids = HashSet::new();
            file_ids.insert(file_info.file_id.clone());

            self.file_offers.insert(seeder_node_id.clone(), file_ids);
        }

        true
    }

    fn remove_seeder(&mut self, seeder_node_id: &String, file_id: &String) -> bool {
        match self.file_seeders.get_mut(file_id) {
            Some(file_seeders) => {
                if !file_seeders.seeders.remove(seeder_node_id) {
                    return false;
                }

                if file_seeders.seeders.is_empty() {
                    self.file_seeders.remove(file_id);
                }
            }
            None => return false,
        };

        if let Some(file_offers) = self.file_offers.get_mut(seeder_node_id) {
            file_offers.remove(file_id);

            if file_offers.is_empty() {
                self.file_offers.remove(seeder_node_id);
            }
        }

        true
    }
}
