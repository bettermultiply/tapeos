// in this file, we will implement the disassembler.

use crate::base::intent::{Intent, SubIntent};
use regex::Regex;

pub fn disassembler(intent: &mut Intent) -> Option<()> {
    let sub_intents: Vec<SubIntent>;
    let rough_intent = disassemble_intent(intent.get_description());
    match format_check(&rough_intent) {
        true => {
            sub_intents = parse_rough_intent(&rough_intent);
            // Logic to disassemble the intent based on resources
        }
        false => {
            return None;
        }
    }

    if !sub_intents.is_empty() {
        intent.add_sub_intent(sub_intents);
    }

    Some(())
}

fn disassemble_intent<'a>(intent: &str) -> String {
    // TODO: use ai here to disassemble the intent.
    // maybe we should design prompt here.
    intent.to_string()
}

fn format_check(rough_intent: &str) -> bool {
    // TODO: use regex here to check the format of the rough intent.
    // Check if string contains only alphanumeric chars, spaces, semicolons and basic punctuation
    let re = Regex::new(r"^[a-zA-Z0-9\s;.,!?'-]+$").unwrap();
    re.is_match(rough_intent)
}

fn parse_rough_intent(rough_intent: &str) -> Vec<SubIntent> {
    // TODO: design actual logic here.
    let sub_intents: Vec<SubIntent> = rough_intent.split(";").map(|s| SubIntent::new(s.to_string(), vec![])).collect();
    sub_intents
}   