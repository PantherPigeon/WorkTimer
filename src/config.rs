use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_focus_minutes")]
    pub focus_minutes: u32,
    #[serde(default = "default_break_minutes")]
    pub break_minutes: u32,
    #[serde(default = "default_true")]
    pub auto_start_break: bool,
    #[serde(default)]
    pub auto_start_focus: bool,
    #[serde(default = "default_true")]
    pub always_on_top: bool,
    #[serde(default = "default_opacity")]
    pub window_opacity: f32,
    #[serde(default = "default_true")]
    pub remember_window_position: bool,
    #[serde(default)]
    pub window_x: Option<f32>,
    #[serde(default)]
    pub window_y: Option<f32>,
    #[serde(default)]
    pub window_width: Option<f32>,
    #[serde(default)]
    pub window_height: Option<f32>,
    #[serde(default = "default_true")]
    pub sound_enabled: bool,
    #[serde(default)]
    pub compact_mode: bool,
}

fn default_focus_minutes() -> u32 {
    25
}
fn default_break_minutes() -> u32 {
    5
}
fn default_true() -> bool {
    true
}
fn default_opacity() -> f32 {
    0.95
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            focus_minutes: 25,
            break_minutes: 5,
            auto_start_break: true,
            auto_start_focus: false,
            always_on_top: true,
            window_opacity: 0.95,
            remember_window_position: true,
            window_x: None,
            window_y: None,
            window_width: None,
            window_height: None,
            sound_enabled: true,
            compact_mode: false,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "WorkTimer").map(|dirs| dirs.config_dir().join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path().context("Could not determine config directory")?;

        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;

        match toml::from_str::<Self>(&contents) {
            Ok(config) => Ok(config),
            Err(e) => {
                eprintln!("Config parse error: {e}. Using defaults.");
                // Back up corrupt config
                let backup = path.with_extension("toml.bak");
                let _ = fs::rename(&path, &backup);
                let config = Self::default();
                config.save()?;
                Ok(config)
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path().context("Could not determine config directory")?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        Ok(())
    }
}
