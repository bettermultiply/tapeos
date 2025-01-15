// this file is used to wait for the resource and subsystem to connect.
// when the resource and subsystem are querying to connect, 
// the waiter will store the information of the resource or subsystem.
// and maintain the connection.
use std::{
    error::Error, 
    sync::{
        mpsc::{Receiver, Sender}, Arc
    }
};
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use crate::components::linkhub::{bluetooth, wifi, internet};

use super::{bluetooth::resource::BluetoothResource, internet::resource::InternetResource};

#[allow(dead_code)]
pub enum WaitMethod {
    Bluetooth,
    Wifi,
    Internet,
    RFID,
    NFC,
}

const WAIT_METHOD: WaitMethod = WaitMethod::Bluetooth;
pub enum ResourceType {
    Bluetooth,
    Internet,
    Other,
    None
}

impl ResourceType {
    pub fn is_none(&self) -> bool {
        match self {
            ResourceType::None => true,
            _ => false,
        }
    }

    pub fn is_bluetooth(&self) -> bool {
        match self {
            ResourceType::Bluetooth => true,
            _ => false,
        }
    }

    pub fn is_internet(&self) -> bool {
        match self {
            ResourceType::Internet => true,
            _ => false,
        }
    }

    pub fn copy(&self) -> ResourceType {
        match self {
            ResourceType::Internet => ResourceType::Internet,
            ResourceType::Bluetooth => ResourceType::Bluetooth,
            ResourceType::None => ResourceType::None,
            _ => ResourceType::Other,
        }
    }


}

lazy_static! {
    pub static ref TAPE: Arc<Mutex<ResourceType>> = Arc::new(Mutex::new(ResourceType::None));
    pub static ref BTAPE: Arc<Mutex<Option<Arc<Mutex<BluetoothResource>>>>> = Arc::new(Mutex::new(None));
    pub static ref ITAPE: Arc<Mutex<Option<Arc<Mutex<InternetResource>>>>> = Arc::new(Mutex::new(None));
    pub static ref WAIT_SEND: Mutex<Option<Sender<String>>> = Mutex::new(None);
    pub static ref WAIT_RECV: Mutex<Option<Receiver<String>>> = Mutex::new(None);
}

pub async fn channel_init(wait_send: Option<Sender<String>>, wait_recv: Option<Receiver<String>>) {
    WAIT_SEND.lock().await.replace(wait_send.unwrap());
    WAIT_RECV.lock().await.replace(wait_recv.unwrap());
}


pub async fn wait() -> Result<(), Box<dyn Error>> {
    match WAIT_METHOD {
        WaitMethod::Bluetooth => bluetooth::wait::wait(),
        WaitMethod::Wifi => wifi::wait::wait(),
        WaitMethod::Internet => internet::wait::wait("".to_string(), "".to_string(), 0).await,
        _ => {
            return Err("Unsupported wait method".into());
        }
    }
}

