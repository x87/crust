mod disassembler;
mod library;
mod loader;
mod parser;
mod platform;
mod types;
extern crate slugify;

use disassembler::scanner;
use library::Library;
use scoped_threadpool;
use std::fs;
use std::sync::Mutex;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let game = &platform::Game::GTA3;
    let input_file = args
        .get(1)
        .unwrap_or_else(|| panic!("Provide input file name"));

    let defs = Library::from_meta_file("gta3.json")
        .map(|x| x.to_map())
        .unwrap_or_default();
    let scripts = loader::load(input_file.to_string(), game, &defs).unwrap();
    let mut pool = scoped_threadpool::Pool::new(4);
    // temp
    if fs::metadata("out").is_ok() {
        fs::remove_dir_all("out").unwrap();
    }
    fs::create_dir_all("out").unwrap();

    let global_context_mutex = Mutex::new(disassembler::GlobalContext { targets: vec![] });
    let irs_mutex: Mutex<Vec<disassembler::IR>> = Mutex::new(vec![]);
    let scanner = scanner::Scanner::new(&defs);
    let dasm = disassembler::Disassembler::new(&defs, &scanner);

    pool.scoped(|scoped| {
        for scr in &scripts {
            scoped.execute(|| {
                let parser = platform::get_parser(game, &scr.chunk, &defs, scr.base_offset);
                let instructions = parser.collect();

                {
                    let global_addresses = scanner.collect_global_addresses(&instructions);
                    let mut global_context = global_context_mutex.lock().unwrap();
                    (*global_context).targets.extend(global_addresses);
                }

                let ir = dasm.run(instructions, scr.script_type);
                let mut irs = irs_mutex.lock().unwrap();
                (*irs).push(ir);
            });
        }
    });

    let context = global_context_mutex.into_inner().unwrap();
    pool.scoped(|scoped| {
        for ir in irs_mutex.into_inner().unwrap() {
            scoped.execute(|| {
                dasm.print(ir, &context);
            });
        }
    });
}
