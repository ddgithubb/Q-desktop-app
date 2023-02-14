use std::collections::{HashSet, VecDeque};

use crate::config::RECEIVED_MESSAGES_SIZE;

pub struct ReceivedMessageQueue {
    queue: VecDeque<String>,
    set: HashSet<String>,
}

impl ReceivedMessageQueue {
    pub fn new() -> Self {
        ReceivedMessageQueue {
            queue: VecDeque::with_capacity(RECEIVED_MESSAGES_SIZE),
            set: HashSet::with_capacity(RECEIVED_MESSAGES_SIZE),
        }
    }

    // Returns true if successfully added, returns false otherwise
    pub fn append_message(&mut self, msg_id: &String) -> bool {
        if self.set.contains(msg_id) {
            return false;
        }

        if self.queue.len() == RECEIVED_MESSAGES_SIZE {
            if let Some(removed_msg_id) = self.queue.pop_front() {
                self.set.remove(&removed_msg_id);
            }
        }

        self.queue.push_back(msg_id.clone());
        self.set.insert(msg_id.clone());

        return true;
    }
}
