// in this file, we will implement the verifier for the intent execution.
// the verifier will verify the specific execution of intent and their dependency
// Try to find the best way to execute it. If there is any conflict, it will info the distributer to redistribute the intent.

use crate::base::intent::Intent;

pub fn schedule_intent(intent: &Intent) -> bool {
    schedule_dependency(intent) && verify_resource(intent) && verify_subintent(intent)
}


fn schedule_dependency(intent: &Intent) -> bool {
    // TODO: implement the logic to verify the dependency of the intent
    !intent.is_complete()
}


fn verify_resource(intent: &Intent) -> bool {
    // TODO: implement the logic to verify the resource of the intent
    !intent.is_complete()
}


fn verify_subintent(intent: &Intent) -> bool {
    // TODO: implement the logic to verify the subintent of the intent
    !intent.is_complete()
}




