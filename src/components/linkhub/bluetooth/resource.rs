use std::{path::PathBuf, time::Duration};
use bluer::{
    Address, Device, DeviceProperty, 
    gatt::remote::{Characteristic, Service}
};
use crate::base::resource::{Interpreter, Resource, ResourceAddress, Status};

pub struct BluetoothResource {
    name: String,
    description: String,
    address: Address,
    status: Status,
    command: Vec<String>,
    interpreter: Interpreter, 


    device: Device,
    props: Vec<DeviceProperty>,
    service: Option<Service>,
    char: Option<Characteristic>,
}

impl BluetoothResource {
    pub fn new(
        name: String, device: Device, props: Vec<DeviceProperty>, service: Option<Service>, char: Option<Characteristic>
    ) -> Self {
        Self {
            name,
            address: device.address(),
            device, props, service, char, 
            status: Status::new(true, (0.0, 0.0, 0.0), Duration::from_secs(0)), 
            description: "".to_string(), 
            command: Vec::new(), 
            interpreter: Interpreter::PathBuf(PathBuf::new()), 
        }
    }

    pub fn get_props(&self) -> &Vec<DeviceProperty> {
        &self.props
    }

    pub fn get_service(&self) -> &Option<Service> {
        &self.service
    }

    pub fn get_device(&self) -> &Device {
        &self.device
    }

    pub async fn get_address(&self) -> Address {
        self.device.address()
    }

    pub fn get_char(&self) -> &Option<Characteristic> {
        &self.char
    }

}


#[allow(unused)]
impl Resource for BluetoothResource {
    fn get_name(&self) -> &str {
        for prop in self.props.iter() {
            match prop {
                DeviceProperty::Name(name) => return name,
                _ => (),
            }
        }
        return "";
    }

    fn get_address(&self) -> ResourceAddress {
        ResourceAddress::Bluetooth(self.address)
    }

    fn get_status(&mut self) -> &mut Status {
        &mut self.status
    }

    fn display_status(&self) -> String {
        format!("{:?}", self.status)
    }

    fn get_description(&self) -> &str {
        &self.description
    }

    fn get_command(&self) -> &Vec<String> {
        &self.command
    }

    fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    fn set_command(&mut self, command: Vec<String>) {
        self.command = command;
    }

    fn set_interpreter(&mut self, interpreter: Interpreter) {
        self.interpreter = interpreter;
    }

    fn set_description(&mut self, description: String) {
        self.description = description;
    }

    fn reject_intent(&self, intent_description: &str) {
        let char = self.get_char().as_ref().unwrap();
        let reject = "Reject:".to_string() + intent_description;
        let data: Vec<u8> = reject.as_bytes().to_vec();
        char.write(&data);
    }

    fn send_intent(&self, intent_description: &str) {
        let char = self.get_char().as_ref().unwrap();
        let intent = "Intent:".to_string() + intent_description;
        let data: Vec<u8> = intent.as_bytes().to_vec();
        char.write(&data);
    }

}