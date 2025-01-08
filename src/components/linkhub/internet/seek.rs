use core::time;
use std::error::Error;
use std::thread::sleep;
use futures::lock::Mutex;
use lazy_static::lazy_static;
use log::info;
use log::warn;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use std::str;
use std::net::SocketAddr;
use tokio::time::interval;
use tokio::time::Duration;
use crate::base::intent::Intent;
use crate::base::intent::IntentType;
use crate::base::message::Message;
use crate::base::message::MessageType;
use crate::base::resource::Resource;
use crate::components::linkhub::internet::resource::InternetResource;
use crate::components::linkhub::seeker::INTERNET_RESOURCES;

use crate::base::intent::*;
use crate::core::inxt::intent::handler;
use std::sync::Arc;

pub const TAPE_ADDRESS: &str = "127.0.0.1:8888";
lazy_static! {
    pub static ref SOCKET: Mutex<Option<UdpSocket>> = Mutex::new(None);
}

pub async fn seek() -> Result<(), Box<dyn Error>> {

    let socket = UdpSocket::bind("127.0.0.1:8889").await.expect("Failed to bind to socket");
    SOCKET.lock().await.replace(socket);
    let socket_clone = UdpSocket::bind(TAPE_ADDRESS).await.expect("Failed to bind to socket");

    let (tx, mut rx) = mpsc::channel::<(String, SocketAddr)>(32);

    tokio::spawn(async move {
        let mut buf = [0; 1024];
        loop {
            println!("Listening on {}", TAPE_ADDRESS);
            let (amt, src) = socket_clone.recv_from(&mut buf).await.expect("Failed to receive data");
            println!("Received {} bytes from {}", amt, src);

            let received_data = str::from_utf8(&buf[..amt]).expect("Failed to convert to string");

            if tx.send((received_data.to_string(), src)).await.is_err() {
                println!("Failed to send message");
                break;
            }
        }
    });

    // every 60 seconds, check if the socket addresses are still valid
    let mut interval = interval(Duration::from_secs(10));


    loop {
        tokio::select! {
            Some((message, src)) = rx.recv() => {
                info!("Receive message: {}", message);
                // TODO: ERROR here
                let m: Message = match serde_json::from_str(&message) {
                    Ok(m) => m,
                    Err(e) => {
                        warn!("{:?}", e);
                        // TODO: try to parse
                        Message::new(MessageType::Unknow, "".to_string(), None)
                    },
                };
                match m.get_type() {
                    MessageType::Intent => {
                        fn find_resource_by_addr() -> String {
                            "Intent input".to_string()
                        }

                        let intent = Intent::new(m.get_body(), IntentSource::Resource, IntentType::Intent, Some(find_resource_by_addr()));
                        info!("get intent: {}", intent.get_description());
                        handler(intent).await;
                    },
                    MessageType::Register => {
                        match message2resource(m.get_body()) {
                            Ok(resource) => {
                                let m_json = match store_resource(resource) {
                                    Some(_) => {
                                        let m = Message::new(MessageType::Response, "Success".to_string(), None);
                                        serde_json::to_string(&m)?
                                    },
                                    None => {
                                        let m = Message::new(MessageType::Response, "Duplicate".to_string(), None);
                                        serde_json::to_string(&m)?
                                    },
                                };
                                info!("send to src: {}", src);
                                SOCKET.lock().await.as_ref().unwrap().send_to(&m_json.as_bytes().to_vec(), src).await?;
                            },
                            Err(e) => {
                                warn!("{:?}", e);
                            },
                        }
                    },
                    MessageType::Response => {},
                    MessageType::Reject => {},
                    _ => {
                        warn!("no such type");
                    }
                }
            }
            // heartbeat
            _ = interval.tick() => {
                // Check stored socket addresses are still valid
                info!("sending heart beat to check whether resource alive.");
                let mut invalid_addrs = Vec::new();
                for (name, resource) in INTERNET_RESOURCES.lock().unwrap().iter() {
                    let address = resource.get_address();
                    let m = Message::new(MessageType::Heartbeat, "".to_string(), None);
                    let m_json = serde_json::to_string(&m)?;
                    match SOCKET.lock().await.as_ref().unwrap().try_send_to(&m_json.as_bytes(), *address) {
                        Ok(_) => {
                            info!("Heartbeat sent to {}", address);
                        },
                        Err(e) => {
                            println!("Failed to send heartbeat to {}: {}", address, e);
                        }
                    }
                    let mut retry = 3;
                    let mut heart_buf = [0; 1024];
                    loop {
                        match SOCKET.lock().await.as_ref().unwrap().try_recv_from(&mut heart_buf) {
                            Ok(_) => {
                                info!("resource <{}> alive!", address);
                                break;
                            },
                            Err(_) => {
                                warn!("heartbeat of {} retry remain {}", address, retry);
                            },
                        }
                        retry -= 1;
                        if retry == 0 {
                            warn!("resource of {} disappear", address);
                            invalid_addrs.push(name.clone());
                            break;
                        }
                        sleep(time::Duration::from_secs(1));
                    }
                }
                
                // Remove invalid addresses
                for name in invalid_addrs {
                    warn!("remove resource {} ", &name);
                    INTERNET_RESOURCES.lock().unwrap().remove(&name);
                }
            }
        }
    }   
}

fn store_resource(resource: InternetResource) -> Option<()> {
    let mut i_rs = INTERNET_RESOURCES.lock().unwrap();
    let name = resource.get_name();
    if i_rs.get(name).is_none() {
        info!("store internet resource: {}", name);
        i_rs.insert(resource.get_name().to_string(), Arc::new(resource));
        Some(())
    } else {
        warn!("this resource has registered: {}", name);
        None
    }
}

fn message2resource(message: String) -> Result<InternetResource, Box<dyn Error>> {
    let resource: InternetResource = serde_json::from_str(&message)?;
    Ok(resource)
}