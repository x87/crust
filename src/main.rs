mod definitions;
mod loader;
mod parser;
mod platform;
mod process;
mod types;

use scoped_threadpool;
use std::io::prelude::*;
use std::{fs, path};

fn get_out_file_name(script_name: String) -> String {
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

fn main() {
    let definitions = &definitions::load();

    let args: Vec<String> = std::env::args().collect();
    let game = &platform::Game::GTA3;

    let input_file = args
        .get(1)
        .unwrap_or_else(|| panic!("Provide input file name"));

    let scripts = loader::load(input_file.to_string(), game).unwrap();
    let mut pool = scoped_threadpool::Pool::new(4);

    // temp
    fs::remove_dir_all("out").unwrap();
    fs::create_dir_all("out").unwrap();

    pool.scoped(|scoped| {
        for scr in scripts {
            scoped.execute(move || {
                let parser = platform::get_parser(game, &scr, definitions);
                let instructions = parser.collect();
                let s = process::Script::new(instructions);

                let mut f = fs::File::create(get_out_file_name(s.name)).unwrap();

                for inst in s.instructions {
                    writeln!(f, "{}", inst.print()).unwrap()
                }
            });
        }
    });
}
