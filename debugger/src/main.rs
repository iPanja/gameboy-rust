use std::borrow::Cow;
use std::error::Error;
use std::io::{Cursor, Read};
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::{env, thread, time};

use emulator::*;
use glium::backend::Facade;
use glium::glutin::event::{Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::texture::{ClientFormat, RawImage2d};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior};
use glium::{Surface, Texture2d};
use imgui::sys::ImVec2;
use imgui::{
    CollapsingHeader, ComboBoxFlags, Condition, Id, ImColor32, Image, InputText, InputTextFlags,
    Key, TabBarFlags, TabItem, TableBgTarget, TableColumnFlags, TableColumnSetup, TableFlags,
    TextureId, Textures, Ui,
};
use std::collections::HashMap;

pub mod ui;
use ui::ScreenTextureManager;

const TITLE: &str = "GB Emulator & Debugger";
const SCALE: u32 = 1;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const DEBUGGER_SCREEN_WIDTH: usize = 16 * 8;
const DEBUGGER_SCREEN_HEIGHT: usize = 32 * 8;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    //
    // Emulator
    // Read boot ROM file
    let mut bootstrap_buffer: Vec<u8> = Vec::new();
    let mut rom_buffer: Vec<u8> = Vec::new();

    let mut bootstrap_rom = std::fs::File::open("../roms/DMG_ROM.bin").expect("INVALID ROM");
    let mut rom = std::fs::File::open("../roms/instr_timing.gb").expect("INVALID ROM");
    //let mut rom = std::fs::File::open("../roms/individual/02-interrupts.gb").expect("INVALID ROM");

    //let mut rom = std::fs::File::open("../roms/Kirby.gb").expect("INVALID ROM");
    //let mut rom = std::fs::File::open("../roms/dmg-acid2.gb").expect("INVALID ROM");
    bootstrap_rom.read_to_end(&mut bootstrap_buffer).unwrap();
    rom.read_to_end(&mut rom_buffer).unwrap();

    // Create emulator
    let mut gameboy = GameBoy::new();
    gameboy.read_rom(&rom_buffer);
    //gameboy.read_boot_rom(&bootstrap_buffer);

    // Common setup for creating a winit window and imgui context, not specifc
    // to this renderer at all except that glutin is used to create the window
    // since it will give us access to a GL context
    let (event_loop, display) = create_window();
    let (mut winit_platform, mut imgui_context) = imgui_init(&display);
    let style_ref = imgui_context.style_mut();
    style_ref.use_light_colors();
    style_ref.frame_border_size = 1.0;

    // Create renderer from this crate
    let mut renderer = imgui_glium_renderer::Renderer::init(&mut imgui_context, &display)
        .expect("Failed to initialize renderer");

    // Timer for FPS calculation
    let mut first_frame = Instant::now(); // Used to keep track of current fps
    let mut last_frame = std::time::Instant::now(); // Used by imgui for time/delta manipulation
    let frame_duration = Duration::new(0, (1000.0 / 60.0) as u32);
    let cycles_per_frame = (4194304 / 60) as f64; //4194304f64 / 70224f64;
    let mut last_gb_frame = Instant::now(); // Used to track last gameboy frame
    let mut frame_count = 0;

    // Screen Renders
    let mut gb_display_manager: ScreenTextureManager = ScreenTextureManager {
        texture_id: None,
        width: SCREEN_WIDTH as f32,
        height: SCREEN_HEIGHT as f32,
    };
    let mut gb_debugger_manager: ScreenTextureManager = ScreenTextureManager {
        texture_id: None,
        width: DEBUGGER_SCREEN_WIDTH as f32,
        height: DEBUGGER_SCREEN_HEIGHT as f32,
    };
    let mut oam_stms: Vec<ScreenTextureManager> = Vec::with_capacity(40);
    for _ in 0..40 {
        oam_stms.push(ScreenTextureManager {
            texture_id: None,
            width: 8.0,
            height: 8.0,
        });
    }
    // Debugger
    //  > Breakpoints
    let mut selected_breakpoint: i32 = 0;
    let mut breakpoints: Vec<u16> = Vec::new();
    let mut breakpoint_labels: Vec<String> = Vec::new();
    let mut breakpoint_input = String::with_capacity(8);

    // Lets break after the Boot ROM
    breakpoint_labels.push("0x0100".to_string());
    breakpoints.push(0x100 as u16);

    //  > Tick Rate
    let mut is_playing: bool = false;
    let mut tick_rate: i16 = 5;

    // Input handling
    let input_mapper: HashMap<VirtualKeyCode, JoypadInputKey> = HashMap::from([
        (VirtualKeyCode::W, JoypadInputKey::Up),
        (VirtualKeyCode::A, JoypadInputKey::Left),
        (VirtualKeyCode::S, JoypadInputKey::Down),
        (VirtualKeyCode::D, JoypadInputKey::Right),
        (VirtualKeyCode::Right, JoypadInputKey::A),
        (VirtualKeyCode::Left, JoypadInputKey::B),
        (VirtualKeyCode::Q, JoypadInputKey::Start),
        (VirtualKeyCode::E, JoypadInputKey::Select),
    ]);

    // Standard winit event loop
    event_loop.run(move |event, _, control_flow| {
        let start_time = Instant::now();
        //println!("{:?}", event);
        match event {
            Event::NewEvents(StartCause::ResumeTimeReached {
                start,
                requested_resume,
            }) => {
                // Tick emulator
                //println!("resume time reached! {:?}\t{:?}", start, requested_resume);
                perform_gameboy_frame(
                    &mut is_playing,
                    &mut gameboy,
                    cycles_per_frame,
                    &breakpoints,
                );
                update_display(
                    &mut gameboy,
                    &display,
                    &mut renderer,
                    &mut gb_display_manager,
                    &mut gb_debugger_manager,
                    &mut oam_stms,
                );
                last_gb_frame = Instant::now();

                if is_playing {
                    frame_count += 1;
                }
            }
            Event::NewEvents(_) => {
                let now = std::time::Instant::now();
                imgui_context.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }
            Event::MainEventsCleared => {
                let gl_window = display.gl_window();
                winit_platform
                    .prepare_frame(imgui_context.io_mut(), gl_window.window())
                    .expect("Failed to prepare frame");
                gl_window.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                // Create frame for the all important `&imgui::Ui`
                let ui = imgui_context.frame();
                let time_elapsed = first_frame.elapsed().as_secs();
                let fps = if time_elapsed == 0 {
                    0
                } else {
                    frame_count / time_elapsed
                };

                // CREATE UI
                {
                    //ui.show_demo_window(&mut true); // - DEMO WINDOW
                    render_display_window(
                        ui,
                        ["Main Display", "Tile Map"],
                        [gb_display_manager, gb_debugger_manager],
                        &mut gameboy,
                        fps,
                    );
                    render_gameboy_registers(ui, &mut gameboy);
                    let prev_playing = is_playing;
                    render_gameboy_instruction_stepper(
                        ui,
                        &mut gameboy,
                        &mut breakpoints,
                        &mut breakpoint_labels,
                        &mut breakpoint_input,
                        &mut selected_breakpoint,
                        &mut tick_rate,
                        &mut is_playing,
                    );
                    ppu_debugger(ui, &mut gameboy, &oam_stms);
                    if prev_playing != is_playing {
                        first_frame = Instant::now();
                        frame_count = 0;
                    }
                }

                // Setup for drawing
                let gl_window = display.gl_window();
                let mut target = display.draw();

                // Renderer doesn't automatically clear window
                target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);

                // Perform rendering
                winit_platform.prepare_render(ui, gl_window.window());
                let draw_data = imgui_context.render();
                renderer
                    .render(&mut target, draw_data)
                    .expect("Rendering failed");
                target.finish().expect("Failed to swap buffers");
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        device_id: _,
                        input: keyboard_input,
                        is_synthetic: _,
                    },
                ..
            } => {
                if let Some(virtual_keycode) = keyboard_input.virtual_keycode {
                    if let Some(joypad_key) = input_mapper.get(&virtual_keycode) {
                        match keyboard_input.state {
                            glium::glutin::event::ElementState::Pressed => {
                                gameboy.bus.joypad.press_key(*joypad_key);
                            }
                            glium::glutin::event::ElementState::Released => {
                                gameboy.bus.joypad.unpress_key(*joypad_key);
                            }
                        }
                    }
                }
            }
            event => {
                let gl_window = display.gl_window();
                winit_platform.handle_event(imgui_context.io_mut(), gl_window.window(), &event);
            }
        }

        if *control_flow != ControlFlow::Exit {
            *control_flow = ControlFlow::WaitUntil(last_gb_frame + frame_duration);
        }
        //*control_flow = ControlFlow::Poll;
    });
}

