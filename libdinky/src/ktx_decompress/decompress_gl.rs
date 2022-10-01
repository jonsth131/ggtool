use glutin::PossiblyCurrent;
use ktx::KtxInfo;
use std::ffi::c_void;

use crate::ktx_decompressor::KTXDecompressor;

mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub struct OpenGLKTXDecompressor {
    _event_loop: glutin::event_loop::EventLoop<()>,
    _context: glutin::Context<PossiblyCurrent>,
}

impl OpenGLKTXDecompressor {
    pub fn new() -> Self {
        let event_loop = glutin::event_loop::EventLoop::new();
        let contextbuilder = glutin::ContextBuilder::new();
        let context = contextbuilder
            .build_headless(
                &event_loop,
                glutin::dpi::PhysicalSize {
                    width: 1,
                    height: 1,
                },
            )
            .expect("Failed to create OpenGL Context");

        unsafe {
            let current_context = context
                .make_current()
                .expect("Failed to make OpenGL context current");

            gl::load_with(|s| current_context.get_proc_address(s) as *const _);
            
            let mut gl_texture: u32 = 0;
            gl::Enable(gl::TEXTURE_2D);
            gl::GenTextures(1, &mut gl_texture);
            gl::BindTexture(gl::TEXTURE_2D, gl_texture);

            return Self {
                _event_loop: event_loop,
                _context: current_context,
            };
        }
    }

    fn gpu_decompress_texture(
        &self,
        ktx_texture_data: &Vec<u8>,
        target_texture_data: &mut Vec<u8>,
        width: u32,
        height: u32,
        gl_internal_format: u32,
    ) {
        unsafe {
            gl::CompressedTexImage2D(
                gl::TEXTURE_2D,
                0,
                gl_internal_format,
                width as gl::types::GLsizei,
                height as gl::types::GLsizei,
                0,
                ktx_texture_data.len() as gl::types::GLsizei,
                ktx_texture_data.as_ptr() as *const c_void,
            );

            gl::GetTexImage(
                gl::TEXTURE_2D,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                target_texture_data.as_mut_ptr() as *mut c_void,
            );
        }
    }
}

impl KTXDecompressor for OpenGLKTXDecompressor {
    fn decompress_ktx(&self, data: &Vec<u8>, output_texture_data: &mut Vec<u8>) -> () {
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
