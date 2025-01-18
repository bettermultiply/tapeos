use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, str, thread::sleep, time::{Duration, Instant}};

use log::{info, warn};
use tapeos::{
    base::{errort::BoxResult, message::{Message, MessageType}, resource::Status}, components::linkhub::internet::{resource::InternetResource, seek::{seek, NOW, TAPE_ADDRESS}, wait::wait}, core::inxt::intent::random_execute, tools::{idgen::init_id_generator, rserver::tape_server}
};
use tokio::net::UdpSocket;



#[tokio::main]
async fn main() {
    // println!("{:?}", Instant::now());
    info!("main: Try to execute intent");
    env_logger::init();
    init_id_generator();

    tokio::spawn(async {
        tape_server();
    });

    
    // let intent = Intent::new("store my name".to_string(), IntentSource::Resource, IntentType::Intent, None);
    // *NOW.lock().await = Instant::now();
    // for i in 0..10000 {
    //     tokio::spawn(async move {
    //         let s = format!("MySQL{i}");
    //         let _ = wait(s, MY_SQL_DESCRIPTION.to_string(), 9001+i).await;
    //     });
    // }
    tokio::spawn(async {
        let _ = wait("MySQL".to_string(), MY_SQL_DESCRIPTION.to_string(), 8001).await;
    });
    tokio::spawn(async {
        let _ = wait("MongoDB".to_string(), MONGO_DB_DESCRIPTION.to_string(), 8002).await;
    });
    tokio::spawn(async {
        let _ = wait("GooGle Drive".to_string(), GOO_GLE_DRIVE_DESCRIPTION.to_string(), 8003).await;
    });
    tokio::spawn(async move {
        let _ = wait("Intent input".to_string(), INTENT_INPUT_DESCRIPTION.to_string(), 8004).await;
    });
    tokio::spawn(async move {
        let _ = seek().await;
    });
    sleep(Duration::from_secs(1));
    sleep(Duration::from_secs(100));
    
    // intent::handler(intent).await;
    info!("main: Try ended");
}

#[allow(warnings)]
async fn send_intent(name: String, desc: String, port: u16) -> BoxResult<()>{
    
    let (socket, tape, tape_clone)=send_register(name, desc, port).await?;
    
    loop {

        let m = Message::new(MessageType::Intent, "store my name: BM".to_string(), None);
        match send_message(&socket, &tape_clone, m).await {
            Err(_e) => continue,
            _ => (),
        }
        loop {
            let content = "intent solve successfully";
            match recv_message(&socket, &tape, content).await {
                Ok(0) => {break},
                _ => (),
            }
            info!("Input didnt get info");
        }
        break;
    }
    info!("Input Over");
    Ok(())
}
#[allow(warnings)]
async fn register(name: String, desc: String, port: u16) -> BoxResult<()>{
    
    let (socket, tape, tape_clone)=send_register(name, desc, port).await?;

    let mut buf = [0; 1024];
    loop {
        // waiting for intent
        match socket.recv_from(&mut buf).await {
            Ok((amt, src)) => {
                if src != tape { continue; }

                let m: Message = parse_message(&buf[..amt]);
                match m.get_type() {
                    MessageType::Heartbeat => heart_beat_report(&socket, &tape).await?,
                    MessageType::Intent => {
                    random_execute(&m.get_body())?;
                        loop {
                            let m = Message::new(MessageType::Response, "Over".to_string(), m.get_id());
                            
                            match send_message(&socket, &tape_clone, m).await {
                                Err(_e) => continue,
                                _ => (),
                            }
                            info!("wait for success");
                            match recv_message(&socket, &tape, "Intent finish report successfully").await {
                                Ok(0) => {
                                    info!("Intent finish");
                                    break
                                },
                                _ => (),
                            }
                        }
                    },
                    _ => { warn!("do not support such intent: {} port: {}", m.get_body(), port); }
                }    
            },
            Err(e) => {
                warn!("Failed to received from {}: {}, retry later", TAPE_ADDRESS, e);
            },
        }
    }
}

async fn heart_beat_report(socket: &UdpSocket, tape: &SocketAddr) -> BoxResult<()>{
    let h = Message::new(MessageType::Heartbeat, "".to_string(), None);
    let h_json = serde_json::to_string(&h)?;
    info!("heart beat alive");
    socket.send_to(&h_json.as_bytes(), tape).await?;
    Ok(())
}

fn parse_message(v: &[u8]) -> Message {
    let received_data = str::from_utf8(v).expect("Failed to convert to string");
    match serde_json::from_str(received_data) {
        Ok(m) => m,
        Err(e) => {
            warn!("{:?}", e);
            // TODO: try to parse
            Message::new(MessageType::Unknown, "".to_string(), None)
        },
    }
}