fn create_window() -> (EventLoop<()>, glium::Display) {
    let event_loop = EventLoop::new();
    let context = glium::glutin::ContextBuilder::new().with_vsync(true);
    let builder = glium::glutin::window::WindowBuilder::new()
        .with_title(TITLE.to_owned())
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(1024f64, 768f64));
    let display =
        glium::Display::new(builder, context, &event_loop).expect("Failed to initialize display");

    (event_loop, display)
}

fn imgui_init(display: &glium::Display) -> (imgui_winit_support::WinitPlatform, imgui::Context) {
    let mut imgui_context = imgui::Context::create();
    imgui_context.set_ini_filename(None);

    let mut winit_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);

    let gl_window = display.gl_window();
    let window = gl_window.window();

    let dpi_mode = imgui_winit_support::HiDpiMode::Default;

    winit_platform.attach_window(imgui_context.io_mut(), window, dpi_mode);

    imgui_context
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    (winit_platform, imgui_context)
}

/* PROCESS */
fn perform_gameboy_frame(
    is_playing: &mut bool,
    gameboy: &mut GameBoy,
    cycles_per_frame: f64,
    breakpoints: &Vec<u16>,
) {
    let mut cycles = 0f64;
    if *is_playing {
        while cycles < cycles_per_frame {
            //while cycles < gameboy.bus.timer.get_clock_freq() {
            cycles += gameboy.step() as f64;
            if breakpoints.contains(&gameboy.cpu.registers.pc) {
                *is_playing = false;
                break;
            }
            //}
        }
        //println!("cycles consumed {:?}", cycles);
    }
}
/* UPDATE DISPLAY BUFFERS */
fn update_display(
    gameboy: &mut GameBoy,
    display: &glium::Display,
    renderer: &mut imgui_glium_renderer::Renderer,
    gb_display_manager: &mut ScreenTextureManager,
    gb_debugger_manager: &mut ScreenTextureManager,
    oam_stms: &mut Vec<ScreenTextureManager>,
) {
    // Screen renders
    {
        // Main display
        let mut gb_display_buffer: Vec<u8> = Vec::with_capacity(SCREEN_WIDTH * SCREEN_HEIGHT * 4);
        gameboy.export_display(&mut gb_display_buffer);
        match gb_display_manager.insert_or_update(
            display.get_context(),
            &mut renderer.textures(),
            gb_display_buffer,
        ) {
            Ok(_id) => {}
            Err(_e) => println!("{:?}", _e),
        }
        // Update Non-Essential Textures
        // Tile map
        let mut gb_debugger_buffer =
            Vec::with_capacity(DEBUGGER_SCREEN_WIDTH * DEBUGGER_SCREEN_HEIGHT * 4);
        gameboy.export_tile_map_display(&mut gb_debugger_buffer);
        match gb_debugger_manager.insert_or_update(
            display.get_context(),
            &mut renderer.textures(),
            gb_debugger_buffer,
        ) {
            Ok(_id) => {}
            Err(_e) => println!("{:?}", _e),
        }
        // OAM Sprites
        return for (index, sprite) in gameboy.bus.ppu.oam.iter().enumerate() {
            //for bytes in gameboy.bus.ppu.raw_oam.chunks(4) {
            //let tile_index = bytes[2];
            let tile_index = sprite[2];
            let tile_data = gameboy.bus.ppu.tile_set[tile_index as usize];
            let _ = oam_stms[index].insert_or_update(
                display.get_context(),
                renderer.textures(),
                gameboy::PPU::tile_to_vec(&tile_data),
            );
        };
    }
}
/*    IMGUI WINDOWS RENDERING    */
fn render_display_window(
    ui: &mut Ui,
    labels: [&str; 2],
    stms: [ScreenTextureManager; 2],
    gameboy: &mut GameBoy,
    fps: u64,
) {
    ui.window("Displays")
        .position([5.0, 415.0], Condition::Always)
        .size(
            [
                SCREEN_WIDTH as f32 + 18.0,
                DEBUGGER_SCREEN_HEIGHT as f32 + 60.0,
            ],
            Condition::Appearing,
        )
        .resizable(true)
        .build(|| {
            ui.text(format!("FPS: {:?}", fps));
            if let Some(_token) =
                ui.tab_bar_with_flags("DisplayBar", TabBarFlags::AUTO_SELECT_NEW_TABS)
            {
                for index in 0..labels.len() {
                    TabItem::new(labels[index]).build(ui, || {
                        stms[index].show(ui);
                        //stms[index].show_textures(ui);
                    });
                }
            }
        });
}

