pub trait KTXDecompressor {
    fn decompress_ktx(
        &self,
        data: &Vec<u8>,
        output_texture_data: &mut Vec<u8>,
    ) -> ();
}
