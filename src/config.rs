use serde::{Serialize, Deserialize};
use directories::ProjectDirs;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

// Default configuration
pub const DEFAULT_SERVER: &str = "irc.slirc.net:6667";
pub const DEFAULT_CHANNEL: &str = "#straylight";

/// Represents a saved IRC network with connection settings
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Network {
    pub name: String,
    pub servers: Vec<String>, // e.g. ["irc.libera.chat:6667", "irc.libera.chat:6697"]
    pub nick: String,
    pub auto_connect: bool,
    pub favorite_channels: Vec<String>, // Auto-join channels
    #[serde(default)]
    pub nickserv_password: Option<String>, // TODO: Encrypt this
}

impl Default for Network {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            servers: vec![DEFAULT_SERVER.to_string()],
            nick: "slirc_user".to_string(),
            auto_connect: false,
            favorite_channels: vec![],
            nickserv_password: None,
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Settings {
    pub server: String,
    pub nick: String,
    pub default_channel: String,
    pub history: Vec<String>,
    pub theme: String,
    #[serde(default)]
    pub networks: Vec<Network>,
}

pub fn settings_path() -> Option<PathBuf> {
    if let Some(proj) = ProjectDirs::from("com", "sid3xyz", "slirc-client") {
        let dir = proj.config_dir();
        if let Err(e) = fs::create_dir_all(dir) {
            eprintln!("Failed to create config dir: {}", e);
            return None;
        }
        return Some(dir.join("settings.json"));
    }
    None
}

pub fn load_settings() -> Option<Settings> {
    let path = settings_path()?;
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_settings(settings: &Settings) -> std::io::Result<()> {
    if let Some(path) = settings_path() {
        let mut file = fs::File::create(path)?;
        let data = serde_json::to_string_pretty(settings).unwrap();
        file.write_all(data.as_bytes())?;
    }
    Ok(())
}
