use crate::ast::{Expr, ExprKind, TokenKind};
use crate::parser::Parser;

pub struct Interpreter {
    parser: Parser,
}

impl Interpreter {
    pub fn new(parser: Parser) -> Interpreter {
        Interpreter { parser }
    }

    fn visit(&self, node: &Expr) -> f64 {
        match &node.inner {
            ExprKind::Int(value) => *value as f64,
            ExprKind::BinaryOp { op, left, right } => {
                let left_val = self.visit(left);
                let right_val = self.visit(right);
                
                match op {
                    TokenKind::Plus => left_val + right_val,
                    TokenKind::Minus => left_val - right_val,
                    TokenKind::Multiply => left_val * right_val,
                    TokenKind::Divide => left_val / right_val,
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn interpret(&mut self) -> f64 {
        let tree = self.parser.parse();
        let result = self.visit(&tree);

        result
    }
}