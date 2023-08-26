use std::rc::Rc;

use bcndecode::{BcnDecoderFormat, BcnEncoding};
use glow::{Context, HasContext, NativeTexture};
use vtflib::{BoundVtfFile, ImageFormat};

use super::Renderer;

pub struct Texture {
    context: Rc<Context>,
    texture: NativeTexture,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn create_from_vtf(renderer: &Renderer, image: &BoundVtfFile) -> Result<Self, String> {
        let context = renderer.get_context();
        unsafe {
            let texture = context.create_texture()?;
            context.bind_texture(glow::TEXTURE_2D, Some(texture));
            context.texture_parameter_i32(texture, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            context.texture_parameter_i32(texture, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
            context.texture_parameter_i32(texture, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            context.texture_parameter_i32(texture, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);

            let data = image.data(0, 0, 0, 0).ok_or("Can' get image data")?;
            let (data, format, ty, internal_format) = get_image_data_format(
                image.format().ok_or("No Texture loaded")?,
                data,
                image.width(),
                image.height(),
            )?;
            context.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                internal_format as i32,
                image.width() as i32,
                image.height() as i32,
                0,
                format,
                ty,
                Some(&data),
            );
            context.generate_mipmap(glow::TEXTURE_2D);
            Ok(Self {
                context,
                texture,
                width: image.width(),
                height: image.height(),
            })
        }
    }

    pub(super) fn bind(&self) {
        unsafe {
            self.context
                .bind_texture(glow::TEXTURE_2D, Some(self.texture))
        };
    }

    // TODO: this shouldn't be pub, but is required for imgui
    pub fn get_id(&self) -> u32 {
        self.texture.0.get()
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { self.context.delete_texture(self.texture) };
    }
}

fn get_image_data_format(
    format: ImageFormat,
    data: &[u8],
    width: u32,
    height: u32,
) -> Result<(Vec<u8>, u32, u32, u32), String> {
    Ok(match format {
        ImageFormat::A8 => {
            let mut transformed_data = vec![];
            for alpha in data {
                transformed_data.push(0); // R
                transformed_data.push(0); // G
                transformed_data.push(0); // V
                transformed_data.push(*alpha); // A
            }
            (
                transformed_data,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::RGBA8,
            )
        }
        ImageFormat::Bgra8888 => (data.to_vec(), glow::BGRA, glow::UNSIGNED_BYTE, glow::RGB8),
        ImageFormat::Dxt1 => (
            bcndecode::decode(
                data,
                width as usize,
                height as usize,
                BcnEncoding::Bc1,
                BcnDecoderFormat::RGBA,
            )
            .or(Err("Can't decode DXT1"))?,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::RGBA8,
        ),
        ImageFormat::Dxt3 => (
            bcndecode::decode(
                data,
                width as usize,
                height as usize,
                BcnEncoding::Bc2,
                BcnDecoderFormat::RGBA,
            )
            .or(Err("Can't decode DXT1"))?,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::RGBA8,
        ),
        ImageFormat::Dxt5 => (
            bcndecode::decode(
                data,
                width as usize,
                height as usize,
                BcnEncoding::Bc3,
                BcnDecoderFormat::RGBA,
            )
            .or(Err("Can't decode DXT1"))?,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::RGBA8,
        ),
        x => return Err(format!("Unsupported VTF format {:?}", x)),
    })
}
