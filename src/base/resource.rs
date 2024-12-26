// in this file, we will maintain the information and status of resources
// and provide the interface for other components to access and update the 
// information.

use bluer::{Address, Uuid, AddressType};

use crate::components::controlhub::interpreter::Interpreter;
use crate::tools::idgen::{generate_id, IdType};
use std::time::Duration;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use lazy_static::lazy_static;

use bluer::{Device, gatt::remote::Characteristic};

lazy_static! {
    pub static ref RESOURCES: Mutex<Vec<Box<dyn Resource>>> = Mutex::new(Vec::new());
}

// resource is a physical or virtual device(including human and software), 
// which can be used to execute intents. However, it may not be able to 
// process intents directly, so we need an interpreter to interpret the 
//intent and then execute it. *subsystem* is a special resource, which can 
// process intents, which means it do not need an interpreter to interpret 
// the intent.
pub trait Resource: Send + Sync {
    fn get_id(&self) -> i64;
    fn get_name(&self) -> &str;
    fn get_status(&self) -> &Status;
    fn get_description(&self) -> &str;
    fn get_command(&self) -> &Vec<String>;
    fn set_status(&mut self, status: Status);
    fn set_command(&mut self, command: Vec<String>);
    fn set_interpreter(&mut self, interpreter: Option<Box<dyn Interpreter>>);
}

pub(crate) struct BluetoothResource {
    // id is a unique identifier for the resource, can't be changed.
    id: i64,

    device: Device,

    char: Characteristic,
    // name can be easily identified by the user.
    name: String,
    // remote_address is the address of the resource.
    remote_address: Address,
    // uuids is the uuids of the resource.
    uuids: HashSet<Uuid>,
    // alias is the alias of the resource.
    alias: String,
    // service_data is the service data of the resource.
    service_data: HashMap<bluer::Uuid, Vec<u8>>,
    // status is unique for each resource.
    status: Status,
    // description is a brief description of the resource.
    description: String,
    // command is a list of commands that the resource can execute.
    command: Vec<String>,
    // interpreter is a trait that can be implemented by different 
    // interpreters. For subsystems, this field is set to None.
    interpreter: Option<Box<dyn Interpreter>>, 
    // properties is a map of properties of the resource.
    properties: HashMap<String, String>,
    // address_type is the type of the address of the resource.
    address_type: AddressType,
    // class is the class of the resource.
    class: u32,
    // legacy_pairing is the legacy pairing of the resource.
    legacy_pairing: bool,
    // rssi is the rssi of the resource.
    rssi: i16,
    // service_resolved is the service resolved of the resource.
    service_resolved: bool,
    // TODO: other fields as needed


}

impl BluetoothResource {
    pub fn new(name: String, status: Status, description: String, 
        command: Vec<String>, interpreter: Option<Box<dyn Interpreter>>, 
        properties: HashMap<String, String>, remote_address: Address, 
        address_type: AddressType, uuids: HashSet<Uuid>, alias: String, 
        service_data: HashMap<Uuid, Vec<u8>>, class: u32, legacy_pairing: bool, 
        rssi: i16, service_resolved: bool, device: Device, char: Characteristic) -> Self {
        Self { id: generate_id(IdType::Resource), name, status, description, command, interpreter, properties, remote_address, address_type, uuids, alias, service_data, class, legacy_pairing, rssi, service_resolved, device, char }
    }

    pub fn get_remote_address(&self) -> &Address {
        &self.remote_address
    }

    pub fn get_address_type(&self) -> &AddressType {
        &self.address_type
    }

    pub fn get_uuids(&self) -> &HashSet<Uuid> {
        &self.uuids
    }

    pub fn get_alias(&self) -> &str {
        &self.alias
    }

    pub fn get_service_data(&self) -> &HashMap<Uuid, Vec<u8>> {
        &self.service_data
    }

    pub fn get_class(&self) -> u32 {
        self.class
    }

    pub fn get_legacy_pairing(&self) -> bool {
        self.legacy_pairing
    }   

    pub fn get_rssi(&self) -> i16 {
        self.rssi
    }

    pub fn get_service_resolved(&self) -> bool {
        self.service_resolved
    }

    pub fn set_remote_address(&mut self, remote_address: Address) {
        self.remote_address = remote_address;
    }

    pub fn set_address_type(&mut self, address_type: AddressType) {
        self.address_type = address_type;
    }

    pub fn set_uuids(&mut self, uuids: HashSet<Uuid>) {
        self.uuids = uuids;
    }

    pub fn set_alias(&mut self, alias: String) {
        self.alias = alias;
    }   

    pub fn set_service_data(&mut self, service_data: HashMap<Uuid, Vec<u8>>) {
        self.service_data = service_data;
    }

    pub fn set_class(&mut self, class: u32) {
        self.class = class;
    }   

    pub fn set_legacy_pairing(&mut self, legacy_pairing: bool) {
        self.legacy_pairing = legacy_pairing;
    }

    pub fn set_rssi(&mut self, rssi: i16) {
        self.rssi = rssi;
    }

    pub fn set_service_resolved(&mut self, service_resolved: bool) {
        self.service_resolved = service_resolved;
    }
}

    impl Resource for BluetoothResource {
    fn get_id(&self) -> i64 {
        self.id
    }

    fn get_name(&self) -> &str {
        &self.name
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

    fn set_command(&mut self, command: Vec<String>) {
        self.command = command;
    }

    fn set_interpreter(&mut self, interpreter: Option<Box<dyn Interpreter>>) {
        self.interpreter = interpreter;
    }

}

// Status is unique for each resource. However, there are some common statuses.
pub(crate) struct Status {
    // aviliability shows the resource is available or not.
    aviliability: bool,
    // paired shows the resource is paired or not.
    paired: bool,
    // connected shows the resource is connected or not.
    connected: bool,
    // trusted shows the resource is trusted or not.
    trusted: bool,
    // blocked shows the resource is blocked or not.
    blocked: bool,
    // position shows the resource's position.
    position: Position,
    // busy_time shows how much time the resource need to execute next intent.
    busy_time: Duration, // TODO: do duration here ok? I need a specific type to describe the time.
    // TODO: other fields as needed
}

impl Status {
    pub fn new(aviliability: bool, position: Position, busy_time: Duration, 
        paired: bool, connected: bool, trusted: bool, blocked: bool) -> Self {
        Self { aviliability, position, busy_time, paired, connected, trusted, blocked }
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
