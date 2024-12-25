fn disassemble_intent(intent: &str, resources: &[&str]) -> Vec<String> {
    let mut sub_intents = Vec::new();

    // Logic to disassemble the intent based on resources
    for resource in resources {
        let sub_intent = format!("{} for {}", intent, resource);
        sub_intents.push(sub_intent);
    }

    sub_intents
}
