use crate::common::HttpBoxBody;
use futures::future;
use hyper::Request;
use hyper::Response;
use std::error::Error;
use tokio::io::{self};
use tokio::net::TcpListener;
use tokio::net::TcpStream;

use hyper::Method;

use shadowsocks::ProxyClientStream;
use shadowsocks::ServerAddr;
use shadowsocks::ServerConfig;
use shadowsocks::config::ServerType;
use shadowsocks::context::Context;
use shadowsocks::net::TcpStream as ShadowTcpStream;
use shadowsocks::relay::Address;
use shadowsocks::relay::tcprelay::utils::copy_from_encrypted;
use shadowsocks::relay::tcprelay::utils::copy_to_encrypted;
use shadowsocks_crypto::CipherKind;

use tokio::io::AsyncWriteExt;

use tokio::io::AsyncReadExt;

fn get_server_config(
    ss_server_addr: &str,
    ss_server_port: u16,
    ss_password: &str,
    ss_cipher: &str,
) -> ServerConfig {
    let method = ss_cipher.parse::<CipherKind>().unwrap();
    // Create the Shadowsocks configuration
    ServerConfig::new(
        ServerAddr::DomainName(ss_server_addr.to_owned(), ss_server_port),
        ss_password.to_owned(),
        method,
    )
    .unwrap()
}

async fn connect_to_shadowsocks(
    target_addr: &str,
    server_config: &ServerConfig,
) -> Result<ProxyClientStream<ShadowTcpStream>, Box<dyn Error>> {
    // Create a shared context
    let context = Context::new_shared(ServerType::Local);
    // Parse the target address
    let target_addr = target_addr.parse::<Address>()?;
    // Wrap the TCP stream with Shadowsocks encryption
    let proxy_stream = ProxyClientStream::connect(context, &server_config, target_addr).await?;
    Ok(proxy_stream)
}

//-L :8080 -F ss://xxxx
async fn handle_shadowsocks(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    // 读取客户端请求
    // let mut reader = BufReader::new(stream);
    let mut buf = [0; 4096]; // 一次最多读 4KB
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

    let server_config = get_server_config("127.0.0.1", 8488, "123456", "aes-128-gcm");
    let method: Method = req.method.unwrap().parse().unwrap();
    let target = req.headers.iter().find(|h| h.name == "Host").unwrap().value;

    let ss_conn = connect_to_shadowsocks(std::str::from_utf8(target).unwrap(), &server_config)
        .await
        .unwrap();
    println!("method: {:?}", method);
    println!("host: {:?}", std::str::from_utf8(target).unwrap());
    if Method::CONNECT == method {
        let (mut lr, mut lw) = tokio::io::split(stream);
        lw.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await
            .unwrap();
        tokio::task::spawn(async move {
            let (mut sr, mut sw) = tokio::io::split(ss_conn);
            let l2s = copy_to_encrypted(server_config.method(), &mut lr, &mut sw);
            let s2l = copy_from_encrypted(server_config.method(), &mut sr, &mut lw);

            tokio::pin!(l2s);
            tokio::pin!(s2l);

            let _ = future::select(l2s, s2l).await;
        });
    } else {
        let (mut sr, mut sw) = tokio::io::split(ss_conn);
        sw.write_all(&buf[..n]).await.unwrap();
        let (mut lr, mut lw) = tokio::io::split(stream);
        let l2s = copy_to_encrypted(server_config.method(), &mut lr, &mut sw);
        let s2l = copy_from_encrypted(server_config.method(), &mut sr, &mut lw);
        tokio::pin!(l2s);
        tokio::pin!(s2l);
        let _ = future::select(l2s, s2l).await;
    }
    return Ok(());
}

// -L:8080 -F http://xxxx
async fn handle_http_forward(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<HttpBoxBody>, hyper::Error> {
    todo!()
}

