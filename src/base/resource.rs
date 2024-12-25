// in this file, we will maintain the information and status of resources
// and provide the interface for other components to access and update the 
// information.

use crate::components::controlhub::interpreter::Interpreter;
use crate::tools::idgen::{generate_id, IdType};
use std::time::Duration;

// resource is a physical or virtual device(including human and software), 
// which can be used to execute intents. However, it may not be able to 
// process intents directly, so we need an interpreter to interpret the 
//intent and then execute it. *subsystem* is a special resource, which can 
// process intents, which means it do not need an interpreter to interpret 
// the intent.
pub(crate) struct Resource {
    // id is a unique identifier for the resource.
    id: i64,
    // name can be easily identified by the user.
    name: String,
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
    // TODO: other fields as needed
}   

impl Resource {
    fn new(name: String, status: Status, description: String, 
        command: Vec<String>, interpreter: Option<Box<dyn Interpreter>>, 
        properties: HashMap<String, String>) -> Self {
        Self { id: generate_id(IdType::Resource), 
            name, status, description, command, interpreter, properties }
    }

    pub fn get_id(&self) -> i64 {
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
struct Status {
    // aviliability shows the resource is available or not.
    aviliability: bool,
    // position shows the resource's position.
    position: Position,
    // busy_time shows how much time the resource need to execute next intent.
    busy_time: Duration, // TODO: do duration here ok? I need a specific type to describe the time.
    // TODO: other fields as needed
}

// position is a common field for all resources.
// it is a 3D vector, which can be used to describe the position of the resource.
struct Position {
    x: f32,
    y: f32,
    z: f32,
} // TODO: make three dimension more flexible,
    // so that it can be used to describe discrete position like floor and room.

