// use std::time;
use std::{
    error::Error, net::SocketAddr, str, sync::Arc, time::Instant
};
use log::{info, error, warn};
use tokio::{
    net::UdpSocket,
    time::{Duration, interval},
    sync::Mutex,
    sync::mpsc::{self, Receiver, Sender}
};
use lazy_static::lazy_static;
use crate::{
    base::{
        errort::BoxResult, intent::{Intent, IntentSource, IntentType}, message::{Message, MessageType}, resource::{Interpreter, Position, Resource} 
    },
    components::linkhub::{
        internet::resource::InternetResource,
        seeker::{reject_intent, INTENT_QUEUE, INTERNET_RESOURCES},
    },
    core::inxt::{
        intent::handler, 
        preprocess::JudgeResult, 
        router::reroute
    }, tools::llmq,
};

macro_rules! get_udp {
    () => {
        SOCKET.lock().await.as_ref().unwrap()
    }
}

pub const TAPE_ADDRESS: &str = "127.0.0.1:8888";
lazy_static! {
    pub static ref SOCKET: Mutex<Option<UdpSocket>> = Mutex::new(None);
}

pub async fn seek() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<(String, SocketAddr)>(32);

    receive(tx).await;
    response(rx).await
}

// act as a listener to receive message
async fn receive(tx: Sender<(String, SocketAddr)>) {
    tokio::spawn(async move {
        let socket = UdpSocket::bind(TAPE_ADDRESS).await.expect("Failed to bind to socket");
        let mut buf = [0; 1024];
        loop {
            let (amt, src) = socket.recv_from(&mut buf).await.expect("Failed to receive data");
            let received_data = str::from_utf8(&buf[..amt]).expect("Failed to convert to string");

            if tx.send((received_data.to_string(), src)).await.is_err() {
                error!("Failed to send message");
                break;
            }
        }
    });
}

async fn response(mut rx: Receiver<(String, SocketAddr)>) -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind("127.0.0.1:8889").await.expect("Failed to bind to socket");
    SOCKET.lock().await.replace(socket);

    // every 60 seconds, check if the socket addresses are still valid
    let mut heartbeat_inter = interval(Duration::from_secs(200));
    let mut reroute_inter = interval(Duration::from_secs(10));
    let mut status_inter = interval(Duration::from_secs(60));

    loop {
        tokio::select! {
            Some((message, src)) = rx.recv() => {
                // handler message from resources.
                info!("Receive message from: {}", src);
                message_handler(&message, src).await?;
            },
            _ = reroute_inter.tick() => {
                const EXPIRE_D: Duration = Duration::from_secs(60);
                for i in INTENT_QUEUE.lock().await.iter_mut() {
                    for s_i in i.iter_sub_intent() {
                        let live = Instant::now() - s_i.get_routed();
                        if live > EXPIRE_D {
                            reroute(s_i).await?;
                        }
                    }
                }
            },
            // heartbeat
            _ = heartbeat_inter.tick() => {
                // Check stored socket addresses are still valid
                info!("sending heart beat to check whether resource alive.");
                send_heartbeat().await?;
            },
            _ = status_inter.tick() => {
                query_status().await?;
                check_status().await;
            },
        }
    }    
}


