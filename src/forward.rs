use std::error::Error;
use tokio::io::{self};
use tokio::net::TcpListener;
use tokio::net::TcpStream;

async fn tcp_forward() -> Result<(), Box<dyn Error>> {
    let local_addr = "127.0.0.1:9090"; // 监听本地 9090 端口
    let remote_addr = "172.25.176.1:8080"; // 目标服务器地址

    let listener = TcpListener::bind(local_addr).await?;
    println!("正在监听本地地址: {}", local_addr);

    loop {
        // 接受新来的连接
        let (mut stream, client_addr) = listener.accept().await?;
        println!("接受来自 {} 的连接", client_addr);

        // 将 remote_addr 克隆后移入异步任务
        let remote_addr = remote_addr.to_string();

        // 为每个连接启动一个新的任务处理转发
        tokio::spawn(async move {
            // 尝试连接到目标服务器
            match TcpStream::connect(&remote_addr).await {
                Ok(mut outbound) => {
                    // 双向传输数据
                    // copy_bidirectional 会同时将 inbound 的数据写入 outbound，
                    // 以及将 outbound 的数据写入 inbound，直到某一端关闭连接
                    match io::copy_bidirectional(&mut stream, &mut outbound).await {
                        Ok((bytes_from_inbound, bytes_from_outbound)) => {
                            println!(
                                "连接关闭，转发数据：客户端->目标 {} 字节，目标->客户端 {} 字节",
                                bytes_from_inbound, bytes_from_outbound
                            );
                        }
                        Err(e) => {
                            eprintln!("数据转发发生错误: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("无法连接到目标 {}: {}", remote_addr, e);
                }
            }
        });
    }
}
