// seek by bluetooth. And for different platform, we will implement different logic.

use bluer::{
    Address, Device, AdapterEvent, 
    gatt::remote::{Characteristic, Service}
};
use std::{
    collections::HashMap, 
    time::Duration, 
    error::Error, 
    sync::Arc, 
    path::PathBuf
};
use futures::{pin_mut, StreamExt, future};
use tokio::{
    io::{BufReader, AsyncBufReadExt},
    time::{sleep, interval}
};

use crate::{
    tools::llmq,
    base::{ 
        intent::{IntentSource, Intent, IntentType},
        resource::{Resource, Position, BluetoothResource}
    },
    components::linkhub::seeker::{RESOURCES, SEEK_RECV, RESPONSE_QUEUE},
    core::inxt::intent::handler
};

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
pub fn seek() -> Result<(), Box<dyn Error>> {
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
                        remove_resource(addr);
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
                match SEEK_RECV.lock().unwrap().as_ref().unwrap().try_recv() {
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
    let resources = RESOURCES.lock().unwrap();
    for resource in resources.iter() {
        query_status(resource).await?;
        let char = resource.get_char().as_ref().unwrap();
        if char.flags().await?.read {
            let (key, value) = receive_message(resource).await?;
            match key.as_str() {
                "Intent" => {
                    let intent = receive_intent(value, resource).await?;
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
    println!("check resources {} times", resources.len());
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
    let mut resource = BluetoothResource::new(
        device,
        props,
        Some(service),
        Some(cha),
    );
    complete_resource(&mut resource).await?;
    RESOURCES.lock().unwrap().push(Arc::new(resource));
    Ok(())
}

fn remove_resource(addr: Address) {
    RESOURCES.lock().unwrap().retain(|resource| !resource.compare_address(addr));
}

// complete the resource by the resource pool
async fn complete_resource(blue_resource: &mut BluetoothResource) -> bluer::Result<()> {
    query_status(blue_resource).await?;
    let value: String;
    loop {
        let (k, v) = receive_message(blue_resource).await?;
        if k == "Response" {
            value = v;
            break;
        }
    }
    let response = receive_response(value).await?;
    let resource: &mut dyn Resource = blue_resource;
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
    resource.set_command(response.get("command").unwrap().to_owned().split(':').map(|s| s.to_string()).collect());
    resource.set_description(response.get("description").unwrap().to_owned());
    let path = response.get("interpreter").unwrap().to_owned();
    resource.set_interpreter(PathBuf::from(path));
    Ok(())
}

pub async fn query_status(blue_resource: &BluetoothResource) -> bluer::Result<()> {
    blue_resource.send_intent("query for status; query for command; query for description; query for interpreter;")
}



pub async fn receive_message(blue_resource: &BluetoothResource) -> bluer::Result<(String, String)> {
    let char = blue_resource.get_char().as_ref().unwrap();
    let data = char.read().await?;
    let raw = String::from_utf8(data).unwrap();
    let parts = raw.splitn(2, ':').collect::<Vec<&str>>();
    if parts.len() < 2 {
        // if there is no specific format, use it as intent.
        return Ok(("Intent".to_string(), parts[0].to_string()))
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

pub async fn receive_intent(raw_intent: String, blue_resource: &BluetoothResource) -> bluer::Result<Intent> {
    let intent = Intent::new(
        raw_intent,
        IntentSource::Resource, 
        IntentType::Intent,
        Some(blue_resource)
    );

    Ok(intent)
}

pub async fn receive_response(response: String) -> bluer::Result<HashMap<String, String>> {
    let parsed = try_parse_response(response).await;
    Ok(parsed)
}

pub async fn reject_intent<'a>(blue_resource: &BluetoothResource, intent: Intent<'a>) -> bluer::Result<()> {
    let char = blue_resource.get_char().as_ref().unwrap();
    let reject = "reject: ".to_string() + intent.get_description();
    let data: Vec<u8> = reject.as_bytes().to_vec();
    char.write(&data).await?;
    
    drop(intent);
    Ok(())
}

// try to parse the response from untape resource
async fn try_parse_response(data: String) -> HashMap<String, String> {
    let rough_parsed = llmq::prompt(&data).await;
    parse_rough_response(&rough_parsed)
}

// use ':' to unwrap the key and value
fn parse_rough_response(rough_response: &str) -> HashMap<String, String> {
    let sub_intents: HashMap<String, String> = rough_response.split(";").map(|s| (s.split(":").next().unwrap().to_string(), s.split(":").last().unwrap().to_string())).collect();
    sub_intents
}

pub async fn store_response(response: HashMap<String, String>) -> bluer::Result<()> {
    RESPONSE_QUEUE.lock().unwrap().push(response);
    Ok(())
}

pub async fn execute_waiter_request(request: String) {
    let map = parse_request(request);
    
    for (key, value) in map {
        match key.as_str() {
            "Intent" => {
                println!("Intent: {}", value);
                handler(Intent::new(value, IntentSource::Resource, IntentType::Intent, None)).await;
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
