// in this file, we will implement the seeker, which is used to seek the 
// higher level system when not connected.

// we will support multiple seeker, and each seeker will use different strategy to seek the higher level system.
// now we will implement in three strategies:
// 1. bluetooth
// 2. wifi
// 3. internet

use bluer::{AdapterEvent, Device, gatt::remote::Characteristic, gatt::remote::Service};
use futures::{pin_mut, StreamExt};
use std::{error::Error, time::Duration, sync::Mutex};
use crate::base::resource::{BluetoothResource, Resource};
use tokio::time::sleep;
use lazy_static::lazy_static;

// TODO: make TAPE a single resource instead of a vector.
lazy_static! {
    pub static ref TAPE: Mutex<Vec<Box<dyn Resource>>> = Mutex::new(Vec::new());
}

#[allow(dead_code)]
enum SeekMethod {
    Bluetooth,
    Wifi,
    Internet,
    RFID,
    NFC,
    // TODO: add more seek methods
}

#[allow(dead_code)]
enum Platform {
    Linux,
    Windows,
    Mac,
    Android,
    IOS,
}

const SEEK_METHOD: SeekMethod = SeekMethod::Bluetooth;
const PLATFORM: Platform = Platform::Linux;

// seek the higher level system periodically.
pub fn seek() -> Result<(), Box<dyn Error>> {
    
    if TAPE.lock().unwrap().len() > 0 {
        return Err("Tape exists".into());
    }

    match SEEK_METHOD {
        SeekMethod::Bluetooth => seek_by_bluetooth(),
        SeekMethod::Wifi => seek_by_wifi(),
        SeekMethod::Internet => seek_by_internet(),
        _ => {
            return Err("Unsupported seek method".into());
        }
    }
}

// seek by bluetooth. And for different platform, we will implement different logic.
fn seek_by_bluetooth() -> Result<(), Box<dyn Error>> {
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
    env_logger::init();

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
                // AdapterEvent::DeviceRemoved(addr) => {
                //     // TODO: Maybe we can detect if connection is lost here.
                //     println!("Device removed: {addr}");
                // }
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
                        store_bluetooth_tape(device, cha, service).await?;
                        return Ok(Ok(Some(c)));
                    }
                }
            }
        }
    }

    Ok(Ok(None))
}

// create new resource and store the bluetooth device properties into the resource pool
async fn store_bluetooth_tape(device: Device, cha: Characteristic, service: Service) -> bluer::Result<()> {
    let props = device.all_properties().await?;
    let mut tape = BluetoothResource::new(
        device,
        props,
        Some(service),
        Some(cha),
    );
    complete_tape(&mut tape).await?;
    TAPE.lock().unwrap().push(Box::new(tape));
    Ok(())
}

// complete the tape by the resource pool
async fn complete_tape(tape: &mut dyn Resource) -> bluer::Result<()> {
    tape.set_description("tape".to_string());
    // TODO: we need to communicate with the higher level system to get the tape information
    Ok(())
}

fn seek_by_wifi() -> Result<(), Box<dyn Error>> {
    // TODO: implement the logic to seek the higher level system by wifi
    Err("Not implemented".into())
}

fn seek_by_internet() -> Result<(), Box<dyn Error>> {
    // TODO: implement the logic to seek the higher level system by internet
    Err("Not implemented".into())
}

