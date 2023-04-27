use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::mpsc;

use crate::Channel;

pub struct MpscChannel<T> {
    sender: Arc<Mutex<mpsc::Sender<T>>>,
    receiver: Arc<Mutex<Option<mpsc::Receiver<T>>>>,
}

impl<T: Send> Channel for MpscChannel<T> {
    type Sender = mpsc::Sender<T>;
    type Receiver = mpsc::Receiver<T>;

    fn create() -> Self {
        let (sender, receiver) = mpsc::channel(1024);
        Self {
            sender: Arc::new(Mutex::new(sender)),
            receiver: Arc::new(Mutex::new(Some(receiver))),
        }
    }

    fn sender(&self) -> Self::Sender {
        self.sender.lock().clone()
    }

    fn receiver(&self) -> Self::Receiver {
        self.receiver.lock().take().unwrap_or_else(|| mpsc::channel(1).1)
    }
}
