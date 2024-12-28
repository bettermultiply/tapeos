// in this file, we will store rules for judging the intent.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::LazyLock;
use crate::tools::idgen::{generate_id, IdType};
use chrono::Weekday;
use crate::base::resource::Resource;
pub static RULES: LazyLock<RuleSet> = LazyLock::new(|| RuleSet::new());

// judge whether to accept the intent.
// actually rule means not to dp.
pub enum RuleDetail<'a> {
    Essential,
    Source(&'a dyn Resource), // based on the source of the intent.
    Description(String), // based on the description of the intent. which actually we will use ai to judge intent.
    Time(Duration), // based on the time to reject the intent.
    Weekday(Weekday), // based on the weekday to reject the intent.
}

#[allow(unused)]
pub struct Rule<'a> {
    id: i64,
    name: String,
    description: String,
    // we encourage that one Rule judge one feature of the intent.
    rule_detail: RuleDetail<'a>,
    // rule: String, // TODO: we need more specific and controllable ways to describe and apply the rule.
    valid_time: Duration,
    created_time: Instant,
}

impl<'a> Rule<'a> {
    pub fn new(name: String, description: String, rule_detail: RuleDetail<'a>, valid_time: Duration) -> Self {
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

pub struct RuleSet<'a> {
    rules: Vec<Rule<'a>>,
}

impl<'a> RuleSet<'a> {
    
    pub fn new() -> Self {
        Self { rules: vec![] }
    }
    
    pub fn add_rule(&mut self, rule: Rule<'a>) {
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

pub fn iter_rules<'a>() -> impl Iterator<Item = &'a Rule<'a>> {
    RULES.rules.iter()
}

pub static STATIC_RULES: LazyLock<HashMap<&str, Rule>> = LazyLock::new(|| HashMap::from([
    (
        "risk", 
        Rule {
            id: 0,
            name: "risk".to_string(), 
            description: "risk".to_string(), 
            rule_detail: RuleDetail::Essential, 
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
            rule_detail: RuleDetail::Essential, 
            valid_time: Duration::from_secs(0),
            created_time: Instant::now(),
        },
    )
]));
