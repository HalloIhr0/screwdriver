use glow::*;
use imgui::Context;
use imgui_glow_renderer::AutoRenderer;
use imgui_sdl2_support::SdlPlatform;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;
use std::{env, path::Path};

use screwdriver::keyvalue::KeyValues;

mod renderer;

fn main() {
    let args = &env::args().collect::<Vec<String>>();
    let kv = KeyValues::parse(Path::new(&args[1])).unwrap();
    test_get(&kv).unwrap();
    kv.write(Path::new(&args[2])).unwrap();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem
        .window("Window", 1280, 720)
        .position_centered()
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let ctx = window.gl_create_context().unwrap();
    window.gl_make_current(&ctx).unwrap();
    video_subsystem.gl_set_swap_interval(1).unwrap(); //Vsync
    let gl = unsafe {
        glow::Context::from_loader_function(|s| video_subsystem.gl_get_proc_address(s) as *const _)
    };
    let mut imgui = Context::create();
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);
    let mut imgui_platform = SdlPlatform::init(&mut imgui);
    let mut imgui_renderer = AutoRenderer::initialize(gl, &mut imgui).unwrap();
    let gl = imgui_renderer.gl_context().clone();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'main_loop: loop {
        for event in event_pump.poll_iter() {
            imgui_platform.handle_event(&mut imgui, &event);

            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main_loop,
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Resized(width, height) => {
                        unsafe { gl.viewport(0, 0, width, height) };
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        imgui_platform.prepare_frame(&mut imgui, &window, &event_pump);

        let ui = imgui.new_frame();
        ui.show_demo_window(&mut true);

        let draw_data = imgui.render();

        unsafe {
            gl.clear(COLOR_BUFFER_BIT);
            gl.clear_color(0.27, 0.27, 0.27, 1.0);
        }

        imgui_renderer.render(draw_data).unwrap();

        window.gl_swap_window();
    }
}

fn test_get(kv: &KeyValues) -> Option<()> {
    println!("{}", kv.get("versioninfo")?.get("mapversion")?.get_value()?);
    println!("{:#?}", kv.get("world")?.get_all("solid"));
    Some(())
}
