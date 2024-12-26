// in this file, we will implement the seeker, which is used to seek the 
// higher level system when not connected.

// we will support multiple seeker, and each seeker will use different strategy to seek the higher level system.
// now we will implement in three strategies:
// 1. bluetooth
// 2. wifi
// 3. internet

use bluer::{Adapter, AdapterEvent, Address, Device, gatt::remote::Characteristic};
use futures::{pin_mut, StreamExt};
use std::{error::Error, time::Duration};
use crate::base::resource::{BluetoothResource, Status, Position, RESOURCES};
use std::collections::HashMap;
use std::time;
use tokio::time::sleep;


enum SeekMethod {
    Bluetooth,
    Wifi,
    Internet,
    RFID,
    NFC,
    // TODO: add more seek methods
}

enum Platform {
    Linux,
    Windows,
    Mac,
    Android,
    IOS,
}

fn seek() -> Result<(), Box<dyn Error>> {
    // TODO: we need to define seek method by configuration file.
    let seek_method = SeekMethod::Bluetooth;
    match seek_method {
        SeekMethod::Bluetooth => seek_by_bluetooth(),
        SeekMethod::Wifi => seek_by_wifi(),
        SeekMethod::Internet => seek_by_internet(),
        _ => {
            return Err("Unsupported seek method".into());
        }
    }
}

// TODO: seek means that there is a waiter waiting for connection. Where should we put the waiter?

// seek by bluetooth. And for different platform, we will implement different logic.
fn seek_by_bluetooth() -> Result<(), Box<dyn Error>> {
    let platform = Platform::Linux;
    match platform {
        Platform::Linux => {
            tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                seek_bluetooth_linux().await;
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
        let discover = adapter.discover_devices().await?;
        pin_mut!(discover);
        let mut done = false;
        while let Some(event) = discover.next().await {
            match event {
                AdapterEvent::DeviceAdded(addr) => {
                    println!("Found device: {addr}");
                    let device = adapter.device(addr)?;
                    match find_tape_characteristic(device).await? {
                        Ok(Some(char)) => {
                            println!("    find tapeos {:?}", char);
                            done = true;
                        },
                        Ok(None) => (),
                        Err(err) => {
                            println!("    Device failed: {}", err);
                            let _ = adapter.remove_device(addr).await;
                        }
                    }
                }
                AdapterEvent::DeviceRemoved(addr) => {
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

async fn find_tape_characteristic(device: Device) -> bluer::Result<bluer::Result<Option<Characteristic>>> {
    let addr = device.address();
    let uuids = device.uuids().await?.unwrap_or_default();
    println!("Discovered device {} with service UUIDs {:?}", addr, &uuids);
    let md = device.manufacturer_data().await?;
    println!("    Manufacturer data: {:x?}", md);

    if uuids.contains(&TAPE_SERVICE_UUID) {
        println!("    Device provides tape service");

        sleep(Duration::from_secs(2)).await;
        if !device.is_connected().await? {
            println!("    Connecting...");
            let mut retries = 2; // TODO: make it configurable
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
            println!("    Connected");
        } else {
            println!("    Already connected");
        }

        for service in device.services().await? {
            if TAPE_SERVICE_UUID == service.uuid().await? {
                for cha in service.characteristics().await? {
                    if TAPE_CHARACTERISTIC_UUID == cha.uuid().await? {
                        let c = cha.clone();
                        store_bluetooth_device(device, cha).await?;
                        return Ok(Ok(Some(c)));
                    }
                }
            }
        }
    }

    Ok(Ok(None))
}

async fn connect_bluetooth_device(adapter: &Adapter, addr: Address) -> bluer::Result<()> {
    let device = adapter.device(addr)?;
    device.connect().await?;
    Ok(())
}

// create new resource and store the bluetooth device properties into the resource pool
async fn store_bluetooth_device(device: Device, char: Characteristic) -> bluer::Result<()> {
    let resource = BluetoothResource::new(
        device.name().await?.unwrap_or_default(), 
        
        Status::new(true, Position::new(0.0, 0.0, 0.0), time::Duration::from_secs(0), device.is_paired().await?, device.is_connected().await?, device.is_trusted().await?, device.is_blocked().await?),
        "".to_string(), 
        vec![], 
        None, 
        HashMap::new(), 
        device.remote_address().await?, 
        device.address_type().await?, 
        device.uuids().await?.unwrap_or_default(), 
        device.alias().await?, 
        device.service_data().await?.unwrap_or_default(), 
        device.class().await?.unwrap_or_default(), 
        device.is_legacy_pairing().await?, 
        device.rssi().await?.unwrap_or_default(), 
        device.is_services_resolved().await?,
        device, 
        char,
    );
    RESOURCES.lock().unwrap().push(Box::new(resource));
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

