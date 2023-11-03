pub use self::broadcast::BroadcastChannel;
pub use self::mpsc::{MpscChannel, UnboundedMpscChannel};
pub use self::watch::WatchChannel;

pub mod broadcast;
pub mod mpsc;
pub mod watch;

pub trait Channel: Send {
    type Sender;
    type Receiver;

    fn create() -> Self;

    fn actor_create(_actor_id: String) -> Self
    where
        Self: Sized,
    {
        Self::create()
    }

    fn sender(&self) -> Self::Sender;

    fn receiver(&self) -> Self::Receiver;

    fn is_closed(&self) -> bool;
}

pub trait Message: 'static {
    type Channel: Channel;

    fn create_channel() -> Self::Channel {
        Self::Channel::create()
    }

    fn create_actor_channel(actor_id: String) -> Self::Channel {
        Self::Channel::actor_create(actor_id)
    }
}

pub type Sender<T> = <<T as Message>::Channel as Channel>::Sender;
pub type Receiver<T> = <<T as Message>::Channel as Channel>::Receiver;
