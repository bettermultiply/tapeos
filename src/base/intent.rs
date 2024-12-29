// in this file, we will implement the intent structure and the intent related functions to manipulate the intent.

use std::sync::Arc;

use crate::base::resource::ResourceType;
use crate::base::resource::Resource;
use bluer::Address;
use crate::base::resource::find_resource;
// the intent struct is not used for sending between outside and inside the system.
// it is used for internal manipulation.S
pub struct Intent<'a> {
    description: String,
    complete: bool,
    intent_source: IntentSource,
    source: Option<&'a dyn Resource>,
    sub_intent: Vec<SubIntent>,
}

#[derive(PartialEq, Eq)]
pub enum IntentSource {
    Resource,
    Subsystem,
    Tape
}

pub struct SubIntent {
    description: String,
    complete: bool,
    available_resources: Vec<Arc<ResourceType>>,
    selected_resource: Option<Arc<ResourceType>>,
}

impl<'a> Intent<'a> {
    pub fn new(description: String, intent_source: IntentSource, source: Option<&'a dyn Resource>) -> Self {
        Self { description, intent_source, complete: false, source: source, sub_intent: vec![] }
    }

    pub fn iter_sub_intent(&mut self) -> impl Iterator<Item = &mut SubIntent> {
        self.sub_intent.iter_mut()
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_intent_source(&self) -> &IntentSource {
        &self.intent_source
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }

    pub fn complete(&mut self) {
        self.complete = true;
    }

    pub fn add_sub_intent(&mut self, sub_intent: Vec<SubIntent>) {
        self.sub_intent.extend(sub_intent);
    }

    pub fn get_source(&self) -> &'a dyn Resource {
        self.source.unwrap()
    }
}

impl SubIntent {
    pub fn new(description: String, available_resources: Vec<String>) -> Self {
        let available_resources: Vec<Arc<ResourceType>> = 
            available_resources
            .iter()
            .map(|r| {
                find_resource(r.to_string()).unwrap()
            })
            .collect();
        Self { description, complete: false, available_resources, selected_resource: None }
    }

    pub fn iter_available_resources(&self) -> impl Iterator<Item = &Arc<ResourceType>> {
        self.available_resources.iter()
    }

    pub fn remove_resource(&mut self, address: Address) {
        self.available_resources.retain(|r| r.compare_address(address));
    }

    pub fn get_selected_resource(&self) -> Option<Arc<ResourceType>> {
        self.selected_resource.clone()
    }

    pub fn set_selected_resource(&mut self, address: Address) {
        for (index, r) in self.available_resources.iter().enumerate() {
            if r.compare_address(address) {
                self.selected_resource = Some(self.available_resources.remove(index));
                break;
            }
        }
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

    pub fn add(&mut self, resources: Vec<Arc<ResourceType>>) {
        self.available_resources.extend(resources);
    }

    pub fn pop(&mut self) -> Arc<ResourceType> {
        self.available_resources.pop().unwrap()
    }
    
    pub fn is_empty(&self) -> bool {
        self.available_resources.is_empty()
    }
}
