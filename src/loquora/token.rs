use std::ops::Range;

pub type Span = Range<usize>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    // Literals
    Int,
    Float,
    String,
    Char,
    True,
    False,
    Null,

    // Identifiers
    Identifier,

    // Keywords
    Import,
    From,
    Export,
    Schema,
    Template,
    Model,
    Struct,
    Tool,
    If,
    Else,
    Elif,
    While,
    For,
    Loop,
    With,
    As,
    Return,
    Break,
    Continue,

    // Operators
    Plus,        // +
    Minus,       // -
    Multiply,    // *
    Divide,      // /
    Modulo,      // %
    At,          // @
    BitAnd,      // &
    BitOr,       // |
    BitXor,      // ^
    BitNot,      // ~
    LogicalNot,  // !
    LogicalAnd,  // &&
    LogicalOr,   // ||
    EqualEqual,  // ==
    NotEqual,    // !=
    Less,        // <
    Greater,     // >
    LessEqual,   // <=
    GreaterEqual,// >=
    ShiftLeft,   // <<
    ShiftRight,  // >>
    Assign,      // =
    Arrow,       // ->

    // Quaternary and ternary parts
    Question,    // ?
    Colon,       // :
    QQuestion,   // ??
    DColon,      // ::
    BangBang,    // !!

    // Punctuation
    Dot,         // .
    Comma,       // ,
    Semicolon,   // ;
    LeftParen,   // (
    RightParen,  // )
    LeftBrace,   // {
    RightBrace,  // }

    MultilineString, // <<~...delimiter

    // End of input
    EOF,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }
}


