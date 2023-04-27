use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use parking_lot::Mutex;
use tokio::task::JoinHandle;

use crate::{ActorId, Channel, Message, System, WatchChannel};

#[derive(Clone, Default)]
pub struct TaskHandles {
    handles: Arc<Mutex<HashMap<ActorId, JoinHandle<()>>>>,
}

impl TaskHandles {
    pub fn add(&self, id: ActorId, handle: JoinHandle<()>) {
        self.handles.lock().insert(id, handle);
    }

    pub async fn join_all(&self) {
        for (_, handle) in self.handles.lock().drain() {
            handle.await.ok();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SystemShutdown;

impl Message for SystemShutdown {
    type Channel = WatchChannel<Self>;
}

#[derive(Clone, Default)]
pub struct Context {
    system: Arc<Mutex<System>>,
    handles: TaskHandles,
}

impl Context {
    pub fn new(system: System) -> Self {
        Self {
            system: Arc::new(Mutex::new(system)),
            handles: Default::default(),
        }
    }

    pub fn new_system() -> Self {
        Self::default()
    }

    pub fn spawn<T>(&self, future: T)
    where
        T: Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(future);
        let actor_id = self.system.lock().next_actor_id();
        self.handles.add(actor_id, handle);
    }

    pub fn sender<M: Message>(&self) -> <M::Channel as Channel>::Sender {
        self.system.lock().sender::<M>()
    }

    pub fn receiver<M: Message>(&self) -> <M::Channel as Channel>::Receiver {
        self.system.lock().receiver::<M>()
    }

    pub fn extract_channel<M: Message>(&self) -> Option<M::Channel> {
        self.system.lock().extract_channel::<M>()
    }

    pub fn shutdown_system(&self) {
        self.sender::<SystemShutdown>().send_replace(Some(SystemShutdown));
        self.system.lock().close_all_channels();
    }

    pub fn recv_system_shutdown(&self) -> impl Future<Output = bool> {
        let mut receiver = self.receiver::<SystemShutdown>();
        async move {
            if receiver.changed().await.is_ok() {
                receiver.borrow().is_some()
            } else {
                false
            }
        }
    }

    pub async fn join_all(&self) {
        self.handles.join_all().await
    }
}

impl From<System> for Context {
    fn from(system: System) -> Self {
        Self::new(system)
    }
}
