mod decoder;

use binary_reader::{BinaryReader, Endian};
use decoder::{decode_data, open_file};

pub fn read_root(file_name: &str) -> Vec<u8> {
    let mut reader = open_file(file_name);
    let offset = reader.read_u32().unwrap();
    let size = reader.read_u32().unwrap();
    reader.jmp(offset as usize);
    let mut data = reader.read(size as usize).unwrap().to_vec();
    decode_data(&mut data);
    return data.to_vec();
}

pub fn read_metadata(data: &Vec<u8>) -> Result<Vec<String>, std::io::Error> {
    let mut br = BinaryReader::from_vec(data);
    br.set_endian(Endian::Little);
    let magic = br.read_u32()?;
    assert!(
        magic == 0x04030201,
        "decryption magic signature wasn't 01 02 03 04"
    );
    let _ = br.read_u32()?; // Unk
    let table_offset = br.read_u32()?;
    let file_structure = br.read_bytes(table_offset as usize - 12)?;

    println!("{:?}", file_structure);

    let components = read_components(&mut br, table_offset)?;

    Ok(components)
}

fn read_components(br: &mut BinaryReader, offset: u32) -> Result<Vec<String>, std::io::Error> {
    br.jmp(offset as usize);
    let offset_table_id = br.read_u8()?; // offset table id should be 7
    assert!(offset_table_id == 7, "invalid offset table id");

    let mut offsets: Vec<u32> = Vec::new();

    loop {
        let offset = br.read_u32()?;
        if offset == 0xFF_FF_FF_FF {
            break;
        }
        offsets.push(offset);
    }

    let mut components: Vec<String> = Vec::new();

    for offset in offsets {
        br.jmp(offset as usize);
        let value = br.read_cstr()?;
        components.push(value);
    }

    println!("{:?}", components);

    Ok(components)
}
