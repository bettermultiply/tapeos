// in this file, we will implement the router for the intent execution.
// the router will distribute the intent to the corresponding resource or subsystem.

use std::error::Error;

use crate::{
    base::intent::{Intent, SubIntent}, 
    components::linkhub::seeker::{get_resource_description, get_resource_status_str, send_intent}, 
    tools::llmq::prompt
};

const RETRY_COUNT: i32 = 3;

// the distributer will distribute the sub-intents from disassembler to the corresponding resource or subsystem.
pub async fn router(intent: &mut Intent) {
    println!("router: Start to router intent");
    for sub_intent in intent.iter_sub_intent() {
        let retry_count = RETRY_COUNT;
        for _ in 0..retry_count {
            let address = select_resource(&sub_intent).await;
            sub_intent.set_selected_resource(address);
            let resource = sub_intent.get_selected_resource().unwrap();
            match send_intent(resource.to_string(), sub_intent.get_description(), sub_intent.get_id()).await {
                Ok(_) => {
                    // sleep(time::Duration::from_secs(1));
                    break;
                },
                Err(_) => continue
            };
        }
    }
}

async fn select_resource(sub_intent: &SubIntent) -> String {
    let mut best_resource: String = "".to_string();
    let mut best_score = -1;
    for resource in sub_intent.iter_available_resources() {
        let score = score(sub_intent.get_description(), &resource).await;
        if score > best_score {
            best_score = score;
            best_resource = resource.to_string();
        }
    }
    best_resource
}

const SCORE_METHOD: &str = "";

async fn score(sub_intent: &str, resource: &str) -> i32 {
    match SCORE_METHOD {
        "ai" => {
            score_by_ai(sub_intent, resource).await
        }
        _ => {
            0
        }
    }
}

async fn score_by_ai(sub_intent: &str, resource: &str) -> i32 {
    
        
        let score = prompt("score the resource for whether it is suitable to deal the sub-intent, return a score between 0 and 100.",
        format!("
        sub_intent: {sub_intent}
        resource: {},{}
        ",
        get_resource_description(resource),
        get_resource_status_str(resource)
        ).as_str()
    ).await;
    score.parse::<i32>().unwrap()
}

pub async fn reroute(sub_intent: &mut SubIntent)  -> Result<(), Box<dyn Error>> {
        let address = select_resource(&sub_intent).await;
        sub_intent.set_selected_resource(address);
        let resource = sub_intent.get_selected_resource().unwrap();
        match send_intent(resource.to_string(), sub_intent.get_description(), sub_intent.get_id()).await {
            Ok(_) => (),
            Err(_) => (),
        };
        Ok(())
}