use crate::base::intent::Intent;
use crate::base::intent::IntentType;

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
