use std::error::Error;
use std::fmt;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::i18n::Language;
use crate::ui::NodeColorThemePreset;

const SETTINGS_DIR_NAME: &str = ".family-tree-creator";
const SETTINGS_FILE_NAME: &str = "settings.toml";

#[derive(Debug)]
pub enum AppSettingsError {
    CreateDirectory(String),
    Read(String),
    Write(String),
    Serialize(String),
    Deserialize(String),
}

impl fmt::Display for AppSettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppSettingsError::CreateDirectory(message) => {
                write!(f, "Failed to create settings directory: {message}")
            }
            AppSettingsError::Read(message) => write!(f, "Failed to read settings file: {message}"),
            AppSettingsError::Write(message) => write!(f, "Failed to write settings file: {message}"),
            AppSettingsError::Serialize(message) => {
                write!(f, "Failed to serialize settings: {message}")
            }
            AppSettingsError::Deserialize(message) => {
                write!(f, "Failed to parse settings file: {message}")
            }
        }
    }
}

impl Error for AppSettingsError {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AppSettings {
    pub language: Language,
    pub show_grid: bool,
    pub grid_size: f32,
    pub node_color_theme: NodeColorThemePreset,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: Language::Japanese,
            show_grid: true,
            grid_size: 50.0,
            node_color_theme: NodeColorThemePreset::Default,
        }
    }
}

impl AppSettings {
    pub fn load_from_default_path() -> Result<Option<Self>, AppSettingsError> {
        let path = default_settings_path();
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)
            .map_err(|error| AppSettingsError::Read(error.to_string()))?;

        let settings = toml::from_str::<AppSettings>(&content)
            .map_err(|error| AppSettingsError::Deserialize(error.to_string()))?;

        Ok(Some(settings))
    }

    pub fn save_to_default_path(&self) -> Result<(), AppSettingsError> {
        let dir = default_settings_dir();
        fs::create_dir_all(&dir)
            .map_err(|error| AppSettingsError::CreateDirectory(error.to_string()))?;

        let serialized = toml::to_string_pretty(self)
            .map_err(|error| AppSettingsError::Serialize(error.to_string()))?;

        fs::write(default_settings_path(), serialized)
            .map_err(|error| AppSettingsError::Write(error.to_string()))
    }
}

fn default_settings_dir() -> PathBuf {
    PathBuf::from(SETTINGS_DIR_NAME)
}

fn default_settings_path() -> PathBuf {
    default_settings_dir().join(SETTINGS_FILE_NAME)
}
