// in this file, we will maintain the information and status of resources
// and provide the interface for other components to access and update the 
// information.

use std::{net::SocketAddr, path::PathBuf, time::Duration};

use bluer::Address;
use serde::{Deserialize, Serialize};


// resource is a physical or virtual device(including human and software), 
// which can be used to execute intents. However, it may not be able to 
// process intents directly, so we need an interpreter to interpret the 
//intent and then execute it. *subsystem* is a special resource, which can 
// process intents, which means it do not need an interpreter to interpret 
// the intent.

// we will display a virtual resource as model
// struct Resource {
//     name: String,
//     type_name: String,
//     description: String,
//     address: ResourceAddress,
//     status: Status,
//     command: Vec<String>,
//     interpreter: PathBuf,
// }

pub trait Resource: Send + Sync {
    fn get_name(&self) -> &str;
    fn get_description(&self) -> &str;
    fn get_address(&self) -> ResourceAddress;
    fn get_status(&mut self) -> &mut Status;
    fn display_status(&self) -> String;

    fn set_status(&mut self, status: Status);
    fn set_interpreter(&mut self, interpreter: Interpreter);
    fn set_description(&mut self, description: String);

    fn is_interpreter_none(&self) -> bool;
}

#[derive(PartialEq, Eq)]
pub enum ResourceAddress {
    Bluetooth(Address),
    Wifi(String),
    Internet(SocketAddr),
}

#[derive(Serialize, Deserialize)]
pub enum Interpreter {
    PathBuf(PathBuf),
    LLM(String),
    Unknow,
    None,
}

// Status is unique for each resource. However, there are some common statuses.
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub fn new(aviliability: bool, position: (f32, f32, f32), busy_time: Duration) -> Self {
        Self { aviliability, position: Position::new(position.0, position.1, position.2), busy_time }
    }

    fn get_aviliability(&self) -> bool {
        self.aviliability
    }

    fn get_position(&self) -> &Position {
        &self.position
    }

    pub fn set_aviliability(&mut self, aviliability: bool) {
        self.aviliability = aviliability;
    }

    pub fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    pub fn set_busy_time(&mut self, busy_time: Duration) {
        self.busy_time = busy_time;
    }
}

// position is a common field for all resources.
// it is a 3D vector, which can be used to describe the position of the resource.
#[derive(Debug, Serialize, Deserialize, Clone)]
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

    pub fn new_from_vec(position: Vec<f32>) -> Self {
        Self { x: position[0], y: position[1], z: position[2] }
    }
}

// we do not need such function, instead we will use hashmap.
// pub fn find_resource<'a>(device_name: String)

