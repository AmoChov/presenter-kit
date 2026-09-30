//! Portable settings management — loads settings.toml from next to the .exe

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "SpotlightSettings::default")]
    pub spotlight: SpotlightSettings,
    #[serde(default = "MagnifierSettings::default")]
    pub magnifier: MagnifierSettings,
    #[serde(default = "FreezeSettings::default")]
    pub freeze: FreezeSettings,
    #[serde(default = "TrailSettings::default")]
    pub trail: TrailSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpotlightSettings {
    #[serde(default = "default_radius")]
    pub radius: f32,
    #[serde(default = "default_feather")]
    pub feather: f32,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MagnifierSettings {
    #[serde(default = "default_zoom")]
    pub default_zoom: f32,
    #[serde(default = "default_chromatic")]
    pub chromatic_strength: f32,
    #[serde(default = "default_max_instances")]
    pub max_instances: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FreezeSettings {
    #[serde(default = "default_max_regions")]
    pub max_regions: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrailSettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_trail_length")]
    pub length: usize,
    #[serde(default = "default_trail_color")]
    pub color: String,
    #[serde(default = "default_trail_opacity")]
    pub opacity: f32,
    #[serde(default = "default_fade_ms")]
    pub fade_ms: u64,
}

fn default_radius() -> f32 { 200.0 }
fn default_feather() -> f32 { 60.0 }
fn default_opacity() -> f32 { 0.85 }
fn default_zoom() -> f32 { 2.5 }
fn default_chromatic() -> f32 { 0.5 }
fn default_max_instances() -> usize { 8 }
fn default_max_regions() -> usize { 8 }
fn default_true() -> bool { true }
fn default_trail_length() -> usize { 40 }
fn default_trail_color() -> String { "#FFFFFF".into() }
fn default_trail_opacity() -> f32 { 0.5 }
fn default_fade_ms() -> u64 { 600 }

impl Default for SpotlightSettings {
    fn default() -> Self {
        Self { radius: default_radius(), feather: default_feather(), opacity: default_opacity() }
    }
}

impl Default for MagnifierSettings {
    fn default() -> Self {
        Self { default_zoom: default_zoom(), chromatic_strength: default_chromatic(), max_instances: default_max_instances() }
    }
}

impl Default for FreezeSettings {
    fn default() -> Self {
        Self { max_regions: default_max_regions() }
    }
}

impl Default for TrailSettings {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            length: default_trail_length(),
            color: default_trail_color(),
            opacity: default_trail_opacity(),
            fade_ms: default_fade_ms(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            spotlight: SpotlightSettings::default(),
            magnifier: MagnifierSettings::default(),
            freeze: FreezeSettings::default(),
            trail: TrailSettings::default(),
        }
    }
}

impl Settings {
    /// Get the path to settings.toml (next to the executable)
    fn config_path() -> PathBuf {
        let exe_path = std::env::current_exe().unwrap_or_default();
        exe_path.parent().unwrap_or(&PathBuf::from(".")).join("settings.toml")
    }

    /// Load settings from settings.toml, or return defaults
    pub fn load() -> Self {
        let path = Self::config_path();
        match fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => {
                let settings = Self::default();
                // Auto-create default config
                let _ = settings.save();
                settings
            }
        }
    }

    /// Save current settings to settings.toml
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
