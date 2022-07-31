use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to a peer
    let mut stream = TcpStream::connect("127.0.0.1:6142").await?;

    // Write some data.
    for _ in 1..5 {
        stream.write_all(b"hello world!").await?;
        let v = better_read(&mut stream).await?;
        let ans = String::from_utf8(v)?;
        println!("got: {:?}", ans);
        time::sleep(time::Duration::from_secs(5)).await;
        println!("slept");
    }

    Ok(())
}

async fn better_read(reader: &mut TcpStream) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buff = [0u8; 128];
    let n = reader.read(&mut buff).await?;
    let v = buff[..n].to_vec();
    Ok(v)
}
