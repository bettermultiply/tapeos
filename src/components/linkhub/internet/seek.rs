use std::error::Error;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use std::str;
use std::net::SocketAddr;
use tokio::time::interval;
use tokio::time::Duration;
use crate::base::resource::Resource;
use crate::components::linkhub::internet::resource::InternetResource;
use crate::components::linkhub::seeker::INTERNET_RESOURCES;
use std::sync::Arc;

const TAPE_ADDRESS: &str = "127.0.0.1:8888";


pub async fn seek() -> Result<(), Box<dyn Error>> {
    // TODO: implement the logic to seek the higher level system by internet
    let socket = UdpSocket::bind(TAPE_ADDRESS).await.expect("Failed to bind to socket");
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
    let mut interval = interval(Duration::from_secs(60));


    loop {
        tokio::select! {
            Some((message, _)) = rx.recv() => {
                println!("Received message: {}", message);
                let parsed_data: InternetResource = serde_json::from_str(&message)?;
                store_resource(parsed_data);
            }
            // heartbeat
            _ = interval.tick() => {
                // Check stored socket addresses are still valid
                let mut invalid_addrs = Vec::new();
                for (name, resource) in INTERNET_RESOURCES.lock().unwrap().iter() {
                    let address = resource.get_address();
                    match socket.send_to(b"heartbeat", address).await {
                        Ok(_) => {
                            println!("Heartbeat sent to {}", address);
                        },
                        Err(e) => {
                            println!("Failed to send heartbeat to {}: {}", address, e);
                            invalid_addrs.push(name.clone());
                        }
                    }
                }
                
                // Remove invalid addresses
                for name in invalid_addrs {
                    INTERNET_RESOURCES.lock().unwrap().remove(&name);
                }
            }
        }
    }   
}

fn store_resource(resource: InternetResource) {

    INTERNET_RESOURCES.lock().unwrap().insert(resource.get_name().to_string(), Arc::new(resource));
}