use std::collections::HashSet;

use btleplug::api::BDAddr;
use serde::de::{self, Deserializer};
use serde::Deserialize;

pub struct LightConfig {
    pub id: BDAddr,
    pub universe: u16,
    pub address: u16,
}

#[derive(Deserialize)]
pub struct Config {
    pub lights: Vec<LightConfig>,
}

impl Config {
    pub async fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = tokio::fs::read_to_string(path).await?;
        let lights: Vec<LightConfig> = serde_json::from_str(&data)?;
        let config = Config { lights };
        Ok(config)
    }

    pub fn get_universes(&self) -> Vec<u16> {
        let mut universes = HashSet::new();
        for light in &self.lights {
            universes.insert(light.universe);
        }
        universes.into_iter().collect()
    }
}
impl<'de> Deserialize<'de> for LightConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct LightConfigHelper {
            id: String,
            universe: u16,
            address: u16,
        }

        let helper = LightConfigHelper::deserialize(deserializer)?;
        let id = helper.id.parse::<BDAddr>().map_err(de::Error::custom)?;
        Ok(LightConfig {
            id,
            universe: helper.universe,
            address: helper.address,
        })
    }
}
