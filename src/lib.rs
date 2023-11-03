pub use tokio;

pub use crate::channel::{
    BroadcastChannel, Channel, Message, MpscChannel, Receiver, Sender, UnboundedMpscChannel, WatchChannel,
};
pub use crate::context::{Context, DefaultActorId, DefaultContext};
pub use crate::continuous_stream::ContinuousStream;
pub use crate::system::System;

#[macro_export]
macro_rules! raw_event_loop {
    ($($select: tt)*) => {
        loop {
            $crate::tokio::select! $($select)*
        }
    };
}

#[macro_export]
macro_rules! min_event_loop {
    ({ $($select: tt)* }) => {
        loop {
            $crate::tokio::select! { $($select)* else => break }
        }
    };
}

#[macro_export]
macro_rules! event_loop {
    ($ctx: expr, { $($select: tt)* }) => {{
        // If shutdown channel exist and close
        if $ctx.is_channel_closed::<$crate::system::SystemShutdown>().unwrap_or(false) {
            drop($ctx);

            for _ in 0..1 {
                $crate::tokio::select! {
                    biased;
                    $($select)*
                    else => break
                };
            }
        } else {
            #[allow(non_snake_case)]
            let mut crate__shutdown_in_ = $ctx.receiver::<$crate::system::SystemShutdown>();
            drop($ctx);

            $crate::min_event_loop!({
                biased;
                $($select)*
                Ok(_) = crate__shutdown_in_.changed() => {
                    if let Some($crate::system::SystemShutdown) = *crate__shutdown_in_.borrow() {
                        break
                    }
                },
            });
        }
    }};
}

#[macro_export]
macro_rules! spawn_raw_event_loop {
    ($ctx: expr, $($select: tt)*) => {
        $ctx.spawn(async move {
            $crate::raw_event_loop!($($select)*)
        })
    };
}

#[macro_export]
macro_rules! spawn_min_event_loop {
    ($ctx: expr, { $($select: tt)* }) => {
        $ctx.spawn(async move {
            $crate::min_event_loop!({ $($select)* })
        })
    };
}

#[macro_export]
macro_rules! spawn_event_loop {
    ($ctx: expr, { $($select: tt)* }) => {
        $ctx.clone().spawn(async move {
            $crate::event_loop!(
                $ctx,
                { $($select)* }
            )
        })
    };
}

pub mod channel;
pub mod context;
pub mod continuous_stream;
pub mod system;

#[derive(Debug, Copy, Clone, Hash, PartialOrd, PartialEq, Ord, Eq)]
pub struct TaskId(u64);
