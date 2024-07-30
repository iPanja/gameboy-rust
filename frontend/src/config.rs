use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use winit::event::VirtualKeyCode;

use crate::gameboy::joypad::JoypadInputKey;

#[derive(Clone, Serialize, Deserialize)]
pub struct GameBoyConfig {
    pub selected_rom: Option<PathBuf>,
    pub input_mapper: HashMap<JoypadInputKey, [Option<VirtualKeyCode>; 2]>,
    pub color_palette: [PixelColor; 4],
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
            color_palette: [
                pc_from_gray_value(255),
                pc_from_gray_value(170),
                pc_from_gray_value(85),
                pc_from_gray_value(0),
            ],
        }
    }
}

// Save and Load
impl GameBoyConfig {
    /// Attempt to save the config instance to gb_config.json
    pub fn save(&self) -> Result<(), String> {
        let file = File::open("gb_config.json");
        match file {
            Ok(file) => {
                let writer: BufWriter<File> = BufWriter::new(file);

                match serde_json::to_writer(writer, self) {
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
        let file = File::open("gb_config.json");
        match file {
            Ok(file) => {
                let reader: BufReader<File> = BufReader::new(file);

                let result: Result<Self, serde_json::Error> = serde_json::from_reader(reader);

                match result {
                    Ok(config) => Ok(config),
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

pub type PixelColor = [u8; 3];
fn pc_from_gray_value(value: u8) -> PixelColor {
    [value, value, value]
}
