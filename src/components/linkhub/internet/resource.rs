use std::net::SocketAddr;

use crate::base::resource::{Interpreter, Resource, ResourceAddress, Status};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct InternetResource {
    name: String,
    description: String,
    address: SocketAddr,
    status: Status,
    interpreter: Interpreter,
}

impl InternetResource {
    pub fn new(name: String, description: String, address: SocketAddr, status: Status) -> Self {
        Self {
            name, description, address, status, interpreter: Interpreter::Unknow
        }
    }

    pub fn get_address(&self) -> &SocketAddr {
        &self.address
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
        self.status = status;
    }

    fn set_interpreter(&mut self, interpreter: Interpreter) {
        self.interpreter = interpreter;
    }

    fn set_description(&mut self, description: String) {
        self.description = description;
    }
}
