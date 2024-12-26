// in this file, we will implement the adaptor between the higher level 
// system(tapeos will act as its subsystem/resource).
// adaptor is the gateway between the higher level system and the lower level system(tapeos).
// it will receive the intent from the higher level system and send response back.
// adaptor will maintain the information and but not maintain the status of the higher level system.

use crate::base::intent::Intent;

#[allow(unused)]
fn receive_intent(intent: &Intent) -> bool {
    // TODO: implement the logic to receive the intent from the higher level system
    true
}

#[allow(unused)]
fn send_response(response: &str) -> bool {
    // TODO: implement the logic to send the response to the higher level system
    true
}

#[allow(unused)]
fn maintain_information(information: &str) -> bool {
    // TODO: implement the logic to maintain the information of the higher level system
    true
}