async fn message_handler(message: &str, src: SocketAddr) -> Result<(), Box<dyn Error>> {
    // parse the message into available format
    let m: Message = parse_message(&message);
    match m.get_type() {
        MessageType::Intent => {
            let r = find_resource_by_addr(&src).await;
            
            // if resource haven't register, reject it.
            if r.is_none() { 
                let s = "Register First".to_string();
                let m = Message::new(MessageType::Response, s, m.get_id());
                let m_json = serde_json::to_string(&m)?;
                let data: Vec<u8> = m_json.as_bytes().to_vec();
                get_udp!().send_to(&data, src).await?;
                return Ok(());
            }
            // init intent
            let mut intent = Intent::new(m.get_body(), IntentSource::Resource, IntentType::Intent, r.clone());
            if m.get_id().is_some() {
                intent.set_id(m.get_id().unwrap());
            }
            info!("get intent: {}", intent.get_description());

            tokio::spawn(async {
                match handler(intent).await {
                    JudgeResult::Reject(e) => reject_intent(r.unwrap(), &e).await.unwrap(),
                    _ => (),
                };
                info!("handler over");
            });
        },
        MessageType::Register => {
            let r = message2resource(m.get_body())?;
            let m_body = match store_resource(r).await {
                Some(_) => "Success",
                None => "Duplicate"
            };
            let m = Message::new(MessageType::Response, m_body.to_string(), None);
            let m_json = serde_json::to_string(&m)?;
            info!("send to src: {}", src);
            
            get_udp!().send_to(&m_json.as_bytes().to_vec(), src).await?;
        },
        MessageType::Response => {
            info!("Get Response: {}", m.get_body());
            mark_complete(m.get_id().unwrap_or(0)).await?;
            
            let m = Message::new(MessageType::Response, "Success".to_string(), None);
            let m_json = serde_json::to_string(&m)?;
            get_udp!().send_to(&m_json.as_bytes().to_vec(), src).await?;
            info!("Send Over: {}", m.get_body());
        },
        MessageType::Reject => {
            let id = m.get_id().unwrap();
            for i in INTENT_QUEUE.lock().await.iter_mut() {
                for ii in i.iter_sub_intent() {
                    if ii.get_id() != id { continue; }
                    match reroute(ii).await {
                        Ok(_) => (),
                        Err(_) => {
                            // TODO
                        },
                    }
                }
            }   
        },
        _ => {
            warn!("no such type");
        }
    }
    Ok(())
}

// assume the message is a Message Serilization if not try to parse it.
fn parse_message(message: &str) -> Message{
    match serde_json::from_str(message) {
        Ok(m) => m,
        Err(e) => {
            warn!("{:?}", e);
            fn parse_unknown(m: &str) -> Message {
                let mut m_type: MessageType = MessageType::Unknown;
                if m.contains("Intent") { m_type = MessageType::Intent }
                else if  m.contains("Reponse") { m_type = MessageType::Response }
                else if  m.contains("Register") { m_type = MessageType::Register }
                else if  m.contains("Reject") { m_type = MessageType::Reject }
                else if  m.contains("Finish") { m_type = MessageType::Finish }

                Message::new(m_type, m.to_string(), None)
            }
            parse_unknown(message)
        },
    }
}

async fn store_resource(resource: InternetResource) -> Option<()> {
    let mut i_rs = INTERNET_RESOURCES.lock().await;
    let name = resource.get_name();
    if i_rs.get(name).is_some() {return None;}
    
    info!("store internet resource: {}", name);
    i_rs.insert(name.to_string(), Arc::new(Mutex::new(resource)));
    Some(())
}

fn message2resource(message: String) -> Result<InternetResource, Box<dyn Error>> {
    let resource: InternetResource = serde_json::from_str(&message)?;
    Ok(resource)
}

pub async fn complete_intent(intent: &mut Intent) -> Result<i64, Box<dyn Error>> {
    let intent_source = intent.get_resource().unwrap();
    let src = match INTERNET_RESOURCES.lock().await.get(intent_source) {
        Some(resource) => resource.lock().await.get_address().clone(),
        None => {
            warn!("resource have been removed");
            return Ok(0);
        },
    };
    let m = Message::new(MessageType::Response, "Success".to_string(), None);
    let m_json = serde_json::to_string(&m)?;
    get_udp!().send_to(&m_json.as_bytes().to_vec(), src).await?;
    intent.complete();
    Ok(intent.get_id())
}

async fn find_resource_by_addr(addr: &SocketAddr) -> Option<String> {
    for (_, i) in INTERNET_RESOURCES.lock().await.iter() {
        let i_r = i.lock().await;
        if i_r.get_address() == addr {
            return Some(i_r.get_name().to_string());
        }
    }
    // "Intent input".to_string()
    warn!("Outer intent");
    None
}

