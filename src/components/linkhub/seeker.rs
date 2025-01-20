// in this file, we will implement the seeker, which is used to seek the 
// higher level system when not connected.

// we will support multiple seeker, and each seeker will use different strategy to seek the higher level system.
// now we will implement in three strategies:
// 1. bluetooth
// 2. wifi
// 3. internet

use std::{
    collections::HashMap, sync::{
        mpsc::{Receiver, Sender}, Arc
    }, time::Duration
};
use lazy_static::lazy_static;
use log::info;
use tokio::sync::Mutex;

use crate::{base::{errort::BoxResult, intent::Intent, message::{Message, MessageType}, resource::Status}, components::linkhub::{bluetooth, internet, wifi}};
use crate::base::resource::Resource;
use super::{bluetooth::resource::BluetoothResource, internet::{resource::InternetResource, seek::send_message_internet}, waiter::{ResourceType, BTAPE, ITAPE, TAPE}};

#[allow(dead_code)]
enum SeekMethod {
    Bluetooth,
    Wifi,
    Internet,
    RFID,
    NFC,
}

type ResourcePool<T> = Arc<Mutex<HashMap<String, Arc<Mutex<T>>>>>;
type Queue<T> = Mutex<Vec<T>>;

const SEEK_METHOD: SeekMethod = SeekMethod::Internet;
lazy_static! {
    // we use these resource seperately for different seeker.
    pub static ref INTERNET_RESOURCES: ResourcePool<InternetResource>= Arc::new(Mutex::new(HashMap::new()));
    pub static ref BLUETOOTH_RESOURCES: ResourcePool<BluetoothResource> = Arc::new(Mutex::new(HashMap::new()));
    // ...
    pub static ref INTENT_QUEUE: Queue<Intent> = Mutex::new(Vec::new());
    pub static ref RESPONSE_QUEUE: Queue<HashMap<String, String>> = Mutex::new(Vec::new());
    pub static ref SEEK_SEND: Mutex<Option<Sender<String>>> = Mutex::new(None);
    pub static ref SEEK_RECV: Mutex<Option<Receiver<String>>> = Mutex::new(None);
}

pub async fn channel_init(seek_send: Option<Sender<String>>, seek_recv: Option<Receiver<String>>) {
    SEEK_SEND.lock().await.replace(seek_send.unwrap());
    SEEK_RECV.lock().await.replace(seek_recv.unwrap());
}

// seek resources and subsystems depend on the SEEK_METHOD.
pub async fn seek() -> BoxResult<()> {
    match SEEK_METHOD {
        SeekMethod::Bluetooth => bluetooth::seek::seek(),
        SeekMethod::Wifi => wifi::seek::seek(),
        SeekMethod::Internet => internet::seek::seek().await,
        _ => {
            return Err("Unsupported seek method".into());
        }
    }
}

pub async fn get_all_resource_info() -> String {
    let mut resources_info = String::new();
    for (_, resource) in BLUETOOTH_RESOURCES.lock().await.iter() {
        let r = resource.lock().await;
        resources_info += format!("{}/{}/{};", r.get_name(), r.get_description(), r.display_status()).as_str();
    }
    for (_, resource) in INTERNET_RESOURCES.lock().await.iter() {
        let r = resource.lock().await;
        resources_info += format!("{}/{}/{};", r.get_name(), r.get_description(), r.display_status()).as_str();
    }

    resources_info
}

pub async fn get_resource_info(name: &str) -> String {
    match INTERNET_RESOURCES.lock().await.get(name) {
        Some(r) => {
            return format!("{}", r.lock().await);
        },
        None => (),
    }
    match BLUETOOTH_RESOURCES.lock().await.get(name) {
        Some(r) => {
            return format!("{}", r.lock().await);
        },
        None => (),
    } 
    "".to_string()
}

pub async fn get_resource_description(name: &str) -> String {
    match INTERNET_RESOURCES.lock().await.get(name) {
        Some(resource) => {
            return resource.lock().await.get_description().to_string();
        },
        None => (),
    }
    match BLUETOOTH_RESOURCES.lock().await.get(name) {
        Some(resource) => {
            return resource.lock().await.get_description().to_string();
        },
        None => (),
    } 
    "".to_string()
}

pub async fn fresh_resource_status(name: &str, s: Status) -> bool {
    match INTERNET_RESOURCES.lock().await.get(name) {
        Some(r) => {
            r.lock().await.set_status(s.clone());
        },
        None => (),
    }
    match BLUETOOTH_RESOURCES.lock().await.get(name) {
        Some(r) => {
            r.lock().await.set_status(s.clone());
        },
        None => (),
    } 
    false
}

// if op is true add one dealing
pub async fn change_resource_dealing(name: &str, op: bool) {

    match INTERNET_RESOURCES.lock().await.get(name) {
        Some(r) => {
            r.lock().await.get_status().change_dealing(op);
        },
        None => (),
    }
    match BLUETOOTH_RESOURCES.lock().await.get(name) {
        Some(r) => {
            r.lock().await.get_status().change_dealing(op);
        },
        None => (),
    } 
}

pub async fn add_resource_total_busy(name: &str, d: Duration) -> Duration {

    match INTERNET_RESOURCES.lock().await.get(name) {
        Some(r) => {
            return r.lock().await.get_status().add_total_busy(d);
        },
        None => (),
    }
    match BLUETOOTH_RESOURCES.lock().await.get(name) {
        Some(r) => {
            return r.lock().await.get_status().add_total_busy(d);
        },
        None => (),
    } 
    Duration::from_secs(0)
}

