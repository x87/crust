use std::collections::HashMap;

use clap::ValueEnum;

use crate::library::Command;
use crate::parser;
use crate::types;
pub mod gta3;
pub mod vc;

#[derive(Debug, Clone, ValueEnum)]
pub enum Game {
    GTA3,
    VC
}

pub fn get_parser<'a>(
    game: &Game,
    chunk: &'a types::ScriptChunk,
    definitions: &'a HashMap<types::Opcode, Command>,
    base_offset: u32,
) -> Box<dyn parser::Parse<'a> + 'a> {
    match game {
        Game::GTA3 => Box::new(gta3::Parser3::new(chunk, definitions, base_offset)),
        Game::VC => Box::new(vc::ParserVC::new(chunk, definitions, base_offset)),
    }
}
