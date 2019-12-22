use crate::definitions;
use crate::types;

pub struct Script<'a> {
    pub name: String,
    pub instructions: Vec<Box<types::Instruction<'a>>>,
}

impl<'a> Script<'a> {
    pub fn new(
        instructions: Vec<Box<types::Instruction<'a>>>,
        definitions: &'a definitions::DefinitionMap,
    ) -> Self {
        let mut name = String::from("noname");
        let (name_op, _) = definitions
            .find_by_attr(definitions::ATTRIBUTE_NAME)
            .expect(&format!(
                "Can't find a command with attribute {}",
                definitions::ATTRIBUTE_NAME
            ));

        for i in &instructions {
            if i.opcode == *name_op {
                name = i.params.get(0).unwrap().to_string()
            }
        }
        Self { name, instructions }
    }
}
