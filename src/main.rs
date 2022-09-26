mod binary_reader_extensions;

use binary_reader::{BinaryReader, Endian};
use binary_reader_extensions::ReadAt;
use std::fs::File;
use std::path::Path;

type Metadata = Vec<(String, String)>;

fn read_metadata(data: &Vec<u8>) -> Result<Metadata, std::io::Error> {
    let mut br = BinaryReader::from_vec(data);
    br.set_endian(Endian::Little);
    let magic = br.read_u32()?;
    assert!(
        magic == 0x04030201,
        "decryption magic signature wasn't 01 02 03 04"
    );
    let _ = br.read_u32()?; // Unk
    let table_offset = br.read_u32()?;

    // Jump to offset table
    br.jmp(table_offset as usize);
    let _ = br.read_u8()?; // Unk, always 7?
    let _files_offset = br.read_u32()?; // Always points to "files"?

    let mut meta: Vec<(String, String)> = Vec::new();

    loop {
        let key_offset = br.read_u32()?;
        if key_offset == 0xFF_FF_FF_FF {
            break;
        }
        let key = br.read_at(key_offset as usize, |br| br.read_cstr())?;

        let value_offset = br.read_u32()?;
        let value = br.read_at(value_offset as usize, |br| br.read_cstr())?;

        meta.push((key, value));
    }

    Ok(meta)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: ggtool <pack-file>");
        std::process::exit(1);
    }

    let pack_path = &args[1];

    let mut data = read_root(&pack_path);
    decode_data(&mut data);
    std::fs::write("out/blah.bin", &data).unwrap();

    let metatable = read_metadata(&data).expect("Failed to parse file metadata");
    for (key, value) in metatable {
        println!("{} = {}", key, value);
    }
}

fn read_root(file_name: &str) -> Vec<u8> {
    let mut reader = open_file(file_name);
    let offset = reader.read_u32().unwrap();
    let size = reader.read_u32().unwrap();
    reader.jmp(offset as usize);
    let data = reader.read(size as usize).unwrap();
    return data.to_vec();
}

fn read_file(file_name: &str, size: usize) -> Vec<u8> {
    let mut reader = open_file(file_name);
    let data = reader.read(size).unwrap();
    return data.to_vec();
}

fn decode_data(data: &mut Vec<u8>) {
    let key1 = read_file("keys/key1.bin", 65536);
    let key2 = read_file("keys/key2.bin", 256);

    let mut xor_sum = (data.len() + 120) as u16;

    for c in data {
        *c ^= key1[xor_sum as usize] ^ key2[(((xor_sum as usize) + 120) as u8) as usize];
        xor_sum = xor_sum.wrapping_add(key2[(xor_sum as u8) as usize] as u16);
    }
}

fn open_file(file_name: &str) -> BinaryReader {
    let mut file = File::open(&Path::new(file_name)).unwrap();
    let mut reader = BinaryReader::from_file(&mut file);
    reader.set_endian(Endian::Little);
    return reader;
}