fn render_gameboy_registers(ui: &mut Ui, gameboy: &mut GameBoy) {
    ui.window("Registers")
        .size([315.0, 400.0], Condition::Appearing)
        .resizable(true)
        .position([5.0, 5.0], Condition::Always)
        .build(|| {
            let table_flags = imgui::TableFlags::ROW_BG
                | imgui::TableFlags::RESIZABLE
                | imgui::TableFlags::BORDERS_H
                | imgui::TableFlags::BORDERS_V; //| imgui::TableFlags::SCROLL_Y;

            if CollapsingHeader::new("CPU").default_open(true).build(ui) {
                if let Some(_t) = ui.begin_table_with_sizing(
                    "cpu_registers_table",
                    2,
                    table_flags,
                    [300.0, 100.0],
                    0.0,
                ) {
                    ui.table_setup_column("Register");
                    ui.table_setup_column("Value");

                    ui.table_setup_scroll_freeze(2, 1);
                    ui.table_headers_row();

                    // Data
                    let registers: [String; 7] = [
                        format!(
                            "{:02X}\t{:02X}",
                            gameboy.cpu.registers.a, gameboy.cpu.registers.f
                        ),
                        format!(
                            "{:02X}\t{:02X}",
                            gameboy.cpu.registers.b, gameboy.cpu.registers.c
                        ),
                        format!(
                            "{:02X}\t{:02X}",
                            gameboy.cpu.registers.d, gameboy.cpu.registers.e
                        ),
                        format!(
                            "{:02X}\t{:02X}",
                            gameboy.cpu.registers.h, gameboy.cpu.registers.l
                        ),
                        format!(
                            "{:04b}\t{:04b}",
                            (u8::from(gameboy.cpu.registers.f) & 0xF0) >> 4,
                            u8::from(gameboy.cpu.registers.f) & 0x0F
                        ),
                        format!("{:04X}", gameboy.cpu.registers.pc),
                        format!("{:04X}", gameboy.cpu.registers.sp),
                    ];
                    let labels = ["AF", "BC", "DE", "HL", "f", "pc", "sp"];

                    let clip = imgui::ListClipper::new(registers.len() as i32).begin(ui);

                    for row_num in clip.iter() {
                        ui.table_next_row();
                        ui.table_set_column_index(0);
                        ui.text(format!("{}", labels[row_num as usize]));
                        ui.table_set_column_index(1);
                        ui.text(&registers[row_num as usize]);
                    }
                }
            }

            // Internal registers
            ui.separator();
            if CollapsingHeader::new("Other Registers")
                .default_open(true)
                .build(ui)
            {
                ui.text(format!(
                    "IF Reg - {:08b}",
                    gameboy.bus.ram_read_byte(0xFF0F)
                ));
                ui.text(format!(
                    "IE Reg - {:08b}",
                    gameboy.bus.ram_read_byte(0xFFFF)
                ));
                ui.separator();

                if let Some(c_h) = &gameboy.cartridge_header {
                    ui.text(format!("Cartridge Game Title: {:?}", c_h.title));
                    ui.text(format!(
                        "Cartridge Type Code: {:#X}",
                        c_h.cartridge_type_code
                    ));
                    ui.text(format!("Cartridge ROM Code: {:#X}", c_h.rom_size_code));
                    ui.text(format!("Cartridge RAM Code: {:#X}", c_h.ram_size_code));
                } else {
                    ui.text("Cartridge Game Title: N/A");
                }
            }
            // Timer internal registers
            ui.separator();
            if CollapsingHeader::new("Timer Registers")
                .default_open(true)
                .build(ui)
            {
                ui.text(format!(
                    "DIV Reg - {:#X}",
                    gameboy.bus.ram_read_byte(0xFF04)
                ));
                ui.text(format!(
                    "TIMA Reg - {:#X}",
                    gameboy.bus.ram_read_byte(0xFF05)
                ));
                ui.text(format!(
                    "TMA Reg - {:#X}",
                    gameboy.bus.ram_read_byte(0xFF06)
                ));
                ui.text(format!(
                    "TAC Reg - {:#X}",
                    gameboy.bus.ram_read_byte(0xFF07)
                ));
            }

            // Serial Port
            ui.separator();
            if CollapsingHeader::new("Serial Port")
                .default_open(true)
                .build(ui)
            {
                let serial_output: String = gameboy.bus.dbg.iter().collect();
                ui.text(serial_output);
            }

            // Joypad
            ui.separator();
            if CollapsingHeader::new("Joypad Input")
                .default_open(true)
                .build(ui)
            {
                ui.text(format!("input byte: {:08b}", gameboy.bus.joypad.input_byte));
                ui.text(format!(
                    "selection mask: {:?}",
                    gameboy.bus.joypad.selection_mask
                ));
                ui.text(format!(
                    "interrupt pending: {:?}",
                    gameboy.bus.joypad.raise_interrupt
                ));
            }
        });
}

