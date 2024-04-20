mod gameboy;

use gameboy::ppu::Pixel;
use gameboy::{cpu, GameBoy, CPU};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::fs::File;
use std::io::Read;
use std::{env, fs};
use std::{fs::OpenOptions, io::prelude::*};

const SCALE: u32 = 4;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;
const TICKS_PER_FRAME: usize = 1;

fn draw_screen(emu: &mut GameBoy, canvas: &mut Canvas<Window>) {
    // Clear canvas as black
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buf = emu.get_display();

    for (y, row) in screen_buf.iter().enumerate() {
        for (x, pixel) in row.iter().enumerate() {
            match pixel {
                Pixel::White => {
                    canvas.set_draw_color(Color::RGB(255, 255, 255));
                }
                Pixel::LightGray => {
                    canvas.set_draw_color(Color::RGB(255, 0, 0));
                }
                Pixel::DarkGray => {
                    canvas.set_draw_color(Color::RGB(0, 255, 0));
                }
                Pixel::Black => {
                    canvas.set_draw_color(Color::RGB(0, 0, 0));
                }
            }

            let rect = Rect::new(
                (x as u32 * SCALE) as i32,
                (y as u32 * SCALE) as i32,
                SCALE,
                SCALE,
            );
            canvas.fill_rect(rect).unwrap()
        }
    }

    canvas.present();
}

fn main() {
    // Clear LOG
    if cpu::IS_DEBUGGING {
        fs::remove_file("gb-log");
    }

    env::set_var("RUST_BACKTRACE", "1");
    // Read boot ROM file
    let mut bootstrap_buffer: Vec<u8> = Vec::new();
    let mut rom_buffer: Vec<u8> = Vec::new();

    let mut bootstrap_rom = File::open("../roms/DMG_ROM.bin").expect("INVALID ROM");
    let mut rom = File::open("../roms/individual/01-special.gb").expect("INVALID ROM");
    bootstrap_rom.read_to_end(&mut bootstrap_buffer).unwrap();
    rom.read_to_end(&mut rom_buffer).unwrap();

    // Create emulator
    let mut gameboy = GameBoy::new();
    gameboy.read_rom(&rom_buffer);
    gameboy.read_rom(&bootstrap_buffer);

    // Simulate ticks
    // gameboy.ppu.set_enabled(&mut gameboy.bus, true);

    // Setup SDL
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Game Boy Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    gameboy.enable_display();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'gameloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'gameloop;
                }
                _ => (),
            }
        }

        for _ in 0..TICKS_PER_FRAME {
            gameboy.tick();
            //draw_screen(&mut gameboy, &mut canvas);
            println!("ly: {}", gameboy.bus.ppu.ly);
        }
        //tick_timers();

        draw_screen(&mut gameboy, &mut canvas);
        //if gameboy.cpu.registers.pc >= 0x100 {
        if gameboy.bus.ppu.ly > 100 {
            //log_data(gameboy.bus.ppu.tile_set);
            gameboy.bus.ppu.log_tileset();
        }

        if gameboy.cpu.registers.pc > 0x100 {
            panic!("a");
        }
    }

    /*
    loop {
        gameboy.tick();
    }
    */
}

pub fn log_data(tile_set: [gameboy::ppu::Tile; 384]) {
    for tile in tile_set.iter() {
        //print!("{:?}\n", tile);
        log(format!("{:?}", tile));
    }
    log(format!("--------------------------------\n"));
}
fn log(s: String) {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("gb-vram-log")
        .unwrap();

    if let Err(e) = writeln!(file, "{}", s) {
        eprintln!("Couldn't write to file: {}", e);
    }
}
