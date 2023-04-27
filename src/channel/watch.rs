use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::watch;

use crate::Channel;

pub struct WatchChannel<T> {
    sender: Arc<Mutex<Option<watch::Sender<Option<T>>>>>,
    receiver: Arc<Mutex<watch::Receiver<Option<T>>>>,
}

impl<T: Send + Sync> Channel for WatchChannel<T> {
    type Sender = watch::Sender<Option<T>>;
    type Receiver = watch::Receiver<Option<T>>;

    fn create() -> Self {
        let (sender, receiver) = watch::channel(None);
        Self {
            sender: Arc::new(Mutex::new(Some(sender))),
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    fn sender(&self) -> Self::Sender {
        self.sender.lock().take().unwrap_or_else(|| watch::channel(None).0)
    }

    fn receiver(&self) -> Self::Receiver {
        self.receiver.lock().clone()
    }
}