fn render_gameboy_instruction_stepper(
    ui: &mut Ui,
    gameboy: &mut GameBoy,
    breakpoints: &mut Vec<u16>,
    breakpoint_labels: &mut Vec<String>,
    breakpoint_input: &mut String,
    selected_breakpoint: &mut i32,
    tick_rate: &mut i16,
    is_playing: &mut bool,
) {
    ui.window("Stepper")
        .size([335.0, 700.0], Condition::FirstUseEver)
        .position([325.0, 5.0], Condition::Always)
        .build(|| {
            if CollapsingHeader::new("Instructions")
                .default_open(true)
                .build(ui)
            {
                let table_flags = imgui::TableFlags::ROW_BG
                    | imgui::TableFlags::RESIZABLE
                    | imgui::TableFlags::BORDERS_H
                    | imgui::TableFlags::BORDERS_V
                    | TableFlags::SIZING_FIXED_FIT;

                if let Some(_t) = ui.begin_table_header_with_sizing(
                    "cpu_instruction_headers",
                    [
                        TableColumnSetup {
                            name: "Addr",
                            flags: TableColumnFlags::WIDTH_FIXED,
                            init_width_or_weight: 0.0,
                            user_id: Id::default(),
                        },
                        TableColumnSetup {
                            name: "OP Code",
                            flags: TableColumnFlags::WIDTH_FIXED,
                            init_width_or_weight: 0.0,
                            user_id: Id::default(),
                        },
                        TableColumnSetup {
                            name: "Instruction",
                            flags: TableColumnFlags::WIDTH_STRETCH,
                            init_width_or_weight: 0.0,
                            user_id: Id::default(),
                        },
                        TableColumnSetup {
                            name: "T-Cycles",
                            flags: TableColumnFlags::WIDTH_FIXED,
                            init_width_or_weight: 50.0,
                            user_id: Id::default(),
                        },
                    ],
                    table_flags,
                    [0.0, 0.0],
                    0.0,
                ) {
                    let mut pc_addr = gameboy.cpu.registers.pc.saturating_sub(5);

                    for _ in 0..10 {
                        let opcode = gameboy.bus.ram_read_byte(pc_addr);
                        let next_byte = gameboy.bus.ram_read_byte(pc_addr.wrapping_add(1));
                        let next_word = gameboy.bus.ram_read_word(pc_addr.wrapping_add(1));

                        let mut text = format!("");

                        if !*is_playing || true {
                            if let Ok((cycles, bytes_consumed)) = gameboy::instruction::parse_opcode(
                                opcode, next_byte, next_word, &mut text,
                            ) {
                                let text: [String; 4] = [
                                    format!("0x{:04X}", pc_addr),
                                    match bytes_consumed {
                                        1 => format!("{:02X}", opcode),
                                        2 => format!("{:02X}\t{:02X}", opcode, next_byte),
                                        _ => format!("{:02X}\t{:04X}", opcode, next_word),
                                    },
                                    format!("{}", text),
                                    format!("{}", cycles),
                                ];

                                for col in 0..4 {
                                    ui.table_next_column();
                                    if pc_addr == gameboy.cpu.registers.pc {
                                        ui.table_set_bg_color(
                                            TableBgTarget::all(),
                                            ImColor32::from_rgba(255, 0, 0, 125),
                                        );
                                    }

                                    ui.text(&text[col]);
                                }

                                pc_addr = pc_addr.wrapping_add(bytes_consumed as u16);
                            }
                        }
                    }
                }
            }

            ui.separator();
            // Step/Play Functionality
            {
                if ui.button("STEP") {
                    gameboy.step();
                }

                ui.same_line_with_spacing(0.0, 10.0);
                if ui.button("TICK") {
                    gameboy.tick_bp(Some(&breakpoints));
                }

                ui.same_line_with_spacing(0.0, 10.0);
                if ui.button("TICK >>") {
                    for _ in 0..*tick_rate {
                        if gameboy.tick_bp(Some(&breakpoints)) {
                            break;
                        }
                    }
                }

                ui.same_line_with_spacing(0.0, 10.0);
                if let Some(_t) = ui.begin_combo_with_flags(
                    format!("Tick Rate - {}", tick_rate),
                    format!("{}", tick_rate),
                    ComboBoxFlags::NO_PREVIEW,
                ) {
                    for tr in [5, 10, 25, 50, 100] {
                        if ui
                            .selectable_config(tr.to_string())
                            .selected(*tick_rate == tr)
                            .build()
                        {
                            *tick_rate = tr;
                        }
                    }
                }

                ui.same_line_with_spacing(0.0, 10.0);
                if ui.button(if *is_playing { "Pause" } else { "Play" }) {
                    *is_playing = !*is_playing;
                }
            }
            ui.separator();
            // Breakpoint Manager
            {
                if CollapsingHeader::new("Breakpoints")
                    .default_open(true)
                    .build(ui)
                {
                    let labels: Vec<&str> = breakpoint_labels.iter().map(AsRef::as_ref).collect();
                    ui.set_next_item_width(-1.0);
                    ui.list_box("Breakpoints", selected_breakpoint, &labels, 10);

                    if ui.button("-") {
                        if (*selected_breakpoint as usize) < breakpoints.len() {
                            breakpoints.remove(*selected_breakpoint as usize);
                            breakpoint_labels.remove(*selected_breakpoint as usize);
                        }
                    }
                    ui.modal_popup_config("Add Breakpoint")
                        .always_auto_resize(true)
                        .build(|| {
                            ui.input_text("Breakpoint Address", breakpoint_input)
                                .hint("0x0100")
                                .build();
                            ui.separator();

                            let without_prefix = breakpoint_input.trim_start_matches("0x");
                            if ui.button_with_size("OK", [120.0, 0.0]) {
                                let result = u16::from_str_radix(&without_prefix, 16);
                                if let Ok(addr) = &result {
                                    breakpoints.push(*addr);
                                    breakpoint_labels.push(format!("0x{:04X}", addr));
                                    ui.close_current_popup();
                                } else if let Err(err) = &result {
                                    ui.text_colored([255.0, 0.0, 0.0, 255.0], "INVALID INPUT!");
                                    println!("{:?}", err);
                                }
                                *breakpoint_input = String::with_capacity(32);
                            }
                            ui.same_line();
                            if ui.button_with_size("Cancel", [120.0, 0.0]) {
                                ui.close_current_popup();
                            }
                        });
                    ui.same_line_with_spacing(0.0, 10.0);
                    if ui.button("+") {
                        ui.open_popup("Add Breakpoint");
                    }
                };
            }

            if ui.is_key_pressed_no_repeat(imgui::Key::MouseRight) {
                if (*selected_breakpoint as usize) < breakpoints.len() {
                    breakpoints.remove(*selected_breakpoint as usize);
                    breakpoint_labels.remove(*selected_breakpoint as usize);
                }
            }
        });
}

