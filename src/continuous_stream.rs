use futures::stream::BoxStream;
use futures::{future, Stream, StreamExt};

#[derive(Default)]
pub struct ContinuousStream<S> {
    stream: Option<S>,
}

pub type ContinuousBoxStream<T> = ContinuousStream<BoxStream<'static, T>>;

impl<S> ContinuousStream<S> {
    pub fn new(stream: S) -> Self {
        Self { stream: Some(stream) }
    }

    pub fn set(&mut self, stream: S) {
        self.stream = Some(stream);
    }

    pub fn clear(&mut self) {
        self.stream = None;
    }

    pub fn is_empty(&self) -> bool {
        self.stream.is_none()
    }

    pub fn get_mut(&mut self) -> Option<&mut S> {
        self.stream.as_mut()
    }

    pub async fn recv(&mut self) -> Option<S::Item>
    where
        S: Stream + Unpin,
    {
        if let Some(stream) = &mut self.stream {
            stream.next().await.or_else(|| {
                self.stream = None;
                None
            })
        } else {
            future::pending().await
        }
    }
}
