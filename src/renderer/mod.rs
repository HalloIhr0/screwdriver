use std::rc::Rc;
use glow::Context;

mod shader;
pub use shader::Shader;

pub struct Renderer {
    context: Rc<Context>
}

impl Renderer {
    fn get_context(&self) -> Rc<Context> {
        self.context.clone()
    }
}