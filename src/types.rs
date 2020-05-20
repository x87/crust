use itertools::Itertools;
use std::str;

// const PARAM_ANY: &str = "any";
pub const PARAM_ARGUMENTS: &str = "arguments";
pub const PARAM_OFFSET: &str = "label";
pub const INVALID_OPCODE: &str = "invalid";

pub type Opcode = u16;
pub type ScriptChunk = Vec<u8>;

#[derive(Debug)]
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

impl Instruction {
    pub fn print(&mut self) -> String
// where
    //     F: FnMut((usize, &Box<dyn InstructionParam>)) -> &dyn std::fmt::Display,
    {
        format!(
            "{{{:>0width$}}} {} {}",
            self.offset,
            self.name,
            self.params.iter().join(" "),
            width = 6
        )
    }
}
