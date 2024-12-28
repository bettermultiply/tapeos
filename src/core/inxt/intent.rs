// in this file, we will implement the whole intent execution.

use crate::core::inxt::intent::monitor::monitor;
use crate::base::intent::Intent;
use super::exec::monitor;
use super::router::router::router;
use super::exec::schedule::schedule_intent;
use super::filter::judge::{reject_intent, intent_judge, reject_judge};
use super::disassembler::dis::disassembler;

// this function is used to execute the intent.
// it connect the whole inxt process.
// consists of filter, disassembler, router, verifier, monitor.
pub async fn execute_intent<'a>(mut intent: Intent<'a>) {
    if !reject_judge(&intent) {
        return;
    }

    // filter the intent.
    if !intent_judge(&intent) {
        // tell the intent source that the intent can not be executed.
        reject_intent(&intent);
        return ;
    }
        
    // disassemble the intent.
    disassembler(&mut intent);

    router(&mut intent).await;

    schedule_intent(&intent);

    monitor(&mut intent).await;

    // complete should report completion to tape monitor.
    intent.complete();

}
