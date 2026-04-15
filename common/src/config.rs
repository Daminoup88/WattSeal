use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "wattseal.toml";

/// Top-level configuration loaded from wattseal.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub api: ApiConfig,
}

/// HTTP API server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default)]
    pub auth: AuthConfig,
}

/// Authentication configuration for the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(default = "default_auth_type", rename = "type")]
    pub auth_type: String,
    #[serde(default)]
    pub token: String,
}

fn default_port() -> u16 {
    8080
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_auth_type() -> String {
    "none".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig::default(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: default_port(),
            host: default_host(),
            auth: AuthConfig::default(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            auth_type: default_auth_type(),
            token: String::new(),
        }
    }
}

impl AppConfig {
    /// Loads configuration from wattseal.toml, returning defaults if the file is missing.
    pub fn load() -> Self {
        let path = Path::new(CONFIG_FILE);
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!("Failed to parse {}: {}", CONFIG_FILE, e);
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read {}: {}", CONFIG_FILE, e);
                }
            }
        }
        Self::default()
    }
}
