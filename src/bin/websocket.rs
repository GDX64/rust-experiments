use tokio::{self, io::AsyncReadExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (mut conn, _) = listener.accept().await?;
        let mut buf = vec![0u8; 1024];
        let n = conn.read(&mut buf).await?;
        println!("GOT: {}", String::from_utf8_lossy(&buf[..n]));
    }
}
