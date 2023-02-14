use std::collections::VecDeque;

use parking_lot::Mutex;

use crate::{poolpb::PoolMessage, config::LATEST_MESSAGES_SIZE};

pub struct MessagesDB {
    internal: Mutex<MessagesDBInternal>,
}

impl MessagesDB {
    pub fn init() -> Self {
        MessagesDB {
            internal: Mutex::new(MessagesDBInternal::init()),
        }
    }

    pub fn latest_messages(&self) -> Vec<PoolMessage> {
        let internal = self.internal.lock();
        internal.latest_messages()
    }

    // Not necessarily latest?
    pub fn add_message(&self, msg: PoolMessage) {
        let mut internal = self.internal.lock();
        internal.add_latest_message(msg);

        // fire event
        todo!()
    }

    pub fn add_latest_messages(&self, msgs: Vec<PoolMessage>) {
        let mut internal = self.internal.lock();
        internal.add_latest_messages(msgs);

        // fire event
        todo!()
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
