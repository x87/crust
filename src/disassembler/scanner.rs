use crate::definitions;
use crate::types;
use std::collections;

pub struct Scanner {
    // definitions: &'a definitions::DefinitionMap,
    branch_ops: Vec<types::Opcode>,
}

impl<'a> Scanner {
    pub fn new(definitions: &'a definitions::DefinitionMap) -> Self {
        let branch_ops: Vec<u16> = definitions
            .find_all_by_attr(definitions::attribute::BRANCH)
            .expect(&format!(
                "Can't find a command with attribute {}",
                definitions::attribute::BRANCH
            ))
            .iter()
            .map(|c| c.id)
            .collect();
        Self {
            // definitions,
            branch_ops,
        }
    }
    pub fn collect_global_addresses(
        &self,
        instructions: &Vec<Box<types::Instruction>>,
    ) -> collections::HashSet<i32> {
        let mut res: collections::HashSet<i32> = collections::HashSet::new();

        for i in instructions {
            if self.branch_ops.contains(&i.opcode) {
                match i.params[0].to_offset() {
                    Some(x) if x >= 0 => {
                        res.insert(x);
                    }
                    _ => continue,
                }
            }
        }
        res
    }

    pub fn collect_relative_addresses(
        &self,
        instructions: &Vec<Box<types::Instruction>>,
    ) -> collections::HashSet<i32> {
        let mut res: collections::HashSet<i32> = collections::HashSet::new();

        for i in instructions {
            if self.branch_ops.contains(&i.opcode) {
                match i.params[0].to_offset() {
                    Some(x) if x < 0 => {
                        res.insert(x);
                    }
                    _ => continue,
                }
            }
        }
        res
    }
}
