use std::{fmt, time::Duration};
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
    interpreter: Interpreter, 


    device: Device,
    props: Vec<DeviceProperty>,
    service: Option<Service>,
    char: Option<Characteristic>,
}

impl fmt::Display for BluetoothResource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}/{};", self.get_name(), self.get_description(), self.display_status())
    }
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
            interpreter: Interpreter::None, 
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
        &self.name
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

    fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    fn set_interpreter(&mut self, interpreter: Interpreter) {
        self.interpreter = interpreter;
    }

    fn set_description(&mut self, description: String) {
        self.description = description;
    }

    fn is_interpreter_none(&self) -> bool {
        match self.interpreter {
            Interpreter::None => {true},
            _ => {false},
        }
    }
}