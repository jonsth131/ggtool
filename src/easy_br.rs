use std::io::{BufRead, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

pub trait EasyRead: ReadBytesExt + BufRead + Seek {
    #[inline]
    fn read_u32_le(&mut self) -> Result<u32, std::io::Error> {
        self.read_u32::<LittleEndian>()
    }

    fn read_cstring(&mut self) -> Result<String, std::io::Error> {
        let mut buf = Vec::new();
        let len = self.read_until(0, &mut buf)?;
        buf.resize(len - 1, 0);
        String::from_utf8(buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    #[inline]
    fn read_u16_le(&mut self) -> Result<u16, std::io::Error> {
        self.read_u16::<LittleEndian>()
    }

    fn read_at<F, T>(&mut self, seek: SeekFrom, f: F) -> Result<T, std::io::Error>
        where F: Fn(&mut Self) -> Result<T, std::io::Error> {
            let tell = self.seek(SeekFrom::Current(0))?;
            self.seek(seek)?;
            let res = f(self)?;
            self.seek(SeekFrom::Start(tell))?;
            Ok(res)
        }
}

/// All types that implement `Read` and `BufRead` get methods defined in `EasyRead`
/// for free.
impl<R: std::io::Read + BufRead + Seek + ?Sized> EasyRead for R {}