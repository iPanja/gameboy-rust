mod gameboy;

use gameboy::ppu::Pixel;
use gameboy::GameBoy;
use ggez::glam::*;
use ggez::graphics::{self, Canvas, Color, Image, ImageFormat};
use ggez::*;
use std::env;
use std::fs::File;
use std::io::Read;

const SCALE: u32 = 2;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;
const TICKS_PER_FRAME: usize = 10;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    // Read boot ROM file
    let mut bootstrap_buffer: Vec<u8> = Vec::new();
    let mut rom_buffer: Vec<u8> = Vec::new();
    let mut bootstrap_rom = File::open("../roms/DMG_ROM.bin").expect("INVALID ROM");
    let mut rom = File::open("../roms/individual/01-special.gb").expect("INVALID ROM");
    bootstrap_rom.read_to_end(&mut bootstrap_buffer).unwrap();
    rom.read_to_end(&mut rom_buffer).unwrap();

    // GGEZ
    let mut state = State {
        gameboy: GameBoy::new(),
        dt: std::time::Duration::new(0, 0),
    };

    // Create emulator
    state.gameboy.read_rom(&rom_buffer);
    state.gameboy.read_rom(&bootstrap_buffer);
    state.gameboy.enable_display();

    // GGEZ
    let c = conf::Conf::new();
    let (ctx, event_loop) = ContextBuilder::new("gameboy-rust", "Fletcher")
        .default_conf(c)
        .build()
        .unwrap();

    event::run(ctx, event_loop, state);
}

struct State {
    gameboy: GameBoy,
    dt: std::time::Duration,
}
impl ggez::event::EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.dt = ctx.time.delta();
        for _ in 0..TICKS_PER_FRAME {
            self.gameboy.tick();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        //println!("Drawing: {:#X}", self.gameboy.cpu.registers.pc);
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        /*
        let dest_point = Vec2::new(10.0, 10.0);
        let text = graphics::Text::new("Hello, world!").set_scale(48.0).clone();
        canvas.draw(
            &text,
            graphics::DrawParam::from(dest_point).color(Color::from_rgba(192, 128, 64, 255)),
        );
        */
        self.draw_screen(ctx, &mut canvas);

        canvas.finish(ctx)
        //Ok(())
    }
}

impl State {
    fn draw_screen(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult<()> {
        let pixel_buffer: &[[Pixel; 160]; 144] = self.gameboy.get_display();
        let mut screen_buffer: Vec<u8> = Vec::new();
        for row in pixel_buffer.iter() {
            for pixel in row.iter() {
                let color = Color::from(*pixel);

                screen_buffer.push(color.r as u8);
                screen_buffer.push(color.g as u8);
                screen_buffer.push(color.b as u8);
                screen_buffer.push(color.a as u8);
            }
        }

        let screen_image = Image::from_pixels(
            ctx,
            &screen_buffer.as_slice(),
            ImageFormat::Rgba8UnormSrgb,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
        );

        /*
        let mut screen_image = Image::from_color(
            ctx,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
            Some(Color::WHITE),
        );
        */

        let dest_point = Vec2::new(10.0, 10.0);
        canvas.draw(
            &screen_image,
            graphics::DrawParam::from(dest_point).scale(Vec2::new(SCALE as f32, SCALE as f32)),
        );

        Ok(())
    }
}
