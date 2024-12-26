use tapeos::base::intent::Intent;
use tapeos::core::inxt::intent::execute_intent;
use tapeos::components::linkhub::seeker::seek;

fn main() {
    println!("Hello, world!");
    let intent: Intent<'_> = Intent::new("intent1".to_string());
    execute_intent(intent);
    seek();
}
