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

fn munin_data_root() -> anyhow::Result<PathBuf> {
    const MUNIN_DIR: &str = "munin-daemon";
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
    pub fn initial_allowed_nodes() -> anyhow::Result<BTreeSet<NodeId>> {
        match std::env::var("MUNIN_ALLOWED_NODES") {
            Ok(val) => val
                .split(',')
                .map(|s| Ok(NodeId::from_str(s)?))
                .collect::<Result<_, _>>(),
            Err(_) => Ok(BTreeSet::new()),
        }
    }

    pub fn default_path() -> anyhow::Result<PathBuf> {
        let dir = munin_data_root()?;
        Ok(dir.join("config.toml"))
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
                name: "munin-daemon".to_string(),
                secret_key: iroh_net::key::SecretKey::generate(),
                allowed_nodes: Config::initial_allowed_nodes()?,
            };
            let data = toml::to_string_pretty(&TomlConfig::from(config.clone()))?;
            std::fs::write(&path, data)?;
            Ok(config)
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let dir = munin_data_root()?;
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("config.toml");
        let data = toml::to_string_pretty(&TomlConfig::from(self.clone()))?;
        tracing::info!("Saving config to {}", path.display());
        std::fs::write(&path, data)?;
        Ok(())
    }
}
