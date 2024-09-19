use btleplug::api::{BDAddr, Central, Peripheral as _, WriteType};
use btleplug::platform::{Adapter, Peripheral};
use lazy_static::lazy_static;
use uuid::Uuid;

use crate::color;

const UUID_STR: &str = "69400002-B5A3-F393-E0A9-E50E24DCCA99";
lazy_static! {
    static ref write_uuid: Uuid = Uuid::parse_str(UUID_STR).unwrap();
}

pub struct Light {
    peripheral: Peripheral,
    uuid: Uuid,
    universe: u16,
    address: u16,
}

impl Light {
    pub fn new(peripheral: Peripheral, universe: u16, address: u16) -> Self {
        let uuid = Uuid::parse_str(UUID_STR).unwrap();
        Self {
            peripheral,
            uuid,
            universe,
            address,
        }
    }

    fn add_checksum(send_value: &[u8]) -> Vec<u8> {
        let mut return_array = vec![];
        let mut check_sum: u8 = 0;

        for value in send_value {
            check_sum = check_sum.wrapping_add(*value) as u8;
            return_array.push(*value);
        }

        return_array.push(check_sum);
        return return_array;
    }

    pub async fn set_color_hsi(
        &self,
        hue: u16,
        saturation: u8,
        brightness: u8,
    ) -> Result<(), btleplug::Error> {
        let hue_lsb = (hue & 0xFF) as u8;
        let hue_msb = ((hue >> 8) & 0xFF) as u8;

        let color_cmd = vec![120, 134, 4, hue_lsb, hue_msb, saturation, brightness];
        let send_value = Self::add_checksum(&color_cmd);

        println!("Sending {:?}", send_value);

        // find the characteristic we want
        let chars = self.peripheral.characteristics();
        let cmd_char = chars.iter().find(|c| c.uuid == self.uuid);
        if let Some(cmd_char) = cmd_char {
            return self
                .peripheral
                .write(cmd_char, &send_value, WriteType::WithoutResponse)
                .await;
        } else {
            return Err(btleplug::Error::Other(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not find characteristic",
            ))));
        }
    }

    pub async fn set_color_rgb(&self, red: u8, green: u8, blue: u8) -> Result<(), btleplug::Error> {
        let (hue, saturation, intensity) = color::rgb_to_hsv(red, green, blue);
        return self.set_color_hsi(hue, saturation, intensity).await;
    }

    pub async fn connect(&self) -> Result<(), btleplug::Error> {
        println!("Connecting to {:?}", self.get_name().await);
        return self.peripheral.connect().await;
    }

    pub async fn disconnect(&self) -> Result<(), btleplug::Error> {
        println!("Disconnecting from {:?}", self.get_name().await);
        return self.peripheral.disconnect().await;
    }

    pub async fn discover_services(&self) -> Result<(), btleplug::Error> {
        return self.peripheral.discover_services().await;
    }

    pub async fn get_name(&self) -> Option<String> {
        return self
            .peripheral
            .properties()
            .await
            .unwrap()
            .unwrap()
            .local_name;
    }

    pub fn get_address(&self) -> u16 {
        return self.address;
    }

    pub fn get_universe(&self) -> u16 {
        return self.universe;
    }

    pub async fn is_connected(&self) -> Result<bool, btleplug::Error> {
        return self.peripheral.is_connected().await;
    }

    // name is "NEEWER-TL21C"
    pub async fn find_by_name(
        central: &Adapter,
        names: Vec<&str>,
        universe: u16,
        first_address: u16,
    ) -> Vec<Light> {
        let mut lights: Vec<Light> = vec![];
        let mut address = first_address;
        for p in central.peripherals().await.unwrap() {
            let props = p.properties().await.unwrap().unwrap();
            if p.properties()
                .await
                .unwrap()
                .unwrap()
                .local_name
                .iter()
                .any(|name| names.contains(&name.as_str()))
            {
                println!("Found device {:?}", props.local_name);
                let light = Light::new(p, universe, address);
                address += 3;
                lights.push(light);
            }
        }
        return lights;
    }

    pub async fn find_by_id(
        central: &Adapter,
        universe: u16,
        dmx_address: u16,
        bt_address: BDAddr,
    ) -> Option<Light> {
        let address = dmx_address;
        println!("Looking for {:?}", bt_address);
        for p in central.peripherals().await.unwrap() {
            let props = p.properties().await.unwrap().unwrap();

            if p.properties().await.unwrap().unwrap().address == bt_address {
                println!("Found device {:?}", props.local_name);
                let light = Light::new(p, universe, address);
                return Some(light);
            }
        }
        return None;
    }

    pub async fn get_id(&self) -> BDAddr {
        return self.peripheral.properties().await.unwrap().unwrap().address;
    }
}
