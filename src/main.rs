use tapeos::base::intent::Intent;
use tapeos::base::intent::IntentSource;
use tapeos::core::inxt::intent::execute_intent;
use tapeos::components::linkhub::seeker::seek;

fn main() {
    println!("Hello, world!");
    let intent: Intent = Intent::new("intent1".to_string(), IntentSource::Resource);
    execute_intent(intent);
    let _ = seek();
}
