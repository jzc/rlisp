use crate::ast::{Token, ParseError, SExpr, Object, Memory};

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
                        // let mut prev_ref = None;
                        // let mut head_ref = None;
                        // loop {
                        //     let previous = self.current;
                        //     match self.expr() {
                        //         Ok(e) => {
                        //             let curr_ref = self.mem.alloc(Object::Cons(e, SExpr::Nil));
                        //             match prev_ref {
                        //                 Some(prev_loc) => self.mem.set_cdr(prev_loc, SExpr::Ref(curr_ref)),
                        //                 None => head_ref = Some(curr_ref),
                        //             }
                        //             prev_ref = Some(curr_ref)
                        //         },
                        //         _ => { self.current = previous; break }
                        //     }
                        // }
                        // match self.advance() {
                        //     Some(Token::ClosedParen) => Ok(SExpr::Ref(head_ref.unwrap())),
                        //     _ => Err(ParseError { message: "Missing closing parenthesis", line: 0}),
                        // }
                        let mut head = None;
                        let mut tail: Option<SExpr<'s, 'm>> = None;
                        loop {
                            let previous = self.current;
                            match self.expr() {
                                Ok(e) => {
                                    let curr = self.mem.alloc(Object::Cons(e, SExpr::Nil));
                                    match tail {
                                        None => head = Some(curr),
                                        Some(tail_loc) => tail_loc.set_cdr(curr).ok().unwrap(),
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

    struct Tester<'s, 'm> {
        mem: Memory<'s, 'm>,
    }

    // fn parse_ok<'a>(s: &'a str, a: SExpr<'a, 'a>) {
    //     let scanner = Scanner::new(s);
    //     let tokens = scanner.scan_tokens().ok().unwrap();
    //     let mut mem = Memory::new(100);
    //     let mut parser = Parser::new(tokens, &mut mem);
    //     let res = parser.parse();
    //     assert!(res.is_ok());
    //     assert_eq!(res.ok().unwrap(), a);
    //     // assert_eq!(init_mem, mem);
    // }

    macro_rules! parse_ok {
        ( $( ($left:expr, $right:expr) ),* )  => {
            $({
                let scanner = Scanner::new($left);
                let tokens = scanner.scan_tokens().ok().unwrap();
                let mut mem = Memory::new(100);
                let mut parser = Parser::new(tokens, &mut mem);
                let res = parser.parse();
                assert!(res.is_ok());
                assert_eq!(res.ok().unwrap(), $right);
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

    // #[test]
    // fn test_exprs() {
    //     let scanner = Scanner::new("(+ 1 2)");
    //     let tokens = scanner.scan_tokens().ok().unwrap();
    //     let mut mem = Memory::new(500);
    //     let parser = Parser::new(tokens, &mut mem);
    //     let res = parser.parse();
    //     assert!(res.is_ok());
    //     assert_eq!(res.ok().unwrap(), SExpr::Ref(0));
    //     assert_eq!(mem.car(0), SExpr::Sym("+"));
    //     assert_eq!(mem.cdr(0), SExpr::Ref(1));
    //     assert_eq!(mem.car(1), SExpr::Int(1));
    //     assert_eq!(mem.cdr(1), SExpr::Ref(2));
    //     assert_eq!(mem.car(2), SExpr::Int(2));
    //     assert_eq!(mem.cdr(2), SExpr::Nil);
        
    //     // let tests = vec![
    //     //     ("(+ 1 2)", l(vec![sy("+"), i(1), i(2)])),
    //     //     ("(+ 1 2 3 4 5)", l(vec![sy("+"), i(1), i(2), i(3), i(4), i(5)])),
    //     //     ("(- 3 4)", l(vec![sy("-"), i(3), i(4)])),
    //     //     ("(* (+ 5 2) (- 5 3))", l(vec![sy("*"), l(vec![sy("+"), i(5), i(2)]), l(vec![sy("-"), i(5), i(3)])])),
    //     //     ("(())", l(vec![n()])),
    //     //     ("(() () ())", l(vec![n(), n(), n()])),
    //     //     ("(cons 1 (cons 2 (cons 3 ())))", l(vec![sy("cons"), i(1), l(vec![sy("cons"), i(2), l(vec![sy("cons"), i(3), n()])])]))
    //     // ];
    //     // for (x, y) in tests { parse_ok(x, y); }
    // }

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