pub mod color;
pub mod config;
pub mod light;
pub mod light_controller;
pub mod sacn_client;
pub mod sacn_packet;
pub mod tests;

use std::env;
use std::error::Error;
use std::sync::Arc;

use btleplug::api::{Central, Manager as _, ScanFilter};
use btleplug::platform::Manager;
use config::Config;
use light_controller::LightController;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let manager = Manager::new().await.unwrap();
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).unwrap();

    if args.len() == 2 && args[1] == "scan" {
        LightController::scan(central).await.unwrap();
    } else {
        central.start_scan(ScanFilter::default()).await.unwrap();

        let config = Config::from_file("data/config.json").await.unwrap();
        let controller = LightController::new(&config).await;

        let controller_arc = Arc::new(tokio::sync::RwLock::new(controller));
        let read_lock = controller_arc.read().await;

        tokio::select! {
            _ = read_lock.listen() => {},
            _ = read_lock.find_light_loop() => {},
            _ = tokio::signal::ctrl_c() => {
                println!("Received Ctrl-C");
            }
        };

        println!("Exiting");

        controller_arc.read().await.disconnect().await;
    }

    Ok(())
}
