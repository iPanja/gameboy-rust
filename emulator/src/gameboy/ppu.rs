use ggez::graphics::Color;
//use sdl2::pixels::Color;
use std::{fmt, fs::OpenOptions, io::prelude::*};

use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};

use super::Bus;

const VRAM_SIZE: usize = (0x9FFF - 0x8000) + (0xFE9F - 0xFE00);

const TILE_MAP_SIZE: usize = 0x3FF;

pub struct PPU {
    // Memory Map (entirety of VRAM) 0x8000-0x9FFF and 0xFE00-0xFE9F
    raw_tile_vram: [u8; 384 * 16],
    pub tile_set: [Tile; 384],   // Tile Set Blocks 0-3 - 0x8000-0x97FF
    tile_map_1: [u8; 0x3FF + 1], // Background Map 1 - 0x9800 - 0x9BFF    // Each entry (byte, u8) is a tile number (tile located in tile_set)
    tile_map_2: [u8; 0x3FF + 1], // Background Map 2 - 0x9C00 - 0x9FFF    // "                                                                "
    raw_oam: [u8; 0xA0], // Object Attribute Memory - 0xFE00 - 0xFE9F // Each entry is 4 bytes, [u8; 4] - https://gbdev.io/pandocs/OAM.html#object-attribute-memory-oam
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
    // Internal data structures
    background_fifo: [Pixel; 8],
    sprite_fifo: [Pixel; 8],
    bg_x_pc: usize,
    window_lc: usize,
}

