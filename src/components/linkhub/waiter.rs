// this file is used to wait for the resource and subsystem to connect.
// when the resource and subsystem are querying to connect, 
// the waiter will store the information of the resource or subsystem.
// and maintain the connection.
use std::{
    time,
    net::{IpAddr, Ipv4Addr, SocketAddr}, 
    sync::{
        Arc,
        mpsc::{Receiver, Sender}, 
    }, 
};
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use crate::{
    base::{
        errort::BoxResult, 
        intent::Intent, 
        resource::{ResourceType, Status},
    }, 
    components::linkhub::{
        bluetooth::{self, resource::BluetoothResource}, 
        internet::{self, resource::InternetResource}, 
        wifi,
    }
};

pub enum WaitMethod {
    Bluetooth,
    Wifi,
    Internet,
    RFID,
    NFC,
}
const WAIT_METHOD: WaitMethod = WaitMethod::Bluetooth;

type Queue<T> = Mutex<Vec<T>>;
type Glo<T> = Arc<Mutex<T>>;

lazy_static! {
    pub static ref TAPE: Arc<Mutex<ResourceType>> = Arc::new(Mutex::new(ResourceType::None));
    pub static ref BTAPE: Glo<Option<Arc<Mutex<BluetoothResource>>>> = Arc::new(Mutex::new(None));
    pub static ref ITAPE: Arc<Mutex<InternetResource>> = Arc::new(Mutex::new(InternetResource::new("Tape".to_string(), "".to_string(), SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8888), Status::new(true, (1.0, 1.0, 1.0), time::Duration::from_secs(0)))));
    pub static ref WAIT_SEND: Mutex<Option<Sender<String>>> = Mutex::new(None);
    pub static ref WAIT_RECV: Mutex<Option<Receiver<String>>> = Mutex::new(None);
    pub static ref HEART: Mutex<bool> = Mutex::new(true);

    pub static ref WAIT_EXEC_ADDR: Mutex<String> = Mutex::new("127.0.0.1:8000".to_string());
    pub static ref TAPE_INTENT_QUEUEUE: Queue<Intent> = Mutex::new(vec![]);
}

pub async fn channel_init(wait_send: Option<Sender<String>>, wait_recv: Option<Receiver<String>>) {
    WAIT_SEND.lock().await.replace(wait_send.unwrap());
    WAIT_RECV.lock().await.replace(wait_recv.unwrap());
}

pub async fn wait() -> BoxResult<()> {
    match WAIT_METHOD {
        WaitMethod::Bluetooth => bluetooth::wait::wait(),
        WaitMethod::Wifi => wifi::wait::wait(),
        WaitMethod::Internet => internet::wait::wait("".to_string(), "".to_string(), 0).await,
        _ => {
            return Err("Unsupported wait method".into());
        }
    }
}

