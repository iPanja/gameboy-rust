use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
    result,
};

use bincode::deserialize_from;
use chrono::{DateTime, Utc};
use scan_dir::ScanDir;
use serde::Deserialize;

use crate::{
    gameboy::{cartridge::MBC, GameBoy},
    GameBoyState,
};

#[derive(serde::Serialize, serde::Deserialize)]
/// Represents a Game Boy Save (ROM and internal RAM) **file**.
/// This struct can create or override GameBoy instances to load/save
pub struct GameBoyGameSave {
    pub name: String,
    path_buf: PathBuf,
}

impl GameBoyGameSave {
    // Create a new instance
    // Does not save any data to disk!
    pub fn new(name: String, path_buf: &PathBuf) -> Self {
        GameBoyGameSave {
            name: name.to_owned(),
            path_buf: path_buf.clone(),
        }
    }

    // Create a new instance, name it: "<current game> - <timestamp>"
    // Does not save any data to disk!
    pub fn new_by_game(gameboy: &GameBoy, path_buf: &PathBuf) -> Self {
        let cartridge_title = if let Some(c_h) = &gameboy.cartridge_header {
            c_h.title.to_owned()
        } else {
            "Title not found!".to_string()
        };

        let time = Utc::now();

        let save_name = format!("{:?} - {:?}", cartridge_title, time);

        GameBoyGameSave {
            name: save_name,
            path_buf: path_buf.clone(),
        }
    }

    // Create a new save in the default location with the specified file name
    pub fn new_by_filename(save_name: &String) -> Self {
        let mut path = PathBuf::new();
        path.push("saves/");
        path.push(save_name.to_string() + ".gbr");

        GameBoyGameSave::new(save_name.to_owned(), &path)
    }
}

impl GameBoyGameSave {
    // Save the MBC (ROM and internal RAM) to disk
    pub fn save(&self, gameboy: &GameBoy) {
        let file = File::create(&self.path_buf);
        match file {
            Ok(file) => {
                let writer: BufWriter<File> = BufWriter::new(file);

                let mbc: &Box<dyn MBC> = &gameboy.bus.mbc;
                let _ = bincode::serialize_into(writer, &mbc);
            }
            Err(error) => println!("{:?}", error),
        }
    }

    // Read the saved MBC (ROM and internal RAM) from disk into the supplied GameBoy
    pub fn load(&self, gameboy: &mut GameBoy) {
        let file = File::open(&self.path_buf);
        if let Ok(file) = file {
            let reader: BufReader<File> = BufReader::new(file);

            let result: Result<Box<dyn MBC>, Box<bincode::ErrorKind>> =
                bincode::deserialize_from(reader);

            if let Ok(mbc) = result {
                println!("reading it in!");
                gameboy.bus.mbc = mbc;
            }
        }
    }

    // Scans the save directory (./saves/* */) and returns all save files found
    pub fn scan() -> Vec<GameBoyGameSave> {
        let mut saves = Vec::new();

        let files: Vec<_> = ScanDir::files()
            .read("./saves", |iter| {
                iter.filter(|&(_, ref name)| name.ends_with(".gbr"))
                    .map(|(entry, _)| entry.path())
                    .collect()
            })
            .unwrap();

        for file in files {
            saves.push(GameBoyGameSave::new(
                file.to_str().unwrap_or("unknown").to_owned(),
                &file,
            ));
        }

        saves
    }
}

impl Default for GameBoyGameSave {
    fn default() -> Self {
        GameBoyGameSave {
            name: "Default Save".to_owned(),
            path_buf: PathBuf::from("saves/gb_save.gbr"),
        }
    }
}

/*
impl GameBoyGameSave {
    // Save serialized gameboy state to file path
    pub fn save(gameboy: &GameBoy) {
        let file = File::create("gb_save.gbr");
        if let Ok(file) = file {
            let f: BufWriter<File> = BufWriter::new(file);
            if let Err(error) = bincode::serialize_into(f, &gameboy) {
                println!("error serializing gameboy save: {:?}", error);
            }
        }
    }

    // Create an instance of GameBoyGameSave from the file "gb_save.gbr"
    pub fn load(gameboy: &mut GameBoy) {
        let file = File::open("gb_save.gbr");
        if let Ok(file) = file {
            let f: BufReader<File> = BufReader::new(file);
            let result: Result<GameBoy, Box<bincode::ErrorKind>> = deserialize_from(f);

            if let Ok(gb) = result {
                println!("decoded save for");
                *gameboy = gb;
            } else {
                println!("error: {:?}", result.err());
            }
        } else {
            println!("Could not find config save - using a default config");
        }
    }
}
*/
