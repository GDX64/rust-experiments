use std::borrow::BorrowMut;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::{Stream, StreamExt};

#[tokio::main]
async fn main() {
    bench_map().await;
}

impl<T> SuperStream for T where T: Stream {}

struct MStream<F, St, T>
where
    St: Stream,
    F: Fn(St::Item) -> T,
{
    inner_stream: St,
    f_map: F,
}

trait SuperStream: Stream {
    fn my_map<F, T>(self, f: F) -> MStream<F, Self, T>
    where
        Self: Sized,
        F: Fn(Self::Item) -> T + Unpin,
    {
        return MStream {
            inner_stream: self,
            f_map: f,
        };
    }
}

impl<F, St, T> Stream for MStream<F, St, T>
where
    St: Stream + Unpin,
    F: Fn(St::Item) -> T + Unpin,
{
    type Item = F::Output;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let smut = self.inner_stream.borrow_mut();
        let mut pinned = Pin::new(smut);
        match pinned.as_mut().poll_next(cx) {
            Poll::Ready(Some(x)) => Poll::Ready(Some((self.f_map)(x))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::SuperStream;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_map() {
        let mapped = tokio_stream::iter([1, 2]).my_map(|v| v.to_string());
        let v = mapped.collect::<Vec<_>>().await;
        assert_eq!(v, vec!["1", "2"])
    }
}

use experiments::bench;

async fn bench_map() {
    let f = || {
        let value = tokio_stream::iter(0..10_000_000)
            .my_map(|v: u64| v + 1)
            .fold(0, |acc, v| acc + v);
        return async {
            println!("value {}", value.await);
        };
    };
    let time = bench(f, 1).await;
    println!("millis {}", time);
    let f = || {
        let value = tokio_stream::iter(0..10_000_000)
            .map(|v: u64| v + 1)
            .fold(0, |acc, v| acc + v);
        return async {
            println!("value {}", value.await);
        };
    };
    let time = bench(f, 1).await;
    println!("millis {}", time);
}
