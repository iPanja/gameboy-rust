use gameboy::GameBoy;
//use imgui::tables;
use imgui::*;

mod gameboy;

const SCALE: u32 = 2;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;
const TICKS_PER_FRAME: usize = 10;

fn main() {
    // Emulator
    let mut gameboy = GameBoy::new();

    // IMGUI-RS
    let system = gameboy::ui::init(file!());

    let mut value = 0;
    let choices = ["test test this is 1", "test test this is 2"];

    system.main_loop(move |_, ui| {
        ui.show_demo_window(&mut true);
        ui.window("CPU")
            .size([315.0, 400.0], Condition::FirstUseEver)
            .build(|| {
                let num_cols = 2;
                let num_rows = 1000;

                let flags = imgui::TableFlags::ROW_BG
                    | imgui::TableFlags::RESIZABLE
                    | imgui::TableFlags::BORDERS_H
                    | imgui::TableFlags::BORDERS_V; //| imgui::TableFlags::SCROLL_Y;

                if let Some(_t) = ui.begin_table_with_sizing(
                    "cpu_registers",
                    2,
                    flags,
                    [300.0, 100.0],
                    /*inner width=*/ 0.0,
                ) {
                    // Headers (always visible via freeze)
                    ui.table_setup_column("Register");
                    ui.table_setup_column("Value");

                    ui.table_setup_scroll_freeze(num_cols, 1);
                    ui.table_headers_row();

                    // Data
                    let registers: [&u16; 10] = [
                        &(gameboy.cpu.registers.a as u16),
                        &(gameboy.cpu.registers.b as u16),
                        &(gameboy.cpu.registers.c as u16),
                        &(gameboy.cpu.registers.d as u16),
                        &(gameboy.cpu.registers.e as u16),
                        &(u8::from(gameboy.cpu.registers.f) as u16),
                        &(gameboy.cpu.registers.h as u16),
                        &(gameboy.cpu.registers.l as u16),
                        &gameboy.cpu.registers.pc,
                        &gameboy.cpu.registers.sp,
                    ];
                    let labels = ["a", "b", "c", "d", "e", "f", "h", "l", "pc", "sp"];

                    let clip = imgui::ListClipper::new(10).begin(ui);

                    for row_num in clip.iter() {
                        ui.table_next_row();
                        ui.table_set_column_index(0);
                        ui.text(format!("{}", labels[row_num as usize]));
                        ui.table_set_column_index(1);
                        ui.text(format!("{:#X}", registers[row_num as usize]));
                    }
                }

                // Internal registers
                ui.separator();
                ui.text(format!("IF REG - {:#X}", gameboy.bus.ram_read_byte(0xFF0F)));
                ui.text(format!("IE REG - {:#X}", gameboy.bus.ram_read_byte(0xFFFF)));
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
    });
}
