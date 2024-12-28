// in this file, we will maintain the information and status of resources
// and provide the interface for other components to access and update the 
// information.


use crate::components::controlhub::interpreter::Interpreter;
use std::time::Duration;
use std::sync::Mutex;
use bluer::Address;
use lazy_static::lazy_static;
use bluer::{Device, gatt::remote::Characteristic, DeviceProperty, gatt::remote::Service};
use crate::base::intent::{Intent, SubIntent};
lazy_static! {
    pub static ref RESOURCES: Mutex<Vec<Box<ResourceType>>> = Mutex::new(Vec::new());
}

pub type ResourceType = BluetoothResource;

// resource is a physical or virtual device(including human and software), 
// which can be used to execute intents. However, it may not be able to 
// process intents directly, so we need an interpreter to interpret the 
//intent and then execute it. *subsystem* is a special resource, which can 
// process intents, which means it do not need an interpreter to interpret 
// the intent.
pub trait Resource: Send + Sync {
    fn get_type_name(&self) -> &str;
    fn get_status(&self) -> &Status;
    fn get_description(&self) -> &str;
    fn get_command(&self) -> &Vec<String>;

    fn set_type_name(&mut self, type_name: String);
    fn set_status(&mut self, status: Status);
    fn set_command(&mut self, command: Vec<String>);
    fn set_interpreter(&mut self, interpreter: Option<Box<dyn Interpreter>>);
    fn set_description(&mut self, description: String);

    // send intent to the resource. tape->resource(include tape).
    fn send_intent(&self, intent: &SubIntent);
    // reject intent to the source. tape->source.
    fn reject_intent(&self, intent: &Intent);
    // send response to the source. tape->source.
    fn send_response(&self, response: &Intent);
    // query the resource's status.
    fn query_status(&self);
    // tell the resource to execute the intent.
    fn execute_intent(&self, intent: &SubIntent);
}

#[allow(unused)]
pub struct BluetoothResource {
    // id is a unique identifier for the resource, can't be changed.
    address: Address,
    // different from device name, type name shows the kind of the resource.
    type_name: String,
    device: Device,
    // props is the properties of the device. 
    // although it can be get from device
    // however, do not need async here.
    props: Vec<DeviceProperty>,

    service: Option<Service>,

    char: Option<Characteristic>,
    // status is unique for each resource.
    status: Status,
    // description is a brief description of the resource.
    description: String,
    // command is a list of commands that the resource can execute.
    command: Vec<String>,
    // interpreter is a trait that can be implemented by different 
    // interpreters. For subsystems, this field is set to None.
    interpreter: Option<Box<dyn Interpreter>>, 
}

#[allow(unused)]
impl BluetoothResource {
    pub fn new(device: Device, props: Vec<DeviceProperty>, service: Option<Service>, char: Option<Characteristic>) -> Self {
        Self { 
            address: device.address(),
            type_name: "bluetooth".to_string(), 
            device, props, service, char, 
            status: Status {
                aviliability: true, 
                position: Position::new(0.0, 0.0, 0.0), 
                busy_time: Duration::from_secs(0)
            }, 
            description: "".to_string(), 
            command: Vec::new(), 
            interpreter: None 
        }
    }

    pub fn compare_address(&self, address: Address) -> bool {
        self.address == address
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
    fn get_type_name(&self) -> &str {
        &self.type_name
    }

    fn get_status(&self) -> &Status {
        &self.status
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

    fn set_type_name(&mut self, type_name: String) {
        self.type_name = type_name;
    }

    fn set_command(&mut self, command: Vec<String>) {
        self.command = command;
    }

    fn set_interpreter(&mut self, interpreter: Option<Box<dyn Interpreter>>) {
        self.interpreter = interpreter;
    }

    fn set_description(&mut self, description: String) {
        self.description = description;
    }

    fn send_intent(&self, intent: &SubIntent) {
        // TODO: implement the logic to send intent to the resource.
    }

    fn reject_intent(&self, intent: &Intent) {
        // TODO: implement the logic to reject intent to the source.
    }

    fn send_response(&self, response: &Intent) {
        // TODO: implement the logic to send response to the source.
    }

    fn query_status(&self) {
        // TODO: implement the logic to query the resource's status.
    }

    fn execute_intent(&self, intent: &SubIntent) {
        // TODO: implement the logic to execute the intent.
    }
}

// Status is unique for each resource. However, there are some common statuses.
#[allow(unused)]
pub struct Status {
    // aviliability shows the resource is available or not.
    aviliability: bool,
    // position shows the resource's position.
    position: Position,
    // busy_time shows how much time the resource need to execute next intent.
    busy_time: Duration, // TODO: do duration here ok? I need a specific type to describe the time.
}

#[allow(unused)]
impl Status {
    pub fn new(aviliability: bool, position: Position, busy_time: Duration) -> Self {
        Self { aviliability, position, busy_time }
    }

    fn get_aviliability(&self) -> bool {
        self.aviliability
    }

    fn get_position(&self) -> &Position {
        &self.position
    }
}

// position is a common field for all resources.
// it is a 3D vector, which can be used to describe the position of the resource.
#[allow(unused)]
pub struct Position {
    x: f32,
    y: f32,
    z: f32,
} // TODO: make three dimension more flexible,
    // so that it can be used to describe discrete position like floor and room.

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}
