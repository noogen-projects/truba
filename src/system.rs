use std::future::Future;
use std::marker::PhantomData;

use typemap::{Entry, SendMap};

use crate::{ActorId, Channel, Context, Message, WatchChannel};

struct ChannelKey<M>(PhantomData<M>);

impl<M: Message> typemap::Key for ChannelKey<M> {
    type Value = M::Channel;
}

#[derive(Debug, Clone, Copy)]
pub struct SystemShutdown;

impl Message for SystemShutdown {
    type Channel = WatchChannel<Self>;
}

pub struct System {
    next_actor_id: ActorId,
    channels: SendMap,
}

impl Default for System {
    fn default() -> Self {
        Self {
            next_actor_id: ActorId(1),
            channels: SendMap::custom(),
        }
    }
}

impl System {
    pub fn sender_of_custom_channel<M: Message>(
        &mut self,
        constructor: impl FnOnce() -> M::Channel,
    ) -> <M::Channel as Channel>::Sender {
        match self.channels.entry::<ChannelKey<M>>() {
            Entry::Occupied(entry) => entry.get().sender(),
            Entry::Vacant(entry) => entry.insert(constructor()).sender(),
        }
    }

    pub fn receiver_of_custom_channel<M: Message>(
        &mut self,
        constructor: impl FnOnce() -> M::Channel,
    ) -> <M::Channel as Channel>::Receiver {
        match self.channels.entry::<ChannelKey<M>>() {
            Entry::Occupied(entry) => entry.get().receiver(),
            Entry::Vacant(entry) => entry.insert(constructor()).receiver(),
        }
    }

    pub fn sender<M: Message>(&mut self) -> <M::Channel as Channel>::Sender {
        self.sender_of_custom_channel::<M>(|| M::create_channel())
    }

    pub fn receiver<M: Message>(&mut self) -> <M::Channel as Channel>::Receiver {
        self.receiver_of_custom_channel::<M>(|| M::create_channel())
    }

    pub fn extract_channel<M: Message>(&mut self) -> Option<M::Channel> {
        self.channels.remove::<ChannelKey<M>>()
    }

    pub fn close_all_channels(&mut self) {
        self.channels.clear();
    }

    pub fn next_actor_id(&mut self) -> ActorId {
        let actor_id = self.next_actor_id;
        self.next_actor_id = ActorId(actor_id.0 + 1);
        actor_id
    }

    pub fn into_context(self) -> Context {
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
        self.close_all_channels();
    }
}
