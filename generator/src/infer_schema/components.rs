mod table_builder;

pub use table_builder::TableInfoBuilder;

use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub enum IntegerKind {
    I8,
    I16,
    I32,
    I64,
    I128,
    ISize,
    U8,
    U16,
    U32,
    U64,
    U128,
    USize,
}

impl std::fmt::Display for IntegerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            IntegerKind::I8 => write!(f, "i8"),
            IntegerKind::I16 => write!(f, "i16"),
            IntegerKind::I32 => write!(f, "i32"),
            IntegerKind::I64 => write!(f, "i64"),
            IntegerKind::I128 => write!(f, "i128"),
            IntegerKind::ISize => write!(f, "isize"),
            IntegerKind::U8 => write!(f, "u8"),
            IntegerKind::U16 => write!(f, "u16"),
            IntegerKind::U32 => write!(f, "u32"),
            IntegerKind::U64 => write!(f, "u64"),
            IntegerKind::U128 => write!(f, "u128"),
            IntegerKind::USize => write!(f, "usize"),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub enum FloatKind {
    F32,
    F64,
}

impl std::fmt::Display for FloatKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FloatKind::F32 => write!(f, "f32"),
            FloatKind::F64 => write!(f, "f64"),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub enum RelationKind {
    HasOne,
    HasMany,
}

impl Display for RelationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RelationKind::HasOne => write!(f, "HasOne"),
            RelationKind::HasMany => write!(f, "HasMany"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DataKind {
    String,
    Integer(IntegerKind),
    Float(FloatKind),
    Boolean,
    Array(Box<DataKind>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColumnInfo {
    pub required: bool,
    pub kind: DataKind,
}

impl ColumnInfo {
    pub fn new(required: bool, kind: DataKind) -> Self {
        Self { required, kind }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub struct RelationInfo {
    pub required: bool,
    pub property: String,
    pub target: String,
    pub kind: RelationKind,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TableInfo {
    pub name: String,
    pub columns: HashMap<String, ColumnInfo>,
    pub references: Vec<RelationInfo>,
}

#[derive(Debug, Default)]
pub struct Schema {
    pub tables: HashMap<String, TableInfo>,
    pub graphs: Vec<GraphInfo>,
}

#[derive(Debug, Default)]
pub struct GraphInfo {
    pub members: HashSet<String>,
    pub edges: HashSet<EdgeInfo>,
}

#[derive(Debug, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub struct EdgeInfo {
    pub origin: String,
    pub relation: RelationInfo,
}
