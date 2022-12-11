use glfw::Context;
use glow::{HasContext, RGBA, TEXTURE_2D, UNSIGNED_BYTE};
use ktx::KtxInfo;

use crate::ktx_decompressor::KTXDecompressor;

pub struct OpenGLKTXDecompressor {
    gl: glow::Context,
    window: glfw::Window,
}

impl OpenGLKTXDecompressor {
    pub fn new() -> Self {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        let (mut window, _event_receiver) = glfw
            .create_window(640, 480, "KTX Decompressor", glfw::WindowMode::Windowed)
            .unwrap();
        window.make_current();

        unsafe {
            let gl = glow::Context::from_loader_function(|s| window.get_proc_address(s));

            use glow::*;

            gl.enable(TEXTURE_2D);
            let gl_texture = gl.create_texture().expect("Failed to create texture");
            gl.bind_texture(TEXTURE_2D, Some(gl_texture));

            Self { gl, window }
        }
    }

    fn gpu_decompress_texture(
        &self,
        ktx_texture_data: &Vec<u8>,
        target_texture_data: &mut [u8],
        width: u32,
        height: u32,
        gl_internal_format: u32,
    ) {
        unsafe {
            self.gl.compressed_tex_image_2d(
                TEXTURE_2D,
                0,
                gl_internal_format as i32,
                width as i32,
                height as i32,
                0,
                ktx_texture_data.len() as i32,
                ktx_texture_data,
            );

            self.gl.get_tex_image(
                TEXTURE_2D,
                0,
                RGBA,
                UNSIGNED_BYTE,
                glow::PixelPackData::Slice(target_texture_data),
            );
        }
    }
}

impl KTXDecompressor for OpenGLKTXDecompressor {
    fn decompress_ktx(&self, data: &Vec<u8>, output_texture_data: &mut Vec<u8>) {
        let cursor = std::io::Cursor::new(&data);
        let decoder = ktx::Decoder::new(cursor).expect("Failed to create KTX decoder");

        let width = decoder.pixel_width();
        let height = decoder.pixel_height();
        let gl_internal_format = decoder.gl_internal_format();

        let textures: Vec<Vec<u8>> = decoder.read_textures().take(1).collect();
        let ktx_texture_data = &textures[0];

        let mut target_texture_data: Vec<u8> = Vec::new();
        target_texture_data.resize((width * height * 4) as usize, 0);

        self.gpu_decompress_texture(
            ktx_texture_data,
            &mut target_texture_data,
            width,
            height,
            gl_internal_format,
        );

        let mut encoder = png::Encoder::new(output_texture_data, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("Failed to write PNG header");
        writer
            .write_image_data(&target_texture_data)
            .expect("Failed to write PNG data");
    }
}
