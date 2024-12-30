// in this file, we will implement the filter for the intent.
// the filter will judge if the intent can be processed by the tapeos.

use chrono::{Local, Datelike};

use crate::{
    components::linkhub::waiter::TAPE,
    tools::llmq::prompt,
    base::resource::Resource
};
use crate::base::{
    intent::{Intent, IntentSource},
    rule::{Rule, RuleDetail, STATIC_RULES, iter_rules}
};

pub enum JudgeResult {
    Rejected,
    Accept,
    Reject,
}

// judge if the intent can be processed by the tapeos.
pub async fn intent_judge<'a>(intent: &Intent<'a>) -> JudgeResult {
    println!("intent judge the intent: {}", intent.get_description());
    
    if reject_judge(intent).await {
        return JudgeResult::Rejected;
    }

    if essential_judge(intent).await && user_judge(intent).await {
        return JudgeResult::Accept;
    }

    JudgeResult::Reject
}

// judge if the intent is to reject.
// if the intent is rejected, return true.
pub async fn reject_judge<'a>(intent: &Intent<'a>) -> bool {
    if judge(intent, &STATIC_RULES["reject"]).await {
        println!("intent: {} is rejected", intent.get_description());
        return true;
    }
    false
}

// should be used to judge  every intent.
async fn essential_judge<'a>(intent: &Intent<'a>) -> bool {
    println!("essential judge the intent: {}", intent.get_description());
    
    // in essential part, all rule's will be hard coded, here.
    // essential rules will never be expired.
    // other than changing the code, user can't not change the rules.
    risk_judge(intent).await && privilege_judge(intent).await // && ...
}

// prevent the intent from risk users and the system.
async fn risk_judge<'a>(intent: &Intent<'a>) -> bool {
    judge(intent, &STATIC_RULES["risk"]).await
}

// judge the privilege of the intent.
async fn privilege_judge<'a>(intent: &Intent<'a>) -> bool {
    judge(intent, &STATIC_RULES["privilege"]).await
}

// this judge is conducted depends on intent's attributes.
async fn user_judge<'a>(intent: &Intent<'a>) -> bool {
    // TODO: Maybe rule can be specified for intent type.
    for rule in iter_rules() {
        if !judge(intent, rule).await {
            return false;
        }    println!("user judge the intent: {}", intent.get_description());

    }
    true
}

// judge the intent by user defined rule.
async fn judge<'a>(intent: &Intent<'a>, rule: &Rule) -> bool {
    match rule.get_rule_detail() {
        RuleDetail::Source(intent_source) => intent.get_intent_source() != intent_source,
        RuleDetail::Time(_) => rule.is_expired(),
        RuleDetail::Weekday(weekday) => {
            let now = Local::now().date_naive().weekday();
            now != *weekday
        },
        RuleDetail::UserDefined(rule_description) => {
            let prompt_content = format!("the rule description is: {}\n the intent description is: {}. If the intent conform to the rule, return true, otherwise return false.", rule_description, intent.get_description());
            match prompt(&prompt_content).await.as_str() {
                "true" => true,
                "false" => false,
                _ => false,
            }
        },
        RuleDetail::Essential(rule_func) => {
            rule_func(intent)
        }, 
    }
}

// sending intnet back to source.
pub fn reject_intent(intent: &Intent) {
    println!("reject the intent: {}", intent.get_description());
    match intent.get_intent_source() {
        IntentSource::Tape => {
            (TAPE.lock().unwrap()
                .first().unwrap() as &dyn Resource)
                .reject_intent(intent.get_description());
        },
        _ => {
            let source = intent.get_source();
            source.reject_intent(intent.get_description());
        },
    }
    // TODO: Maybe we should tell the source why the intent is rejected.
}