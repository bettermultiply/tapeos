// in this file, we will implement the whole intent execution.

use crate::{
    base::{intent::Intent, resource::Resource}, components::linkhub::waiter::TAPE, core::inxt::{
        disassembler::disassembler, monitor::monitor, preprocess::{process, JudgeResult}, router::router, schedule::schedule_intent
    }
};

use std::error::Error;

// this function is used to execute the intent.
// it connect the whole inxt process.
// consists of filter, disassembler, router, verifier, monitor.
pub async fn handler<'a>(mut intent: Intent<'a>) {
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
                    let _ = TAPE.lock().unwrap().first().unwrap().reject_intent(&intent.get_description());
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
pub fn execute<'a>(intent: &Intent<'a>) -> Result<(), Box<dyn Error>> {
    println!("execute: ");
    println!("execute: Start to execute intent");
    // TODO:
    Ok(())
}