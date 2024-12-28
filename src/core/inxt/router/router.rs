// in this file, we will implement the router for the intent execution.
// the router will distribute the intent to the corresponding resource or subsystem.

use crate::base::intent::{Intent, SubIntent};
use crate::base::resource::Resource;

// the distributer will distribute the sub-intents from disassembler to the corresponding resource or subsystem.
pub fn router<'a>(intent: &mut Intent<'a>) {
    for sub_intent in intent.iter_sub_intent() {
        let resource = select_resource(&sub_intent);
        sub_intent.set_selected_resource(resource);
        // TODO: implement the logic to distribute the sub-intent to the corresponding resource or subsystem
    }
}

// TODO: when shall we select the resource? when disassemble? when distribute?
fn select_resource<'a>(sub_intent: &SubIntent<'a>) -> &'a dyn Resource {
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

const SCORE_METHOD: &str = "ai";

fn score(sub_intent: &str, resource: &dyn Resource) -> i32 {
    // TODO: implement the logic to score the sub-intent
    // TODO: use ai here to score the sub-intent as a test.
    match SCORE_METHOD {
        "ai" => {
            // TODO: use ai here to score the sub-intent
            
            score_by_ai(sub_intent, resource)
        }
        _ => {
            0
        }
    }
}

fn score_by_ai(sub_intent: &str, resource: &dyn Resource) -> i32 {
    // TODO: implement the logic to score the sub-intent by ai
    sub_intent.len() as i32 + resource.get_address().len() as i32
}

fn reroute(sub_intent: &mut SubIntent) {
    // Logic to redistribute the rejected sub-intent to available resources
    // TODO: implement the logic to redistribute the rejected sub-intent to available resources
    let resource = select_resource(&sub_intent);
    sub_intent.set_selected_resource(resource);

}
