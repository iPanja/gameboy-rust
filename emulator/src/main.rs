use std::{io::Read, thread, time};

use gameboy::GameBoy;
use image::math::utils::clamp;
//use imgui::tables;
use imgui::{
    sys::{ImColor, ImVec2},
    *,
};

mod gameboy;

const SCALE: u32 = 2;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;
const TICKS_PER_FRAME: usize = 10;

fn main() {
    // Emulator
    // Read boot ROM file
    let mut bootstrap_buffer: Vec<u8> = Vec::new();
    let mut rom_buffer: Vec<u8> = Vec::new();

    let mut bootstrap_rom = std::fs::File::open("../roms/DMG_ROM.bin").expect("INVALID ROM");
    let mut rom = std::fs::File::open("../roms/individual/01-special.gb").expect("INVALID ROM");
    bootstrap_rom.read_to_end(&mut bootstrap_buffer).unwrap();
    rom.read_to_end(&mut rom_buffer).unwrap();

    // Create emulator
    let mut gameboy = GameBoy::new();
    gameboy.read_rom(&rom_buffer);
    gameboy.read_rom(&bootstrap_buffer);
    gameboy.enable_display();

    // IMGUI-RS
    let system = gameboy::ui::init(file!());

    /// Breakpoints
    let mut selected_breakpoint: i32 = 0;
    let mut breakpoints: Vec<u16> = Vec::new();
    let mut breakpoints_label: Vec<String> = Vec::new();
    let mut breakpoint_input = String::with_capacity(8);

    /// Tick Rate
    let mut is_playing: bool = false;
    let mut tick_rate: i16 = 5;

    // Sleep rate
    let should_sleep: bool = true;
    let sleep_time = time::Duration::from_millis(10);

    system.main_loop(move |_, ui| {
        ui.show_demo_window(&mut true);

        if is_playing {
            gameboy.tick();
        }

        // Registers Window
        ui.window("Registers")
            .size([315.0, 400.0], Condition::Always)
            .resizable(false)
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
                    ui.text(format!("IF Reg - {:#X}", gameboy.bus.ram_read_byte(0xFF0F)));
                    ui.text(format!("IE Reg - {:#X}", gameboy.bus.ram_read_byte(0xFFFF)));
                    ui.text(format!("LY Reg - {:#X}", gameboy.bus.ram_read_byte(0xFF44)));
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
            });

        // Instructions Window
        ui.window("Instructions")
            .size([315.0, 700.0], Condition::FirstUseEver)
            .position([325.0, 5.0], Condition::Always)
            .build(|| {
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

                        if let Ok((cycles, bytes_consumed)) = gameboy::instruction::parse_opcode(
                            opcode, next_byte, next_word, &mut text,
                        ) {
                            /*
                            for col in 0..3 {
                                if pc_addr == gameboy.cpu.registers.pc {
                                    ui.table_set_bg_color_with_column(
                                        TableBgTarget::all(),
                                        ImColor32::from_rgba(255, 0, 0, 125),
                                        col,
                                    );
                                }
                            }
                            */
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
                        for _ in 0..tick_rate {
                            gameboy.tick_bp(Some(&breakpoints));
                        }
                    }

                    ui.same_line_with_spacing(0.0, 10.0);
                    if let Some(_t) = ui.begin_combo_with_flags(
                        format!("Tick Rate - {}", tick_rate),
                        format!("{}", &mut tick_rate),
                        ComboBoxFlags::NO_PREVIEW,
                    ) {
                        for tr in [5, 10, 25, 50, 100] {
                            if ui
                                .selectable_config(tr.to_string())
                                .selected(tick_rate == tr)
                                .build()
                            {
                                tick_rate = tr;
                            }
                        }
                    }

                    ui.same_line_with_spacing(0.0, 10.0);
                    if ui.button(if is_playing { "Pause" } else { "Play" }) {
                        is_playing = !is_playing;
                    }
                }
                ui.separator();
                // Breakpoint Manager
                {
                    let labels: Vec<&str> = breakpoints_label.iter().map(AsRef::as_ref).collect();
                    ui.set_next_item_width(-1.0);
                    ui.list_box("Breakpoints", &mut selected_breakpoint, &labels, 10);

                    if ui.button("-") {
                        if (selected_breakpoint as usize) < breakpoints.len() {
                            breakpoints.remove(selected_breakpoint as usize);
                            breakpoints_label.remove(selected_breakpoint as usize);
                        }
                    }
                    ui.modal_popup_config("Add Breakpoint")
                        .always_auto_resize(true)
                        .build(|| {
                            ui.input_text("Breakpoint Address", &mut breakpoint_input)
                                .build();
                            ui.separator();

                            let without_prefix = breakpoint_input.trim_start_matches("0x");
                            if ui.button_with_size("OK", [120.0, 0.0]) {
                                let result = u16::from_str_radix(&without_prefix, 16);
                                if let Ok(addr) = &result {
                                    breakpoints.push(*addr);
                                    breakpoints_label.push(format!("0x{:04X}", addr));
                                    ui.close_current_popup();
                                } else if let Err(err) = &result {
                                    ui.text_colored([255.0, 0.0, 0.0, 255.0], "INVALID INPUT!");
                                    println!("{:?}", err);
                                }
                                breakpoint_input = String::with_capacity(32);
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
                }
            });
        /*
        let mut tab_id: String = String::default();
        if let Some(_t) = ui.tab_bar(&tab_id) {
            if let Some(_token) = ui.tab_item("Test") {
                ui.text("WATP");
            }
            if let Some(_token) = ui.tab_item("BEE") {
                ui.text("WAASDASD");
            }
        }*/
        /*
        ui.text_wrapped("Hello world!");
        ui.text_wrapped("こんにちは世界！");
        if ui.button(choices[value]) {
            value += 1;
            value %= 2;
        }

        ui.button("This...is...imgui-rs!");
        ui.separator();
        let mouse_pos = ui.io().mouse_pos;
        ui.text(format!(
            "Mouse Position: ({:.1},{:.1})",
            mouse_pos[0], mouse_pos[1]
        ));
        */

        if should_sleep {
            thread::sleep(sleep_time);
        }
    });
}
