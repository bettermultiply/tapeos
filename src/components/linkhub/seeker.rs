// in this file, we will implement the seeker, which is used to seek the 
// higher level system when not connected.

// we will support multiple seeker, and each seeker will use different strategy to seek the higher level system.
// now we will implement in three strategies:
// 1. bluetooth
// 2. wifi
// 3. internet

use bluer::{Adapter, Device, DiscoveryFilter, DiscoveryTransport};
fn seek() -> bool {
    // TODO: implement the logic to seek the higher level system
    true
}

// TODO: seek means that there is a waiter waiting for connection. Where should we put the waiter?


fn seek_by_bluetooth() -> bool {
    tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(async {
        seek_bluetooth().await;
    });
    // TODO: implement the logic to seek the higher level system by bluetooth
    true
}

async fn seek_bluetooth() -> Result<(), Box<dyn std::error::Error>> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;

    let le_only = false;
    let br_edr_only = false;
    let filter = DiscoveryFilter {
        transport: if le_only {
            DiscoveryTransport::Le
        } else if br_edr_only {
            DiscoveryTransport::BrEdr
        } else {
            DiscoveryTransport::Auto
        },
        ..Default::default()
    };
    adapter.set_discovery_filter(filter).await?;
    Ok(())
}

fn seek_by_wifi() -> bool {
    // TODO: implement the logic to seek the higher level system by wifi
    true
}

fn seek_by_internet() -> bool {
    // TODO: implement the logic to seek the higher level system by internet
    true
}

