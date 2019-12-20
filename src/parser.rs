use crate::definitions;
use crate::types;
use std::io;

pub struct Parser<'a> {
    pub cursor: io::Cursor<&'a types::ScriptChunk>,
    pub definitions: &'a definitions::DefinitionMap,
    pub size: u32,
}

impl<'a> Parser<'a> {
    pub fn new(chunk: &'a types::ScriptChunk, definitions: &'a definitions::DefinitionMap) -> Self {
        Self {
            definitions,
            size: chunk.len() as u32,
            cursor: io::Cursor::new(chunk),
        }
    }
    pub fn get_position(&self) -> u32 {
        self.cursor.position() as u32
    }

    pub fn set_position(&mut self, position: u32) {
        self.cursor.set_position(position as u64)
    }
}
pub trait Parse<'a>: Iterator<Item = Box<types::Instruction<'a>>> {
    fn get_parser_as_mut(&mut self) -> &mut Parser<'a>;
    fn get_parser(&self) -> &Parser<'a>;
}
