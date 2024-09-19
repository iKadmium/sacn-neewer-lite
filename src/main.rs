pub mod color;
pub mod config;
pub mod light;
pub mod light_controller;
pub mod sacn_client;
pub mod sacn_packet;
pub mod tests;

use std::env;
use std::error::Error;

use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 && args[1] == "scan" {
        let controller = light_controller::LightController::new(vec![]).await;
        controller.scan().await?;
    } else {
        let config = Config::from_file("data/config.json").await.unwrap();
        let mut controller = light_controller::LightController::new(config.get_universes()).await;
        controller.listen(config).await?;
        controller.disconnect().await;
    }

    Ok(())
}
