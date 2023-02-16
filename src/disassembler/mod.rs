pub mod scanner;

use crate::library::Command;
use crate::types;
use crate::types::*;

use std::collections::HashMap;
use std::convert::TryInto;
use std::io::prelude::*;
use std::{collections, fs, path};

use slugify::slugify;

#[derive(Default)]
pub struct GlobalContext {
    pub targets: Vec<i32>,
}

struct LocalContext {
    targets: collections::HashSet<i32>,
}

fn get_out_file_name(script_name: &String) -> String {
    let mut out_file = format!("out/{}.txt", script_name);
    let mut count = 0;
    loop {
        if path::Path::new(&out_file).is_file() {
            count += 1;
            out_file = format!("out/{}_{}.txt", script_name, count);
        } else {
            return out_file;
        }
    }
}

pub struct Disassembler<'a> {
    definitions: &'a HashMap<types::Opcode, Command>,
    scanner: &'a scanner::Scanner,
}

impl<'a> Disassembler<'a> {
    pub fn new(
        definitions: &'a HashMap<types::Opcode, Command>,
        scanner: &'a scanner::Scanner,
    ) -> Self {
        Self {
            definitions,
            scanner,
        }
    }

    pub fn run(&self, instructions: Vec<Box<Instruction>>, script_type: ScriptType) -> IR {
        let mut name = String::from("noname");

        let name_def = self
            .definitions
            .iter()
            .find(|(_id, c)| c.name == "SCRIPT_NAME")
            .expect(&format!("Can't find a command with name SCRIPT_NAME"));

        for i in &instructions {
            if i.opcode == *name_def.0 {
                name = i.params.get(0).unwrap().to_string();
                break;
            }
        }
        let targets = self.scanner.collect_relative_addresses(&instructions);

        if let ScriptType::MAIN = script_type {
            if targets.len() > 0 {
                println!("Warning: Relative offsets found in the MAIN script");
            }
        }

        IR {
            name: slugify!(name.as_str(), separator = "_"),
            instructions,
            script_type,
            state: LocalContext { targets },
        }
    }
}

pub struct IR {
    pub name: String,
    pub instructions: Vec<Box<Instruction>>,
    script_type: ScriptType,
    state: LocalContext,
}

impl IR {
    pub fn print(self, global_context: &GlobalContext) {
        let mut f = fs::File::create(get_out_file_name(&self.name)).unwrap();
        for inst in self.instructions.iter() {
            let inst_offset: i32 = inst.offset.try_into().unwrap();

            match self.script_type {
                ScriptType::MAIN => {
                    if global_context.targets.contains(&inst_offset) {
                        writeln!(f, "\n:{}", inst.offset).unwrap()
                    }
                }
                _ => {
                    if self.state.targets.contains(&(-inst_offset)) {
                        writeln!(f, "\n:{}", inst.offset).unwrap()
                    }
                }
            }

            writeln!(f, "{}", inst).unwrap();
        }
    }
}
