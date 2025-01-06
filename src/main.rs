use tapeos::{
    core::inxt::intent,
    base::intent::Intent,
    base::intent::IntentSource,
    base::intent::IntentType,
};

#[tokio::main]
async fn main() {
    let intent = Intent::new("store my name".to_string(), IntentSource::Resource, IntentType::Intent, None);
    println!("main: ");
    println!("main: Try to execute intent");
    intent::handler(intent).await;
    println!("main: Try ended");
}

/*use tapeos::{
    components::linkhub::{seeker, waiter},
    tools::idgen::init_id_generator
};
use std::{
    thread, 
    sync::mpsc::{Sender, Receiver, channel}
};

const ENABLE_SEEK: bool = true;
const ENABLE_WAIT: bool = true;
const ENABLE_BOTH: bool = ENABLE_SEEK && ENABLE_WAIT;

fn main() {
    let mut seek_send: Option<Sender<String>> = None;
    let mut wait_send: Option<Sender<String>> = None;
    let mut seek_recv: Option<Receiver<String>> = None;
    let mut wait_recv: Option<Receiver<String>> = None;
    if ENABLE_BOTH {
        let (send, recv) = channel::<String>();
        (seek_send, wait_recv) = (Some(send), Some(recv));
        let (send, recv) = channel::<String>();
        (wait_send, seek_recv) = (Some(send), Some(recv));
    }
    match init_id_generator() {
        Ok(_) => (),
        Err(e) => {
            println!("Error initializing id generator: {}", e);
            return;
        }
    }

    let mut handles = vec![];
    if ENABLE_SEEK {
        handles.push(thread::spawn(move || {
            if ENABLE_BOTH {
                seeker::channel_init(seek_send, seek_recv);
            }
            let _ = seeker::seek();
        }));
    }

    if ENABLE_WAIT {
        handles.push(thread::spawn(move || {
            if ENABLE_BOTH {
                waiter::channel_init(wait_send, wait_recv);
            }
            let _ = waiter::wait();
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
 */