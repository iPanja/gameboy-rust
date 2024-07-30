use crate::{snapshot::GameBoyGameSave, GameBoyState};
use egui::{ClippedPrimitive, Context, TexturesDelta};
use egui_wgpu::renderer::{Renderer, ScreenDescriptor};
use emulator::gameboy::joypad::JoypadInputKey;
use pixels::{wgpu, PixelsContext};
use rfd::FileDialog;
use std::fmt;
use std::slice::Iter;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;

// Settings Window Tabs/States
#[derive(PartialEq, Copy, Clone)]
enum SettingsTabEnum {
    Keybinds,
    ColorPalette,
    SaveStates,
}

impl SettingsTabEnum {
    pub fn iter() -> Iter<'static, SettingsTabEnum> {
        static TABS: [SettingsTabEnum; 3] = [
            SettingsTabEnum::Keybinds,
            SettingsTabEnum::ColorPalette,
            SettingsTabEnum::SaveStates,
        ];
        TABS.iter()
    }
}

impl fmt::Debug for SettingsTabEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SettingsTabEnum::Keybinds => write!(f, "Keybinds"),
            SettingsTabEnum::ColorPalette => write!(f, "Color Palette"),
            SettingsTabEnum::SaveStates => write!(f, "Save Manager"),
        }
    }
}

/// Manages all state required for rendering egui over `Pixels`.
pub struct Framework {
    // State for egui.
    egui_ctx: Context,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    renderer: Renderer,
    paint_jobs: Vec<ClippedPrimitive>,
    textures: TexturesDelta,

    // State for the GUI
    pub gui: Gui,
}

/// Example application state. A real application will need a lot more state than this.
pub struct Gui {
    /// Only show the egui window when true.
    about_window_open: bool,
    pub settings_window_open: bool,
    debug_window_open: bool,
    settings_tab_state: SettingsTabEnum,
    // Key Binding
    pub binding_tuple: Option<(JoypadInputKey, usize)>,
}

impl Framework {
    /// Create egui.
    pub(crate) fn new<T>(
        event_loop: &EventLoopWindowTarget<T>,
        width: u32,
        height: u32,
        scale_factor: f32,
        pixels: &pixels::Pixels,
    ) -> Self {
        let max_texture_size = pixels.device().limits().max_texture_dimension_2d as usize;

        let egui_ctx = Context::default();
        let mut egui_state = egui_winit::State::new(event_loop);
        egui_state.set_max_texture_side(max_texture_size);
        egui_state.set_pixels_per_point(scale_factor);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: scale_factor,
        };
        let renderer = Renderer::new(pixels.device(), pixels.render_texture_format(), None, 1);
        let textures = TexturesDelta::default();
        let gui = Gui::new();

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            renderer,
            paint_jobs: Vec::new(),
            textures,
            gui,
        }
    }

    /// Handle input events from the window manager.
    pub(crate) fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        let _ = self.egui_state.on_event(&self.egui_ctx, event);
    }

    /// Resize egui.
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.screen_descriptor.size_in_pixels = [width, height];
        }
    }

    /// Update scaling factor.
    pub(crate) fn scale_factor(&mut self, scale_factor: f64) {
        self.screen_descriptor.pixels_per_point = scale_factor as f32;
    }

    /// Prepare egui.
    pub(crate) fn prepare(&mut self, window: &Window, gameboy_state: &mut GameBoyState) {
        // Run the egui frame and create all paint jobs to prepare for rendering.
        let raw_input = self.egui_state.take_egui_input(window);
        let output = self.egui_ctx.run(raw_input, |egui_ctx| {
            // Draw the demo application.
            if gameboy_state.is_menu_visible {
                self.gui.ui(egui_ctx, gameboy_state);
            }
        });

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(window, &self.egui_ctx, output.platform_output);
        self.paint_jobs = self.egui_ctx.tessellate(output.shapes);
    }

    /// Render egui.
    pub(crate) fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) {
        // Upload all resources to the GPU.
        for (id, image_delta) in &self.textures.set {
            self.renderer
                .update_texture(&context.device, &context.queue, *id, image_delta);
        }
        self.renderer.update_buffers(
            &context.device,
            &context.queue,
            encoder,
            &self.paint_jobs,
            &self.screen_descriptor,
        );

        // Render egui with WGPU
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: render_target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.renderer
                .render(&mut rpass, &self.paint_jobs, &self.screen_descriptor);
        }

        // Cleanup
        let textures = std::mem::take(&mut self.textures);
        for id in &textures.free {
            self.renderer.free_texture(id);
        }
    }
}

impl Gui {
    /// Create a `Gui`.
    fn new() -> Self {
        Self {
            about_window_open: false,
            settings_window_open: false,
            debug_window_open: false,
            binding_tuple: None,
            settings_tab_state: SettingsTabEnum::Keybinds,
        }
    }

