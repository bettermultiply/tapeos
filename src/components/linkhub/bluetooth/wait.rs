// wait by bluetooth. And for different platform, we will implement different logic.

use bluer::{
    adv::Advertisement,
    gatt::{
        local::{
            Service,
            Characteristic, CharacteristicControlEvent,
            CharacteristicWrite, CharacteristicWriteMethod,
            CharacteristicNotify, CharacteristicNotifyMethod, 
            characteristic_control, service_control, Application, 
        },
        CharacteristicReader,
    }, 
    AdapterEvent, Device, Address
};

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    time::{interval, sleep},
};

use std::{
    error::Error, 
    time::Duration, 
    collections::{HashMap, BTreeMap}
};
use futures::{future, pin_mut, StreamExt};

use crate::{
    base::intent::{Intent, IntentSource, IntentType},
    components::linkhub::{bluetooth::resource::BluetoothResource, seeker::{send_intent, INTENT_QUEUE}, waiter::{ResourceType, TAPE, WAIT_RECV}},
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

// wait only execute the seeker request and tapeos request.
pub fn wait() -> Result<(), Box<dyn Error>> {
    match PLATFORM {
        Platform::Linux => {
            tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                match wait_bluetooth_linux().await {
                    Ok(_) => (),
                    Err(err) => {
                        println!("wait bluetooth failed: {}", &err);
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

// include!("gatt.inc")
const TAPE_SERVICE_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00001523_1212_efde_1523_785feabcd123);
// Characteristic UUID for tapeos
const TAPE_CHARACTERISTIC_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00001524_1212_efde_1523_785feabcd123);
// Manufaturer id for LE advertise.
#[allow(dead_code)]
const TAPE_MANUFACTURER_ID: u16 = 0xf00d;

async fn wait_bluetooth_linux() -> bluer::Result<()> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    let mut manufacturer_data = BTreeMap::new();
    manufacturer_data.insert(TAPE_MANUFACTURER_ID, vec![0x21, 0x22, 0x23, 0x24]);
    let le_advertisement = Advertisement {
        service_uuids: vec![TAPE_SERVICE_UUID].into_iter().collect(),
        manufacturer_data,
        discoverable: Some(true),
        local_name: Some("tapeos".to_string()),
        ..Default::default()
    };
    let adv_handle = adapter.advertise(le_advertisement).await?;
    let (_, service_handle) = service_control();
    let (char_control, char_handle) = characteristic_control();
    let app = Application {
        services: vec![Service {
            uuid: TAPE_SERVICE_UUID,
            primary: true,
            characteristics: vec![Characteristic {
                uuid: TAPE_CHARACTERISTIC_UUID,
                write: Some(CharacteristicWrite {
                    write: true,
                    write_without_response: true,
                    method: CharacteristicWriteMethod::Io,
                    ..Default::default()
                }),
                notify: Some(CharacteristicNotify {
                    notify: true,
                    method: CharacteristicNotifyMethod::Io,
                    ..Default::default()
                }),
                control_handle: char_handle,
                ..Default::default()
            }],
            control_handle: service_handle,
            ..Default::default()
        }],
        ..Default::default()
    };
    // drop the app_handle to unregister the advertisement.
    let app_handle = adapter.serve_gatt_application(app).await?;

    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    let mut value: Vec<u8> = vec![];
    let mut read_buf = Vec::new();
    let mut reader_opt: Option<CharacteristicReader> = None;
    let mut interval = interval(Duration::from_secs(1));
    let device_events = adapter.discover_devices().await?;
    pin_mut!(device_events);
    pin_mut!(char_control);
    // -----------
    // init finished
    // -----------

    loop {
        tokio::select! {
            // exit the loop
            _ = lines.next_line() => break,
            // handle the characteristic control event
            evt = char_control.next() => {
                match evt {
                    Some(CharacteristicControlEvent::Write(req)) => {
                        println!("Accepting write event with MTU {} from {}", req.mtu(), req.device_address());
                        read_buf = vec![0; req.mtu()];
                        reader_opt = Some(req.accept()?);
                    },
                    _ => (),
                }
            }
            // handle the device connect and disconnect
            Some(device_event) = device_events.next() => {
                match device_event {
                    AdapterEvent::DeviceAdded(addr) => {
                        let device = adapter.device(addr)?;
                        store_bluetooth_tape(device).await?;
                    },
                    AdapterEvent::DeviceRemoved(addr) => {
                        remove_tape(addr).await?;
                    },
                    _ => (),
                }
            }
            // check device itself initiative action
            _ = interval.tick(), if TAPE.lock().unwrap().is_some() => {
                check_device();
            }
            // handle .
            request = async {
                match WAIT_RECV.lock().unwrap().as_ref().unwrap().try_recv() {
                    Ok(v) => v,
                    Err(err) => {
                        println!("wait: receive waiter request failed: {}", &err);
                        future::pending().await
                    }
                }
            } => {
                execute_seeker_request(request).await;
            }
            // handle tapeos intent
            read_res = async {
                match &mut reader_opt {
                    Some(reader) => reader.read(&mut read_buf).await,
                    None => future::pending().await,
                }} => {
                match read_res {
                    Ok(0) => {
                        println!("Write stream ended");
                        let mut intent = parse_to_intent(&value);
                        handler(& mut intent).await;
                        INTENT_QUEUE.lock().unwrap().push(intent);
                        reader_opt = None;
                    }
                    Ok(n) => {
                        value.extend_from_slice(&read_buf[0..n]);
                    }
                    Err(err) => {
                        println!("Write stream error: {}", &err);
                        reader_opt = None;
                    }
                }
            }
        }
    }

    println!("Removing service and advertisement");
    drop(app_handle);
    drop(adv_handle);
    sleep(Duration::from_secs(1)).await;

    Ok(())
}

fn check_device() {
    println!("check device itself initiative action");
}

fn parse_to_intent(value: &Vec<u8>) -> Intent {
    Intent::new(String::from_utf8(value.clone()).unwrap(), IntentSource::Tape, IntentType::Intent, None)
}

async fn execute_seeker_request(request: String) {
    let map = parse_request(request);
    
    for (key, value) in map {
        match key.as_str() {
            "Intent" => {
                send_intent("TAPE".to_string(), value, 0).await.unwrap();
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

async fn store_bluetooth_tape(device: Device) -> bluer::Result<()> {
    
    let props = device.all_properties().await?;
    for service in device.services().await? {
        let uuid = service.uuid().await?;
        if uuid == TAPE_SERVICE_UUID {
            for cha in service.characteristics().await? {
                let uuid = cha.uuid().await?;
                if uuid == TAPE_CHARACTERISTIC_UUID {
                    let resource = BluetoothResource::new(
                        device.name().await?.unwrap_or_default(),
                        device,
                        props,
                        Some(service),
                        Some(cha),
                    ); 
                    
                    TAPE.lock().unwrap().replace(ResourceType::Bluetooth(resource));
                    return Ok(());
                }
            }
        }
    }
    Ok(())
}

async fn remove_tape(address: Address) -> bluer::Result<()> {
    TAPE.lock().unwrap().take();

    println!("Device removed: {:?}", address);
    Ok(())
}
