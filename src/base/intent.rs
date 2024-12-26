// in this file, we will implement the intent structure and the intent related functions to manipulate the intent.

use crate::base::resource::Resource;

// the intent struct is not used for sending between outside and inside the system.
// it is used for internal manipulation.S
pub struct Intent<'a> {
    description: String,
    available_resources: Vec<&'a dyn Resource>,
    // TODO: define the intent structure
}

impl<'a> Intent<'a> {
    pub fn new(description: String) -> Self {
        Self { description, available_resources: vec![] }
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_available_resources(&self) -> &Vec<&dyn Resource> {
        &self.available_resources
    }

    pub fn set_available_resources(&mut self, available_resources: Vec<&'a dyn Resource>) {
        self.available_resources = available_resources;
    }

    pub fn add_available_resources(&mut self, resources: Vec<&'a dyn Resource>) {
        self.available_resources.extend(resources.iter());
    }

    pub fn remove_available_resource(&mut self, resource: &'a dyn Resource) {
        self.available_resources.retain(|r| r.get_id() != resource.get_id());
    }

    pub fn remove_available_resources(&mut self, resources: Vec<&'a dyn Resource>) {
        self.available_resources.retain(|r| !resources.iter().any(|r2| r2.get_id() == r.get_id()));
    }

    pub fn clear_available_resources(&mut self) {
        self.available_resources.clear();
    }
}

fn intent_extract(intent: &Intent) -> String {
    // TODO: implement the logic to extract the intent
    intent.description.clone()
}

// TODO: do we really need this?
fn intent_serialize(intent: &Intent) -> String {
    // TODO: implement the logic to serialize the intent
    intent.description.clone()
}

fn intent_deserialize(intent: &Intent) -> String {
    // TODO: implement the logic to deserialize the intent
    intent.description.clone()
}



