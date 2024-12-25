// in this file, we will implement the seeker, which is used to seek the 
// higher level system when not connected.

// we will support multiple seeker, and each seeker will use different strategy to seek the higher level system.
// now we will implement in three strategies:
// 1. bluetooth
// 2. wifi
// 3. internet

use bluer::{Adapter, Device, DiscoveryFilter, DiscoveryTransport};
use futures::{pin_mut, stream::SelectAll, StreamExt};

fn seek() -> bool {
    // TODO: implement the logic to seek the higher level system
    true
}

// TODO: seek means that there is a waiter waiting for connection. Where should we put the waiter?


fn seek_by_bluetooth() -> bool {
    tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(async {
        seek_bluetooth().await;
    });
    // TODO: implement the logic to seek the higher level system by bluetooth
    true
}

async fn seek_bluetooth_linux() -> bluer::Result<()> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    println!("Discovering devices using Bluetooth adapter {}\n", adapter.name());
    adapter.set_powered(true).await?;

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
    let device_events = adapter.discover_devices().await?;
    pin_mut!(device_events);   

    let mut all_change_events = SelectAll::new();

    loop {
        tokio::select! {
            Some(device_event) = device_events.next() => {
                match device_event {
                    AdapterEvent::DeviceAdded(addr) => {
                        if !filter_addr.is_empty() && !filter_addr.contains(&addr) {
                            continue;
                        }

                        println!("Device added: {addr}");
                        let res = if all_properties {
                            query_all_device_properties(&adapter, addr).await
                        } else {
                            query_device(&adapter, addr).await
                        };
                        if let Err(err) = res {
                            println!("    Error: {}", &err);
                        }

                        if with_changes {
                            let device = adapter.device(addr)?;
                            let change_events = device.events().await?.map(move |evt| (addr, evt));
                            all_change_events.push(change_events);
                        }
                    }
                    AdapterEvent::DeviceRemoved(addr) => {
                        println!("Device removed: {addr}");
                    }
                    _ => (),
                }
                println!();
            }
            Some((addr, DeviceEvent::PropertyChanged(property))) = all_change_events.next() => {
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
    let props = device.all_properties().await?;
    let mut resource = Resource::new(
        device.name().await?, 
        Status::Available, 
        device.description().await?, 
        device.command().await?, 
        None,
        props);
    Ok(())
}

fn seek_by_wifi() -> bool {
    // TODO: implement the logic to seek the higher level system by wifi
    true
}

fn seek_by_internet() -> bool {
    // TODO: implement the logic to seek the higher level system by internet
    true
}

