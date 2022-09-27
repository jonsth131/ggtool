use std::{fs::File, io::Read, io::Seek, io::SeekFrom, path::Path};

pub struct Keys {
    pub key1: Vec<u8>,
    pub key2: Vec<u8>,
}

pub fn read_keys(exe_path: &str) -> Result<Keys, std::io::Error> {
    let mut ef = File::open(&Path::new(exe_path)).unwrap();
    let mut exe_data = Vec::new();
    ef.seek(SeekFrom::Start(4000000))?;
    ef.read_to_end(&mut exe_data)?;
    let key1 = read_key(
        &exe_data,
        &[
            0xF7, 0xEC, 0x7E, 0xB6, 0xE3, 0x42, 0x5C, 0x36, 0x55, 0x5E, 0xA2, 0x97, 0xC0, 0x1E,
            0xBE, 0x2C,
        ],
        65536,
    )?;
    let key2 = read_key(
        &exe_data,
        &[
            0xD5, 0x7D, 0xFB, 0x4D, 0x51, 0xF5, 0x5E, 0xF4, 0xAA, 0x0B, 0x8A, 0x7E, 0x00, 0x8D,
            0xCB, 0x66,
        ],
        256,
    )?;

    Ok(Keys { key1, key2 })
}

fn read_key(exe_data: &[u8], key_data: &[u8], len: usize) -> Result<Vec<u8>, std::io::Error> {
    let pos = find_pos(exe_data, key_data).unwrap();
    let data = &exe_data[pos..pos + len];
    Ok(data.to_vec())
}

fn find_pos(data: &[u8], find: &[u8]) -> Option<usize> {
    data.windows(find.len()).position(|data| data == find)
}
