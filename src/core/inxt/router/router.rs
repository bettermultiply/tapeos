// in this file, we will implement the router for the intent execution.
// the router will distribute the intent to the corresponding resource or subsystem.

use crate::base::intent::{Intent, SubIntent};
use crate::base::resource::Resource;

// the distributer will distribute the sub-intents from disassembler to the corresponding resource or subsystem.
pub fn router<'a>(intent: &mut Intent<'a>) {
    for mut sub_intent in intent.iter_sub_intent() {
        let resource = select_resource(&mut sub_intent);
        sub_intent.set_selected_resource(resource);
        // TODO: implement the logic to distribute the sub-intent to the corresponding resource or subsystem
    }
}

// TODO: when shall we select the resource? when disassemble? when distribute?
fn select_resource<'a>(sub_intent: & mut SubIntent<'a>) -> &'a dyn Resource {
    // TODO: implement the logic to select the resource for the sub-intent
    let mut best_resource: Option<&dyn Resource> = None;
    let mut best_score = 0;
    for resource in sub_intent.iter_available_resources() {
        let score = score(sub_intent.get_description(), *resource);
        if score > best_score {
            best_score = score;
            best_resource = Some(*resource);
        }
    }
    best_resource.unwrap()
}

fn score(sub_intent: &str, resource: &dyn Resource) -> u8 {
    // TODO: implement the logic to score the sub-intent
    0
}


// Function to handle the event of resource rejection
fn redistribute_rejected_subintent(sub_intent: Intent, resources: &[&str]) {
    // Logic to redistribute the rejected sub-intent to available resources
    // TODO: implement the logic to redistribute the rejected sub-intent to available resources
}
