// in this file, we will implement the monitor for the intent execution.
// the monitor will monitor the execution of the intent and provide the feedback to the higher level system.

use crate::{base::intent::Intent, components::linkhub::seek::bluetooth::receive_response};
use crate::core::inxt::router::router::reroute;

pub async fn monitor<'a>(intent: &mut Intent<'a>) {
    // TODO: implement the logic to monitor the intent execution
    loop {
        let mut is_finished = true;
        for sub_intent in intent.iter_sub_intent() {
            if sub_intent.is_complete() {
                continue;
            }
            let resource = sub_intent.get_selected_resource().unwrap();
            let response = receive_response(resource).await.unwrap();
            if response.get("Unfinished").is_some() {
                reroute(sub_intent).await;
            } 
            if response.get("Finished").is_some() {
                is_finished = false;
            }
        }
        if is_finished {
            break;
        }
    }
    intent.complete();
}