fn ppu_debugger(ui: &mut Ui, gameboy: &mut GameBoy, oam_stms: &Vec<ScreenTextureManager>) {
    ui.window("PPU Info")
        .size([300.0, 500.0], Condition::FirstUseEver)
        .position([675.0, 5.0], Condition::Always)
        .build(|| {
            ui.text(format!("LY Reg - {:#X}", gameboy.bus.ram_read_byte(0xFF44)));
            ui.text(format!(
                "LYC Reg - {:#X}",
                gameboy.bus.ram_read_byte(0xFF45)
            ));
            ui.text(format!(
                "LCDC Reg - {:08b}",
                gameboy.bus.ram_read_byte(0xFF40)
            ));
            ui.text(format!(
                "STAT Reg - {:08b}",
                gameboy.bus.ram_read_byte(0xFF41)
            ));
            ui.text(format!("SCY: {:#X}", gameboy.bus.ram_read_byte(0xFF42)));
            ui.text(format!("SCX: {:#X}", gameboy.bus.ram_read_byte(0xFF43)));
            ui.text(format!(
                "sprite cache: {:?}",
                gameboy.bus.ppu.scanline_sprite_cache
            ));
            ui.text(format!(
                "sprite height: {:?}",
                gameboy.bus.ppu.get_sprite_height()
            ));
            ui.separator();

            ui.child_window("OAM")
                .size([275.0, 400.0])
                .horizontal_scrollbar(true)
                .border(true)
                .build(|| {
                    for (i, sprite) in gameboy.bus.ppu.oam.iter().enumerate() {
                        oam_stms[i].show(ui);
                        ui.same_line();
                        ui.text(format!("{:#4X}\t\t{:?}", i, sprite));
                    }
                });
        });
}
