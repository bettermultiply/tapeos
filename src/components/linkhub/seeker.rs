// in this file, we will implement the seeker, which is used to seek the 
// higher level system when not connected.

// we will support multiple seeker, and each seeker will use different strategy to seek the higher level system.
// now we will implement in three strategies:
// 1. bluetooth
// 2. wifi
// 3. internet

use std::error::Error;
use crate::components::linkhub::seek::{bluetooth, wifi, internet};

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
pub enum Platform {
    Linux,
    Windows,
    Mac,
    Android,
    IOS,
}

const SEEK_METHOD: SeekMethod = SeekMethod::Bluetooth;
pub const PLATFORM: Platform = Platform::Linux;

// seek resources and subsystems depend on the SEEK_METHOD.
pub fn seek() -> Result<(), Box<dyn Error>> {
    match SEEK_METHOD {
        SeekMethod::Bluetooth => bluetooth::seek(),
        SeekMethod::Wifi => wifi::seek(),
        SeekMethod::Internet => internet::seek(),
        _ => {
            return Err("Unsupported seek method".into());
        }
    }
}


