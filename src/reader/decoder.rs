use binary_reader::{BinaryReader, Endian};
use std::fs::File;
use std::path::Path;

pub fn decode_data(data: &mut Vec<u8>) {
    let key1 = read_file("keys/key1.bin", 65536);
    let key2 = read_file("keys/key2.bin", 256);

    let mut xor_sum = (data.len() + 120) as u16;

    for c in data {
        *c ^= key1[xor_sum as usize] ^ key2[(((xor_sum as usize) + 120) as u8) as usize];
        xor_sum = xor_sum.wrapping_add(key2[(xor_sum as u8) as usize] as u16);
    }
}

pub fn open_file(file_name: &str) -> BinaryReader {
    let mut file = File::open(&Path::new(file_name)).unwrap();
    let mut reader = BinaryReader::from_file(&mut file);
    reader.set_endian(Endian::Little);
    return reader;
}

fn read_file(file_name: &str, size: usize) -> Vec<u8> {
    let mut reader = open_file(file_name);
    let data = reader.read(size).unwrap();
    return data.to_vec();
}
