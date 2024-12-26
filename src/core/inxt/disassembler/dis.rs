// in this file, we will implement the disassembler.

use crate::base::{intent::Intent, resource::RESOURCES};

pub fn disassemble_intent<'a>(intent: &'a Intent) -> Option<Vec<Intent<'a>>> {
    let mut sub_intents: Vec<Intent<'a>> = Vec::new();

    // Logic to disassemble the intent based on resources
    for resource in RESOURCES.lock().unwrap().iter() {
        let sub_intent = format!("{} for {}", intent.get_description(), resource.get_name());
        sub_intents.push(Intent::new(sub_intent));
    }

    if sub_intents.is_empty() {
        return None;
    }

    Some(sub_intents)
}
