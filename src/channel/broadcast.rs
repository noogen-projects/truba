use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::broadcast;

use crate::Channel;

pub struct BroadcastChannel<T> {
    sender: Arc<Mutex<broadcast::Sender<T>>>,
}

impl<T: Send + Clone> BroadcastChannel<T> {
    pub fn new(buffer: usize) -> Self {
        let (sender, _receiver) = broadcast::channel(buffer);
        Self {
            sender: Arc::new(Mutex::new(sender)),
        }
    }
}

impl<T: Send + Clone> Channel for BroadcastChannel<T> {
    type Sender = broadcast::Sender<T>;
    type Receiver = broadcast::Receiver<T>;

    fn create() -> Self {
        Self::new(1024)
    }

    fn sender(&self) -> Self::Sender {
        self.sender.lock().clone()
    }

    fn receiver(&self) -> Self::Receiver {
        self.sender.lock().subscribe()
    }
}
