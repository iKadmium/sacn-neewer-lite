pub mod light;
pub mod sacn_client;
pub mod sacn_packet;
pub mod tests;

use btleplug::api::{Central, Manager as _, ScanFilter};
use btleplug::platform::Manager;
use light::find_lights_by_name;
use std::error::Error;
use std::time::Duration;
use std::vec;
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await.unwrap();

    // get the first bluetooth adapter
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).unwrap();

    // start scanning for devices
    central.start_scan(ScanFilter::default()).await?;
    // instead of waiting, you can use central.events() to get a stream which will
    // notify you of new devices, for an example of that see examples/event_driven_discovery.rs
    time::sleep(Duration::from_secs(2)).await;

    // find the device we're interested in
    //let lights = find_lights_by_name(&central, vec!["NEEWER-TL21C"], 1, 1).await;
    let lights = find_lights_by_name(&central, vec!["NEEWER-TL21C"], 1, 1).await;

    for light in &lights {
        // connect to the device
        light.connect().await?;

        // discover services and characteristics
        light.discover_services().await?;
    }

    let sacn_client = sacn_client::SacnClient::new(1, lights).unwrap();
    sacn_client.listen().await?;
    sacn_client.disconnect().await?;

    Ok(())
}
