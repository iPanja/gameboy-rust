use sdl2::pixels::Color;
use std::{fmt, fs::OpenOptions, io::prelude::*};

use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};

use super::Bus;

const VRAM_SIZE: usize = (0x9FFF - 0x8000) + (0xFE9F - 0xFE00);

const TILE_MAP_SIZE: usize = 0x3FF;

pub struct PPU {
    // Memory Map (entirety of VRAM) 0x8000-0x9FFF and 0xFE00-0xFE9F
    raw_tile_vram: [u8; 384 * 16],
    pub tile_set: [Tile; 384], // Tile Set Blocks 0-3 - 0x8000-0x97FF
    bg_map_1: [u8; 0x3FF + 1], // Background Map 1 - 0x9800 - 0x9BFF    // Each entry (byte, u8) is a tile number (tile located in tile_set)
    bg_map_2: [u8; 0x3FF + 1], // Background Map 2 - 0x9C00 - 0x9FFF    // "                                                                "
    oam: [[u8; 4]; 40], // Object Attribute Memory - 0xFE00 - 0xFE9F // Each entry is 4 bytes, [u8; 4] - https://gbdev.io/pandocs/OAM.html#object-attribute-memory-oam
    // IO Registers 0xFF40-0xFF4B
    lcdc: u8,         // PPU control register - 0xFF40
    stat: u8,         // PPU status register - 0xFF41
    scy: u8,          // Vertical scroll register - 0xFF42
    scx: u8,          // Horizontal scroll register - 0xFF43
    pub ly: u8,       // Scanline register - 0xFF44
    bg_palette: u8,   // Background color palette - 0xFF47
    ob_palette_1: u8, // Object color palette 1 - 0xFF48
    ob_palette_2: u8, // Object color palette 2 - 0xFF49
    wy: u8,           // Window Y position - 0xFF4A
    wx: u8,           // Window X position - 0xFF4B
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    HBlank = 0,
    VBlank = 1,
    /// Accessing sprite memory, Sprite attributes RAM [0xfe00, 0xfe9f]
    /// can't be accessed
    Prelude = 2,
    /// Accessing sprite memory and video memory [0x8000, 0x9fff],
    /// both can't be accessed from CPU
    Active = 3,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)] // Debug
pub enum Pixel {
    White,     // 0b11
    DarkGray,  // 0b10
    LightGray, // 0b01
    Black,     // 0b00
}

impl std::convert::From<Pixel> for Color {
    fn from(pixel: Pixel) -> Color {
        match pixel {
            Pixel::White => Color::RGB(255, 255, 255),
            Pixel::LightGray => Color::RGB(255, 0, 0),
            Pixel::DarkGray => Color::RGB(0, 255, 0),
            Pixel::Black => Color::RGB(0, 0, 0),
        }
    }
}
impl std::convert::From<Pixel> for u8 {
    fn from(pixel: Pixel) -> u8 {
        match pixel {
            Pixel::White => 0b11,
            Pixel::LightGray => 0b10,
            Pixel::DarkGray => 0b01,
            Pixel::Black => 0b00,
        }
    }
}
impl std::convert::From<u8> for Pixel {
    fn from(byte: u8) -> Pixel {
        match byte {
            0b11 => Pixel::White,
            0b10 => Pixel::LightGray,
            0b01 => Pixel::DarkGray,
            _ => Pixel::Black,
        }
    }
}

impl fmt::Debug for Pixel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Pixel::White => write!(f, "W"),
            Pixel::LightGray => write!(f, " "),
            Pixel::DarkGray => write!(f, "+"),
            Pixel::Black => write!(f, "B"),
        }
    }
}

pub type Tile = [[Pixel; 8]; 8];
fn empty_tile() -> Tile {
    [[Pixel::Black; 8]; 8]
}

/* Registers
    0xFF40 - LCDC - PPU control register
    0xFF41 - STAT - PPU status register
    0xFF42 - SCY - Vertical scroll register
    0xFF43 - SCX - Horizontal scroll register
    0xFF44 - LY - Scanline register
*/

/* VRAM Data
    Tile Set Block 0 - $8000-87FF
    Tile Set Block 1 - $8800-8FFF
    Tile Set Block 2 - $9000-97FF

    Tile Map 0 - $9800-9BFF
    Tile Map 1 - $9C00-9FFF
*/

