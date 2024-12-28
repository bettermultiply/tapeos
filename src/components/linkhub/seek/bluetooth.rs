use crate::base::resource::RESOURCES;
use crate::base::resource::BluetoothResource;
use bluer::{Device, AdapterEvent, gatt::remote::{Characteristic, Service}};
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;
use futures::{pin_mut, StreamExt};
use crate::components::linkhub::seeker::{Platform, PLATFORM};
use crate::base::intent::Intent;
use crate::tools::llmq;
use std::collections::HashMap;
use crate::base::intent::IntentSource;
use crate::base::resource::Resource;
use std::path::PathBuf;
use crate::base::resource::Position;

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
    
    {
        println!(
            "Discovering on Bluetooth adapter {} with address {}", 
            adapter.name(), 
            adapter.address().await?
        );
        
        let devices_events = adapter.discover_devices().await?;
        pin_mut!(devices_events);
        let mut done = false;

        while let Some(device_event) = devices_events.next().await {
            match device_event {
                AdapterEvent::DeviceAdded(addr) => {
                    let device = adapter.device(addr)?;
                    let name = device.name().await?.unwrap_or_default();
                    match find_tape_characteristic(device).await? {
                        Ok(Some(_)) => {
                            println!("    find tapeos {}", name);
                            done = true;
                        },
                        Ok(None) => {
                            println!("    no tapeos {}", name);

                        },
                        Err(err) => {
                            println!("    Device failed: {}", err);
                            let _ = adapter.remove_device(addr).await;
                        }
                    }
                }
                AdapterEvent::DeviceRemoved(addr) => {
                    // TODO: Maybe we can detect if connection is lost here.
                    RESOURCES.lock().unwrap().retain(|resource| !resource.compare_address(addr));
                    println!("Device removed: {addr}");
                }
                _ => (),
            }
            if done {
                break;
            }
        }
        println!("Discovery completed");
    }

    Ok(())
}

// Tape-related constants and types
const TAPE_SERVICE_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00001234_0000_1000_8000_00805f9b34fb); // Example UUID
const TAPE_CHARACTERISTIC_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00005678_0000_1000_8000_00805f9b34fb); // Example UUID
const RETRIES: u8 = 2;

async fn find_tape_characteristic(device: Device) -> bluer::Result<bluer::Result<Option<Characteristic>>> {
    
    let uuids = device.uuids().await?.unwrap_or_default();

    if uuids.contains(&TAPE_SERVICE_UUID) {
        println!("    Device provides tape service");

        sleep(Duration::from_secs(2)).await;
        if !device.is_connected().await? {
            println!("    Connecting...");
            let mut retries = RETRIES; // TODO: make it configurable
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
    RESOURCES.lock().unwrap().push(Box::new(resource));
    Ok(())
}

// complete the resource by the resource pool
async fn complete_resource(blue_resource: &mut BluetoothResource) -> bluer::Result<()> {
    query_status(blue_resource).await?;
    let response = receive_response(blue_resource).await?;
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
    send_intent(blue_resource, "query for status; query for command; query for description; query for interpreter;").await?;
    Ok(())
}

pub async fn send_intent<'a>(blue_resource: &BluetoothResource, intent_description: &str) -> bluer::Result<()> {
    // TODO: implement the logic to send the intent to the resource
    // let resource: &dyn Resource = blue_resource.clone();
    let char = blue_resource.get_char().as_ref().unwrap();
    let data: Vec<u8> = intent_description.as_bytes().to_vec();
    char.write(&data).await?;
    sleep(Duration::from_secs(1)).await;

    Ok(())
}

pub async fn receive_intent(blue_resource: &BluetoothResource) -> bluer::Result<Intent> {
    let char = blue_resource.get_char().as_ref().unwrap();
    let data = char.read().await?;
    let intent = Intent::new
    (
        String::from_utf8(data).unwrap(), 
        IntentSource::Resource, 
        Some(blue_resource));
    Ok(intent)
}

pub async fn receive_response(blue_resource: &BluetoothResource) -> bluer::Result<HashMap<String, String>> {
    let char: &Characteristic = blue_resource.get_char().as_ref().unwrap();
    let data = char.read().await?;
    let parsed = try_parse_response(data);
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
fn try_parse_response(data: Vec<u8>) -> HashMap<String, String> {
    let response = String::from_utf8(data).unwrap();
    let rough_parsed = llmq::prompt(&response);
    parse_rough_response(&rough_parsed)
}

// use ':' to unwrap the key and value
fn parse_rough_response(rough_response: &str) -> HashMap<String, String> {
    let sub_intents: HashMap<String, String> = rough_response.split(";").map(|s| (s.split(":").next().unwrap().to_string(), s.split(":").last().unwrap().to_string())).collect();
    sub_intents
}
