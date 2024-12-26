// this file is used to wait for the resource and subsystem to connect.
// when the resource and subsystem are querying to connect, 
// the waiter will store the information of the resource or subsystem.
// and maintain the connection.

use crate::base::intent::Intent;
use crate::base::resource::Resource;

fn store_resource(resource: &Resource) {
    RESOURCES.lock().unwrap().push(Box::new(resource));
}

// need to notice that the subsystem is a special resource.
fn store_subsystem(subsystem: &Subsystem) {
    SUBSYSTEMS.lock().unwrap().push(Box::new(subsystem));
}

fn waiter() {
    // TODO: wait for the resource and subsystem to connect.


    let session = bluer::Session::new();
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    {
        
    }
    store_resource();
    
}
