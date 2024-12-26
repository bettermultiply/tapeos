// in this file, we will implement the seeker, which is used to seek the 
// higher level system when not connected.

// we will support multiple seeker, and each seeker will use different strategy to seek the higher level system.
// now we will implement in three strategies:
// 1. bluetooth
// 2. wifi
// 3. internet

use bluer::{Adapter, AdapterEvent, Address, DeviceEvent, DiscoveryFilter, DiscoveryTransport};
use futures::{pin_mut, stream::SelectAll, StreamExt};
use std::{collections::HashSet, env, error::Error};
use crate::base::resource::{BluetoothResource, Status, Position, RESOURCES};
use std::collections::HashMap;
use std::time;

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
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?; // -> ErrorKind::NotFound
    println!("Discovering devices using Bluetooth adapter {}\n", adapter.name());
    adapter.set_powered(true).await?; // the value of this property is not persistent.

    // TODO: these parameters should be set by the user.
    // And we need a configuration file to store these parameters.
    let le_only = false;
    let br_edr_only = false;
    let filter = DiscoveryFilter {
        transport: if le_only {
            DiscoveryTransport::Le
        } else if br_edr_only {
            DiscoveryTransport::BrEdr
        } else {
            DiscoveryTransport::Auto
        },
        ..Default::default()
    };
    adapter.set_discovery_filter(filter).await?;
    println!("Using discovery filter:\n{:#?}\n\n", adapter.discovery_filter().await);

    // start to discover the devices
    // all already known devices are also included in the stream. we can check the Device::rssi peoperty to see if it is already known.
    let device_events = adapter.discover_devices().await?;
    pin_mut!(device_events);
     
    // TODO: these parameters should be set by the user.
    // And we need a configuration file to store these parameters.
    let filter_addr: HashSet<_> = env::args().filter_map(|arg| arg.parse::<Address>().ok()).collect();

    let mut all_change_events = SelectAll::new();

    // TODO: leave the loop in specific time.
    loop {
        tokio::select! {
            Some(device_event) = device_events.next() => {
                match device_event {
                    // Add and remove operations are happened for adapter.
                    AdapterEvent::DeviceAdded(addr) => {
                        if !filter_addr.is_empty() && !filter_addr.contains(&addr) {
                            continue;
                        }

                        println!("Device added: {addr}");
                        store_bluetooth_device(&adapter, addr).await?;
                        let device = adapter.device(addr)?;
                        let change_events = device.events().await?.map(move |evt| (addr, evt));
                        all_change_events.push(change_events);
                    }
                    AdapterEvent::DeviceRemoved(addr) => {
                        // TODO: remove the device from the resource pool.
                        println!("Device removed: {addr}");
                    }
                    _ => (),
                }
            }
            Some((addr, DeviceEvent::PropertyChanged(property))) = all_change_events.next() => {
                // TODO: update the device properties in the resource pool.
                println!("Device changed: {addr}");
                println!("    {property:?}");
            }
            else => break
        }
    }

    Ok(())
}

// create new resource and store the bluetooth device properties into the resource pool
async fn store_bluetooth_device(adapter: &Adapter, addr: Address) -> bluer::Result<()> {
    let device = adapter.device(addr)?;
    // TODO: Maybe the better way is to use props.
    // let props = device.all_properties().await?;
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
        device.is_services_resolved().await?
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