pub async fn get_resource_average_busy(name: &str) -> Duration {

    match INTERNET_RESOURCES.lock().await.get(name) {
        Some(r) => {
            return r.lock().await.get_status().get_average_time();
        },
        None => (),
    }
    match BLUETOOTH_RESOURCES.lock().await.get(name) {
        Some(r) => {
            return r.lock().await.get_status().get_average_time();
        },
        None => (),
    } 
    Duration::from_secs(0)
}


pub async fn calculate_base_dealing(name: &str) -> u64 {

    match INTERNET_RESOURCES.lock().await.get(name) {
        Some(r) => {

            let dealing = r.lock().await.get_status().get_dealing() as u64;
            let average_time = r.lock().await.get_status().get_average_time().as_secs();
            let busy_time = r.lock().await.get_status().get_busy_time().as_secs();
            return dealing*average_time + busy_time;
        },
        None => (),
    }
    match BLUETOOTH_RESOURCES.lock().await.get(name) {
        Some(r) => {
            let dealing = r.lock().await.get_status().get_dealing() as u64;
            let average_time = r.lock().await.get_status().get_average_time().as_secs();
            let busy_time = r.lock().await.get_status().get_busy_time().as_secs();
            return dealing*average_time + busy_time;
        },
        None => (),
    } 

    u64::MAX
}

pub async fn get_resource_status_str(name: &str) -> String {
    match INTERNET_RESOURCES.lock().await.get(name) {
        Some(resource) => {
            return resource.lock().await.display_status().to_string();
        },
        None => (),
    }
    match BLUETOOTH_RESOURCES.lock().await.get(name) {
        Some(resource) => {
            return resource.lock().await.display_status().to_string();
        },
        None => (),
    } 
    "".to_string()
}



async fn send_message_bluetooth(r: Arc<Mutex<BluetoothResource>>, i: &str, i_type: MessageType, id: Option<i64>) -> BoxResult<()> {
    let r = r.lock().await;
    let char = r.get_char().as_ref().unwrap();
    let reject = if r.is_interpreter_none() {
        let m = Message::new(i_type, i.to_string(), id);
        serde_json::to_string(&m)?
    } else {
        // TODO
        let id = if id.is_none() {""} else {&(id.unwrap().to_string() + ":")};
        i_type.to_string() + ":" + id + i
    };
    let data: Vec<u8> = reject.as_bytes().to_vec();
    char.write(&data).await?;
    info!("message send");
    Ok(())
}

pub async fn reject_intent(resource_name: String, intent: &str) -> BoxResult<()> {
    let r_m = INTERNET_RESOURCES.lock().await;
    let r = r_m.get(&resource_name);
    if r.is_some() {
        match send_message_internet(r.unwrap().lock().await, intent, MessageType::Reject, None).await {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
    }
    let _ = r;

    let r_m = BLUETOOTH_RESOURCES.lock().await;
    let r = r_m.get(&resource_name);
    if r.is_some() {
        match send_message_bluetooth(Arc::clone(r.unwrap()), intent, MessageType::Reject, None).await {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
    }
    let _ = r;
    
    if resource_name == "TAPE" {
        match TAPE.lock().await.copy() {
            ResourceType::Bluetooth => {
                match send_message_bluetooth(Arc::clone(BTAPE.lock().await.as_ref().unwrap()), intent, MessageType::Reject, None).await {
                    Ok(()) => (),
                    Err(e) => return Err(e),
                }
            },
            ResourceType::Internet => {
                match send_message_internet(ITAPE.lock().await.lock().await, intent, MessageType::Reject, None).await {
                    Ok(()) => (),
                    Err(e) => return Err(e),
                }
            },
            _ => (),
        }
    }
    
    Ok(())
}

pub async fn send_intent(resource_name: String, intent: &str, id: i64) -> BoxResult<()> {
    // println!("{resource_name}");
    let r_m = INTERNET_RESOURCES.lock().await;
    let r = r_m.get(&resource_name);
    if r.is_some() {
        match send_message_internet(r.unwrap().lock().await, intent, MessageType::Intent, Some(id)).await {
            Ok(()) => (),
            Err(e) => return Err(e),
        }
    }
    let _ = r;

    let r_m = BLUETOOTH_RESOURCES.lock().await;
    let r = r_m.get(&resource_name);
    if r.is_some() {
        match send_message_bluetooth(Arc::clone(r.unwrap()), intent, MessageType::Intent, Some(id)).await {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
    }
    let _ = r; 

    if resource_name == "TAPE" {
        match TAPE.lock().await.copy() {
            ResourceType::Bluetooth => {
                match send_message_bluetooth(Arc::clone(BTAPE.lock().await.as_ref().unwrap()), intent, MessageType::Intent, Some(id)).await {
                    Ok(()) => (),
                    Err(e) => return Err(e),
                }
            },
            ResourceType::Internet => {
                // TODO: may error here.
                match send_message_internet(ITAPE.lock().await.lock().await, intent, MessageType::Intent, Some(id)).await {
                    Ok(()) => (),
                    Err(e) => return Err(e),
                }
            },
            _ => (),
        }

    }
    
    Ok(())
}