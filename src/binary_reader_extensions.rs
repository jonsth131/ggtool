use binary_reader::BinaryReader;

pub trait ReadAt {
    fn read_at<T, F>(&mut self, offset: usize, f: F) -> Result<T, std::io::Error>
    where
        F: Fn(&mut BinaryReader) -> Result<T, std::io::Error>;
}

impl ReadAt for BinaryReader {
    fn read_at<T, F>(&mut self, offset: usize, f: F) -> Result<T, std::io::Error>
    where
        F: Fn(&mut BinaryReader) -> Result<T, std::io::Error>,
    {
        let tell = self.pos;
        self.jmp(offset);
        let result = f(self)?;
        self.jmp(tell);
        return Ok(result);
    }
}
