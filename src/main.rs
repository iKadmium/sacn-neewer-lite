pub mod color;
pub mod config;
pub mod light;
pub mod light_controller;
pub mod sacn_client;
pub mod sacn_packet;
pub mod terminal_ui;
pub mod tests;

use std::env;
use std::error::Error;
use std::sync::Arc;

use btleplug::api::{Central, Manager as _, ScanFilter};
use btleplug::platform::Manager;
use config::Config;
use light_controller::LightController;
use terminal_ui::TerminalUi;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let manager = Manager::new().await.unwrap();
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).unwrap();

    if args.len() == 2 && args[1] == "scan" {
        LightController::scan(central).await.unwrap();
    } else {
        let termui = TerminalUi::new();
        let terminal_mutex = RwLock::new(termui);

        terminal_mutex
            .write()
            .await
            .set_app_status("Starting", ratatui::style::Color::Reset);

        central.start_scan(ScanFilter::default()).await.unwrap();

        let config = Config::from_file("data/config.json").await.unwrap();
        let controller = LightController::new(&config).await;

        let controller_arc = Arc::new(tokio::sync::RwLock::new(controller));
        let controller_read_lock = controller_arc.read().await;

        terminal_mutex
            .write()
            .await
            .set_app_status("Running", ratatui::style::Color::Green);

        tokio::select! {
            _ = controller_read_lock.listen(&terminal_mutex) => {},
            _ = controller_read_lock.find_light_loop(&terminal_mutex) => {},
            _ = TerminalUi::ui_loop(&terminal_mutex) => {},
        };

        terminal_mutex
            .write()
            .await
            .set_app_status("Exiting", ratatui::style::Color::Reset);

        controller_read_lock.disconnect(&terminal_mutex).await;
        let mut terminal_lock = terminal_mutex.write().await;
        let _result = terminal_lock.restore_terminal();
    }

    Ok(())
}
