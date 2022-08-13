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
                    let mut pinned = Box::pin(delay_future);
                    match pinned.as_mut().poll(cx) {
                        Poll::Pending => {
                            self.delay = Some(pinned);
                            Poll::Pending
                        }
                        Poll::Ready(x) => Poll::Ready(Some(x)),
                    }
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

impl<T> DelayExt for T where T: Stream {}

trait DelayExt: Stream {
    fn my_delay(self) -> MyDelay<Self>
    where
        Self: Unpin + Sized,
    {
        MyDelay {
            delay: None,
            stream: self,
        }
    }
}

#[cfg(test)]
mod test {
    use futures::StreamExt;
    use tokio_stream;

    use crate::DelayExt;
    #[tokio::test]
    async fn test_delay() {
        let st = tokio_stream::iter([1]).my_delay().collect::<Vec<_>>().await;
        assert_eq!(st, [1]);
    }
}

fn main() {}