async fn mark_complete(sub_id: i64) ->Result<(), Box<dyn Error>> {
    // let mut id = 0;
    for i in INTENT_QUEUE.lock().await.iter_mut() {
        let mut c = false;
        for ii in i.iter_sub_intent() {
            if ii.get_id() != sub_id { continue; }
            ii.complete();
            c = true;
        }

        if c {
            // if i.is_complete() {
                // complete_intent(i).await?;
                // id = i.get_id();
            // }
            break;
        }
    }
    // INTENT_QUEUE.lock().await.retain(|i| i.get_id() != id);
    Ok(())
}

async fn send_heartbeat() -> Result<(), Box<dyn Error>> {
    for (name, resource) in INTERNET_RESOURCES.lock().await.iter() {
        let r = resource.lock().await;
        let address = r.get_address();
        let m = Message::new(MessageType::Heartbeat, "".to_string(), None);
        let m_json = serde_json::to_string(&m)?;
        match get_udp!().try_send_to(&m_json.as_bytes(), *address) {
            Ok(_) => info!("Heartbeat sent to {}", address),
            Err(e) => warn!("Failed to send heartbeat to {}: {}", address, e),
        }
        let mut retry = 3;
        let mut heart_buf = [0; 1024];
        
        loop {
            match get_udp!().recv_from(&mut heart_buf).await {
                Ok(_) => {
                    info!("resource <{}> alive!", address);
                    break;
                },
                Err(_) => warn!("heartbeat of {} retry remain {}", address, retry),
            }
            retry -= 1;
            if retry == 0 {
                warn!("resource of {} disappear", address);
                warn!("remove resource {} ", name);
                INTERNET_RESOURCES.lock().await.remove(name);
                break;
            }
            // sleep(time::Duration::from_secs(1));
        }
    }
    Ok(())
}

async fn interpret_intent(interpreter: &Interpreter, i: &str) -> String {
    // match 
    let command = match interpreter {
        Interpreter::LLM(s) => {
            let s_prompt = format!("the target commands are {s}, and you have to base on the intent given to choose one to execute");
            let u_prompt = format!("intent: {i}");
            llmq::prompt(&s_prompt, &u_prompt).await
        },
        Interpreter::PathBuf(_p) => {
            "".to_string()
        },
        _ => {
            "".to_string()
        },
    };
    command
}

pub async fn send_message_internet(r: Arc<Mutex<InternetResource>>, i: &str, m_type: MessageType, id: Option<i64>) -> Result<(), Box<dyn Error>> {
    let r = r.lock().await;
    let addr = r.get_address();
    let message = if r.is_interpreter_none() {
        let m = Message::new(m_type, i.to_string(), id);
        serde_json::to_string(&m)?
    } else {
        interpret_intent(r.get_interpreter(), i).await
    };
    let data: Vec<u8> = message.as_bytes().to_vec();
    get_udp!().send_to(&data, addr).await?;
    info!("message send");
    Ok(())
}

async fn query_status() -> BoxResult<()> {
    let m = Message::new(MessageType::Status, "".to_string(), None);
    let m_json = serde_json::to_string(&m)?;
    for s in INTERNET_RESOURCES.lock().await.values() {
        let addr = s.lock().await.get_address().clone();
        let buf = &m_json.as_bytes().to_vec();
        SOCKET.lock().await.as_ref().unwrap().send_to(buf, addr).await?;
    }
    Ok(())
}

async fn check_status() {
    for s in INTERNET_RESOURCES.lock().await.values() {
        let mut s = s.lock().await;
        let status = s.get_status();
        check_position(&status.get_position());
    }
}

fn check_position(p: &Position) -> bool {
    let v_position = ((-100.0, 100.0), (-100.0, 100.0), (-100.0, 100.0));
        p.x > (v_position.0).0 
    &&  p.x < (v_position.0).1
    &&  p.y > (v_position.1).0
    &&  p.y < (v_position.1).1
    &&  p.z > (v_position.2).0
    &&  p.z < (v_position.2).1
}