    /// Create the UI using egui.
    fn ui(&mut self, ctx: &Context, gameboy_state: &mut GameBoyState) {
        egui::TopBottomPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("About...").clicked() {
                        self.about_window_open = true;
                        ui.close_menu();
                    }
                    if ui.button("Load ROM").clicked() {
                        let file_path: Option<std::path::PathBuf> = FileDialog::new()
                            .add_filter("ROMs", &["gb", "rom", "bin"])
                            .add_filter("Anything", &["*"])
                            .pick_file();

                        if let Some(file_path) = file_path {
                            if file_path.is_file() {
                                // probably redundant?
                                gameboy_state.reset();
                                gameboy_state.load_rom(&file_path);
                            }
                        }

                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Settings...").clicked() {
                        self.settings_window_open = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Debug", |ui| {
                    if ui.button("Show debug window").clicked() {
                        self.debug_window_open = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // About Window
        egui::Window::new("GameBoy Emulator!")
            .open(&mut self.about_window_open)
            .show(ctx, |ui| {
                ui.label("GameBoy Emulator, written in Rust");
                ui.label("Version: 1.0.0");

                ui.separator();

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x /= 2.0;
                    ui.label("Visit the github repository: ");
                    ui.hyperlink("https://github.com/iPanja/gameboy-rust");
                });
            });

        // Settings Window
        egui::Window::new("Emulator Settings")
            .open(&mut self.settings_window_open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for tab in SettingsTabEnum::iter() {
                        ui.selectable_value(
                            &mut self.settings_tab_state,
                            *tab,
                            format!("{:?}", *tab),
                        );
                    }
                });
                ui.separator();

                match self.settings_tab_state {
                    SettingsTabEnum::Keybinds => {
                        egui::Grid::new("keybind_grid").show(ui, |ui| {
                            // Header row
                            {
                                ui.label(egui::RichText::new("Joypad Key").strong());
                                ui.label(egui::RichText::new("Primary Bind").strong());
                                ui.label(egui::RichText::new("Secondary Bind").strong());
                                ui.end_row();
                            }

                            // Keybinds
                            for (joypad_key, virtual_keys) in
                                gameboy_state.config.input_mapper.iter_mut()
                            {
                                ui.label(format!("{:?}", joypad_key));

                                for (i, virtual_key) in virtual_keys.iter_mut().enumerate() {
                                    // Determine what text to display for each potential binding
                                    // Default: unbound
                                    let mut text = format!("-");

                                    // Normal label
                                    if let Some(_vk) = virtual_key {
                                        text = format!("{:?}", _vk)
                                    }

                                    // Actively binding
                                    if let Some((_jk, _i)) = self.binding_tuple {
                                        if joypad_key.eq(&_jk) && _i == i {
                                            text = format!("-- Press any key --");
                                        }
                                    }

                                    // Display button
                                    let response = ui.button(text);
                                    if response.clicked() {
                                        self.binding_tuple = Some((*joypad_key, i));
                                    } else if response.secondary_clicked() {
                                        *virtual_key = None;
                                    }
                                }

                                ui.end_row();
                            }
                        });
                    }
                    SettingsTabEnum::ColorPalette => {
                        ui.label("Color Palette Settings");
                        ui.separator();

                        for pixel_color in gameboy_state.config.color_palette.iter_mut() {
                            ui.color_edit_button_srgb(pixel_color);
                        }
                    }
                    SettingsTabEnum::SaveStates => {
                        ui.horizontal(|ui| {
                            ui.label("Save Manager");
                            ui.separator();

                            // Save current state
                            if ui.button("Save State").clicked() {
                                GameBoyGameSave::default().save(&gameboy_state.gameboy);
                            }

                            // Load (default/last) state
                            if ui.button("Load State...").clicked() {
                                GameBoyGameSave::default().load(&mut gameboy_state.gameboy);
                            }
                        });
                        ui.separator();
                        ui.label("Recent saves:");
                        egui::Grid::new("keybind_grid").show(ui, |ui| {
                            // Saves
                            ui.horizontal(|ui| {
                                ui.button("▶");
                                ui.label("Save 1 - 6/6/2024 9:04PM");
                            });
                        });
                    }
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Save Config").clicked() {
                        gameboy_state.config.save();
                    }
                });
            });

        // Debug Window
        egui::Window::new("Debug")
            .open(&mut self.debug_window_open)
            .show(ctx, |ui| {
                ui.label("Version: 1.0.0");

                ui.separator();
                ui.label("[Cartridge Header]");
                if let Some(c_h) = &gameboy_state.gameboy.cartridge_header {
                    ui.label(format!("Cartridge Game Title: {:?}", c_h.title));
                    ui.label(format!(
                        "Cartridge Type Code: {:#X}",
                        c_h.cartridge_type_code
                    ));
                    ui.label(format!("Cartridge ROM Code: {:#X}", c_h.rom_size_code));
                    ui.label(format!("Cartridge RAM Code: {:#X}", c_h.ram_size_code));
                } else {
                    ui.label("Cartridge Game Title: N/A");
                }
            });
    }
}
