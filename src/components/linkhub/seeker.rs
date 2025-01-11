// in this file, we will implement the seeker, which is used to seek the 
// higher level system when not connected.

// we will support multiple seeker, and each seeker will use different strategy to seek the higher level system.
// now we will implement in three strategies:
// 1. bluetooth
// 2. wifi
// 3. internet

use std::{
    error::Error, 
    sync::{
        Arc, Mutex,
        mpsc::{Sender, Receiver}
    }, 
    collections::HashMap
};
use lazy_static::lazy_static;
use log::info;

use crate::{base::{intent::Intent, message::{Message, MessageType}}, components::linkhub::{bluetooth, internet, wifi}};
use crate::base::resource::Resource;
use super::{bluetooth::resource::BluetoothResource, internet::{resource::InternetResource, seek::SOCKET}, waiter::{ResourceType, TAPE}};

#[allow(dead_code)]
enum SeekMethod {
    Bluetooth,
    Wifi,
    Internet,
    RFID,
    NFC,
}

const SEEK_METHOD: SeekMethod = SeekMethod::Internet;
lazy_static! {
    // we use these resource seperately for different seeker.
    // pub static ref RESOURCES: Mutex<Vec<Arc<ResourceType>>> = Mutex::new(Vec::new());
    pub static ref INTERNET_RESOURCES: Mutex<HashMap<String, Arc<InternetResource>>> = Mutex::new(HashMap::new());
    pub static ref BLUETOOTH_RESOURCES: Mutex<HashMap<String, Arc<BluetoothResource>>> = Mutex::new(HashMap::new());
    // ...
    pub static ref SUBINTENT_QUEUE: Mutex<Vec<Intent>> = Mutex::new(Vec::new());
    pub static ref INTENT_QUEUE: Mutex<Vec<Intent>> = Mutex::new(Vec::new());
    pub static ref RESPONSE_QUEUE: Mutex<Vec<HashMap<String, String>>> = Mutex::new(Vec::new());
    pub static ref SEEK_SEND: Mutex<Option<Sender<String>>> = Mutex::new(None);
    pub static ref SEEK_RECV: Mutex<Option<Receiver<String>>> = Mutex::new(None);
}

pub fn channel_init(seek_send: Option<Sender<String>>, seek_recv: Option<Receiver<String>>) {
    SEEK_SEND.lock().unwrap().replace(seek_send.unwrap());
    SEEK_RECV.lock().unwrap().replace(seek_recv.unwrap());
}

// seek resources and subsystems depend on the SEEK_METHOD.
pub async fn seek() -> Result<(), Box<dyn Error>> {
    match SEEK_METHOD {
        SeekMethod::Bluetooth => bluetooth::seek::seek(),
        SeekMethod::Wifi => wifi::seek::seek(),
        SeekMethod::Internet => internet::seek::seek().await,
        _ => {
            return Err("Unsupported seek method".into());
        }
    }
}

pub fn get_all_resource_info() -> String {
    let mut resources_info = String::new();
    for (_, resource) in BLUETOOTH_RESOURCES.lock().unwrap().iter() {
        resources_info += format!("{}/{}/{};", resource.get_name(), resource.get_description(), resource.display_status()).as_str();
    }
    for (_, resource) in INTERNET_RESOURCES.lock().unwrap().iter() {
        resources_info += format!("{}/{}/{};", resource.get_name(), resource.get_description(), resource.display_status()).as_str();
    }

    resources_info
}

pub fn get_resource_description(name: &str) -> String {
    match INTERNET_RESOURCES.lock().unwrap().get(name) {
        Some(resource) => {
            return resource.get_description().to_string();
        },
        None => (),
    }
    match BLUETOOTH_RESOURCES.lock().unwrap().get(name) {
        Some(resource) => {
            return resource.get_description().to_string();
        },
        None => (),
    } 
    "".to_string()
}

pub fn get_resource_status_str(name: &str) -> String {
    match INTERNET_RESOURCES.lock().unwrap().get(name) {
        Some(resource) => {
            return resource.display_status().to_string();
        },
        None => (),
    }
    match BLUETOOTH_RESOURCES.lock().unwrap().get(name) {
        Some(resource) => {
            return resource.display_status().to_string();
        },
        None => (),
    } 
    "".to_string()
}

