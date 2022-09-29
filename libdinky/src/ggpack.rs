use crate::{
    decoder::{self, decode_data, decode_yack_data},
    directory::GGValue,
    easy_br::EasyRead,
    keys::Keys,
    ktx_decompress,
    yack::parse_yack,
};
use std::{
    fs::File,
    io::{BufReader, Seek, SeekFrom},
    path::Path,
};

pub struct OpenGGPack {
    reader: BufReader<File>,
    directory: GGValue,
    keys: Keys,
}

impl OpenGGPack {
    pub fn from_path(pack_path: &str) -> Result<Self, std::io::Error> {
        let keys = Keys::from_disk();

        let file = File::open(&Path::new(pack_path)).unwrap();
        let mut reader = BufReader::new(file);

        let offset = reader
            .read_u32_le()
            .expect("Failed to read directory offset") as u64;
        let size = reader.read_u32_le().expect("Failed to read directory size") as usize;

        reader.seek(SeekFrom::Start(offset))?;
        let mut directory_data = reader.read_bytes(size)?;
        decoder::decode_data(&mut directory_data, &keys.key1, &keys.key2);

        Ok(Self {
            reader,
            directory: GGValue::parse(directory_data).expect("Failed to parse directory"),
            keys,
        })
    }

    pub fn list_files(&self) {
        let file_list = self.directory.get_files();
        let filenames: Vec<&String> = file_list.iter().map(|f| f.filename).collect();

        println!("{}", serde_json::to_string_pretty(&filenames).unwrap());
    }

    pub fn extract_file(&mut self, filename: &str, outpath: &str) {
        let file_list = self.directory.get_files();

        let file = file_list
            .iter()
            .find(|f| f.filename.eq(&filename))
            .expect("File not found in ggpack");

        println!(
            "Extracting {}. Size = {}, offset = {}",
            file.filename, file.size, file.offset
        );

        self.reader
            .seek(SeekFrom::Start(file.offset))
            .expect("Failed to seek to offset");

        let mut data = self
            .reader
            .read_bytes(file.size)
            .expect("Failed to read data");

        if file.filename.ends_with(".bank") == false {
            decode_data(&mut data, &self.keys.key1, &self.keys.key2);
        }

        if file.filename.ends_with(".yack") {
            decode_yack_data(&mut data, &self.keys.key3, &file.filename);
            let yack_lines = parse_yack(&data).expect("Failed to parse yack data");
            for line in yack_lines {
                println!("{}", line);
            }
        }

        let final_path = format!("{}/{}", outpath, file.filename);

        if file.filename.ends_with(".json") || file.filename.ends_with(".wimpy") {
            let expanded = GGValue::parse(data).expect("Failed to expand file");

            std::fs::write(final_path, serde_json::to_string_pretty(&expanded).unwrap())
                .expect("Failed to write data to disk");
        } else if file.filename.ends_with(".ktxbz") || file.filename.ends_with(".ktxaz") {
            let decompressed =
                inflate::inflate_bytes_zlib(&data).expect("Failed to inflate compressed data");

            let mut output_buffer: Vec<u8> = Vec::new();
            ktx_decompress::decompress_gl::decompress_bptc(&decompressed, &mut output_buffer);

            std::fs::write(format!("{}.png", final_path), output_buffer)
                .expect("Failed to write data to disk");
        } else {
            std::fs::write(final_path, data).expect("Failed to write data to disk");
        }
    }
}
