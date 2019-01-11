use crate::ast::{Token, ParseError, SExpr, Memory};

pub struct Parser<'s, 'm> {
    tokens: Vec<Token<'s>>,
    start: usize,
    current: usize,
    mem: &'m Memory<'s, 'm>,
}

impl<'s, 'm> Parser<'s, 'm> {
    pub fn new(tokens: Vec<Token<'s>>, mem: &'m mut Memory<'s, 'm>) -> Self {
        Parser {
            tokens: tokens,
            start: 0,
            current: 0,
            mem: mem,
        }
    }

    // sexpr ::= int | float | symbol | string | '(' ')' | '(' sexpr (sexpr)* ')'

    pub fn parse(&mut self) -> Result<SExpr<'s, 'm>, ParseError> {
        self.expr()
    }

    fn expr(&mut self) -> Result<SExpr<'s, 'm>, ParseError> {
        match self.advance() {
            None => Err(ParseError { message: "Empty expression", line: 0 }),
            Some(token) => match token {
                Token::Int(x) => Ok(SExpr::Int(x)),
                Token::Float(x) => Ok(SExpr::Float(x)),
                Token::Str(x) => Ok(SExpr::Str(x)),
                Token::Symbol(x) => Ok(SExpr::Sym(x)),
                Token::OpenParen => match self.peek() {
                    None => Err(ParseError { message: "Missing closing parenthesis", line: 0}),
                    Some(Token::ClosedParen) => { self.advance(); Ok(SExpr::Nil) },
                    _ => {
                        let mut head = None;
                        let mut tail: Option<SExpr<'s, 'm>> = None;
                        loop {
                            let previous = self.current;
                            match self.expr() {
                                Ok(e) => {
                                    let curr = SExpr::cons(self.mem, e, SExpr::Nil); //self.mem.alloc(Object::Pair(e, SExpr::Nil));
                                    match tail {
                                        None => head = Some(curr),
                                        Some(tail_loc) => tail_loc.set_cdr(curr).expect("tail not pair"),
                                    }
                                    tail = Some(curr);
                                },
                                Err(_) => { self.current = previous; break }
                            }
                        }

                        match self.advance() {
                            Some(Token::ClosedParen) => Ok(head.unwrap()),
                            _ => Err(ParseError { message: "Missing closing parenthesis", line: 0}),
                        }
                    }
                }
                Token::ClosedParen => Err(ParseError { message: "Unexpected closing parenthesis", line: 0}),
            }
        }
    }    

    fn advance(&mut self) -> Option<Token<'s>> {
        match self.tokens.get(self.current) {
            Some(&token) => {
                self.current += 1;
                Some(token)
            }
            None => None
        }
    }

    fn peek(&self) -> Option<Token<'s>> {
        match self.tokens.get(self.current) {
            Some(&token) => Some(token),
            None => None,
        }
    }

    fn at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;

    macro_rules! parse_ok {
        ( $( ($left:expr, $right:expr) ),* )  => {
            $({
                let scanner = Scanner::new($left);
                let tokens = scanner.scan_tokens().ok().unwrap();
                let mut mem = Memory::new(100);
                let mut parser = Parser::new(tokens, &mut mem);
                let res = parser.parse();
                assert!(res.is_ok());
                assert_eq!(res.unwrap(), $right);
            })*
        };
    }

    macro_rules! parse_ok_fn {
        ( $( ($left:expr, $right:expr) ),* )  => {
            $({
                let scanner = Scanner::new($left);
                let tokens = scanner.scan_tokens().ok().unwrap();
                let mut mem = Memory::new(100);
                let mut parser = Parser::new(tokens, &mut mem);
                let res = parser.parse();
                assert!(res.is_ok());
                assert!($right(res.unwrap()));
            })*
        };
    }

    // fn parse_err<'a>(s: &'a str) {
    //     let scanner = Scanner::new(s);
    //     let tokens = scanner.scan_tokens().ok().unwrap();
    //     let mem = &mut Memory::new(100);
    //     let parser = Parser::new(tokens, mem);
    //     let res = parser.parse();
    //     assert!(res.is_err());
    // }

    fn f<'s, 'm>(x: f64) -> SExpr<'s, 'm> { SExpr::Float(x) }
    fn i<'s, 'm>(x: i64) -> SExpr<'s, 'm> { SExpr::Int(x) }
    fn sy<'s, 'm>(x: &'s str) -> SExpr<'s, 'm> { SExpr::Sym(x) }
    fn st<'s, 'm>(x: &'s str) -> SExpr<'s, 'm> { SExpr::Str(x) }
    fn n<'s, 'm>() -> SExpr<'s, 'm> { SExpr::Nil }

    #[test]
    fn test_values() {
        parse_ok![
            ("123", i(123)),
            ("1.0", f(1.0)),
            ("abc", sy("abc")),
            ("\"abc\"", st("abc")),
            ("()", n())
        ];
    }

    #[test]
    fn test_exprs() {
        parse_ok_fn![
            ("123", |x| x == i(123)),
            ("(+)", |x: SExpr| x.list_to_vec().unwrap() == vec![sy("+")]),
            ("(+ 1 2)", |x: SExpr| x.list_to_vec().unwrap() == vec![sy("+"), i(1), i(2)]),
            ("(+ (+ 1 2) 3)", |x: SExpr| {
                let a = x.list_to_vec().unwrap();
                let as1 = a[0] == sy("+");
                let as2 = a[2] == i(3);
                let as3 = a[1].list_to_vec().unwrap() == vec![sy("+"), i(1), i(2)];
                as1 && as2 && as3
            })
        ];
    }
    // #[test]
    // fn test_err() {
    //     let tests = vec![
    //         ")", "(+ 1", "", "(()"
    //     ];
    //     for x in tests { println!("{}", x); parse_err(x); }
    //     let scanner = Scanner::new("1 2 )");
    //     let tokens = scanner.scan_tokens().ok().unwrap();
    //     let mem = &mut Memory::new(100);
    //     let parser = Parser::new(tokens, mem);
    //     let res = parser.parse();
    //     assert!(res.is_ok());
    // }
}