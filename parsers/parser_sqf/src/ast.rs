use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum SqfExpr {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    Variable(String),
    Array(Vec<SqfExpr>),
    Block(Vec<SqfExpr>),
    BinaryOp {
        op: String,
        left: Box<SqfExpr>,
        right: Box<SqfExpr>,
    },
    Assignment {
        name: String,
        value: Box<SqfExpr>,
    },
    FunctionCall {
        name: String,
        args: Vec<SqfExpr>,
    },
    ArrayAccess {
        array: Box<SqfExpr>,
        index: Box<SqfExpr>,
    },
    ForceAssignment {
        name: String,
        value: Box<SqfExpr>,
    },
    Comment(String),
    ForEach {
        body: Box<SqfExpr>,
        array: Box<SqfExpr>,
    },
}

#[derive(Debug)]
pub struct ItemReference {
    pub item_id: String,
    pub context: String,
}

#[derive(Debug, Clone)]
struct VarInfo {
    value: String,
    is_item: bool,
    used_by: Vec<String>,
    source_vars: Vec<String>,
}

#[derive(Debug, Default)]
pub struct SqfFile {
    pub expressions: Vec<SqfExpr>,
    pub variables: HashMap<String, SqfExpr>,
}

#[derive(Debug)]
pub struct SqfAst {
    pub expressions: Vec<SqfExpr>,
} 