use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

use crate::types;

#[derive(Debug, PartialEq, Clone)]
pub enum CommandParamType {
    Int,
    Float,
    String,
    Boolean,
    Label,
    Arguments,
    Vector(usize),
    Any(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum CommandParamSource {
    Any,
    AnyVar,
    AnyVarGlobal,
    AnyVarLocal,
    Literal,
    Pointer,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Platform {
    Any,
    PC,
    Console,
    Mobile,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Version {
    Any,
    _10,
    _10DE,
}

impl<'de> Deserialize<'de> for CommandParamType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer).as_deref() {
            Ok("float") => Ok(Self::Float),
            Ok("int" | "model_any" | "model_char" | "model_object" | "model_vehicle") => {
                Ok(Self::Int)
            }
            Ok("label") => Ok(Self::Label),
            Ok("string" | "gxt_key" | "zone_key") => Ok(Self::String),
            Ok("bool" | "boolean") => Ok(Self::Boolean),
            Ok("arguments") => Ok(Self::Arguments),
            Ok("Object") => Ok(Self::Any("ScriptObject".to_string())),
            Ok("Vector3") => Ok(Self::Vector(3)),
            Ok(name) => Ok(Self::Any(name.to_string())), // todo: enum/class name
            _ => Ok(Self::Int),                          // todo: error handling?
        }
    }
}

impl<'de> Deserialize<'de> for CommandParamSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer).as_deref() {
            Ok("any") => Ok(Self::Any),
            Ok("var_any") => Ok(Self::AnyVar),
            Ok("var_global") => Ok(Self::AnyVarGlobal),
            Ok("var_local") => Ok(Self::AnyVarLocal),
            Ok("literal") => Ok(Self::Literal),
            Ok("pointer") => Ok(Self::Pointer),
            _ => Ok(Self::Any),
        }
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Attr {
    #[serde(default)]
    pub is_branch: bool,
    #[serde(default)]
    pub is_condition: bool,
    #[serde(default)]
    pub is_constructor: bool,
    #[serde(default)]
    pub is_destructor: bool,
    #[serde(default)]
    pub is_keyword: bool,
    #[serde(default)]
    pub is_nop: bool,
    #[serde(default)]
    pub is_overload: bool,
    #[serde(default)]
    pub is_segment: bool,
    #[serde(default)]
    pub is_static: bool,
    #[serde(default)]
    pub is_unsupported: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CommandParam {
    pub r#name: String,
    pub r#source: CommandParamSource,
    pub r#type: CommandParamType,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Command {
    #[serde(default, deserialize_with = "convert_to_number")]
    pub id: Option<u16>,
    pub name: String,
    pub num_params: i32,
    #[serde(default)]
    pub short_desc: String,
    pub class: Option<String>,
    pub member: Option<String>,
    #[serde(default)]
    pub attrs: Attr,
    #[serde(default)]
    pub input: Vec<CommandParam>,
    #[serde(default)]
    pub output: Vec<CommandParam>,
    #[serde(default, deserialize_with = "convert_platform")]
    pub platforms: Vec<Platform>,
    #[serde(default, deserialize_with = "convert_version")]
    pub versions: Vec<Version>,
}

#[derive(Deserialize, Debug)]
pub struct Extension {
    pub name: String,
    pub commands: Vec<Command>,
}

#[derive(Deserialize, Debug)]
pub struct Meta {
    pub last_update: u64,
    pub url: String,
    pub version: String,
}

#[derive(Deserialize, Debug)]
pub struct ClassMeta {
    #[serde(deserialize_with = "rename_object")]
    pub name: String,
    #[serde(default)]
    pub desc: String,
    pub extends: Option<String>,
    pub constructable: bool,
}

#[derive(Deserialize, Debug)]
pub struct Library {
    pub meta: Meta,
    pub extensions: Vec<Extension>,
    #[serde(default)]
    pub classes: Vec<ClassMeta>,
}

impl Library {
    pub fn new(path: &str) -> Option<Self> {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let library = serde_json::from_str::<Library>(&content).ok()?;
        Some(library)
    }

    pub fn to_map(&self) -> HashMap<types::Opcode, Command> {
        let mut map = HashMap::new();
        for command in self.extensions.iter().flat_map(|ext| ext.commands.iter()) {
            if let Some(id) = command.id {
                map.insert(id, command.clone());
            }
        }

        map
    }
}

fn convert_to_number<'de, D>(deserializer: D) -> Result<Option<u16>, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer) {
        Ok(val) => match u16::from_str_radix(val.as_str(), 16) {
            Ok(res) => Ok(Some(res)),
            Err(e) => Err(serde::de::Error::custom(e)),
        },
        Err(_) => Ok(None),
    }
}

fn convert_platform<'de, D>(deserializer: D) -> Result<Vec<Platform>, D::Error>
where
    D: Deserializer<'de>,
{
    let res = match Vec::deserialize(deserializer) {
        Ok(x) => x
            .iter()
            .try_fold(vec![], |mut acc: Vec<Platform>, el: &String| {
                match el.as_str() {
                    "any" => acc.push(Platform::Any),
                    "pc" => acc.push(Platform::PC),
                    "console" => acc.push(Platform::Console),
                    "mobile" => acc.push(Platform::Mobile),
                    _ => {
                        panic!("Unknown platform name {el}");
                    }
                };
                Some(acc)
            })
            .unwrap_or_default(),
        _ => vec![],
    };
    Ok(res)
}

fn convert_version<'de, D>(deserializer: D) -> Result<Vec<Version>, D::Error>
where
    D: Deserializer<'de>,
{
    let res = match Vec::deserialize(deserializer) {
        Ok(x) => x
            .iter()
            .try_fold(vec![], |mut acc: Vec<Version>, el: &String| {
                match el.as_str() {
                    "any" => acc.push(Version::Any),
                    "1.0" => acc.push(Version::_10),
                    "1.0 [DE]" => acc.push(Version::_10DE),
                    _ => {
                        // panic!("Unknown version {el}");
                    }
                };
                Some(acc)
            })
            .unwrap_or_default(),
        _ => vec![],
    };
    Ok(res)
}

fn rename_object<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer) {
        Ok(x) if x.eq("Object") => Ok(String::from("ScriptObject")),
        x => x,
    }
}
