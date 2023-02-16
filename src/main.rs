mod disassembler;
mod library;
mod loader;
mod parser;
mod platform;
mod types;
extern crate slugify;

use clap::Parser;
use disassembler::scanner;
use library::Library;
use scoped_threadpool;
use std::fs;
use std::sync::Mutex;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file to disassemble
    input_file: String,

    /// File with command definitions (.json)
    defs: String,

    /// Target game
    #[arg(long)]
    game: platform::Game,

    // #[arg(long("out"), default_value_t=String::from("out"))]
    // out_dir: String,
}

fn main() {
    let cli = Args::parse();
    let game = &cli.game;
    let defs = Library::new(&cli.defs)
        .map(|x| x.to_map())
        .unwrap_or_default();
    let scripts = loader::load(cli.input_file, game, &defs).unwrap();
    let mut pool = scoped_threadpool::Pool::new(4);
    // temp
    if fs::metadata("out").is_ok() {
        fs::remove_dir_all("out").unwrap();
    }
    fs::create_dir_all("out").unwrap();

    let global_context_mutex = Mutex::new(disassembler::GlobalContext::default());
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
                ir.print(&context);
            });
        }
    });
}
