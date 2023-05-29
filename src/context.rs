use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::sync::Arc;

use futures::future;
use parking_lot::{Mutex, MutexGuard};
use tokio::task::JoinHandle;

use crate::{Channel, Message, System, TaskId};

#[derive(Clone, Default)]
pub struct TaskHandles {
    handles: Arc<Mutex<HashMap<TaskId, JoinHandle<()>>>>,
}

impl TaskHandles {
    pub fn add(&self, id: TaskId, handle: JoinHandle<()>) {
        self.handles.lock().insert(id, handle);
    }

    pub async fn join_all(&self) {
        let handles: Vec<_> = self.handles.lock().drain().map(|(_, handle)| handle).collect();
        future::join_all(handles).await;
    }
}

pub type DefaultActorId = String;
pub type DefaultContext = Context<DefaultActorId>;

#[derive(Clone)]
pub struct Context<ActorId = DefaultActorId> {
    system: Arc<Mutex<System<ActorId>>>,
    handles: TaskHandles,
}

impl<ActorId> Default for Context<ActorId> {
    fn default() -> Self {
        Self {
            system: Default::default(),
            handles: Default::default(),
        }
    }
}

impl<ActorId> Context<ActorId> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_system(system: System<ActorId>) -> Self {
        Self {
            system: Arc::new(Mutex::new(system)),
            handles: Default::default(),
        }
    }

    pub fn spawn<T>(&self, future: T) -> TaskId
    where
        T: Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(future);
        let task_id = self.system().next_task_id();
        self.handles.add(task_id, handle);
        task_id
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

    pub fn system(&self) -> MutexGuard<'_, System<ActorId>> {
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

impl<ActorId: Eq + Hash> Context<ActorId> {
    pub fn actor_sender_of_custom_channel<M: Message>(
        &self,
        actor_id: ActorId,
        constructor: impl FnOnce() -> M::Channel,
    ) -> <M::Channel as Channel>::Sender {
        self.system().actor_sender_of_custom_channel::<M>(actor_id, constructor)
    }

    pub fn actor_receiver_of_custom_channel<M: Message>(
        &self,
        actor_id: ActorId,
        constructor: impl FnOnce() -> M::Channel,
    ) -> <M::Channel as Channel>::Receiver {
        self.system()
            .actor_receiver_of_custom_channel::<M>(actor_id, constructor)
    }

    pub fn actor_sender<M: Message>(&self, actor_id: ActorId) -> <M::Channel as Channel>::Sender {
        self.system().actor_sender::<M>(actor_id)
    }

    pub fn actor_receiver<M: Message>(&self, actor_id: ActorId) -> <M::Channel as Channel>::Receiver {
        self.system().actor_receiver::<M>(actor_id)
    }

    pub fn extract_actor_channel<M: Message>(&self, actor_id: &ActorId) -> Option<M::Channel> {
        self.system().extract_actor_channel::<M>(actor_id)
    }
}

impl<ActorId> From<System<ActorId>> for Context<ActorId> {
    fn from(system: System<ActorId>) -> Self {
        Self::from_system(system)
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc::error::SendError;

    use crate::{Context, Message, MpscChannel};

    struct Value(&'static str);

    impl Message for Value {
        type Channel = MpscChannel<Self>;
    }

    #[tokio::test]
    async fn actor_channels() {
        let ctx = Context::<usize>::new();

        let sender = ctx.sender::<Value>();
        let mut receiver = ctx.receiver::<Value>();

        sender.send(Value("common")).await.ok().unwrap();
        assert_eq!(receiver.recv().await.unwrap().0, "common");

        let actor_sender = ctx.actor_sender::<Value>(1);
        let mut actor_receiver = ctx.actor_receiver::<Value>(1);

        sender.send(Value("common")).await.ok().unwrap();
        actor_sender.send(Value("actor")).await.ok().unwrap();

        assert_eq!(receiver.recv().await.unwrap().0, "common");
        assert_eq!(actor_receiver.recv().await.unwrap().0, "actor");

        let (extracted_actor_sender, _) = ctx.extract_actor_channel::<Value>(&1).unwrap().into_inner();

        extracted_actor_sender
            .send(Value("extracted actor"))
            .await
            .ok()
            .unwrap();

        assert!(receiver.try_recv().is_err());
        assert_eq!(actor_receiver.recv().await.unwrap().0, "extracted actor");

        drop(actor_receiver);

        assert!(matches!(
            actor_sender.send(Value("actor closed")).await,
            Err(SendError(Value("actor closed")))
        ));
        assert!(matches!(
            extracted_actor_sender.send(Value("actor closed")).await,
            Err(SendError(Value("actor closed")))
        ));

        sender.send(Value("common")).await.ok().unwrap();
        assert_eq!(receiver.recv().await.unwrap().0, "common");
    }
}
