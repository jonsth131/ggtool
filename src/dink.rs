use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};

use crate::easy_br::EasyRead;

pub fn read_dink(data: &Vec<u8>) -> Result<(), std::io::Error> {
	let mut reader = Cursor::new(data);

	let start_marker = reader.read_u32_le()?;
	assert!(start_marker == 0x45_41_78_9C);

	let block_size = reader.read_u32_le()?;
	
	let header_start_marker = reader.read_u32_le()?;
	assert!(header_start_marker == 0x7F_46_A1_25);

	let _unk = reader.read_u32_le()?;
	let _unk2 = reader.read_u32_le()?;
	let _unk3 = reader.read_u16_le()?;

	let name_start_marker = reader.read_u32_le()?;
	assert!(name_start_marker == 0x16_F9_4B_62);

	let name_size = reader.read_u32_le()?;



	todo!()
}