use std::{collections::HashMap, rc::Rc};

use glow::{Context, HasContext, NativeProgram, NativeShader, NativeUniformLocation};
use nalgebra_glm as glm;

use super::Renderer;

pub struct Shader {
    context: Rc<Context>,
    program: NativeProgram,
    uniform_cache: HashMap<String, NativeUniformLocation>,
}

impl Shader {
    pub fn create(
        renderer: &Renderer,
        vertex_code: &str,
        fragment_code: &str,
    ) -> Result<Self, String> {
        let context = renderer.get_context();
        unsafe {
            let vertex = create_shader(&context.clone(), glow::VERTEX_SHADER, vertex_code)?;
            let fragment = create_shader(&context.clone(), glow::FRAGMENT_SHADER, fragment_code)?;

            let program = context.create_program()?;
            context.attach_shader(program, vertex);
            context.attach_shader(program, fragment);
            context.link_program(program);
            if !context.get_program_link_status(program) {
                return Err(context.get_program_info_log(program));
            }

            context.detach_shader(program, vertex);
            context.detach_shader(program, fragment);
            context.delete_shader(vertex);
            context.delete_shader(fragment);

            Ok(Self {
                context,
                program,
                uniform_cache: HashMap::new(),
            })
        }
    }

    pub fn set_uniform_mat3(&mut self, name: &str, value: &glm::Mat3) {
        let location = self.get_uniform_location(name);
        unsafe {
            self.context
                .uniform_matrix_3_f32_slice(location.as_ref(), false, glm::value_ptr(value))
        }
    }

    pub fn set_uniform_mat4(&mut self, name: &str, value: &glm::Mat4) {
        let location = self.get_uniform_location(name);
        unsafe {
            self.context
                .uniform_matrix_4_f32_slice(location.as_ref(), false, glm::value_ptr(value))
        }
    }

    pub(super) fn bind(&self) {
        unsafe { self.context.use_program(Some(self.program)) };
    }

    fn get_uniform_location(&mut self, name: &str) -> Option<NativeUniformLocation> {
        if let Some(location) = self.uniform_cache.get(name) {
            return Some(*location);
        }
        let location = unsafe { self.context.get_uniform_location(self.program, name) };
        if let Some(location) = location {
            self.uniform_cache.insert(name.to_string(), location);
        } else {
            eprintln!("Uniform location \"{}\" not found", name);
        }
        location
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { self.context.delete_program(self.program) };
    }
}

unsafe fn create_shader(
    context: &Rc<Context>,
    shader_type: u32,
    code: &str,
) -> Result<NativeShader, String> {
    let shader = context.create_shader(shader_type)?;
    context.shader_source(shader, code);
    context.compile_shader(shader);
    if !context.get_shader_compile_status(shader) {
        return Err(context.get_shader_info_log(shader));
    }
    Ok(shader)
}
