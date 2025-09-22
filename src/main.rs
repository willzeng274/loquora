mod ast;
mod token;
mod lexer;
mod parser;
mod interpreter;
mod loquora;

use std::io;
use std::io::Write;
use std::env;
use std::fs;

use loquora::parser as lqparser;
use loquora::lexer as lqlexer;
use loquora::token::TokenKind;

fn main() {
    if let Some(path) = env::args().nth(1) {
        if path.ends_with(".loq") {
            let source = fs::read_to_string(&path).expect("Failed to read .loq file");
            let lx = lqlexer::Lexer::new(source.clone());
            let mut parser = lqparser::Parser::new(lx);
            let program = parser.parse_program();
            println!("{:#?}", program);
            return;
        }
    }

    let mut buffer = String::new();
    loop {
        let prompt = if buffer.is_empty() { "spi> " } else { "...> " };
        let _ = io::stdout().write(prompt.as_bytes());
        let _ = io::stdout().flush();

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() { break; }

        let trimmed = line.trim();
        if buffer.is_empty() && (trimmed == ":q" || trimmed == ":quit" || trimmed == "quit" || trimmed == "exit") {
            break;
        }

        buffer.push_str(&line);

        if !is_repl_input_complete(&buffer) {
            continue;
        }

        let source = buffer.clone();
        buffer.clear();

        let lx = lqlexer::Lexer::new(source);
        let mut parser = lqparser::Parser::new(lx);

        let parsed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            parser.parse_program()
        }));

        match parsed {
            Ok(program) => {
                println!("{:#?}", program);
            }
            Err(_) => {
                eprintln!("Parse error. Input was not a valid statement.");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::interpreter::Interpreter;

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
        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_expression2() {
        let mut interpreter = make_interpreter("2 + 7 * 4");
        let result = interpreter.interpret();
        assert_eq!(result, 30.0);
    }

    #[test]
    fn test_expression3() {
        let mut interpreter = make_interpreter("7 - 8 / 4");
        let result = interpreter.interpret();
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_expression4() {
        let mut interpreter = make_interpreter("14 + 2 * 3 - 6 / 2");
        let result = interpreter.interpret();
        assert_eq!(result, 17.0);
    }

    #[test]
    fn test_expression5() {
        let mut interpreter = make_interpreter("7 + 3 * (10 / (12 / (3 + 1) - 1))");
        let result = interpreter.interpret();
        assert_eq!(result, 22.0);
    }

    #[test]
    fn test_expression6() {
        let mut interpreter =
            make_interpreter("7 + 3 * (10 / (12 / (3 + 1) - 1)) / (2 + 3) - 5 - 3 + (8)");
        let result = interpreter.interpret();
        assert_eq!(result, 10.0);
    }

    #[test]
    fn test_expression7() {
        let mut interpreter = make_interpreter("7 + (((3 + 2)))");
        let result = interpreter.interpret();
        assert_eq!(result, 12.0);
    }

    #[test]
    #[should_panic]
    fn test_expression_invalid_syntax() {
        let mut interpreter = make_interpreter("10 *");
        interpreter.interpret();
    }
}

fn is_repl_input_complete(src: &str) -> bool {
    // Empty input is never complete
    if src.trim().is_empty() { return false; }

    let mut paren_depth: isize = 0;
    let mut brace_depth: isize = 0;

    let mut lx = lqlexer::Lexer::new(src.to_string());
    let mut last_sig: Option<TokenKind> = None;
    loop {
        let tok = lx.next_token();
        match tok.kind {
            TokenKind::LeftParen => paren_depth += 1,
            TokenKind::RightParen => paren_depth -= 1,
            TokenKind::LeftBrace => brace_depth += 1,
            TokenKind::RightBrace => brace_depth -= 1,
            TokenKind::EOF => { break; }
            _ => {}
        }

        match tok.kind {
            TokenKind::EOF => {}
            _ => { last_sig = Some(tok.kind.clone()); }
        }
    }

    if paren_depth > 0 || brace_depth > 0 { return false; }

    match last_sig {
        Some(TokenKind::Semicolon) | Some(TokenKind::RightBrace) => true,
        _ => false,
    }
}
