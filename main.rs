use binary_reader::{BinaryReader, Endian};
use std::fs;
use std::fs::File;
use std::path::Path;

fn main() {
    let data = read_root("files/Weird.ggpack1a");
    let decoded = decode_data(data);
    fs::write("out/test.dat", decoded).unwrap();
}

fn read_root(file_name: &str) -> Vec<u8> {
    let mut reader = open_file(file_name);
    let offset = reader.read_u32().unwrap();
    let size = reader.read_u32().unwrap();
    reader.jmp(offset as usize);
    let data = reader.read(size as usize).unwrap();
    return data.to_vec();
}

fn read_file(file_name: &str, size: u32) -> Vec<u8> {
    let mut reader = open_file(file_name);
    let data = reader.read(size.try_into().unwrap()).unwrap();
    return data.to_vec();
}

fn decode_data(data: Vec<u8>) -> Vec<u8> {
    let key1 = read_file("keys/key1.bin", 65536);
    let key2 = read_file("keys/key2.bin", 256);

    let mut xor_sum = (data.len() + 120) as u16;
    let mut decoded: Vec<u8> = vec![];

    for i in 0..data.len() {
        let val = data[i] ^ key1[xor_sum as usize] ^ key2[((xor_sum + 120) as u8) as usize];
        xor_sum = xor_sum.wrapping_add(key2[(xor_sum as u8) as usize] as u16);
        decoded.push(val);
    }

    return decoded;
}

fn open_file(file_name: &str) -> BinaryReader {
    let mut file = File::open(&Path::new(file_name)).unwrap();
    let mut reader = BinaryReader::from_file(&mut file);
    reader.set_endian(Endian::Little);
    return reader;
}
