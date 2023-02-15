use crate::{
    library::{Command, CommandParamType},
    types,
};
use std::collections::{self, HashMap};

pub struct Scanner {
    branch_ops: Vec<types::Opcode>,
}

impl<'a> Scanner {
    pub fn new(definitions: &'a HashMap<types::Opcode, Command>) -> Self {
        let branch_ops = definitions
            .iter()
            .filter(|(_id, c)| {
                c.input
                    .iter()
                    .find(|i| i.r#type == CommandParamType::Label)
                    .is_some()
            })
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        Self { branch_ops }
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
