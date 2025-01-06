// in this file, we will implement the filter for the intent.
// the filter will judge if the intent can be processed by the tapeos.
// here we'll simply show the cal map:
// process -> filter -> essential_judge 
//                   -> user_judge      
//         ->special_execution            => rule_judge
//         ->reject
use chrono::{Local, Datelike};

use crate::{
    base::{
        intent::{Intent, IntentSource}, rule::{Rule, RuleDetail, RuleSet, STATIC_RULES}
    }, components::linkhub::seeker::reject_intent, tools::{llmq::{self, prompt}, record::record}
};

pub enum JudgeResult {
    Reject,
    Accept,
    SpecialExecution,
}

// preprocess the intent.
pub async fn process(intent: &Intent) -> JudgeResult {
    println!("process: ");
    println!("process: Judge the intent: {}", intent.get_description());
    
    if filter(intent).await {
        return JudgeResult::Reject;
    }
    
    println!("process: Filter passed");
    
    if special_execution(intent).await {
        return JudgeResult::SpecialExecution;
    }

    println!("process: Special execution passed");

    JudgeResult::Accept
}

// filter the unacceptible intent.
async fn filter(intent: &Intent) -> bool {
    if essential_judge(intent).await || user_judge(intent).await {
        println!("filter: Essential and user judge error");
        reject(intent);
        return true;
    }

    println!("filter: Essential and user judge passed");
    false
}

// special execution for the intent.
async fn special_execution(intent: &Intent) -> bool {
    println!("special execution: ");
    let special_id = STATIC_RULES["reject"].get_id();
    for rule in STATIC_RULES.values() {
        if rule.get_id() < special_id {
            continue;
        }

        if rule_judge(intent, rule).await {
            println!("special execution: Judge based on the rule: {}", rule.get_name());
            return true;
        }
    }

    false
}

// should be used to judge  every intent.
async fn essential_judge(intent: &Intent) -> bool {
    println!("essential judge: ");
    
    // in essential part, all rule's will be hard coded.
    // essential rules will never be expired.
    // other than changing the code, user can't not change the rules.
    let special_id = STATIC_RULES["reject"].get_id();
    for rule in STATIC_RULES.values() {
        if rule.get_id() >= special_id {
            break;
        }

        if rule_judge(intent, rule).await {
            println!("essential judge: Judge based on the rule: {}", rule.get_name());
            return true;
        }
    }

    println!("essential judge: Judge passed");
    false
}

// this judge is conducted depends on intent's attributes.
async fn user_judge(intent: &Intent) -> bool {
    // TODO: Maybe rule can be specified for intent type.
    println!("user judge: ");
    for rule in RuleSet::iter_rules() {
        if rule_judge(intent, rule).await {
            println!("user judge: Judge based on the rule: {}", rule.get_name());
            return true;
        }
    }

    println!("user judge: Judge passed");
    false
}

// judge the intent by user defined rule.
// if {
//  judge result is true then prevent intent to be executed.
//} else {
//  judge result is false then allow intent to be executed.
//}
async fn rule_judge(intent: &Intent, rule: &Rule) -> bool {
    match rule.get_rule_detail() {
        RuleDetail::Source(intent_source) 
            => intent.get_source() == intent_source,
        RuleDetail::Time(_) 
            => rule.is_expired(),
        RuleDetail::Weekday(weekday) => {
            let now = Local::now().date_naive().weekday();
            now == *weekday
        },
        RuleDetail::UserDefined(rule_description) => {
            let prompt_content = format!("the rule description is: {}\n the intent description is: {}. If the intent conform to the rule, return true, otherwise return false.", rule_description, intent.get_description());
            match prompt(&prompt_content).await.as_str() {
                "true" => true,
                "false" => false,
                _ => false,
            }
        },
        RuleDetail::Function(rule_func) => {
            rule_func(intent)
        }, 
        RuleDetail::Prompt(prompt_content) => {
            let prompt = format!("{} {}", prompt_content, intent.get_description());
            match llmq::prompt(&prompt).await.as_str() {
                "true" => true,
                "false" => false,
                _ => false,
            }
        },
    }
}

// sending intnet back to source.
pub fn reject(intent: &Intent) {
    println!("reject: ");
    println!("reject: Reject the intent: {}", intent.get_description());
    let response = 
        format!("intent: {}\nreject reason: {}", intent.get_description(), intent.get_reject_reason().unwrap());
    
    match intent.get_source() {
        IntentSource::Tape => {
            println!("reject: Reject Tape intent");
            
            let _ = reject_intent("TAPE".to_string(), response.clone());
        },
        _ => {
            println!("reject: Reject resource intent");
            let source = intent.get_resource();
            if source.is_some() {
                let _ = reject_intent(source.unwrap().to_string(), intent.get_description().to_string());
            } else {
                println!("No resource found, error intent");
            }
        },
    }
 
    record(response);
}