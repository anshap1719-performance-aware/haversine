use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Number {
    I64(i64),
    F64(f64),
}

#[derive(Debug, PartialEq)]
pub enum Value {
    String(String),
    Number(Number),
    Boolean(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Null,
}
