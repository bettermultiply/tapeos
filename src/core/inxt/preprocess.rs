// in this file, we will implement the filter for the intent.
// the filter will judge if the intent can be processed by the tapeos.
// here we'll simply show the cal map:
// process -> filter -> essential_judge 
//                   -> user_judge      
//         ->special_execution            => rule_judge
//         ->reject
use chrono::{Local, Datelike};
use log::info;
use std::process::Command;

use crate::{
    base::{
        errort::{BoxResult, JudgeError},
        intent::Intent, 
        rule::{Rule, RuleDetail, RULES, STATIC_RULES}
    }, tools::llmq::prompt,
};

pub enum JudgeResult {
    Reject(String),
    Accept,
    Execution,
}

// preprocess the intent.
pub async fn process(intent: &Intent) -> JudgeResult {
    info!("process: Judge the intent: {}", intent.get_description());
    
    match filter(intent).await {
        Ok(_) => (),
        Err(e) => return JudgeResult::Reject(format_reject(intent.get_description(), &format!("{}", e))),
    }
    
    info!("process: Filter passed");
    
    match spec_exec(intent).await {
        Ok(false) => (),
        Ok(true) => return JudgeResult::Execution,
        Err(e) => return JudgeResult::Reject(format_reject(intent.get_description(), &format!("{}", e))),

    }

    info!("process: Special execution passed");

    JudgeResult::Accept
}

// filter the unacceptible intent.
async fn filter(intent: &Intent) -> BoxResult<()> {
    match essential_judge(intent).await {
        Ok(_) => (),
        Err(e) => {
            return Err(e);
        }
    }
    match user_judge(intent).await {
        Ok(_) => (),
        Err(e) => {
            return Err(e);
        }
    }

    Ok(())
}

// special execution for the intent.
async fn spec_exec(intent: &Intent) -> BoxResult<bool> {
    
    let i = intent.get_description();
    let i_pair = i.split(":").collect::<Vec<&str>>();
    if i_pair.len() != 2 {
        return Ok(false);
    }

    let special_id = STATIC_RULES["reject"].get_id();
    for rule in STATIC_RULES.values().into_iter() {
        if rule.get_id() < special_id || i_pair[1] != rule.get_name() {
            continue;
        }

        match rule_judge(intent, rule).await {
            Ok(()) => {
                info!("special execution: ");
                return Ok(true);   
            },
            Err(e) => return Err(e),
        }
    }

    Ok(false)
}

// should be used to judge  every intent.
async fn essential_judge(intent: &Intent) -> BoxResult<()> {
    info!("essential judge: ");
    
    // in essential part, all rule's will be hard coded.
    // essential rules will never be expired.
    // other than changing the code, user can't not change the rules.
    let special_id = STATIC_RULES["reject"].get_id();
    for rule in STATIC_RULES.values() {
        if rule.get_id() >= special_id {
                info!("judge np, rule id: {}",rule.get_id() );
                continue;
        }

        match rule_judge(intent, rule).await {
            Ok(()) => {
                info!("judge Pass")
            },
            Err(e) => return Err(e),
        }
    }

    info!("essential judge: Judge passed");
    Ok(())
}

// this judge is conducted depends on intent's attributes.
async fn user_judge(intent: &Intent) -> BoxResult<()> {
    // TODO: Maybe rule can be specified for intent type.
    for rule in RULES.lock().unwrap().iter_rules() {
        match rule_judge(intent, rule).await {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
    }

    info!("user judge: Judge passed");
    Ok(())
}

// judge the intent by user defined rule.
// if {
//  judge result is true then prevent intent to be executed.
//} else {
//  judge result is false then allow intent to be executed.
//}
async fn rule_judge(intent: &Intent, rule: &Rule) -> BoxResult<()> {
    
    match rule.get_rule_detail() {
        RuleDetail::Source(intent_source) 
            => if intent.get_source() == intent_source { 
                return Err(Box::new(JudgeError::new("We do not accept intent from the source now.")));
            },
        RuleDetail::Time 
            => if rule.is_expired() {
                return Err(Box::new(JudgeError::new("We do not accept intent from the source now.")));
            },
        RuleDetail::Weekday(weekday) => {
            let now = Local::now().date_naive().weekday();
            if now == *weekday {
                return Err(Box::new(JudgeError::new("We do not accept intent today.")));
            }
        },
        RuleDetail::Prompt(rule_description) => {
            let prompt_content = format!("the rule description is: {}\n the intent description is: {}.", rule_description, intent.get_description());
            match prompt("If the intent conform to the rule, return true, otherwise return false. do not ouput dot or any other things",&prompt_content).await.as_str() {
                "true" => return Err(Box::new(JudgeError::new("We do not accept such intent now."))),
                _ => (),
            };
        },
        RuleDetail::Function(rule_func) 
            => {
                if rule_func(intent) {
                    return Err(Box::new(JudgeError::new("We do not accept such intent.")));
                }
            },
        RuleDetail::Program(path) 
            => {
                let status = Command::new("rustc")
                    .arg(path)
                    .status()
                    .expect("no such function");
                if status.success() {
                    return Err(Box::new(JudgeError::new("We do not accept such intent now.")));
                }
            },
        _ => (),
    };
    Ok(())
}

pub fn format_reject(intent: &str, reason: &str) -> String {
    info!("reject: Reject the intent: {}", intent);
    let response = format!(
        "intent: {}\nreject reason: {}", 
        intent, 
        reason
    );
    response
}