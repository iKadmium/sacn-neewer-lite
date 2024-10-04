use btleplug::api::{BDAddr, Central, Manager as _, Peripheral as _, WriteType};
use btleplug::platform::{Manager, Peripheral};
use lazy_static::lazy_static;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::color::Color;

const UUID_STR: &str = "69400002-B5A3-F393-E0A9-E50E24DCCA99";
lazy_static! {
    static ref write_uuid: Uuid = Uuid::parse_str(UUID_STR).unwrap();
}

pub struct Light {
    id: BDAddr,
    universe: u16,
    address: u16,
    peripheral: RwLock<Option<Peripheral>>,
    color: RwLock<Color>,
}

impl Light {
    pub fn new(id: BDAddr, universe: u16, address: u16) -> Self {
        Self {
            id,
            universe,
            address,
            peripheral: RwLock::new(None),
            color: RwLock::new(Color::new(0, 0, 0)),
        }
    }

    fn get_checksum(send_value: &[u8]) -> u8 {
        let mut check_sum: u8 = 0;

        for value in send_value {
            check_sum = check_sum.wrapping_add(*value) as u8;
        }

        return check_sum;
    }

    async fn send_color(&self) -> Result<(), btleplug::Error> {
        let color = self.color.read().await;
        let (hue, saturation, brightness) = color.to_hsv();
        drop(color);

        let hue_lsb = (hue & 0xFF) as u8;
        let hue_msb = ((hue >> 8) & 0xFF) as u8;

        let mut color_cmd = vec![120, 134, 4, hue_lsb, hue_msb, saturation, brightness];
        color_cmd.push(Light::get_checksum(&color_cmd));

        let lock = self.peripheral.read().await;

        // find the characteristic we want
        if lock.is_some() {
            let peripheral = lock.as_ref().unwrap();
            let chars = peripheral.characteristics();
            let cmd_char = chars.iter().find(|c| c.uuid == *write_uuid).unwrap();

            return peripheral
                .write(cmd_char, &color_cmd, WriteType::WithoutResponse)
                .await;
        } else {
            return Err(btleplug::Error::Other(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not find characteristic",
            ))));
        }
    }

    pub async fn set_color_rgb(&self, red: u8, green: u8, blue: u8) {
        let mut lock = self.color.write().await;
        lock.red = red;
        lock.green = green;
        lock.blue = blue;
    }

    pub async fn connect(&self, peripheral: Peripheral) -> Result<(), btleplug::Error> {
        println!("Connecting to {:?}", self.get_name().await);
        let mut lock = self.peripheral.write().await;
        lock.replace(peripheral);

        lock.as_ref().unwrap().connect().await?;
        lock.as_ref().unwrap().discover_services().await?;
        println!("Connected");
        drop(lock);

        return Ok(());
    }

    pub async fn disconnect(&self) -> Result<(), btleplug::Error> {
        println!("Disconnecting from {:?}", self.get_name().await);
        let lock = self.peripheral.read().await;
        if lock.as_ref().is_some() {
            lock.as_ref().unwrap().disconnect().await?;
        }
        return Ok(());
    }

    pub async fn get_name(&self) -> Option<String> {
        let lock = self.peripheral.read().await;
        match lock.as_ref() {
            Some(p) => {
                let props = p.properties().await.unwrap().unwrap();
                return props.local_name;
            }
            None => return None,
        }
    }

    pub fn get_address(&self) -> u16 {
        return self.address;
    }

    pub fn get_universe(&self) -> u16 {
        return self.universe;
    }

    pub async fn is_connected(&self) -> Result<bool, btleplug::Error> {
        let lock = self.peripheral.read().await;
        match lock.as_ref() {
            Some(ref p) => return p.is_connected().await,
            None => return Ok(false),
        }
    }

    pub async fn find_loop(&self) {
        let manager = Manager::new().await.unwrap();
        let adapters = manager.adapters().await.unwrap();
        let central = adapters.into_iter().nth(0).unwrap();

        loop {
            if !self.is_connected().await.unwrap() {
                println!("Looking for {:?}", self.id);
            }
            while !self.is_connected().await.unwrap() {
                for p in central.peripherals().await.unwrap() {
                    let props_result = p.properties().await;

                    if let Ok(Some(props)) = props_result {
                        if props.address == self.id {
                            println!("Found device {:?}", props.local_name);
                            self.connect(p).await.unwrap();
                        }
                    } else {
                        println!("Failed to get properties for peripheral");
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
            let send_result = self.send_color().await;
            if send_result.is_err() {
                println!("Failed to send color: {:?}", send_result.err().unwrap());
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }

    pub async fn get_id(&self) -> BDAddr {
        return self.id;
    }
}
