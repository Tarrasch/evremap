use anyhow::Context;
pub use evdev_rs::enums::{EventCode, EventType, EV_KEY as KeyCode};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct MappingConfig {
    pub device_name: Option<String>,
    pub phys: Option<String>,
    pub mappings: Vec<Mapping>,
}

impl MappingConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let toml_data = std::fs::read_to_string(path)
            .context(format!("reading toml from {}", path.display()))?;
        let config_file: ConfigFile =
            toml::from_str(&toml_data).context(format!("parsing toml from {}", path.display()))?;
        let mut mappings = vec![];
        for remap in config_file.remap {
            mappings.push(remap.into());
        }
        Ok(Self {
            device_name: config_file.device_name,
            phys: config_file.phys,
            mappings,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Mapping {
    Remap {
        input: KeyCode,
        modifiers: HashSet<Modifier>,
        output: KeyCode,
    },
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum Modifier {
    Fn,
    LeftAlt,
    RightAlt,
    LeftMeta,
    RightMeta,
    LeftCtrl,
    RightCtrl,
    LeftShift,
    RightShift,
}

#[derive(Debug, Deserialize)]
#[serde(try_from = "String")]
struct KeyCodeWrapper {
    pub code: KeyCode,
}

// #[derive(Debug, Deserialize)]
// struct ModifierWrapper {
//     pub modifier: Modifier,
// }

impl Into<KeyCode> for KeyCodeWrapper {
    fn into(self) -> KeyCode {
        self.code
    }
}

// impl Into<Modifier> for ModifierWrapper {
//     fn into(self) -> Modifier {
//         self.modifier
//     }
// }

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid key `{0}`.  Use `evremap list-keys` to see possible keys.")]
    InvalidKey(String),
    #[error("Impossible: parsed KEY_XXX but not into an EV_KEY")]
    ImpossibleParseKey,
}

impl std::convert::TryFrom<String> for KeyCodeWrapper {
    type Error = ConfigError;
    fn try_from(s: String) -> Result<KeyCodeWrapper, Self::Error> {
        match EventCode::from_str(&EventType::EV_KEY, &s) {
            Some(code) => match code {
                EventCode::EV_KEY(code) => Ok(KeyCodeWrapper { code }),
                _ => Err(ConfigError::ImpossibleParseKey),
            },
            None => Err(ConfigError::InvalidKey(s)),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RemapConfig {
    input: KeyCodeWrapper,
    #[serde(default)]
    modifiers: Vec<Modifier>,
    output: KeyCodeWrapper,
}

impl Into<Mapping> for RemapConfig {
    fn into(self) -> Mapping {
        Mapping::Remap {
            input: Into::into(self.input),
            modifiers: self.modifiers.into_iter().collect(),
            output: Into::into(self.output),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    device_name: Option<String>,

    #[serde(default)]
    phys: Option<String>,

    #[serde(default)]
    remap: Vec<RemapConfig>,
}
