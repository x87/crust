use crate::definitions;
use crate::platform;
use crate::types;

use byteorder::{LittleEndian, ReadBytesExt};
use std::fmt::Debug;
use std::{fs, io};

const MISSIONS_SEG: usize = 2;
const EXTERNALS_SEG: usize = 3;

enum ScriptType {
    MAIN,
    MISSION,
    EXTERNAL,
}

struct ScriptFile {
    code: types::ScriptChunk,
    size: u32,
}

impl ScriptFile {
    fn new(code: types::ScriptChunk) -> Self {
        Self {
            size: code.len() as u32,
            code,
        }
    }

    fn extract(&self, start: u32, end: u32) -> &[u8] {
        &self.code[start as usize..end as usize]
    }
}

#[derive(Debug)]
pub struct Script(Vec<types::ScriptChunk>);
impl Script {
    fn new() -> Self {
        Self { 0: Vec::new() }
    }
    fn register_script(&mut self, buf: &[u8], _script_type: ScriptType) {
        self.0.push(buf.to_vec());
    }
}

impl IntoIterator for Script {
    type Item = types::ScriptChunk;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

struct Missions {
    main_size: u32,
    missions: Vec<u32>,
    file_size: u32,
    current: usize,
}
impl Missions {
    fn new(chunk: &[u8], file_size: u32) -> Self {
        let mut cursor = io::Cursor::new(chunk);
        cursor.set_position(1); // todo: assert segment id?

        let main_size = cursor.read_u32::<LittleEndian>().unwrap();
        let _largest_mission = cursor.read_u32::<LittleEndian>().unwrap();
        let num_missions = cursor.read_u16::<LittleEndian>().unwrap();
        let _num_exclusive_missions = cursor.read_u16::<LittleEndian>().unwrap();
        let mut missions = Vec::new();
        for _ in 0..num_missions {
            missions.push(cursor.read_u32::<LittleEndian>().unwrap())
        }

        Self {
            main_size,
            missions,
            file_size,
            current: 0,
        }
    }
}
impl Iterator for Missions {
    type Item = (u32, u32);
    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.missions.get(self.current)?;
        self.current += 1;
        // todo: missions can be unordered, need to sort offsets first
        let end = if self.current == self.missions.len() {
            self.file_size
        } else {
            self.missions[self.current]
        };
        Some((*offset, end))
    }
}

struct Externals {}
impl Externals {
    fn new(_chunk: &[u8]) -> Self {
        unimplemented!()
    }
}
impl Iterator for Externals {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

struct ImgArchive {}
impl ImgArchive {
    fn new(_filename: String) -> Self {
        unimplemented!()
    }
    fn extract(&self, _name: String) -> &[u8] {
        unimplemented!()
    }
}

fn get_segments<'a>(
    chunk: &types::ScriptChunk,
    game: &platform::Game,
    defs: &definitions::DefinitionMap,
) -> Vec<(u32, u32)> {
    let mut offsets: Vec<(u32, u32)> = Vec::new();
    let (op, c) = defs
        .find_by_attr(definitions::ATTRIBUTE_GOTO)
        .expect(&format!(
            "Can't find a command with attribute {}",
            definitions::ATTRIBUTE_GOTO
        ));
    let defs = definitions::DefinitionMap::from_pairs(vec![(*op, c.clone())]);

    let mut parser = platform::get_parser(game, chunk, &defs);
    loop {
        match parser.next() {
            Some(inst) if inst.opcode != 0xFFFF => match inst.params[0].to_i32() {
                Some(destination) if destination > 0 => {
                    offsets.push((parser.get_parser().get_position(), destination as u32));
                    parser.get_parser_as_mut().set_position(destination as u32);
                }
                _ => break,
            },
            Some(_) => break,
            None => break,
        }
    }
    offsets
}

pub fn load(
    input_file: String,
    game: &platform::Game,
    defs: &definitions::DefinitionMap,
) -> Result<Script, String> {
    let chunk =
        fs::read(&input_file).map_err(|_| format!("Can't read input file {}", input_file))?;

    let segments = get_segments(&chunk, game, defs);
    let script_file = ScriptFile::new(chunk);
    let mut script = Script::new();

    if segments.len() > 0 {
        match segments.len() {
            0..=2 => return Err(String::from("No missions segment found")),
            3 | 6 => {}
            _ => return Err(String::from("Invalid header structure")),
        }
        let (offset, end) = segments.get(MISSIONS_SEG).unwrap();
        let missions = Missions::new(script_file.extract(*offset, *end), script_file.size);
        let (_, main_start) = segments.last().unwrap();
        let main_script = script_file.extract(*main_start, missions.main_size);
        script.register_script(main_script, ScriptType::MAIN);
        for (start, end) in missions {
            // todo: empty missions
            if end > start {
                script.register_script(script_file.extract(start, end), ScriptType::MISSION);
            }
        }
        if let Some((offset, end)) = segments.get(EXTERNALS_SEG) {
            let externals: Vec<String> =
                Externals::new(script_file.extract(*offset, *end)).collect();
            if externals.len() > 0 {
                let script_img = ImgArchive::new("script.img".to_string());
                for name in externals {
                    script.register_script(script_img.extract(name), ScriptType::EXTERNAL);
                }
            }
        }
    } else {
        let main_script = script_file.extract(0, script_file.size);
        script.register_script(main_script, ScriptType::EXTERNAL);
    }
    Ok(script)
}
