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
    pub fn save(&self) {
        let mut file = File::create("gb_config.json");
        if let Ok(mut file) = file {
            let serialized = serde_json::to_string(&self).unwrap();
            file.write_all(&serialized.as_bytes());
        }
    }

    pub fn load() -> Self {
        let mut file = File::open("gb_config.json");
        if let Ok(mut file) = file {
            let mut serialized = String::new();
            file.read_to_string(&mut serialized);

            serde_json::from_str(&serialized).unwrap()
        } else {
            println!("Could not find config save - using a default config");
            GameBoyConfig::default()
        }
    }
}
