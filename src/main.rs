use imgui::Context;
use imgui_glow_renderer::AutoRenderer;
use imgui_sdl2_support::SdlPlatform;
use nalgebra_glm as glm;
use renderer::{Renderer, Texture, VertexData};
use screwdriver::{
    gameinfo::Gameinfo,
    material::Material,
    vmf::{Brush, Face, VMF},
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

    let vertex_data = get_vertexdatas(&gameinfo, &renderer, &vmf.worldbrushes);

    let mut textures = HashMap::new();
    for material in vertex_data.keys() {
        for texture in material.get_all_textures() {
            let content = gameinfo
                .get_file(&format!("materials/{}", texture), "vtf")
                .unwrap();
            vtf.load(&content).unwrap();
            let texture_data = Texture::create_from_vtf(&renderer, &vtf).unwrap();
            textures.insert(texture.clone(), texture_data);
        }
    }

    let missing_texture = Texture::create_from_data(&renderer, 8, 8, &[
        0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255,
        0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255,
        0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255,
        0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255,
        255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255,
        255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255,
        255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255,
        255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255,
        ]).expect("Data is safe");

    let mut lightmappedgeneric = renderer::Shader::create(
        &renderer,
        include_str!("shaders/lightmappedgeneric-vert.glsl"),
        include_str!("shaders/lightmappedgeneric-frag.glsl"),
    )
    .unwrap();
    let mut unlitgeneric = renderer::Shader::create(
        &renderer,
        include_str!("shaders/unlitgeneric-vert.glsl"),
        include_str!("shaders/unlitgeneric-frag.glsl"),
    )
    .unwrap();

    let mut worldvertextransition = renderer::Shader::create(
        &renderer,
        include_str!("shaders/worldvertextransition-vert.glsl"),
        include_str!("shaders/worldvertextransition-frag.glsl"),
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

        let view = glm::look_at(&camera_pos, &(camera_pos + camera_front), &camera_up);

        let normal_transform = glm::mat4_to_mat3(&glm::inverse(&view).transpose());

        renderer.clear_color_buffer();
        renderer.clear_depth_buffer();
        renderer.fill(0.27, 0.27, 0.5, 1.0);

        lightmappedgeneric.set_uniform_mat4("projection", &proj);
        lightmappedgeneric.set_uniform_mat4("view", &view);
        lightmappedgeneric.set_uniform_mat3("normal_transform", &normal_transform);
        unlitgeneric.set_uniform_mat4("projection", &proj);
        unlitgeneric.set_uniform_mat4("view", &view);
        worldvertextransition.set_uniform_mat4("projection", &proj);
        worldvertextransition.set_uniform_mat4("view", &view);
        worldvertextransition.set_uniform_mat3("normal_transform", &normal_transform);

        for (material, data) in &vertex_data {
            if draw_tool || !material.is_tool() {
                match material {
                    Material::LightmappedGeneric { basetexture } => {
                        let texture = &textures[basetexture];
                        lightmappedgeneric.set_uniform_texture("basetexture", texture, 0);
                        lightmappedgeneric.set_uniform_vec2(
                            "tex_size",
                            &glm::vec2(texture.width as f32, texture.height as f32),
                        );
                        renderer.draw(data, &lightmappedgeneric);
                    }
                    Material::UnlitGeneric { basetexture } => {
                        let texture = &textures[basetexture];
                        unlitgeneric.set_uniform_texture("basetexture", texture, 0);
                        unlitgeneric.set_uniform_vec2(
                            "tex_size",
                            &glm::vec2(texture.width as f32, texture.height as f32),
                        );
                        renderer.draw(data, &unlitgeneric);
                    }
                    Material::WorldVertexTransition {
                        basetexture,
                        basetexture2,
                    } => {
                        let texture = &textures[basetexture];
                        let texture2 = &textures[basetexture2];
                        worldvertextransition.set_uniform_texture("basetexture", texture, 0);
                        worldvertextransition.set_uniform_texture("basetexture2", texture2, 1);
                        worldvertextransition.set_uniform_vec2(
                            "tex_size",
                            &glm::vec2(texture.width as f32, texture.height as f32),
                        );
                        renderer.draw(data, &worldvertextransition);
                    }
                    Material::MissingMaterial => {
                        lightmappedgeneric.set_uniform_texture("basetexture", &missing_texture, 0);
                        lightmappedgeneric.set_uniform_vec2(
                            "tex_size",
                            &glm::vec2(16.0, 16.0),
                        );
                        renderer.draw(data, &lightmappedgeneric);
                    },
                }
            }
        }

        imgui_renderer.render(draw_data).unwrap();
        window.gl_swap_window();
    }
}

