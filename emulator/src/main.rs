use std::borrow::Cow;
use std::env;
use std::error::Error;
use std::io::Cursor;
use std::rc::Rc;

use glium::backend::Facade;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::texture::{ClientFormat, RawImage2d};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior};
use glium::{Surface, Texture2d};
use imgui::{Condition, Image, TextureId, Textures, Ui};
use imgui_glium_renderer::Texture;

const TITLE: &str = "Hello, imgui-rs!";
const SCALE: u32 = 2;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;
const TICKS_PER_FRAME: usize = 10;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    // Common setup for creating a winit window and imgui context, not specifc
    // to this renderer at all except that glutin is used to create the window
    // since it will give us access to a GL context
    let (event_loop, display) = create_window();
    let (mut winit_platform, mut imgui_context) = imgui_init(&display);

    // Create renderer from this crate
    let mut renderer = imgui_glium_renderer::Renderer::init(&mut imgui_context, &display)
        .expect("Failed to initialize renderer");

    // Timer for FPS calculation
    let mut last_frame = std::time::Instant::now();

    // Fix: Assign the imgui::Textures instance to a variable before borrowing it
    let mut textures = imgui::Textures::<imgui_glium_renderer::Texture>::new();
    //let textures_ref: &'static mut imgui::Textures<imgui_glium_renderer::Texture> = &mut textures;
    let mut texture_id: Option<TextureId> = None;

    // Standard winit event loop
    event_loop.run(move |event, _, control_flow| match event {
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

            // Draw our example content
            ui.show_demo_window(&mut true);
            render_gameboy_window(ui, texture_id);

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

            match dummy_texture(display.get_context(), &mut renderer.textures(), texture_id) {
                Ok(_id) => texture_id = Some(_id),
                Err(_e) => println!("{:?}", _e),
            }
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = ControlFlow::Exit,
        event => {
            let gl_window = display.gl_window();
            winit_platform.handle_event(imgui_context.io_mut(), gl_window.window(), &event);
        }
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

fn render_gameboy_window(ui: &mut Ui, texture_id: Option<TextureId>) {
    ui.window("Display")
        .size(
            [SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32],
            Condition::Appearing,
        )
        .resizable(true)
        .position([5.0, 450.0], Condition::Appearing)
        .build(|| {
            if let Some(my_texture_id) = texture_id {
                ui.text("Some generated texture");
                Image::new(my_texture_id, [SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32]).build(ui);
            } else {
                ui.text("Failed to load texture");
            }
        });
}

fn dummy_texture(
    gl_ctx: &dyn Facade,
    textures: &mut Textures<Texture>,
    last_texure_id: Option<TextureId>,
) -> Result<TextureId, Box<dyn Error>> {
    let mut data: Vec<u8> = Vec::with_capacity(SCREEN_WIDTH * SCREEN_HEIGHT);
    for i in 0..SCREEN_WIDTH {
        for j in 0..SCREEN_HEIGHT {
            // Insert RGB values
            data.push(i as u8);
            data.push(j as u8);
            data.push((i + j) as u8);
        }
    }
    /*
    let lenna_bytes = include_bytes!("DuhPatrick.jpg");
    let byte_stream = Cursor::new(lenna_bytes.as_ref());
    let decoder = image::jpeg::JpegDecoder::new(byte_stream)?;

    let (width, height) = image::ImageDecoder::dimensions(&decoder);
    let mut image = vec![0; image::ImageDecoder::total_bytes(&decoder) as usize];
    image::ImageDecoder::read_image(decoder, &mut image)?;
    */

    let raw = RawImage2d {
        data: Cow::Owned(data),
        width: SCREEN_WIDTH as u32,
        height: SCREEN_HEIGHT as u32,
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

    if let Some(lti) = last_texure_id {
        textures.replace(lti, texture);
        return Ok(lti);
    } else {
        return Ok(textures.insert(texture));
    }
}
