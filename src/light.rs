use std::error::Error;
use std::sync::Mutex;

use btleplug::api::{BDAddr, Central, Characteristic, Manager as _, Peripheral as _, WriteType};
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
    characteristic: Mutex<Option<Characteristic>>,
    color: Mutex<Color>,
    dirty_details: Mutex<DirtyDetails>,
}

impl Light {
    pub fn new(id: BDAddr, universe: u16, address: u16) -> Self {
        Self {
            id,
            universe,
            address,
            peripheral: RwLock::new(None),
            color: Mutex::new(Color::new(0, 0, 0)),
            characteristic: Mutex::new(None),
            dirty_details: Mutex::new(DirtyDetails::new()),
        }
    }

    fn get_checksum(send_value: &[u8]) -> u8 {
        let mut check_sum: u8 = 0;

        for value in send_value {
            check_sum = check_sum.wrapping_add(*value);
        }

        return check_sum;
    }

    async fn send_color(&self) -> Result<bool, impl Error> {
        let (hue, saturation, brightness) = { self.color.lock().unwrap().to_hsv() };

        let hue_lsb = (hue & 0xFF) as u8;
        let hue_msb = ((hue >> 8) & 0xFF) as u8;

        let mut color_cmd = vec![120, 134, 4, hue_lsb, hue_msb, saturation, brightness];
        color_cmd.push(Light::get_checksum(&color_cmd));

        let cmd_char_lock = self.characteristic.lock().unwrap();
        let peripheral_lock = self.peripheral.read().await;

        if peripheral_lock.is_some() && cmd_char_lock.is_some() {
            let peripheral = peripheral_lock.as_ref().unwrap();
            let cmd_char = cmd_char_lock.as_ref().unwrap();

            let dirty = self.dirty_details.lock().unwrap().is_dirty();
            if dirty {
                let send_result = peripheral
                    .write(cmd_char, &color_cmd, WriteType::WithoutResponse)
                    .await;

                self.dirty_details.lock().unwrap().clean();
                match send_result {
                    Ok(_) => {
                        return Ok(true);
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            } else {
                return Ok(false);
            }
        } else {
            return Err(btleplug::Error::NoSuchCharacteristic);
        }
    }

    pub async fn set_color_rgb(&self, red: u8, green: u8, blue: u8) {
        let current = { self.color.lock().unwrap().clone() };
        let new_color = Color::new(red, green, blue);
        if new_color.eq(&current) {
            return;
        }
        {
            let mut lock = self.color.lock().unwrap();
            *lock = new_color;
            self.dirty_details.lock().unwrap().dirty();
        }
    }

    pub async fn connect(&self, peripheral: Peripheral, terminal: &RwLock<TerminalUi>) {
        terminal.write().await.set_light_status(
            self.id.to_string().as_str(),
            "Connecting",
            ratatui::style::Color::Yellow,
        );

        {
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

            let chars: Vec<Characteristic> = peripheral_lock
                .as_ref()
                .unwrap()
                .characteristics()
                .iter()
                .cloned()
                .collect();

            let mut found = false;
            chars.iter().for_each(|c| {
                if c.uuid == *write_uuid {
                    let mut char_lock = self.characteristic.lock().unwrap();
                    char_lock.replace(c.clone());
                    found = true;
                }
            });

            if !found {
                peripheral_lock.take();
                self.set_error_status(
                    terminal,
                    "Failed to find characteristic",
                    std::io::Error::new(std::io::ErrorKind::Other, "aargh"),
                )
                .await;
                return;
            }
        }

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

            match self.send_color().await {
                Ok(sent) => {
                    if sent {
                        terminal
                            .write()
                            .await
                            .add_light_event(self.id.to_string().as_str());
                    }
                }
                Err(e) => {
                    self.set_error_status(terminal, "Failed to send color", e)
                        .await;
                }
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

    pub fn get_id(&self) -> BDAddr {
        return self.id;
    }
}
