use glow::{Context, HasContext};
use std::rc::Rc;

mod shader;
pub use shader::Shader;
mod vertex_data;
pub use vertex_data::{VertexData, VertexSize};
mod texture;
pub use texture::Texture;

pub struct Renderer {
    context: Rc<Context>,
}

impl Renderer {
    pub fn create(context: Rc<Context>) -> Self {
        unsafe {
            context.cull_face(glow::BACK);
            context.front_face(glow::CCW);
        }
        Self { context }
    }

    pub fn viewport(&self, width: i32, height: i32) {
        unsafe { self.context.viewport(0, 0, width, height) };
    }

    pub fn clear_color_buffer(&self) {
        unsafe { self.context.clear(glow::COLOR_BUFFER_BIT) };
    }

    pub fn clear_depth_buffer(&self) {
        unsafe { self.context.clear(glow::DEPTH_BUFFER_BIT) };
    }

    pub fn fill(&self, r: f32, g: f32, b: f32, a: f32) {
        unsafe { self.context.clear_color(r, g, b, a) };
    }

    pub fn draw(&self, data: &VertexData, shader: &Shader) {
        let count = data.prepare_rendering();
        shader.bind();
        unsafe { self.context.draw_arrays(glow::TRIANGLES, 0, count) };
    }

    pub fn enable_depth_test(&self, enable: bool) {
        match enable {
            true => unsafe {
                self.context.enable(glow::DEPTH_TEST);
            },
            false => unsafe {
                self.context.disable(glow::DEPTH_TEST);
            },
        }
    }

    pub fn enable_backface_culling(&self, enable: bool) {
        match enable {
            true => unsafe {
                self.context.enable(glow::CULL_FACE);
            },
            false => unsafe {
                self.context.disable(glow::CULL_FACE);
            },
        }
    }

    fn get_context(&self) -> Rc<Context> {
        self.context.clone()
    }
}
