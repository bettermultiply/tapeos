// in this file, we will implement the filter for the intent.
// the filter will judge if the intent can be processed by the tapeos.

use crate::base::intent::Intent;
use crate::base::rule::{Rule, RuleDetail, STATIC_RULES, iter_rules};
use crate::base::intent::IntentSource;
use crate::components::linkhub::waiter::TAPE;
use crate::base::resource::Resource;
use chrono::{Local, Datelike};

// judge if the intent can be processed by the tapeos.
pub fn intent_judge(intent: &Intent) -> bool {
    println!("intent judge the intent: {}", intent.get_description());
    essential_judge(intent) && user_judge(intent)
}

// should be used to judge  every intent.
fn essential_judge(intent: &Intent) -> bool {
    println!("essential judge the intent: {}", intent.get_description());
    
    // in essential part, all rule's will be hard coded, here.
    // essential rules will never be expired.
    // other than changing the code, user can't not change the rules.
    risk_judge(intent) && privilege_judge(intent) // && ...
}

// judge if the intent is to reject.
pub fn reject_judge(intent: &Intent) -> bool {
    if judge(intent, &STATIC_RULES["reject"]) {
        // TODO: do more things here.
        println!("intent: {} is rejected", intent.get_description());
        return false;
    }
    true
}

// prevent the intent from risk users and the system.
fn risk_judge(intent: &Intent) -> bool {
    judge(intent, &STATIC_RULES["risk"])
}

// judge the privilege of the intent.
fn privilege_judge(intent: &Intent) -> bool {
    judge(intent, &STATIC_RULES["privilege"])
}

// this judge is conducted depends on intent's attributes.
fn user_judge(intent: &Intent) -> bool {
    println!("user judge the intent: {}", intent.get_description());
    // TODO: Maybe rule can be specified for intent type.
    for rule in iter_rules() {
        if !judge(intent, rule) {
            return false;
        }
    }
    true
}

// judge the intent by user defined rule.
fn judge(intent: &Intent, rule: &Rule) -> bool {
    match rule.get_rule_detail() {
        RuleDetail::Source(intent_source) => intent.get_intent_source() != intent_source,
        RuleDetail::Description(description) => intent.get_description() != description, // TODO: we should use ai to judge the description.
        RuleDetail::Time(_) => rule.is_expired(),
        RuleDetail::Weekday(weekday) => {
            let now = Local::now().date_naive().weekday();
            now != *weekday
        },
        RuleDetail::Essential => true, // TODO: we should use ai to judge the essential.
    }
}

// sending intnet back to source.
pub fn reject_intent(intent: &Intent) {
    println!("reject the intent: {}", intent.get_description());
    match intent.get_intent_source() {
        IntentSource::Tape => {
            TAPE.lock().unwrap()
                .first().unwrap()
                .reject_intent(intent);
        },
        _ => {
            let source = intent.get_source();
            source.reject_intent(intent);
        },
    }
    // TODO: Maybe we should tell the source why the intent is rejected.

}