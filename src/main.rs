mod decoder;
mod directory;
mod easy_br;
use easy_br::EasyRead;
use std::{
    fs::File,
    io::{BufReader, Cursor, Read, Seek, SeekFrom},
    path::Path,
};

pub fn open_file(file_name: &str) -> BufReader<File> {
    let file = File::open(&Path::new(file_name)).unwrap();
    let reader = BufReader::new(file);
    return reader;
}

fn read_bytes(reader: &mut BufReader<File>, count: usize) -> Result<Vec<u8>, std::io::Error> {
    let mut buffer = vec![0; count];
    reader.read_exact(&mut buffer)?;
    Ok(buffer)
}

fn read_file(file_name: &str, size: usize) -> Result<Vec<u8>, std::io::Error> {
    let mut reader = open_file(file_name);
    read_bytes(&mut reader, size)
}

pub fn read_root(file_name: &str) -> Result<Vec<u8>, std::io::Error> {
    let key1 = read_file("keys/key1.bin", 65536)?;
    let key2 = read_file("keys/key2.bin", 256)?;

    let mut reader = open_file(file_name);
    let offset = reader.read_u32_le().unwrap();
    let size = reader.read_u32_le().unwrap();
    reader.seek(SeekFrom::Start(offset as u64))?;

    let mut data = read_bytes(&mut reader, size as usize)?;

    decoder::decode_data(&mut data, &key1, &key2);

    Ok(data)
}

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: ggtool <pack-file>");
        std::process::exit(1);
    }

    let pack_path = &args[1];

    let data = read_root(&pack_path)?;

    let mut reader = Cursor::new(data);
    let dict = directory::read_directory(&mut reader)?;
    println!("{:#?}", dict);

    Ok(())
}
