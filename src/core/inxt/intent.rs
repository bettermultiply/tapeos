// in this file, we will implement the whole intent execution.

use crate::{
    base::intent::Intent,
    core::inxt::{
        exec::monitor::monitor,
        router::router::router,
        exec::schedule::schedule_intent,
        filter::judge::{intent_judge, reject_intent, JudgeResult},
        disassembler::dis::disassembler
    }
};

// this function is used to execute the intent.
// it connect the whole inxt process.
// consists of filter, disassembler, router, verifier, monitor.
pub async fn execute_intent<'a>(mut intent: Intent<'a>) {
    // filter the intent.
    match intent_judge(&intent).await {
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
    disassembler(&mut intent).await;

    router(&mut intent).await;

    schedule_intent(&intent);

    monitor(&mut intent).await;

    // complete should report completion to tape monitor.
    intent.complete();

}
