#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use crate::gui::Framework;
use config::GameBoyConfig;
use emulator::*;
use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

mod config;
mod gui;
mod snapshot;

const SCALE: u32 = 4;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;
const CYCLES_PER_FRAME: f64 = (4194304 / 60) as f64;

/// Representation of the application state. In this example, a box will bounce around the screen.
struct GameBoyState {
    gameboy: Box<GameBoy>,
    config: GameBoyConfig,
}

fn main() -> Result<(), Error> {
    std::env::set_var("RUST_MIN_STACK", format!("{:?}", 100 * 1024 * 1024));
    std::env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("GameBoy Emulator")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let (mut pixels, mut framework) = {
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32 + 10,
            surface_texture,
        )?;
        let framework = Framework::new(
            &event_loop,
            window_size.width,
            window_size.height,
            scale_factor,
            &pixels,
        );

        (pixels, framework)
    };

    let mut gameboy_state = GameBoyState::new();
    let frame_duration = Duration::new(0, (1000.0 / 60.0) as u32);
    let mut last_frame = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Read joypad input (if not in the keybinding settings page)
            if !framework.gui.settings_window_open {
                gameboy_state.update_joypad_state(&input);
            }

            // Update the scale factor
            if let Some(scale_factor) = input.scale_factor() {
                framework.scale_factor(scale_factor);
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                framework.resize(size.width, size.height);
            }

            // Update internal state and request a redraw
            // We aren't necesarrily updating the gameboy's framebuffer, but this will keep the egui responsive
            window.request_redraw();
        }

        *control_flow = ControlFlow::WaitUntil(last_frame + frame_duration); // Ideally "wakes up" when the next frame should be processed & displayed

        match event {
            Event::WindowEvent { event, .. } => {
                // Update egui inputs
                framework.handle_event(&event);
                match event {
                    WindowEvent::KeyboardInput {
                        device_id,
                        input,
                        is_synthetic,
                    } => {
                        if let Some(new_virtual_key) = input.virtual_keycode {
                            if let Some((joypad_key, index)) = framework.gui.binding_tuple {
                                println!("binding!");
                                // Actively awaiting key press for binding
                                let joypad_bindings =
                                    gameboy_state.config.input_mapper.get_mut(&joypad_key);

                                if let Some(joypad_bindings) = joypad_bindings {
                                    joypad_bindings[index] = Some(new_virtual_key);
                                }

                                framework.gui.binding_tuple = None;
                            }
                        }
                    }
                    _ => {}
                }
            }
            // Draw the current frame
            Event::RedrawRequested(_) => {
                // Draw the world
                gameboy_state.draw(pixels.frame_mut());

                // Prepare egui
                framework.prepare(&window, &mut gameboy_state);

                // Render everything together
                let render_result = pixels.render_with(|encoder, render_target, context| {
                    // Render the world texture
                    context.scaling_renderer.render(encoder, render_target);

                    // Render egui
                    framework.render(encoder, render_target, context);

                    Ok(())
                });

                // Basic error handling
                if let Err(err) = render_result {
                    log_error("pixels.render", err);
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::NewEvents(winit::event::StartCause::ResumeTimeReached {
                start,
                requested_resume,
            }) => {
                gameboy_state.update();
                last_frame = Instant::now();
            }
            _ => {
                window.request_redraw();
            }
        }
    });
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

impl GameBoyState {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        let mut gbs = Self {
            gameboy: Box::new(GameBoy::new()),
            config: GameBoyConfig::load(),
        };

        gbs.gameboy
            .read_boot_rom(&GameBoyState::read_rom_into_buffer("DMG_ROM.bin"));
        gbs.gameboy.read_rom(&GameBoyState::read_rom_into_buffer(
            "emulator-only/mbc1/ram_256kb.gb",
        ));

        //GameBoySnapshot::load(&mut gbs.gameboy);
        gbs
    }

    fn reset(&mut self) {
        self.gameboy = Box::new(GameBoy::new());
    }

    fn read_rom_into_buffer(rom_name: &str) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        let path = format!("../roms/{}", rom_name);
        let mut rom = std::fs::File::open(path).expect("INVALID ROM");
        rom.read_to_end(&mut buffer).unwrap();

        buffer
    }

    fn read_rom_from_file_path(path_buf: &PathBuf) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        if let Some(file_path) = path_buf.to_str() {
            let mut rom = std::fs::File::open(file_path).expect("INVALID ROM");
            rom.read_to_end(&mut buffer).unwrap();
        }
        buffer
    }

    fn load_rom(&mut self, path_buf: &PathBuf) {
        self.gameboy
            .read_rom(&GameBoyState::read_rom_from_file_path(&path_buf));
    }

    /// Update the Gameboy internal state; process a frame worth of cycles
    fn update(&mut self) {
        let mut cycles: f64 = 0.0;
        while cycles < CYCLES_PER_FRAME {
            cycles += self.gameboy.step() as f64;
        }
    }

    fn update_joypad_state(&mut self, input: &WinitInputHelper) {
        for (joypad_key, keyboard_codes) in self.config.input_mapper.iter() {
            for keyboard_code in keyboard_codes.iter() {
                if let Some(keyboard_code) = keyboard_code {
                    if input.key_pressed(*keyboard_code) {
                        self.gameboy.bus.joypad.press_key(*joypad_key);
                    } else if input.key_released(*keyboard_code) {
                        self.gameboy.bus.joypad.unpress_key(*joypad_key);
                    }
                }
            }
        }
    }

    /// Draw the Gameboy state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        let display = self.gameboy.bus.ppu.get_display();
        let fixed_display = [0 as u8; 160 * 10 * 4].as_slice();
        let display_slice = display.as_slice();
        let result = [fixed_display, display_slice].concat();

        frame.copy_from_slice(&result);
    }
}
