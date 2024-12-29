// this file is used to wait for the resource and subsystem to connect.
// when the resource and subsystem are querying to connect, 
// the waiter will store the information of the resource or subsystem.
// and maintain the connection.
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::sync::mpsc::{Sender, Receiver};
use std::error::Error;
use crate::components::linkhub::bluetooth;
use crate::components::linkhub::wifi;
use crate::components::linkhub::internet;
use crate::base::resource::ResourceType;

#[allow(dead_code)]
pub enum WaitMethod {
    Bluetooth,
    Wifi,
    Internet,
    RFID,
    NFC,
}

const WAIT_METHOD: WaitMethod = WaitMethod::Bluetooth;

// TODO: make TAPE a single resource instead of a vector.
lazy_static! {
    pub static ref TAPE: Mutex<Vec<ResourceType>> = Mutex::new(Vec::new());
    pub static ref WAIT_SEND: Mutex<Option<Sender<String>>> = Mutex::new(None);
    pub static ref WAIT_RECV: Mutex<Option<Receiver<String>>> = Mutex::new(None);
}

pub fn channel_init(wait_send: Option<Sender<String>>, wait_recv: Option<Receiver<String>>) {
    WAIT_SEND.lock().unwrap().replace(wait_send.unwrap());
    WAIT_RECV.lock().unwrap().replace(wait_recv.unwrap());
}


pub fn wait() -> Result<(), Box<dyn Error>> {
    match WAIT_METHOD {
        WaitMethod::Bluetooth => bluetooth::wait::wait(),
        WaitMethod::Wifi => wifi::wait::wait(),
        WaitMethod::Internet => internet::wait::wait(),
        _ => {
            return Err("Unsupported wait method".into());
        }
    }
}


