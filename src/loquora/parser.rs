use crate::loquora::ast::*;
use crate::loquora::lexer::Lexer;
use crate::loquora::token::{Span, Token, TokenKind};

pub struct Parser {
    lexer: Lexer,
    current: Token,
    input: String,
    in_tool: bool,
    in_loop: usize,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let input = lexer.source().to_string();
        let current = lexer.next_token();
        Parser {
            lexer,
            current,
            input,
            in_tool: false,
            in_loop: 0,
        }
    }

    fn advance(&mut self) {
        self.current = self.lexer.next_token();
    }

    fn eat(&mut self, expected: TokenKind) {
        if std::mem::discriminant(&self.current.kind) == std::mem::discriminant(&expected) {
            self.advance();
        } else {
            panic!("Expected {:?}, found {:?}", expected, self.current.kind);
        }
    }

    fn at(&self, kind: TokenKind) -> bool {
        std::mem::discriminant(&self.current.kind) == std::mem::discriminant(&kind)
    }

    pub fn parse_program(&mut self) -> Program {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.at(TokenKind::EOF) {
            let stmt = self.parse_top_level();
            statements.push(stmt);
        }
        Program { statements }
    }

    fn parse_top_level(&mut self) -> Stmt {
        if self.at(TokenKind::Load) {
            return self.parse_load_stmt_with_run(false);
        }
        if self.at(TokenKind::LoadAndRun) {
            return self.parse_load_stmt_with_run(true);
        }
        if self.at(TokenKind::Export) {
            return self.parse_export_decl();
        }
        if self.at(TokenKind::Template) {
            return self.parse_template_decl();
        }
        if self.at(TokenKind::Struct) {
            return self.parse_struct_decl();
        }
        if self.at(TokenKind::Tool) {
            return self.parse_tool_decl();
        }
        self.parse_statement()
    }

    fn slice_current<'a>(&'a self) -> &'a str {
        &self.input[self.current.span.clone()]
    }

    fn parse_load_stmt_with_run(&mut self, run: bool) -> Stmt {
        let start = self.current.span.start;
        if !run {
            self.eat(TokenKind::Load);
        } else {
            self.eat(TokenKind::LoadAndRun);
        }

        let mut path = Vec::new();
        if let TokenKind::Identifier = self.current.kind {
            path.push(self.slice_current().to_string());
            self.advance();
        } else {
            panic!("Expected module path after load");
        }

        while self.at(TokenKind::Divide) {
            self.advance();
            if let TokenKind::Identifier = self.current.kind {
                path.push(self.slice_current().to_string());
                self.advance();
            } else {
                panic!("Expected identifier after /");
            }
        }

        let alias = if self.at(TokenKind::As) {
            self.advance();
            if let TokenKind::Identifier = self.current.kind {
                let a = self.slice_current().to_string();
                self.advance();
                Some(a)
            } else {
                panic!("Expected alias identifier");
            }
        } else {
            None
        };
        if !run {
            self.eat(TokenKind::Semicolon);
            Spanned::new(
                StmtKind::Load { path, alias },
                start..self.current.span.start,
            )
        } else {
            self.eat(TokenKind::Semicolon);
            Spanned::new(
                StmtKind::LoadAndRun { path, alias },
                start..self.current.span.start,
            )
        }
    }

    fn parse_export_decl(&mut self) -> Stmt {
        let start = self.current.span.start;
        self.eat(TokenKind::Export);

        let decl = if self.at(TokenKind::Struct) {
            self.parse_struct_decl()
        } else if self.at(TokenKind::Tool) {
            self.parse_tool_decl()
        } else if self.at(TokenKind::Template) {
            self.parse_template_decl()
        } else {
            panic!("Expected struct, tool, or template after export");
        };

        Spanned::new(
            StmtKind::ExportDecl {
                decl: Box::new(decl),
            },
            start..self.current.span.start,
        )
    }

    fn is_assignment_start(&mut self) -> bool {
        if !self.at(TokenKind::Identifier) {
            return false;
        }
        let mut lx = self.lexer.clone();
        loop {
            let next = lx.next_token();
            if matches!(next.kind, TokenKind::Dot) {
                let after = lx.next_token();
                if matches!(after.kind, TokenKind::Identifier) {
                    continue;
                }
                return false;
            }
            return matches!(next.kind, TokenKind::Assign);
        }
    }

    fn parse_assignable_path(&mut self) -> (Vec<String>, Span) {
        let mut parts = Vec::new();
        let start = self.current.span.start;
        let mut end = start;
        loop {
            if let TokenKind::Identifier = self.current.kind {
                parts.push(self.slice_current().to_string());
                end = self.current.span.end;
                self.advance();
            } else {
                break;
            }
            if self.at(TokenKind::Dot) {
                self.advance();
            } else {
                break;
            }
        }
        (parts, start..end)
    }

    fn parse_expression(&mut self) -> Expr {
        self.parse_quaternary()
    }

    fn parse_quaternary(&mut self) -> Expr {
        let left = self.parse_ternary();
        if self.at(TokenKind::QQuestion) {
            self.advance();
            let if_true = self.parse_expression();
            self.eat(TokenKind::DColon);
            let if_false = self.parse_expression();
            self.eat(TokenKind::BangBang);
            let if_null = self.parse_quaternary();
            let start = left.span.start;
            let end = if_null.span.end;
            return Spanned::new(
                ExprKind::Quaternary {
                    cond: Box::new(left),
                    if_true: Box::new(if_true),
                    if_false: Box::new(if_false),
                    if_null: Box::new(if_null),
                },
                start..end,
            );
        }
        left
    }

    fn parse_ternary(&mut self) -> Expr {
        let cond = self.parse_logical_or();
        if self.at(TokenKind::Question) {
            self.advance();
            let if_true = self.parse_expression();
            self.eat(TokenKind::Colon);
            let if_false = self.parse_ternary();
            let start = cond.span.start;
            let end = if_false.span.end;
            return Spanned::new(
                ExprKind::Ternary {
                    cond: Box::new(cond),
                    if_true: Box::new(if_true),
                    if_false: Box::new(if_false),
                },
                start..end,
            );
        }
        cond
    }

    fn parse_statement(&mut self) -> Stmt {
        if self.at(TokenKind::With) {
            return self.parse_with_stmt();
        }
        if self.at(TokenKind::Loop) {
            return self.parse_loop_stmt();
        }
        if self.at(TokenKind::If) {
            return self.parse_if_stmt();
        }
        if self.at(TokenKind::While) {
            return self.parse_while_stmt();
        }
        if self.at(TokenKind::For) {
            return self.parse_for_stmt();
        }
        if self.at(TokenKind::Return) {
            return self.parse_return_stmt();
        }
        if self.at(TokenKind::Break) {
            return self.parse_break_stmt();
        }
        if self.at(TokenKind::Continue) {
            return self.parse_continue_stmt();
        }
        if self.is_assignment_start() {
            return self.parse_assignment_stmt();
        }
        self.parse_expr_stmt()
    }

    fn parse_with_stmt(&mut self) -> Stmt {
        let start = self.current.span.start;
        self.eat(TokenKind::With);
        let expr = self.parse_expression();
        self.eat(TokenKind::LeftBrace);
        let body = self.parse_statements_until(TokenKind::RightBrace);
        self.eat(TokenKind::RightBrace);
        self.eat(TokenKind::Semicolon);
        Spanned::new(
            StmtKind::With { expr, body },
            start..self.current.span.start,
        )
    }

    fn parse_loop_stmt(&mut self) -> Stmt {
        let start = self.current.span.start;
        self.eat(TokenKind::Loop);
        self.eat(TokenKind::LeftBrace);
        self.in_loop += 1;
        let body = self.parse_loop_body_until();
        self.in_loop -= 1;
        self.eat(TokenKind::RightBrace);
        Spanned::new(StmtKind::Loop { body }, start..self.current.span.start)
    }

    fn parse_if_stmt(&mut self) -> Stmt {
        let start = self.current.span.start;
        let mut arms: Vec<(Expr, Vec<Stmt>)> = Vec::new();
        self.eat(TokenKind::If);
        let cond = self.parse_expression();
        self.eat(TokenKind::LeftBrace);
        let then_body = self.parse_statements_until(TokenKind::RightBrace);
        self.eat(TokenKind::RightBrace);
        arms.push((cond, then_body));
        while self.at(TokenKind::Elif) {
            self.eat(TokenKind::Elif);
            let c = self.parse_expression();
            self.eat(TokenKind::LeftBrace);
            let b = self.parse_statements_until(TokenKind::RightBrace);
            self.eat(TokenKind::RightBrace);
            arms.push((c, b));
        }
        let else_body = if self.at(TokenKind::Else) {
            self.eat(TokenKind::Else);
            self.eat(TokenKind::LeftBrace);
            let b = self.parse_statements_until(TokenKind::RightBrace);
            self.eat(TokenKind::RightBrace);
            Some(b)
        } else {
            None
        };
        Spanned::new(
            StmtKind::If { arms, else_body },
            start..self.current.span.start,
        )
    }

    fn parse_while_stmt(&mut self) -> Stmt {
        let start = self.current.span.start;
        self.eat(TokenKind::While);
        let cond = self.parse_expression();
        self.eat(TokenKind::LeftBrace);
        self.in_loop += 1;
        let body = self.parse_loop_body_until();
        self.in_loop -= 1;
        self.eat(TokenKind::RightBrace);
        Spanned::new(
            StmtKind::While { cond, body },
            start..self.current.span.start,
        )
    }

    fn parse_for_stmt(&mut self) -> Stmt {
        let start = self.current.span.start;
        self.eat(TokenKind::For);
        let var = if let TokenKind::Identifier = self.current.kind {
            let v = self.slice_current().to_string();
            self.advance();
            v
        } else {
            panic!("Expected identifier after for");
        };
        self.eat(TokenKind::In);
        let iter = self.parse_expression();
        self.eat(TokenKind::LeftBrace);
        self.in_loop += 1;
        let body = self.parse_loop_body_until();
        self.in_loop -= 1;
        self.eat(TokenKind::RightBrace);
        Spanned::new(
            StmtKind::For { var, iter, body },
            start..self.current.span.start,
        )
    }

    fn parse_return_stmt(&mut self) -> Stmt {
        let start = self.current.span.start;
        self.eat(TokenKind::Return);
        let expr = if self.at(TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression())
        };
        self.eat(TokenKind::Semicolon);
        Spanned::new(StmtKind::Return { expr }, start..self.current.span.start)
    }

    fn parse_break_stmt(&mut self) -> Stmt {
        let start = self.current.span.start;
        self.eat(TokenKind::Break);
        self.eat(TokenKind::Semicolon);
        Spanned::new(StmtKind::Break, start..self.current.span.start)
    }

    fn parse_continue_stmt(&mut self) -> Stmt {
        let start = self.current.span.start;
        self.eat(TokenKind::Continue);
        self.eat(TokenKind::Semicolon);
        Spanned::new(StmtKind::Continue, start..self.current.span.start)
    }

    fn parse_assignment_stmt(&mut self) -> Stmt {
        let start = self.current.span.start;
        let (target, _) = self.parse_assignable_path();
        self.eat(TokenKind::Assign);
        let value = self.parse_expression();
        self.eat(TokenKind::Semicolon);
        Spanned::new(
            StmtKind::Assignment { target, value },
            start..self.current.span.start,
        )
    }

    fn parse_expr_stmt(&mut self) -> Stmt {
        let start = self.current.span.start;
        let expr = self.parse_expression();
        self.eat(TokenKind::Semicolon);
        Spanned::new(StmtKind::ExprStmt { expr }, start..self.current.span.start)
    }

    fn parse_statements_until(&mut self, end: TokenKind) -> Vec<Stmt> {
        let mut v = Vec::new();
        while !self.at(end.clone()) && !self.at(TokenKind::EOF) {
            v.push(self.parse_statement());
        }
        v
    }

    fn parse_loop_body_until(&mut self) -> Vec<Stmt> {
        let mut v = Vec::new();
        while !self.at(TokenKind::RightBrace) && !self.at(TokenKind::EOF) {
            if self.at(TokenKind::Break) {
                v.push(self.parse_break_stmt());
                continue;
            }
            if self.at(TokenKind::Continue) {
                v.push(self.parse_continue_stmt());
                continue;
            }
            v.push(self.parse_statement());
        }
        v
    }

    fn parse_type_expr(&mut self) -> TypeExpr {
        let start = self.current.span.start;
        let name = match self.current.kind {
            TokenKind::Identifier => {
                let s = self.slice_current().to_string();
                self.advance();
                s
            }
            _ => panic!("type name expected"),
        };
        if self.at(TokenKind::Less) {
            self.eat(TokenKind::Less);
            let mut params: Vec<TypeExpr> = Vec::new();
            if !self.at(TokenKind::Greater) {
                loop {
                    params.push(self.parse_type_expr());
                    if self.at(TokenKind::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
            self.eat(TokenKind::Greater);
            return Spanned::new(
                TypeExprKind::Generic { name, params },
                start..self.current.span.start,
            );
        }
        Spanned::new(TypeExprKind::Name(name), start..self.current.span.start)
    }

    fn parse_param_list(&mut self) -> Vec<ParamDecl> {
        let mut params = Vec::new();
        if self.at(TokenKind::RightParen) {
            return params;
        }
        loop {
            let name = match self.current.kind {
                TokenKind::Identifier => {
                    let s = self.slice_current().to_string();
                    self.advance();
                    s
                }
                _ => panic!("param name expected"),
            };
            self.eat(TokenKind::Colon);
            let ty = self.parse_type_expr();
            params.push(ParamDecl { name, ty });
            if self.at(TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        params
    }

    fn parse_template_decl(&mut self) -> Stmt {
        let start = self.current.span.start;
        self.eat(TokenKind::Template);
        let name = match self.current.kind {
            TokenKind::Identifier => {
                let s = self.slice_current().to_string();
                self.advance();
                s
            }
            _ => panic!("template name expected"),
        };
        self.eat(TokenKind::LeftParen);
        let params = self.parse_param_list();
        self.eat(TokenKind::RightParen);
        self.eat(TokenKind::LeftBrace);
        let body = match self.current.kind {
            TokenKind::String => {
                let s = self.slice_current().trim_matches('"').to_string();
                self.advance();
                s
            }
            TokenKind::MultilineString => {
                let s = self.slice_current().to_string();
                self.advance();
                s
            }
            _ => panic!("template body expected"),
        };
        self.eat(TokenKind::RightBrace);
        self.eat(TokenKind::Semicolon);
        Spanned::new(
            StmtKind::TemplateDecl { name, params, body },
            start..self.current.span.start,
        )
    }

    fn parse_struct_decl(&mut self) -> Stmt {
        let start = self.current.span.start;
        self.eat(TokenKind::Struct);
        let name = match self.current.kind {
            TokenKind::Identifier => {
                let s = self.slice_current().to_string();
                self.advance();
                s
            }
            _ => panic!("struct name expected"),
        };
        self.eat(TokenKind::LeftBrace);
        let mut members: Vec<StructMember> = Vec::new();
        while !self.at(TokenKind::RightBrace) {
            if self.at(TokenKind::Tool) {
                let (n, p, r, b) = self.parse_tool_decl_inner();
                members.push(StructMember::ToolDecl {
                    name: n,
                    params: p,
                    return_type: r,
                    body: b,
                });
                self.eat(TokenKind::Semicolon);
            } else {
                let fname = match self.current.kind {
                    TokenKind::Identifier => {
                        let s = self.slice_current().to_string();
                        self.advance();
                        s
                    }
                    _ => panic!("field name expected"),
                };
                self.eat(TokenKind::Colon);
                let ty = self.parse_type_expr();
                let suffix = if self.at(TokenKind::Question) {
                    self.advance();
                    if self.at(TokenKind::LogicalNot) {
                        self.advance();
                        Some("?!".to_string())
                    } else {
                        Some("?".to_string())
                    }
                } else if self.at(TokenKind::LogicalNot) {
                    self.advance();
                    Some("!".to_string())
                } else {
                    None
                };
                if self.at(TokenKind::Comma) {
                    self.advance();
                }
                members.push(StructMember::Field(StructField {
                    name: fname,
                    ty,
                    suffix,
                }));
            }
        }
        self.eat(TokenKind::RightBrace);
        Spanned::new(
            StmtKind::StructDecl { name, members },
            start..self.current.span.start,
        )
    }

    fn parse_tool_decl(&mut self) -> Stmt {
        let start = self.current.span.start;
        let (name, params, ret, body) = self.parse_tool_decl_inner();
        // Tool declarations don't end with semicolons, they end with }
        Spanned::new(
            StmtKind::ToolDecl {
                name,
                params,
                return_type: ret,
                body,
            },
            start..self.current.span.start,
        )
    }

    fn parse_tool_decl_inner(&mut self) -> (String, Vec<ParamDecl>, Option<TypeExpr>, Vec<Stmt>) {
        self.eat(TokenKind::Tool);
        let name = match self.current.kind {
            TokenKind::Identifier => {
                let s = self.slice_current().to_string();
                self.advance();
                s
            }
            _ => panic!("tool name expected"),
        };
        self.eat(TokenKind::LeftParen);
        let params = self.parse_param_list();
        self.eat(TokenKind::RightParen);
        let ret = if self.at(TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type_expr())
        } else {
            None
        };
        self.eat(TokenKind::LeftBrace);
        let was_in_tool = self.in_tool;
        self.in_tool = true;
        let body = self.parse_statements_until(TokenKind::RightBrace);
        self.in_tool = was_in_tool;
        self.eat(TokenKind::RightBrace);
        (name, params, ret, body)
    }

    fn parse_logical_or(&mut self) -> Expr {
        self.parse_left_assoc_bin(|p| p.parse_logical_and(), &[TokenKind::LogicalOr])
    }
    fn parse_logical_and(&mut self) -> Expr {
        self.parse_left_assoc_bin(|p| p.parse_bitwise_or(), &[TokenKind::LogicalAnd])
    }
    fn parse_bitwise_or(&mut self) -> Expr {
        self.parse_left_assoc_bin(|p| p.parse_bitwise_xor(), &[TokenKind::BitOr])
    }
    fn parse_bitwise_xor(&mut self) -> Expr {
        self.parse_left_assoc_bin(|p| p.parse_bitwise_and(), &[TokenKind::BitXor])
    }
    fn parse_bitwise_and(&mut self) -> Expr {
        self.parse_left_assoc_bin(|p| p.parse_equality(), &[TokenKind::BitAnd])
    }
    fn parse_equality(&mut self) -> Expr {
        self.parse_left_assoc_bin(
            |p| p.parse_relational(),
            &[TokenKind::EqualEqual, TokenKind::NotEqual],
        )
    }
    fn parse_relational(&mut self) -> Expr {
        self.parse_left_assoc_bin(
            |p| p.parse_shift(),
            &[
                TokenKind::Less,
                TokenKind::Greater,
                TokenKind::LessEqual,
                TokenKind::GreaterEqual,
            ],
        )
    }
    fn parse_shift(&mut self) -> Expr {
        self.parse_left_assoc_bin(
            |p| p.parse_additive(),
            &[TokenKind::ShiftLeft, TokenKind::ShiftRight],
        )
    }
    fn parse_additive(&mut self) -> Expr {
        self.parse_left_assoc_bin(
            |p| p.parse_multiplicative(),
            &[TokenKind::Plus, TokenKind::Minus],
        )
    }
    fn parse_multiplicative(&mut self) -> Expr {
        self.parse_left_assoc_bin(
            |p| p.parse_unary(),
            &[
                TokenKind::Multiply,
                TokenKind::Divide,
                TokenKind::Modulo,
                TokenKind::At,
            ],
        )
    }

    fn parse_left_assoc_bin<F>(&mut self, mut sub: F, ops: &[TokenKind]) -> Expr
    where
        F: FnMut(&mut Parser) -> Expr,
    {
        let mut node = sub(self);
        loop {
            let mut matched = None;
            for op in ops {
                if self.at(op.clone()) {
                    matched = Some(op.clone());
                    break;
                }
            }
            if let Some(opkind) = matched {
                let start = node.span.start;
                self.advance();
                let right = sub(self);
                let end = right.span.end;
                node = Spanned::new(
                    ExprKind::BinaryOp {
                        op: opkind,
                        left: Box::new(node),
                        right: Box::new(right),
                    },
                    start..end,
                );
            } else {
                break;
            }
        }
        node
    }

    fn parse_unary(&mut self) -> Expr {
        if self.at(TokenKind::BitNot)
            || self.at(TokenKind::Minus)
            || self.at(TokenKind::Plus)
            || self.at(TokenKind::LogicalNot)
        {
            let op = self.current.kind.clone();
            let start = self.current.span.start;
            self.advance();
            let expr = self.parse_postfix();
            let end = expr.span.end;
            return Spanned::new(
                ExprKind::UnaryOp {
                    op,
                    expr: Box::new(expr),
                },
                start..end,
            );
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Expr {
        let mut node = self.parse_primary();
        loop {
            if self.at(TokenKind::Dot) {
                self.advance();
                let name = match self.current.kind {
                    TokenKind::Identifier => {
                        let s = self.slice_current().to_string();
                        self.advance();
                        s
                    }
                    _ => panic!("property expected"),
                };

                if self.at(TokenKind::LeftBrace) {
                    let mut peek_lexer = self.lexer.clone();
                    let next_after_brace = peek_lexer.next_token();
                    let is_object_init = match next_after_brace.kind {
                        TokenKind::RightBrace => true,
                        TokenKind::Identifier => {
                            let token_after_id = peek_lexer.next_token();
                            matches!(token_after_id.kind, TokenKind::Colon)
                        }
                        _ => false,
                    };

                    if is_object_init {
                        let type_expr = Spanned::new(
                            ExprKind::Property {
                                object: Box::new(node.clone()),
                                property: name,
                            },
                            node.span.start..self.current.span.start,
                        );
                        let fields = self.parse_field_init_list();
                        let start = node.span.start;
                        let end = self.current.span.start;
                        node = Spanned::new(
                            ExprKind::ObjectInit {
                                type_expr: Box::new(type_expr),
                                fields,
                            },
                            start..end,
                        );
                        continue;
                    }
                }

                let start = node.span.start;
                let end = self.current.span.start;
                node = Spanned::new(
                    ExprKind::Property {
                        object: Box::new(node),
                        property: name,
                    },
                    start..end,
                );
                continue;
            }
            if self.at(TokenKind::LeftParen) {
                self.advance();
                let mut args: Vec<Expr> = Vec::new();
                if !self.at(TokenKind::RightParen) {
                    loop {
                        let e = self.parse_expression();
                        args.push(e);
                        if self.at(TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                let endtok = self.current.span.end;
                self.eat(TokenKind::RightParen);
                let start = node.span.start;
                node = Spanned::new(
                    ExprKind::Call {
                        callee: Box::new(node),
                        args,
                    },
                    start..endtok,
                );
                continue;
            }
            break;
        }
        node
    }

    fn parse_primary(&mut self) -> Expr {
        match self.current.kind {
            TokenKind::Identifier => {
                let start = self.current.span.start;
                let s = self.slice_current().to_string();
                self.advance();

                if self.at(TokenKind::LeftBrace) {
                    let mut peek_lexer = self.lexer.clone();
                    let next_after_brace = peek_lexer.next_token();
                    let is_object_init = match next_after_brace.kind {
                        TokenKind::RightBrace => true,
                        TokenKind::Identifier => {
                            let token_after_id = peek_lexer.next_token();
                            matches!(token_after_id.kind, TokenKind::Colon)
                        }
                        _ => false,
                    };

                    if is_object_init {
                        let type_expr = Box::new(Spanned::new(
                            ExprKind::Identifier(s.clone()),
                            start..self.current.span.start,
                        ));
                        let fields = self.parse_field_init_list();
                        let end = self.current.span.start;
                        Spanned::new(ExprKind::ObjectInit { type_expr, fields }, start..end)
                    } else {
                        let end = self.current.span.start;
                        Spanned::new(ExprKind::Identifier(s), start..end)
                    }
                } else {
                    let end = self.current.span.start;
                    Spanned::new(ExprKind::Identifier(s), start..end)
                }
            }
            TokenKind::Int => {
                let start = self.current.span.start;
                let n = self.slice_current().parse::<i64>().unwrap();
                let end = self.current.span.end;
                self.advance();
                Spanned::new(ExprKind::Int(n), start..end)
            }
            TokenKind::Float => {
                let start = self.current.span.start;
                let n = self.slice_current().parse::<f64>().unwrap();
                let end = self.current.span.end;
                self.advance();
                Spanned::new(ExprKind::Float(n), start..end)
            }
            TokenKind::String => {
                let start = self.current.span.start;
                let s = self.slice_current().trim_matches('"').to_string();
                let end = self.current.span.end;
                self.advance();
                Spanned::new(ExprKind::String(s), start..end)
            }
            TokenKind::MultilineString => {
                let start = self.current.span.start;
                let mut s = self.slice_current().to_string();
                // Remove trailing newline from heredoc strings
                if s.ends_with('\n') {
                    s.pop();
                }
                let end = self.current.span.end;
                self.advance();
                Spanned::new(ExprKind::String(s), start..end)
            }
            TokenKind::Char => {
                let start = self.current.span.start;
                let raw = self.slice_current();
                let ch = raw.trim_matches('\'').chars().next().unwrap_or('\0');
                let end = self.current.span.end;
                self.advance();
                Spanned::new(ExprKind::Char(ch), start..end)
            }
            TokenKind::True => {
                let start = self.current.span.start;
                self.advance();
                Spanned::new(ExprKind::Bool(true), start..self.current.span.start)
            }
            TokenKind::False => {
                let start = self.current.span.start;
                self.advance();
                Spanned::new(ExprKind::Bool(false), start..self.current.span.start)
            }
            TokenKind::Null => {
                let start = self.current.span.start;
                self.advance();
                Spanned::new(ExprKind::Null, start..self.current.span.start)
            }
            TokenKind::LeftParen => {
                self.eat(TokenKind::LeftParen);
                let e = self.parse_expression();
                self.eat(TokenKind::RightParen);
                e
            }
            _ => panic!(
                "primary expected, found {:?} at span {:?}",
                self.current.kind, self.current.span
            ),
        }
    }

    fn parse_field_init_list(&mut self) -> Vec<FieldInit> {
        self.eat(TokenKind::LeftBrace);
        let mut fields = Vec::new();

        if !self.at(TokenKind::RightBrace) {
            loop {
                let field_name = if let TokenKind::Identifier = self.current.kind {
                    let name = self.slice_current().to_string();
                    self.advance();
                    name
                } else {
                    panic!("Expected field name, found {:?}", self.current.kind);
                };

                self.eat(TokenKind::Colon);
                let value = self.parse_expression();

                fields.push(FieldInit {
                    name: field_name,
                    value,
                });

                if self.at(TokenKind::Comma) {
                    self.advance();
                    if self.at(TokenKind::RightBrace) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        self.eat(TokenKind::RightBrace);
        fields
    }
}
