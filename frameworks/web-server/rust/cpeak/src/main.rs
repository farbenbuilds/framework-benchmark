use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let _ = handle_connection(socket).await;
        });
    }
}

async fn handle_connection(mut socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = [0u8; 1024];
    loop {
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        let req = std::str::from_utf8(&buf[..n])?;
        if req.starts_with("GET /healthz") {
            socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                .await?;
        } else if req.starts_with("GET /plaintext") {
            let body = b"Hello, World!";
            let res = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\nHello, World!",
                body.len()
            );
            socket.write_all(res.as_bytes()).await?;
        } else if req.starts_with("GET /json") {
            let body = b"{\"message\": \"Hello, World!\"}";
            let res = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{\"message\": \"Hello, World!\"}",
                body.len()
            );
            socket.write_all(res.as_bytes()).await?;
        } else {
            socket
                .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n")
                .await?;
        }
    }
    Ok(())
}
