use crate::platform;
use crate::types;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{collections, fs};

pub const ATTRIBUTE_GOTO: &str = "goto";
pub const ATTRIBUTE_NAME: &str = "name";

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct CommandDefinition {
    pub id: String,
    pub name: String,
    pub params: Vec<CommandDefinitionParam>,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct CommandDefinitionParam {
    pub r#type: String,
    pub attr: Vec<String>,
}

pub struct DefinitionMap(HashMap<types::Opcode, CommandDefinition>);

impl DefinitionMap {
    pub fn find_by_op(&self, op: &types::Opcode) -> Option<&CommandDefinition> {
        self.0.get(op)
    }
    pub fn find_by_attr(&self, attr: &str) -> Option<(&types::Opcode, &CommandDefinition)> {
        self.0
            .iter()
            .find(|(_, c)| c.params.len() > 0 && c.params[0].attr.contains(&attr.to_string()))
    }
    pub fn new() -> Self {
        let file_content = fs::read_to_string("def.json").expect("Can't read def.json");

        let mut data: Vec<CommandDefinition> =
            serde_json::from_str(&file_content).expect("Can't parse def.json");
        let mut map = HashMap::new();
        for c in data.drain(..) {
            let op = u16::from_str_radix(&c.id, 16)
                .expect(&format!("Unexpected opcode number {}", c.id));

            map.insert(op, c);
        }
        DefinitionMap { 0: map }
    }
    pub fn empty() -> Self {
        DefinitionMap { 0: HashMap::new() }
    }
    pub fn from_pairs(v: Vec<(types::Opcode, CommandDefinition)>) -> Self {
        let mut res = DefinitionMap::empty();

        for (op, c) in v {
            res.0.insert(op, c);
        }
        res
    }
}
