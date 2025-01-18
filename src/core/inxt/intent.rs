// in this file, we will implement the whole intent execution.

use log::info;
use rand::Rng;

use crate::{
    base::{errort::BoxResult, intent::Intent}, components::linkhub::seeker::{reject_intent, INTENT_QUEUE}, core::inxt::{
        disassembler::disassembler, monitor::monitor, preprocess::{process, JudgeResult}, router::router, schedule::schedule_intent
    }
};

use std::{thread::sleep, time};

// this function is used to execute the intent.
// it connect the whole inxt process.
// consists of filter, disassembler, router, verifier, monitor.
pub async fn handler(mut intent: Intent) -> JudgeResult {
    info!("handler: Start to execute intent");

    // preprocess the intent, including filter and special execution.
    match process(&mut intent).await {
        JudgeResult::Execution => {
            return JudgeResult::Execution;
        },
        JudgeResult::Reject(e) => {
            return JudgeResult::Reject(e);
        },
        JudgeResult::Accept => (),
    }
    
    // disassemble the intent.
    match disassembler(&mut intent).await {
        Some(_) => {
        },  
        None => {
            match execute(intent.get_description()) {
                // TODO special execution here.
                Ok(_) => (),
                Err(err) => {
                    println!("execute failed: {}", err);
                    let _ = reject_intent("TAPE".to_string(), intent.get_description());
                }
            }
            return JudgeResult::Accept;
        }
    }
    schedule_intent(&intent);

    router(&mut intent).await;
    let id = intent.get_id();
    INTENT_QUEUE.lock().await.push(intent);
    

    monitor(id).await;

    // complete should report completion to tape monitor.
    // intent.complete();
    JudgeResult::Accept

}

// execute is used to execute the intent route to itself.
pub fn execute(intent: &str) -> BoxResult<()> {
    random_execute(intent)
}

pub fn random_execute(intent: &str) -> BoxResult<()> {
    let random_sleep_duration = rand::thread_rng().gen_range(1..=intent.len()); // Random duration between 1 and 5 seconds
    info!("execute {} in {} seconds", intent, random_sleep_duration);
    // tokio::time::sleep(tokio::time::Duration::from_secs(random_sleep_duration as u64)).await;
    sleep(time::Duration::from_secs(random_sleep_duration as u64));  
    Ok(())
}