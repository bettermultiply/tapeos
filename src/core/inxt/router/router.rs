// in this file, we will implement the router for the intent execution.
// the router will distribute the intent to the corresponding resource or subsystem.

use bluer::Address;
use crate::{
    base::{
        intent::{Intent, SubIntent},
        resource::{ResourceType, Resource}
    },
    tools::llmq::prompt,
    components::linkhub::bluetooth::seek::{receive_message, receive_response, send_intent}
};

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

async fn select_resource(sub_intent: &SubIntent) -> Address {
    let mut best_resource: Option<&ResourceType> = None;
    let mut best_score = -1;
    for resource in sub_intent.iter_available_resources() {
        let score = score(sub_intent.get_description(), resource).await;
        if score > best_score {
            best_score = score;
            best_resource = Some(resource);
        }
    }
    best_resource.unwrap().get_address().await
}

const SCORE_METHOD: &str = "";

async fn score(sub_intent: &str, resource: &ResourceType) -> i32 {
    match SCORE_METHOD {
        "ai" => {
            score_by_ai(sub_intent, resource).await
        }
        _ => {
            0
        }
    }
}

async fn score_by_ai(sub_intent: &str, resource: &ResourceType) -> i32 {
    let resource_ref: &dyn Resource = resource;
    let score = prompt(
        format!("
        score the resource for the sub-intent, return a score between 0 and 100 please.
        sub_intent: {sub_intent}
        resource: {},{}
        ",
        resource_ref.get_description(),
        resource_ref.get_status_str()
        ).as_str()
    ).await;
    score.parse::<i32>().unwrap()
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