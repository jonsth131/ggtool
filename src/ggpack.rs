mod directory;
mod decoder;
use std::{io::{BufReader, Seek, SeekFrom, Read}, fs::File, path::Path};
use crate::{keys::Keys, easy_br::EasyRead};
use self::directory::GGValue;
use self::decoder::{decode_data, decode_yack_data};


struct OpenGGPack {
    reader: BufReader<File>,
    directory: GGValue,
    keys: Keys,
}

fn decode_at(
    reader: &mut BufReader<File>,
    keys: &Keys,
    offset: u64,
    size: usize,
) -> Result<Vec<u8>, std::io::Error> {
    reader.seek(SeekFrom::Start(offset))?;
    let mut data = read_bytes(reader, size)?;
    decoder::decode_data(&mut data, &keys.key1, &keys.key2);
    Ok(data)
}

fn read_bytes(reader: &mut BufReader<File>, count: usize) -> Result<Vec<u8>, std::io::Error> {
    let mut buffer = vec![0; count];
    reader.read_exact(&mut buffer)?;
    Ok(buffer)
}

fn open_ggpack(pack_path: &str) -> OpenGGPack {
    let keys = Keys::from_disk();

    let file = File::open(&Path::new(pack_path)).unwrap();
    let mut reader = BufReader::new(file);

    let offset = reader
        .read_u32_le()
        .expect("Failed to read directory offset") as u64;
    let size = reader.read_u32_le().expect("Failed to read directory size") as usize;

    let directory_data =
        decode_at(&mut reader, &keys, offset, size).expect("Failed to decode directory");

    OpenGGPack {
        reader,
        directory: GGValue::parse(directory_data).expect("Failed to parse directory"),
        keys,
    }
}

pub fn list_files(pack_path: &str) {
    let ggpack = open_ggpack(pack_path);
    let file_list = ggpack.directory.get_files();
    let filenames: Vec<&String> = file_list.iter().map(|f| f.filename).collect();

    println!("{}", serde_json::to_string_pretty(&filenames).unwrap());
}

pub fn extract_file(pack_path: &str, filename: &str, outpath: &str) {
    let mut ggpack = open_ggpack(pack_path);
    let file_list = ggpack.directory.get_files();

    let file = file_list
        .iter()
        .find(|f| f.filename.eq(&filename))
        .expect("File not found in ggpack");

    println!(
        "Extracting {}. Size = {}, offset = {}",
        file.filename, file.size, file.offset
    );

    ggpack
        .reader
        .seek(SeekFrom::Start(file.offset))
        .expect("Failed to seek to offset");

    let mut data = read_bytes(&mut ggpack.reader, file.size).expect("Failed to read data");

    decode_data(&mut data, &ggpack.keys.key1, &ggpack.keys.key2);

    if file.filename.ends_with(".yack") {
        decode_yack_data(&mut data, &ggpack.keys.key3, &file.filename);
    }

    let final_path = format!("{}/{}", outpath, file.filename);

    if file.filename.ends_with(".json") || file.filename.ends_with(".wimpy") {
        let expanded = GGValue::parse(data).expect("Failed to expand file");

        std::fs::write(final_path, serde_json::to_string_pretty(&expanded).unwrap())
            .expect("Failed to write data to disk");
    } else if file.filename.ends_with(".ktxbz") || file.filename.ends_with(".ktxaz") {
        let decompressed =
            inflate::inflate_bytes_zlib(&data).expect("Failed to inflate compressed data");
        std::fs::write(final_path, decompressed).expect("Failed to write data to disk");
    } else {
        std::fs::write(final_path, data).expect("Failed to write data to disk");
    }
}