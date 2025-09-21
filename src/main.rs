use std::io;
use std::io::Write;

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Integer(i32),
    Plus,
    Minus,
    Multiply,
    Divide,
    LeftParen,
    RightParen,
    EOF,
}

pub struct Lexer {
    text: String,
    pos: usize,
    current_char: Option<char>,
}

impl Lexer {
    fn new(text: String) -> Lexer {
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

    fn integer(&mut self) -> i32 {
        let mut result = 0;

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                result = result * 10 + (ch as i32 - '0' as i32);
                self.advance();
            } else {
                break;
            }
        }

        result
    }

    fn get_next_token(&mut self) -> Token {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.skip_whitespace();
                continue;
            }

            if ch.is_ascii_digit() {
                return Token::Integer(self.integer());
            }

            match ch {
                '+' => {
                    self.advance();
                    return Token::Plus;
                }
                '-' => {
                    self.advance();
                    return Token::Minus;
                }
                '*' => {
                    self.advance();
                    return Token::Multiply;
                }
                '/' => {
                    self.advance();
                    return Token::Divide;
                }
                '(' => {
                    self.advance();
                    return Token::LeftParen;
                }
                ')' => {
                    self.advance();
                    return Token::RightParen;
                }
                _ => panic!("Invalid character at {}", self.pos),
            }
        }

        Token::EOF
    }
}

struct AST {
    token: Token,
    children: Vec<AST>,
}

impl AST {
    fn new(token: Token, children: Vec<Self>) -> Self {
        AST { token, children }
    }
}

pub struct Parser {
    lexer: Lexer,
    current_token: Option<Token>,
}

impl Parser {
    fn new(lexer: Lexer) -> Parser {
        let mut parser = Parser {
            lexer,
            current_token: None,
        };

        parser.current_token = Some(parser.lexer.get_next_token());

        parser
    }

    fn eat(&mut self, expected: Token) {
        let curr_token = self.current_token.clone().expect("Invalid syntax");

        if std::mem::discriminant(&expected) == std::mem::discriminant(&curr_token) {
            self.current_token = Some(self.lexer.get_next_token());
        } else {
            panic!("Invalid syntax");
        }
    }

    fn factor(&mut self) -> AST {
        let token = self.current_token.clone().unwrap();

        match &token {
            Token::Integer(_) => {
                self.eat(Token::Integer(0));
                AST::new(token, vec![])
            }
            Token::LeftParen => {
                self.eat(Token::LeftParen);
                let node = self.expr();
                self.eat(Token::RightParen);

                node
            }
            _ => panic!("Invalid syntax"),
        }
    }

    fn term(&mut self) -> AST {
        let mut node = self.factor();

        while let Some(ref curr_token) = self.current_token.clone() {
            match curr_token {
                Token::Multiply => {
                    self.eat(Token::Multiply);
                    node = AST::new(Token::Multiply, vec![node, self.factor()]);
                }
                Token::Divide => {
                    self.eat(Token::Divide);
                    node = AST::new(Token::Divide, vec![node, self.factor()]);
                }
                _ => break,
            }
        }

        node
    }

    fn expr(&mut self) -> AST {
        let mut node = self.term();

        while let Some(ref curr_token) = self.current_token.clone() {
            match curr_token {
                Token::Plus => {
                    self.eat(Token::Plus);
                    node = AST::new(Token::Plus, vec![node, self.term()]);
                }
                Token::Minus => {
                    self.eat(Token::Minus);
                    node = AST::new(Token::Minus, vec![node, self.term()]);
                }
                _ => break,
            }
        }

        node
    }

    fn parse(&mut self) -> AST {
        self.expr()
    }
}

pub struct Interpreter {
    parser: Parser,
}

impl Interpreter {
    fn new(parser: Parser) -> Interpreter {
        Interpreter { parser }
    }

    fn visit(&self, node: &AST) -> i32 {
        match &node.token {
            Token::Integer(value) => *value,
            Token::Plus => {
                let left_val = self.visit(&node.children[0]);
                let right_val = self.visit(&node.children[1]);
                left_val + right_val
            }
            Token::Minus => {
                let left_val = self.visit(&node.children[0]);
                let right_val = self.visit(&node.children[1]);
                left_val - right_val
            }
            Token::Multiply => {
                let left_val = self.visit(&node.children[0]);
                let right_val = self.visit(&node.children[1]);
                left_val * right_val
            }
            Token::Divide => {
                let left_val = self.visit(&node.children[0]);
                let right_val = self.visit(&node.children[1]);
                left_val / right_val
            }
            _ => unreachable!(),
        }
    }

    fn interpret(&mut self) -> i32 {
        let tree = self.parser.parse();
        let result = self.visit(&tree);

        result
    }
}

fn main() {
    loop {
        let mut input = String::new();

        let _ = io::stdout().write(b"spi> ");
        let _ = io::stdout().flush();

        io::stdin().read_line(&mut input).unwrap();

        let text = String::from(input.trim());
        let lexer = Lexer::new(text);
        let parser = Parser::new(lexer);

        let mut interpreter = Interpreter::new(parser);
        let result = interpreter.interpret();
        println!("{}", result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_interpreter(text: &str) -> Interpreter {
        let lexer = Lexer::new(String::from(text));
        let parser = Parser::new(lexer);
        let interpreter = Interpreter::new(parser);

        interpreter
    }

    #[test]
    fn test_expression1() {
        let mut interpreter = make_interpreter("3");
        let result = interpreter.interpret();
        assert_eq!(result, 3);
    }

    #[test]
    fn test_expression2() {
        let mut interpreter = make_interpreter("2 + 7 * 4");
        let result = interpreter.interpret();
        assert_eq!(result, 30);
    }

    #[test]
    fn test_expression3() {
        let mut interpreter = make_interpreter("7 - 8 / 4");
        let result = interpreter.interpret();
        assert_eq!(result, 5);
    }

    #[test]
    fn test_expression4() {
        let mut interpreter = make_interpreter("14 + 2 * 3 - 6 / 2");
        let result = interpreter.interpret();
        assert_eq!(result, 17);
    }

    #[test]
    fn test_expression5() {
        let mut interpreter = make_interpreter("7 + 3 * (10 / (12 / (3 + 1) - 1))");
        let result = interpreter.interpret();
        assert_eq!(result, 22);
    }

    #[test]
    fn test_expression6() {
        let mut interpreter =
            make_interpreter("7 + 3 * (10 / (12 / (3 + 1) - 1)) / (2 + 3) - 5 - 3 + (8)");
        let result = interpreter.interpret();
        assert_eq!(result, 10);
    }

    #[test]
    fn test_expression7() {
        let mut interpreter = make_interpreter("7 + (((3 + 2)))");
        let result = interpreter.interpret();
        assert_eq!(result, 12);
    }

    #[test]
    #[should_panic]
    fn test_expression_invalid_syntax() {
        let mut interpreter = make_interpreter("10 *");
        interpreter.interpret();
    }
}
