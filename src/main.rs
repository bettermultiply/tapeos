use tapeos::{
    core::inxt::intent,
    base::intent::Intent,
    base::intent::IntentSource,
    base::intent::IntentType,
};

#[tokio::main]
async fn main() {
    let intent = Intent::new("store my name".to_string(), IntentSource::Resource, IntentType::Intent, None);
    println!("main: ");
    println!("main: Try to execute intent");
    intent::handler(intent).await;
    println!("main: Try ended");
}