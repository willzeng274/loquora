mod ast;
mod token;
mod lexer;
mod parser;
mod interpreter;

use std::io;
use std::io::Write;

use lexer::Lexer;
use parser::Parser;
use interpreter::Interpreter;

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
