// in this file, we will implement the seeker, which is used to seek the 
// higher level system when not connected.

// we will support multiple seeker, and each seeker will use different strategy to seek the higher level system.
// now we will implement in three strategies:
// 1. bluetooth
// 2. wifi
// 3. internet

use std::error::Error;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::base::resource::ResourceType;
use crate::components::linkhub::{bluetooth, wifi, internet};

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
    pub static ref RESOURCES: Mutex<Vec<Arc<ResourceType>>> = Mutex::new(Vec::new());
    pub static ref RESPONSE_QUEUE: Mutex<Vec<HashMap<String, String>>> = Mutex::new(Vec::new());
    pub static ref SEEK_SEND: Mutex<Option<Sender<String>>> = Mutex::new(None);
    pub static ref SEEK_RECV: Mutex<Option<Receiver<String>>> = Mutex::new(None);
}

pub fn channel_init(seek_send: Option<Sender<String>>, seek_recv: Option<Receiver<String>>) {
    SEEK_SEND.lock().unwrap().replace(seek_send.unwrap());
    SEEK_RECV.lock().unwrap().replace(seek_recv.unwrap());
}

// seek resources and subsystems depend on the SEEK_METHOD.
pub fn seek() -> Result<(), Box<dyn Error>> {
    match SEEK_METHOD {
        SeekMethod::Bluetooth => bluetooth::seek::seek(),
        SeekMethod::Wifi => wifi::seek::seek(),
        SeekMethod::Internet => internet::seek::seek(),
        _ => {
            return Err("Unsupported seek method".into());
        }
    }
}