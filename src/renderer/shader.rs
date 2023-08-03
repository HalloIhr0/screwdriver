use std::rc::Rc;

use glow::{Context, HasContext, NativeProgram, NativeShader};

use super::Renderer;

pub struct Shader {
    context: Rc<Context>,
    program: NativeProgram,
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

            Ok(Self { context, program })
        }
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
