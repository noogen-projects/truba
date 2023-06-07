pub use tokio;

pub use crate::channel::{
    BroadcastChannel, Channel, Message, MpscChannel, Receiver, Sender, UnboundedMpscChannel, WatchChannel,
};
pub use crate::context::{Context, DefaultActorId, DefaultContext};
pub use crate::continuous_stream::ContinuousStream;
pub use crate::system::System;

#[macro_export]
macro_rules! event_loop {
    ($($select: tt)*) => {
        loop {
            $crate::tokio::select! $($select)*
        }
    };
}

#[macro_export]
macro_rules! spawn_event_loop {
    ($ctx: expr, $($select: tt)*) => {
        $ctx.spawn(async move {
            $crate::event_loop!($($select)*)
        })
    };
}

pub mod channel;
pub mod context;
pub mod continuous_stream;
pub mod system;

#[derive(Debug, Copy, Clone, Hash, PartialOrd, PartialEq, Ord, Eq)]
pub struct TaskId(u64);
