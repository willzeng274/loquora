use crate::ast::TokenKind;
use crate::token::Token;

pub struct Lexer {
    pub text: String,
    pos: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(text: String) -> Lexer {
        Lexer {
            text: text.clone(),
            pos: 0,
            current_char: text.chars().next(),
        }
    }

    fn advance(&mut self) {
        self.pos += 1;

        if self.pos > self.text.len() - 1 {
            self.current_char = None;
        }

        self.current_char = self.text.chars().nth(self.pos);
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn number(&mut self) -> Token {
        let start = self.pos;
        let mut has_dot = false;

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.' && !has_dot {
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }

        let span = start..self.pos;
        let kind = if has_dot { TokenKind::Float } else { TokenKind::Int };
        Token::new(kind, span)
    }

    pub fn get_next_token(&mut self) -> Token {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.skip_whitespace();
                continue;
            }

            if ch.is_ascii_digit() {
                return self.number();
            }

            let start = self.pos;
            match ch {
                '+' => {
                    self.advance();
                    return Token::new(TokenKind::Plus, start..self.pos);
                }
                '-' => {
                    self.advance();
                    return Token::new(TokenKind::Minus, start..self.pos);
                }
                '*' => {
                    self.advance();
                    return Token::new(TokenKind::Multiply, start..self.pos);
                }
                '/' => {
                    self.advance();
                    return Token::new(TokenKind::Divide, start..self.pos);
                }
                '(' => {
                    self.advance();
                    return Token::new(TokenKind::LeftParen, start..self.pos);
                }
                ')' => {
                    self.advance();
                    return Token::new(TokenKind::RightParen, start..self.pos);
                }
                _ => panic!("Invalid character at {}", self.pos),
            }
        }

        Token::new(TokenKind::EOF, self.pos..self.pos)
    }
}