/* Addressing
    > $8000 method
        > $8000 as base pointer
        > unsigned addressing
            > Block 0: 0-127
            > Block 1: 128-255
            > Block 2: ...
    > $8800 method
        > $9000 as base pointer
        > signed addressing
            > Block 2: 0-127
            > tiles -128 to -1 are in Block 1
    > Objects always use $8000 addressing
    > BG, Window can use either mode (determined by LCDC bit 4)
*/

/* Tile
    A tile assigns a color ID to a pixel, ranging from 0-3 (2 bits).
        > 0b11 => white (W)
        > 0b10 => dark-gray (D)
        > 0b01 => light-gray (L)
        > 0b00 => black (B)
    For the Background or Window, the color ID => a pallete
    For Objects, color ID 0 = transparent, 1-3 => a pallete

    Each tile occupies 16 bytes, where each line is represented by 2 bytes.
        > Each pixel requires 2 bits
        > Each tile is 8x8 = 64 pixels
        > 64 pixels * 2bits/pixel = 128 bits or 16 bytes per tile
    Each row of a tile is 2 bytes:
        > The first byte specifies the LSB of the color ID of each pixel
        > The second byte specifies the MSB
        > In both bytes, bit 7 represents the leftmost pixel, and bit 0 the rightmost

    Example: $3C $7E
        > 00111100  01111110
        > 0         0       = 00 => 00 (COLOR ID)
        >  0         1      = 01 => 10 (COLOR ID, d2)
        > Entire row: 0 2 3 3 3 3 2 0
        > so color IDs range from 0-3 (requires only 2 bits)
*/

/* VRAM Tile Maps
   The Game Boy contains two 32x32 tile maps in VRAM:
       > $9800-9BFF
       > $9C00-9FFF
    Any of these maps can be used to display the Background or the Window.
    Each map contains the 1-byte indexes of the tiles to be displayed.
        > Tiles are obtained from the Tile Data Table
*/

/* The Three Layers
    Background:
        > Composed of a tilemap
        > Scrolling performed via ...
    Window:
        > A rectangle canvas/UI layer of sorts
        > No transparency
        > You can only control the position of the top-left pixel of the window
    Objects:
        > "Sprites"
        > Objects are made of 1 or 2 stacked tiles (8x8 or 8x16)
        > Can be displayed anywhere on screen
        > Sometimes many sprites are combined to create larger ones
        > Use the $8000 addressing
*/

/* Object Attribute Memory (OAM) - https://gbdev.io/pandocs/OAM.html
    > Up to 40 movable objects (sprites) can be displayed on the screen
    > Only up to 10 of these can occupy the same scanline
        > Regardless of if they are within the viewport
    > Taken from tile blocks 0 and 1, locatd at $8000-8FFF and have unsigned numbering
    > Object attributes are stored in the OAM at $FE00-FE9F
        > Byte 0 - Y Position
        > Byte 1 - X Position
        > Byte 2 - Tile Index
        > Byte 3 - Attributes/Flags
*/

impl PPU {
    pub fn new() -> Self {
        PPU {
            raw_tile_vram: [0; 384 * 16],
            tile_set: [empty_tile(); 384],
            bg_map_1: [0; 0x3FF + 1],
            bg_map_2: [0; 0x3FF + 1],
            oam: [[0; 4]; 40],
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            bg_palette: 0,
            ob_palette_1: 0,
            ob_palette_2: 0,
            wy: 0,
            wx: 0,
        }
    }

    pub fn read_byte(&self, index: usize, real_addr: usize) -> u8 {
        match real_addr {
            0xFF40 => self.lcdc,
            0xFF41 => self.stat,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF47 => self.bg_palette,
            0xFF48 => self.ob_palette_1,
            0xFF49 => self.ob_palette_2,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            0x8000..=0x97FF => self.read_tile_set_data_as_byte(real_addr - 0x8000),
            0x9800..=0x9BFF => self.bg_map_1[real_addr - 0x9800],
            0x9C00..=0x9FFF => self.bg_map_2[real_addr - 0x9C00],
            _ => {
                panic!("Unsupported VRAM access at byte: {:#X}", real_addr);
            }
        }
    }

