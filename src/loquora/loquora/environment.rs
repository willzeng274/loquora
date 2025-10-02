use crate::loquora::ast::{ParamDecl, Stmt, StructMember};
use crate::loquora::value::{RuntimeError, Value};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum TypeDef {
    Struct {
        name: String,
        members: Vec<StructMember>,
    },
    Template {
        name: String,
        params: Vec<ParamDecl>,
        body: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolDef {
    pub name: String,
    pub params: Vec<ParamDecl>,
    pub body: Vec<Stmt>,
}

pub struct Environment {
    frames: Vec<HashMap<String, Value>>,
    pub global_tools: HashMap<String, ToolDef>,
    pub type_definitions: HashMap<String, TypeDef>,
    pub in_loop: usize,
    pub in_tool: bool,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            frames: vec![HashMap::new()],
            global_tools: HashMap::new(),
            type_definitions: HashMap::new(),
            in_loop: 0,
            in_tool: false,
        }
    }

    pub fn get(&self, name: &str) -> Result<Value, RuntimeError> {
        // TODO: replace with a proper built-in function implementation
        // standard library
        let builtin_result = match name {
            "print" => Some(Value::ToolRef {
                name: "print".to_string(),
                params: vec![],
                body: vec![],
            }),
            "panic" => Some(Value::ToolRef {
                name: "panic".to_string(),
                params: vec![],
                body: vec![],
            }),
            "list" => Some(Value::ToolRef {
                name: "list".to_string(),
                params: vec![],
                body: vec![],
            }),
            "cons" => Some(Value::ToolRef {
                name: "cons".to_string(),
                params: vec![],
                body: vec![],
            }),
            "nil" => Some(Value::List(vec![])),
            "object" => Some(Value::ToolRef {
                name: "object".to_string(),
                params: vec![],
                body: vec![],
            }),
            "pair" => Some(Value::ToolRef {
                name: "pair".to_string(),
                params: vec![],
                body: vec![],
            }),
            "get" => Some(Value::ToolRef {
                name: "get".to_string(),
                params: vec![],
                body: vec![],
            }),
            "lookup" => Some(Value::ToolRef {
                name: "lookup".to_string(),
                params: vec![],
                body: vec![],
            }),
            "int" => Some(Value::ToolRef {
                name: "int".to_string(),
                params: vec![],
                body: vec![],
            }),
            "float" => Some(Value::ToolRef {
                name: "float".to_string(),
                params: vec![],
                body: vec![],
            }),
            "bool" => Some(Value::ToolRef {
                name: "bool".to_string(),
                params: vec![],
                body: vec![],
            }),
            "str" => Some(Value::ToolRef {
                name: "str".to_string(),
                params: vec![],
                body: vec![],
            }),
            _ => None,
        };

        if let Some(builtin_value) = builtin_result {
            return Ok(builtin_value);
        }

        // check local variables from innermost to outermost scope
        for frame in self.frames.iter().rev() {
            if let Some(value) = frame.get(name) {
                return Ok(value.clone());
            }
        }

        // check global tools, we don't have local tools yet
        if let Some(tool_def) = self.global_tools.get(name) {
            return Ok(Value::ToolRef {
                name: tool_def.name.clone(),
                params: tool_def.params.clone(),
                body: tool_def.body.clone(),
            });
        }

        Err(RuntimeError::UndefinedVariable(name.to_string()))
    }

    pub fn set(&mut self, name: &str, value: Value) {
        if let Some(current_frame) = self.frames.last_mut() {
            current_frame.insert(name.to_string(), value);
        }
    }

    pub fn set_path(&mut self, path: &[String], value: Value) -> Result<(), RuntimeError> {
        if path.is_empty() {
            return Err(RuntimeError::EmptyPath);
        }

        if path.len() == 1 {
            // x = value
            self.set(&path[0], value);
            return Ok(());
        }

        // a.b.c = value
        let root_name = &path[0];
        let root_value = self.get(root_name)?;

        // update recursively nested object
        let new_root = self.update_nested_object(root_value, &path[1..], value)?;
        self.set(root_name, new_root);
        Ok(())
    }

    fn update_nested_object(
        &self,
        obj: Value,
        path: &[String],
        value: Value,
    ) -> Result<Value, RuntimeError> {
        if path.is_empty() {
            return Ok(value);
        }

        if path.len() == 1 {
            // set the property on the nested object
            return obj.set_property(&path[0], value);
        }

        // get nested object, update it, then set it back on the nested object
        let nested_obj = obj.get_property(&path[0])?;
        let updated_nested = self.update_nested_object(nested_obj, &path[1..], value)?;
        obj.set_property(&path[0], updated_nested)
    }

    pub fn push_scope(&mut self) {
        self.frames.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    pub fn enter_loop(&mut self) {
        self.in_loop += 1;
    }

    pub fn exit_loop(&mut self) {
        if self.in_loop > 0 {
            self.in_loop -= 1;
        }
    }

    pub fn is_in_loop(&self) -> bool {
        self.in_loop > 0
    }

    pub fn enter_tool(&mut self) {
        self.in_tool = true;
    }

    pub fn exit_tool(&mut self) {
        self.in_tool = false;
    }

    pub fn is_in_tool(&self) -> bool {
        self.in_tool
    }

    pub fn define_tool(&mut self, name: String, params: Vec<ParamDecl>, body: Vec<Stmt>) {
        self.global_tools
            .insert(name.clone(), ToolDef { name, params, body });
    }

    pub fn define_type(&mut self, type_def: TypeDef) {
        let name = match &type_def {
            TypeDef::Struct { name, .. } => name.clone(),
            TypeDef::Template { name, .. } => name.clone(),
        };
        self.type_definitions.insert(name, type_def);
    }

    pub fn create_object_from_typedef(
        &self,
        type_def: &TypeDef,
        field_values: HashMap<String, Value>,
    ) -> Result<Value, RuntimeError> {
        self.validate_object_fields(type_def, &field_values)?;

        let type_name = match type_def {
            TypeDef::Struct { name, .. } => name.clone(),
            TypeDef::Template { name, .. } => {
                return Err(RuntimeError::InvalidArguments(format!(
                    "Cannot instantiate template {}",
                    name
                )));
            }
        };

        Ok(Value::Object {
            type_name,
            fields: field_values,
        })
    }

    fn validate_object_fields(
        &self,
        type_def: &TypeDef,
        fields: &HashMap<String, Value>,
    ) -> Result<(), RuntimeError> {
        match type_def {
            TypeDef::Struct { members, .. } => {
                for member in members {
                    if let StructMember::Field(field) = member {
                        let field_name = &field.name;
                        let is_optional = field.suffix.as_ref().map_or(false, |s| s.contains('?'));
                        let is_required = field.suffix.as_ref().map_or(true, |s| s.contains('!'));

                        if is_required && !is_optional && !fields.contains_key(field_name) {
                            return Err(RuntimeError::RequiredFieldMissing(field_name.clone()));
                        }

                        if let Some(value) = fields.get(field_name) {
                            let is_nullable =
                                field.suffix.as_ref().map_or(false, |s| s.contains('?'));
                            if !is_nullable && matches!(value, Value::Null) {
                                return Err(RuntimeError::TypeMismatch {
                                    expected: "non-null".to_string(),
                                    actual: "null".to_string(),
                                });
                            }
                        }
                    }
                }
                Ok(())
            }
            TypeDef::Template { .. } => Ok(()),
        }
    }
}
