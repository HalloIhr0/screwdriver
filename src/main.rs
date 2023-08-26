use imgui::{Context, TextureId};
use imgui_glow_renderer::AutoRenderer;
use imgui_sdl2_support::SdlPlatform;
use nalgebra_glm as glm;
use renderer::{Renderer, Texture, VertexData};
use screwdriver::{
    gameinfo::Gameinfo,
    keyvalue::KeyValues,
    vmf::{BrushShape, Face, VMF},
};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;
use std::{collections::HashMap, f32::consts::PI};
use std::{env, path::Path};
use vtflib::VtfLib;

mod renderer;

fn main() {
    let args = &env::args().collect::<Vec<String>>();
    // println!("{}", String::from_utf8(VPK::parse(&args[2]).unwrap().get("scripts/population/mvm_mannworks_intermediate", "pop").unwrap()).unwrap());
    let gameinfo = Gameinfo::parse(Path::new(&args[2])).unwrap();

    let vmf = VMF::parse(Path::new(&args[1])).unwrap();

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

    let (vtflib, mut guard) = VtfLib::initialize().unwrap();
    let vtf = vtflib.new_vtf_file();
    let mut vtf = vtf.bind(&mut guard);

    let vertex_data = get_vertexdatas(
        &renderer,
        vmf.worldbrushes.iter().map(|x| &x.shape).collect(),
    );

    let mut textures = HashMap::new();
    for material in vertex_data.keys() {
        if let Ok(kv) =
            KeyValues::parse_from_searchpath(&gameinfo, &format!("materials/{}", material), "vmt")
        {
            let (shader, properties) = kv.get_all_kv_pairs()[0]; // A meterial file shouldn't be empty
            match shader.to_lowercase().as_str() {
                "lightmappedgeneric" => {
                    let texname = properties
                        .get("$basetexture")
                        .expect(material)
                        .get_value()
                        .expect("b");
                    let content = gameinfo
                        .get_file(&format!("materials/{}", texname), "vtf")
                        .unwrap();
                    vtf.load(&content).unwrap();
                    let texture = Texture::create_from_vtf(&renderer, &vtf).unwrap();
                    textures.insert(material.clone(), texture);
                }
                x => {
                    eprintln!("Unknown shader {} in {}", x, material)
                }
            }
        } else {
            eprintln!("material {material} not found")
        }
    }

    let mut shader = renderer::Shader::create(
        &renderer,
        r#"#version 330 core
        layout (location=0) in vec3 pos;
        layout (location=1) in vec3 normal;
        layout (location=2) in vec2 uvcoord;
        out vec2 uvcoord_pass;
        out float light;
        uniform mat4 transform;
        uniform mat4 view;
        uniform mat4 projection;
        uniform mat3 normal_transform;
        void main() {
            vec3 view_dir = vec3(0, 0, 1);
            light = clamp(dot(normalize(normal_transform * normal), view_dir), 0.0, 1.0)*0.8 + 0.2;
            uvcoord_pass = uvcoord;
            gl_Position = projection*(view*(transform*vec4(pos, 1.0)));
        }"#,
        r#"#version 330 core
        in vec2 uvcoord_pass;
        in float light;
        out vec4 out_color;
        uniform sampler2D color_texture;
        uniform vec2 tex_size;
        void main() {
            vec3 base_color = texture(color_texture, uvcoord_pass/tex_size).rgb;
            out_color = vec4(base_color*light, 1.0);
        }"#,
    )
    .unwrap();

    let proj = glm::perspective::<f32>(1280.0 / 720.0, f32::to_radians(45.0), 1.0, 16384.0);
    renderer.enable_depth_test(true);
    renderer.enable_backface_culling(true);

    let mut camera_pos = glm::vec3(0.0, 0.0, 0.0);
    let mut camera_pitch: f32 = 0.0;
    let mut camera_yaw: f32 = 0.0;

    let camera_speed = 64.0;
    let camera_rotate_speed = 0.1;
    let camera_up = glm::vec3(0.0, 0.0, 1.0);

    let mut draw_tool = true;

    let mut event_pump = sdl_context.event_pump().unwrap();
    'main_loop: loop {
        camera_pitch = camera_pitch.clamp(-PI / 2.0 + 0.01, PI / 2.0 - 0.01);
        // Camera code adapted from https://learnopengl.com/Getting-started/Camera
        let mut direction = glm::vec3(0.0, 0.0, 0.0);
        direction.x = f32::cos(camera_yaw) * f32::cos(camera_pitch);
        direction.y = f32::sin(camera_yaw) * f32::cos(camera_pitch);
        direction.z = f32::sin(camera_pitch);
        let camera_front = glm::normalize(&direction);

        let camera_right = glm::normalize(&glm::cross(&camera_front, &camera_up));

        for event in event_pump.poll_iter() {
            imgui_platform.handle_event(&mut imgui, &event);

            match event {
                Event::Quit { .. } => break 'main_loop,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::W => camera_pos += camera_speed * camera_front,
                    Keycode::S => camera_pos -= camera_speed * camera_front,
                    Keycode::A => camera_pos -= camera_speed * camera_right,
                    Keycode::D => camera_pos += camera_speed * camera_right,
                    Keycode::Up => camera_pitch += camera_rotate_speed,
                    Keycode::Down => camera_pitch -= camera_rotate_speed,
                    Keycode::Left => camera_yaw += camera_rotate_speed,
                    Keycode::Right => camera_yaw -= camera_rotate_speed,
                    _ => {}
                },
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
        ui.window("Image").build(|| {
            ui.checkbox("Draw Tool Textures", &mut draw_tool);
            // imgui::Image::new(
            //     TextureId::new(texture.get_id() as usize),
            //     [texture.width as f32, texture.height as f32],
            // )
            // .build(ui);
        });

        let draw_data = imgui.render();

        let transform = glm::Mat4::identity();
        // let mut transform = glm::Mat4::identity();
        // transform = glm::translate(&transform, &glm::vec3(x_pos, y_pos, z_pos));
        // transform = glm::rotate(
        //     &transform,
        //     f32::to_radians(rotation),
        //     &glm::vec3(0.0, 1.0, 0.0),
        // );
        // transform = glm::scale(&transform, &glm::vec3(0.5, 0.5, 0.5));
        let view = glm::look_at(&camera_pos, &(camera_pos + camera_front), &camera_up);

        let normal_transform = glm::mat4_to_mat3(&glm::inverse(&(view * transform)).transpose());

        renderer.clear_color_buffer();
        renderer.clear_depth_buffer();
        renderer.fill(0.27, 0.27, 0.5, 1.0);

        shader.set_uniform_mat4("projection", &proj);
        shader.set_uniform_mat4("view", &view);
        shader.set_uniform_mat4("transform", &transform);
        shader.set_uniform_mat3("normal_transform", &normal_transform);

        for (material, data) in &vertex_data {
            if !(!draw_tool && material.to_lowercase().starts_with("tools/")) {
                if let Some(texture) = textures.get(material) {
                    shader.set_uniform_texture("color_texture", texture, 0);
                    shader.set_uniform_vec2(
                        "tex_size",
                        &glm::vec2(texture.width as f32, texture.height as f32),
                    );
                    renderer.draw(data, &shader);
                } else {
                    //println!("cant get texture for {material}")
                }
            }
        }

        imgui_renderer.render(draw_data).unwrap();

        window.gl_swap_window();
    }
}

