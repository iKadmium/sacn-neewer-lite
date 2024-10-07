use std::time::Duration;

use btleplug::{
    api::{Central, Peripheral as _},
    platform::Adapter,
};
use ratatui::style::Color;
use tokio::{sync::RwLock, time};

use crate::{
    config::Config, light::Light, sacn_client::SacnClient, sacn_packet::SacnDmxPacket,
    terminal_ui::TerminalUi,
};

pub struct LightController {
    sacn_client: Option<SacnClient>,
    lights: Vec<Light>,
}

impl LightController {
    pub async fn new(config: &Config) -> Self {
        let mut lights = vec![];
        for light_config in config.lights.iter() {
            let light = Light::new(light_config.id, light_config.universe, light_config.address);
            lights.push(light);
        }
        let sacn_client = SacnClient::new(config.get_universes()).await.unwrap();

        Self {
            sacn_client: Some(sacn_client),
            lights,
        }
    }

    async fn handle_packet(&self, packet: &SacnDmxPacket) -> Result<(), btleplug::Error> {
        for light in self.lights.iter() {
            if light.get_universe() == packet.universe {
                let red = packet.dmx_data[light.get_address() as usize];
                let green = packet.dmx_data[light.get_address() as usize + 1];
                let blue = packet.dmx_data[light.get_address() as usize + 2];
                light.set_color_rgb(red, green, blue).await;
            }
        }
        Ok(())
    }

    pub async fn listen(&self, terminal: &RwLock<TerminalUi>) {
        loop {
            let _result = tokio::select! {
                packet = self.sacn_client.as_ref().unwrap().receive() => {
                    let mut lock = terminal.write().await;
                    lock.set_sacn_status("Received Sacn Packet", Color::Green);
                    drop(lock);

                    if let Err(e) = self.handle_packet(&packet.unwrap()).await {
                        eprintln!("Error handling packet: {:?}", e);
                    }
                }
                _timeout = time::sleep(Duration::from_secs(1)) => {
                    let mut lock = terminal.write().await;
                    lock.set_sacn_status("Timeout", Color::Red);
                    drop(lock);
                }
            };
        }
    }

    pub async fn find_light_loop(&self, terminal: &RwLock<TerminalUi>) {
        // start scanning for devices
        let futures: Vec<_> = self
            .lights
            .iter()
            .map(|light| light.find_loop(terminal))
            .collect();
        futures::future::join_all(futures).await;
    }

    pub async fn disconnect(&self, terminal: &RwLock<TerminalUi>) {
        for light in self.lights.iter() {
            light.disconnect(terminal).await.unwrap();
        }

        self.sacn_client
            .as_ref()
            .unwrap()
            .disconnect(terminal)
            .await
            .unwrap();
    }

    pub async fn scan(adapter: Adapter) -> Result<(), btleplug::Error> {
        for p in adapter.peripherals().await? {
            let props = p.properties().await?;
            if let Some(properties) = props {
                if properties.local_name.is_some() {
                    println!(
                        "{:?} -> {:?}",
                        properties.local_name.unwrap(),
                        properties.address
                    );
                }
            }
        }

        Ok(())
    }
}
