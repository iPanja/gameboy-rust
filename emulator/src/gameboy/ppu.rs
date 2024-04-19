use std::io::empty;

use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};

use super::Bus;

const VRAM_SIZE: usize = (0x9FFF - 0x8000) + (0xFE9F - 0xFE00);

const TILE_MAP_SIZE: usize = 0x3FF;

pub struct PPU {
    vram: [u8; VRAM_SIZE],
    tile_set: [Tile; 384],
    lcdc: u8, // PPU control register - 0xFF40
    stat: u8, // PPU status register - 0xFF41
    scy: u8,  // Vertical scroll register - 0xFF42
    scx: u8,  // Horizontal scroll register - 0xFF43
    ly: u8,   // Scanline register - 0xFF44
              // bg_map_1: [Tile; 0x3FF], // Background Map 1 - 0x9800 - 9x9BFF
              // bg_map_2: [Tile; 0x3FF], // Background Map 2 - 0x9C00 - 0x9FFF
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Pixel {
    White,     // 0b11
    DarkGray,  // 0b10
    LightGray, // 0b01
    Black,     // 0b00
}

type Tile = [[Pixel; 8]; 8];
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
            vram: [0; VRAM_SIZE],
            tile_set: [empty_tile(); 384],
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
        }
    }

    pub fn read_byte(&self, index: usize, real_addr: usize) -> u8 {
        match real_addr {
            0xFF40 => self.lcdc,
            0xFF41 => self.stat,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            _ => self.vram[index],
        }
    }

    pub fn write_byte(&mut self, index: usize, real_addr: usize, value: u8) {
        //println!("offset of: {:#X}", index);

        // println!("writing to: {:#X} - {:#X}", real_addr, value);

        match real_addr {
            0xFF40 => self.lcdc = value,
            0xFF41 => self.stat = value,
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => self.ly = value,
            _ => self.vram[index] = value,
        };

        // Not writing to tile set storage => no need to cache
        if index >= 0x1800 {
            return;
        }

        // Cache
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
    }

    pub fn enable_lcd(&mut self, bus: &mut Bus) {
        bus.ram.ram[0xFF40] = 0b1010000 | self.vram[0xFF40];
    }

    fn get_address_mode(&self) -> u16 {
        match self.lcdc & 0x10 {
            0 => 0x8800,
            _ => 0x8000,
        }
    }

    fn get_tile(&self, id: u16) -> Tile {
        match self.get_address_mode() {
            0x8800 => {
                let idx = id as i16;
                self.tile_set[(192 + id as i16) as usize]
            }
            0x8000 => self.tile_set[id as usize],
            _ => panic!("Unsupported address mode!"),
        }
    }

    fn get_background_map(&self) -> [Tile; 1024] {
        //println!("SCX: {:#X}, SCY: {:#X}", self.scx, self.scy);

        let bg_map_address: usize = match self.lcdc & 0x8 {
            0 => 0x9800,
            _ => 0x9C00,
        };
        let mut tiles: [Tile; 1024] = [empty_tile(); 1024];
        // Background consists of 32x32 tiles, or 256x256 pixels
        // The viewport (Game Boy screen) can only show 20x18 tiles (160x144 pixels)
        let starting_offset = (self.scx + self.scy * 20) as usize;
        let start_index: usize = (bg_map_address + starting_offset) - 0x8000;
        let end_index: usize = start_index + (20 * 18) + 1;

        for (i, tile_id) in self.vram[start_index..=end_index].iter().enumerate() {
            tiles[i] = self.get_tile(*tile_id as u16);
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

        for (tile_index, tile) in self.get_background_map().iter().enumerate() {
            let x = tile_index % 20;
            let y = tile_index / 20;
            for dx in 0..8 {
                for dy in 0..8 {
                    // (y + dy) * 160 + (x + dx) * 8
                    buffer[y + dy][x + dx] = Pixel::LightGray; //tile[dy][dx];
                                                               //println!("writing tile data: {:?}", tile[dy][dx]);
                }
            }
        }

        buffer
    }
}
