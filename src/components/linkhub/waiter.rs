// this file is used to wait for the resource and subsystem to connect.
// when the resource and subsystem are querying to connect, 
// the waiter will store the information of the resource or subsystem.
// and maintain the connection.

use bluer::{
    adv::Advertisement,
    gatt::{
        local::{
            characteristic_control, service_control, Application, Characteristic, CharacteristicControlEvent,
            CharacteristicNotify, CharacteristicNotifyMethod, CharacteristicWrite, CharacteristicWriteMethod,
            Service,
        },
        CharacteristicReader, CharacteristicWriter,
    },
};
use futures::{future, pin_mut, StreamExt};
use std::{collections::BTreeMap, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    time::{interval, sleep},
};
use crate::base::resource::{Resource, RESOURCES};

fn store_resource(resource: impl Resource + 'static) {
    RESOURCES.lock().unwrap().push(Box::new(resource));
}

// need to notice that the subsystem is a special resource.
fn store_subsystem(subsystem: impl Resource + 'static) {
    RESOURCES.lock().unwrap().push(Box::new(subsystem));
}

// include!("gatt.inc")
const TAPE_SERVICE_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00001523_1212_efde_1523_785feabcd123);
// Characteristic UUID for tapeos
const TAPE_CHARACTERISTIC_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00001524_1212_efde_1523_785feabcd123);

// Manufaturer id for LE advertise.
#[allow(dead_code)]
const TAPE_MANUFACTURER_ID: u16 = 0xf00d;


pub async fn waiter() -> bluer::Result<()> {
    // TODO: wait for the resource and subsystem to connect.

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
    let app_handle = adapter.serve_gatt_application(app).await?;

    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    let mut value: Vec<u8> = vec![0x10, 0x01, 0x01, 0x10];
    let mut read_buf = Vec::new();
    let mut reader_opt: Option<CharacteristicReader> = None;
    let mut writer_opt: Option<CharacteristicWriter> = None;
    let mut interval = interval(Duration::from_secs(1));
    pin_mut!(char_control);

    loop {
        tokio::select! {
            _ = lines.next_line() => break,
            evt = char_control.next() => {
                match evt {
                    Some(CharacteristicControlEvent::Write(req)) => {
                        read_buf = vec![0; req.mtu()];
                        reader_opt = Some(req.accept()?);
                    },
                    Some(CharacteristicControlEvent::Notify(notifier)) => {
                        writer_opt = Some(notifier);
                    },
                    None => break,
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
