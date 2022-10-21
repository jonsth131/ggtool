use std::{fs::File, io::BufReader, path::Path};

use crate::easy_br::EasyRead;

enum DinkOpCode {
    Nop,
    PushConst,
    PushNull,
    PushLocal,
    PushUpvar,
    PushGlobal,
    PushFunction,
    PushVar,
    PushGlobalRef,
    PushLocalRef,
    PushUpVarRef,
    PushVarRef,
    PushIndexedRef,
    DupTop,
    Unot,
    Uminus,
    UoneComp,
    Math,
    Land,
    Lor,
    Index,
    Iterate,
    IterateKv,
    Call,
    Fcall,
    CallIndexed,
    CallNative,
    FcallNative,
    Pop,
    StoreLocal,
    StoreUpval,
    StoreRoot,
    StoreVar,
    StoreIndexed,
    SetLocal,
    NullLocal,
    MathRef,
    IncRef,
    DecRef,
    AddLocal,
    Jump,
    JumpTrue,
    JumpFalse,
    JumpTopTrue,
    JumpTopFalse,
    Ternary,
    NewTable,
    NewArray,
    NewSlot,
    NewThisSlot,
    DeleteSlot,
    Return,
    OClone,
    Breakpoint,
    Removed,
    Last,
    Label,
}

pub fn read_dink(file: &str) -> Result<Vec<String>, std::io::Error> {
    let file = File::open(&Path::new(file)).unwrap();
    let mut reader = BufReader::new(file);
    let mut result = vec![];

    let start_marker = reader.read_u32_le()?;
    //assert!(start_marker == 0x45_41_78_9C);

    let _block_size = reader.read_u32_le()?;

    let header_start_marker = reader.read_u32_le()?;
    assert!(header_start_marker == 0x7F_46_A1_25);

    let _unk = reader.read_u32_le()?;
    let _unk2 = reader.read_u32_le()?;
    let _unk3 = reader.read_u16_le()?;

    let name_start_marker = reader.read_u32_le()?;
    assert!(name_start_marker == 0x16_F9_4B_62);

    let _name_size = reader.read_u32_le()?;

    result.push(format!("{}", _name_size));

    return Ok(result);
}
