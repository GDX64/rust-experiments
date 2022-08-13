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
    duration: Duration,
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
                    let timer = tokio::time::sleep(self.duration);
                    let delay_future = async move {
                        timer.await;
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
    fn my_delay(self, duration: Duration) -> MyDelay<Self>
    where
        Self: Unpin + Sized,
    {
        MyDelay {
            delay: None,
            stream: self,
            duration,
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::{Duration, Instant};

    use futures::StreamExt;
    use tokio_stream;

    use crate::DelayExt;
    #[tokio::test]
    async fn test_delay() {
        let duration = Duration::from_micros(1);
        let st = tokio_stream::iter([1])
            .my_delay(duration)
            .collect::<Vec<_>>()
            .await;
        assert_eq!(st, [1]);
    }

    #[tokio::test]
    async fn test_delay_time() {
        let now = Instant::now();
        let duration = Duration::from_millis(500);
        let st = tokio_stream::iter([1, 2, 3, 4])
            .my_delay(duration)
            .collect::<Vec<_>>()
            .await;
        assert_eq!(2, now.elapsed().as_secs());
        assert_eq!(st, [1, 2, 3, 4]);
    }
}

fn main() {}
