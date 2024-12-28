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
    let resource = &blue_resource;
    // TODO: we need to communicate with the higher level system to get the tape information

    Ok(())
}

pub async fn send_intent<'a>(blue_resource: &BluetoothResource, intent: &Intent<'a>) -> bluer::Result<()> {
    // TODO: implement the logic to send the intent to the resource
    // let resource: &dyn Resource = blue_resource.clone();
    let char = blue_resource.get_char().as_ref().unwrap();
    let data: Vec<u8> = intent.get_description().as_bytes().to_vec();
    char.write(&data).await?;
    sleep(Duration::from_secs(1)).await;

    Ok(())
}

pub async fn receive_intent(blue_resource: &BluetoothResource) -> bluer::Result<()> {
    let char = blue_resource.get_char().as_ref().unwrap();
    let data = char.read().await?;
    println!("Received data: {:?}", data);
    Ok(())
}

// try to parse the response from untape resource
fn try_parse_response(data: Vec<u8>) -> HashMap<String, Vec<String>> {
    let response = String::from_utf8(data).unwrap();
    let rough_parsed = llmq::prompt(&response);
    let parsed = parse_rough_response(&rough_parsed);
    parsed
}

fn parse_rough_response(rough_response: &str) -> HashMap<String, Vec<String>> {
    let sub_intents: HashMap<String, Vec<String>> = rough_response.split(";").map(|s| (s.to_string(), vec![])).collect();
    sub_intents
}