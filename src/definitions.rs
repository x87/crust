use crate::types;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{collections, fs};

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

pub type DefinitionMap = HashMap<types::Opcode, CommandDefinition>;

pub fn new_with_goto() -> collections::HashMap<u16, CommandDefinition> {
    let mut only_goto = collections::HashMap::new();
    only_goto.insert(
        2,
        CommandDefinition {
            id: String::from("0002"),
            name: String::from("goto"),
            params: vec![CommandDefinitionParam {
                r#type: String::from(types::PARAM_LABEL),
            }],
        },
    );
    only_goto
}

pub fn load() -> collections::HashMap<u16, CommandDefinition> {
    let file_content = fs::read_to_string("def.json").expect("Can't read def.json");

    let mut data: Vec<CommandDefinition> =
        serde_json::from_str(&file_content).expect("Can't parse def.json");

    let mut map = collections::HashMap::new();

    for c in data.drain(..) {
        let op =
            u16::from_str_radix(&c.id, 16).expect(&format!("Unexpected opcode number {}", c.id));
        map.insert(op, c);
    }
    map
}
