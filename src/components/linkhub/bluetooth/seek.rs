// seek by bluetooth. And for different platform, we will implement different logic.

use bluer::{
    Device, AdapterEvent, 
    gatt::remote::{Characteristic, Service}
};
use std::{
    collections::HashMap, sync::Arc, time::Duration 
};
use futures::{pin_mut, StreamExt, future};
use tokio::{
    io::{AsyncBufReadExt, BufReader}, sync::Mutex, time::{interval, sleep}

};

use crate::{
    base::{ 
        intent::{Intent, IntentSource, IntentType}, resource::{Interpreter, Position, Resource}
    }, components::linkhub::{bluetooth::resource::BluetoothResource, seeker::{send_intent, BLUETOOTH_RESOURCES, RESPONSE_QUEUE, SEEK_RECV}}, core::inxt::intent::handler, tools::llmq
};

use crate::base::errort::BoxResult;

#[allow(dead_code)]
enum Platform {
    Linux,
    Windows,
    Mac,
    Android,
    IOS,
}

const PLATFORM: Platform = Platform::Linux;

// seek by bluetooth. And for different platform, we will implement different logic.
pub fn seek() -> BoxResult<()> {
    match PLATFORM {
        Platform::Linux => {
            tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                match seek_bluetooth_linux().await {
                    Ok(_) => (),
                    Err(err) => {
                        println!("seek bluetooth failed: {}", &err);
                    }
                }
            });
            Ok(())
        }
        _ => {
            return Err("Unsupported platform".into());
        }
    }
}

// seek by bluetooth on linux
async fn seek_bluetooth_linux() -> bluer::Result<()> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;
    
    println!(
    "Discovering on Bluetooth adapter {} with address {}", 
        adapter.name(), 
        adapter.address().await?
    );
    let stdin = BufReader::new(tokio::io::stdin());
    
    let mut lines = stdin.lines();
    let mut interval = interval(Duration::from_secs(1));
    
    let devices_events = adapter.discover_devices().await?;
    pin_mut!(devices_events);

    loop {
        tokio::select! {
            // exit the loop
            _ = lines.next_line() => break,
            // handle the device connect and disconnect
            Some(device_event) = devices_events.next() => {
                match device_event {
                    AdapterEvent::DeviceAdded(addr) => {
                        let device = adapter.device(addr)?;
                        let name = device.name().await?.unwrap_or_default();
                        match find_tape_characteristic(device).await? {
                            Ok(Some(_)) => {
                                println!("    find tapeos {}", name);
                            },
                            Ok(None) => {
                                println!("    no tapeos {} {}", name, addr);
                                let _ = adapter.remove_device(addr).await;
                            },
                            Err(err) => {
                                println!("    Device failed: {}", err);
                                let _ = adapter.remove_device(addr).await;
                            }
                        }
                    }
                    AdapterEvent::DeviceRemoved(addr) => {
                        let device = adapter.device(addr)?;
                        let name = device.name().await?.unwrap_or_default();
                        remove_resource(name).await;
                        println!("Device removed: {addr}");
                    }
                    _ => (),
                }
            },
            // check resource action
            _ = interval.tick() => {
                check_resources().await?;
            },
            // check waiter request
            request = async {
                match SEEK_RECV.lock().await.as_ref().unwrap().try_recv() {
                    Ok(v) => v,
                    Err(err) => {
                        println!("seek: receive waiter request failed: {}", &err);
                        future::pending().await
                    }
                }
            } => {
                execute_waiter_request(request).await;
            }
        }
    }

    Ok(())
}

// Tape-related constants and types
const TAPE_SERVICE_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00001234_0000_1000_8000_00805f9b34fb); // Example UUID
const TAPE_CHARACTERISTIC_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00005678_0000_1000_8000_00805f9b34fb); // Example UUID
const RETRIES: u8 = 2;

async fn check_resources() -> bluer::Result<()> {
    let resources = BLUETOOTH_RESOURCES.lock().await;
    for (_, resource) in resources.iter() {
        let r = resource.lock().await;
        let _ = query_status(r.get_name()).await;
        let char = r.get_char().as_ref().unwrap();
        if char.flags().await?.read {
            let (key, value) = receive_message(Arc::clone(resource)).await?;
            match key.as_str() {
                "Intent" => {
                    let intent = receive_intent(value, Arc::clone(resource)).await?;
                    handler(intent).await;
                }
                "Response" => {
                    let response = receive_response(value).await?;
                    store_response(response).await?;
                }
                _ => (),
            }
        }
    }
    println!("check resources {} times", BLUETOOTH_RESOURCES.lock().await.len());
    Ok(())
}

async fn find_tape_characteristic(device: Device) -> bluer::Result<bluer::Result<Option<Characteristic>>> {
    
    let uuids = device.uuids().await?.unwrap_or_default();

    if uuids.contains(&TAPE_SERVICE_UUID) {
        println!("    Device provides tape service");

        sleep(Duration::from_secs(2)).await;
        if !device.is_connected().await? {
            println!("    Connecting...");
            let mut retries = RETRIES;
            loop {
                match device.connect().await {
                    Ok(()) => break,
                    Err(err) if retries > 0 => {
                        println!("    Connect error: {:?}", &err);
                        retries -= 1;
                    }
                    Err(err) => return Err(err),
                }
            }
        } 
        println!("    Connected");

        for service in device.services().await? {
            let uuid = service.uuid().await?;
            if TAPE_SERVICE_UUID == uuid {
                for cha in service.characteristics().await? {
                    let uuid = cha.uuid().await?;
                    if TAPE_CHARACTERISTIC_UUID == uuid {
                        let c = cha.clone();
                        store_resource(device, cha, service).await?;
                        return Ok(Ok(Some(c)));
                    }
                }
            }
        }
    }

    Ok(Ok(None))
}

