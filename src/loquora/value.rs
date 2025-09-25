use std::collections::HashMap;
use std::fmt;
use crate::loquora::ast::{ParamDecl, Stmt};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    Null,
    Object {
        type_name: String,
        fields: HashMap<String, Value>,
    },
    ToolRef {
        name: String,
        params: Vec<ParamDecl>,
        body: Vec<Stmt>,
    },
    List(Vec<Value>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Char(c) => write!(f, "'{}'", c),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::Object { type_name, fields } => {
                write!(f, "{} {{ ", type_name)?;
                let mut first = true;
                for (key, value) in fields {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                    first = false;
                }
                write!(f, " }}")
            }
            Value::ToolRef { name, .. } => write!(f, "tool<{}>", name),
            Value::List(items) => {
                write!(f, "[")?;
                let mut first = true;
                for item in items {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                    first = false;
                }
                write!(f, "]")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
    UndefinedVariable(String),
    UndefinedTool(String),
    UndefinedType(String),
    TypeMismatch {
        expected: String,
        actual: String,
    },
    FieldNotFound(String),
    RequiredFieldMissing(String),
    NotAnObject,
    NotCallable,
    InvalidArguments(String),
    DivisionByZero,
    BreakOutsideLoop,
    ContinueOutsideLoop,
    ReturnOutsideFunction,
    EmptyPath,
    Custom(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            RuntimeError::UndefinedTool(name) => write!(f, "Undefined tool: {}", name),
            RuntimeError::UndefinedType(name) => write!(f, "Undefined type: {}", name),
            RuntimeError::TypeMismatch { expected, actual } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, actual)
            }
            RuntimeError::FieldNotFound(name) => write!(f, "Field not found: {}", name),
            RuntimeError::RequiredFieldMissing(name) => {
                write!(f, "Required field missing: {}", name)
            }
            RuntimeError::NotAnObject => write!(f, "Value is not an object"),
            RuntimeError::NotCallable => write!(f, "Value is not callable"),
            RuntimeError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::BreakOutsideLoop => write!(f, "Break statement outside of loop"),
            RuntimeError::ContinueOutsideLoop => write!(f, "Continue statement outside of loop"),
            RuntimeError::ReturnOutsideFunction => {
                write!(f, "Return statement outside of function")
            }
            RuntimeError::EmptyPath => write!(f, "Empty assignment path"),
            RuntimeError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for RuntimeError {}

impl Value {
    pub fn get_property(&self, name: &str) -> Result<Value, RuntimeError> {
        match self {
            Value::Object { fields, .. } => fields
                .get(name)
                .cloned()
                .ok_or_else(|| RuntimeError::FieldNotFound(name.to_string())),
            _ => Err(RuntimeError::NotAnObject),
        }
    }

    pub fn set_property(&self, name: &str, value: Value) -> Result<Value, RuntimeError> {
        match self {
            Value::Object { type_name, fields } => {
                let mut new_fields = fields.clone();
                new_fields.insert(name.to_string(), value);
                Ok(Value::Object {
                    type_name: type_name.clone(),
                    fields: new_fields,
                })
            }
            _ => Err(RuntimeError::NotAnObject),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::String(_) => "String",
            Value::Char(_) => "Char",
            Value::Bool(_) => "Bool",
            Value::Null => "Null",
            Value::Object { .. } => "Object",
            Value::ToolRef { .. } => "Tool",
            Value::List(_) => "List",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Int(0) => false,
            Value::Float(f) if *f == 0.0 => false,
            Value::String(s) if s.is_empty() => false,
            Value::List(items) if items.is_empty() => false,
            _ => true,
        }
    }

    pub fn to_int(&self) -> Result<i64, RuntimeError> {
        match self {
            Value::Int(n) => Ok(*n),
            Value::Float(f) => Ok(*f as i64),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "Int".to_string(),
                actual: self.type_name().to_string(),
            }),
        }
    }

    pub fn to_float(&self) -> Result<f64, RuntimeError> {
        match self {
            Value::Int(n) => Ok(*n as f64),
            Value::Float(f) => Ok(*f),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "Float".to_string(),
                actual: self.type_name().to_string(),
            }),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            _ => format!("{}", self),
        }
    }
}