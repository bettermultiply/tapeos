// in this file, we will implement the whole intent execution.

use log::info;
use rand::Rng;
use tokio::sync::Mutex;

use crate::{
    base::{errort::BoxResult, intent::Intent, resource::Status}, components::linkhub::seeker::{reject_intent, INTENT_QUEUE}, core::inxt::{
        disassembler::disassembler, monitor::monitor, preprocess::{process, JudgeResult}, router::router,
    }
};

use std::{sync::Arc, thread::sleep, time::Duration};

// this function is used to execute the intent.
// it connect the whole inxt process.
// consists of filter, disassembler, router, verifier, monitor.
pub async fn handler(mut intent: Intent) -> JudgeResult {
    // info!("handler: Start to execute intent");

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
            let _ = reject_intent(intent.get_resource().unwrap().to_string(), intent.get_description());
        }
    }
    // schedule_intent(&intent);

    router(&mut intent).await;
    // let id = intent.get_id();
    INTENT_QUEUE.lock().await.push(intent);
    

    // monitor(id).await;

    // complete should report completion to tape monitor.
    // intent.complete();
    JudgeResult::Accept

}

// execute is used to execute the intent route to itself.
pub async fn execute(intent: &str, status: Arc<Mutex<Status>>) -> BoxResult<u64> {
    random_execute(intent, status).await
}

macro_rules! execute_with_status {
    { $e:expr, $status:ident, $exec_time:ident } => { 
        $status.lock().await.add_busy_time($exec_time);
        $status.lock().await.change_dealing(true);
        $status.lock().await.change_average_time($exec_time);
        $status.lock().await.add_total_busy($exec_time);
        $e;  
        $status.lock().await.change_dealing(false);
        $status.lock().await.sub_busy_time($exec_time);
    }
}

pub async fn random_execute(intent: &str, status: Arc<Mutex<Status>>) -> BoxResult<u64> {
    let random_sleep_duration = rand::thread_rng().gen_range(1..=intent.len()) as u64; // Random duration between 1 and 5 seconds
    info!("execute {} in {} seconds", intent, random_sleep_duration);
    let exec_time = Duration::from_secs(random_sleep_duration);
    execute_with_status!(sleep(exec_time), status, exec_time);
    Ok(exec_time.as_secs())
}