async fn send_message(socket: &UdpSocket, tape_clone: &SocketAddr, m: Message) -> BoxResult<u8> {

    let m_json = serde_json::to_string(&m)?;

    match socket.send_to(&m_json.as_bytes().to_vec(), *tape_clone).await {
        Ok(_) => {
            info!("send message successfully: {}, {}", tape_clone.port(), m_json);
        },
        Err(e) => {
            warn!("Failed send to {}: {}, retry later", TAPE_ADDRESS, e);
            return Err(Box::new(e));
        }
    }

    Ok(0)
}

async fn send_register(name: String, desc: String, port: u16) -> BoxResult<(UdpSocket, SocketAddr, SocketAddr)> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let socket = UdpSocket::bind(addr).await.expect("Failed to bind to socket");

    let status = Status::new(true, (0.0, 0.0, 0.0), Duration::from_secs(0));
    let resource = InternetResource::new(name, desc, addr, status);
    let r_json = serde_json::to_string(&resource)?;

    let tape_clone = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8888);
    let tape = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8889);
    
    let m = Message::new(MessageType::Register, r_json, None);
    let m_json = serde_json::to_string(&m)?;

    loop {
        match socket.send_to(&m_json.as_bytes().to_vec(), tape_clone).await {
            Ok(r) => {
                info!("send register information successfully: {}", r);
            },
            Err(e) => {
                warn!("Failed to register to {}: {}, retry later", TAPE_ADDRESS, e);
            }
        }
        match recv_message(&socket, &tape, "register successfully").await {
            Ok(0) => break,
            _ => ()
        }
    }
    Ok((socket, tape, tape_clone))
}

async fn recv_message(socket: &UdpSocket, tape: &SocketAddr, content: &str) -> BoxResult<u8> {
    let mut buf = [0; 1024];
    match socket.recv_from(&mut buf).await {
        Ok((amt, src)) => {
            if src != *tape {warn!("source error");  return Ok(1); }

            let m: Message = parse_message(&buf[..amt]);
            match *m.get_type() {
                MessageType::Heartbeat => {heart_beat_report(&socket, &tape).await?;}
                MessageType::Response => {
                    
                    if m.get_body() == "Success" {
                        info!("{content}: {}", str::from_utf8(&buf[..amt]).expect("Fail to convert to String"));
                        return Ok(0);
                    } else if m.get_body() == "Registerd" {
                        info!("register successfully: {}", str::from_utf8(&buf[..amt]).expect("Fail to convert to String"));
                        return Ok(0);
                    } else {
                        warn!("do not support such response yet : {}", m.get_body());
                    }
                },
                MessageType::Intent => {
                    warn!("last work haven't over: {}", m.get_body());
                },
                _ => {
                    warn!("do not support such type: {}", m.get_body());
                },
            }
        },
        Err(e) => {
            warn!("Failed to received from {}: {}, retry later", TAPE_ADDRESS, e); 
        },
    }
    Ok(1)
}

const MY_SQL_DESCRIPTION: &str = "MySQL can store, organize, and manage data in structured tables. It allows users to create, read, update, and delete data using SQL queries. It supports data sorting, filtering, and searching, and can handle complex operations like joining multiple tables. MySQL ensures data integrity through constraints, transactions, and indexing. It can manage large datasets, support multiple users simultaneously, and provide secure access control. Additionally, it enables backups, replication, and scalability for growing applications.";
#[allow(warnings)]
const MONGO_DB_DESCRIPTION: &str = "MongoDB is a NoSQL database that stores data in flexible, JSON-like documents instead of tables. It can handle unstructured or semi-structured data, making it ideal for dynamic or evolving data models. MongoDB allows you to store, query, and manage large volumes of data efficiently. It supports indexing for fast searches, horizontal scaling for handling big data, and replication for high availability. MongoDB also enables complex queries, aggregation, and real-time analytics, making it suitable for modern applications with diverse data needs.";
#[allow(warnings)]
const GOO_GLE_DRIVE_DESCRIPTION: &str = "Google Drive is a cloud-based storage service that allows you to store, share, and access files from anywhere. It can store documents, photos, videos, and other file types, and sync them across devices. You can create and edit files using Google Workspace tools like Docs, Sheets, and Slides directly within Drive. It supports file sharing with customizable permissions, collaboration in real-time, and version history to track changes. Google Drive also provides search functionality to quickly find files and integrates with other Google services and third-party apps.";
#[allow(warnings)]
const INTENT_INPUT_DESCRIPTION: &str = "Intent Input is a device which can get intent from user, but can not reveive any intent from other ways";
