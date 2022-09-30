use ktx::KtxInfo;
use std::ffi::c_void;
use surfman::{ContextAttributeFlags, ContextAttributes, GLVersion};
mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

fn gpu_decompress_texture(
    ktx_texture_data: &Vec<u8>,
    target_texture_data: &mut Vec<u8>,
    width: u32,
    height: u32,
    gl_internal_format: u32,
) {
    let connection = surfman::Connection::new().expect("Failed to create surfman connection");
    let adapter = connection
        .create_adapter()
        .expect("Failed to create surfman adapter");
    let mut device = connection
        .create_device(&adapter)
        .expect("Failed to create surfman device");

    let context_attributes = ContextAttributes {
        version: GLVersion::new(4, 6),
        flags: ContextAttributeFlags::empty(),
    };
    let context_descriptor = device
        .create_context_descriptor(&context_attributes)
        .expect("Failed to create surfman context descriptor");

    let mut context = device
        .create_context(&context_descriptor, None)
        .expect("Failed to create GL context");
    device
        .make_context_current(&context)
        .expect("Failed to make GL context current");

    gl::load_with(|s| device.get_proc_address(&context, s) as *const _);

    unsafe {
        let mut gl_texture: u32 = 0;
        gl::Enable(gl::TEXTURE_2D);
        gl::GenTextures(1, &mut gl_texture);
        gl::BindTexture(gl::TEXTURE_2D, gl_texture);
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

    device
        .destroy_context(&mut context)
        .expect("Failed to destroy GL context");
}

pub fn decompress_bptc(data: &Vec<u8>, output_buffer: &mut Vec<u8>) {
    let cursor = std::io::Cursor::new(&data);
    let decoder = ktx::Decoder::new(cursor).expect("Failed to create KTX decoder");

    let width = decoder.pixel_width();
    let height = decoder.pixel_height();
    let gl_internal_format = decoder.gl_internal_format();

    let textures: Vec<Vec<u8>> = decoder.read_textures().take(1).collect();
    let ktx_texture_data = &textures[0];

    let mut target_texture_data: Vec<u8> = Vec::new();
    target_texture_data.resize((width * height * 4) as usize, 0);

    gpu_decompress_texture(
        ktx_texture_data,
        &mut target_texture_data,
        width,
        height,
        gl_internal_format,
    );

    let mut encoder = png::Encoder::new(output_buffer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("Failed to write PNG header");
    writer
        .write_image_data(&target_texture_data)
        .expect("Failed to write PNG data");
}
