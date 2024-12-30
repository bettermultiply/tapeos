// in this file, we will implement the disassembler.

use regex::Regex;
use crate::{
    base::{
        resource::Resource,
        intent::{Intent, SubIntent}
    },
    tools::llmq::prompt,
    components::linkhub::seeker::RESOURCES
};

pub async fn disassembler<'a>(intent: &mut Intent<'a>) -> Option<()> {
    let sub_intents: Vec<SubIntent>;
    let mut tries_count = 3;
    let mut last_outcome = "".to_string();
    loop {  
        let rough_intent = disassemble_intent(intent.get_description(), last_outcome.as_str()).await;
        let to_parse_intent = rough_intent.clone();
        last_outcome = rough_intent.clone();
        match format_check(&to_parse_intent) {
            true => {
                sub_intents = parse_rough_intent(to_parse_intent);
                break;
            }
            false => {
                tries_count -= 1;
                if tries_count == 0 {
                    return None;
                }
            }
        }
    }
    if !sub_intents.is_empty() {
        intent.add_sub_intent(sub_intents);
    }

    Some(())
}

async fn disassemble_intent(intent: &str, last_outcome: &str) -> String {
    let mut resources = String::new();
    for resource in RESOURCES.lock().unwrap().iter() {
        let r: &dyn Resource = resource.as_ref();
        resources += format!("{}/{}/", r.get_type_name(), r.get_description()).as_str();
    }
    let prompt_content = 
        format!(
            "
                the intent is: `{}`\n and the last outcome is: `{}`\n your last outcome maybe in wrong format, disassemble again and try to fix it.
                and we have some resources can be used to execute the intent, and will have format: `type_name/description/status`,
                they are: `{}`\n
                please disassemble the intent into sub-intents based on resources.and use the format: `1_sub-intent:available_device_1/available_device_2/.../available_device_n;2_sub-intent:available_device_1/available_device_2/.../available_device_m;...;n_sub-intent:available_device_1/available_device_2/.../available_device_z;` for output.\n
                it is essential that if you do not think it is ok to implement the intent with these resources, please return `None`.
            ",
            intent,
            last_outcome,
            resources
        );
    
    prompt(&prompt_content).await
}

fn format_check(rough_intent: &str) -> bool {
    if rough_intent == "None" {
        return false;
    }
    let re = Regex::new(r"(:/*;)+").unwrap();
    re.is_match(rough_intent)
}

fn parse_rough_intent(rough_intent: String) -> Vec<SubIntent> {
    let mut sub_intents: Vec<SubIntent> = vec![];
    let sub_intents_pairs = rough_intent.split(";").collect::<Vec<&str>>();
    for sub_intent_pair in sub_intents_pairs {
        let sub_intent_pair = sub_intent_pair.split(":").collect::<Vec<&str>>();
        let sub_intent_name = sub_intent_pair[0].to_string();
        let sub_intent_resources = sub_intent_pair[1].split("/").map(|r| r.to_string()).collect::<Vec<String>>();

        let sub_intent = SubIntent::new(sub_intent_name, sub_intent_resources);
        sub_intents.push(sub_intent);
    }
    sub_intents
}   