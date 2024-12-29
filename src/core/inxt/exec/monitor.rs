// in this file, we will implement the monitor for the intent execution.
// the monitor will monitor the execution of the intent and provide the feedback to the higher level system.

use crate::{
    base::intent::Intent,
    components::linkhub::bluetooth::seek::{receive_message, receive_response},
    core::inxt::router::router::reroute
};

pub async fn monitor<'a>(intent: &mut Intent<'a>) {
    loop {
        let mut is_finished = true;
        for sub_intent in intent.iter_sub_intent() {
            if sub_intent.is_complete() {
                continue;
            }
            let resource = sub_intent.get_selected_resource().unwrap();
            let (key, message) = receive_message(&resource).await.unwrap();
            match key.as_str() {
                "Intent" => {
                    let response = receive_response(message).await.unwrap();
                    if response.get("Unfinished").is_some() {
                        reroute(sub_intent).await;
                    } 
                    if response.get("Finished").is_some() {
                        is_finished = false;
                    }
                }
                _ => ()
            }
            
        }
        if is_finished {
            break;
        }
    }
    intent.complete();
}

