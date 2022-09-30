use crate::easy_br::EasyRead;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

#[derive(Debug, PartialEq)]
enum YackOpcode {
    ActorSay = 1,
    Assign = 2,
    Pause = 5,
    WaitFor = 7,
    EmitCode = 8,
    DefineLabel = 9,
    GotoLabel = 10,
    EndChoices = 11,
    StartChoices = 12,
    ElseGoto = 19,
    Reply1 = 100,
    Reply2 = 101,
    Reply3 = 102,
    Reply4 = 103,
    Reply5 = 104,
    Reply6 = 105,
    Reply7 = 106,
    Reply8 = 107,
    Reply9 = 108,
    Unknown,
}
impl From<u8> for YackOpcode {
    fn from(what: u8) -> Self {
        match what {
            1 => Self::ActorSay,
            2 => Self::Assign,
            5 => Self::Pause,
            7 => Self::WaitFor,
            8 => Self::EmitCode,
            9 => Self::DefineLabel,
            10 => Self::GotoLabel,
            11 => Self::EndChoices,
            12 => Self::StartChoices,
            19 => Self::ElseGoto,
            100 => Self::Reply1,
            101 => Self::Reply2,
            102 => Self::Reply3,
            103 => Self::Reply4,
            104 => Self::Reply5,
            105 => Self::Reply6,
            106 => Self::Reply7,
            107 => Self::Reply8,
            108 => Self::Reply9,
            _ => Self::Unknown,
        }
    }
}

pub fn parse_yack(data: &Vec<u8>) -> Result<String, std::io::Error> {
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

    let mut pending_else = false;
    let mut if_stack: Vec<String> = Vec::new();

    let mut outp = String::new();
    let mut indentation_level: u32 = 0;

    let mut emit = |indentation_level: u32, what: &str| {
        outp += &format!(
            "{}{}",
            (0..indentation_level).map(|_| { '\t' }).collect::<String>(),
            what
        );
    };

    loop {
        let raw_opcode = reader.read_u8()?;
        if raw_opcode == 0 {
            break;
        }

        let opcode = YackOpcode::from(raw_opcode);

        let _sequence_number = reader.read_u64::<LittleEndian>()?; // Unused for now

        let num_conditions = reader.read_u8()?;

        let mut conditions = Vec::new();
        for _ in 0..num_conditions {
            let condition_index = reader.read_u32_le()?;
            let condition = &string_table[condition_index as usize];
            if condition.starts_with("?") {
                conditions.push("once".to_string());
            } else {
                conditions.push(condition.clone());
            }
        }

        let arg_indices = [
            reader.read_i32::<LittleEndian>()?,
            reader.read_i32::<LittleEndian>()?,
        ];

        let get_arg = |index| {
            let str_index = arg_indices[index];
            if str_index == -1 {
                None
            } else {
                Some(string_table[str_index as usize].clone())
            }
        };

        if pending_else {
            if raw_opcode == 19 {
                indentation_level -= 1;
                emit(indentation_level, "else ");
            } else {
                indentation_level -= 1;
                emit(indentation_level, "endif\n");
            }
            pending_else = false;
        }

        let mut emit_dialogue_choice = |x| {
            let what = get_arg(0).expect("Expected arg 0");
            let goto = get_arg(1).expect("Expected arg 1");

            emit(indentation_level, &format!("{x} SAY({what}) -> {goto}"));
            if conditions.len() > 0 {
                let ored_conditions = conditions.join(" || ");
                emit(indentation_level, &format!("[{ored_conditions}]"));
            }
            emit(indentation_level, "\n");
        };

        match opcode {
            YackOpcode::ActorSay => {
                let talker = get_arg(0).expect("Expected arg 0");
                let what = get_arg(1).expect("Expected arg 1");
                emit(indentation_level, &format!("{talker}: SAY({what})\n"));
            }
            YackOpcode::Assign => {
                emit(
                    indentation_level,
                    &format!(
                        "{} <- {}",
                        get_arg(0).expect("Expected arg 0"),
                        get_arg(1).expect("Expected arg 1")
                    ),
                );
            }
            YackOpcode::Pause => {
                emit(
                    indentation_level,
                    &format!("pause {}", get_arg(0).expect("Expected arg 0")),
                );
            }
            YackOpcode::WaitFor => {
                let actor = get_arg(0).expect("Expected arg 0");
                emit(indentation_level, &format!("waitfor {actor}\n"));
            }
            YackOpcode::EmitCode => {
                let code = get_arg(0).expect("Expected arg 0");
                if conditions.len() > 0 {
                    let ored_conditions = conditions.join(" || ");
                    emit(indentation_level, &format!("if [{ored_conditions}]\n"));
                    pending_else = true;
                    indentation_level += 1;
                }
                emit(indentation_level, &format!("{code}\n"));
            }
            YackOpcode::DefineLabel => {
                let label = get_arg(0).expect("Expected arg 0");

                match if_stack.last() {
                    Some(s) if s.eq(&label) => {
                        if_stack.pop();
                        pending_else = true;
                    }
                    _ => {
                        emit(indentation_level, "\n");
                        emit(indentation_level, &format!("==={label}===\n"));
                    }
                };
            }
            YackOpcode::GotoLabel => {
                emit(
                    indentation_level,
                    &format!("-> {}\n", get_arg(0).expect("Expected arg 0")),
                );
            }
            YackOpcode::EndChoices => {
                indentation_level -= 1;
                emit(indentation_level, "end dialogue\n");
            }
            YackOpcode::StartChoices => {
                emit(indentation_level, "begin dialogue\n");
                indentation_level += 1;
            }
            YackOpcode::ElseGoto => {
                let goto_else = get_arg(0).expect("Expected arg 0");
                let ored_conditions = conditions.join(" || ");
                emit(indentation_level, &format!("if [{ored_conditions}]\n"));
                if_stack.push(goto_else);
                indentation_level += 1;
            }
            YackOpcode::Reply1 => emit_dialogue_choice(1),
            YackOpcode::Reply2 => emit_dialogue_choice(2),
            YackOpcode::Reply3 => emit_dialogue_choice(3),
            YackOpcode::Reply4 => emit_dialogue_choice(4),
            YackOpcode::Reply5 => emit_dialogue_choice(5),
            YackOpcode::Reply6 => emit_dialogue_choice(6),
            YackOpcode::Reply7 => emit_dialogue_choice(7),
            YackOpcode::Reply8 => emit_dialogue_choice(8),
            YackOpcode::Reply9 => emit_dialogue_choice(9),
            YackOpcode::Unknown => {
                emit(
                    indentation_level,
                    &format!(
                        "?? Unknown opcode {raw_opcode} cond={:?} arg1={:?} arg2={:?}\n",
                        &conditions,
                        get_arg(0),
                        get_arg(1)
                    ),
                );
            }
        }
    }

    Ok(outp)
}
