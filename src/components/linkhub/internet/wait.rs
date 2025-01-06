use std::error::Error;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use std::str;
use std::net::SocketAddr;
use tokio::time::interval;
use tokio::time::Duration;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

use crate::base::resource::{Resource, InternetResource};

const TAPE_ADDRESS: &str = "127.0.0.1:8888";

pub async fn wait() -> Result<(), Box<dyn Error>> {
    
    // TODO:
    // 1. 发现 外部seeker 的广播，并建立连接
    // 2. 监听来自 外部seeker 的请求和消息
    // 3. 接发 内部seeker 的信息
    
    // loop {
    //     tokio::select! {
    //         _ = ,
    //     }
    // }

    let socket = UdpSocket::bind(TAPE_ADDRESS).await.expect("Failed to bind to socket");
    println!("Listening on {}", TAPE_ADDRESS);

    let (tx, mut rx) = mpsc::channel::<(String, SocketAddr)>(32);

    tokio::spawn(async move {
        let mut buf = [0; 1024];
        loop {
            let (amt, src) = socket.recv_from(&mut buf).await.expect("Failed to receive data");
            println!("Received {} bytes from {}", amt, src);

            let received_data = str::from_utf8(&buf[..amt]).expect("Failed to convert to string");

            if tx.send((received_data.to_string(), src)).await.is_err() {
                println!("Failed to send message");
                break;
            }
        }
    });

    // every 60 seconds, check if the socket addresses are still valid
    let mut interval = interval(Duration::from_secs(60));


    loop {
        tokio::select! {
            Some((message, src)) = rx.recv() => {
                println!("Received message: {}", message);
                println!("Received src: {}", src);
                let parsed_data = serde_json::from_str(&message)?;
                store_resource(parsed_data, src);
            }
            // heartbeat
            _ = interval.tick() => {
                // Check stored socket addresses are still valid
                let mut invalid_addrs = Vec::new();
                for r in RESOURCES.lock().unwrap().iter() {
                    let resource: &dyn Resource = r.as_ref();
                    match resource.get_address() {
                        ResourceAddress::Internet(addr) => {
                            match socket.send_to(b"heartbeat", addr).await {
                                Ok(_) => {
                                    println!("Heartbeat sent to {}", addr);
                                },
                                Err(e) => {
                                    println!("Failed to send heartbeat to {}: {}", addr, e);
                                    invalid_addrs.push(addr);
                                }
                            }
                        },
                        _ => (),
                    }
                }
                
                // Remove invalid addresses
                for addr in invalid_addrs {
                    RESOURCES.lock().unwrap().retain(|resource| resource.get_address() != addr);
                }
            }
        }


    }   

}