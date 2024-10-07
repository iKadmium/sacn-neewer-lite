use std::error::Error;

use btleplug::api::{BDAddr, Central, Manager as _, Peripheral as _, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral};
use lazy_static::lazy_static;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::color::Color;
use crate::dirty_details::DirtyDetails;
use crate::terminal_ui::TerminalUi;

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
    dirty_details: RwLock<DirtyDetails>,
}

impl Light {
    pub fn new(id: BDAddr, universe: u16, address: u16) -> Self {
        Self {
            id,
            universe,
            address,
            peripheral: RwLock::new(None),
            color: RwLock::new(Color::new(0, 0, 0)),
            dirty_details: RwLock::new(DirtyDetails::new()),
        }
    }

    fn get_checksum(send_value: &[u8]) -> u8 {
        let mut check_sum: u8 = 0;

        for value in send_value {
            check_sum = check_sum.wrapping_add(*value);
        }

        return check_sum;
    }

    async fn send_color(&self) -> Result<(), impl Error> {
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
            let maybe_cmd_char = chars.iter().find(|c| c.uuid == *write_uuid);

            match maybe_cmd_char {
                Some(cmd_char) => {
                    let details_read_lock = self.dirty_details.read().await;
                    if details_read_lock.is_dirty() {
                        drop(details_read_lock);
                        let send_result = peripheral
                            .write(cmd_char, &color_cmd, WriteType::WithoutResponse)
                            .await;
                        let mut details_write_lock = self.dirty_details.write().await;
                        details_write_lock.clean();
                        return send_result;
                    } else {
                        return Ok(());
                    }
                }
                None => {
                    return Err(btleplug::Error::NoSuchCharacteristic);
                }
            }
        } else {
            return Err(btleplug::Error::NoSuchCharacteristic);
        }
    }

    pub async fn set_color_rgb(&self, red: u8, green: u8, blue: u8) {
        let read_lock = self.color.read().await;
        if read_lock.red == red && read_lock.green == green && read_lock.blue == blue {
            return;
        }
        drop(read_lock);

        let mut lock = self.color.write().await;
        lock.red = red;
        lock.green = green;
        lock.blue = blue;
        self.dirty_details.write().await.dirty();
    }

    pub async fn connect(&self, peripheral: Peripheral, terminal: &RwLock<TerminalUi>) {
        terminal.write().await.set_light_status(
            self.id.to_string().as_str(),
            "Connecting",
            ratatui::style::Color::Yellow,
        );

        let mut peripheral_lock = self.peripheral.write().await;
        peripheral_lock.replace(peripheral);

        if let Err(e) = peripheral_lock.as_ref().unwrap().connect().await {
            peripheral_lock.take();
            self.set_error_status(terminal, "Failed to connect", e)
                .await;
            return;
        }
        if let Err(e) = peripheral_lock.as_ref().unwrap().discover_services().await {
            peripheral_lock.take();
            self.set_error_status(terminal, "Failed to discover services", e)
                .await;
            return;
        }

        drop(peripheral_lock);

        terminal.write().await.set_light_status(
            self.id.to_string().as_str(),
            "Connected",
            ratatui::style::Color::Green,
        );
    }

    pub async fn disconnect(&self, terminal: &RwLock<TerminalUi>) -> Result<(), btleplug::Error> {
        terminal.write().await.set_light_status(
            self.id.to_string().as_str(),
            "Disconnected",
            ratatui::style::Color::Red,
        );

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

    pub async fn find_loop(&self, terminal: &RwLock<TerminalUi>) {
        let manager = Manager::new().await.unwrap();
        let adapters = manager.adapters().await.unwrap();
        let central = adapters.into_iter().nth(0).unwrap();

        loop {
            self.search(&central, terminal).await;

            if let Err(e) = self.send_color().await {
                self.set_error_status(terminal, "Failed to send color", e)
                    .await;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }

    async fn search(&self, central: &Adapter, terminal: &RwLock<TerminalUi>) {
        if !self.is_connected().await.unwrap() {
            terminal.write().await.set_light_status(
                self.id.to_string().as_str(),
                "Searching",
                ratatui::style::Color::Yellow,
            );
        }

        while !self.is_connected().await.unwrap() {
            for p in central.peripherals().await.unwrap() {
                let props_result = p.properties().await;

                if let Ok(Some(props)) = props_result {
                    if props.address == self.id {
                        self.connect(p, terminal).await;
                    }
                } else {
                    self.set_error_status(
                        terminal,
                        "Failed to get properties",
                        props_result.err().unwrap(),
                    )
                    .await;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

    async fn set_error_status(
        &self,
        terminal: &RwLock<TerminalUi>,
        status: &str,
        error: impl Error,
    ) {
        let err = format!("{}: {:?}", status, error);
        terminal.write().await.set_light_status(
            self.id.to_string().as_str(),
            err.as_str(),
            ratatui::style::Color::Red,
        );
    }

    pub async fn get_id(&self) -> BDAddr {
        return self.id;
    }
}
