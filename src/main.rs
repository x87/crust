mod data_type;
mod loader;
mod parser;

use data_type::{CommandDefinition, Instruction, InstructionParam3};
use scoped_threadpool;
use std::{collections, fs};

fn load_definitions() -> collections::HashMap<u16, CommandDefinition> {
    let file_content = fs::read_to_string("def.json").expect("Can't read def.json");

    let mut data: Vec<CommandDefinition> =
        serde_json::from_str(&file_content).expect("Can't parse def.json");

    let mut map = collections::HashMap::new();

    for c in data.drain(..) {
        let op =
            u16::from_str_radix(&c.id, 16).expect(&format!("Unexpected opcode number {}", c.id));
        map.insert(op, c);
    }
    return map;
}

fn main() {
    let definitions = load_definitions();
    let reader = data_type::Reader3 {};

    let args: Vec<String> = std::env::args().collect();

    let input_file = args
        .get(1)
        .unwrap_or_else(|| panic!("Provide input file name"));

    let scripts = loader::load(input_file.to_string(), Box::new(reader)).unwrap();
    let mut pool = scoped_threadpool::Pool::new(4);

    let def = &definitions;
    pool.scoped(|scoped| {
        for scr in scripts {
            scoped.execute(move || {
                let reader3 = data_type::Reader3 {};
                let parser = parser::Parser::new(&scr, def, Box::new(reader3));
                println!(
                    "script {:#?}",
                    parser.collect::<Vec<Instruction<InstructionParam3>>>()
                );
            });
        }
    });
}
