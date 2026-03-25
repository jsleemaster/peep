use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;

/// Top-level configuration loaded from `~/.config/packmen-tui/config.toml`.
/// All fields are optional — missing values fall back to the defaults below.
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub watcher: WatcherConfig,
    #[serde(default)]
    pub tui: TuiConfig,
    #[allow(dead_code)]
    #[serde(default)]
    pub theme: ThemeConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    /// HTTP hook server port.
    pub port: u16,
    /// HTTP hook server bind address.
    pub bind: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 3100,
            bind: "127.0.0.1".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct WatcherConfig {
    /// Enable the JSONL watcher.
    pub enabled: bool,
    /// Override the directory to watch (defaults to `~/.claude/projects`).
    pub watch_dir: Option<PathBuf>,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            watch_dir: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TuiConfig {
    /// UI tick rate in milliseconds.
    pub tick_rate: u64,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self { tick_rate: 100 }
    }
}

/// Reserved for future custom colour scheme support.
#[derive(Debug, Deserialize, Default)]
pub struct ThemeConfig {}

impl Config {
    /// Load config from `~/.config/packmen-tui/config.toml`.
    /// Returns `Ok(Config::default())` if the file does not exist.
    pub fn load() -> Result<Self> {
        let path = config_path();
        if !path.exists() {
            return Ok(Config::default());
        }

        let raw = std::fs::read_to_string(&path)?;
        let cfg: Config = toml::from_str(&raw)?;
        Ok(cfg)
    }

    /// Return the expected config file path without requiring it to exist.
    #[allow(dead_code)]
    pub fn path() -> PathBuf {
        config_path()
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("packmen-tui")
        .join("config.toml")
}
