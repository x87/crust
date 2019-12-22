use itertools::Itertools;
use std::str;

// const PARAM_ANY: &str = "any";
pub const PARAM_ARGUMENTS: &str = "arguments";
pub const PARAM_LABEL: &str = "label";
pub const INVALID_OPCODE: &str = "invalid";

pub type Opcode = u16;
pub type ScriptChunk = Vec<u8>;

pub trait InstructionParam: std::fmt::Debug + std::fmt::Display {
    fn to_string(&self) -> Option<String>;
    fn to_i32(&self) -> Option<i32>;
}

pub struct Instruction<'a> {
    pub opcode: Opcode,
    pub name: &'a str,
    pub offset: u32,
    pub params: Vec<Box<dyn InstructionParam>>,
}

impl<'a> Instruction<'a> {
    pub fn print(&mut self) -> String {
        let params: String = self.params.iter().join(" ");
        format!(
            "{:>0width$}: {} {}",
            self.offset,
            self.name,
            params,
            width = 6
        )
    }
}