fn get_vertexdatas(renderer: &Renderer, brushes: Vec<&BrushShape>) -> HashMap<String, VertexData> {
    let mut data: HashMap<String, (Vec<f32>, Vec<f32>, Vec<f32>)> = HashMap::new(); // Positions, Normals, UVs
    for brush in brushes {
        for (info, face) in &brush.faces {
            let info = info
                .as_ref()
                .expect("Invalid Brush: Face not clipped (Brush may be too big)");
            if !data.contains_key(&info.material) {
                data.insert(info.material.clone(), (vec![], vec![], vec![]));
            }
            let material_data = data
                .get_mut(&info.material)
                .expect("Has beem inserted before");

            let normal = glm::normalize(&glm::cross(
                &(brush.vertices[face[1]] - brush.vertices[face[0]]),
                &(brush.vertices[face[2]] - brush.vertices[face[0]]),
            ));
            for i in 2..face.len() {
                material_data
                    .0
                    .extend_from_slice(glm::value_ptr(&brush.vertices[face[0]]));
                material_data
                    .0
                    .extend_from_slice(glm::value_ptr(&brush.vertices[face[i - 1]]));
                material_data
                    .0
                    .extend_from_slice(glm::value_ptr(&brush.vertices[face[i]]));
                material_data.1.extend_from_slice(glm::value_ptr(&normal));
                material_data.1.extend_from_slice(glm::value_ptr(&normal));
                material_data.1.extend_from_slice(glm::value_ptr(&normal));
                material_data
                    .2
                    .extend_from_slice(glm::value_ptr(&get_uv_point(
                        info,
                        &brush.vertices[face[0]],
                    )));
                material_data
                    .2
                    .extend_from_slice(glm::value_ptr(&get_uv_point(
                        info,
                        &brush.vertices[face[i - 1]],
                    )));
                material_data
                    .2
                    .extend_from_slice(glm::value_ptr(&get_uv_point(
                        info,
                        &brush.vertices[face[i]],
                    )));
            }
        }
    }

    let mut renderer_data = HashMap::new();
    for (material, (positions, normals, uvs)) in data {
        let mut vertex_data = VertexData::create(renderer).unwrap();
        vertex_data
            .add_data(&positions, renderer::VertexSize::VEC3, 0)
            .unwrap();
        vertex_data
            .add_data(&normals, renderer::VertexSize::VEC3, 1)
            .unwrap();
        vertex_data
            .add_data(&uvs, renderer::VertexSize::VEC2, 2)
            .unwrap();
        renderer_data.insert(material, vertex_data);
    }

    renderer_data
}

#[inline]
fn get_uv_point(info: &Face, point: &glm::Vec3) -> glm::Vec2 {
    glm::vec2(
        (glm::dot(point, &info.uaxis.dir) / glm::dot(&info.uaxis.dir, &info.uaxis.dir))
            / info.uaxis.scaling
            + info.uaxis.translation,
        (glm::dot(point, &info.vaxis.dir) / glm::dot(&info.vaxis.dir, &info.vaxis.dir))
            / info.vaxis.scaling
            + info.vaxis.translation,
    )
}
