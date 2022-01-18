use prc::param::*;
use regex::Regex;

#[derive(Debug)]
pub struct TreeData {
    pub rows: Vec<TreeRow>,
    pub kind: TreeKind,
}

#[derive(Debug)]
pub enum TreeKind {
    Struct,
    List,
}

#[derive(Debug)]
pub struct TreeRow {
    pub index: usize,
    pub name: String,
    pub kind: &'static str,
    pub value: String,
    pub is_parent: bool,
}

impl TreeData {
    pub fn new(param: &ParamKind) -> Self {
        match param {
            ParamKind::Struct(s) => {
                let rows =
                    s.0.iter()
                        .enumerate()
                        .map(|(index, (hash, param))| TreeRow {
                            index,
                            name: format!("{}", hash),
                            kind: get_kind(param),
                            value: get_value(param),
                            is_parent: is_parent(param),
                        })
                        .collect();
                TreeData {
                    rows,
                    kind: TreeKind::Struct,
                }
            }
            ParamKind::List(l) => {
                let rows =
                    l.0.iter()
                        .enumerate()
                        .map(|(index, param)| TreeRow {
                            index,
                            name: format!("{}", index),
                            kind: get_kind(param),
                            value: get_value(param),
                            is_parent: is_parent(param),
                        })
                        .collect();
                TreeData {
                    rows,
                    kind: TreeKind::List,
                }
            }
            _ => panic!("Can't create tree data from non-iterable param type"),
        }
    }

    pub fn apply_filter(mut self, filter: Option<&Regex>) -> Self {
        if let Some(reg) = filter {
            self.rows.retain(|r| reg.is_match(&r.name));
        }
        self
    }
}

fn get_kind(param: &ParamKind) -> &'static str {
    match param {
        ParamKind::Bool(_) => "bool",
        ParamKind::I8(_) => "i8",
        ParamKind::U8(_) => "u8",
        ParamKind::I16(_) => "i16",
        ParamKind::U16(_) => "u16",
        ParamKind::I32(_) => "i32",
        ParamKind::U32(_) => "u32",
        ParamKind::Float(_) => "f32",
        ParamKind::Hash(_) => "hash",
        ParamKind::Str(_) => "string",
        ParamKind::List(_) => "list",
        ParamKind::Struct(_) => "struct",
    }
}

fn get_value(param: &ParamKind) -> String {
    match param {
        ParamKind::Bool(v) => format!("{}", v),
        ParamKind::I8(v) => format!("{}", v),
        ParamKind::U8(v) => format!("{}", v),
        ParamKind::I16(v) => format!("{}", v),
        ParamKind::U16(v) => format!("{}", v),
        ParamKind::I32(v) => format!("{}", v),
        ParamKind::U32(v) => format!("{}", v),
        ParamKind::Float(v) => format!("{}", v),
        ParamKind::Hash(v) => format!("{}", v),
        ParamKind::Str(v) => v.to_string(),
        ParamKind::List(v) => format!("({} children)", v.0.len()),
        ParamKind::Struct(v) => format!("({} children)", v.0.len()),
    }
}

fn is_parent(param: &ParamKind) -> bool {
    matches!(param, ParamKind::Struct(_) | ParamKind::List(_))
}
