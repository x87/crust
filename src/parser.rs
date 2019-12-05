use crate::data_type::{DataTypeMeta, DefinitionMap, Instruction, Reader, PARAM_ARGUMENTS};
use crate::loader::ScriptChunk;
use byteorder::{LittleEndian, ReadBytesExt};
use std::convert::{TryFrom, TryInto};
use std::io;

pub struct Parser<'a, T, U> {
    cursor: io::Cursor<&'a ScriptChunk>,
    definitions: &'a DefinitionMap,
    reader: Box<dyn Reader<T, U>>,
    size: u32,
}

impl<'a, T, U> Parser<'a, T, U>
where
    T: TryFrom<u8> + DataTypeMeta,
    U: TryInto<i32> + Clone,
{
    pub fn get_position(&self) -> u32 {
        self.cursor.position() as u32
    }

    pub fn set_position(&mut self, position: u32) {
        self.cursor.set_position(position as u64)
    }

    fn rollback(&mut self, offset: u32) -> Instruction<U> {
        self.cursor.set_position(offset as u64);
        Instruction {
            opcode: 0xFFFF,
            offset,
            params: vec![self.reader.get_raw(&mut self.cursor).unwrap()],
        }
    }

    fn try_next(&mut self, offset: u32) -> Result<Instruction<U>, io::Error> {
        let opcode = self.cursor.read_u16::<LittleEndian>()?;
        let def = self.definitions.get(&(opcode & 0x7FFF)).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown opcode {} at {}", opcode, offset),
            )
        })?;
        let mut params = vec![];

        'outer: for param in &def.params {
            loop {
                let offset = self.cursor.position();
                let next_byte = self.cursor.read_u8()?;

                let data_type = T::try_from(next_byte).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Unknown data type {} at {}", next_byte, offset),
                    )
                })?;

                if !data_type.has_data_type() {
                    self.cursor.set_position(self.cursor.position() - 1);
                }

                if data_type.is_eol() {
                    if param.r#type != PARAM_ARGUMENTS {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, ""));
                    }
                    break 'outer;
                }

                params.push(self.reader.to_param(&mut self.cursor, data_type)?);

                if param.r#type != PARAM_ARGUMENTS {
                    break;
                }
            }
        }

        Ok(Instruction {
            opcode,
            offset,
            params,
        })
    }

    pub fn new(
        chunk: &'a ScriptChunk,
        definitions: &'a DefinitionMap,
        reader: Box<dyn Reader<T, U>>,
    ) -> Self {
        Self {
            reader,
            definitions,
            size: chunk.len() as u32,
            cursor: io::Cursor::new(chunk),
        }
    }
}

impl<T, U> Iterator for Parser<'_, T, U>
where
    T: TryFrom<u8> + DataTypeMeta,
    U: TryInto<i32> + Clone,
{
    type Item = Instruction<U>;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.cursor.position() as u32;

        if self.size == offset {
            return None;
        }

        Some(
            self.try_next(offset)
                .unwrap_or_else(|_| self.rollback(offset)),
        )
    }
}
