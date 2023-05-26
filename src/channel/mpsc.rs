use std::mem;
use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::mpsc;

use crate::Channel;

pub struct MpscChannel<T> {
    sender: Arc<Mutex<mpsc::Sender<T>>>,
    receiver: Arc<Mutex<Option<mpsc::Receiver<T>>>>,
}

impl<T: Send> MpscChannel<T> {
    pub fn new(buffer: usize) -> Self {
        let (sender, receiver) = mpsc::channel(buffer);
        Self {
            sender: Arc::new(Mutex::new(sender)),
            receiver: Arc::new(Mutex::new(Some(receiver))),
        }
    }
}

impl<T: Send> Channel for MpscChannel<T> {
    type Sender = mpsc::Sender<T>;
    type Receiver = mpsc::Receiver<T>;

    fn create() -> Self {
        Self::new(1024)
    }

    fn sender(&self) -> Self::Sender {
        self.sender.lock().clone()
    }

    fn receiver(&self) -> Self::Receiver {
        self.receiver.lock().take().unwrap_or_else(|| mpsc::channel(1).1)
    }
}

pub struct UnboundedMpscChannel<T> {
    sender: Arc<Mutex<mpsc::UnboundedSender<T>>>,
    receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<T>>>>,
}

impl<T: Send> Default for UnboundedMpscChannel<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send> UnboundedMpscChannel<T> {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            sender: Arc::new(Mutex::new(sender)),
            receiver: Arc::new(Mutex::new(Some(receiver))),
        }
    }

    pub fn into_inner(self) -> (mpsc::UnboundedSender<T>, Option<mpsc::UnboundedReceiver<T>>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        (
            mem::replace(&mut *self.sender.lock(), sender),
            mem::replace(&mut *self.receiver.lock(), Some(receiver)),
        )
    }
}

impl<T: Send> Channel for UnboundedMpscChannel<T> {
    type Sender = mpsc::UnboundedSender<T>;
    type Receiver = mpsc::UnboundedReceiver<T>;

    fn create() -> Self {
        Self::new()
    }

    fn sender(&self) -> Self::Sender {
        self.sender.lock().clone()
    }

    fn receiver(&self) -> Self::Receiver {
        self.receiver
            .lock()
            .take()
            .unwrap_or_else(|| mpsc::unbounded_channel().1)
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc::error::SendError;

    use crate::{Context, Message, UnboundedMpscChannel};

    #[tokio::test]
    async fn extract_unbounded_channel() {
        struct Value(&'static str);

        impl Message for Value {
            type Channel = UnboundedMpscChannel<Self>;
        }

        let ctx = Context::default();

        assert!(matches!(ctx.extract_channel::<Value>(), None));

        let sender = ctx.sender::<Value>();
        let mut receiver = ctx.receiver::<Value>();

        sender.send(Value("inside")).ok().unwrap();
        assert_eq!(receiver.recv().await.unwrap().0, "inside");

        let (extracted_sender, _) = ctx.extract_channel::<Value>().unwrap().into_inner();

        extracted_sender.send(Value("extracted")).ok().unwrap();
        assert_eq!(receiver.recv().await.unwrap().0, "extracted");

        drop(sender);
        drop(extracted_sender);
        assert!(matches!(receiver.recv().await, None));

        let sender = ctx.sender::<Value>();
        drop(sender);

        let (sender, receiver) = ctx.extract_channel::<Value>().unwrap().into_inner();
        let mut receiver = receiver.unwrap();

        sender.send(Value("extracted")).ok().unwrap();
        assert_eq!(receiver.recv().await.unwrap().0, "extracted");

        drop(sender);
        assert!(matches!(receiver.recv().await, None));

        let mut receiver = ctx.receiver::<Value>();
        let (sender, _) = ctx.extract_channel::<Value>().unwrap().into_inner();

        sender.send(Value("extracted")).ok().unwrap();
        assert_eq!(receiver.recv().await.unwrap().0, "extracted");

        drop(sender);
        assert!(matches!(receiver.recv().await, None));

        let mut receiver = ctx.receiver::<Value>();
        let (sender, _) = ctx.extract_channel::<Value>().unwrap().into_inner();

        sender.send(Value("extracted")).ok().unwrap();
        assert_eq!(receiver.recv().await.unwrap().0, "extracted");

        drop(receiver);
        assert!(matches!(sender.send(Value("closed")), Err(SendError(Value("closed")))));
    }
}
