// in this file, we will store rules for judging the intent.

use std::{
    collections::HashMap, 
    path::PathBuf, 
    sync::{Arc, LazyLock}, 
    time::{Duration, Instant},
};
use chrono::Weekday;
use tokio::sync::Mutex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{
    tools::idgen::{generate_id, IdType},
    base::intent::IntentSource,
    base::intent::Intent,
    base::staticrule,
};

lazy_static! {
    pub static ref RULES: Arc<Mutex<RuleSet>> = Arc::new(Mutex::new(RuleSet::new()));
}

// judge whether to accept the intent.
// actually rule means not to do something.
pub enum RuleDetail {
    Function(fn(&mut Intent) -> bool),
    AsyncF(String),
    Program(PathBuf),
    Prompt(String),
    Source(IntentSource), // based on the source of the intent.
    Time, // based on the time to reject the intent.
    Weekday(Weekday), // based on the weekday to reject the intent.
    Undefine,
}

#[derive(Deserialize, Serialize)]
pub enum TransRuleDetail {
    Prompt(String),
    Source(IntentSource), // based on the source of the intent.
    Time, // based on the time to reject the intent.
    Weekday(TransWeekday), // based on the weekday to reject the intent.
}

#[derive(Deserialize, Serialize)]
pub enum TransWeekday {
    /// Monday.
    Mon = 0,
    /// Tuesday.
    Tue = 1,
    /// Wednesday.
    Wed = 2,
    /// Thursday.
    Thu = 3,
    /// Friday.
    Fri = 4,
    /// Saturday.
    Sat = 5,
    /// Sunday.
    Sun = 6,
}

pub struct Rule {
    id: i64,
    name: String,
    description: String,
    // we encourage that one Rule judge one feature of the intent.
    detail: RuleDetail,
    valid_time: Duration,
    created_time: Instant,
}

#[derive(Deserialize, Serialize)]
pub struct TransRule {
    pub name: String,
    pub description: String,
    pub valid_time: Duration,
    pub detail: TransRuleDetail,
}

impl Rule {
    pub fn new(name: String, description: String, detail: RuleDetail, valid_time: Duration) -> Self {
        let mut new_id: i64;
        loop {
            new_id = generate_id(IdType::Resource);
            // id < 1000 is used for essential rules.
            if new_id >= 1000 {
                break;
            }
        }
        Self { id: new_id, name, description, detail, valid_time, created_time: Instant::now() }
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
        &self.detail
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
    fn new() -> Self {
        Self {
            rules: vec![]
        }
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

    pub fn iter_rules(&self) -> impl Iterator<Item = &Rule> {
        self.rules.iter()
    }
}

pub static STATIC_RULES: LazyLock<HashMap<&str, Rule>> = LazyLock::new(|| HashMap::from([
    (
        "risk", 
        Rule {
            id: 0,
            name: "risk".to_string(), 
            description: "risk".to_string(), 
            // detail: RuleDetail::AsyncF("risk".to_string()), 
            detail: RuleDetail::Prompt(staticrule::RISK_PROMPT.to_string()), 
            valid_time: Duration::from_secs(0),
            created_time: Instant::now(),
        },
    ),
    // (
    //     "privilege", 
    //     Rule {
    //         id: 1,
    //         name: "privilege".to_string(), 
    //         description: "privilege".to_string(), 
    //         detail: RuleDetail::Prompt(staticrule::PRIVILEGE_PROMPT.to_string()), 
    //         valid_time: Duration::from_secs(0),
    //         created_time: Instant::now(),
    //     },
    // ),
    (
        "emergency", 
        Rule {
            id: 2,
            name: "emergency".to_string(), 
            description: "emergency".to_string(), 
            detail: RuleDetail::Function(staticrule::emergency), 
            valid_time: Duration::from_secs(0),
            created_time: Instant::now(),
        },
    ),
    (
        "reject", 
        Rule {
            id: 500,
            name: "reject".to_string(), 
            description: "reject".to_string(), 
            detail: RuleDetail::Function(staticrule::reject), 
            valid_time: Duration::from_secs(0),
            created_time: Instant::now(),
        },
    ),
    (
        "rule", 
        Rule {
            id: 501,
            name: "rule".to_string(), 
            description: "set new rule".to_string(), 
            detail: RuleDetail::AsyncF("rule".to_string()), 
            valid_time: Duration::from_secs(0),
            created_time: Instant::now(),
        },
    ),
    (
        "status", 
        Rule {
            id: 502,
            name: "status".to_string(), 
            description: "refresh status".to_string(), 
            detail: RuleDetail::AsyncF("status".to_string()), 
            valid_time: Duration::from_secs(0),
            created_time: Instant::now(),
        },
    ),
    (
        "direct", 
        Rule {
            id: 502,
            name: "direct".to_string(), 
            description: "send directly to resource".to_string(), 
            detail: RuleDetail::AsyncF("direct".to_string()), 
            valid_time: Duration::from_secs(0),
            created_time: Instant::now(),
        },
    ),
]));
