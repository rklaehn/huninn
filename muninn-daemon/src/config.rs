use std::{collections::BTreeSet, path::PathBuf, str::FromStr};

use anyhow::anyhow;
use iroh_net::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Config {
    pub name: String,
    pub secret_key: iroh_net::key::SecretKey,
    pub allowed_nodes: BTreeSet<NodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TomlConfig {
    name: String,
    secret_key: String,
    allowed_nodes: Vec<String>,
}

impl TryFrom<TomlConfig> for Config {
    type Error = anyhow::Error;

    fn try_from(value: TomlConfig) -> Result<Self, Self::Error> {
        let secret_key = iroh_net::key::SecretKey::from_str(&value.secret_key)?;
        let allowed_nodes = value
            .allowed_nodes
            .into_iter()
            .map(|s| NodeId::from_str(&s))
            .collect::<Result<_, _>>()?;
        Ok(Config {
            name: value.name,
            secret_key,
            allowed_nodes,
        })
    }
}

impl From<Config> for TomlConfig {
    fn from(value: Config) -> Self {
        TomlConfig {
            name: value.name,
            secret_key: value.secret_key.to_string(),
            allowed_nodes: value
                .allowed_nodes
                .into_iter()
                .map(|id| id.to_string())
                .collect(),
        }
    }
}

pub fn muninn_data_root() -> anyhow::Result<PathBuf> {
    const MUNINN_DIR: &str = "muninn-daemon";
    let path = if let Some(val) = std::env::var_os("MUNINN_DATA_DIR") {
        PathBuf::from(val)
    } else {
        let path = dirs_next::data_dir().ok_or_else(|| {
            anyhow!("operating environment provides no directory for application data")
        })?;
        path.join(MUNINN_DIR)
    };
    let path = if !path.is_absolute() {
        std::env::current_dir()?.join(path)
    } else {
        path
    };
    Ok(path)
}

impl Config {
    pub fn get_or_create() -> anyhow::Result<Self> {
        let dir = muninn_data_root()?;
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
                name: "muninn-daemon".to_string(),
                secret_key: iroh_net::key::SecretKey::generate(),
                allowed_nodes: BTreeSet::new(),
            };
            let data = toml::to_string_pretty(&TomlConfig::from(config.clone()))?;
            std::fs::write(&path, &data)?;
            Ok(config)
        }
    }
}
