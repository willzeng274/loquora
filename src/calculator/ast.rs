use std::ops::Range;

pub type Span = Range<usize>;

#[derive(Clone, Debug, PartialEq)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(inner: T, span: Span) -> Self {
        Spanned { inner, span }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Int(i64),
    BinaryOp {
        op: TokenKind,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

pub type Expr = Spanned<ExprKind>;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Int,
    Float,
    Plus,
    Minus,
    Multiply,
    Divide,
    LeftParen,
    RightParen,
    EOF,
}