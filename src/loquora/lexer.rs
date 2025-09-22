use crate::loquora::token::{Token, TokenKind};

#[derive(Clone)]
pub struct Lexer {
    input: String,
    chars: Vec<char>,
    index: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let chars: Vec<char> = input.chars().collect();
        Lexer {
            input,
            chars,
            index: 0,
        }
    }

    pub fn source(&self) -> &str {
        &self.input
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn peek_n(&self, n: usize) -> Option<char> {
        self.chars.get(self.index + n).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        if ch.is_some() {
            self.index += 1;
        }
        ch
    }

    fn make_token(&self, kind: TokenKind, start: usize, end: usize) -> Token {
        Token::new(kind, start..end)
    }

    fn is_ident_start(ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_'
    }

    fn is_ident_continue(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_'
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) {
        while self.peek().is_some() {
            if self.peek() == Some('*') && self.peek_n(1) == Some('/') {
                self.advance();
                self.advance();
                break;
            }
            self.advance();
        }
    }

    fn lex_number(&mut self, start: usize) -> Token {
        let mut saw_dot = false;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.' && !saw_dot {
                saw_dot = true;
                self.advance();
            } else if ch == 'e' || ch == 'E' {
                self.advance();
                if self.peek() == Some('+') || self.peek() == Some('-') {
                    self.advance();
                }
                while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    self.advance();
                }
            } else {
                break;
            }
        }
        let end = self.index;
        if saw_dot {
            self.make_token(TokenKind::Float, start, end)
        } else {
            self.make_token(TokenKind::Int, start, end)
        }
    }

    fn lex_identifier_or_keyword(&mut self, start: usize) -> Token {
        while self.peek().map(Self::is_ident_continue).unwrap_or(false) {
            self.advance();
        }
        let end = self.index;
        let slice = &self.input[start..end];
        let kind = match slice {
            "import" => TokenKind::Import,
            "from" => TokenKind::From,
            "export" => TokenKind::Export,
            "schema" => TokenKind::Schema,
            "template" => TokenKind::Template,
            "model" => TokenKind::Model,
            "struct" => TokenKind::Struct,
            "tool" => TokenKind::Tool,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "elif" => TokenKind::Elif,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "loop" => TokenKind::Loop,
            "with" => TokenKind::With,
            "as" => TokenKind::As,
            "return" => TokenKind::Return,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            _ => TokenKind::Identifier,
        };
        self.make_token(kind, start, end)
    }

    fn lex_string(&mut self, start: usize) -> Token {
        // assumes the opening quote was already consumed by caller
        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.advance();
                    break;
                }
                '\\' => {
                    self.advance();
                    if self.peek().is_some() {
                        self.advance();
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }
        self.make_token(TokenKind::String, start, self.index)
    }

    fn lex_char(&mut self, start: usize) -> Token {
        if self.peek() == Some('\\') {
            self.advance();
            if self.peek().is_some() {
                self.advance();
            }
        } else {
            if self.peek().is_some() {
                self.advance();
            }
        }
        if self.peek() == Some('\'') {
            self.advance();
        }
        self.make_token(TokenKind::Char, start, self.index)
    }

    fn lex_heredoc(&mut self, _start: usize) -> Token {
        // After <<~, read delimiter (identifier), then read until a line that exactly matches it
        let delim_start = self.index;
        while let Some(c) = self.peek() {
            if Self::is_ident_continue(c) {
                self.advance();
            } else {
                break;
            }
        }
        let delim_end = self.index;
        let delimiter = self.input[delim_start..delim_end].to_string();
        let delim_len = delimiter.len();
        if self.peek() == Some('\n') {
            self.advance();
        }
        let body_start = self.index;
        let mut end_of_token = body_start;
        let total_len = self.chars.len();
        while self.index <= total_len {
            if self.index >= total_len {
                break;
            }
            let line_start = self.index;
            while self.index < total_len && self.chars[self.index] != '\n' {
                self.index += 1;
            }
            let line_end = self.index;
            let slice = &self.input[line_start..line_end];
            let is_delim_exact = (line_end - line_start) == delim_len && slice == delimiter;
            let is_delim_with_semicolon = (line_end - line_start) == delim_len + 1
                && &self.input[line_start..line_start + delim_len] == delimiter
                && &self.input[line_start + delim_len..line_end] == ";";
            let is_delim = is_delim_exact || is_delim_with_semicolon;
            if is_delim {
                if is_delim_with_semicolon {
                    let semicolon_pos = line_start + delim_len;
                    self.index = semicolon_pos;
                } else {
                    if self.index < total_len && self.chars[self.index] == '\n' {
                        self.index += 1;
                    }
                }
                break;
            } else {
                if self.index < total_len && self.chars[self.index] == '\n' {
                    self.index += 1;
                }
                end_of_token = self.index;
            }
        }
        self.make_token(TokenKind::MultilineString, body_start, end_of_token)
    }

    pub fn next_token(&mut self) -> Token {
        loop {
            self.skip_whitespace();
            let start = self.index;
            let ch = match self.peek() {
                Some(c) => c,
                None => return self.make_token(TokenKind::EOF, start, start),
            };

            if ch == '/' {
                if self.peek_n(1) == Some('/') {
                    self.advance();
                    self.advance();
                    self.skip_line_comment();
                    continue;
                } else if self.peek_n(1) == Some('*') {
                    self.advance();
                    self.advance();
                    self.skip_block_comment();
                    continue;
                }
            }

            if ch == '<' && self.peek_n(1) == Some('<') && self.peek_n(2) == Some('~') {
                self.advance();
                self.advance();
                self.advance();
                return self.lex_heredoc(start);
            }

            if ch.is_ascii_digit()
                || (ch == '.' && self.peek_n(1).map(|c| c.is_ascii_digit()).unwrap_or(false))
            {
                if ch == '.' {
                    self.advance();
                }
                return self.lex_number(start);
            }

            if Self::is_ident_start(ch) {
                self.advance();
                return self.lex_identifier_or_keyword(start);
            }

            if ch == '"' {
                self.advance();
                return self.lex_string(start);
            }

            if ch == '\'' {
                self.advance();
                return self.lex_char(start);
            }

            match (ch, self.peek_n(1)) {
                ('&', Some('&')) => {
                    self.advance();
                    self.advance();
                    return self.make_token(TokenKind::LogicalAnd, start, self.index);
                }
                ('|', Some('|')) => {
                    self.advance();
                    self.advance();
                    return self.make_token(TokenKind::LogicalOr, start, self.index);
                }
                ('=', Some('=')) => {
                    self.advance();
                    self.advance();
                    return self.make_token(TokenKind::EqualEqual, start, self.index);
                }
                ('!', Some('=')) => {
                    self.advance();
                    self.advance();
                    return self.make_token(TokenKind::NotEqual, start, self.index);
                }
                ('<', Some('=')) => {
                    self.advance();
                    self.advance();
                    return self.make_token(TokenKind::LessEqual, start, self.index);
                }
                ('>', Some('=')) => {
                    self.advance();
                    self.advance();
                    return self.make_token(TokenKind::GreaterEqual, start, self.index);
                }
                ('<', Some('<')) => {
                    self.advance();
                    self.advance();
                    return self.make_token(TokenKind::ShiftLeft, start, self.index);
                }
                ('>', Some('>')) => {
                    self.advance();
                    self.advance();
                    return self.make_token(TokenKind::ShiftRight, start, self.index);
                }
                ('?', Some('?')) => {
                    self.advance();
                    self.advance();
                    return self.make_token(TokenKind::QQuestion, start, self.index);
                }
                (':', Some(':')) => {
                    self.advance();
                    self.advance();
                    return self.make_token(TokenKind::DColon, start, self.index);
                }
                ('!', Some('!')) => {
                    self.advance();
                    self.advance();
                    return self.make_token(TokenKind::BangBang, start, self.index);
                }
                ('-', Some('>')) => {
                    self.advance();
                    self.advance();
                    return self.make_token(TokenKind::Arrow, start, self.index);
                }
                _ => {}
            }

            match ch {
                '+' => {
                    self.advance();
                    return self.make_token(TokenKind::Plus, start, self.index);
                }
                '-' => {
                    self.advance();
                    return self.make_token(TokenKind::Minus, start, self.index);
                }
                '*' => {
                    self.advance();
                    return self.make_token(TokenKind::Multiply, start, self.index);
                }
                '/' => {
                    self.advance();
                    return self.make_token(TokenKind::Divide, start, self.index);
                }
                '%' => {
                    self.advance();
                    return self.make_token(TokenKind::Modulo, start, self.index);
                }
                '@' => {
                    self.advance();
                    return self.make_token(TokenKind::At, start, self.index);
                }
                '&' => {
                    self.advance();
                    return self.make_token(TokenKind::BitAnd, start, self.index);
                }
                '|' => {
                    self.advance();
                    return self.make_token(TokenKind::BitOr, start, self.index);
                }
                '^' => {
                    self.advance();
                    return self.make_token(TokenKind::BitXor, start, self.index);
                }
                '~' => {
                    self.advance();
                    return self.make_token(TokenKind::BitNot, start, self.index);
                }
                '!' => {
                    self.advance();
                    return self.make_token(TokenKind::LogicalNot, start, self.index);
                }
                '=' => {
                    self.advance();
                    return self.make_token(TokenKind::Assign, start, self.index);
                }
                '<' => {
                    self.advance();
                    return self.make_token(TokenKind::Less, start, self.index);
                }
                '>' => {
                    self.advance();
                    return self.make_token(TokenKind::Greater, start, self.index);
                }
                '?' => {
                    self.advance();
                    return self.make_token(TokenKind::Question, start, self.index);
                }
                ':' => {
                    self.advance();
                    return self.make_token(TokenKind::Colon, start, self.index);
                }
                '.' => {
                    self.advance();
                    return self.make_token(TokenKind::Dot, start, self.index);
                }
                ',' => {
                    self.advance();
                    return self.make_token(TokenKind::Comma, start, self.index);
                }
                ';' => {
                    self.advance();
                    return self.make_token(TokenKind::Semicolon, start, self.index);
                }
                '(' => {
                    self.advance();
                    return self.make_token(TokenKind::LeftParen, start, self.index);
                }
                ')' => {
                    self.advance();
                    return self.make_token(TokenKind::RightParen, start, self.index);
                }
                '{' => {
                    self.advance();
                    return self.make_token(TokenKind::LeftBrace, start, self.index);
                }
                '}' => {
                    self.advance();
                    return self.make_token(TokenKind::RightBrace, start, self.index);
                }
                _ => {
                    self.advance();
                    continue;
                }
            }
        }
    }
}
