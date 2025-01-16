// in this file, we will implement the router for the intent execution.
// the router will distribute the intent to the corresponding resource or subsystem.

use log::{info, warn};

use crate::{
    base::{errort::BoxResult, intent::{Intent, SubIntent}}, 
    components::linkhub::seeker::{
        get_resource_description, 
        get_resource_status_str, 
        send_intent
    }, 
    tools::llmq::prompt
};

const RETRY_COUNT: i32 = 3;
const SCORE_METHOD: &str = "";

async fn route_intent(resource_name: &str, intent: &str, id: i64) -> BoxResult<()> {   
    send_intent(resource_name.to_string(), intent, id).await
}

pub async fn reroute(s_intent: &mut SubIntent)  -> BoxResult<()> {
    let name = select_resource(& s_intent).await;
    let s = name.to_string().clone();
    s_intent.set_selected_resource(s.clone());
    s_intent.remove_resource(s);
    let resource = s_intent.get_selected_resource().unwrap();
    route_intent(resource, s_intent.get_description(), s_intent.get_id()).await
}

pub async fn route_all(s_intent: &mut SubIntent)  -> BoxResult<()> {

    for r in s_intent.iter_available_resources() {
    route_intent(r, s_intent.get_description(), s_intent.get_id()).await?;
    }
    s_intent.remove_resource_all();
    Ok(())
}

// the distributer will distribute the sub-intents from disassembler to the corresponding resource or subsystem.
pub async fn router(i: &mut Intent) {
    info!("router: Start to router intent");
    if i.get_emergency() {
        for s_intent in i.iter_sub_intent() {
            route_all(s_intent).await.unwrap();
        }
    } else {
        for s_intent in i.iter_sub_intent() {
            for _ in 0..RETRY_COUNT {
                match reroute(s_intent).await {
                    Ok(_) => break,
                    Err(_) => continue
                };
            }
        }
    }
}

async fn select_resource(s_intent: &SubIntent) -> &str {
    let mut best_resource: &str = "";
    let mut best_score: i32 = -1;
    for resource in s_intent.iter_available_resources() {
        let r = format!("{}", resource);
        let score: i32 = score(s_intent.get_description(), &r).await;
        if score > best_score {
            best_score = score;
            best_resource = resource;
        }
    }
    best_resource
}

async fn score(sub_intent: &str, resource: &str) -> i32 {
    match SCORE_METHOD {
        "ai" => {
            score_by_ai(sub_intent, resource).await
        }
        _ => {
            // which means just use resource in turn
            warn!("no such score method, use in turn");
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
        get_resource_description(resource).await,
        get_resource_status_str(resource).await
        ).as_str()
    ).await;
    score.parse::<i32>().unwrap()
}
