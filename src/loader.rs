use crate::library::Command;
// use crate::definitions;
use crate::platform;
use crate::types;
use crate::types::*;

use byteorder::{LittleEndian, ReadBytesExt};
use io::Cursor;
use std::collections::HashMap;
use std::{fs, io};

const MISSIONS_SEG: usize = 2;
const EXTERNALS_SEG: usize = 3;

struct ScriptFile {
    code: ScriptChunk,
    size: u32,
}

impl ScriptFile {
    fn new(code: ScriptChunk) -> Self {
        Self {
            size: code.len() as u32,
            code,
        }
    }

    fn extract(&self, start: u32, end: u32) -> &[u8] {
        &self.code[start as usize..end as usize]
    }
}

pub struct Script {
    pub chunk: ScriptChunk,
    pub script_type: ScriptType,
    pub base_offset: u32,
}

impl Script {
    fn new(chunk: ScriptChunk, script_type: ScriptType, base_offset: u32) -> Self {
        Self {
            chunk,
            script_type,
            base_offset,
        }
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
        let mut cursor = Cursor::new(chunk);
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
    chunk: &ScriptChunk,
    game: &platform::Game,
    defs: &HashMap<types::Opcode, Command>,
) -> Vec<(u32, u32)> {
    let mut offsets: Vec<(u32, u32)> = Vec::new();
    let (id, c) = defs
        .iter()
        .find(|(_id, c)| c.attrs.is_segment)
        .expect("Can't find a command with attribute is_segment");
    let mut defs = HashMap::new();
    defs.insert(id.clone(), c.clone());

    let mut parser = platform::get_parser(game, chunk, &defs, 0);
    loop {
        match parser.next() {
            Some(inst) if inst.opcode != 0xFFFF => match inst.params[0].to_offset() {
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
    defs: &HashMap<types::Opcode, Command>,
) -> Result<Vec<Script>, String> {
    let chunk =
        fs::read(&input_file).map_err(|_| format!("Can't read input file {}", input_file))?;

    let segments = get_segments(&chunk, game, defs);
    let script_file = ScriptFile::new(chunk);

    match segments.len() {
        0 => {
            let main_script = script_file.extract(0, script_file.size);
            return Ok(vec![Script::new(
                main_script.to_vec(),
                ScriptType::EXTERNAL,
                0,
            )]);
        }
        1 | 2 => return Err(String::from("No missions segment found")),
        3 | 6 => {}
        _ => return Err(String::from("Invalid header structure")),
    }
    let (offset, end) = segments.get(MISSIONS_SEG).unwrap();
    let missions = Missions::new(script_file.extract(*offset, *end), script_file.size);
    let (_, main_start) = segments.last().unwrap();
    let main_script = script_file.extract(*main_start, missions.main_size);
    let mut scripts = vec![Script::new(
        main_script.to_vec(),
        ScriptType::MAIN,
        *main_start,
    )];

    for (start, end) in missions {
        // todo: empty missions
        if end > start {
            scripts.push(Script::new(
                script_file.extract(start, end).to_vec(),
                ScriptType::MISSION,
                0,
            ));
        }
    }
    if let Some((offset, end)) = segments.get(EXTERNALS_SEG) {
        let externals: Vec<String> = Externals::new(script_file.extract(*offset, *end)).collect();
        if externals.len() > 0 {
            let script_img = ImgArchive::new(String::from("script.img"));
            for name in externals {
                scripts.push(Script::new(
                    script_img.extract(name).to_vec(),
                    ScriptType::EXTERNAL,
                    0,
                ));
            }
        }
    }

    Ok(scripts)
}
