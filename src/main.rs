use std::{error::Error, net::{IpAddr, Ipv4Addr, SocketAddr}, thread::sleep, time, str};

use log::{info, warn};
use tapeos::{
    base::{message::{Message, MessageType}, resource::Status}, components::linkhub::internet::{resource::InternetResource, seek::{seek, TAPE_ADDRESS}}, core::inxt::intent::random_execute, tools::idgen::init_id_generator
};
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() {
    env_logger::init();
    init_id_generator();

    // let intent = Intent::new("store my name".to_string(), IntentSource::Resource, IntentType::Intent, None);
    info!("main: Try to execute intent");
    tokio::spawn(async {
        let _ = register("MySQL".to_string(), MY_SQL_DESCRIPTION.to_string(), 8001).await;
    });
    tokio::spawn(async {
        let _ = register("MongoDB".to_string(), MONGO_DB_DESCRIPTION.to_string(), 8002).await;
    });
    tokio::spawn(async {
        let _ = register("GooGle Drive".to_string(), GOO_GLE_DRIVE_DESCRIPTION.to_string(), 8003).await;
    });

    tokio::spawn(async move {
        let _ = send_intent("Intent input".to_string(), INTENT_INPUT_DESCRIPTION.to_string(), 8004).await;
        
    });

    sleep(time::Duration::from_secs(3));

    let _ = seek().await;
    
    // intent::handler(intent).await;
    println!("main: Try ended");
}

async fn send_intent(name: String, desc: String, port: u16) -> Result<(), Box<dyn Error>>{
    info!("start to send intent");
    
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let socket = UdpSocket::bind(addr).await.expect("Failed to bind to socket");
    info!("socket binded");

    let status = Status::new(true, (0.0, 0.0, 0.0), time::Duration::from_secs(0));
    let resource = InternetResource::new(name, desc, addr, status);
    let r_json = serde_json::to_string(&resource)?;

    let m = Message::new(MessageType::Register, r_json);
    let m_json = serde_json::to_string(&m)?;

    info!("start to send message");
    let tape_clone = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8888);
    let tape = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8889);
    loop {
        match socket.try_send_to(&m_json.as_bytes().to_vec(), tape_clone) {
            Ok(r) => {
                info!("send register information successfully: {}", r);
            },
            Err(e) => {
                warn!("Failed to register to {}: {}, retry later", TAPE_ADDRESS, e);
                sleep(time::Duration::from_secs(1));
            }
        }
        let mut buf = [0; 1024];
        match socket.try_recv_from(&mut buf) {
            Ok((amt, src)) => {
                if src == tape {
                    let received_data = str::from_utf8(&buf[..amt])?;
                        let m: Message = match serde_json::from_str(&received_data) {
                            Ok(m) => m,
                            Err(e) => {
                                warn!("{:?}", e);
                                // TODO: try to parse
                                Message::new(MessageType::Unknow, "".to_string())
                            },
                        }; 
                        if *m.get_type() == MessageType::Response && m.get_body() == "Success" {
                            info!("intent solve successfully: {}", str::from_utf8(&buf[..amt]).expect("Fail to convert to String"));
                            break;
                        }
                }
            },
            Err(e) => {
                warn!("Failed to received from {}: {}, retry later", TAPE_ADDRESS, e);
                sleep(time::Duration::from_secs(1));
            },
                
        }
        sleep(time::Duration::from_secs(1));

    }

    let intent = "store my name: 'Betmul'".to_string();
    let m = Message::new(MessageType::Intent, intent);
    let m_json = serde_json::to_string(&m)?;

    loop {
        match socket.try_send_to(&m_json.as_bytes().to_vec(), tape_clone) {
            Ok(r) => {
                info!("send intent successfully: {}", r);
            },
            Err(e) => {
                warn!("Failed to register to {}: {}, retry later", TAPE_ADDRESS, e);
                sleep(time::Duration::from_secs(1));
            }
        }
        let mut buf = [0; 1024];
        loop {
            match socket.try_recv_from(&mut buf) {
                Ok((amt, src)) => {
                    if src == tape {
                        let received_data = str::from_utf8(&buf[..amt])?;
                        let m: Message = match serde_json::from_str(&received_data) {
                            Ok(m) => m,
                            Err(e) => {
                                warn!("{:?}", e);
                                // TODO: try to parse
                                Message::new(MessageType::Unknow, "".to_string())
                            },
                        }; 
                        match *m.get_type() {
                            MessageType::Response => {
                                info!("intent solve successfully: {}", str::from_utf8(&buf[..amt]).expect("Fail to convert to String"));
                            }
                            MessageType::Heartbeat => {
                                let h = Message::new(MessageType::Heartbeat, "".to_string());
                                let h_json = serde_json::to_string(&h)?;
                                info!("heart beat alive");
                                socket.send_to(&h_json.as_bytes(), tape).await?;
                            }
                            _ => {
                                warn!("do not support such type: {}", m.get_body());
                            },
                        }
                    }
                },
                Err(e) => {
                    // warn!("Failed to received from {}: {}, retry later", TAPE_ADDRESS, e);
                    sleep(time::Duration::from_secs(1));
                },
                
            }
            sleep(time::Duration::from_secs(1));
        }
    }

}

