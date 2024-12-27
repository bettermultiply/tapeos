// this file is used to wait for the resource and subsystem to connect.
// when the resource and subsystem are querying to connect, 
// the waiter will store the information of the resource or subsystem.
// and maintain the connection.

use bluer::{
    adv::Advertisement,
    gatt::{
        local::{
            characteristic_control, service_control, Application, Characteristic,
            CharacteristicNotify, CharacteristicNotifyMethod, CharacteristicWrite, CharacteristicWriteMethod,
            Service,
        },
        CharacteristicReader, CharacteristicWriter,
    }, 
    AdapterEvent,
    Device,
};

use futures::{future, pin_mut, StreamExt};
use std::{collections::BTreeMap, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    time::{interval, sleep},
};
use crate::base::resource::RESOURCES;
use crate::base::resource::BluetoothResource;

async fn store_resource(device: Device) -> bluer::Result<()> {
    let props = device.all_properties().await?;
    for service in device.services().await? {
        let uuid = service.uuid().await?;
        if uuid == TAPE_SERVICE_UUID {
            for cha in service.characteristics().await? {
                let uuid = cha.uuid().await?;
                if uuid == TAPE_CHARACTERISTIC_UUID {
                    let resource = BluetoothResource::new(
                        device,
                        props,
                        Some(service),
                        Some(cha),
                    );
                    RESOURCES.lock().unwrap().push(Box::new(resource));
                    return Ok(());
                }
            }
        }
    }
    Ok(())
}

async fn remove_resource(device: Device) -> bluer::Result<()> {
    // TODO: remove the resource from the RESOURCES.
    println!("device removed uuid: {:?}", device.uuids().await?);
    for (i, resource) in RESOURCES.lock().unwrap().iter_mut().enumerate() {
        if resource.get_id() == 0 {
            RESOURCES.lock().unwrap().remove(i);
            return Ok(());
        }
    }
    Ok(())
}

// include!("gatt.inc")
const TAPE_SERVICE_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00001523_1212_efde_1523_785feabcd123);
// Characteristic UUID for tapeos
const TAPE_CHARACTERISTIC_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00001524_1212_efde_1523_785feabcd123);
// Manufaturer id for LE advertise.
#[allow(dead_code)]
const TAPE_MANUFACTURER_ID: u16 = 0xf00d;


pub async fn waiter() -> bluer::Result<()> {
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
    let (_, char_handle) = characteristic_control();
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
    let mut value: Vec<u8> = vec![0x10, 0x01, 0x01, 0x10];
    let mut read_buf = Vec::new();
    let mut reader_opt: Option<CharacteristicReader> = None;
    let mut writer_opt: Option<CharacteristicWriter> = None;
    let mut interval = interval(Duration::from_secs(1));
    let device_events = adapter.discover_devices().await?;
    pin_mut!(device_events);

    loop {
        tokio::select! {
            _ = lines.next_line() => break,
            Some(device_event) = device_events.next() => {
                match device_event {
                    AdapterEvent::DeviceAdded(addr) => {
                        let device = adapter.device(addr)?;
                        store_resource(device);
                    },
                    AdapterEvent::DeviceRemoved(addr) => {
                        let device = adapter.device(addr)?;
                        remove_resource(device);
                    },
                    _ => (),
                }
            }
            _ = interval.tick() => {
                for v in &mut *value {
                    *v = v.saturating_sub(1);
                }
                if let Some(writer) = writer_opt.as_mut() {
                    println!("Notifying with value {:x?}", &value);
                    if let Err(err) = writer.write(&value).await {
                        println!("Notification stream error: {}", &err);
                        writer_opt = None;
                    }
                }
            }
            read_res = async {
                match &mut reader_opt {
                    Some(reader) => reader.read(&mut read_buf).await,
                    None => future::pending().await,
                }
            } => {
                match read_res {
                    Ok(0) => {
                        println!("Write stream ended");
                        reader_opt = None;
                    }
                    Ok(n) => {
                        value = read_buf[0..n].to_vec();
                        println!("Write request with {} bytes: {:x?}", n, &value);
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
