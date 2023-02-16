use itertools::Itertools;
use std::{str, fmt::{Formatter, Display}};

pub const INVALID_OPCODE: &str = "invalid";

pub type Opcode = u16;
pub type ScriptChunk = Vec<u8>;

#[derive(Debug, Copy, Clone)]
pub enum ScriptType {
    MAIN,
    MISSION,
    EXTERNAL,
}

pub trait InstructionParam: std::fmt::Debug + std::fmt::Display + Send {
    fn to_string(&self) -> Option<String>;
    fn to_offset(&self) -> Option<i32>;
}

pub struct Instruction {
    pub opcode: Opcode,
    pub name: String,
    pub offset: u32,
    pub params: Vec<Box<dyn InstructionParam>>,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{{:>0width$}}} {} {}",
            self.offset,
            self.name,
            self.params.iter().join(" "),
            width = 6
        )
    }
}
