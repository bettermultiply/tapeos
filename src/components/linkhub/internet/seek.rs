// use std::time;
use std::{
    str,
    sync::Arc,
    sync::Mutex,
    error::Error,
    // thread::sleep,
    net::SocketAddr,
};
use log::{info, error, warn};
use tokio::{
    net::UdpSocket,
    time::{Duration, interval},
    sync::mpsc::{self, Receiver, Sender}
};
use lazy_static::lazy_static;
use crate::{
    base::{
        intent::{Intent, IntentSource, IntentType},
        message::{Message, MessageType},
        resource::Resource, 
    },
    components::linkhub::{
        internet::resource::InternetResource,
        seeker::{reject_intent, INTENT_QUEUE, INTERNET_RESOURCES},
    },
    core::inxt::{
        intent::handler, 
        preprocess::JudgeResult, 
        router::reroute
    },
};

macro_rules! get_udp {
    () => {
        SOCKET.lock().unwrap().as_ref().unwrap()
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

fn store_resource(resource: InternetResource) -> Option<()> {
    let mut i_rs = INTERNET_RESOURCES.lock().unwrap();
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

async fn complete_intent(intent: &mut Intent) -> Result<i64, Box<dyn Error>> {
    let intent_source = intent.get_resource().unwrap();
    let src = match INTERNET_RESOURCES.lock().unwrap().get(intent_source) {
        Some(resource) => resource.lock().unwrap().get_address().clone(),
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

async fn receive(tx: Sender<(String, SocketAddr)>) {
    tokio::spawn(async move {
        let socket_clone = UdpSocket::bind(TAPE_ADDRESS).await.expect("Failed to bind to socket");
        let mut buf = [0; 1024];
        loop {
            let (amt, src) = socket_clone.recv_from(&mut buf).await.expect("Failed to receive data");
            let received_data = str::from_utf8(&buf[..amt]).expect("Failed to convert to string");

            if tx.send((received_data.to_string(), src)).await.is_err() {
                error!("Failed to send message");
                break;
            }
        }
    });

}

fn parse_message(message: &str) -> Message{
    match serde_json::from_str(message) {
        Ok(m) => m,
        Err(e) => {
            warn!("{:?}", e);
            // TODO: try to parse
            Message::new(MessageType::Unknown, message.to_string(), None)
        },
    }
}

fn find_resource_by_addr(addr: &SocketAddr) -> Option<String> {
    for (_, i) in INTERNET_RESOURCES.lock().unwrap().iter() {
        let i_r = i.lock().unwrap();
        if i_r.get_address() == addr {
            return Some(i_r.get_name().to_string());
        }
    }
    // "Intent input".to_string()
    warn!("Outer intent");
    None
}

async fn mark_complete(sub_id: i64) ->Result<(), Box<dyn Error>> {
    let mut id = 0;
    for i in INTENT_QUEUE.lock().unwrap().iter_mut() {
        let mut c = false;
        for ii in i.iter_sub_intent() {
            if ii.get_id() != sub_id { continue; }
            ii.complete();
            c = true;
        }

        if c && i.is_complete() {
            complete_intent(i).await?;
            id = i.get_id();
            break;
        }
    }
    INTENT_QUEUE.lock().unwrap().retain(|i| i.get_id() != id);
    Ok(())
}

async fn message_handler(message: &str, src: SocketAddr) -> Result<(), Box<dyn Error>> {
    let m: Message = parse_message(&message);
    match m.get_type() {
        MessageType::Intent => {
            let r = find_resource_by_addr(&src);
            if r.is_none() { return Ok(()); }
            let intent = Intent::new(m.get_body(), IntentSource::Resource, IntentType::Intent, r.clone());
            info!("get intent: {}", intent.get_description());

            // tokio::spawn(async {
            // TODO make here multithread
            match handler(intent).await {
                JudgeResult::Reject(e) => reject_intent(r.unwrap(), &e).await?,
                _ => (),
            };
            // });
            info!("handler over");
        },
        MessageType::Register => {
            let r = message2resource(m.get_body())?;
            let m_body = match store_resource(r) {
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
            for i in INTENT_QUEUE.lock().unwrap().iter_mut() {
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

async fn send_heartbeat() -> Result<(), Box<dyn Error>> {
    for (name, resource) in INTERNET_RESOURCES.lock().unwrap().iter() {
        let r = resource.lock().unwrap();
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
                INTERNET_RESOURCES.lock().unwrap().remove(name);
                break;
            }
            // sleep(time::Duration::from_secs(1));
        }
    }
    Ok(())
}

pub async fn send_message_internet(r: Arc<std::sync::Mutex<InternetResource>>, i: &str, i_type: MessageType, id: Option<i64>) -> Result<(), Box<dyn Error>> {
    let r = r.lock().unwrap();
    let addr = r.get_address();
    let reject = if r.is_interpreter_none() {
        let m = Message::new(i_type, i.to_string(), id);
        serde_json::to_string(&m)?
    } else {
        let id = if id.is_none() {""} else {&(id.unwrap().to_string() + ":")};
        i_type.to_string() + ":" + id + i
    };
    let data: Vec<u8> = reject.as_bytes().to_vec();
    get_udp!().send_to(&data, addr).await?;
    info!("message send");
    Ok(())
}

async fn response(mut rx: Receiver<(String, SocketAddr)>) -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind("127.0.0.1:8889").await.expect("Failed to bind to socket");
    SOCKET.lock().unwrap().replace(socket);

    // every 60 seconds, check if the socket addresses are still valid
    let mut heartbeat_inter = interval(Duration::from_secs(200));
    let mut reroute_inter = interval(Duration::from_secs(60));

    loop {
        tokio::select! {
            Some((message, src)) = rx.recv() => {
                info!("Receive message from: {}", src);
                message_handler(&message, src).await?;
            },
            _ = reroute_inter.tick() => {
                // TODO reroute the intent if didn't finish in time
                
            },
            // heartbeat
            _ = heartbeat_inter.tick() => {
                // Check stored socket addresses are still valid
                info!("sending heart beat to check whether resource alive.");
                send_heartbeat().await?;
            }
        }
    }    
}