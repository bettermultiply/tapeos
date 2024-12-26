// in this file, we will implement the filter for the intent.
// the filter will judge if the intent can be processed by the tapeos.

use crate::base::intent::Intent;

// judge if the intent can be processed by the tapeos.
pub fn intent_judge(intent: &Intent) -> bool {
    essential_judge(intent) && rule_judge(intent)
}

// should be used to judge  every intent.
fn essential_judge(intent: &Intent) -> bool {
    // TODO: implement the logic to judge if the intent is essential
    
    true
}
// this judge is conducted depends on intent's attributes.
fn rule_judge(intent: &Intent) -> bool {
    // TODO: implement the logic to judge if the intent is allowed by the rule
    true
}

pub fn reject_intent(intent: &Intent) {
    // TODO: implement the logic to reject the intent
}
