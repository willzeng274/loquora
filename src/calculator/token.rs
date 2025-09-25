use crate::calculator::ast::{Span, TokenKind};

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { span, kind }
    }

    pub fn text<'a>(&self, input: &'a str) -> &'a str {
        &input[self.span.clone()]
    }
}
