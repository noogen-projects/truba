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
    fn sender(&self) -> Self::Sender;
    fn receiver(&self) -> Self::Receiver;
}

pub trait Message: 'static {
    type Channel: Channel;

    fn create_channel() -> Self::Channel {
        Self::Channel::create()
    }
}

pub type Sender<T> = <<T as Message>::Channel as Channel>::Sender;
pub type Receiver<T> = <<T as Message>::Channel as Channel>::Receiver;
