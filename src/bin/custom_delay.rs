use futures::Future;
use std::borrow::BorrowMut;
use std::pin::Pin;
use std::process::Output;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio_stream::{Stream, StreamExt};

struct MyDelay<T>
where
    T: Stream + Unpin + 'static,
{
    stream: T,
    delay: Option<Pin<Box<dyn Future<Output = T::Item>>>>,
}

impl<T> Stream for MyDelay<T>
where
    T: Stream + Unpin + 'static,
{
    type Item = T::Item;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.delay {
            None => match Pin::new(self.stream.borrow_mut()).as_mut().poll_next(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Ready(Some(x)) => {
                    let delay_future = async move {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        x
                    };
                    let pinned = Box::pin(delay_future);
                    self.delay = Some(pinned);
                    Poll::Pending
                }
            },
            Some(ref mut delay) => match delay.as_mut().poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(x) => {
                    self.delay = None;
                    Poll::Ready(Some(x))
                }
            },
        }
    }
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn test_delay() {}
}

fn main() {}
