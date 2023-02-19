use std::collections::{VecDeque, HashMap};

use parking_lot::Mutex;

use crate::{poolpb::PoolMessage, config::LATEST_MESSAGES_SIZE, events::{append_pool_message_event, init_pool_messages_event}};

pub struct MessagesDB {
    pool_messages: Mutex<HashMap<String, MessagesDBInternal>>, // pool_id -> internal
}

impl MessagesDB {
    pub fn init() -> Self {
        MessagesDB {
            pool_messages: Mutex::new(HashMap::new()),
        }
    }

    pub fn latest_messages(&self, pool_id: &String) -> Vec<PoolMessage> {
        let pool_messages = self.pool_messages.lock();
        if let Some(internal) = pool_messages.get(pool_id) {
            return internal.latest_messages()
        }
        Vec::new()
    }

    // Not necessarily latest?
    pub fn add_message(&self, pool_id: &String, msg: PoolMessage) {
        {
            let mut pool_messages = self.pool_messages.lock();
            if !pool_messages.contains_key(pool_id) {
                pool_messages.insert(pool_id.clone(), MessagesDBInternal::init());
            }

            if let Some(internal) = pool_messages.get_mut(pool_id) {
                internal.add_latest_message(msg.clone());
            }
        }

        append_pool_message_event(pool_id, msg);
    }

    pub fn add_latest_messages(&self, pool_id: &String, msgs: Vec<PoolMessage>) {
        {
            let mut pool_messages = self.pool_messages.lock();
            if !pool_messages.contains_key(pool_id) {
                pool_messages.insert(pool_id.clone(), MessagesDBInternal::init());
            }

            if let Some(internal) = pool_messages.get_mut(pool_id) {
                internal.add_latest_messages(msgs.clone());
            }
        }

        init_pool_messages_event(pool_id, msgs);
    }

}

struct MessagesDBInternal {
    latest_messages: VecDeque<PoolMessage>,
}

impl MessagesDBInternal {
    fn init() -> Self {
        MessagesDBInternal {
            latest_messages: VecDeque::new(),
        }
    }

    fn latest_messages(&self) -> Vec<PoolMessage> {
        self.latest_messages.iter().cloned().collect()
    }

    // SHOULD HAVE MECHANISM FOR MESSAGES DUPS
    fn add_latest_message(&mut self, msg: PoolMessage) {
        self.latest_messages.push_back(msg);

        if self.latest_messages.len() > LATEST_MESSAGES_SIZE {
            self.latest_messages.pop_front();
        }
    }

    // SHOULD HAVE MECHANISM FOR MESSAGES DUPS
    // ESPECIALY THIS, latest messages will be loaded by own db files
    // BUT new latest messages can override all that even if it is at latest
    fn add_latest_messages(&mut self, msgs: Vec<PoolMessage>) {
        self.latest_messages.extend(msgs.into_iter());

        let overflow_amount: isize = self.latest_messages.len() as isize - LATEST_MESSAGES_SIZE as isize;
        if overflow_amount > 0 {
            self.latest_messages.drain(..overflow_amount as usize);
        }
    }

}
