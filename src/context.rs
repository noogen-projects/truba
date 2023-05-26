use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use futures::future;
use parking_lot::{Mutex, MutexGuard};
use tokio::task::JoinHandle;

use crate::{ActorId, Channel, Message, System};

#[derive(Clone, Default)]
pub struct TaskHandles {
    handles: Arc<Mutex<HashMap<ActorId, JoinHandle<()>>>>,
}

impl TaskHandles {
    pub fn add(&self, id: ActorId, handle: JoinHandle<()>) {
        self.handles.lock().insert(id, handle);
    }

    pub async fn join_all(&self) {
        let handles: Vec<_> = self.handles.lock().drain().map(|(_, handle)| handle).collect();
        future::join_all(handles).await;
    }
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

    pub fn spawn<T>(&self, future: T)
    where
        T: Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(future);
        let actor_id = self.system().next_actor_id();
        self.handles.add(actor_id, handle);
    }

    pub fn sender_of_custom_channel<M: Message>(
        &self,
        constructor: impl FnOnce() -> M::Channel,
    ) -> <M::Channel as Channel>::Sender {
        self.system().sender_of_custom_channel::<M>(constructor)
    }

    pub fn receiver_of_custom_channel<M: Message>(
        &self,
        constructor: impl FnOnce() -> M::Channel,
    ) -> <M::Channel as Channel>::Receiver {
        self.system().receiver_of_custom_channel::<M>(constructor)
    }

    pub fn sender<M: Message>(&self) -> <M::Channel as Channel>::Sender {
        self.system().sender::<M>()
    }

    pub fn receiver<M: Message>(&self) -> <M::Channel as Channel>::Receiver {
        self.system().receiver::<M>()
    }

    pub fn extract_channel<M: Message>(&self) -> Option<M::Channel> {
        self.system().extract_channel::<M>()
    }

    pub fn system(&self) -> MutexGuard<'_, System> {
        self.system.lock()
    }

    pub async fn shutdown(&self) {
        self.system().shutdown();
        self.join_all().await
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
