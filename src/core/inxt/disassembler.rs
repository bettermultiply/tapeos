// in this file, we will implement the disassembler.

use regex::Regex;
use crate::{
    base::intent::{Intent, SubIntent}
    , components::linkhub::seeker::get_all_resource_info, tools::llmq::prompt
};

pub async fn disassembler(intent: &mut Intent) -> Option<()> {
    println!("disassembler: Start to disassemble intent");
    let sub_intents: Vec<SubIntent>;
    let mut tries_count = 3;
    let mut last_outcome = "".to_string();
    loop {  
        let rough_intent = 
            disassemble_intent(
                intent.get_description(), 
                last_outcome.as_str()
            ).await;

        println!("disassembler: rough_intent: {}", rough_intent);
        
        last_outcome = rough_intent.clone();
        let to_parse_intent = rough_intent;
        match format_check(&to_parse_intent) {
            true => {
                sub_intents = parse_rough_intent(to_parse_intent);
                println!("disassembler: sub_intents complete");
                break;
            }
            false => {
                tries_count -= 1;
                if tries_count == 0 {
                    return Some(());
                }
            }
        }
    }
    if !sub_intents.is_empty() {
        intent.add_sub_intent(sub_intents);
    }
    println!("disassembler: sub_intents error");

    None
}

async fn disassemble_intent(intent: &str, last_outcome: &str) -> String {
    let resource_info = get_all_resource_info();
    
    let prompt_content = 
        format!(
"Intent: {}
Last Outcome: {}
Available Resources: {}

Resources is in format: `type_name/description/status`, and will have format: `type_name/description/status`,
Disassemble the intent into sub-intents based on the available resources. Use the following format for output:
1_sub-intent:available_device_1/available_device_2/.../available_device_n;2_sub-intent:available_device_1/available_device_2/.../available_device_m;...;n_sub-intent:available_device_1/available_device_2/.../available_device_k;

If the intent cannot be implemented with the given resources, return None. Do not output any additional information.",
            // "
            //     the intent is: `{}`\n and the last outcome is: `{}`\n your last outcome maybe in wrong format, disassemble again and try to fix it.and we have some resources can be used to execute the intent, and will have format: `type_name/description/status`, they are: `{}`\nplease disassemble the intent into sub-intents based on resources.and use the format: `1_sub-intent:available_device_1/available_device_2/.../available_device_n;2_sub-intent:available_device_1/available_device_2/.../available_device_m;...;n_sub-intent:available_device_1/available_device_2/.../available_device_z;` for output.\nIt is essential that if you do not think it is ok to implement the intent with these resources, please return `None`.And don't output anyother information.
            // ",
            intent,
            last_outcome,
            resource_info
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
        let sub_intent = sub_intent_pair.split(":").collect::<Vec<&str>>();
        let sub_intent_name = sub_intent[0].to_string();
        let sub_intent_resources = sub_intent[1].split("/").map(|r| r.to_string()).collect::<Vec<String>>();

        let sub_intent = 
            SubIntent::new(
                sub_intent_name, sub_intent_resources
            );
        sub_intents.push(sub_intent);
    }

    sub_intents
}   