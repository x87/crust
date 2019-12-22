use crate::definitions;
use crate::parser;
use crate::types;

use byteorder::{LittleEndian, ReadBytesExt};
use std::convert::TryFrom;
use std::io::Read;
use std::{fmt, io, str};

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
    GVAR(u16),
    LVAR(u16),
}

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
impl fmt::Display for InstructionParam3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstructionParam3::EOL => write!(f, ""),
            InstructionParam3::STR(d) => write!(f, "{}", d),
            InstructionParam3::NUM8(d) => write!(f, "{}", d),
            InstructionParam3::NUM16(d) => write!(f, "{}", d),
            InstructionParam3::NUM32(d) => write!(f, "{}", d),
            InstructionParam3::FLOAT(d) => write!(f, "{}", d),
            InstructionParam3::GVAR(d) => write!(f, "${}", d),
            InstructionParam3::LVAR(d) => write!(f, "{}@", d),
            InstructionParam3::RAW(d) => write!(f, "{:02X}", d),
        }
    }
}

impl types::InstructionParam for InstructionParam3 {
    fn to_string(&self) -> Option<String> {
        match self {
            InstructionParam3::STR(d) => Some(String::from(d)),
            _ => None,
        }
    }
    fn to_i32(&self) -> Option<i32> {
        match self {
            InstructionParam3::NUM8(d) => i32::try_from(*d).ok(),
            InstructionParam3::NUM16(d) => i32::try_from(*d).ok(),
            InstructionParam3::NUM32(d) => i32::try_from(*d).ok(),
            _ => None,
        }
    }
}

impl<'a> Iterator for Parser3<'a> {
    type Item = Box<types::Instruction<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.0.get_position();

        if self.0.size == offset {
            return None;
        }

        Some(Box::new(
            self.try_next(offset)
                .unwrap_or_else(|_| self.rollback(offset).unwrap()),
        ))
    }
}

pub struct Parser3<'a>(pub parser::Parser<'a>);

impl<'a> Parser3<'a> {
    pub fn get_raw(&mut self) -> Result<InstructionParam3, io::Error> {
        Ok(InstructionParam3::RAW(self.0.cursor.read_u8()?))
    }

    pub fn rollback(&mut self, offset: u32) -> Result<types::Instruction<'a>, io::Error> {
        self.0.set_position(offset);
        Ok(types::Instruction {
            opcode: 0xFFFF,
            name: &types::INVALID_OPCODE,
            offset,
            params: vec![Box::new(self.get_raw()?)],
        })
    }

    fn to_param(
        &mut self,
        data_type: DataType3,
    ) -> Result<Box<dyn types::InstructionParam>, io::Error> {
        match data_type {
            DataType3::EOL => Ok(Box::new(InstructionParam3::EOL)),
            // DataType3::RAW => Ok(InstructionParam3::RAW(cursor.read_u8()?)),
            DataType3::NUM8 => Ok(Box::new(InstructionParam3::NUM8(self.0.cursor.read_i8()?))),
            DataType3::NUM16 => Ok(Box::new(InstructionParam3::NUM16(
                self.0.cursor.read_i16::<LittleEndian>()?,
            ))),
            DataType3::NUM32 => Ok(Box::new(InstructionParam3::NUM32(
                self.0.cursor.read_i32::<LittleEndian>()?,
            ))),
            DataType3::GVAR => Ok(Box::new(InstructionParam3::GVAR(
                self.0.cursor.read_u16::<LittleEndian>()?,
            ))),
            DataType3::LVAR => Ok(Box::new(InstructionParam3::LVAR(
                self.0.cursor.read_u16::<LittleEndian>()?,
            ))),
            DataType3::STR8 => {
                let mut buf = vec![0; 8];
                self.0.cursor.read_exact(buf.as_mut_slice())?;
                unsafe {
                    let s = str::from_utf8_unchecked(buf.as_mut_slice());
                    Ok(Box::new(InstructionParam3::STR(
                        s.split(char::from(0)).next().unwrap().to_string(),
                    )))
                }
            }
            DataType3::FLOAT => Ok(Box::new(InstructionParam3::FLOAT(
                f32::from(self.0.cursor.read_i16::<LittleEndian>()?) / 16.0,
            ))),
        }
    }

    pub fn try_next(&mut self, offset: u32) -> Result<types::Instruction<'a>, io::Error> {
        let opcode = self.0.cursor.read_u16::<LittleEndian>()?;
        let def = self
            .0
            .definitions
            .find_by_op(&(opcode & 0x7FFF))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unknown opcode {} at {}", opcode, offset),
                )
            })?;
        let mut params = vec![];

        'outer: for param in &def.params {
            loop {
                let offset = self.0.get_position();
                let next_byte = self.0.cursor.read_u8()?;

                let data_type = DataType3::try_from(next_byte).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Unknown data type {} at {}", next_byte, offset),
                    )
                })?;

                if let DataType3::STR8 = data_type {
                    self.0.set_position(self.0.get_position() - 1);
                }

                if let DataType3::EOL = data_type {
                    if param.r#type != types::PARAM_ARGUMENTS {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Unexpected EOL parameter at {}", offset),
                        ));
                    }
                    break 'outer;
                }

                params.push(self.to_param(data_type)?);

                if param.r#type != types::PARAM_ARGUMENTS {
                    break;
                }
            }
        }

        Ok(types::Instruction {
            opcode,
            name: &def.name,
            offset,
            params,
        })
    }

    pub fn new(chunk: &'a types::ScriptChunk, definitions: &'a definitions::DefinitionMap) -> Self {
        Self {
            0: parser::Parser::new(chunk, definitions),
        }
    }
}

impl<'a> parser::Parse<'a> for Parser3<'a> {
    fn get_parser(&self) -> &parser::Parser<'a> {
        &self.0
    }
    fn get_parser_as_mut(&mut self) -> &mut parser::Parser<'a> {
        &mut self.0
    }
}