    pub fn write_byte(&mut self, index: usize, real_addr: usize, value: u8) {
        //println!("offset of: {:#X}", index);

        // println!("writing to: {:#X} - {:#X}", real_addr, value);

        match real_addr {
            0xFF40 => {
                self.lcdc = value;
            }
            0xFF41 => {
                self.stat = value;
            }
            0xFF42 => {
                self.scy = value;
            }
            0xFF43 => {
                self.scx = value;
            }
            0xFF44 => {
                self.ly = value;
            }
            0xFF47 => {
                self.bg_palette = value;
            }
            0xFF48 => {
                self.ob_palette_1 = value;
            }
            0xFF49 => {
                self.ob_palette_2 = value;
            }
            0xFF4A => {
                self.wy = value;
            }
            0xFF4B => {
                self.wx = value;
            }
            0x8000..=0x97FF => {
                self.write_tile_set_data(real_addr - 0x8000, value);
                //log_vec(Vec::from(self.raw_tile_vram));
                //log_data(self.tile_set);
            }
            0x9800..=0x9BFF => {
                self.bg_map_1[real_addr - 0x9800] = value;
            }
            0x9C00..=0x9FFF => {
                self.bg_map_2[real_addr - 0x9C00] = value;
            }
            _ => {
                panic!("Unsupported VRAM access at byte: {:#X}", real_addr);
            }
        };

        // Not writing to tile set storage => no need to cache
        if index >= 0x1800 {
            return;
        }

        /* Cache
        let first_index = index & 0xFFFE;

        let byte1 = self.vram[first_index];
        let byte2 = self.vram[first_index + 1];

        let tile_index = index / 16;
        let row_index = (index % 16) / 2;
        for pixel_index in 0..8 {
            // To determine a pixel's value we must first find the corresponding bit that encodes
            // that pixels value:
            // 1111_1111
            // 0123 4567
            //
            // As you can see the bit that corresponds to the nth pixel is the bit in the nth
            // position *from the left*. Bits are normally indexed from the right.
            //
            // To find the first pixel (a.k.a pixel 0) we find the left most bit (a.k.a bit 7). For
            // the second pixel (a.k.a pixel 1) we first the second most left bit (a.k.a bit 6) and
            // so on.
            //
            // We then create a mask with a 1 at that position and 0s everywhere else.
            //
            // Bitwise ANDing this mask with our bytes will leave that particular bit with its
            // original value and every other bit with a 0.
            let mask = 1 << (7 - pixel_index);
            let lsb = byte1 & mask;
            let msb = byte2 & mask;

            // If the masked values are not 0 the masked bit must be 1. If they are 0, the masked
            // bit must be 0.
            //
            // Finally we can tell which of the four tile values the pixel is. For example, if the least
            // significant byte's bit is 1 and the most significant byte's bit is also 1, then we
            // have tile value `Three`.
            let value = match (lsb != 0, msb != 0) {
                (true, true) => Pixel::White,
                (false, true) => Pixel::DarkGray,
                (true, false) => Pixel::LightGray,
                (false, false) => Pixel::Black,
            };

            self.tile_set[tile_index][row_index][pixel_index] = value;
        }
        */
    }

    fn write_tile_set_data(&mut self, index: usize, byte: u8) {
        self.raw_tile_vram[index] = byte;
        // A tile is 2 bytes. 1 byte will only populate half the tile, or 4 pixels
        // Cache
        let first_index = index & 0xFFFE;

        let byte1 = self.raw_tile_vram[first_index];
        let byte2 = self.raw_tile_vram[first_index + 1];

        let tile_index = index / 16;
        let row_index = (index % 16) / 2;
        for pixel_index in 0..8 {
            let mask = 1 << (7 - pixel_index);
            let lsb = byte1 & mask;
            let msb = byte2 & mask;
            let value = match (lsb != 0, msb != 0) {
                (true, true) => Pixel::White,
                (false, true) => Pixel::DarkGray,
                (true, false) => Pixel::LightGray,
                (false, false) => Pixel::Black,
            };

            self.tile_set[tile_index][row_index][pixel_index] = value;
        }
    }

