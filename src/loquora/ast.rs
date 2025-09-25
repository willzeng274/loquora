use crate::loquora::token::{Span, TokenKind};

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
    Identifier(String),
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    Null,
    BinaryOp {
        op: TokenKind,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: TokenKind,
        expr: Box<Expr>,
    },
    Ternary {
        cond: Box<Expr>,
        if_true: Box<Expr>,
        if_false: Box<Expr>,
    },
    Quaternary {
        cond: Box<Expr>,
        if_true: Box<Expr>,
        if_false: Box<Expr>,
        if_null: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Property {
        object: Box<Expr>,
        property: String,
    },
    ObjectInit {
        type_name: String,
        fields: Vec<FieldInit>,
    },
}

pub type Expr = Spanned<ExprKind>;

#[derive(Clone, Debug, PartialEq)]
pub enum TypeExprKind {
    Name(String),
    Generic { name: String, params: Vec<TypeExpr> },
}

pub type TypeExpr = Spanned<TypeExprKind>;

#[derive(Clone, Debug, PartialEq)]
pub enum ImportItem {
    Identifier(String),
    String(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExportItem {
    Identifier(String),
    String(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParamDecl {
    pub name: String,
    pub ty: TypeExpr,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StmtKind {
    ImportModule {
        module: Vec<String>,
    },
    ImportFrom {
        module: Vec<String>,
        items: Vec<ImportItem>,
    },
    Export {
        items: Vec<ExportItem>,
    },
    SchemaDecl {
        name: String,
        fields: Vec<SchemaField>,
    },
    StructDecl {
        name: String,
        members: Vec<StructMember>,
    },
    TemplateDecl {
        name: String,
        params: Vec<ParamDecl>,
        body: String,
    },
    ModelDecl {
        name: String,
        base: Option<String>,
        members: Vec<ModelMember>,
    },
    ToolDecl {
        name: String,
        params: Vec<ParamDecl>,
        return_type: Option<TypeExpr>,
        body: Vec<Stmt>,
    },
    Assignment {
        target: Vec<String>,
        value: Expr,
    },
    ExprStmt {
        expr: Expr,
    },
    With {
        expr: Expr,
        body: Vec<Stmt>,
    },
    Loop {
        body: Vec<Stmt>,
    },
    If {
        arms: Vec<(Expr, Vec<Stmt>)>,
        else_body: Option<Vec<Stmt>>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    For {
        init: Option<(Vec<String>, Expr)>,
        cond: Option<Expr>,
        step: Option<Expr>,
        body: Vec<Stmt>,
    },
    Return {
        expr: Option<Expr>,
    },
    Break,
    Continue,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SchemaField {
    pub name: String,
    pub ty: TypeExpr,
    pub suffix: Option<String>, // ?, !, or ?!
}

#[derive(Clone, Debug, PartialEq)]
pub enum StructMember {
    SchemaField(SchemaField),
    ToolDecl {
        name: String,
        params: Vec<ParamDecl>,
        return_type: Option<TypeExpr>,
        body: Vec<Stmt>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ModelMember {
    ToolDecl {
        name: String,
        params: Vec<ParamDecl>,
        return_type: Option<TypeExpr>,
        body: Vec<Stmt>,
    },
    Assignment {
        target: Vec<String>,
        value: Expr,
    },
}

pub type Stmt = Spanned<StmtKind>;

#[derive(Clone, Debug, PartialEq)]
pub struct FieldInit {
    pub name: String,
    pub value: Expr,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
