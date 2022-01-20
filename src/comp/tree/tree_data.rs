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
    pub kind: RowKind,
    pub value: String,
    pub is_parent: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum RowKind {
    Bool,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    Float,
    Hash,
    Str,
    List,
    Struct,
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
                            kind: RowKind::from_param(param),
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
                            kind: RowKind::from_param(param),
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

impl RowKind {
    pub fn from_param(param: &ParamKind) -> Self {
        match param {
            ParamKind::Bool(_) => RowKind::Bool,
            ParamKind::I8(_) => RowKind::I8,
            ParamKind::U8(_) => RowKind::U8,
            ParamKind::I16(_) => RowKind::I16,
            ParamKind::U16(_) => RowKind::U16,
            ParamKind::I32(_) => RowKind::I32,
            ParamKind::U32(_) => RowKind::U32,
            ParamKind::Float(_) => RowKind::Float,
            ParamKind::Hash(_) => RowKind::Hash,
            ParamKind::Str(_) => RowKind::Str,
            ParamKind::List(_) => RowKind::List,
            ParamKind::Struct(_) => RowKind::Struct,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RowKind::Bool => "bool",
            RowKind::I8 => "i8",
            RowKind::U8 => "u8",
            RowKind::I16 => "i16",
            RowKind::U16 => "u16",
            RowKind::I32 => "i32",
            RowKind::U32 => "u32",
            RowKind::Float => "f32",
            RowKind::Hash => "hash",
            RowKind::Str => "string",
            RowKind::List => "list",
            RowKind::Struct => "struct",
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(
            self,
            RowKind::I8
                | RowKind::U8
                | RowKind::I16
                | RowKind::U16
                | RowKind::I32
                | RowKind::U32
                | &RowKind::Float
        )
    }

    pub fn is_incremental(&self) -> bool {
        if let RowKind::Bool = self {
            true
        } else {
            self.is_number()
        }
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
