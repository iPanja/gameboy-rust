use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use winit::event::VirtualKeyCode;

use crate::gameboy::joypad::JoypadInputKey;

#[derive(Clone, Serialize, Deserialize)]
pub struct GameBoyConfig {
    pub selected_rom: Option<PathBuf>,
    pub input_mapper: HashMap<JoypadInputKey, [Option<VirtualKeyCode>; 2]>,
}

impl Default for GameBoyConfig {
    fn default() -> Self {
        GameBoyConfig {
            selected_rom: None,
            input_mapper: HashMap::from([
                (JoypadInputKey::Up, [Some(VirtualKeyCode::W), None]),
                (JoypadInputKey::Left, [Some(VirtualKeyCode::A), None]),
                (JoypadInputKey::Down, [Some(VirtualKeyCode::S), None]),
                (JoypadInputKey::Right, [Some(VirtualKeyCode::D), None]),
                (JoypadInputKey::A, [Some(VirtualKeyCode::Right), None]),
                (JoypadInputKey::B, [Some(VirtualKeyCode::Left), None]),
                (JoypadInputKey::Start, [Some(VirtualKeyCode::Q), None]),
                (JoypadInputKey::Select, [Some(VirtualKeyCode::E), None]),
            ]),
        }
    }
}

// Save and Load
impl GameBoyConfig {
    /// Attempt to save the config instance to gb_config.json
    pub fn save(&self) -> Result<(), String> {
        let mut file = File::create("gb_config.json");

        match file.as_mut() {
            Ok(file) => {
                let serialized = serde_json::to_string(&self).unwrap();
                match file.write_all(&serialized.as_bytes()) {
                    Ok(_) => Ok(()),
                    Err(error) => Err(error.to_string()),
                }
            }
            Err(error) => Err(error.to_string()),
        }
    }

    /// Attempt to load config from gb_config.json.
    /// Return instance if found, otherwise the error as a string
    fn _load() -> Result<Self, String> {
        match File::open("gb_config.json").as_mut() {
            Ok(file) => {
                let mut serialized = String::new();
                match file.read_to_string(&mut serialized) {
                    Ok(_) => serde_json::from_str(&serialized).unwrap(),
                    Err(error) => Err(error.to_string()),
                }
            }
            Err(error) => Err(error.to_string()),
        }
    }

    /// Return an instance of the saved config at gb_config.json if found, otherwise GameBoyConfig::default()
    pub fn load() -> Self {
        match GameBoyConfig::_load() {
            Ok(config) => config,
            Err(error) => {
                println!(
                    "Failed to grab GameBoy config file, using default layout!\n\t{:?}",
                    error
                );
                GameBoyConfig::default()
            }
        }
    }
}
