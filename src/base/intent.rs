// in this file, we will implement the intent structure and the intent related functions to manipulate the intent.

use crate::tools::idgen;

// raw intent format is "Intent:intent_description"
// the intent struct is not used for sending between outside and inside the system.
// it is used for internal manipulation.S
pub struct Intent {
    id: i64,
    description: String,
    complete: bool,
    source: IntentSource,
    resource: Option<String>,
    itype: IntentType,
    sub_intent: Vec<SubIntent>,
    reject_reason: Option<String>,
}

#[derive(PartialEq, Eq)]
pub enum IntentSource {
    Tape,
    Resource,
    Subsystem,
}

#[derive(PartialEq, Eq)]
pub enum IntentType {
    Intent,
    Response,
    Reject,
}

pub struct SubIntent {
    description: String,
    complete: bool,
    available_resources: Vec<String>,
    selected_resource: Option<String>,
}

impl Intent {
    pub fn new(description: String, source: IntentSource, itype: IntentType, resource: Option<String>) -> Self {
        Self { 
            id: idgen::generate_id(idgen::IdType::Intent),
            description, 
            complete: false, 
            source, 
            itype, 
            resource, 
            sub_intent: vec![] , 
            reject_reason: None}
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }

    pub fn iter_sub_intent(&mut self) -> impl Iterator<Item = &mut SubIntent> {
        self.sub_intent.iter_mut()
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_source(&self) -> &IntentSource {
        &self.source
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

    pub fn get_resource(&self) -> Option<&String> {
        self.resource.as_ref()
    }

    pub fn get_reject_reason(&self) -> Option<String> {
        self.reject_reason.clone()
    }

    pub fn get_intent_type(&self) -> &IntentType {
        &self.itype
    }
}

impl SubIntent {
    pub fn new(description: String, available_resources: Vec<String>) -> Self {
        Self { description, complete: false, available_resources, selected_resource: None }
    }

    pub fn iter_available_resources(&self) -> impl Iterator<Item = &String> {
        self.available_resources.iter()
    }

    pub fn remove_resource(&mut self, address: String) {
        self.available_resources.retain(|r| *r != address);
    }

    pub fn get_selected_resource(&self) -> Option<&String> {
        self.selected_resource.as_ref() 
    }

    pub fn set_selected_resource(&mut self, resource: String) {
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

    pub fn add(&mut self, resources: Vec<String>) {
        self.available_resources.extend(resources);
    }

    pub fn pop(&mut self) -> String {
        self.available_resources.pop().unwrap()
    }
    
    pub fn is_empty(&self) -> bool {
        self.available_resources.is_empty()
    }
}

