// in this file, we will implement the monitor for the intent execution.
// the monitor will monitor the execution of the intent and provide the feedback to the higher level system.

use std::{thread::sleep, time::Duration};

use log::info;

use crate::components::linkhub::{internet::seek::complete_intent, seeker::INTENT_QUEUE};

pub async fn monitor(id: i64) {
    info!("monitor: Start to monitor intent");
    loop {
        let mut i_q = INTENT_QUEUE.lock().await;
        for i in i_q.iter_mut() {
            if i.get_id() != id {
                continue;
            }
            if i.is_complete() {
                complete_intent(i).await.unwrap();
                i_q.retain(|i| i.get_id() != id);
                info!("Handler Over");
                return;
            }

        }
        let _ = i_q;
        sleep(Duration::from_secs(2));
    }
}

