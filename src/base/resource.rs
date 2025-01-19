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
    fn get_status(&mut self) -> &mut Status;
    fn get_address(&self) -> ResourceAddress;
    fn get_interpreter(&self) -> &Interpreter;
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
    Classification(Vec<String>), // all avaiable command here. 
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
    // dealing means resource now dealing inten number.
    dealing: u8,
    // busy_time shows how much time the resource need to execute next intent.
    busy_time: Duration, 

    average_time: Duration, // every average time is calculate by 0.8x(average_time) + 0.2x(busy_time/dealing)  
}

impl Status {
    pub fn new(aviliability: bool, position: (f32, f32, f32), busy_time: Duration) -> Self {
        Self { 
            aviliability, 
            position: Position::new(position.0, position.1, position.2), 
            dealing: 0, 
            busy_time,
            average_time: Duration::from_secs(5),
        }
    }

    pub fn get_aviliability(&self) -> bool {
        self.aviliability
    }

    pub fn get_position(&self) -> &Position {
        &self.position
    }

    pub fn get_dealing(&self) -> u8 {
        self.dealing
    }

    pub fn get_average_time(&self) -> Duration {
        self.average_time
    }

    pub fn set_aviliability(&mut self, aviliability: bool) {
        self.aviliability = aviliability;
    }

    pub fn set_position(&mut self, position: Position) {
        self.position = position;
    }
    
    pub fn set_dealing(&mut self, dealing: u8){
        self.dealing = dealing
    }

    pub fn change_average_time(&mut self, last_execute: Duration) {
        self.average_time = self.average_time.mul_f32(0.9) + last_execute.mul_f32(0.1)
    }

    pub fn get_busy_time(&mut self) -> Duration {
        self.busy_time
    }

    pub fn set_busy_time(&mut self, busy_time: Duration) {
        self.busy_time = busy_time;
    }

    pub fn add_busy_time(&mut self, busy_time: Duration) {
        self.busy_time += busy_time;
    }

    pub fn change_dealing(&mut self, op: bool) {
        if op {
            self.dealing += 1;
        } else {
            self.dealing -= 1;
        }
    }
    
    pub fn sub_busy_time(&mut self, busy_time: Duration) {
        self.busy_time -= busy_time;
    }
}

// position is a common field for all resources.
// it is a 3D vector, which can be used to describe the position of the resource.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
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

#[derive(Serialize, Deserialize)]
pub struct RegisterServer {
    tape: bool,
    iaddr: Option<SocketAddr>,
    oaddr: Option<SocketAddr>,
    x_dis: (f32, f32),
    y_dis: (f32, f32),
    z_dis: (f32, f32),
}

impl RegisterServer {
    pub fn new(
        tape: bool, 
        iaddr: Option<SocketAddr>, 
        oaddr: Option<SocketAddr>, 
        dis: ((f32, f32), (f32, f32), (f32, f32)), 
    ) -> Self {
        Self {
            tape,
            iaddr,
            oaddr,
            x_dis: dis.0,
            y_dis: dis.1,
            z_dis: dis.2,
        }
    }

    pub fn is_tape(&self) -> bool {
        self.tape
    }

    pub fn get_iaddr(&self) -> SocketAddr {
        self.iaddr.unwrap().clone()
    }

    pub fn get_oaddr(&self) -> SocketAddr {
        self.oaddr.unwrap().clone()
    }

    pub fn is_position_suit(&self, p: &RegisterServer) -> bool {
        let v_position = ((-100.0, 100.0), (-100.0, 100.0), (-100.0, 100.0));

            p.x_dis.0 > (v_position.0).0 
        &&  p.x_dis.1 < (v_position.0).1
        &&  p.y_dis.0 > (v_position.1).0
        &&  p.y_dis.1 < (v_position.1).1
        &&  p.z_dis.0 > (v_position.2).0
        &&  p.z_dis.1 < (v_position.2).1
    }
}