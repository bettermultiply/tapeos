// in this file, we will implement the router for the intent execution.
// the router will distribute the intent to the corresponding resource or subsystem.

use crate::base::intent::{Intent, SubIntent};
use crate::base::resource::ResourceType;
use crate::base::resource::Resource;
use bluer::Address;
use crate::components::linkhub::seek::bluetooth::{receive_response, send_intent};
// the distributer will distribute the sub-intents from disassembler to the corresponding resource or subsystem.
pub async fn router<'a>(intent: &mut Intent<'a>) {
    for sub_intent in intent.iter_sub_intent() {
        let address = select_resource(&sub_intent).await;
        sub_intent.set_selected_resource(address);
        let resource = sub_intent.get_selected_resource().unwrap();
        match send_intent(resource, sub_intent.get_description()).await {
            Ok(_) => (),
            Err(_) => continue
        };
        let response = receive_response(resource).await.unwrap();
        if response.get("reject").is_none() {
            break;
        }
    }
}

// TODO: when shall we select the resource? when disassemble? when distribute?
async fn select_resource(sub_intent: &SubIntent) -> Address {
    // TODO: implement the logic to select the resource for the sub-intent
    let mut best_resource: Option<&ResourceType> = None;
    let mut best_score = 0;
    for resource in sub_intent.iter_available_resources() {
        let score = score(sub_intent.get_description(), resource);
        if score > best_score {
            best_score = score;
            best_resource = Some(resource);
        }
    }
    best_resource.unwrap().get_address().await
}

const SCORE_METHOD: &str = "ai";

fn score(sub_intent: &str, resource: &ResourceType) -> i32 {
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

fn score_by_ai(sub_intent: &str, resource: &ResourceType) -> i32 {
    // TODO: implement the logic to score the sub-intent by ai
    let resource_ref: &dyn Resource = resource;
    sub_intent.len() as i32 + resource_ref.get_description().len() as i32
}

pub async fn reroute(sub_intent: &mut SubIntent) {
    // Logic to redistribute the rejected sub-intent to available resources
    // TODO: implement the logic to redistribute the rejected sub-intent to available resources
    let address = select_resource(&sub_intent).await;
    sub_intent.set_selected_resource(address);
}
