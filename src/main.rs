mod cli;
mod common;
mod error;
mod forward;
mod proxy;
mod tunnel;

use hyper::body::Bytes;

use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;

use std::error::Error;

use crate::cli::Cli;
use clap::Parser;
use tokio::io::{self};
use tokio::net::TcpListener;
use tokio::net::TcpStream;

type ClientBuilder = hyper::client::conn::http1::Builder;
type ServerBuilder = hyper::server::conn::http1::Builder;

type HttpBoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let listen = cli.get_listen();
    let forward = cli.get_forward();

    let url_proto = common::parse_url(listen)?;

    Ok(())
}

// async fn http_tunnel() -> Result<(), Box<dyn std::error::Error>> {
//     // 监听本地 1234 端口
//     let listener = TcpListener::bind("127.0.0.1:1234").await?;
//     println!("Proxy server listening on 127.0.0.1:1234");

//     loop {
//         // 接受客户端连接
//         let (stream, _client_addr) = listener.accept().await?;
//         tokio::spawn(async move {
//             if let Err(err) = handle_http_proxy(stream).await {
//                 eprintln!("Error handling client: {:?}", err);
//             }
//         });
//     }
// }

// async fn http_tun() -> Result<(), Box<dyn std::error::Error>> {
//     // 监听本地 1234 端口
//     let listener = TcpListener::bind("127.0.0.1:1234").await?;
//     println!("Proxy server listening on 127.0.0.1:1234");

//     loop {
//         // 接受客户端连接
//         let (stream, _client_addr) = listener.accept().await?;
//         let io = TokioIo::new(stream);
//         if let Err(err) = ServerBuilder::new()
//             .serve_connection(io, service_fn(handle_shadowsocks))
//             .with_upgrades()
//             .await
//         {
//             eprintln!("Error serving connection: {:?}", err);
//         }
//     }
// }

// async fn handle_shadowsocks(
//     mut req: Request<hyper::body::Incoming>,
// ) -> Result<Response<BoxBody<Bytes, std::io::Error>>, hyper::Error> {
//     let server_config = get_server_config("127.0.0.1", 8488, "123456", "aes-128-gcm");
//     let target = {
//         let host = req.uri().host().expect("uri has no host");
//         let port = req.uri().port_u16().unwrap_or(80);
//         format!("{}:{}", host, port)
//     };
//     let ss_conn = connect_to_shadowsocks(&target, &server_config)
//         .await
//         .unwrap();
//     if Method::CONNECT == req.method() {
//         // 执行升级，获取底层 TCP 连接
//         tokio::task::spawn(async move {
//             match hyper::upgrade::on(req).await {
//                 Ok(upgraded) => {
//                     // 使用 Shadowsocks 建立到目标服务器的加密连接
//                     let upgraded = TokioIo::new(upgraded);
//                     let (mut lr, mut lw) = tokio::io::split(upgraded);
//                     let (mut sr, mut sw) = tokio::io::split(ss_conn);

//                     let l2s = copy_to_encrypted(server_config.method(), &mut lr, &mut sw);
//                     let s2l = copy_from_encrypted(server_config.method(), &mut sr, &mut lw);

//                     tokio::pin!(l2s);
//                     tokio::pin!(s2l);

//                     let _ = future::select(l2s, s2l).await;
//                 }
//                 Err(e) => {
//                     eprintln!("升级错误: {:?}", e)
//                 }
//             }
//         });
//         return Ok(Response::new(empty_stream()));
//     } else {
//         // 将 Request 分解为 parts 和 body 方便处理
//         // 重构请求行：METHOD path HTTP/1.1
//         let mut request_bytes = Vec::with_capacity(1024);
//         let method = req.method();
//         let uri = req.uri();
//         let version = match req.version() {
//             hyper::Version::HTTP_10 => "HTTP/1.0",
//             hyper::Version::HTTP_11 => "HTTP/1.1",
//             hyper::Version::HTTP_2 => "HTTP/2.0",
//             _ => "HTTP/1.1", // 默认使用 HTTP/1.1
//         };

//         request_bytes.extend_from_slice(format!("{} {} {}\r\n", method, uri, version).as_bytes());

//         req.headers_mut().remove("Proxy-Connection");
//         req.headers_mut()
//             .insert("Connection", "close".parse().unwrap());

//         // 构造其它请求头：此处简单将所有请求头写入
//         for (name, value) in req.headers().iter() {
//             request_bytes.extend_from_slice(
//                 format!("{}: {}\r\n", name, value.to_str().unwrap_or("")).as_bytes(),
//             );
//         }
//         // 请求头结束标志
//         request_bytes.extend_from_slice(b"\r\n");

//         println!("request_bytes:{}", String::from_utf8_lossy(&request_bytes));

//         // 获取请求 body（如果有）
//         // let body_bytes = hyper::body::to_bytes(body)
//         //     .await
//         //     .unwrap_or_else(|_| hyper::body::Bytes::new());
//         // request_bytes.extend_from_slice(&body_bytes);

//         // 将构造好的 HTTP 请求报文写入 Shadowsocks 隧道

//         let (mut sr, mut sw) = io::split(ss_conn);
//         tokio::task::spawn(async move {
//             if let Err(e) = sw.write_all(&request_bytes).await {
//                 eprintln!("写入目标错误: {:?}", e);
//             }
//             // 写入请求体
//             // while let Some(chunk) = req.body_mut().frame().await {
//             //     let chunk = chunk.unwrap().into_data().unwrap();
//             //     if let Err(e) = sw.write_all(&chunk).await {
//             //         eprintln!("写入目标错误: {:?}", e);
//             //     }
//             // }
//             print!("写入完成");
//             if let Err(e) = sw.flush().await {
//                 eprintln!("刷新连接错误: {:?}", e);
//             }
//             if let Err(e) = sw.shutdown().await {
//                 eprintln!("关闭连接错误: {:?}", e);
//             }
//         });
//         let reader_stream = ReaderStream::new(sr);
//         let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
//         let boxed_body = stream_body.boxed();

//         let response = Response::builder()
//             .status(StatusCode::OK)
//             .body(boxed_body)
//             .unwrap();

//         return Ok(response);
//     }
// }

async fn tcp_tunnel(upgraded: Upgraded, addr: String) -> std::io::Result<()> {
    // Connect to remote server
    let mut remote_conn = TcpStream::connect(addr).await?;
    let upgraded = TokioIo::new(upgraded);
    let (mut ri, mut wi) = io::split(upgraded);
    let (mut ro, mut wo) = remote_conn.split();
    let client_to_server = io::copy(&mut ri, &mut wo);
    let server_to_client = io::copy(&mut ro, &mut wi);
    let _ = tokio::try_join!(client_to_server, server_to_client)?;
    Ok(())
}

//测试用例代码

#[cfg(test)]
mod tests {
    // use crate::connect_to_shadowsocks;

    #[tokio::test]
    async fn test_connect_shadowsocks() {
        // connect_to_shadowsocks(
        //     "www.baidu.com:443",
        //     "127.0.0.1",
        //     8488,
        //     "123456",
        //     "aes-128-gcm",
        // )
        // .await
        // .unwrap();
        // println!("success")
    }
}