async fn send_message_internet(r: &InternetResource, i: &str, i_type: &str, id: Option<i64>) -> Result<(), Box<dyn Error>> {
    let addr = r.get_address();
    let reject = if r.is_interpreter_none() {
        let m = Message::new(MessageType::Reject, i.to_string(), id);
        serde_json::to_string(&m)?
    } else {
        let id = if id.is_none() {""} else {&(id.unwrap().to_string() + ":")};
        "".to_string() + i_type + ":" + id + i
    };
    let data: Vec<u8> = reject.as_bytes().to_vec();
    SOCKET.lock().unwrap().as_ref().unwrap().send_to(&data, addr).await?;
    info!("message send");
    Ok(())
}

async fn send_message_bluetooth(r: &BluetoothResource, i: &str, i_type: &str, id: Option<i64>) -> Result<(), Box<dyn Error>> {
    let char = r.get_char().as_ref().unwrap();
    let reject = if r.is_interpreter_none() {
        let m = Message::new(MessageType::Reject, i.to_string(), id);
        serde_json::to_string(&m)?
    } else {
        let id = if id.is_none() {""} else {&(id.unwrap().to_string() + ":")};
        "".to_string() + i_type + ":" + id + i
    };
    let data: Vec<u8> = reject.as_bytes().to_vec();
    char.write(&data).await?;
    info!("message send");
    Ok(())
}

pub async fn reject_intent(resource_name: String, intent: &str) -> Result<(), Box<dyn Error>> {
    let r_m: std::sync::MutexGuard<'_, HashMap<String, Arc<InternetResource>>> = INTERNET_RESOURCES.lock().unwrap();
    let r: Option<&Arc<InternetResource>> = r_m.get(&resource_name);
    if r.is_some() {
        match send_message_internet(r.unwrap().as_ref(), intent, "Reject", None).await {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
    }
    let _ = r;

    let r_m: std::sync::MutexGuard<'_, HashMap<String, Arc<BluetoothResource>>> = BLUETOOTH_RESOURCES.lock().unwrap();
    let r: Option<&Arc<BluetoothResource>> = r_m.get(&resource_name);
    if r.is_some() {
        match send_message_bluetooth(r.unwrap().as_ref(), intent, "Reject", None).await {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
    }
    let _ = r;
    
    if resource_name == "TAPE" && TAPE.lock().unwrap().is_some() {
        match TAPE.lock().unwrap().as_ref().unwrap() {
            ResourceType::Bluetooth(r) => {
                match send_message_bluetooth(r, intent, "Reject", None).await {
                    Ok(()) => (),
                    Err(e) => return Err(e),
                }
            },
            ResourceType::Internet(r) => {
                match send_message_internet(r, intent, "Reject", None).await {
                    Ok(()) => (),
                    Err(e) => return Err(e),
                }
            },
            _ => (),
        }
    }
    
    Ok(())
}

pub async fn send_intent(resource_name: String, intent: &str, id: i64) -> Result<(), Box<dyn Error>> {
    let r_m = INTERNET_RESOURCES.lock().unwrap();
    let r = r_m.get(&resource_name);
    if r.is_some() {
        match send_message_internet(r.unwrap().as_ref(), intent, "Intent", Some(id)).await {
            Ok(()) => (),
            Err(e) => return Err(e),
        }
    }
    let _ = r;

    let r_m = BLUETOOTH_RESOURCES.lock().unwrap();
    let r = r_m.get(&resource_name);
    if r.is_some() {
        match send_message_bluetooth(r.unwrap().as_ref(), intent, "Intent", Some(id)).await {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
    }
    let _ = r; 

    if resource_name == "TAPE" && TAPE.lock().unwrap().is_some() {
        let t = TAPE.lock().unwrap();
        match t.as_ref().unwrap() {
            ResourceType::Bluetooth(r) => {
                match send_message_bluetooth(r, intent, "Intent", Some(id)).await {
                    Ok(()) => (),
                    Err(e) => return Err(e),
                }
            },
            ResourceType::Internet(r) => {
                // TODO: may error here.
                match send_message_internet(r, intent, "Intent", Some(id)).await {
                    Ok(()) => (),
                    Err(e) => return Err(e),
                }
            },
            _ => (),
        }

    }
    
    Ok(())
}