// in this file, we will implement the intent structure and the intent related functions to manipulate the intent.

use crate::base::resource::Resource;

// the intent struct is not used for sending between outside and inside the system.
// it is used for internal manipulation.S
pub struct Intent<'a> {
    description: String,
    complete: bool,
    source: IntentSource,
    sub_intent: Vec<SubIntent<'a>>,
}

#[derive(PartialEq)]
pub enum IntentSource {
    Resource,
    System,
    SubSystem,
}

pub struct SubIntent<'a> {
    description: String,
    complete: bool,
    available_resources: Vec<&'a dyn Resource>,
    selected_resource: Option<&'a dyn Resource>,
}

impl<'a> Intent<'a> {
    pub fn new(description: String, source: IntentSource) -> Self {
        Self { description, complete: false, source, sub_intent: vec![] }
    }

    pub fn iter_sub_intent(&mut self) -> impl Iterator<Item = &mut SubIntent<'a>> {
        self.sub_intent.iter_mut()
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }

    pub fn complete(&mut self) {
        self.complete = true;
    }

    pub fn add_sub_intent(&mut self, sub_intent: Vec<SubIntent<'a>>) {
        self.sub_intent.extend(sub_intent);
    }

    pub fn get_source(&self) -> &IntentSource {
        &self.source
    }
}

impl <'a>SubIntent<'a> {
    pub fn new(description: String, available_resources: Vec<&'a dyn Resource>) -> Self {
        Self { description, complete: false, available_resources, selected_resource: None }
    }

    pub fn iter_available_resources(&self) -> impl Iterator<Item = &&'a dyn Resource> {
        self.available_resources.iter()
    }

    pub fn get_selected_resource(&self) -> Option<&'a dyn Resource> {
        self.selected_resource
    }

    pub fn set_selected_resource(&mut self, resource: &'a dyn Resource) {
        self.selected_resource = Some(resource);
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }

    pub fn complete(&mut self) {
        self.complete = true;
    }

    pub fn add(&mut self, resources: Vec<&'a dyn Resource>) {
        self.available_resources.extend(resources.iter());
    }

    pub fn pop(&mut self) -> &'a dyn Resource {
        self.available_resources.pop().unwrap()
    }
    
    pub fn is_empty(&self) -> bool {
        self.available_resources.is_empty()
    }
}
