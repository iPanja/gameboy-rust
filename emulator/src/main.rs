mod gameboy;

use gameboy::ppu::Pixel;
use gameboy::GameBoy;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::env;
use std::fs::File;
use std::io::Read;

const SCALE: u32 = 1;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;
const TICKS_PER_FRAME: usize = 60;

fn draw_screen(emu: &mut GameBoy, canvas: &mut Canvas<Window>) {
    // Clear canvas as black
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buf = emu.get_display();

    // Set draw color to white, and update screen
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    /*
    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel != Pixel::White {
            // True => draw as white pixel
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;

            // Draw a rectangle at (x, y) scaled up by SCALE value
            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap()
        }
    }*/

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

            let sx = ((x + y) % SCREEN_WIDTH) as u32;
            let sy = ((x + y) % SCREEN_HEIGHT) as u32;

            let rect = Rect::new((sx * SCALE) as i32, (sy * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap()
        }
    }

    canvas.present();
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    // Read boot ROM file
    let mut bootstrap_buffer: Vec<u8> = Vec::new();
    let mut rom_buffer: Vec<u8> = Vec::new();

    let mut bootstrap_rom = File::open("../roms/DMG_ROM.bin").expect("INVALID ROM");
    let mut rom = File::open("../roms/individual/02-interrupts.gb").expect("INVALID ROM");
    bootstrap_rom.read_to_end(&mut bootstrap_buffer).unwrap();
    rom.read_to_end(&mut rom_buffer).unwrap();

    // Create emulator
    let mut gameboy = GameBoy::new();
    gameboy.read_rom(&rom_buffer);
    //gameboy.read_rom(&bootstrap_buffer);

    // Simulate ticks
    // gameboy.ppu.set_enabled(&mut gameboy.bus, true);

    // Setup SDL
    /*
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
        }
        //tick_timers();

        draw_screen(&mut gameboy, &mut canvas);
    }
    */

    loop {
        gameboy.tick();
    }
}
