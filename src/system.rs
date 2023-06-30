use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::marker::PhantomData;

use typemap_ors::{Entry, Key, SendMap};

use crate::{Channel, Context, Message, TaskId, WatchChannel};

struct ChannelKey<M>(PhantomData<M>);

impl<M: Message> Key for ChannelKey<M> {
    type Value = M::Channel;
}

struct Channels(SendMap);

impl Default for Channels {
    fn default() -> Self {
        Self(SendMap::custom())
    }
}

impl Channels {
    fn sender<M: Message>(&mut self, constructor: impl FnOnce() -> M::Channel) -> <M::Channel as Channel>::Sender {
        match self.0.entry::<ChannelKey<M>>() {
            Entry::Occupied(entry) => entry.get().sender(),
            Entry::Vacant(entry) => entry.insert(constructor()).sender(),
        }
    }

    fn receiver<M: Message>(&mut self, constructor: impl FnOnce() -> M::Channel) -> <M::Channel as Channel>::Receiver {
        match self.0.entry::<ChannelKey<M>>() {
            Entry::Occupied(entry) => entry.get().receiver(),
            Entry::Vacant(entry) => entry.insert(constructor()).receiver(),
        }
    }

    fn get<M: Message>(&self) -> Option<&M::Channel> {
        self.0.get::<ChannelKey<M>>()
    }

    fn remove<M: Message>(&mut self) -> Option<M::Channel> {
        self.0.remove::<ChannelKey<M>>()
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SystemShutdown;

impl Message for SystemShutdown {
    type Channel = WatchChannel<Self>;
}

pub struct System<ActorId> {
    next_task_id: TaskId,
    channels: Channels,
    actor_channels: HashMap<ActorId, Channels>,
}

impl<ActorId> Default for System<ActorId> {
    fn default() -> Self {
        Self {
            next_task_id: TaskId(1),
            channels: Default::default(),
            actor_channels: Default::default(),
        }
    }
}

impl<ActorId: Eq + Hash> System<ActorId> {
    pub fn get_actor_channel<M: Message>(&self, actor_id: &ActorId) -> Option<&M::Channel> {
        self.actor_channels.get(actor_id)?.get::<M>()
    }

    pub fn extract_actor_channel<M: Message>(&mut self, actor_id: &ActorId) -> Option<M::Channel> {
        self.actor_channels.get_mut(actor_id)?.remove::<M>()
    }

    pub fn actor_sender_of_custom_channel<M: Message>(
        &mut self,
        actor_id: ActorId,
        constructor: impl FnOnce() -> M::Channel,
    ) -> <M::Channel as Channel>::Sender {
        self.actor_channels
            .entry(actor_id)
            .or_default()
            .sender::<M>(constructor)
    }

    pub fn actor_receiver_of_custom_channel<M: Message>(
        &mut self,
        actor_id: ActorId,
        constructor: impl FnOnce() -> M::Channel,
    ) -> <M::Channel as Channel>::Receiver {
        self.actor_channels
            .entry(actor_id)
            .or_default()
            .receiver::<M>(constructor)
    }

    pub fn actor_sender<M: Message>(&mut self, actor_id: ActorId) -> <M::Channel as Channel>::Sender {
        self.actor_sender_of_custom_channel::<M>(actor_id, || M::create_channel())
    }

    pub fn actor_receiver<M: Message>(&mut self, actor_id: ActorId) -> <M::Channel as Channel>::Receiver {
        self.actor_receiver_of_custom_channel::<M>(actor_id, || M::create_channel())
    }
}

impl<ActorId> System<ActorId> {
    pub fn get_channel<M: Message>(&self) -> Option<&M::Channel> {
        self.channels.get::<M>()
    }

    pub fn extract_channel<M: Message>(&mut self) -> Option<M::Channel> {
        self.channels.remove::<M>()
    }

    pub fn sender_of_custom_channel<M: Message>(
        &mut self,
        constructor: impl FnOnce() -> M::Channel,
    ) -> <M::Channel as Channel>::Sender {
        self.channels.sender::<M>(constructor)
    }

    pub fn receiver_of_custom_channel<M: Message>(
        &mut self,
        constructor: impl FnOnce() -> M::Channel,
    ) -> <M::Channel as Channel>::Receiver {
        self.channels.receiver::<M>(constructor)
    }

    pub fn sender<M: Message>(&mut self) -> <M::Channel as Channel>::Sender {
        self.sender_of_custom_channel::<M>(|| M::create_channel())
    }

    pub fn receiver<M: Message>(&mut self) -> <M::Channel as Channel>::Receiver {
        self.receiver_of_custom_channel::<M>(|| M::create_channel())
    }

    pub fn close_all_channels(&mut self) {
        self.channels.clear();
    }

    pub fn next_task_id(&mut self) -> TaskId {
        let task_id = self.next_task_id;
        self.next_task_id = TaskId(task_id.0 + 1);
        task_id
    }

    pub fn into_context(self) -> Context<ActorId> {
        self.into()
    }

    pub fn recv_shutdown(&mut self) -> impl Future<Output = bool> {
        let mut receiver = self.receiver::<SystemShutdown>();
        async move {
            if receiver.changed().await.is_ok() {
                receiver.borrow().is_some()
            } else {
                false
            }
        }
    }

    pub fn shutdown(&mut self) {
        self.sender::<SystemShutdown>().send_replace(Some(SystemShutdown));
    }
}
