use log::info;
use chrono::Weekday;

use crate::{
    base::{
        intent::{Intent, IntentType},
        resource::{Position, Status},
        errort::{BoxResult, JudgeError},
        rule::{Rule, RuleDetail, TransRule, TransRuleDetail, RULES}
    },
    components::linkhub::{
        internet::seek::SOCKET,
        seeker::{fresh_resource_status, remove_resource_by_name, BLUETOOTH_RESOURCES, INTERNET_RESOURCES,}
    }
};

pub static RISK_PROMPT: &str = "
    This intent will do harm to systems or humans. For example, it will break some rule or its side effect will cause a disaster";

pub static PRIVILEGE_PROMPT: &str = "
    This resource do not have the enough privilege to do such intent.";

pub fn reject(intent: &mut Intent) -> bool {
    intent.get_intent_type() == &IntentType::Reject
}

pub fn emergency(intent: &mut Intent) -> bool {
    if intent.get_description().contains("emergency") {
        intent.set_emergency();
    }
    false
}

pub async fn rule(intent: &Intent) -> bool {
    match try_add2rule(intent.get_description()).await {
        Ok(_) => true,
        Err(_) => false,
    }
}

async fn try_add2rule(i: &str) -> BoxResult<()> {
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
    RULES.lock().await.add_rule(r);
    Ok(())
}

pub async fn status(intent: &Intent) -> bool {
    match try_fresh2status(intent.get_description(), intent.get_resource().unwrap()).await {
        Ok(_) => true,
        Err(_) => false,
    }
}

async fn try_fresh2status(i: &str, name: &str) -> BoxResult<()> {
    let status: Status = serde_json::from_str(i)?;
    if !check_position(status.get_position()) {
        remove_resource_by_name(name).await;
        return Ok(());
    }
    fresh_resource_status(name, status).await;
    Ok(())
}

fn check_position(p: &Position) -> bool {
    let v_position = ((-100.0, 100.0), (-100.0, 100.0), (-100.0, 100.0));

    p.x > (v_position.0).0 
    &&  p.x < (v_position.0).1
    &&  p.y > (v_position.1).0
    &&  p.y < (v_position.1).1
    &&  p.z > (v_position.2).0
    &&  p.z < (v_position.2).1
}

// direct command will have format:
// trick:`resource_name`:`target_command`
pub async fn direct(intent: &Intent) -> bool {
    let desc = intent.get_description();
    let d_s = desc.split_once(":");
    let (trick, s) = match d_s {
        Some(s) => s,
        None => return false, 
    };
    let d_s = s.split_once(":");
    let (name, command) = match d_s {
        Some(s) => s,
        None => return false, 
    };
    if trick != "trick" {
        return false
    }
    match directly_send(name, command).await {
        Ok(()) => true,
        Err(_) => false,
    }
}

pub async fn directly_send(name: &str, command: &str) -> BoxResult<()> {

    let r_m = INTERNET_RESOURCES.lock().await;
    let r = r_m.get(name);
    if r.is_some() {
        let r = r.unwrap().lock().await;
        let addr = r.get_address();
        let data: Vec<u8> = command.as_bytes().to_vec();
        SOCKET.lock().await.as_ref().unwrap().send_to(&data, addr).await?;
        info!("message send");
        return Ok(());
    }
    let _ = r;

    let r_m = BLUETOOTH_RESOURCES.lock().await;
    let r = r_m.get(name);
    if r.is_some() {
        let r = r.unwrap().lock().await;
        let char = r.get_char().as_ref().unwrap();
        let data: Vec<u8> = command.as_bytes().to_vec();
        char.write(&data).await?;
        info!("message send");
        return Ok(());
    }
    let _ = r;
    
    Err(Box::new(JudgeError::new("")))
}

// pub async fn risk(intent: &mut Intent) -> bool {
//     let s = intent.get_description().to_string().clone();
//     if spawn_blocking(move || {
//         let sequence_classification_model = match ZeroShotClassificationModel::new(Default::default ()) {
//             Ok(m) => m,
//             Err(_) => return true,
//         };
//         let candidate_labels = & ["risk", "no risk"];

//         let output = match sequence_classification_model.predict_multilabel(
//             &[s.as_ref()],
//             candidate_labels,
//             None,
//             128,
//         ) {
//             Ok(o) => o,
//             Err(_) => return true,
//         };
        
//         !(output[0][0].text == "risk")
//     }).await.unwrap() {
//         return true;
//     }
    
//     false
// }