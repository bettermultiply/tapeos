// in this file, we will implement the monitor for the intent execution.
// the monitor will monitor the execution of the intent and provide the feedback to the higher level system.

use core::time;
use std::thread::sleep;

use crate::base::intent::Intent;

pub async fn monitor(intent: &mut Intent) {
    println!("monitor: Start to monitor intent");
    loop {
        let mut is_finished = true;
        for sub_intent in intent.iter_sub_intent() {
            if sub_intent.is_complete() {
                continue;
            }
            is_finished = false;

            
        }
        if is_finished {
            break;
        }
        sleep(time::Duration::from_secs(1));
    }
    intent.complete();
}

