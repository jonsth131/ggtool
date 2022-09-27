use crate::easy_br::EasyRead;
use byteorder::ReadBytesExt;
use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize,
};
use std::{
    collections::HashMap,
    io::{Cursor, SeekFrom},
};

pub enum GGValueType {
    Dictionary = 2,
    List = 3,
    String = 4,
    Integer = 5,
    Float = 6,
    Coordinate = 9,
    CoordinateList = 10,
    Hotspot = 11,
}

impl From<u8> for GGValueType {
    fn from(a: u8) -> Self {
        match a {
            2 => GGValueType::Dictionary,
            3 => GGValueType::List,
            4 => GGValueType::String,
            5 => GGValueType::Integer,
            6 => GGValueType::Float,
            9 => GGValueType::Coordinate,
            10 => GGValueType::CoordinateList,
            11 => GGValueType::Hotspot,
            _ => panic!("{} is not a known GGValueType", a),
        }
    }
}

type IOResult<T> = Result<T, std::io::Error>;

pub enum GGValue {
    GGDict(HashMap<String, GGValue>),
    GGList(Vec<GGValue>),
    GGString(String),
    GGNumber(f32),
}

impl Serialize for GGValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            GGValue::GGDict(d) => {
                let mut map = serializer.serialize_map(Some(d.len()))?;
                for (k, v) in d {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            GGValue::GGList(l) => {
                let mut seq = serializer.serialize_seq(Some(l.len()))?;
                for element in l {
                    seq.serialize_element(element)?;
                }
                seq.end()
            }
            GGValue::GGString(s) => serializer.serialize_str(&s),
            GGValue::GGNumber(f) => serializer.serialize_f32(*f),
        }
    }
}

#[derive(Debug)]
pub struct File<'a> {
    pub filename: &'a String,
    pub size: usize,
    pub offset: u64,
}

impl GGValue {
    pub fn expect_dict(&self) -> &HashMap<String, GGValue> {
        match self {
            GGValue::GGDict(d) => d,
            _ => panic!("Expected dict"),
        }
    }
    pub fn expect_list(&self) -> &Vec<GGValue> {
        match self {
            GGValue::GGList(l) => l,
            _ => panic!("Expected list"),
        }
    }
    pub fn expect_string(&self) -> &String {
        match self {
            GGValue::GGString(s) => s,
            _ => panic!("Expected string"),
        }
    }
    pub fn expect_number(&self) -> &f32 {
        match self {
            GGValue::GGNumber(f) => f,
            _ => panic!("Expected int"),
        }
    }

    pub fn get_files<'a>(&'a self) -> Vec<File<'a>> {
        let rootdict = self.expect_dict();

        rootdict
            .get("files")
            .expect("files entry not found!")
            .expect_list()
            .iter()
            .map(|entry| {
                let entry_dict = entry.expect_dict();

                let filename = entry_dict
                    .get("filename")
                    .expect("filename entry not found!")
                    .expect_string();

                let offset = (*entry_dict
                    .get("offset")
                    .expect("offset entry not found!")
                    .expect_number()) as u64;
                let size = *entry_dict
                    .get("size")
                    .expect("size entry not found!")
                    .expect_number() as usize;
                File {
                    filename,
                    offset,
                    size,
                }
            })
            .collect()
    }

    pub fn parse(data: Vec<u8>) -> IOResult<Self> {
        let mut reader = Cursor::new(data);

        let magic = reader.read_u32_le()?;
        assert!(magic == 0x04030201, "Magic must be 01 02 03 04");

        let _num_tables = reader.read_u32_le()?; // Skip for now

        let offset_to_table = reader.read_u32_le()? as u64;

        let offsets = reader.read_at(SeekFrom::Start(offset_to_table), |reader| {
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

        let mut directory_builder = DirectoryBuilder { reader, offsets };
        directory_builder.read_ggvalue()
    }
}

struct DirectoryBuilder {
    reader: Cursor<Vec<u8>>,
    offsets: Vec<u32>,
}

impl DirectoryBuilder {
    fn read_table_entry(&mut self) -> IOResult<String> {
        let offset = self.reader.read_u16_le()? as usize;

        let str = self
            .reader
            .read_at(SeekFrom::Start(self.offsets[offset] as u64), |reader| {
                reader.read_cstring()
            })?;

        Ok(str)
    }

    fn read_dict(&mut self) -> IOResult<GGValue> {
        let mut dict = HashMap::new();
        let len = self.reader.read_u32_le()?;
        for _ in 0..len {
            let key = self.read_table_entry()?;
            let value = self.read_ggvalue()?;
            let _ = dict.insert(key, value);
        }

        let end_marker = self.reader.read_u8()?;
        assert!(end_marker == (GGValueType::Dictionary as u8));

        Ok(GGValue::GGDict(dict))
    }

    fn read_list(&mut self) -> IOResult<GGValue> {
        let mut list = Vec::new();
        let len = self.reader.read_u32_le()?;
        for _ in 0..len {
            let value = self.read_ggvalue()?;
            list.push(value);
        }

        let end_marker = self.reader.read_u8()?;
        assert!(end_marker == (GGValueType::List as u8));

        Ok(GGValue::GGList(list))
    }

    fn read_string(&mut self) -> IOResult<GGValue> {
        let entry = self.read_table_entry()?;
        Ok(GGValue::GGString(entry))
    }

    fn read_number(&mut self) -> IOResult<GGValue> {
        let entry = self.read_table_entry()?;
        let num: f32 = entry
            .parse()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok(GGValue::GGNumber(num))
    }

    fn read_ggvalue(&mut self) -> IOResult<GGValue> {
        let type_: GGValueType = self.reader.read_u8()?.into();
        match type_ {
            GGValueType::Dictionary => self.read_dict(),
            GGValueType::List => self.read_list(),
            GGValueType::String => self.read_string(),
            GGValueType::Integer => self.read_number(),
            GGValueType::Float => self.read_number(),
            GGValueType::Coordinate => self.read_string(),
            GGValueType::CoordinateList => self.read_string(),
            GGValueType::Hotspot => self.read_string(),
        }
    }
}
