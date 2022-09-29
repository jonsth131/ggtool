use std::ffi::c_void;

use glutin::dpi::PhysicalSize;
use ktx::KtxInfo;
mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub fn decompress_bptc(data: &Vec<u8>, output_buffer: &mut Vec<u8>) {
    let el = glutin::event_loop::EventLoop::new();
    let context = glutin::ContextBuilder::new()
        .build_headless(
            &el,
            PhysicalSize {
                width: 800,
                height: 600,
            },
        )
        .unwrap();

    let current = unsafe { context.make_current().unwrap() };

    gl::load_with(|s| current.get_proc_address(s) as *const _);

    let cursor = std::io::Cursor::new(&data);
    let decoder = ktx::Decoder::new(cursor).expect("Failed to create KTX decoder");
    let width = decoder.pixel_width();
    let height = decoder.pixel_height();
    let depth = decoder.pixel_depth();
    let gl_internal_format = decoder.gl_internal_format();

    println!(
        "Image size {}x{}, depth {}, gl_internal_format {}",
        width, height, depth, gl_internal_format
    );

    let textures: Vec<Vec<u8>> = decoder.read_textures().take(1).collect();
    let texture = &textures[0];

    let mut target_texture: Vec<u8> = Vec::new();
    target_texture.resize((width * height * 4) as usize, 0);

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
            texture.len() as gl::types::GLsizei,
            texture.as_ptr() as *const c_void,
        );

        gl::GetTexImage(
            gl::TEXTURE_2D,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            target_texture.as_mut_ptr() as *mut c_void,
        );
    }

    let mut encoder = png::Encoder::new(output_buffer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("Failed to write PNG header");
    writer
        .write_image_data(&target_texture)
        .expect("Failed to write PNG data");
}
