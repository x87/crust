use crate::library::Command;
use crate::types;
use std::collections::HashMap;
use std::io::Cursor;

pub struct Parser<'a> {
    pub cursor: Cursor<&'a types::ScriptChunk>,
    pub definitions: &'a HashMap<types::Opcode, Command>,
    pub size: u32,
    pub base_offset: u32,
}

impl<'a> Parser<'a> {
    pub fn new(
        chunk: &'a types::ScriptChunk,
        definitions: &'a HashMap<types::Opcode, Command>,
        base_offset: u32,
    ) -> Self {
        Self {
            definitions,
            size: chunk.len() as u32,
            cursor: Cursor::new(chunk),
            base_offset,
        }
    }
    pub fn get_position(&self) -> u32 {
        self.cursor.position() as u32
    }

    pub fn set_position(&mut self, position: u32) {
        self.cursor.set_position(position as u64)
    }
}
pub trait Parse<'a>: Iterator<Item = Box<types::Instruction>> {
    fn get_parser_as_mut(&mut self) -> &mut Parser<'a>;
    fn get_parser(&self) -> &Parser<'a>;
}
