use std::{io, time::Duration};

use btleplug::{
    api::{Central, Manager as _, Peripheral as _, ScanFilter},
    platform::{Adapter, Manager},
};
use tokio::{signal, time};

use crate::{
    config::Config,
    light::Light,
    sacn_client::SacnClient,
    sacn_packet::{from_bytes, is_data_packet, SacnDmxPacket},
};

pub struct LightController {
    sacn_client: SacnClient,
    lights: Vec<Light>,
    bt_adapter: Adapter,
}

impl LightController {
    pub async fn new(universes: Vec<u16>) -> Self {
        let client = SacnClient::new(universes).await.unwrap();

        let manager = Manager::new().await.unwrap();

        // get the first bluetooth adapter
        let adapters = manager.adapters().await.unwrap();
        let bt_adapter = adapters.into_iter().nth(0).unwrap();

        Self {
            sacn_client: client,
            lights: vec![],
            bt_adapter,
        }
    }

    async fn handle_packet(&self, packet: &SacnDmxPacket) -> Result<(), btleplug::Error> {
        for light in &self.lights {
            if light.is_connected().await.unwrap() && light.get_universe() == packet.universe {
                let red = packet.dmx_data[light.get_address() as usize];
                let green = packet.dmx_data[light.get_address() as usize + 1];
                let blue = packet.dmx_data[light.get_address() as usize + 2];
                light.set_color_rgb(red, green, blue).await?;
            }
        }
        Ok(())
    }

    pub async fn listen(&mut self, config: Config) -> io::Result<()> {
        let mut buf = [0; 1024];

        // start scanning for devices
        self.bt_adapter
            .start_scan(ScanFilter::default())
            .await
            .unwrap();

        loop {
            let sigterm_future = signal::ctrl_c();

            self.find_lights(&config).await;
            self.delete_disconnected_lights().await;

            let _result = tokio::select! {
                _sigterm = sigterm_future => {
                    println!("Received SIGTERM, shutting down...");
                    break(Ok(()));
                }
                amt = self.sacn_client.get_socket().recv(&mut buf) => {
                    println!("Received data");
                    let packet = &buf[..amt.unwrap()];
                    if is_data_packet(packet) {
                        let sacn_packet = from_bytes(packet.to_vec()).unwrap();
                        if let Err(e) = self.handle_packet(&sacn_packet).await {
                            eprintln!("Error handling packet: {:?}", e);
                        }
                    }
                }
                _timeout = time::sleep(Duration::from_secs(1)) => {
                    println!("Timeout");
                }
            };
        }
    }

    async fn find_lights(&mut self, config: &Config) {
        for light_config in &config.lights {
            let light_ids: Vec<_> =
                futures::future::join_all(self.lights.iter().map(|light| light.get_id())).await;

            if !light_ids.iter().any(|id| *id == light_config.id) {
                if let Some(light) = Light::find_by_id(
                    &self.bt_adapter,
                    light_config.universe,
                    light_config.address,
                    light_config.id,
                )
                .await
                {
                    if let Err(e) = light.connect().await {
                        eprintln!("Failed to connect: {:?}", e);
                        continue;
                    }
                    if let Err(e) = light.discover_services().await {
                        eprintln!("Failed to discover services: {:?}", e);
                        continue;
                    }
                    self.lights.push(light);
                }
            }
        }
    }

    async fn delete_disconnected_lights(&mut self) {
        let mut i = 0;
        while i < self.lights.len() {
            if !self.lights[i].is_connected().await.unwrap() {
                println!("Removing light {:?}", self.lights[i].get_id().await);
                self.lights.remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub async fn disconnect(&self) {
        for light in &self.lights {
            light.disconnect().await.unwrap();
        }

        self.sacn_client.disconnect().await.unwrap();
    }

    pub async fn scan(&self) -> Result<(), btleplug::Error> {
        for p in self.bt_adapter.peripherals().await? {
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
