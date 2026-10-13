use orchestrator::wait_for_ready;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn readiness_requires_a_200_response() {
    let Ok(listener) = TcpListener::bind("127.0.0.1:0").await else {
        return;
    };
    let address = listener.local_addr().unwrap().to_string();
    let server = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let mut request = [0_u8; 256];
            let _ = stream.read(&mut request).await;
            let _ = stream
                .write_all(b"HTTP/1.1 503 Busy\r\nContent-Length: 0\r\n\r\n")
                .await;
        }
    });

    assert!(wait_for_ready(&address, "/healthz", 1).await.is_err());
    server.abort();
}

#[tokio::test]
async fn readiness_accepts_a_complete_200_response() {
    let Ok(listener) = TcpListener::bind("127.0.0.1:0").await else {
        return;
    };
    let address = listener.local_addr().unwrap().to_string();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = [0_u8; 256];
        let _ = stream.read(&mut request).await;
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
            .await
            .unwrap();
    });

    assert!(wait_for_ready(&address, "/healthz", 1).await.is_ok());
}
