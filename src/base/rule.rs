// in this file, we will store rules for judging the intent.

use std::time;
use std::sync::LazyLock;
use crate::tools::idgen::{generate_id, IdType};

pub static RULES: LazyLock<RuleSet> = LazyLock::new(|| RuleSet::new());


#[allow(unused)]
pub struct Rule {
    id: i64,
    name: String,
    description: String,
    // we encourage that one Rule judge one feature of the intent.
    rule: String, // TODO: we need more specific and controllable ways to describe and apply the rule.
    valid_duration: time::Duration,
}

impl Rule {
    pub fn new(name: String, description: String, rule: String, valid_duration: time::Duration) -> Self {
        let mut new_id: i64;
        loop {
            new_id = generate_id(IdType::Resource);
            // id < 1000 is used for essential rules.
            if new_id >= 1000 {
                break;
            }
        }
        Self { id: new_id, name, description, rule, valid_duration }
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

    pub fn get_rule(&self) -> &str {
        &self.rule
    }

    pub fn get_valid_duration(&self) -> time::Duration {
        self.valid_duration
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
}