fn get_vertexdatas(
    gameinfo: &Gameinfo,
    renderer: &Renderer,
    brushes: &Vec<Brush>,
) -> HashMap<Material, VertexData> {
    let mut data: HashMap<String, (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>)> = HashMap::new(); // Positions, Normals, UVs, Alphas
    for brush in brushes {
        for (info, face) in &brush.shape.faces {
            let info = info
                .as_ref()
                .expect("Invalid Brush: Face not clipped (Brush may be too big)");
            if !data.contains_key(&info.material) {
                data.insert(info.material.clone(), (vec![], vec![], vec![], vec![]));
            }
            let material_data = data
                .get_mut(&info.material)
                .expect("Has been inserted before");
            if !brush.has_displacement() {
                let brush = &brush.shape;
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
                    material_data.3.push(0.0);
                    material_data.3.push(0.0);
                    material_data.3.push(0.0);
                }
            } else if let Some(dispinfo) = &info.dispinfo {
                /* This code is a mess
                s=start_point r=row_point o=opposite_point c=column_point
                left->right = column
                down->up = row

                r---o
                |\  |
                | \ |
                |  \|
                s---c

                */
                let brush = &brush.shape;
                let start_point = face
                    .iter()
                    .position(|x| (brush.vertices[*x] - dispinfo.startpos).norm_squared() < 1.0)
                    .expect(&format!(
                        "A valid displacement should have this {}",
                        info.id
                    ));

                let column_point = brush.vertices[face[(start_point + 1) % face.len()]];
                let opposite_point = brush.vertices[face[(start_point + 2) % face.len()]];
                let row_point = brush.vertices[face[(start_point + 3) % face.len()]];
                let start_point = brush.vertices[face[start_point]];

                for row in 0..(1 << dispinfo.power) {
                    for column in 0..(1 << dispinfo.power) {
                        let n1 = dispinfo.normals[row][column];
                        let p1 = get_disp_point(
                            row,
                            column,
                            dispinfo.power,
                            &start_point,
                            &row_point,
                            &opposite_point,
                            &column_point,
                        ) + dispinfo.offsets[row][column]
                            + n1 * (dispinfo.distances[row][column] + dispinfo.elevation);
                        let n2 = dispinfo.normals[row + 1][column];
                        let p2 = get_disp_point(
                            row + 1,
                            column,
                            dispinfo.power,
                            &start_point,
                            &row_point,
                            &opposite_point,
                            &column_point,
                        ) + dispinfo.offsets[row + 1][column]
                            + n2 * (dispinfo.distances[row + 1][column] + dispinfo.elevation);
                        let n3 = dispinfo.normals[row + 1][column + 1];
                        let p3 = get_disp_point(
                            row + 1,
                            column + 1,
                            dispinfo.power,
                            &start_point,
                            &row_point,
                            &opposite_point,
                            &column_point,
                        ) + dispinfo.offsets[row + 1][column + 1]
                            + n3 * (dispinfo.distances[row + 1][column + 1] + dispinfo.elevation);
                        let n4 = dispinfo.normals[row][column + 1];
                        let p4 = get_disp_point(
                            row,
                            column + 1,
                            dispinfo.power,
                            &start_point,
                            &row_point,
                            &opposite_point,
                            &column_point,
                        ) + dispinfo.offsets[row][column + 1]
                            + n4 * (dispinfo.distances[row][column + 1] + dispinfo.elevation);
                        // println!("{p1} {p2} {p3} {p4}");
                        material_data.0.extend_from_slice(glm::value_ptr(&p1));
                        material_data.0.extend_from_slice(glm::value_ptr(&p4));
                        material_data.0.extend_from_slice(glm::value_ptr(&p2));
                        material_data.0.extend_from_slice(glm::value_ptr(&p2));
                        material_data.0.extend_from_slice(glm::value_ptr(&p4));
                        material_data.0.extend_from_slice(glm::value_ptr(&p3));
                        material_data.1.extend_from_slice(glm::value_ptr(&n1));
                        material_data.1.extend_from_slice(glm::value_ptr(&n4));
                        material_data.1.extend_from_slice(glm::value_ptr(&n2));
                        material_data.1.extend_from_slice(glm::value_ptr(&n2));
                        material_data.1.extend_from_slice(glm::value_ptr(&n4));
                        material_data.1.extend_from_slice(glm::value_ptr(&n3));
                        material_data
                            .2
                            .extend_from_slice(glm::value_ptr(&get_uv_point(info, &p1)));
                        material_data
                            .2
                            .extend_from_slice(glm::value_ptr(&get_uv_point(info, &p4)));
                        material_data
                            .2
                            .extend_from_slice(glm::value_ptr(&get_uv_point(info, &p2)));
                        material_data
                            .2
                            .extend_from_slice(glm::value_ptr(&get_uv_point(info, &p2)));
                        material_data
                            .2
                            .extend_from_slice(glm::value_ptr(&get_uv_point(info, &p4)));
                        material_data
                            .2
                            .extend_from_slice(glm::value_ptr(&get_uv_point(info, &p3)));
                        material_data
                            .3
                            .push(dispinfo.alphas[row][column] as f32 / 255.0);
                        material_data
                            .3
                            .push(dispinfo.alphas[row][column + 1] as f32 / 255.0);
                        material_data
                            .3
                            .push(dispinfo.alphas[row + 1][column] as f32 / 255.0);
                        material_data
                            .3
                            .push(dispinfo.alphas[row + 1][column] as f32 / 255.0);
                        material_data
                            .3
                            .push(dispinfo.alphas[row][column + 1] as f32 / 255.0);
                        material_data
                            .3
                            .push(dispinfo.alphas[row + 1][column + 1] as f32 / 255.0);
                    }
                }
            }
        }
    }
    let mut missing_material_positions = vec![];
    let mut missing_material_normals = vec![];
    let mut missing_material_uvs = vec![];
    let mut missing_material_alphas = vec![];
    let mut renderer_data = HashMap::new();
    for (material, (mut positions, mut normals, mut uvs, mut alphas)) in data {
        if let Some(material) = Material::parse(gameinfo, &material) {
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
            vertex_data
                .add_data(&alphas, renderer::VertexSize::VEC1, 3)
                .unwrap();
            renderer_data.insert(material, vertex_data);
        } else {
            eprintln!("material {material} not found");
            missing_material_positions.append(&mut positions);
            missing_material_normals.append(&mut normals);
            missing_material_uvs.append(&mut uvs);
            missing_material_alphas.append(&mut alphas);
        }
    }

    let mut vertex_data = VertexData::create(renderer).unwrap();
    vertex_data
        .add_data(&missing_material_positions, renderer::VertexSize::VEC3, 0)
        .unwrap();
    vertex_data
        .add_data(&missing_material_normals, renderer::VertexSize::VEC3, 1)
        .unwrap();
    vertex_data
        .add_data(&missing_material_uvs, renderer::VertexSize::VEC2, 2)
        .unwrap();
    vertex_data
        .add_data(&missing_material_alphas, renderer::VertexSize::VEC1, 3)
        .unwrap();
    renderer_data.insert(Material::MissingMaterial, vertex_data);

    renderer_data
}

fn get_uv_point(info: &Face, point: &glm::Vec3) -> glm::Vec2 {
    glm::vec2(
        glm::dot(point, &info.uaxis.dir)
            / glm::dot(&info.uaxis.dir, &info.uaxis.dir)
            / info.uaxis.scaling
            + info.uaxis.translation,
        glm::dot(point, &info.vaxis.dir)
            / glm::dot(&info.vaxis.dir, &info.vaxis.dir)
            / info.vaxis.scaling
            + info.vaxis.translation,
    )
}

fn get_disp_point(
    row: usize,
    column: usize,
    power: u8,
    start_point: &glm::Vec3,
    row_point: &glm::Vec3,
    opposite_point: &glm::Vec3,
    column_point: &glm::Vec3,
) -> glm::Vec3 {
    let max_i = (1 << power) as f32;
    //println!("{} {} {}", row as f32 / max_i, row as f32 / max_i, max_i);
    let p1 = glm::lerp(start_point, row_point, row as f32 / max_i);
    let p2 = glm::lerp(column_point, opposite_point, row as f32 / max_i);
    glm::lerp(&p1, &p2, column as f32 / max_i)
}
