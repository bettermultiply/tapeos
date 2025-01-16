
use std::{error::Error, net::{IpAddr, Ipv4Addr, SocketAddr}, str, thread::sleep, time};

use log::{info, warn};
use crate::{
    base::{errort::BoxResult, intent::{Intent, IntentSource, IntentType}, message::{Message, MessageType}, resource::Status}, components::linkhub::{internet::{resource::InternetResource, seek::TAPE_ADDRESS}, waiter::{TAPE, TAPE_INTENT_QUEUEUE}}, core::inxt::intent::{execute, handler}
};

use tokio::{net::UdpSocket, time::interval};

const NAME: &str = "";
const DESCRIPTION: &str = "";
const PORT: u16 = 8080;
const END: bool = true;

pub async fn wait(mut name: String, mut desc: String, mut port: u16) -> BoxResult<()> {
    if name.is_empty() {
        name = NAME.to_string();
    }
    if desc.is_empty() {
        desc = DESCRIPTION.to_string();
    }
    if port == 0 {
        port = PORT;
    }
    // TODO:
    // 1. 发现 外部seeker 的广播，并建立连接
    // 2. 监听来自 外部seeker 的请求和消息
    // 3. 接发 内部seeker 的信息
    let (
        socket, 
        input_socket,
        tape, 
        tape_clone, 
        m_json
    ) = init(name, desc, port).await?;
    send_register(&socket, &tape_clone, &m_json).await;
    
    let mut register = interval(time::Duration::from_secs(10));

    let mut buf = [0; 1024];
    let mut input_buf = [0; 1024];
    loop {
        // waiting for intent
        tokio::select! {
            _ = register.tick(), if TAPE.lock().await.is_none() => {
                send_register(&socket, &tape_clone, &m_json).await;
            },
            Ok((amt, _))  = input_socket.recv_from(&mut input_buf) => {
                match str::from_utf8(&buf[0..amt]) {
                    Ok(m_body) => {
                        let i = Intent::new(m_body.to_string(), IntentSource::Input, IntentType::Intent, None);
                        
                        // we only send plain text intent so that the bandwidth cost will reduce
                        let m = Message::new(MessageType::Intent, m_body.to_string(), Some(i.get_id()));
                        match send_message(&socket, &tape_clone, &m).await {
                            Ok(_) => {
                                TAPE_INTENT_QUEUEUE.lock().await.push(i);
                            },
                            Err(_e) => (),
                        }
                    },
                    _ => {
                        warn!("Expect utf-8 String")
                    },
                } 
            }
            Ok((amt, src)) = socket.recv_from(&mut buf) => {
                if src != tape { 
                    warn!("source error");
                    continue; 
                } // waiter only accept message from Tape
                
                let m: Message = parse_message(&buf[..amt]);
                match m.get_type() {
                    MessageType::Status => {
                        status_report(&socket, &tape).await?
                    }
                    MessageType::Heartbeat 
                    => heart_beat_report(&socket, &tape).await?,
                    MessageType::Finish => {
                        let id = m.get_id().unwrap();
                        let mut queue = TAPE_INTENT_QUEUEUE.lock().await; 
                        let index = queue.iter().position(|i| i.get_id() == id);
                        if index.is_none() {
                            warn!("not intent here");
                            continue;
                        }
                        let intent = queue.remove(index.unwrap());
                        info!("Intent {} finished", intent.get_description());
                    }
                    MessageType::Response => {
                        match m.get_body().as_ref() {
                            "Registerd" => {

                            },
                            "Intent Duplicate" => {

                            },
                            "Intent Received" => {
                            
                            },
                            "Finish Received" => {
                                // intent finish.
                            },
                            "Register First" => {

                            }
                            _ => {
                                warn!("Do not support such response now.");
                            },
                        }
                    },
                    MessageType::Intent 
                    => {
                        if END {
                            execute(&m.get_body())?;
                        } else {
                            let i: Intent = Intent::new(m.get_body().clone(), IntentSource::Tape, IntentType::Intent, Some("TAPE".to_string()));
                            tokio::spawn(async {
                                handler(i).await;
                            }); 
                        }

                        let m = Message::new(MessageType::Response, "Over".to_string(), m.get_id());
                        loop {
                            match send_message(&socket, &tape_clone, &m).await {
                                Ok(_) => break,
                                Err(_e) => (),
                            }
                        }
                    },
                    _ => { warn!("do not support such intent: {} port: {}", m.get_body(), port); }
                }
            },
        }
    }
    
}


async fn init(name: String, desc: String, port: u16) -> BoxResult<(UdpSocket, UdpSocket, SocketAddr, SocketAddr, String)> {
    let tape = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8889);
    let tape_clone = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8888);
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let socket = UdpSocket::bind(addr).await.expect("Failed to bind to socket");
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port+100);
    let input_socket = UdpSocket::bind(addr).await.expect("Failed to bind to socket");

    let status = Status::new(true, (0.0, 0.0, 0.0), time::Duration::from_secs(0));
    let resource = InternetResource::new(name, desc, addr, status);
    let r_json = serde_json::to_string(&resource)?;

    let m = Message::new(MessageType::Register, r_json, None);
    let m_json = serde_json::to_string(&m)?;

    Ok((socket, input_socket, tape, tape_clone, m_json))
}

async fn send_register(s: &UdpSocket, tape_clone: &SocketAddr, m_json: &str) {
    match s.send_to(&m_json.as_bytes().to_vec(), tape_clone).await {
        Ok(r) => {
            info!("send register information successfully: {}", r);
        },
        Err(e) => {
            warn!("Failed to register to {}: {}, retry later", TAPE_ADDRESS, e);
        }
    }
    sleep(time::Duration::from_micros(100));
}

async fn heart_beat_report(socket: &UdpSocket, tape: &SocketAddr) -> Result<(), Box<dyn Error>>{
    let h = Message::new(MessageType::Heartbeat, "".to_string(), None);
    let h_json = serde_json::to_string(&h)?;
    info!("heart beat alive");
    socket.send_to(&h_json.as_bytes(), tape).await?;
    Ok(())
}

async fn status_report(socket: &UdpSocket, tape: &SocketAddr) -> Result<(), Box<dyn Error>>{
    let s = Status::new(true, (0.0, 0.0, 0.0), time::Duration::from_secs(0));
    let s_json = serde_json::to_string(&s)?;
    let h = Message::new(MessageType::Intent, s_json, None);
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

async fn send_message(socket: &UdpSocket, tape_clone: &SocketAddr, m: &Message) -> Result<(), Box<dyn Error>> {
    let m_json = serde_json::to_string(m)?;

    match socket.send_to(&m_json.as_bytes().to_vec(), *tape_clone).await {
        Ok(_) => {
            info!("send message successfully: {}, {}", tape_clone.port(), m_json);
        },
        Err(e) => {
            warn!("Failed send to {}: {}, retry later", TAPE_ADDRESS, e);
            return Err(Box::new(e));
        }
    }

    Ok(())
}

