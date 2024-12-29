// in this file, we will implement the whole intent execution.

use crate::core::inxt::intent::monitor::monitor;
use crate::base::intent::Intent;
use super::exec::monitor;
use super::router::router::router;
use super::exec::schedule::schedule_intent;
use super::filter::judge::{intent_judge, reject_intent, JudgeResult};
use super::disassembler::dis::disassembler;

// this function is used to execute the intent.
// it connect the whole inxt process.
// consists of filter, disassembler, router, verifier, monitor.
pub async fn execute_intent<'a>(mut intent: Intent<'a>) {
    // filter the intent.
    match intent_judge(&intent) {
        JudgeResult::Rejected => {
            return;
        },
        JudgeResult::Reject => {
            reject_intent(&intent);
            return ;
        }
        JudgeResult::Accept => (),
    }

    // disassemble the intent.
    disassembler(&mut intent);

    router(&mut intent).await;

    schedule_intent(&intent);

    monitor(&mut intent).await;

    // complete should report completion to tape monitor.
    intent.complete();

}
