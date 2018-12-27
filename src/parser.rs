use crate::ast::{Token, ParseError, Expr};
use crate::scanner::Scanner;

pub struct Parser {
    tokens: Vec<Token>,
    start: usize,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens,
            start: 0,
            current: 0,
        }
    }

    // expr ::= int | float | symbol | string | '(' ')' | '(' expr (expr)* ')'

    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        self.expr()
    }

    fn expr(&mut self) -> Result<Expr, ParseError> {
        match self.advance() {
            None => Err(ParseError { message: "Empty expression", line: 0 }),
            Some(token) => match token {
                Token::Int(x) => Ok(Expr::Int(x.clone())),
                Token::Float(x) => Ok(Expr::Float(x.clone())),
                Token::Str(x) => Ok(Expr::Str(x.clone())),
                Token::Symbol(x) => Ok(Expr::Symbol(x.clone())),
                Token::OpenParen => match self.peek() {
                    None => Err(ParseError { message: "Missing closing parenthesis", line: 0}),
                    Some(Token::ClosedParen) => { self.advance(); Ok(Expr::Null) },
                    _ => {
                        let mut exprs = Vec::new();
                        loop {
                            let previous = self.current;
                            match self.expr() {
                                Ok(e) => exprs.push(Box::new(e)),
                                _ => { self.current = previous; break; }
                            }
                        }
                        match self.advance() {
                            Some(Token::ClosedParen) => Ok(Expr::List(exprs)),
                            _ => Err(ParseError { message: "Missing closing parenthesis", line: 0}),
                        }
                    }
                }
                Token::ClosedParen => Err(ParseError { message: "Unexpected closing parenthesis", line: 0}),
            }
        }
    }    

    fn advance(&mut self) -> Option<&Token> {
        if self.at_end() { return None; }
        let ch = &self.tokens[self.current];
        self.current += 1;
        Some(ch)
    }

    fn peek(&self) -> Option<&Token> {
        if self.at_end() { return None; }
        Some(&self.tokens[self.current])
    }

    fn at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(x: &'static str) -> Result<Expr, ParseError> {
        let mut scanner = Scanner::new(x.as_bytes());
        let tokens = scanner.scan_tokens().ok().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }
    fn parse_ok(x: &'static str, expected: Expr) {
        let res = parse(x);
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), expected);
    }
    fn parse_err(x: &'static str) {
        let res = parse(x);
        assert!(res.is_err());
    }

    fn f(x: f64) -> Expr { Expr::Float(x) }
    fn i(x: i64) -> Expr { Expr::Int(x) }
    fn sy(x: &'static str) -> Expr { Expr::Symbol(x.as_bytes().to_vec()) }
    fn st(x: &'static str) -> Expr { Expr::Str(x.as_bytes().to_vec()) }
    fn n() -> Expr { Expr::Null }
    fn l(x: Vec<Expr>) -> Expr { 
        let mut boxed = Vec::new();
        for i in x { boxed.push(Box::new(i)); }
        Expr::List(boxed)
    }

    #[test]
    fn test_values() {
        let tests = vec![
            ("123", i(123)),
            ("1.0", f(1.0)),
            ("abc", sy("abc")),
            ("\"abc\"", st("abc")),
            ("()", n()),
        ];
        for (x, y) in tests { parse_ok(x, y); }
    }

    #[test]
    fn test_exprs() {
        let tests = vec![
            ("(+ 1 2)", l(vec![sy("+"), i(1), i(2)])),
            ("(+ 1 2 3 4 5)", l(vec![sy("+"), i(1), i(2), i(3), i(4), i(5)])),
            ("(- 3 4)", l(vec![sy("-"), i(3), i(4)])),
            ("(* (+ 5 2) (- 5 3))", l(vec![sy("*"), l(vec![sy("+"), i(5), i(2)]), l(vec![sy("-"), i(5), i(3)])])),
            ("(())", l(vec![n()])),
            ("(() () ())", l(vec![n(), n(), n()])),
            ("(cons 1 (cons 2 (cons 3 ())))", l(vec![sy("cons"), i(1), l(vec![sy("cons"), i(2), l(vec![sy("cons"), i(3), n()])])]))
        ];
        for (x, y) in tests { parse_ok(x, y); }
    }

    #[test]
    fn test_err() {
        let tests = vec![
            ")", "(+ 1", "", "(()"
        ];
        for x in tests { parse_err(x); }
    }
}