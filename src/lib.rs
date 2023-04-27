pub use tokio;

pub use crate::channel::{Channel, Message, MpscChannel, Receiver, Sender, WatchChannel};
pub use crate::context::Context;
pub use crate::system::System;

#[macro_export]
macro_rules! spawn_event_loop {
    ($ctx: expr, $($select: tt)*) => {
        $ctx.spawn(async move {
            loop {
                $crate::tokio::select! $($select)*
            }
        })
    };
}

pub mod channel;
pub mod context;
pub mod system;

#[derive(Debug, Copy, Clone, Hash, PartialOrd, PartialEq, Ord, Eq)]
pub struct ActorId(u64);
