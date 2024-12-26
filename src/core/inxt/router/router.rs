// in this file, we will implement the router for the intent execution.
// the router will distribute the intent to the corresponding resource or subsystem.

use crate::base::intent::Intent;
use crate::core::inxt::filter::judge::intent_judge;
use crate::base::resource::Resource;

// the distributer will distribute the sub-intents from disassembler to the corresponding resource or subsystem.
pub fn distribute(sub_intents: &[Intent]) {
    for sub_intent in sub_intents {
        select_resource(sub_intent);
        // TODO: implement the logic to distribute the sub-intent to the corresponding resource or subsystem
    }
    
}

// TODO: when shall we select the resource? when disassemble? when distribute?
fn select_resource<'a>(sub_intent: &'a Intent<'a>) -> &'a dyn Resource {
    // TODO: implement the logic to select the resource for the sub-intent
    let _score = score(sub_intent);
    // Temporary placeholder - replace with actual resource selection logic
    todo!("Implement resource selection logic")
}

fn score(sub_intent: &Intent) -> u8 {
    // TODO: implement the logic to score the sub-intent
    0
}

fn store_unprocessed_subintents(unprocessed: Vec<String>) {
    // Logic to store unprocessed sub-intents for future redistribution
    // This could involve adding them to a queue or a database for later processing
    println!("Storing unprocessed sub-intents for future redistribution: {:?}", unprocessed);
}

// Function to handle the event of resource rejection
fn handle_resource_rejection(sub_intent: Intent, resources: &[&str]) {
    // Logic to redistribute the rejected sub-intent to available resources
    for resource in resources {
        if intent_judge(&sub_intent) {
            println!("Redistributing rejected sub-intent '{}' to resource '{}'", sub_intent.get_description(), resource);
            // Here you can add logic to actually redistribute the sub-intent to the resource
        }
    }
}
