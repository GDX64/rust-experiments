use std::io::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("127.0.0.1:6142").await?;

    let (socket, _) = listener.accept().await?;
    let (mut reader, mut writer) = tokio::io::split(socket);
    writer.write(b"hellom there\r\n").await?;

    let mut buf = vec![0; 128];

    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        println!("GOT {:?}", &buf[..n]);
        writer.write(&buf[..n]).await?;
    }

    Ok(())
}
