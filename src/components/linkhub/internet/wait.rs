
use std::{
    str, sync::Arc, 
    thread::sleep, time,
    net::{IpAddr, Ipv4Addr, SocketAddr}, 
};
use tokio::{net::UdpSocket, sync::Mutex, time::interval};
use log::{info, warn};

use crate::{
    base::{
        errort::BoxResult, intent::{
            Intent, IntentSource, IntentType
        }, 
        message::{Message, MessageType}, 
        resource::{RegisterServer, ResourceType, Status}
    }, 
    components::linkhub::{
        internet::{
            resource::InternetResource, 
            seek::TAPE_ADDRESS
        }, 
        waiter::{ITAPE, TAPE, TAPE_INTENT_QUEUEUE}
    }, 
    core::inxt::intent::{execute, handler}
};



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

    let (
        socket, 
        input_socket,
        m_json
    ) = init(name.clone(), desc, port).await?;

    let tape_i: Arc<Mutex<Option<SocketAddr>>> = Arc::new(Mutex::new(None));
    let tape_o: Arc<Mutex<Option<SocketAddr>>> = Arc::new(Mutex::new(None));
    let socket = Arc::new(socket);
    let status = Arc::new(Mutex::new(Status::new(true, (0.0, 0.0, 0.0), time::Duration::from_secs(0))));

    let v_position: ((f32, f32), (f32, f32), (f32, f32)) = ((0.0, 0.0), (0.0, 0.0), (0.0, 0.0));
    find_register(&socket, false, v_position).await; 
    
    let mut register = interval(time::Duration::from_secs(10));
    let mut check_register = interval(time::Duration::from_secs(40)); // check register must be slower than heart beat
    let _ = check_register.tick();
    let mut input_buf = [0; 1024];
    loop {
        let mut buf = [0; 1024];
        // waiting for intent
        tokio::select! {
            _ = register.tick(), if TAPE.lock().await.is_none() => {
                find_register(&socket, false, v_position).await; 
            },
            _ = check_register.tick(), if !TAPE.lock().await.is_none() => {
                // if *HEART.lock().await {
                //     *HEART.lock().await = false;
                //     continue;
                // }
                // warn!("no heart beat, disconnect");
                // *TAPE.lock().await = ResourceType::None;
                // *tape_i.lock().await = None;
                // *tape_o.lock().await = None;
            },
            Ok((amt, _))  = input_socket.recv_from(&mut input_buf) => {
                match str::from_utf8(&input_buf[0..amt]) {
                    Ok(m_body) => {
                        if tape_i.lock().await.is_none() {
                            warn!("send to seeker please");
                        }

                        let i = Intent::new(m_body.to_string(), IntentSource::Input, IntentType::Intent, None);
                        // info!("send message {m_body}");
                        
                        // we only send plain text intent so that the bandwidth cost will reduce
                        let m = Message::new(MessageType::Intent, m_body.to_string(), Some(i.get_id()));
                        match send_message(&socket, &tape_i.lock().await.unwrap(), &m).await {
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
                let c_tape_i = Arc::clone(&tape_i);
                let c_tape_o = Arc::clone(&tape_o);
                let c_socket = Arc::clone(&socket);
                let c_status = Arc::clone(&status);
                let c_m_json = m_json.clone();
                tokio::spawn(async move{
                    let _ = message_handler(src, amt, c_tape_i, c_tape_o, &buf, c_socket, c_m_json, c_status).await;
                });
            }
        }
    }
}

async fn find_register(socket: &UdpSocket, tape: bool, position: ((f32, f32), (f32, f32), (f32, f32))) {
    let v4: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
    let ipv4 = IpAddr::V4(v4);
    let r = RegisterServer::new(tape, None, None, position);
    let r_json = serde_json::to_string(&r).unwrap();
    let addr = SocketAddr::new(ipv4, 8000);
    socket.send_to(&r_json.as_bytes(), addr).await.unwrap();
} 

async fn message_handler(
    src: SocketAddr, 
    amt: usize, 
    tape_i: Arc<Mutex<Option<SocketAddr>>>, 
    tape_o: Arc<Mutex<Option<SocketAddr>>>,
    buf: &[u8],
    socket: Arc<UdpSocket>,
    m_json: String,
    status: Arc<Mutex<Status>>,
) -> BoxResult<()> {
    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000);
    if src == server_addr {
        let data = str::from_utf8(&buf[..amt]).unwrap();
        let tape: RegisterServer = serde_json::from_str(data)?;
        *tape_i.lock().await = Some(tape.get_iaddr().clone());
        *tape_o.lock().await = Some(tape.get_oaddr().clone());
        info!("tape o is ready: {}", tape_o.lock().await.unwrap().port());
        send_register(&socket, &tape.get_iaddr(), &m_json).await;
        ITAPE.lock().await.set_address(tape.get_iaddr().clone());
        return Ok(());
    }
    if tape_o.lock().await.is_none() {
        warn!("haven't regiterd");
        return Ok(());
    }
    if src != tape_o.lock().await.expect("error") { 
        warn!("source error");
        return Ok(());
    } // waiter only accept message from Tape

    let m: Message = parse_message(&buf[..amt]);
    info!("get {}", m.get_body());
    match m.get_type() {
        MessageType::Status => {
            status_report(&socket, &tape_i.lock().await.unwrap(), Arc::clone(&status)).await?
        }
        MessageType::Heartbeat 
        => {
            heart_beat_report(&socket, &tape_o.lock().await.unwrap()).await?;
            // *HEART.lock().await = true;
        },
        MessageType::Finish => {
            *TAPE.lock().await = ResourceType::None;
            *tape_i.lock().await = None;
            *tape_o.lock().await = None;
        }
        MessageType::Response => {
            match m.get_body().as_ref() {
                "Registerd" => {
                    *TAPE.lock().await = ResourceType::Internet;
                    info!("register successfully: {}", str::from_utf8(&buf[..amt]).expect("Fail to convert to String"));
                },
                "Intent Duplicate" => {
                    
                },
                "Intent Received" => {
                    
                },
                "Finish Received" => {
                    info!("Finish Received get");
                    // intent finish.
                },
                "Finish" => {
                    let id = m.get_id().unwrap();
                    let mut queue = TAPE_INTENT_QUEUEUE.lock().await; 
                    let index = queue.iter().position(|i| i.get_id() == id);
                    if index.is_none() {
                        warn!("not intent here");
                        return Ok(());
                    }
                    let intent = queue.remove(index.unwrap());
                    info!("OKOK Intent {} finished", intent.get_description());
                },
                "Duplicate" => {
                    
                },
                "Register First" => {
                    // knowning the connect broken.
                    *TAPE.lock().await = ResourceType::None;
                    *tape_i.lock().await = None;
                    *tape_o.lock().await = None;
                },
                _ => {
                    warn!("Do not support such response now. {}", m.get_body());
                },
            }
        },
        MessageType::Intent 
        => {
            let c_socket = Arc::clone(&socket);
            let c_status = Arc::clone(&status);
            let c_tape_i = tape_i.clone();
            let c_m = m.get_body().clone();
            let m_id = m.get_id().clone();
            tokio::spawn(async move {
            
                if END {
                    let _ = execute(&c_m, c_status).await.unwrap();
                } else {
                    let i: Intent = Intent::new(c_m.clone(), IntentSource::Tape, IntentType::Intent, Some("TAPE".to_string()));
                    handler(i).await;
                }
                let m = Message::new(MessageType::Response, "Execute Over".to_string(), m_id);
                // let addr = WAIT_EXEC_ADDR.lock();
                // let s = UdpSocket::bind(addr.await.clone()).await.unwrap();
                loop {
                    match send_message(&c_socket, &c_tape_i.lock().await.unwrap(), &m).await {
                        Ok(_) => {
                            // let _ = s;
                            // let _ = addr;
                            break;
                        },
                        Err(_e) => (),
                    }
                }
            }); 
        },
        _ => { warn!("do not support such intent: {} ", m.get_body()); }
    }
    Ok(())
}

async fn init(name: String, desc: String, port: u16) -> BoxResult<(UdpSocket, UdpSocket, String)> {
    // let tape_o = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8889);
    // let tape_i = SocketAddr::new(IpA/ddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8888);
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let socket = UdpSocket::bind(addr).await.expect("Failed to bind to socket");
    let input_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port+20000);
    let input_socket = UdpSocket::bind(input_addr).await.expect("Failed to bind to socket");

    let status = Status::new(true, (0.0, 0.0, 0.0), time::Duration::from_secs(0));
    let resource = InternetResource::new(name, desc, addr, status);
    let r_json = serde_json::to_string(&resource)?;

    let m = Message::new(MessageType::Register, r_json, None);
    let m_json = serde_json::to_string(&m)?;

    Ok((socket, input_socket, m_json))
}

