// in this file, we will implement the whole intent execution.

use crate::core::inxt::intent::monitor::monitor;
use crate::base::intent::Intent;
use super::exec::monitor;
use super::router::router::distribute;
use super::exec::verifier::verify_intent;
use super::filter::judge::{reject_intent, intent_judge};
use super::disassembler::dis::disassemble_intent;


// this function is used to execute the intent.
// it connect the whole inxt process.
// consists of filter, disassembler, router, verifier, monitor.
pub fn execute_intent(intent: Intent) {

    // filter the intent.
    if !intent_judge(&intent) {
        // tell the intent source that the intent can not be executed.
        reject_intent(&intent);
        return ;
    }
        
    // disassemble the intent.
    let sub_intents = match disassemble_intent(&intent) {
        Some(sub_intents) => sub_intents,
        None => {
            // tell the intent source that the intent can not be executed.
            reject_intent(&intent);
            return ;
        }
    };
    
    distribute(&sub_intents);

    // TODO: implement the logic to execute the intent
    
    verify_intent(&intent);

    monitor(&intent);

}



