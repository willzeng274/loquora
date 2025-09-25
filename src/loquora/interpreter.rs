use crate::loquora::ast::*;
use crate::loquora::token::TokenKind;
use crate::loquora::value::{Value, RuntimeError};
use crate::loquora::environment::{Environment, TypeDef};

#[derive(Debug)]
pub enum ControlFlow {
    None,
    Return(Value),
    Break,
    Continue,
}

pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            env: Environment::new(),
        }
    }

    pub fn interpret_program(&mut self, program: &Program) -> Result<Value, RuntimeError> {
        let last_value = Value::Null;

        for stmt in &program.statements {
            match self.interpret_statement(stmt)? {
                ControlFlow::Return(value) => return Ok(value),
                ControlFlow::Break => return Err(RuntimeError::BreakOutsideLoop),
                ControlFlow::Continue => return Err(RuntimeError::ContinueOutsideLoop),
                ControlFlow::None => {}
            }
        }

        Ok(last_value)
    }

    fn interpret_statement(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        match &stmt.inner {
            StmtKind::Assignment { target, value } => {
                let val = self.interpret_expression(value)?;
                self.env.set_path(target, val)?;
                Ok(ControlFlow::None)
            }

            StmtKind::ExprStmt { expr } => {
                self.interpret_expression(expr)?;
                Ok(ControlFlow::None)
            }

            StmtKind::Return { expr } => {
                if !self.env.is_in_tool() {
                    return Err(RuntimeError::ReturnOutsideFunction);
                }
                let value = if let Some(expr) = expr {
                    self.interpret_expression(expr)?
                } else {
                    Value::Null
                };
                Ok(ControlFlow::Return(value))
            }

            StmtKind::Break => {
                if !self.env.is_in_loop() {
                    return Err(RuntimeError::BreakOutsideLoop);
                }
                Ok(ControlFlow::Break)
            }

            StmtKind::Continue => {
                if !self.env.is_in_loop() {
                    return Err(RuntimeError::ContinueOutsideLoop);
                }
                Ok(ControlFlow::Continue)
            }

            StmtKind::ToolDecl { name, params, return_type: _, body } => {
                self.env.define_tool(name.clone(), params.clone(), body.clone());
                Ok(ControlFlow::None)
            }

            StmtKind::SchemaDecl { name, fields } => {
                let type_def = TypeDef::Schema {
                    name: name.clone(),
                    fields: fields.clone(),
                };
                self.env.define_type(type_def);
                Ok(ControlFlow::None)
            }

            StmtKind::StructDecl { name, members } => {
                let type_def = TypeDef::Struct {
                    name: name.clone(),
                    members: members.clone(),
                };
                self.env.define_type(type_def);
                Ok(ControlFlow::None)
            }

            StmtKind::ModelDecl { name, base, members } => {
                let type_def = TypeDef::Model {
                    name: name.clone(),
                    base: base.clone(),
                    members: members.clone(),
                };
                self.env.define_type(type_def);
                Ok(ControlFlow::None)
            }

            StmtKind::TemplateDecl { name, params, body } => {
                let type_def = TypeDef::Template {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                };
                self.env.define_type(type_def);
                Ok(ControlFlow::None)
            }

            StmtKind::If { arms, else_body } => {
                for (condition, body) in arms {
                    let cond_value = self.interpret_expression(condition)?;
                    if cond_value.is_truthy() {
                        let result = self.interpret_block(body)?;
                        return Ok(result);
                    }
                }

                if let Some(else_body) = else_body {
                    let result = self.interpret_block(else_body)?;
                    Ok(result)
                } else {
                    Ok(ControlFlow::None)
                }
            }

            StmtKind::While { cond, body } => {
                self.env.enter_loop();
                loop {
                    let cond_value = self.interpret_expression(cond)?;
                    if !cond_value.is_truthy() {
                        break;
                    }

                    let control = self.interpret_block(body)?;

                    match control {
                        ControlFlow::Break => break,
                        ControlFlow::Continue => continue,
                        ControlFlow::Return(value) => {
                            self.env.exit_loop();
                            return Ok(ControlFlow::Return(value));
                        }
                        ControlFlow::None => {}
                    }
                }
                self.env.exit_loop();
                Ok(ControlFlow::None)
            }

            StmtKind::Loop { body } => {
                self.env.enter_loop();
                loop {
                    let control = self.interpret_block(body)?;

                    match control {
                        ControlFlow::Break => break,
                        ControlFlow::Continue => continue,
                        ControlFlow::Return(value) => {
                            self.env.exit_loop();
                            return Ok(ControlFlow::Return(value));
                        }
                        ControlFlow::None => {}
                    }
                }
                self.env.exit_loop();
                Ok(ControlFlow::None)
            }

            StmtKind::For { init, cond, step, body } => {
                self.env.enter_loop();
                self.env.push_scope();

                if let Some((target, value_expr)) = init {
                    let value = self.interpret_expression(value_expr)?;
                    self.env.set_path(target, value)?;
                }

                loop {
                    if let Some(cond_expr) = cond {
                        let cond_value = self.interpret_expression(cond_expr)?;
                        if !cond_value.is_truthy() {
                            break;
                        }
                    }

                    let control = self.interpret_block(body)?;

                    match control {
                        ControlFlow::Break => break,
                        ControlFlow::Continue => {},
                        ControlFlow::Return(value) => {
                            self.env.pop_scope();
                            self.env.exit_loop();
                            return Ok(ControlFlow::Return(value));
                        }
                        ControlFlow::None => {}
                    }

                    if let Some(step_expr) = step {
                        self.interpret_expression(step_expr)?;
                    }
                }

                self.env.pop_scope();
                self.env.exit_loop();
                Ok(ControlFlow::None)
            }

            StmtKind::With { expr, body } => {
                let _with_value = self.interpret_expression(expr)?;
                self.env.push_scope();
                let result = self.interpret_block(body)?;
                self.env.pop_scope();
                Ok(result)
            }

            // TODO: modules and imports
            StmtKind::ImportModule { .. } => Ok(ControlFlow::None),
            StmtKind::ImportFrom { .. } => Ok(ControlFlow::None),
            StmtKind::Export { .. } => Ok(ControlFlow::None),
        }
    }

    fn interpret_block(&mut self, statements: &[Stmt]) -> Result<ControlFlow, RuntimeError> {
        for stmt in statements {
            let control = self.interpret_statement(stmt)?;
            match control {
                ControlFlow::None => continue,
                _ => return Ok(control),
            }
        }
        Ok(ControlFlow::None)
    }

    fn interpret_expression(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match &expr.inner {
            ExprKind::Int(n) => Ok(Value::Int(*n)),
            ExprKind::Float(f) => Ok(Value::Float(*f)),
            ExprKind::String(s) => Ok(Value::String(s.clone())),
            ExprKind::Char(c) => Ok(Value::Char(*c)),
            ExprKind::Bool(b) => Ok(Value::Bool(*b)),
            ExprKind::Null => Ok(Value::Null),

            ExprKind::Identifier(name) => self.env.get(name),

            ExprKind::BinaryOp { op, left, right } => {
                self.interpret_binary_op(op, left, right)
            }

            ExprKind::UnaryOp { op, expr } => {
                self.interpret_unary_op(op, expr)
            }

            ExprKind::Property { object, property } => {
                let obj_value = self.interpret_expression(object)?;
                obj_value.get_property(property)
            }

            ExprKind::Call { callee, args } => {
                self.interpret_call(callee, args)
            }

            ExprKind::Ternary { cond, if_true, if_false } => {
                let cond_value = self.interpret_expression(cond)?;
                if cond_value.is_truthy() {
                    self.interpret_expression(if_true)
                } else {
                    self.interpret_expression(if_false)
                }
            }

            ExprKind::Quaternary { cond, if_true, if_false, if_null } => {
                let cond_value = self.interpret_expression(cond)?;
                match cond_value {
                    Value::Null => self.interpret_expression(if_null),
                    _ if cond_value.is_truthy() => self.interpret_expression(if_true),
                    _ => self.interpret_expression(if_false),
                }
            }

            ExprKind::ObjectInit { type_name, fields } => {
                self.create_object_instance(type_name, fields)
            }
        }
    }

    fn interpret_binary_op(&mut self, op: &TokenKind, left: &Expr, right: &Expr) -> Result<Value, RuntimeError> {
        match op {
            TokenKind::LogicalAnd => {
                let left_val = self.interpret_expression(left)?;
                if !left_val.is_truthy() {
                    Ok(left_val)
                } else {
                    self.interpret_expression(right)
                }
            }
            TokenKind::LogicalOr => {
                let left_val = self.interpret_expression(left)?;
                if left_val.is_truthy() {
                    Ok(left_val)
                } else {
                    self.interpret_expression(right)
                }
            }
            _ => {
                let left_val = self.interpret_expression(left)?;
                let right_val = self.interpret_expression(right)?;

                match op {
                    // arithmetic
                    TokenKind::Plus => self.add_values(left_val, right_val),
                    TokenKind::Minus => self.subtract_values(left_val, right_val),
                    TokenKind::Multiply => self.multiply_values(left_val, right_val),
                    TokenKind::Divide => self.divide_values(left_val, right_val),
                    TokenKind::Modulo => self.modulo_values(left_val, right_val),
                    // useless @ operator that returns lvalue
                    // Loquora signature
                    TokenKind::At => Ok(left_val),

                    // bitwise
                    TokenKind::BitAnd => self.bitwise_and(left_val, right_val),
                    TokenKind::BitOr => self.bitwise_or(left_val, right_val),
                    TokenKind::BitXor => self.bitwise_xor(left_val, right_val),
                    TokenKind::ShiftLeft => self.shift_left(left_val, right_val),
                    TokenKind::ShiftRight => self.shift_right(left_val, right_val),

                    // comparison
                    TokenKind::EqualEqual => Ok(Value::Bool(self.values_equal(&left_val, &right_val))),
                    TokenKind::NotEqual => Ok(Value::Bool(!self.values_equal(&left_val, &right_val))),
                    TokenKind::Less => self.compare_values(left_val, right_val, |a, b| a < b),
                    TokenKind::Greater => self.compare_values(left_val, right_val, |a, b| a > b),
                    TokenKind::LessEqual => self.compare_values(left_val, right_val, |a, b| a <= b),
                    TokenKind::GreaterEqual => self.compare_values(left_val, right_val, |a, b| a >= b),

                    _ => Err(RuntimeError::Custom(format!("Unsupported binary operator: {:?}", op))),
                }
            }
        }
    }

    fn interpret_unary_op(&mut self, op: &TokenKind, expr: &Expr) -> Result<Value, RuntimeError> {
        let val = self.interpret_expression(expr)?;

        match op {
            TokenKind::Minus => match val {
                Value::Int(n) => Ok(Value::Int(-n)),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => Err(RuntimeError::TypeMismatch {
                    expected: "numeric".to_string(),
                    actual: val.type_name().to_string(),
                }),
            },
            TokenKind::Plus => match val {
                Value::Int(_) | Value::Float(_) => Ok(val),
                _ => Err(RuntimeError::TypeMismatch {
                    expected: "numeric".to_string(),
                    actual: val.type_name().to_string(),
                }),
            },
            TokenKind::LogicalNot => Ok(Value::Bool(!val.is_truthy())),
            TokenKind::BitNot => match val {
                Value::Int(n) => Ok(Value::Int(!n)),
                _ => Err(RuntimeError::TypeMismatch {
                    expected: "Int".to_string(),
                    actual: val.type_name().to_string(),
                }),
            },
            _ => Err(RuntimeError::Custom(format!("Unsupported unary operator: {:?}", op))),
        }
    }

    fn interpret_call(&mut self, callee: &Expr, args: &[Expr]) -> Result<Value, RuntimeError> {
        let callee_value = self.interpret_expression(callee)?;

        match callee_value {
            Value::ToolRef { name, params, body } => {
                if body.is_empty() {
                    return self.call_builtin(&name, args);
                }

                if args.len() != params.len() {
                    return Err(RuntimeError::InvalidArguments(
                        format!("Expected {} arguments, got {}", params.len(), args.len())
                    ));
                }

                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.interpret_expression(arg)?);
                }

                self.env.push_scope();
                self.env.enter_tool();

                for (param, arg_value) in params.iter().zip(arg_values.iter()) {
                    self.env.set(&param.name, arg_value.clone());
                }

                let mut result = Value::Null;
                for stmt in &body {
                    match self.interpret_statement(stmt)? {
                        ControlFlow::Return(value) => {
                            result = value;
                            break;
                        }
                        ControlFlow::Break => return Err(RuntimeError::BreakOutsideLoop),
                        ControlFlow::Continue => return Err(RuntimeError::ContinueOutsideLoop),
                        ControlFlow::None => {}
                    }
                }

                self.env.exit_tool();
                self.env.pop_scope();
                Ok(result)
            }
            _ => Err(RuntimeError::NotCallable),
        }
    }

    fn call_builtin(&mut self, name: &str, args: &[Expr]) -> Result<Value, RuntimeError> {
        match name {
            "print" => {
                for arg in args {
                    let value = self.interpret_expression(arg)?;
                    print!("{} ", value);
                }
                println!();
                Ok(Value::Null)
            }
            "panic" => {
                let message = if args.is_empty() {
                    "panic".to_string()
                } else {
                    let msg_value = self.interpret_expression(&args[0])?;
                    msg_value.to_string()
                };
                Err(RuntimeError::Custom(message))
            }
            "list" => {
                let mut items = Vec::new();
                for arg in args {
                    items.push(self.interpret_expression(arg)?);
                }
                Ok(Value::List(items))
            }
            "cons" => {
                if args.len() != 2 {
                    return Err(RuntimeError::InvalidArguments("cons requires 2 arguments".to_string()));
                }
                let head = self.interpret_expression(&args[0])?;
                let tail = self.interpret_expression(&args[1])?;

                match tail {
                    Value::List(mut items) => {
                        items.insert(0, head);
                        Ok(Value::List(items))
                    }
                    _ => {
                        Ok(Value::List(vec![head, tail]))
                    }
                }
            }
            "get" => {
                if args.len() != 2 {
                    return Err(RuntimeError::InvalidArguments("get requires 2 arguments".to_string()));
                }
                let list_val = self.interpret_expression(&args[0])?;
                let index_val = self.interpret_expression(&args[1])?;

                match (list_val, index_val) {
                    (Value::List(items), Value::Int(index)) => {
                        let idx = index as usize;
                        if idx < items.len() {
                            Ok(items[idx].clone())
                        } else {
                            Ok(Value::Null)
                        }
                    }
                    _ => Err(RuntimeError::TypeMismatch {
                        expected: "List and Int".to_string(),
                        actual: "other".to_string(),
                    }),
                }
            }
            "lookup" => {
                if args.len() != 2 {
                    return Err(RuntimeError::InvalidArguments("lookup requires 2 arguments".to_string()));
                }
                let obj_val = self.interpret_expression(&args[0])?;
                let key_val = self.interpret_expression(&args[1])?;

                match (obj_val, key_val) {
                    (Value::Object { fields, .. }, Value::String(key)) => {
                        Ok(fields.get(&key).cloned().unwrap_or(Value::Null))
                    }
                    _ => Err(RuntimeError::TypeMismatch {
                        expected: "Object and String".to_string(),
                        actual: "other".to_string(),
                    }),
                }
            }
            _ => Err(RuntimeError::UndefinedTool(name.to_string())),
        }
    }

    fn add_values(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "numeric or string".to_string(),
                actual: "other".to_string(),
            }),
        }
    }

    fn subtract_values(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - b as f64)),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: "other".to_string(),
            }),
        }
    }

    fn multiply_values(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * b as f64)),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: "other".to_string(),
            }),
        }
    }

    fn divide_values(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if b == 0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if b == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                if b == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a as f64 / b))
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                if b == 0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / b as f64))
                }
            }
            _ => Err(RuntimeError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: "something else you stupidly entered".to_string(),
            }),
        }
    }

    fn modulo_values(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if b == 0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Int(a % b))
                }
            }
            _ => Err(RuntimeError::TypeMismatch {
                expected: "Int".to_string(),
                actual: "something else you stupidly entered".to_string(),
            }),
        }
    }

    fn bitwise_and(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a & b)),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "Int".to_string(),
                actual: "something else you stupidly entered".to_string(),
            }),
        }
    }

    fn bitwise_or(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a | b)),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "Int".to_string(),
                actual: "something else you stupidly entered".to_string(),
            }),
        }
    }

    fn bitwise_xor(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a ^ b)),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "Int".to_string(),
                actual: "something else you stupidly entered".to_string(),
            }),
        }
    }

    fn shift_left(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a << b)),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "Int".to_string(),
                actual: "something else you stupidly entered".to_string(),
            }),
        }
    }

    fn shift_right(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a >> b)),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "Int".to_string(),
                actual: "something else you stupidly entered".to_string(),
            }),
        }
    }

    fn compare_values<F>(&self, left: Value, right: Value, op: F) -> Result<Value, RuntimeError>
    where
        F: Fn(f64, f64) -> bool,
    {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(op(a as f64, b as f64))),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(op(a, b))),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool(op(a as f64, b))),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(op(a, b as f64))),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: "something else you stupidly entered".to_string(),
            }),
        }
    }

    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Int(a), Value::Float(b)) => *a as f64 == *b,
            (Value::Float(a), Value::Int(b)) => *a == *b as f64,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }

    fn create_object_instance(&mut self, type_name: &str, field_inits: &[FieldInit]) -> Result<Value, RuntimeError> {
        let type_def = match self.env.type_definitions.get(type_name) {
            Some(def) => def.clone(),
            None => return Err(RuntimeError::UndefinedType(type_name.to_string())),
        };

        let mut fields = std::collections::HashMap::new();
        for field_init in field_inits {
            let value = self.interpret_expression(&field_init.value)?;
            fields.insert(field_init.name.clone(), value);
        }

        match type_def {
            TypeDef::Schema { fields: schema_fields, .. } => {
                for field in &schema_fields {
                    if !fields.contains_key(&field.name) {
                        return Err(RuntimeError::RequiredFieldMissing(format!("{}.{}", type_name, field.name)));
                    }
                }

                // TODO: Add type validation here?
                Ok(Value::Object {
                    type_name: type_name.to_string(),
                    fields,
                })
            }
            TypeDef::Struct { members, .. } => {
                for member in members {
                    if let StructMember::SchemaField(field) = member {
                        if !fields.contains_key(&field.name) {
                            return Err(RuntimeError::RequiredFieldMissing(format!("{}.{}", type_name, field.name)));
                        }
                    }
                }

                Ok(Value::Object {
                    type_name: type_name.to_string(),
                    fields,
                })
            }
            TypeDef::Model { members, .. } => {
                for member in members {
                    if let ModelMember::Assignment { target, .. } = member {
                        let field_name = &target[0];
                        if !fields.contains_key(field_name) {
                            return Err(RuntimeError::RequiredFieldMissing(format!("{}.{}", type_name, field_name)));
                        }
                    }
                }

                Ok(Value::Object {
                    type_name: type_name.to_string(),
                    fields,
                })
            }
            TypeDef::Template { .. } => {
                Err(RuntimeError::InvalidArguments(format!("Cannot instantiate template {}", type_name)))
            }
        }
    }
}