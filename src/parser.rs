use crate::ast::{Expr, ExprKind, Spanned, TokenKind};
use crate::lexer::Lexer;
use crate::token::Token;

pub struct Parser {
    lexer: Lexer,
    current_token: Option<Token>,
    input: String,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Parser {
        let input = lexer.text.clone();
        let current_token = Some(lexer.get_next_token());
        
        Parser {
            lexer,
            current_token,
            input,
        }
    }

    fn eat(&mut self, expected_kind: TokenKind) {
        let curr_token = self.current_token.clone().expect("Invalid syntax");

        if std::mem::discriminant(&expected_kind) == std::mem::discriminant(&curr_token.kind) {
            self.current_token = Some(self.lexer.get_next_token());
        } else {
            panic!("Invalid syntax");
        }
    }

    fn factor(&mut self) -> Expr {
        let token = self.current_token.clone().unwrap();

        match &token.kind {
            TokenKind::Int => {
                self.eat(TokenKind::Int);
                let value = token.text(&self.input).parse::<i64>().unwrap();
                Spanned::new(ExprKind::Int(value), token.span)
            }
            TokenKind::Float => {
                self.eat(TokenKind::Float);
                let value = token.text(&self.input).parse::<f64>().unwrap() as i64;
                Spanned::new(ExprKind::Int(value), token.span)
            }
            TokenKind::LeftParen => {
                self.eat(TokenKind::LeftParen);
                let node = self.expr();
                self.eat(TokenKind::RightParen);
                node
            }
            _ => panic!("Invalid syntax"),
        }
    }

    fn term(&mut self) -> Expr {
        let mut node = self.factor();

        while let Some(ref curr_token) = self.current_token.clone() {
            match curr_token.kind {
                TokenKind::Multiply | TokenKind::Divide => {
                    let op = curr_token.kind.clone();
                    let start_span = node.span.start;
                    self.eat(op.clone());
                    let right = self.factor();
                    let end_span = right.span.end;
                    node = Spanned::new(
                        ExprKind::BinaryOp {
                            op,
                            left: Box::new(node),
                            right: Box::new(right),
                        },
                        start_span..end_span,
                    );
                }
                _ => break,
            }
        }

        node
    }

    fn expr(&mut self) -> Expr {
        let mut node = self.term();

        while let Some(ref curr_token) = self.current_token.clone() {
            match curr_token.kind {
                TokenKind::Plus | TokenKind::Minus => {
                    let op = curr_token.kind.clone();
                    let start_span = node.span.start;
                    self.eat(op.clone());
                    let right = self.term();
                    let end_span = right.span.end;
                    node = Spanned::new(
                        ExprKind::BinaryOp {
                            op,
                            left: Box::new(node),
                            right: Box::new(right),
                        },
                        start_span..end_span,
                    );
                }
                _ => break,
            }
        }

        node
    }

    pub fn parse(&mut self) -> Expr {
        self.expr()
    }
}