// create new resource and store the bluetooth device properties into the resource pool
async fn store_resource(device: Device, cha: Characteristic, service: Service) -> bluer::Result<()> {
    let props = device.all_properties().await?;
    let name = device.name().await?.unwrap_or_default();
    let resource = BluetoothResource::new(
        name.clone(),
        device,
        props,
        Some(service),
        Some(cha),
    );
    let r = Arc::new(Mutex::new(resource));
    let r_copy = Arc::clone(&r);
    BLUETOOTH_RESOURCES.lock().await.insert(name, r);
    complete_resource(r_copy).await?;
    Ok(())
}

async fn remove_resource(name: String) {
    BLUETOOTH_RESOURCES.lock().await.remove(&name);
}

// complete the resource by the resource pool
async fn complete_resource(blue_resource: Arc<Mutex<BluetoothResource>>) -> bluer::Result<()> {
    let _ = query_status(blue_resource.lock().await.get_name()).await;
    let value: String;
    loop {
        let (k, v) = receive_message(Arc::clone(&blue_resource)).await?;
        if k == "Response" {
            value = v;
            break;
        }
    }
    let response = receive_response(value).await?;
    let mut resource = blue_resource.lock().await;
    let status = resource.get_status();
    for status_pair in response.get("status").unwrap().split(';') {
        let status_name = status_pair.split(':').next().unwrap();
        let status_value = status_pair.split(':').last().unwrap();
        match status_name {
            "availability" => status.set_aviliability(status_value.parse::<bool>().unwrap()),
            "Position" => status.set_position(Position::new_from_vec(status_value.to_owned().split(':').map(|s| s.to_string().parse::<f32>().unwrap()).collect())),
            "busy_time" => status.set_busy_time(Duration::from_secs(status_value.parse::<u64>().unwrap())),
            _ => (),
        }
    }
    resource.set_description(response.get("description").unwrap().to_owned());
    resource.set_interpreter(Interpreter::LLM("HEEL".to_string()));
    Ok(())
}

// 0 means initial intent
pub async fn query_status(resource: &str) -> BoxResult<()> {
    send_intent(resource.to_string(), "query for status; query for command; query for description; query for interpreter;", 0).await?;
    Ok(())
}

pub async fn receive_message(blue_resource: Arc<Mutex<BluetoothResource>>) -> bluer::Result<(String, String)> {
    let b_resource = blue_resource.lock().await;
    let char = b_resource.get_char().as_ref().unwrap();
    let data = char.read().await?;
    let raw = String::from_utf8(data).unwrap();
    let parts = raw.splitn(2, ':').collect::<Vec<&str>>();
    if parts.len() < 2 {
        // if there is no specific format, use it as intent.
        return Ok(("Intent".to_string(), parts[0].to_string()))
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

pub async fn receive_intent(raw_intent: String, blue_resource: Arc<Mutex<BluetoothResource>>) -> bluer::Result<Intent> {
    let r = blue_resource.lock().await;
    let intent = Intent::new(
        raw_intent,
        IntentSource::Resource, 
        IntentType::Intent,
        Some(r.get_name().to_string())
    );

    Ok(intent)
}

pub async fn receive_response(response: String) -> bluer::Result<HashMap<String, String>> {
    let parsed = try_parse_response(response).await;
    Ok(parsed)
}



// try to parse the response from untape resource
async fn try_parse_response(data: String) -> HashMap<String, String> {
    let rough_parsed = llmq::prompt("try to parse the response", &data).await;
    parse_rough_response(&rough_parsed)
}

// use ':' to unwrap the key and value
fn parse_rough_response(rough_response: &str) -> HashMap<String, String> {
    let sub_intents: HashMap<String, String> = rough_response.split(";").map(|s| (s.split(":").next().unwrap().to_string(), s.split(":").last().unwrap().to_string())).collect();
    sub_intents
}

pub async fn store_response(response: HashMap<String, String>) -> bluer::Result<()> {
    RESPONSE_QUEUE.lock().await.push(response);
    Ok(())
}

pub async fn execute_waiter_request(request: String) {
    let map = parse_request(request);
    
    for (key, value) in map {
        match key.as_str() {
            "Intent" => {
                println!("Intent: {}", value);
                let intent = Intent::new(value, IntentSource::Resource, IntentType::Intent, None);
                handler(intent).await;
            }
            _ => {
                println!("Unsupported request: {}", key);
            },
        }
    }
}

fn parse_request(request: String) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in request.lines() {
        let parts = line.splitn(2, ':').collect::<Vec<&str>>();
        if parts.len() == 2 {
            map.insert(parts[0].to_string(), parts[1].to_string());
        }
    }
    map
}
