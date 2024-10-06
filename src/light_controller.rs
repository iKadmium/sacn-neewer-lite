use std::time::Duration;

use btleplug::{
    api::{Central, Peripheral as _},
    platform::Adapter,
};
use tokio::time;

use crate::{config::Config, light::Light, sacn_client::SacnClient, sacn_packet::SacnDmxPacket};

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

    pub async fn listen(&self) {
        loop {
            let _result = tokio::select! {
                packet = self.sacn_client.as_ref().unwrap().receive() => {
                    println!("Received data");

                    if let Err(e) = self.handle_packet(&packet.unwrap()).await {
                        eprintln!("Error handling packet: {:?}", e);
                    }
                }
                _timeout = time::sleep(Duration::from_secs(1)) => {
                    println!("Timeout");
                }
            };
        }
    }

    pub async fn find_light_loop(&self) {
        // start scanning for devices
        let futures: Vec<_> = self.lights.iter().map(|light| light.find_loop()).collect();
        futures::future::join_all(futures).await;
    }

    pub async fn disconnect(&self) {
        for light in self.lights.iter() {
            light.disconnect().await.unwrap();
        }

        self.sacn_client
            .as_ref()
            .unwrap()
            .disconnect()
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
