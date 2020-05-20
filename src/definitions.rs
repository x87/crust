extern crate multimap;
use crate::types;
use multimap::MultiMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

pub mod attribute {
    pub const SEGMENT: &str = "segment";
    pub const NAME: &str = "name";
    pub const BRANCH: &str = "branch";
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct CommandDefinition {
    pub id: types::Opcode,
    pub name: String,
    pub params: Vec<CommandDefinitionParam>,
    pub attrs: Vec<String>,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct CommandDefinitionParam {
    pub r#type: String,
}

pub struct DefinitionMap {
    ops: HashMap<types::Opcode, CommandDefinition>,
    attrs: MultiMap<String, CommandDefinition>,
}

impl DefinitionMap {
    pub fn find_by_op(&self, op: &types::Opcode) -> Option<&CommandDefinition> {
        self.ops.get(op)
    }

    pub fn find_all_by_attr(&self, attr: &str) -> Option<&Vec<CommandDefinition>> {
        self.attrs.get_vec(attr)
    }

    pub fn find_by_attr(&self, attr: &str) -> Option<&CommandDefinition> {
        match self.find_all_by_attr(attr) {
            Some(collection) => Some(&collection[0]),
            None => None,
        }
    }

    pub fn new() -> Self {
        let file_content = fs::read_to_string("def.json").expect("Can't read def.json");

        let mut data: Vec<CommandDefinition> =
            serde_json::from_str(&file_content).expect("Can't parse def.json");
        let mut map = DefinitionMap::empty();
        for c in data.drain(..) {
            for a in &c.attrs {
                map.attrs.insert(a.to_string(), c.clone());
            }
            map.ops.insert(c.id, c);
        }
        map
    }
    pub fn empty() -> Self {
        DefinitionMap {
            ops: HashMap::new(),
            attrs: MultiMap::new(),
        }
    }
    pub fn from_pairs(v: Vec<(types::Opcode, CommandDefinition)>) -> Self {
        let mut map = DefinitionMap::empty();

        for (op, c) in v {
            map.ops.insert(op, c);
        }
        map
    }
}
