// in this file, we will implement the router for the intent execution.
// the router will distribute the intent to the corresponding resource or subsystem.

use crate::base::intent::{Intent, SubIntent};
use crate::base::resource::ResourceType;
use crate::base::resource::Resource;
use crate::tools::llmq::prompt;
use bluer::Address;
use crate::components::linkhub::bluetooth::seek::{receive_message, receive_response, send_intent};
// the distributer will distribute the sub-intents from disassembler to the corresponding resource or subsystem.
pub async fn router<'a>(intent: &mut Intent<'a>) {
    for sub_intent in intent.iter_sub_intent() {
        let retry_count = 3;
        for _ in 0..retry_count {
            let address = select_resource(&sub_intent).await;
            sub_intent.set_selected_resource(address);
            let resource = sub_intent.get_selected_resource().unwrap();
            match send_intent(&resource, sub_intent.get_description()).await {
                Ok(_) => (),
                Err(_) => continue
            };
            let (key, message) = receive_message(&resource).await.unwrap();
            match key.as_str() {
                "Intent" => {
                    let response = receive_response(message).await.unwrap();
                    if response.get("reject").is_none() {
                        break;
                    }
                }
                _ => ()
            }
        }
    }
}

// TODO: when shall we select the resource? when disassemble? when distribute?
async fn select_resource(sub_intent: &SubIntent) -> Address {
    // TODO: implement the logic to select the resource for the sub-intent
    let mut best_resource: Option<&ResourceType> = None;
    let mut best_score = -1;
    for resource in sub_intent.iter_available_resources() {
        let score = score(sub_intent.get_description(), resource);
        if score > best_score {
            best_score = score;
            best_resource = Some(resource);
        }
    }
    best_resource.unwrap().get_address().await
}

const SCORE_METHOD: &str = "";

fn score(sub_intent: &str, resource: &ResourceType) -> i32 {
    // TODO: implement the logic to score the sub-intent
    // TODO: use ai here to score the sub-intent as a test.
    match SCORE_METHOD {
        "ai" => {
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
    prompt(
        format!("
        score the resource for the sub-intent, return a score between 0 and 100 please.
        sub_intent: {sub_intent}
        resource: {},{}
        ",
        resource_ref.get_description(),
        resource_ref.get_status_str()
        ).as_str()
    );            
    sub_intent.len() as i32 + resource_ref.get_description().len() as i32
}

pub async fn reroute(sub_intent: &mut SubIntent) {
    let retry_count = 3;
    for _ in 0..retry_count {
        let address = select_resource(&sub_intent).await;
        sub_intent.set_selected_resource(address);
        let resource = sub_intent.get_selected_resource().unwrap();
        match send_intent(&resource, sub_intent.get_description()).await {
            Ok(_) => (),
            Err(_) => continue
        };
        let (key, message) = receive_message(&resource).await.unwrap();
        match key.as_str() {
            "Intent" => {
                let response = receive_response(message).await.unwrap();
                if response.get("reject").is_none() {
                    break;
                }
            }
            _ => ()
        }
    }
}