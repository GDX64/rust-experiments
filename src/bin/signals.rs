use futures::Stream;
use std::borrow::BorrowMut;
use std::pin::Pin;
use std::task::{Context, Poll};

#[tokio::main]
async fn main() {}

struct MStream<F, St>
where
    St: Stream,
    F: Fn(St::Item),
{
    inner_stream: St,
    f_map: F,
}

impl<F, St> Stream for MStream<F, St>
where
    St: Stream + Unpin,
    F: Fn(St::Item) + Unpin,
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
