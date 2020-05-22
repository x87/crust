use crate::definitions;
use crate::parser;
use crate::types;
pub mod gta3;

pub enum Game {
    GTA3,
}

pub fn get_parser<'a>(
    game: &Game,
    chunk: &'a types::ScriptChunk,
    definitions: &'a definitions::DefinitionMap,
    base_offset: u32,
) -> Box<dyn parser::Parse<'a> + 'a> {
    match game {
        Game::GTA3 => Box::new(gta3::Parser3::new(chunk, definitions, base_offset)),
        _ => unimplemented!(),
    }
}
