use crate::error::RetResult;
use hyper::Method;
use tokio::io::AsyncWriteExt;
use tokio::io::{self, AsyncReadExt};
use tokio::net::TcpStream;

//-L http://:8080
async fn handle_http_proxy(mut stream: TcpStream) -> RetResult<()> {
    let mut buf = [0; 2048]; // 一次最多读 2KB
    let mut headers = [httparse::EMPTY_HEADER; 16]; // 最多解析16个头
    let mut req = httparse::Request::new(&mut headers);

    let n = stream.read(&mut buf).await.unwrap();
    if let Ok(result) = req.parse(&buf[..n]) {
        match result {
            httparse::Status::Complete(_) => {}
            _ => {
                println!("Failed to parse request");
                return Ok(());
            }
        }
    } else {
        println!("Failed to parse request");
        return Ok(());
    }
    let method: Method = req.method.unwrap().parse().unwrap();
    let target = req.headers.iter().find(|h| h.name == "Host").unwrap().value;
    if Method::CONNECT == method {
        // 等待连接升级完成
        // 用于建立隧道连接，例如在代理服务器中，客户端通过 CONNECT
        // 方法请求与目标服务器建立隧道，服务器同意后，连接升级为原始的 TCP 流。
        let (mut ri, mut wi) = tokio::io::split(stream);
        wi.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await
            .unwrap();
        let mut remote_conn = TcpStream::connect(std::str::from_utf8(target).unwrap())
            .await
            .unwrap();
        tokio::task::spawn(async move {
            let (mut ro, mut wo) = remote_conn.split();
            let client_to_server = io::copy(&mut ri, &mut wo);
            let server_to_client = io::copy(&mut ro, &mut wi);
            let _ = tokio::try_join!(client_to_server, server_to_client).unwrap();
        });
    } else {
        let remote_conn = TcpStream::connect(std::str::from_utf8(target).unwrap())
            .await
            .unwrap();
        let (mut ro, mut wo) = tokio::io::split(remote_conn);
        wo.write_all(&buf[..n]).await.unwrap();
        let (mut ri, mut wi) = tokio::io::split(stream);
        let client_to_server = io::copy(&mut ri, &mut wo);
        let server_to_client = io::copy(&mut ro, &mut wi);
        let _ = tokio::try_join!(client_to_server, server_to_client).unwrap();
    }
    return Ok(());
}
