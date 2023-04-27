use std::marker::PhantomData;

use typemap::{Entry, SendMap};

use crate::{ActorId, Channel, Context, Message};

struct ChannelKey<M>(PhantomData<M>);

impl<M: Message> typemap::Key for ChannelKey<M> {
    type Value = M::Channel;
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
    pub fn sender<M: Message>(&mut self) -> <M::Channel as Channel>::Sender {
        match self.channels.entry::<ChannelKey<M>>() {
            Entry::Occupied(entry) => entry.get().sender(),
            Entry::Vacant(entry) => entry.insert(M::create_channel()).sender(),
        }
    }

    pub fn receiver<M: Message>(&mut self) -> <M::Channel as Channel>::Receiver {
        match self.channels.entry::<ChannelKey<M>>() {
            Entry::Occupied(entry) => entry.get().receiver(),
            Entry::Vacant(entry) => entry.insert(M::create_channel()).receiver(),
        }
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
}
