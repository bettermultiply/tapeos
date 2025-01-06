// in this file, we will implement the whole intent execution.

use crate::{
    base::intent::Intent, components::linkhub::seeker::reject_intent, core::inxt::{
        disassembler::disassembler, monitor::monitor, preprocess::{process, JudgeResult}, router::router, schedule::schedule_intent
    }
};

use std::error::Error;

// this function is used to execute the intent.
// it connect the whole inxt process.
// consists of filter, disassembler, router, verifier, monitor.
pub async fn handler(mut intent: Intent) {
    println!("handler: ");
    println!("handler: Start to execute intent");

    // preprocess the intent, including filter and special execution.
    match process(&intent).await {
        JudgeResult::SpecialExecution => {
            return;
        },
        JudgeResult::Reject => {
            return ;
        },
        JudgeResult::Accept => (),
    }

    // disassemble the intent.
    match disassembler(&mut intent).await {
        Some(_) => {
        },  
        None => {
            match execute(&intent) {
                Ok(_) => (),
                Err(err) => {
                    println!("execute failed: {}", err);
                    let _ = reject_intent("TAPE".to_string(), intent.get_description().to_string());
                }
            }
            return;
        }
    }

    router(&mut intent).await;

    schedule_intent(&intent);

    monitor(&mut intent).await;

    // complete should report completion to tape monitor.
    intent.complete();

}

// execute is used to execute the intent route to itself.
pub fn execute(intent: &Intent) -> Result<(), Box<dyn Error>> {
    println!("execute: ");
    println!("execute: Start to execute intent");
    // TODO:
    Ok(())
}