async fn send_register(s: &UdpSocket, tape_i: &SocketAddr, m_json: &str) {
    match s.send_to(&m_json.as_bytes().to_vec(), tape_i).await {
        Ok(_) => (),
        Err(e) => {
            warn!("Failed to register to {}: {}, retry later", TAPE_ADDRESS, e);
        }
    }
    sleep(time::Duration::from_micros(100));
}

async fn heart_beat_report(socket: &UdpSocket, tape_o: &SocketAddr) -> BoxResult<()>{
    let h = Message::new(MessageType::Heartbeat, "".to_string(), None);
    let h_json = serde_json::to_string(&h)?;
    // info!("heart beat alive");
    socket.send_to(&h_json.as_bytes(), tape_o).await?;
    Ok(())
}

async fn status_report(socket: &UdpSocket, tape_i: &SocketAddr, status: Arc<Mutex<Status>>) -> BoxResult<()>{
    let s = status.lock().await.clone();
    let s_json = serde_json::to_string(&s)?;
    let h = Message::new(MessageType::Intent, s_json, None);
    let h_json = serde_json::to_string(&h)?;
    socket.send_to(&h_json.as_bytes(), tape_i).await?;
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

async fn send_message(socket: &UdpSocket, tape_i: &SocketAddr, m: &Message) -> BoxResult<()> {
    let m_json = serde_json::to_string(m)?;

    match socket.send_to(&m_json.as_bytes().to_vec(), *tape_i).await {
        Ok(_) => {
            // info!("send message successfully: {}, {}", tape_i.port(), m_json);
        },
        Err(e) => {
            warn!("Failed send to {}: {}, retry later", TAPE_ADDRESS, e);
            return Err(Box::new(e));
        }
    }

    Ok(())
}

