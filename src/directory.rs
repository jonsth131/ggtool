use byteorder::ReadBytesExt;
use std::{collections::HashMap, io::{Cursor, SeekFrom}};

use crate::easy_br::EasyRead;

pub enum GGValueType {
    Dictionary = 2,
    List = 3,
    String = 4,
    Integer = 5,
}

impl From<u8> for GGValueType {
    fn from(a: u8) -> Self {
        match a {
            2 => GGValueType::Dictionary,
            3 => GGValueType::List,
            4 => GGValueType::String,
            5 => GGValueType::Integer,
            _ => panic!("{} is not a known GGValueType", a),
        }
    }
}

#[derive(Debug)]
pub enum GGValue {
    GGDict(HashMap<String, GGValue>),
    GGList(Vec<GGValue>),
    GGString(String),
    GGInt(u32),
}

type GGOffsets = Vec<u32>;

fn read_table_entry(
    reader: &mut Cursor<Vec<u8>>,
    offsets: &GGOffsets,
) -> Result<String, std::io::Error> {
    let offset = reader.read_u16_le()? as usize;

    let str = reader.read_at(SeekFrom::Start(offsets[offset] as u64), |reader| {
        reader.read_cstring()
    })?;

    Ok(str)
}

fn read_dict(reader: &mut Cursor<Vec<u8>>, offsets: &GGOffsets) -> Result<GGValue, std::io::Error> {
    let mut dict = HashMap::new();
    let len = reader.read_u32_le()?;
    for _ in 0..len {
        let key = read_table_entry(reader, offsets)?;
        let value = read_ggvalue(reader, offsets)?;
        let _ = dict.insert(key, value);
    }

    let end_marker = reader.read_u8()?;
    assert!(end_marker == (GGValueType::Dictionary as u8));

    Ok(GGValue::GGDict(dict))
}

fn read_list(reader: &mut Cursor<Vec<u8>>, offsets: &GGOffsets) -> Result<GGValue, std::io::Error> {
    let mut list = Vec::new();
    let len = reader.read_u32_le()?;
    for _ in 0..len {
        let value = read_ggvalue(reader, offsets)?;
        list.push(value);
    }

    let end_marker = reader.read_u8()?;
    assert!(end_marker == (GGValueType::List as u8));

    Ok(GGValue::GGList(list))
}

fn read_string(
    reader: &mut Cursor<Vec<u8>>,
    offsets: &GGOffsets,
) -> Result<GGValue, std::io::Error> {
    let entry = read_table_entry(reader, offsets)?;
    Ok(GGValue::GGString(entry))
}

fn read_integer(
    reader: &mut Cursor<Vec<u8>>,
    offsets: &GGOffsets,
) -> Result<GGValue, std::io::Error> {
    let entry = read_table_entry(reader, offsets)?;
    let num: u32 = entry
        .parse()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Ok(GGValue::GGInt(num))
}

fn read_ggvalue(
    reader: &mut Cursor<Vec<u8>>,
    offsets: &GGOffsets,
) -> Result<GGValue, std::io::Error> {
    let type_: GGValueType = reader.read_u8()?.into();
    match type_ {
        GGValueType::Dictionary => read_dict(reader, offsets),
        GGValueType::List => read_list(reader, offsets),
        GGValueType::String => read_string(reader, offsets),
        GGValueType::Integer => read_integer(reader, offsets),
    }
}

pub fn read_directory(reader: &mut Cursor<Vec<u8>>) -> Result<GGValue, std::io::Error> {
    let magic = reader.read_u32_le()?;
    assert!(magic == 0x04030201, "Magic must be 01 02 03 04");

    let _num_tables = reader.read_u32_le()?; // Skip for now

    let offset_to_table = reader.read_u32_le()? as usize;

    let offsets = reader.read_at(SeekFrom::Start(offset_to_table as u64), |reader| {
        // This may be cheating but let's just do it for now
        let table_type = reader.read_u8()?;
        assert!(table_type == 7);

        let mut offsets = Vec::new();
        loop {
            let offset = reader.read_u32_le()?;
            if offset == 0xFF_FF_FF_FF {
                break;
            }

            offsets.push(offset);
        }

        Ok(offsets)
    })?;

    read_ggvalue(reader, &offsets)
}
