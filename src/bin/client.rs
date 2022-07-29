use bytes::Bytes;
use std::error::Error;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}

/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    // Spawn two tasks, one gets a key, the other sets a key
    let t1 = tokio::spawn(async move {
        let (one_sender, one_receiver) = oneshot::channel();
        let cmd = Command::Get {
            key: "hello".to_string(),
            resp: one_sender,
        };
        tx.send(cmd).await.unwrap();
        let answer = one_receiver.await;
        println!("Got = {:?}", answer);
    });

    let t2 = tokio::spawn(async move {
        let (one_sender, one_receiver) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
            resp: one_sender,
        };
        tx2.send(cmd).await.unwrap();
        let answer = flat_result(one_receiver.await);
        println!("Got = {:?}", answer);
    });

    use mini_redis::client;
    // The `move` keyword is used to **move** ownership of `rx` into the task.
    let manager = tokio::spawn(async move {
        // Establish a connection to the server
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        // Start receiving messages
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key, resp } => {
                    let res = client.get(&key).await;
                    // Ignore errors
                    let _ = resp.send(res);
                }
                Command::Set { key, val, resp } => {
                    let res = client.set(&key, val).await;
                    // Ignore errors
                    let _ = resp.send(res);
                }
            }
        }
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}

type BoxDyn = Box<dyn Error + Send + Sync>;
fn flat_result<U, T, G>(r: Result<Result<U, G>, T>) -> Result<U, BoxDyn>
where
    T: Into<BoxDyn>,
    G: Into<BoxDyn>,
{
    r.map(|inner| inner.map_err::<BoxDyn, _>(|err| err.into()))
        .map_err::<BoxDyn, _>(|err| err.into())
        .and_then(|r| r)
}
