use std::net::SocketAddr;

use crate::base::resource::{Interpreter, Resource, ResourceAddress, Status};

pub struct InternetResource {
    name: String,
    description: String,
    address: SocketAddr,
    status: Status,
    command: Vec<String>,
    interpreter: Interpreter,
}

impl Resource for InternetResource {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_address(&self) -> ResourceAddress {
        ResourceAddress::Internet(self.address)
    }

    fn get_status(&mut self) -> &mut Status {
        &mut self.status
    }   

    fn get_description(&self) -> &str {
        &self.description
    }

    fn get_command(&self) -> &Vec<String> {
        &self.command
    }

    fn display_status(&self) -> String {
        format!("{:?}", self.status)
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
    }

    fn send_intent(&self, intent_description: &str) {
        
    }

}
