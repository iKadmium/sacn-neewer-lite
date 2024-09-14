use btleplug::api::{Central, Peripheral as _, WriteType};
use btleplug::platform::{Adapter, Peripheral};
use uuid::Uuid;

const UUID_STR: &str = "69400002-B5A3-F393-E0A9-E50E24DCCA99";

pub struct Light {
    peripheral: Peripheral,
    uuid: Uuid,
    universe: u16,
    address: u8,
}

impl Light {
    pub fn new(peripheral: Peripheral, universe: u16, address: u8) -> Self {
        let uuid = Uuid::parse_str(UUID_STR).unwrap();
        Self {
            peripheral,
            uuid,
            universe,
            address,
        }
    }

    fn add_checksum(send_value: &[u8]) -> Vec<u8> {
        let mut return_array = Vec::new();
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
        let hue_byte1 = ((hue & 65280) >> 8) as u8;
        let hue_byte2 = (hue >> 8) as u8;

        let color_cmd = vec![120, 134, 4, hue_byte1, hue_byte2, saturation, brightness];
        let send_value = Self::add_checksum(&color_cmd);

        println!("Sending {:?}", send_value);

        // find the characteristic we want
        let chars = self.peripheral.characteristics();
        let cmd_char = chars.iter().find(|c| c.uuid == self.uuid).unwrap();
        return self
            .peripheral
            .write(&cmd_char, &send_value, WriteType::WithoutResponse)
            .await;
    }

    pub async fn set_color_rgb(&self, red: u8, green: u8, blue: u8) -> Result<(), btleplug::Error> {
        let (hue, saturation, intensity) = rgb_to_hsv(red, green, blue);
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

    pub fn get_address(&self) -> u8 {
        return self.address;
    }

    pub fn get_universe(&self) -> u16 {
        return self.universe;
    }
}

// name is "NEEWER-TL21C"
pub async fn find_lights_by_name(
    central: &Adapter,
    names: Vec<&str>,
    universe: u16,
    first_address: u8,
) -> Vec<Light> {
    let mut lights: Vec<Light> = vec![];
    let mut address = first_address;
    for p in central.peripherals().await.unwrap() {
        let props = p.properties().await.unwrap().unwrap();
        println!("Found device {:?}", props.local_name);
        if p.properties()
            .await
            .unwrap()
            .unwrap()
            .local_name
            .iter()
            .any(|name| names.contains(&name.as_str()))
        {
            let light = Light::new(p, universe, address);
            address += 3;
            lights.push(light);
        }
    }
    return lights;
}

pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (u16, u8, u8) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let hue = if hue < 0.0 { hue + 360.0 } else { hue };

    let saturation = if max == 0.0 { 0.0 } else { delta / max };

    let value = max;

    (
        hue.round() as u16,
        (saturation * 100.0).round() as u8,
        (value * 100.0).round() as u8,
    )
}
