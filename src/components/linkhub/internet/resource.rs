use std::{fmt, net::SocketAddr};

use crate::base::resource::{Interpreter, Resource, ResourceAddress, Status};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct InternetResource {
    usage: i8,
    name: String,
    status: Status,
    description: String,
    address: SocketAddr,
    interpreter: Interpreter,
}

impl InternetResource {
    pub fn new(name: String, description: String, address: SocketAddr, status: Status) -> Self {
        Self {
            usage: 0, name, description, address, status, interpreter: Interpreter::None
        }
    }

    pub fn get_address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn set_address(&mut self, addr: SocketAddr) {
        self.address = addr;
    }
}

impl fmt::Display for InternetResource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}/{};", self.get_name(), self.get_description(), self.display_status())
    }
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

    fn display_status(&self) -> String {
        format!("{:?}", self.status)
    }

    fn set_status(&mut self, status: Status) {
        self.status = status
    }

    fn set_interpreter(&mut self, interpreter: Interpreter) {
        self.interpreter = interpreter;
    }

    fn set_description(&mut self, description: String) {
        self.description = description;
    }

    fn get_interpreter(&self) -> &Interpreter {
        &self.interpreter
    }

    fn is_interpreter_none(&self) -> bool {
        match self.interpreter {
            Interpreter::None => {true},
            _ => {false},
        }
    }

    fn change_usage(&mut self, usage: i8) {
        self.usage += usage;
    }

    fn get_usage(&self) -> i8 {
        self.usage
    }
}
