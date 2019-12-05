use crate::loader::ScriptChunk;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::io::Read;
use std::{io, str};

// const PARAM_ANY: &str = "any";
pub const PARAM_ARGUMENTS: &str = "arguments";
pub const PARAM_LABEL: &str = "label";

#[derive(Serialize, Debug, Deserialize)]
pub struct CommandDefinition {
    pub id: String,
    pub name: String,
    pub params: Vec<CommandDefinitionParam>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct CommandDefinitionParam {
    pub r#type: String,
}

pub type Opcode = u16;
pub type GVar = u16;
pub type LVar = u16;

pub type DefinitionMap = HashMap<Opcode, CommandDefinition>;

#[derive(Debug)]
pub enum DataType3 {
    EOL,
    NUM8,
    NUM16,
    NUM32,
    FLOAT,
    STR8,
    GVAR,
    LVAR,
}

#[derive(Debug)]
pub struct Instruction<U> {
    pub opcode: Opcode,
    pub offset: u32,
    pub params: Vec<U>,
}

#[derive(Debug, Clone)]
pub enum InstructionParam3 {
    EOL,
    RAW(u8),
    // immediate values
    NUM8(i8),
    NUM16(i16),
    NUM32(i32),
    FLOAT(f32),
    STR(String),

    // variables
    GVARNUM32(GVar),
    LVARNUM32(LVar),
    // GVARSTR8,
    // LVARSTR8,
    // GVARSTR16,
    // LVARSTR16,

    // // arrays
    // GARRSTR8,
    // LARRSTR8,
    // GARRSTR16,
    // LARRSTR16,
    // GARRNUM32,
    // LARRNUM32,
}

pub trait DataTypeMeta {
    fn is_eol(&self) -> bool;
    fn has_data_type(&self) -> bool;
}

impl DataTypeMeta for DataType3 {
    fn is_eol(&self) -> bool {
        match self {
            DataType3::EOL => true,
            _ => false,
        }
    }

    fn has_data_type(&self) -> bool {
        match self {
            DataType3::STR8 => false,
            _ => true,
        }
    }
}

impl TryFrom<u8> for DataType3 {
    type Error = std::convert::Infallible;
    fn try_from(data_type: u8) -> Result<Self, Self::Error> {
        match data_type {
            0 => Ok(DataType3::EOL),
            1 => Ok(DataType3::NUM32),
            2 => Ok(DataType3::GVAR),
            3 => Ok(DataType3::LVAR),
            4 => Ok(DataType3::NUM8),
            5 => Ok(DataType3::NUM16),
            6 => Ok(DataType3::FLOAT),
            _ => Ok(DataType3::STR8),
        }
    }
}

impl TryInto<i32> for InstructionParam3 {
    type Error = String;
    fn try_into(self) -> Result<i32, Self::Error> {
        match self {
            InstructionParam3::NUM8(d) => Ok(i32::try_from(d).unwrap()),
            InstructionParam3::NUM16(d) => Ok(i32::try_from(d).unwrap()),
            InstructionParam3::NUM32(d) => Ok(i32::try_from(d).unwrap()),
            x => Err(format!("Can't convert {:#?} to i32", x)),
        }
    }
}

pub trait Reader<T, U> {
    fn get_raw(&self, cursor: &mut io::Cursor<&ScriptChunk>) -> Result<U, io::Error>;
    fn to_param(&self, cursor: &mut io::Cursor<&ScriptChunk>, data_type: T)
        -> Result<U, io::Error>;
}

pub struct Reader3 {}

impl Reader<DataType3, InstructionParam3> for Reader3 {
    fn get_raw(
        &self,
        cursor: &mut io::Cursor<&ScriptChunk>,
    ) -> Result<InstructionParam3, io::Error> {
        Ok(InstructionParam3::RAW(cursor.read_u8()?))
    }

    fn to_param<'a>(
        &self,
        cursor: &mut io::Cursor<&'a ScriptChunk>,
        data_type: DataType3,
    ) -> Result<InstructionParam3, io::Error> {
        match data_type {
            DataType3::EOL => Ok(InstructionParam3::EOL),
            // DataType3::RAW => Ok(InstructionParam3::RAW(cursor.read_u8()?)),
            DataType3::NUM8 => Ok(InstructionParam3::NUM8(cursor.read_i8()?)),
            DataType3::NUM16 => Ok(InstructionParam3::NUM16(cursor.read_i16::<LittleEndian>()?)),
            DataType3::NUM32 => Ok(InstructionParam3::NUM32(cursor.read_i32::<LittleEndian>()?)),
            DataType3::GVAR => Ok(InstructionParam3::GVARNUM32(
                cursor.read_u16::<LittleEndian>()?,
            )),
            DataType3::LVAR => Ok(InstructionParam3::LVARNUM32(
                cursor.read_u16::<LittleEndian>()?,
            )),
            DataType3::STR8 => {
                let mut buf = vec![0; 8];
                cursor.read_exact(buf.as_mut_slice())?;
                unsafe {
                    let s = str::from_utf8_unchecked(buf.as_mut_slice());
                    Ok(InstructionParam3::STR(
                        s.split(char::from(0)).next().unwrap().to_string(),
                    ))
                }
            }
            DataType3::FLOAT => Ok(InstructionParam3::FLOAT(
                f32::from(cursor.read_i16::<LittleEndian>()?) / 16.0,
            )),
        }
    }
}
