// in this file, we will store rules for judging the intent.

use std::{
    sync::LazyLock,
    collections::HashMap,
    time::{Duration, Instant}
};
use chrono::Weekday;
use crate::{
    tools::idgen::{generate_id, IdType},
    base::intent::{IntentSource, Intent}
};


pub static RULES: LazyLock<RuleSet> = LazyLock::new(|| RuleSet::new());

// judge whether to accept the intent.
// actually rule means not to do something.
pub enum RuleDetail {
    Essential(fn(&Intent) -> bool),
    Source(IntentSource), // based on the source of the intent.
    Time(Duration), // based on the time to reject the intent.
    Weekday(Weekday), // based on the weekday to reject the intent.
    UserDefined(String), // based on user description and ai.
}

pub struct Rule {
    id: i64,
    name: String,
    description: String,
    // we encourage that one Rule judge one feature of the intent.
    rule_detail: RuleDetail,
    valid_time: Duration,
    created_time: Instant,
}

impl Rule {
    pub fn new(name: String, description: String, rule_detail: RuleDetail, valid_time: Duration) -> Self {
        let mut new_id: i64;
        loop {
            new_id = generate_id(IdType::Resource);
            // id < 1000 is used for essential rules.
            if new_id >= 1000 {
                break;
            }
        }
        Self { id: new_id, name, description, rule_detail, valid_time, created_time: Instant::now() }
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_rule_detail(&self) -> &RuleDetail {
        &self.rule_detail
    }

    pub fn get_valid_time(&self) -> Duration {
        self.valid_time
    }

    pub fn get_created_time(&self) -> Instant {
        self.created_time
    }

    pub fn is_expired(&self) -> bool {
        self.created_time + self.valid_time <= Instant::now()
    }

    // we don't provide the function to change the rule directly, 
    // because it's not a good practice to change the rule after it's created.
    // instead, we encourage to create a new rule and delete the old one to change the rule.
    // pub fn change_rule(&mut self, new_rule: Rule) {
        // self.rule = new_rule.rule;
    // }
}

pub struct RuleSet {
    rules: Vec<Rule>,
}

impl RuleSet {
    
    pub fn new() -> Self {
        Self { rules: vec![] }
    }
    
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }
    
    pub fn get_rules_by_name(&self, name: &str) -> Vec<&Rule> {
        self.rules.iter().filter(|r: &&Rule| r.name == name).collect()
    }
    
    pub fn get_rule_by_id(&self, id: i64) -> Option<&Rule> {
        self.rules.iter().find(|r: &&Rule| r.id == id)
    }
    
    pub fn delete_rule(&mut self, id: i64) {
        self.rules.retain(|r: &Rule| r.id != id);
    }

    pub fn expire_rules(&mut self) {
        self.rules.retain(|r: &Rule| !r.is_expired() || r.id < 1000);
    }
}

pub fn iter_rules() -> impl Iterator<Item = &'static Rule> {
    RULES.rules.iter()
}

pub static STATIC_RULES: LazyLock<HashMap<&str, Rule>> = LazyLock::new(|| HashMap::from([
    (
        "risk", 
        Rule {
            id: 0,
            name: "risk".to_string(), 
            description: "risk".to_string(), 
            rule_detail: RuleDetail::Essential(risk), 
            valid_time: Duration::from_secs(0),
            created_time: Instant::now(),
        },
    ),
    (
        "privilege", 
        Rule {
            id: 1,
            name: "privilege".to_string(), 
            description: "privilege".to_string(), 
            rule_detail: RuleDetail::Essential(privilege), 
            valid_time: Duration::from_secs(0),
            created_time: Instant::now(),
        },
    ),
    (
        "reject", 
        Rule {
            id: 2,
            name: "reject".to_string(), 
            description: "reject".to_string(), 
            rule_detail: RuleDetail::Essential(reject), 
            valid_time: Duration::from_secs(0),
            created_time: Instant::now(),
        },
    ),
]));


// TODO: we need to implement the rule functions.
fn risk(intent: &Intent) -> bool {
    intent.get_description().contains("risk")
}

fn privilege(intent: &Intent) -> bool {
    intent.get_description().contains("sudo")
}

fn reject(intent: &Intent) -> bool {
    intent.get_description().contains("reject")
}