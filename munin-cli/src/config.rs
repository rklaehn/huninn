use std::{collections::BTreeMap, path::PathBuf, str::FromStr};

use anyhow::{anyhow, Result};
use iroh_net::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Config {
    pub secret_key: iroh_net::key::SecretKey,
    pub nodes: BTreeMap<String, NodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TomlConfig {
    secret_key: String,
    nodes: BTreeMap<String, String>,
}

impl TryFrom<TomlConfig> for Config {
    type Error = anyhow::Error;

    fn try_from(value: TomlConfig) -> Result<Self> {
        let secret_key = iroh_net::key::SecretKey::from_str(&value.secret_key)?;
        let nodes = value
            .nodes
            .into_iter()
            .map(|(name, id)| anyhow::Ok((name, NodeId::from_str(&id)?)))
            .collect::<Result<_, _>>()?;
        Ok(Config { secret_key, nodes })
    }
}

impl From<Config> for TomlConfig {
    fn from(value: Config) -> Self {
        TomlConfig {
            secret_key: value.secret_key.to_string(),
            nodes: value
                .nodes
                .into_iter()
                .map(|(name, id)| (name, id.to_string()))
                .collect(),
        }
    }
}

pub fn munin_data_root() -> anyhow::Result<PathBuf> {
    const MUNIN_DIR: &str = "munin-cli";
    let path = if let Some(val) = std::env::var_os("MUNIN_DATA_DIR") {
        PathBuf::from(val)
    } else {
        let path = dirs_next::data_dir().ok_or_else(|| {
            anyhow!("operating environment provides no directory for application data")
        })?;
        path.join(MUNIN_DIR)
    };
    let path = if !path.is_absolute() {
        std::env::current_dir()?.join(path)
    } else {
        path
    };
    Ok(path)
}

impl Config {
    pub fn save(&self) -> anyhow::Result<()> {
        let dir = munin_data_root()?;
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("config.toml");
        let data = toml::to_string_pretty(&TomlConfig::from(self.clone()))?;
        tracing::info!("Saving config to {}", path.display());
        std::fs::write(&path, data)?;
        Ok(())
    }

    pub fn get_or_create() -> anyhow::Result<Self> {
        let dir = munin_data_root()?;
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("config.toml");
        if path.exists() {
            tracing::info!("Loading config from {}", path.display());
            let data = std::fs::read_to_string(&path)?;
            let config: TomlConfig = toml::from_str(&data)?;
            let config = Self::try_from(config)?;
            Ok(config)
        } else {
            tracing::info!("Creating new config at {}", path.display());
            let config = Self {
                secret_key: iroh_net::key::SecretKey::generate(),
                nodes: BTreeMap::new(),
            };
            let data = toml::to_string_pretty(&TomlConfig::from(config.clone()))?;
            std::fs::write(&path, data)?;
            Ok(config)
        }
    }
}
