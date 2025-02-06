// in this file, we will implement the disassembler.

use log::warn;
use regex::Regex;
use crate::{
    tools::llmq::prompt,
    base::intent::{Intent, SubIntent},
    components::linkhub::seeker::get_all_resource_info, 
};

pub async fn disassembler(intent: &mut Intent) -> Option<()> {
    // info!("disassembler: Start to disassemble intent");
    let sub_intents: Vec<SubIntent>;
    let mut tries_count = 3;
    let mut last_outcome = "".to_string();
    loop {  
        let rough_intent = 
            disassemble_intent(
                intent.get_description(), 
                last_outcome.as_str()
            ).await;

        last_outcome = rough_intent.clone();
        // info!("rough disassembled intent: {}", rough_intent);
            
        let to_parse_intent = rough_intent;
        match format_check(&to_parse_intent) {
            true => {
                sub_intents = parse_rough_intent(to_parse_intent);
                break;
            }
            false => {
                tries_count -= 1;
                if tries_count == 0 {
                    warn!("disassembler: sub_intents error");
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
    let resource_info = get_all_resource_info().await;
    let s_prompt = 
    "
The user will provide description of Intent, last outcome and information about all available resources, Resources will be given in format: `type_name/description/status`.
And your work is to Disassemble the Intent into sub-intents based on available resources, so that they can be solve by different resources parallel.
Here are some rules you need to know when disassemble intents:
    1. You must try your best to use resources to finish intents.
    2. There may be fuzzy intent that even not tell what should do to finish the intent, but you must guess what the intent really want to do by life knowledge, and disassemble it into many excutable sub-intent so that it can be will finish with better experience. 
    3. You must know that not all resource must be used. So in the extreme case, if you judge that no resource can solve this intent, just return 'None'(without anything else);
    4. Last outcome may be wrong, in error format or there are some hidden disassemble way you do not find(but if you still judge there are no ways to do that you can return None again.).
    5. If different resources can finish same sub intent, but with different effect, the better way is disassemble it into multiple intent. 
Outcome should be given in format: sub-intent_1:available_device_1/available_device_2/.../available_device_n;sub-intent_2:available_device_1/available_device_2/.../available_device_m;...;sub-intent_n:available_device_1/available_device_2/.../available_device_k;
You should not add any blank except the name of resource have one, which means you should not change the resources' name as well.

Example Input1:
Intent: store my name 'BM' and my birthday '12.01'
last outcome: 
resources: 1. MySQL/MySQL can store, organize, and manage data in structured tables./avaiable; 2. MongoDB/MongoDB is a NoSQL database that stores data in flexible, JSON-like documents instead of tables./avaiable; 3. Google Drive/Google Drive is a cloud-based storage service that allows you to store, share, and access files from anywhere./avaiavle;
 
Example Output1:
store name 'BM':MySQL/MongoDB/Google Drive;store birthday '12.01':MongoDB/Google Drive/MySQL;

Example Wrong Ouput1:
store name 'BM': MySQL/MongoDB/Google Drive;store birthday '12.01':MongoDB/Google Drive/MySQL;    reason: wrong name, our resource is 'MySQL' not ' MySQL'

Example Wrong Ouput2:
store name 'BM': MySQL/MongoDB/Google Drive;store birthday '12.01':MongoDB/Google Drive/MySQL     reason: lack of ';'

Example Input2:
Intent: Power on my computer
last outcome: 
resources: 1. MySQL/MySQL can store, organize, and manage data in structured tables./avaiable; 2. MongoDB/MongoDB is a NoSQL database that stores data in flexible, JSON-like documents instead of tables./avaiable; 3. Google Drive/Google Drive is a cloud-based storage service that allows you to store, share, and access files from anywhere./avaiavle;
 
Example Output2:
None

Example Wrong Ouput3:
store name 'BM': MySQL/MongoDB/Google Drive;    reason: no resourc can do that

None;                                           reason: use some things other than None
";
//     let s_prompt = 
// "I'll give you some information about Intent, Last Outcome(which is wrong or error format) and Available Resources.
// Resources is in format: `type_name/description/status`, and will have format: `type_name/description/status`,
// Disassemble the intent into sub-intents based on the available resources. Use the following format for output:
// sub-intent_1:available_device_1/available_device_2/.../available_device_n;sub-intent_2:available_device_1/available_device_2/.../available_device_m;...;sub-intent_n:available_device_1/available_device_2/.../available_device_k;

// Remember that we just want to use the resource to deal with the intent and do not do duplicate things, and we do not sub-intent with device name.
// If the intent cannot be implemented with the given resources, return None. Do not output any additional information.";

    let u_prompt = 
        format!(
"Intent: {}
Last Outcome: {}
Available Resources: {}",
            // "
            //     the intent is: `{}`\n and the last outcome is: `{}`\n your last outcome maybe in wrong format, disassemble again and try to fix it.and we have some resources can be used to execute the intent, and will have format: `type_name/description/status`, they are: `{}`\nplease disassemble the intent into sub-intents based on resources.and use the format: `1_sub-intent:available_device_1/available_device_2/.../available_device_n;2_sub-intent:available_device_1/available_device_2/.../available_device_m;...;n_sub-intent:available_device_1/available_device_2/.../available_device_z;` for output.\nIt is essential that if you do not think it is ok to implement the intent with these resources, please return `None`.And don't output anyother information.
            // ",
            intent,
            last_outcome,
            resource_info
        );
    
    prompt(&s_prompt, &u_prompt).await
}

fn format_check(rough_intent: &str) -> bool {
    if rough_intent == "None" {
        warn!("None Intent");
        return false;
    }
    let re = Regex::new(r"(:*/+*;)+").unwrap();
    let result = re.is_match(rough_intent);
    // info!("check result is {} {}", result, rough_intent);
    result
}

fn parse_rough_intent(rough_intent: String) -> Vec<SubIntent> {
    let mut sub_intents: Vec<SubIntent> = vec![];
    let sub_intents_pairs = rough_intent.split(";").filter(|s| !s.is_empty()).collect::<Vec<&str>>();
    for sub_intent_pair in sub_intents_pairs {
        let sub_intent = sub_intent_pair.split(":").collect::<Vec<&str>>();
        if sub_intent.len() != 2 {
            continue;
        }
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