async fn register(name: String, desc: String, port: u16) -> Result<(), Box<dyn Error>>{
    info!("start to regist");
    
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let socket = UdpSocket::bind(addr).await.expect("Failed to bind to socket");
    info!("socket binded");

    let status = Status::new(true, (0.0, 0.0, 0.0), time::Duration::from_secs(0));
    let resource = InternetResource::new(name, desc, addr, status);
    let r_json = serde_json::to_string(&resource)?;

    let m = Message::new(MessageType::Register, r_json);
    let m_json = serde_json::to_string(&m)?;

    info!("start to send message");
    let tape_clone = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8888);
    let tape = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8889);
    loop {
        match socket.try_send_to(&m_json.as_bytes().to_vec(), tape_clone) {
            Ok(r) => {
                info!("send register information successfully: {}", r);
            },
            Err(e) => {
                warn!("Failed to register to {}: {}, retry later", TAPE_ADDRESS, e);
                sleep(time::Duration::from_secs(1));
            }
        }
        let mut buf = [0; 1024];
        match socket.try_recv_from(&mut buf) {
            Ok((amt, src)) => {
                if src == tape {
                    info!("Register successfully: {}", str::from_utf8(&buf[..amt]).expect("Fail to convert to String"));
                    break;
                }
            },
            Err(e) => {
                warn!("Failed to received from {}: {}, retry later", TAPE_ADDRESS, e);
                sleep(time::Duration::from_secs(1));
            },
                
        }
        sleep(time::Duration::from_secs(1));

    }
    let mut buf = [0; 1024];

    loop {
        // waiting for intent
        match socket.try_recv_from(&mut buf) {
            Ok((amt, src)) => {
                if src == tape {
                    let received_data = str::from_utf8(&buf[..amt]).expect("Failed to convert to string");
                    let m: Message = match serde_json::from_str(received_data) {
                        Ok(m) => m,
                        Err(e) => {
                            warn!("{:?}", e);
                            // TODO: try to parse
                            Message::new(MessageType::Unknow, "".to_string())
                        },
                    };
                    match m.get_type() {
                        MessageType::Intent => {
                            random_execute(&m.get_body()).await?;
                        },
                        _ => {
                            warn!("do not support such type: {}", m.get_body());
                            return Ok(());
                        }
                    }
                }
            },
            Err(e) => {
                // warn!("Failed to received from {}: {}, retry later", TAPE_ADDRESS, e);
                sleep(time::Duration::from_secs(1));
            },
                
        }
    }
}

const MY_SQL_DESCRIPTION: &str = "MySQL can store, organize, and manage data in structured tables. It allows users to create, read, update, and delete data using SQL queries. It supports data sorting, filtering, and searching, and can handle complex operations like joining multiple tables. MySQL ensures data integrity through constraints, transactions, and indexing. It can manage large datasets, support multiple users simultaneously, and provide secure access control. Additionally, it enables backups, replication, and scalability for growing applications.";

const MONGO_DB_DESCRIPTION: &str = "MongoDB is a NoSQL database that stores data in flexible, JSON-like documents instead of tables. It can handle unstructured or semi-structured data, making it ideal for dynamic or evolving data models. MongoDB allows you to store, query, and manage large volumes of data efficiently. It supports indexing for fast searches, horizontal scaling for handling big data, and replication for high availability. MongoDB also enables complex queries, aggregation, and real-time analytics, making it suitable for modern applications with diverse data needs.";

const GOO_GLE_DRIVE_DESCRIPTION: &str = "Google Drive is a cloud-based storage service that allows you to store, share, and access files from anywhere. It can store documents, photos, videos, and other file types, and sync them across devices. You can create and edit files using Google Workspace tools like Docs, Sheets, and Slides directly within Drive. It supports file sharing with customizable permissions, collaboration in real-time, and version history to track changes. Google Drive also provides search functionality to quickly find files and integrates with other Google services and third-party apps.";

const INTENT_INPUT_DESCRIPTION: &str = "Intent Input is a device which can get intent from user, but can not reveive any intent from other ways";
/*use tapeos::{
    components::linkhub::{seeker, waiter},
    tools::idgen::init_id_generator
};
use std::{
    thread, 
    sync::mpsc::{Sender, Receiver, channel}
};

const ENABLE_SEEK: bool = true;
const ENABLE_WAIT: bool = true;
const ENABLE_BOTH: bool = ENABLE_SEEK && ENABLE_WAIT;

fn main() {
    let mut seek_send: Option<Sender<String>> = None;
    let mut wait_send: Option<Sender<String>> = None;
    let mut seek_recv: Option<Receiver<String>> = None;
    let mut wait_recv: Option<Receiver<String>> = None;
    if ENABLE_BOTH {
        let (send, recv) = channel::<String>();
        (seek_send, wait_recv) = (Some(send), Some(recv));
        let (send, recv) = channel::<String>();
        (wait_send, seek_recv) = (Some(send), Some(recv));
    }
    match init_id_generator() {
        Ok(_) => (),
        Err(e) => {
            println!("Error initializing id generator: {}", e);
            return;
        }
    }

    let mut handles = vec![];
    if ENABLE_SEEK {
        handles.push(thread::spawn(move || {
            if ENABLE_BOTH {
                seeker::channel_init(seek_send, seek_recv);
            }
            let _ = seeker::seek();
        }));
    }

    if ENABLE_WAIT {
        handles.push(thread::spawn(move || {
            if ENABLE_BOTH {
                waiter::channel_init(wait_send, wait_recv);
            }
            let _ = waiter::wait();
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
 */