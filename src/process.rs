use crate::types;

pub struct Script<'a> {
    pub name: String,
    pub instructions: Vec<Box<types::Instruction<'a>>>,
}

impl<'a> Script<'a> {
    pub fn new(instructions: Vec<Box<types::Instruction<'a>>>) -> Self {
        let mut name = String::from("noname");
        for i in &instructions {
            if i.opcode == 0x03A4 {
                name = i.params.get(0).unwrap().to_string().unwrap()
            }
        }
        Self { name, instructions }
    }
}
