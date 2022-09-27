use crate::easy_br::EasyRead;
use byteorder::ReadBytesExt;
use std::{
    collections::HashMap,
    io::{Cursor, SeekFrom},
};

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

type IOResult<T> = Result<T, std::io::Error>;

#[derive(Debug)]
pub enum GGValue {
    GGDict(HashMap<String, GGValue>),
    GGList(Vec<GGValue>),
    GGString(String),
    GGInt(u32),
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
    pub fn expect_int(&self) -> &u32 {
        match self {
            GGValue::GGInt(i) => i,
            _ => panic!("Expected int"),
        }
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

    fn read_integer(&mut self) -> IOResult<GGValue> {
        let entry = self.read_table_entry()?;
        let num: u32 = entry
            .parse()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok(GGValue::GGInt(num))
    }

    fn read_ggvalue(&mut self) -> IOResult<GGValue> {
        let type_: GGValueType = self.reader.read_u8()?.into();
        match type_ {
            GGValueType::Dictionary => self.read_dict(),
            GGValueType::List => self.read_list(),
            GGValueType::String => self.read_string(),
            GGValueType::Integer => self.read_integer(),
        }
    }
}

pub struct Directory {
    root: GGValue,
}

#[derive(Debug)]
pub struct File<'a> {
    pub filename: &'a String,
    pub size: usize,
    pub offset: u64
}

impl Directory {
    pub fn parse(data: Vec<u8>) -> IOResult<Directory> {
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
        Ok(Self {
            root: directory_builder.read_ggvalue()?,
        })
    }

    pub fn get_files<'a>(&'a self) -> Vec<File<'a>> {
        let rootdict = self.root.expect_dict();

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

                let offset = *entry_dict.get("offset").expect("offset entry not found!").expect_int() as u64;
                let size = *entry_dict.get("size").expect("size entry not found!").expect_int() as usize;
                File {
                    filename, 
                    offset, 
                    size
                }
            })
            .collect()        
    }
}
