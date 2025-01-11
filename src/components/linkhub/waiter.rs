// this file is used to wait for the resource and subsystem to connect.
// when the resource and subsystem are querying to connect, 
// the waiter will store the information of the resource or subsystem.
// and maintain the connection.
use std::{
    error::Error, 
    sync::{
        mpsc::{Receiver, Sender}, Mutex
    }
};
use lazy_static::lazy_static;
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
    Bluetooth(BluetoothResource),
    Internet(InternetResource),
    Other,
}

lazy_static! {
    pub static ref TAPE: Mutex<Option<ResourceType>> = Mutex::new(None);
    pub static ref WAIT_SEND: Mutex<Option<Sender<String>>> = Mutex::new(None);
    pub static ref WAIT_RECV: Mutex<Option<Receiver<String>>> = Mutex::new(None);
}

pub fn channel_init(wait_send: Option<Sender<String>>, wait_recv: Option<Receiver<String>>) {
    WAIT_SEND.lock().unwrap().replace(wait_send.unwrap());
    WAIT_RECV.lock().unwrap().replace(wait_recv.unwrap());
}


pub async fn wait() -> Result<(), Box<dyn Error>> {
    match WAIT_METHOD {
        WaitMethod::Bluetooth => bluetooth::wait::wait(),
        WaitMethod::Wifi => wifi::wait::wait(),
        WaitMethod::Internet => internet::wait::wait().await,
        _ => {
            return Err("Unsupported wait method".into());
        }
    }
}

