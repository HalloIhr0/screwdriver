use std::{cmp::min, mem, rc::Rc, slice};

use glow::{Context, HasContext, NativeBuffer, NativeVertexArray};

use super::Renderer;

#[allow(dead_code)]
#[derive(Clone)]
pub enum VertexSize {
    VEC1 = 1,
    VEC2 = 2,
    VEC3 = 3,
    VEC4 = 4,
}

pub struct VertexData {
    context: Rc<Context>,
    vao: NativeVertexArray,
    vbos: Vec<NativeBuffer>,
    vertex_count: Option<i32>,
}

impl VertexData {
    pub fn create(renderer: &Renderer) -> Result<Self, String> {
        let context = renderer.get_context();
        unsafe {
            let vao = context.create_vertex_array()?;
            Ok(Self {
                context,
                vao,
                vbos: vec![],
                vertex_count: None,
            })
        }
    }

    pub fn add_data(
        &mut self,
        data: &[f32],
        vertex_size: VertexSize,
        location: u32,
    ) -> Result<(), String> {
        let vertex_size = vertex_size as i32;
        let vertex_count = data.len() as i32 / vertex_size;
        match self.vertex_count {
            Some(x) => {
                if x != vertex_count {
                    eprintln!(
                        "Trying to add vertex data with different size ({} vs. {})",
                        x, vertex_count
                    );
                    self.vertex_count = Some(min(x, vertex_count));
                }
            }
            None => {
                self.vertex_count = Some(vertex_count);
            }
        }
        unsafe {
            self.context.bind_vertex_array(Some(self.vao));
            let vbo = self.context.create_buffer()?;
            self.context.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            self.context.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data)),
                glow::STATIC_DRAW,
            );
            self.context.vertex_attrib_pointer_f32(
                location,
                vertex_size,
                glow::FLOAT,
                false,
                vertex_size * (mem::size_of::<f32>() as i32),
                0,
            );
            self.context.enable_vertex_attrib_array(location);
            self.vbos.push(vbo);
            Ok(())
        }
    }

    pub(super) fn prepare_rendering(&self) -> i32 {
        unsafe {
            self.context.bind_vertex_array(Some(self.vao));
        }
        self.vertex_count.unwrap_or(0)
    }
}

impl Drop for VertexData {
    fn drop(&mut self) {
        unsafe {
            for vbo in &self.vbos {
                self.context.delete_buffer(*vbo);
            }
            self.context.delete_vertex_array(self.vao);
        }
    }
}
