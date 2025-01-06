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
use tokio::net::UdpSocket;

use crate::components::linkhub::{bluetooth, internet, wifi};
use crate::base::resource::Resource;
use super::{bluetooth::resource::BluetoothResource, internet::resource::InternetResource, waiter::{ResourceType, TAPE}};

#[allow(dead_code)]
enum SeekMethod {
    Bluetooth,
    Wifi,
    Internet,
    RFID,
    NFC,
}

const SEEK_METHOD: SeekMethod = SeekMethod::Bluetooth;
lazy_static! {
    // we use these resource seperately for different seeker.
    // pub static ref RESOURCES: Mutex<Vec<Arc<ResourceType>>> = Mutex::new(Vec::new());
    pub static ref INTERNET_RESOURCES: Mutex<HashMap<String, Arc<InternetResource>>> = Mutex::new(HashMap::new());
    pub static ref BLUETOOTH_RESOURCES: Mutex<HashMap<String, Arc<BluetoothResource>>> = Mutex::new(HashMap::new());
    // ...
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

// TODO
// fn find_resource_by_name(name: &str) -> Option<Box<Arc<dyn Resource>>>{
//     match INTERNET_RESOURCES.lock().unwrap().get(name) {
//         Some(resource) => {
//             return Some(Box::new(Arc::clone(resource)));
//         },
//         None => (),
//     }
//     match BLUETOOTH_RESOURCES.lock().unwrap().get(name) {
//         Some(resource) => {
//             let r: &dyn Resource = resource.as_ref(); 

//             return Some(r);
//         },
//         None => (),
//     } 
    
//     None
// }

pub async fn reject_intent(resource_name: String, intent: String) -> Result<(), Box<dyn Error>> {
    match INTERNET_RESOURCES.lock().unwrap().get(&resource_name) {
        Some(resource) => {
            let addr = resource.get_address();
            let socket = UdpSocket::bind(addr).await.expect("Failed to bind to socket");
            let reject = "Reject:".to_string() + &intent;
            let data: Vec<u8> = reject.as_bytes().to_vec();
            socket.send(&data).await?;

            return Ok(());
        },
        None => (),
    } 
    match BLUETOOTH_RESOURCES.lock().unwrap().get(&resource_name) {
        Some(resource) => {
            let char = resource.get_char().as_ref().unwrap();
            let reject = "Reject:".to_string() + &intent;
            let data: Vec<u8> = reject.as_bytes().to_vec();
            char.write(&data).await?;
            return Ok(());
        },
        None => (),
    } 
    
    if resource_name == "TAPE" {
        match TAPE.lock().unwrap().as_ref().unwrap() {
            ResourceType::Bluetooth(resource) => {
                let char = resource.get_char().as_ref().unwrap();
                let reject = "Reject:".to_string() + &intent;
                let data: Vec<u8> = reject.as_bytes().to_vec();
                char.write(&data).await?;
                return Ok(());
            },
            ResourceType::Internet(resource) => {
                let addr = resource.get_address();
                let socket = UdpSocket::bind(addr).await.expect("Failed to bind to socket");
                let reject = "Reject:".to_string() + &intent;
                let data: Vec<u8> = reject.as_bytes().to_vec();
                socket.send(&data).await?;
    
                return Ok(());
            },
            _ => (),
        }
    }
    
    Ok(())
}

pub async fn send_intent(resource_name: String, intent: String) -> Result<(), Box<dyn Error>> {
    match INTERNET_RESOURCES.lock().unwrap().get(&resource_name) {
        Some(resource) => {
            let addr = resource.get_address();
            let socket = UdpSocket::bind(addr).await.expect("Failed to bind to socket");
            let reject = "Intent:".to_string() + &intent;
            let data: Vec<u8> = reject.as_bytes().to_vec();
            socket.send(&data).await?;

            return Ok(());
        },
        None => (),
    } 

    match BLUETOOTH_RESOURCES.lock().unwrap().get(&resource_name) {
        Some(resource) => {
            let char = resource.get_char().as_ref().unwrap();
            let reject = "Intent:".to_string() + &intent;
            let data: Vec<u8> = reject.as_bytes().to_vec();
            char.write(&data).await?;
            return Ok(());
        },
        None => (),
    } 

    if resource_name == "TAPE" {
        match TAPE.lock().unwrap().as_ref().unwrap() {
            ResourceType::Bluetooth(resource) => {
                let char = resource.get_char().as_ref().unwrap();
                let reject = "Intent:".to_string() + &intent;
                let data: Vec<u8> = reject.as_bytes().to_vec();
                char.write(&data).await?;
                return Ok(());
            },
            ResourceType::Internet(resource) => {
                let addr = resource.get_address();
                let socket = UdpSocket::bind(addr).await.expect("Failed to bind to socket");
                let reject = "Intent:".to_string() + &intent;
                let data: Vec<u8> = reject.as_bytes().to_vec();
                socket.send(&data).await?;
    
                return Ok(());
            },
            _ => (),
        }
    }
    
    Ok(())
}