// https://github.com/Hacktix/GBEDG/blob/master/ppu/index.md#the-concept-of-ppu-modes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    HBlank = 0, // Takes place after the current scanline has completed, pads the duration of the line scan to 456 T-Cycles - effectively a pause for the PPU.
    VBlank = 1, // The psuedo lines (144-153), taking place after the entire screen has been scanned
    OAM = 2, // Entered at the start of every scanline (except for V-Blank), before pixels are actually drawn to the screen. Renders sprites.
    Drawing = 3, // Where the PPU transfers pixels to the LCD.
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
            Pixel::White => Color::from_rgb(255, 255, 255),
            Pixel::LightGray => Color::from_rgb(255, 0, 0),
            Pixel::DarkGray => Color::from_rgb(0, 255, 0),
            Pixel::Black => Color::from_rgb(0, 0, 0),
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
/*
impl Pixel {
    fn get_image_byte(&self) -> u8 {
        match self {
            0b11 => Color::WHITE.,
            0b10 => Pixel::LightGray,
            0b01 => Pixel::DarkGray,
            _ => Pixel::Black,
        }
    }
}*/

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
            tile_map_1: [0; 0x3FF + 1],
            tile_map_2: [0; 0x3FF + 1],
            raw_oam: [0; 0xA0],
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
            background_fifo: [Pixel::White; 8],
            sprite_fifo: [Pixel::White; 8],
            bg_x_pc: 0,
            window_lc: 0,
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
            0x9800..=0x9BFF => self.tile_map_1[real_addr - 0x9800],
            0x9C00..=0x9FFF => self.tile_map_2[real_addr - 0x9C00],
            0xFE00..=0xFE9F => self.raw_oam[real_addr - 0xFE00],
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
                self.tile_map_1[real_addr - 0x9800] = value;
            }
            0x9C00..=0x9FFF => {
                self.tile_map_2[real_addr - 0x9C00] = value;
            }
            0xFE00..=0xFE9F => self.raw_oam[real_addr - 0xFE00] = value,
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

    fn get_background_map(&self, buffer: &mut [Tile; 360]) {
        //println!("SCX: {:#X}, SCY: {:#X}", self.scx, self.scy);

        let bg_map: &[u8; 0x3FF + 1] = match self.lcdc & 0x8 {
            0 => &self.tile_map_1,
            _ => &self.tile_map_2,
        };
        *buffer = [empty_tile(); 20 * 18];
        // Background consists of 32x32 tiles, or 256x256 pixels
        // The viewport (Game Boy screen) can only show 20x18 tiles (160x144 pixels)
        let start_index = (self.scx + self.scy * 20) as usize;
        let end_index: usize = start_index + (20 * 18);

        for (i, tile_id) in bg_map[start_index..end_index].iter().enumerate() {
            buffer[i] = match self.get_address_mode() {
                0x8000 => self.tile_set[*tile_id as usize],
                _ => self.tile_set[(*tile_id as i8 as i16 + 128) as usize],
            };
        }
    }

    pub fn get_display(&self, bus: &Bus, scx: u8, scy: u8, buffer: &mut [[Pixel; 20 * 8]; 18 * 8]) {
        let mut tile_buffer: [Tile; 360] = [empty_tile(); 360];
        self.get_background_map(&mut tile_buffer);

        for (i, tile_data) in tile_buffer.iter().enumerate() {
            let start_row = (i / 20) * 8;
            let start_col = (i * 8) % (20 * 8);
            //println!("{}, {}", start_row, start_col);
            for dy in 0..8 {
                for dx in 0..8 {
                    buffer[start_row + dy][start_col + dx] = tile_data[dy][dx];
                }
            }
        }
    }

    pub fn get_debug_display(&self, buffer: &mut [[Pixel; 16 * 8]; 32 * 8]) {
        let mut tile_no = 0;

        for tile_y in 0..24 {
            for tile_x in 0..16 {
                let start_y = tile_y * 8;
                let start_x = tile_x * 8;

                /*
                let tile_set_index = tile_no * 16;
                for byte_start in 0..8 {
                    let byte1 = self.raw_tile_vram[tile_set_index + byte_start*2];
                    let byte2 = self.raw_tile_vram[tile_set_index + byte_start*2 + 1];

                    for bit in 0..8 {
                        let mask = 0b10000000 >> bit;
                        let msb = (byte1 & mask) >> 7-bit;
                        let lsb = (byte2 & mask) >> 7-bit;
                        let pixel = Pixel::from((msb << 1) | lsb);
                        buffer[start_y + byte_start][start_x + bit] = pixel;
                    }
                }
                */
                let tile: Tile = self.tile_set[tile_no];
                for (dy, row) in tile.iter().enumerate() {
                    for (dx, pixel) in row.iter().enumerate() {
                        buffer[start_y + dy][start_x + dx] = *pixel;
                    }
                }

                tile_no += 1;
            }
        }
    }

    pub fn load_tile_data(&self, tile_no: usize, buffer: &mut Tile) {
        let tile: Tile = self.tile_set[tile_no];
        for (dy, row) in tile.iter().enumerate() {
            for (dx, pixel) in row.iter().enumerate() {
                buffer[dy][dx] = *pixel;
            }
        }
    }

    fn copy_from_tile_set(&self, tile_id: u8) -> Tile {
        match self.get_address_mode() {
            0x8000 => self.tile_set[tile_id as usize],
            _ => self.tile_set[(tile_id as i8 as i16 + 128) as usize],
        }
    }

    fn decode_pixels_from_word(&self, byte1: u8, byte2: u8, buffer: &mut [Pixel; 8]) {
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

            buffer[pixel_index] = value;
        }
    }

    //
    //  Pixel FIFI
    //

    fn fetch_background_fifo(&mut self) {
        // Each step takes 2 T-Cycles (8 total).

        // Step 1 - Fetch Tile No.
        // TODO: Determine if rendering window or background - for now I will implement background
        let bg_map: &[u8; 0x3FF + 1] = match self.lcdc & 0x8 {
            0 => &self.tile_map_1,
            _ => &self.tile_map_2,
        };

        let mut tile_index: u8 = bg_map[self.bg_x_pc];
        self.bg_x_pc = (self.bg_x_pc + 1) % SCREEN_WIDTH;

        // vvv IF NOT FETCHING WINDOW PIXELS vvv
        tile_index += self.scx / 8;
        tile_index &= 0x1F; // Wrap around support

        // vvv IF BACKGROUND PIXELS ARE BEING FETCHED vvv
        tile_index += 32 * (((self.ly + self.scy) & 0xFF) / 8);
        // OTHERWISE, IF WINDOW PIXELS ARE BEING FETCHED
        // tile_index += 32 * (WINDOW_LINE_COUNTER / 8)

        //tile_index &= 0x3FF; // Ensure address stays within the tile map

        // Step 2 - Fetch Tile Data (Low)
        let tile_byte_1: u8 = match self.get_address_mode() {
            0x8000 => {
                self.raw_tile_vram[(tile_index * 16 + 2 * ((self.ly + self.scy) % 8)) as usize]
            }
            _ => {
                self.raw_tile_vram[((tile_index as i8 as i16 + 128) * 16
                    + 2 * ((self.ly + self.scy) % 8) as i16)
                    as usize]
            }
        };

        // Step 3 - Fetch Tile Data (High)
        let tile_byte_2: u8 = match self.get_address_mode() {
            0x8000 => {
                self.raw_tile_vram[(tile_index * 16 + 2 * ((self.ly + self.scy) % 8)) as usize + 1]
            }
            _ => {
                self.raw_tile_vram[((tile_index as i8 as i16 + 128) * 16
                    + 2 * ((self.ly + self.scy) % 8) as i16)
                    as usize
                    + 1]
            }
        };

        // Step 4 - Push to FIFO
        let mut pixel_buffer = [Pixel::White; 8];
        self.decode_pixels_from_word(tile_byte_1, tile_byte_2, &mut pixel_buffer);
        self.background_fifo = pixel_buffer;
    }
    fn fetch_sprite_fifo(&mut self) {}
    fn fetch_window_fifo(&mut self) {}
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