    fn read_tile_set_data_as_byte(&self, index: usize) -> u8 {
        /*
        let tile_no = index / 16;
        let pixel_no = (index % 16) / 2;
        let result = (self.tile_set[tile_no][pixel_no][pixel_no] as u8) << 6
            | (self.tile_set[tile_no][pixel_no][pixel_no + 1] as u8) << 4
            | (self.tile_set[tile_no][pixel_no][pixel_no + 2] as u8) << 2
            | self.tile_set[tile_no][pixel_no][pixel_no + 3] as u8;

        result
        */
        self.raw_tile_vram[index]
    }

    pub fn enable_lcd(&mut self, bus: &mut Bus) {
        self.lcdc |= 0b1010000;
        //bus.ram.ram[0xFF40] = 0b1010000 | self.vram[0xFF40];
    }

    fn get_address_mode(&self) -> u16 {
        match self.lcdc & 0x10 {
            0 => 0x8800,
            _ => 0x8000,
        }
    }

    fn get_background_map(&self) -> [Tile; 320] {
        //println!("SCX: {:#X}, SCY: {:#X}", self.scx, self.scy);

        let bg_map: &[u8; 0x3FF + 1] = match self.lcdc & 0x8 {
            0 => &self.bg_map_1,
            _ => &self.bg_map_2,
        };
        let mut tiles: [Tile; 320] = [empty_tile(); 320];
        // Background consists of 32x32 tiles, or 256x256 pixels
        // The viewport (Game Boy screen) can only show 20x18 tiles (160x144 pixels)
        let starting_offset = (self.scx + self.scy * 20) as usize;
        let start_index: usize = starting_offset;
        let end_index: usize = start_index + (20 * 16);

        for (i, tile_id) in bg_map[start_index..end_index].iter().enumerate() {
            //tiles[i] = self.get_tile(*tile_id as u16);
            tiles[i] = match self.get_address_mode() {
                0x8000 => self.tile_set[*tile_id as usize],
                _ => self.tile_set[192 + (*tile_id as i8) as usize],
            };
            //tiles[i] = self.tile_set[*tile_id as usize];
            //println!("tile data: {:?}", tiles[i]);
            //println!("tile {:#X}", *tile_id);
        }

        tiles
    }

    pub fn get_display(
        &self,
        bus: &Bus,
        scx: u8,
        scy: u8,
    ) -> [[Pixel; SCREEN_WIDTH]; SCREEN_HEIGHT] {
        //self.enable_lcd();

        let mut buffer = [[Pixel::White; SCREEN_WIDTH]; SCREEN_HEIGHT];

        for (tile_index, tile_data) in self.get_background_map().iter().enumerate() {
            let x = tile_index % 20;
            let y = tile_index / 20;
            for dy in 0..8 {
                for dx in 0..8 {
                    // (y + dy) * 160 + (x + dx) * 8
                    buffer[y + dy][x + dx] = tile_data[dy][dx];
                    //println!("writing tile data: {:?}", tile[dy][dx]);
                }
            }
        }

        buffer
    }

    pub fn log_tileset(&self) {
        for i in 0..32 * 32 {
            let tile_id = self.bg_map_1[i];
            let tile = match self.get_address_mode() {
                0x8000 => self.tile_set[tile_id as usize],
                _ => self.tile_set[192 + (tile_id as i8) as usize],
            };
            log(format!("{:?}", tile));
        }
        log(format!(
            "------------------------------------------------------"
        ));
        for i in 0..32 * 32 {
            let tile_id = self.bg_map_2[i];
            let tile = match self.get_address_mode() {
                0x8000 => self.tile_set[tile_id as usize],
                _ => self.tile_set[192 + (tile_id as i8) as usize],
            };
            log(format!("{:?}", tile));
        }
    }
}

fn log(s: String) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("gb-vram-log")
        .unwrap();

    if let Err(e) = writeln!(file, "{}", s) {
        eprintln!("Couldn't write to file: {}", e);
    }
}

fn log_vec(vec: Vec<u8>) {
    log(format!("{:?}", vec));
    log(format!("\n"));
}

pub fn log_data(tile_set: [Tile; 384]) {
    for tile in tile_set.iter() {
        print!("{:?}\n", tile);
    }
    println!("--------------------------------\n");
}
