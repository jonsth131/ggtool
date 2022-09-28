use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::easy_br::EasyRead;

#[derive(Debug, PartialEq)]
enum YackOpcode {
    End = 0,
    ActorSay = 1,
    EmitCode = 8,
    DefineLabel = 9,
    GotoLabel = 10,
    ChooseReply = 100,
    Unknown,
}
impl From<u8> for YackOpcode {
    fn from(what: u8) -> Self {
        match what {
            0 => Self::End,
            1 => Self::ActorSay,
            8 => Self::EmitCode,
            9 => Self::DefineLabel,
            10 => Self::GotoLabel,
            100 => Self::ChooseReply,
            _ => Self::Unknown,
        }
    }
}

pub fn parse_yack(data: &Vec<u8>) -> Result<Vec<String>, std::io::Error> {
    let mut reader = Cursor::new(data);
    let _code_start_marker = reader.read_u32_le()?;
    let string_table_offset = reader.read_u32_le()? as u64;

    let string_table = reader.read_at(std::io::SeekFrom::Start(string_table_offset), |reader| {
        let _string_table_start_marker = reader.read_u32_le()?;
        let num_strings = reader.read_u32_le()?;

        let mut strings = Vec::new();
        for _ in 0..num_strings {
            let str = reader.read_cstring()?;
            strings.push(str);
        }

        Ok(strings)
    })?;

    let mut lines = Vec::new();

    loop {
        let raw_opcode = reader.read_u8()?;
        let opcode = YackOpcode::from(raw_opcode);
        if opcode == YackOpcode::End {
            break;
        }

        let _sequence_number = reader.read_u64::<LittleEndian>()?; // Unused for now

        let num_conditions = reader.read_u8()?;

        let mut conditions = Vec::new();
        for _ in 0..num_conditions {
            let condition_index = reader.read_u32_le()?;
            conditions.push(string_table[condition_index as usize].clone());
        }

        let arg_indices = [reader.read_i32::<LittleEndian>()?, reader.read_i32::<LittleEndian>()?];        
        
        let get_arg = |index| {
            let str_index = arg_indices[index];
            if str_index == -1 {
                None
            } else {
                Some(string_table[str_index as usize].clone())
            }
        };

        let actor_say = || {
            let talker = get_arg(0).expect("Expected arg 0");
            let what = get_arg(1).expect("Expected arg 1");
            format!("{}: {}", talker, what)
        };
        let emit_code = || get_arg(0).expect("Expected arg 0");
        let define_label = || format!("\n=== {} ===", get_arg(0).expect("Expected arg 0"));
        let goto_state = || format!("-> {}", get_arg(0).expect("Expected arg 0"));
        let choose_reply = || {
            format!(
                "choice {} then goto {}",
                get_arg(0).expect("Expected arg 0"), get_arg(1).expect("Expected arg 1")
            )
        };

        let opcode_line = match opcode {
            YackOpcode::ActorSay => actor_say(),
            YackOpcode::EmitCode => emit_code(),
            YackOpcode::DefineLabel => define_label(),
            YackOpcode::GotoLabel => goto_state(),
            YackOpcode::ChooseReply => choose_reply(),
            YackOpcode::Unknown => format!("??Unknown opcode {} arg1={:?} arg2={:?}", raw_opcode, get_arg(0), get_arg(1)),
            YackOpcode::End => "end.".to_string(),
        };

        let and_conditions = conditions.join(" && ");
        if conditions.len() > 0 {
            lines.push(format!("if {} then {}", &and_conditions, opcode_line));
        } else {
            lines.push(opcode_line);
        };
    }

    Ok(lines)
}
