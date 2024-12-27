// in this file, we will implement the filter for the intent.
// the filter will judge if the intent can be processed by the tapeos.

use crate::base::intent::Intent;
use crate::base::rule::Rule;
use crate::base::rule::RuleDetail;
use chrono::Datelike;
use chrono::Local;
use crate::base::rule::iter_rules;

// judge if the intent can be processed by the tapeos.
pub fn intent_judge(intent: &Intent) -> bool {
    essential_judge(intent) && user_judge(intent)
}

// this judge is conducted depends on intent's attributes.
fn user_judge(intent: &Intent) -> bool {
    println!("user judge the intent: {}", intent.get_description());
    for rule in iter_rules() {
        if !judge(intent, rule) {
            return false;
        }
    }
    true
}

// sending intnet back to source.
pub fn reject_intent(intent: &Intent) {
    println!("reject the intent: {}", intent.get_description());
    // TODO: implement the logic to reject the intent

    
}

// should be used to judge  every intent.
fn essential_judge(intent: &Intent) -> bool {
    println!("essential judge the intent: {}", intent.get_description());
    // TODO: implement the logic to judge if the intent is essential
    
    // in essential part, all rule's will be hard coded, here.
    // essential rules will never be expired.
    // other than changing the code, user can't not change the rules.
    risk_judge(intent) && privilege_judge(intent)
}

fn risk_judge(intent: &Intent) -> bool {
    // TODO: How can we insert ai here?
    intent.is_complete()
}

fn privilege_judge(intent: &Intent) -> bool {
    // TODO: implement the logic to judge if the intent is allowed by the privilege
    intent.is_complete()
}

fn judge(intent: &Intent, rule: &Rule) -> bool {
    // TODO: implement the logic to judge if the intent is allowed by the rule
    match rule.get_rule_detail() {
        RuleDetail::Source(source) => intent.get_source() != source,
        RuleDetail::Description(description) => intent.get_description() != description,
        RuleDetail::Time(_) => rule.is_expired(),
        RuleDetail::Weekday(weekday) => {
            let now = Local::now().date_naive().weekday();
            now != *weekday
        },
    }
}
