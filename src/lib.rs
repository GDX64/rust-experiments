use std::time;

use futures::Future;

pub async fn bench<F: Future<Output = ()>>(f: impl Fn() -> F, times: usize) -> u128 {
    let now = time::Instant::now();
    for _ in 0..times {
        f().await;
    }
    now.elapsed().as_millis()
}
