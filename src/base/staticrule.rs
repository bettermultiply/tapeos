use chrono::Weekday;

use crate::base::intent::Intent;
use crate::base::intent::IntentType;
use crate::components::linkhub::seeker::fresh_resource_status;

use super::errort::BoxResult;
use super::errort::JudgeError;
use super::resource::Status;
use super::rule::Rule;
use super::rule::RuleDetail;
use super::rule::TransRule;
use super::rule::TransRuleDetail;
use super::rule::RULES;

pub static RISK_PROMPT: &str = "
    check if the intent below will cause risk.
    if yes return true, else return false.
    intent: ";

pub static PRIVILEGE_PROMPT: &str = "
    check if the intent below have privilege to execute.
    if yes return true, else return false.
    intent: ";

pub fn reject(intent: &Intent) -> bool {
    intent.get_intent_type() == &IntentType::Reject
}

pub fn rule(intent: &Intent) -> bool {
    match try_add2rule(intent.get_description()) {
        Ok(_) => false,
        Err(_) => true,
    }
}

fn try_add2rule(i: &str) -> BoxResult<()> {
    // parse the rule
    let rule: TransRule = serde_json::from_str(i)?;
    let r_detail = match rule.detail {
        TransRuleDetail::Time => RuleDetail::Time,
        TransRuleDetail::Prompt(s) => RuleDetail::Prompt(s),
        TransRuleDetail::Source(s) => RuleDetail::Source(s),
        TransRuleDetail::Weekday(w) => {
            let weekday: Weekday = match w as u8 {
                0 => Weekday::Mon,
                1 => Weekday::Thu,
                2 => Weekday::Wed,
                3 => Weekday::Tue,
                4 => Weekday::Fri,
                5 => Weekday::Sat,
                6 => Weekday::Sun,
                _ => return Err(Box::new(JudgeError::new("no such week"))),
            };
            RuleDetail::Weekday(weekday)
        },
    };
    let r = Rule::new(rule.name, rule.description, r_detail, rule.valid_time);
    RULES.lock().unwrap().add_rule(r);
    Ok(())
}

pub fn status(intent: &Intent) -> bool {
    match try_fresh2status(intent.get_description(), intent.get_resource().unwrap()) {
        Ok(_) => false,
        Err(_) => true,
    }
}

fn try_fresh2status(i: &str, name: &str) -> BoxResult<()> {
    let status: Status = serde_json::from_str(i)?;
    fresh_resource_status(name, status);
    Ok(())
}