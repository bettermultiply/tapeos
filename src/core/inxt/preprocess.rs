// in this file, we will implement the filter for the intent.
// the filter will judge if the intent can be processed by the tapeos.
// here we'll simply show the cal map:
// process -> filter -> essential_judge 
//                   -> user_judge      
//         ->special_execution            => rule_judge
//         ->reject
// any time, true means pass the test.
use chrono::{Local, Datelike};
use std::process::Command;

use crate::{
    tools::llmq::prompt,
    base::{
        errort::{BoxResult, JudgeError},
        intent::Intent, 
        rule::{Rule, RuleDetail, RULES, STATIC_RULES}, 
        staticrule,
    }, 
};

pub enum JudgeResult {
    Reject(String),
    Accept,
    Execution,
}

// preprocess the intent.
pub async fn process(intent: &mut Intent) -> JudgeResult {
    // info!("process: Judge the intent: {}", intent.get_description());
    
    match filter(intent).await {
        Ok(_) => (),
        Err(e) => return JudgeResult::Reject(format_reject(intent.get_description(), &format!("{}", e))),
    }
    
    // info!("process: Filter passed");
    
    match spec_exec(intent).await {
        Ok(()) => (),
        Err(_) => return JudgeResult::Execution,
        //  => return JudgeResult::Reject(format_reject(intent.get_description(), &format!("{}", e))),

    }

    // info!("process: Special execution passed");

    JudgeResult::Accept
}

// filter the unacceptible intent.
async fn filter(intent: &mut Intent) -> BoxResult<()> {
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
async fn spec_exec(intent: &mut Intent) -> BoxResult<()> {
    
    // let i = intent.get_description();
    // let i_pair = match i.split_once(":") {
        // Some(p) => (),
        // None => return Ok(()),
    // };
    // if i_pair.len() != 2 {
        // return Ok(false);
    // }

    let special_id = STATIC_RULES["reject"].get_id();
    for rule in STATIC_RULES.values().into_iter() {
            if rule.get_id() < special_id {
            // if rule.get_id() < special_id || i_pair[1] != rule.get_name() {
            continue;
        }
        match rule_judge(intent, rule).await {
            Err(e) => {
                // info!("special execution: {}", rule.get_name());
                return Err(e);   
            },
            Ok(()) => (),
        }
    }

    Ok(())
}

// should be used to judge  every intent.
async fn essential_judge(intent: &mut Intent) -> BoxResult<()> {
    // info!("essential judge: ");
    
    // in essential part, all rule's will be hard coded.
    // essential rules will never be expired.
    // other than changing the code, user can't not change the rules.
    let special_id = STATIC_RULES["reject"].get_id();
    for rule in STATIC_RULES.values() {
        if rule.get_id() >= special_id {
                continue;
        }
        // info!("judge, rule id: {}",rule.get_name() );
        match rule_judge(intent, rule).await {
            Ok(()) => {
                // info!("judge Pass")
            },
            Err(_) => {
                return Err(Box::new(JudgeError::new(&format!(
                    "We do not accept such intent for {} reason",
                    rule.get_name()
                ))));
            },
        }
    }

    // info!("essential judge: Judge passed");
    Ok(())
}

// this judge is conducted depends on intent's attributes.
async fn user_judge(intent: &mut Intent) -> BoxResult<()> {
    // TODO: Maybe rule can be specified for intent type.
    for rule in RULES.lock().await.iter_rules() {
        match rule_judge(intent, rule).await {
            Ok(_) => (),
            Err(_) => {
                return Err(Box::new(JudgeError::new(
                    "We do not accept such intent for user rule reason"
                )));
            },
        }
    }

    // info!("user judge: Judge passed");
    Ok(())
}

// judge the intent by user defined rule.
// if {
//  judge result is true then prevent intent to be executed.
//} else {
//  judge result is false then allow intent to be executed.
//}
async fn rule_judge(intent: &mut Intent, rule: &Rule) -> BoxResult<()> {
    // info!("rule: {}", rule.get_description());
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
            let u_prompt = format!(
                "the rule description is: {}\n the intent description is: {}.", 
                rule_description, intent.get_description()
            );
            let s_prompt = format!(
                "User will give you some Intent, and you need to judge whether it conform to the sentences describe below
                '{}'

                if it conform, return true, otherwise return false, do not ouput dot or any other things.
                You need to be tolerant about some general intent.",
                rule_description
            );
            match prompt(&s_prompt,&u_prompt).await.as_str() {
                "true" => return Err(Box::new(JudgeError::new("We do not accept such intent for reason of risk, privilige, rule limit and so on."))),
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
        RuleDetail::AsyncF(s) => {
            if *s == "rule" {
                if staticrule::rule(intent).await {
                    return Err(Box::new(JudgeError::new("Add rule.")));
                }
            } else if *s == "status" {
                if staticrule::status(intent).await {
                    return Err(Box::new(JudgeError::new("refresh status.")));
                }
            } else if *s == "direct" {
                if staticrule::direct(intent).await {
                    return Err(Box::new(JudgeError::new("direct send intent.")));
                }
            }
        }
        _ => (),
    };
    Ok(())
}

pub fn format_reject(intent: &str, reason: &str) -> String {
    // info!("reject: Reject the intent: {}", intent);
    let response = format!(
        "intent: {}\nreject reason: {}", 
        intent, 
        reason
    );
    response
}