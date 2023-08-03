use imgui::Context;
use imgui_glow_renderer::AutoRenderer;
use imgui_sdl2_support::SdlPlatform;
use nalgebra_glm as glm;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;
// use std::{env, path::Path};

use screwdriver::keyvalue::KeyValues;

mod renderer;

fn main() {
    // let args = &env::args().collect::<Vec<String>>();
    // let kv = KeyValues::parse(Path::new(&args[1])).unwrap();
    // test_get(&kv).unwrap();
    // kv.write(Path::new(&args[2])).unwrap();

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

    let renderer = renderer::Renderer::create(gl);
    let mut cube = renderer::VertexData::create(&renderer).unwrap();
    cube.add_data(
        //Postions
        &[
            -1.000000, 1.000000, 1.000000, -1.000000, -1.000000, -1.000000, -1.000000, -1.000000,
            1.000000, -1.000000, 1.000000, -1.000000, 1.000000, -1.000000, -1.000000, -1.000000,
            -1.000000, -1.000000, 1.000000, 1.000000, -1.000000, 1.000000, -1.000000, 1.000000,
            1.000000, -1.000000, -1.000000, 1.000000, 1.000000, 1.000000, -1.000000, -1.000000,
            1.000000, 1.000000, -1.000000, 1.000000, 1.000000, -1.000000, -1.000000, -1.000000,
            -1.000000, 1.000000, -1.000000, -1.000000, -1.000000, -1.000000, 1.000000, -1.000000,
            1.000000, 1.000000, 1.000000, 1.000000, 1.000000, -1.000000, -1.000000, 1.000000,
            1.000000, -1.000000, 1.000000, -1.000000, -1.000000, -1.000000, -1.000000, -1.000000,
            1.000000, -1.000000, 1.000000, 1.000000, -1.000000, 1.000000, -1.000000, -1.000000,
            1.000000, 1.000000, -1.000000, 1.000000, 1.000000, 1.000000, 1.000000, -1.000000,
            1.000000, 1.000000, 1.000000, 1.000000, -1.000000, 1.000000, 1.000000, -1.000000,
            -1.000000, 1.000000, 1.000000, -1.000000, -1.000000, 1.000000, -1.000000, 1.000000,
            -1.000000, -1.000000, 1.000000, -1.000000, 1.000000, -1.000000, -1.000000, 1.000000,
            1.000000, 1.000000, 1.000000, 1.000000,
        ],
        renderer::VertexSize::VEC3,
        0,
    )
    .unwrap();

    cube.add_data(
        &[
            // Normals
            -1.0000, 0.0000, 0.0000, -1.0000, 0.0000, 0.0000, -1.0000, 0.0000, 0.0000, 0.0000,
            0.0000, -1.0000, 0.0000, 0.0000, -1.0000, 0.0000, 0.0000, -1.0000, 1.0000, 0.0000,
            0.0000, 1.0000, 0.0000, 0.0000, 1.0000, 0.0000, 0.0000, 0.0000, 0.0000, 1.0000, 0.0000,
            0.0000, 1.0000, 0.0000, 0.0000, 1.0000, 0.0000, -1.0000, 0.0000, 0.0000, -1.0000,
            0.0000, 0.0000, -1.0000, 0.0000, 0.0000, 1.0000, 0.0000, 0.0000, 1.0000, 0.0000,
            0.0000, 1.0000, 0.0000, -1.0000, 0.0000, 0.0000, -1.0000, 0.0000, 0.0000, -1.0000,
            0.0000, 0.0000, 0.0000, 0.0000, -1.0000, 0.0000, 0.0000, -1.0000, 0.0000, 0.0000,
            -1.0000, 1.0000, 0.0000, 0.0000, 1.0000, 0.0000, 0.0000, 1.0000, 0.0000, 0.0000,
            0.0000, 0.0000, 1.0000, 0.0000, 0.0000, 1.0000, 0.0000, 0.0000, 1.0000, 0.0000,
            -1.0000, 0.0000, 0.0000, -1.0000, 0.0000, 0.0000, -1.0000, 0.0000, 0.0000, 1.0000,
            0.0000, 0.0000, 1.0000, 0.0000, 0.0000, 1.0000, 0.0000,
        ],
        renderer::VertexSize::VEC3,
        1,
    )
    .unwrap();

    let mut shader = renderer::Shader::create(
        &renderer,
        r#"#version 330 core
        layout (location=0) in vec3 pos;
        layout (location=1) in vec3 normal;
        out vec3 color;
        out float light;
        uniform mat4 transform;
        uniform mat4 projection;
        uniform mat3 normal_transform;
        void main() {
            vec3 view_dir = vec3(0, 0, 1);
            light = clamp(dot(normalize(normal_transform * normal), view_dir), 0.2, 1.0);
            //light = clamp(dot(normal, view_dir), 0.2, 1.0);
            color = pos;
            gl_Position = projection*transform*vec4(pos, 1.0);
        }"#,
        r#"#version 330 core
        in vec3 color;
        in float light;
        out vec4 out_color;
        void main() {
            //out_color = vec4(color*light, 1.0);
            out_color = vec4(vec3(1)*light, 1.0);
        }"#,
    )
    .unwrap();

    let proj = glm::perspective::<f32>(1280.0 / 720.0, f32::to_radians(45.0), 0.1, 100.0);
    renderer.enable_depth_test(true);
    renderer.enable_backface_culling(true);

    let mut x_pos = 0.0;
    let mut y_pos = -1.0;
    let mut z_pos = -3.0;
    let mut rotation = 30.0;

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
                        renderer.viewport(width, height);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        imgui_platform.prepare_frame(&mut imgui, &window, &event_pump);

        let ui = imgui.new_frame();
        ui.show_metrics_window(&mut true);
        ui.window("Cube mover 5000").build(|| {
            ui.slider("X", -5.0, 5.0, &mut x_pos);
            ui.slider("Y", -5.0, 5.0, &mut y_pos);
            ui.slider("Z", -5.0, 5.0, &mut z_pos);
            ui.slider("Rotation", -180.0, 180.0, &mut rotation);
        });

        let draw_data = imgui.render();

        let mut transform = glm::Mat4::identity();
        transform = glm::translate(&transform, &glm::vec3(x_pos, y_pos, z_pos));
        transform = glm::rotate(
            &transform,
            f32::to_radians(rotation),
            &glm::vec3(0.0, 1.0, 0.0),
        );
        transform = glm::scale(&transform, &glm::vec3(0.5, 0.5, 0.5));

        let normal_transform = glm::mat4_to_mat3(&glm::inverse(&transform).transpose());

        renderer.clear_color_buffer();
        renderer.clear_depth_buffer();
        renderer.fill(0.27, 0.27, 0.27, 1.0);

        shader.set_uniform_mat4("projection", &proj);
        shader.set_uniform_mat4("transform", &transform);
        shader.set_uniform_mat3("normal_transform", &normal_transform);

        renderer.draw(&cube, &shader);

        imgui_renderer.render(draw_data).unwrap();

        window.gl_swap_window();
    }
}

fn test_get(kv: &KeyValues) -> Option<()> {
    println!("{}", kv.get("versioninfo")?.get("mapversion")?.get_value()?);
    println!("{:#?}", kv.get("world")?.get_all("solid"));
    Some(())
}
