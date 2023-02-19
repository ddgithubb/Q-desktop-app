use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use flume::{Receiver, Sender};
use parking_lot::Mutex;

use crate::{
    events::state_update_event,
    ipc::{IPCFileDownloadProgress, IPCStateUpdate},
    STATE_UPDATER,
};

const STATE_UPDATER_INTERVAL: Duration = Duration::from_millis(500);

struct State {
    download_progress: HashMap<String, Arc<AtomicUsize>>, // file_id -> progress
}

impl State {
    fn new() -> Self {
        State {
            download_progress: HashMap::new(),
        }
    }

    fn check_inactive(&self) -> bool {
        self.download_progress.is_empty()
    }
}

pub struct StateUpdater {
    state: Mutex<State>,
    wake_updater_tx: Sender<()>,
}

impl StateUpdater {
    pub fn init() -> Self {
        let (wake_updater_tx, wake_updater_rx) = flume::bounded(0);

        Self::start_state_updater(wake_updater_rx);

        StateUpdater {
            state: Mutex::new(State::new()),
            wake_updater_tx,
        }
    }

    fn start_state_updater(wake_updater_rx: Receiver<()>) {
        tokio::spawn(async move {
            loop {
                let _ = wake_updater_rx.recv_async().await;

                loop {
                    if !STATE_UPDATER.trigger_update_state() {
                        break;
                    }

                    tokio::time::sleep(STATE_UPDATER_INTERVAL).await;
                }
            }
        });
    }

    fn wake_updater(&self) {
        let _ = self.wake_updater_tx.try_send(());
    }

    fn trigger_update_state(&self) -> bool {
        let state = self.state.lock();
        if state.check_inactive() {
            return false;
        }

        let mut file_downloads_progress = Vec::with_capacity(state.download_progress.len());

        for (file_id, progress) in state.download_progress.iter() {
            file_downloads_progress.push(IPCFileDownloadProgress {
                file_id: file_id.clone(),
                progress: progress.load(Ordering::Relaxed),
            });
        }

        state_update_event(IPCStateUpdate {
            file_downloads_progress,
        });

        true
    }

    pub fn register_download_progress(&self, file_id: String, progress: Arc<AtomicUsize>) {
        let mut state = self.state.lock();
        state.download_progress.insert(file_id, progress);
        self.wake_updater();
    }

    pub fn unregister_download_progress(&self, file_id: &String) {
        let mut state = self.state.lock();
        state.download_progress.remove(file_id);
    }
}
