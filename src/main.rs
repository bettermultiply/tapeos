use log::info;
use tapeos::{
    config::{INTENT_INPUT_DESCRIPTION, MYSQL_DESCRIPTION}, 
    components::linkhub::internet::{seek::seek, wait::wait}, 
    tools::{idgen::init_id_generator, rserver::tape_server},
};
use std::{thread::sleep, time::Duration,};


#[tokio::main]
async fn main() {
    env_logger::init();
    init_id_generator();

    tokio::spawn(async {
        tape_server();
    });

    // loop_resource(100, "MySQL".to_string(), MY_SQL_DESCRIPTION.to_string(), 9001);
    tokio::spawn(async {
        let _ = wait("MySQL".to_string(), MYSQL_DESCRIPTION.to_string(), 8001).await;
    });
    // tokio::spawn(async {
    //     let _ = wait("MongoDB".to_string(), MONGO_DB_DESCRIPTION.to_string(), 8002).await;
    // });
    // tokio::spawn(async {
    //     let _ = wait("GooGle Drive".to_string(), GOO_GLE_DRIVE_DESCRIPTION.to_string(), 8003).await;
    // });
    // tokio::spawn(async move {
    //     let _ = wait("Intent input".to_string(), INTENT_INPUT_DESCRIPTION.to_string(), 8004).await;
    // });
    tokio::spawn(async move {
        let _ = seek().await;
    });
    sleep(Duration::from_secs(1000));
    
    info!("main: Try ended");
}

#[allow(unused)]
fn loop_resource(times: u16, name: String, desc: String, start_port: u16) {
    for i in 0..times {
        let n = name.clone();
        let d = desc.clone();
        tokio::spawn(async move {
            let s = format!("{}{}", n, i);
            let _ = wait(s, d, start_port+i).await;
        });
    }
}