use imgui::{
    sys::{ImColor, ImTextureID, ImVec2, ImVec2_ImVec2_Float},
    *,
};
use imgui_glium_renderer::Texture;
use std::{borrow::Cow, error::Error};

use crate::gameboy::{gameboy, GameBoy};

use std::{io::Read, rc::Rc};

use glium::{
    backend::Facade,
    texture::{ClientFormat, RawImage2d},
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior},
    Texture2d,
};

#[derive(Default, Clone, Copy)]
pub struct ScreenTextureManager {
    pub texture_id: Option<TextureId>,
    pub width: f32,
    pub height: f32,
}

impl ScreenTextureManager {
    pub fn show(&self, ui: &Ui) {
        if let Some(t_id) = self.texture_id {
            Image::new(t_id, [self.width, self.height]).build(ui);
        } else {
            ui.text("Failed to load texture");
        }
    }

    pub fn insert_or_update<F>(
        &mut self,
        gl_ctx: &F,
        textures: &mut Textures<Texture>,
        data: Vec<u8>,
    ) -> Result<(), Box<dyn Error>>
    where
        F: Facade,
    {
        // Import as new entry into textures
        //let mut data = Vec::with_capacity(self.width * self.height);
        //self.gameboy.export_display(&mut data);

        let raw = RawImage2d {
            data: Cow::Owned(data),
            width: self.width as u32,
            height: self.height as u32,
            format: ClientFormat::U8U8U8,
        };
        let gl_texture = Texture2d::new(gl_ctx, raw)?;
        let texture = Texture {
            texture: Rc::new(gl_texture),
            sampler: SamplerBehavior {
                magnify_filter: MagnifySamplerFilter::Linear,
                minify_filter: MinifySamplerFilter::Linear,
                ..Default::default()
            },
        };

        if let Some(t_id) = self.texture_id {
            textures.replace(t_id, texture);
        } else {
            let texture_id = textures.insert(texture);

            self.texture_id = Some(texture_id);
        }

        Ok(())
    }

    pub fn show_textures(&self, ui: &Ui) {
        ui.window("Hello textures")
            .size([400.0, 400.0], Condition::FirstUseEver)
            .build(|| {
                /*
                let draw_list = ui.get_window_draw_list();
                draw_list
                    .add_image(lenna.texture_id, ui.item_rect_min(), ui.item_rect_max())
                    .col(tint)
                    .build();
                }
                */
            });
    }
}
