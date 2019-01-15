use crate::sexpr::{SExpr, Memory};
use crate::scanner::{Token, ParseError};

pub struct Parser<'s, 'm> {
    tokens: Vec<Token<'s>>,
    start: usize,
    current: usize,
    mem: &'m mut Memory<'s>,
}

impl<'s, 'm> Parser<'s, 'm> {
    pub fn new(tokens: Vec<Token<'s>>, mem: &'m mut Memory<'s>) -> Self {
        Parser {
            tokens: tokens,
            start: 0,
            current: 0,
            mem: mem,
        }
    }

    // sexpr ::= int | float | symbol | string | '(' ')' | '(' sexpr (sexpr)* ')'

    pub fn parse(mut self) -> Result<SExpr<'s>, ParseError> {
        self.expr()
    }

    fn expr(&mut self) -> Result<SExpr<'s>, ParseError> {
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
                        let mut exprs = Vec::new();
                        loop {
                            let previous = self.current;
                            match self.expr() {
                                Ok(e) => exprs.push(e), 
                                Err(_) => { self.current = previous; break }
                            }
                        }
                        println!("{:?}", exprs);
                        match self.advance() {
                            Some(Token::ClosedParen) => Ok(self.mem.list_from_vec(exprs).unwrap()), // unimplemented!(), //Ok(SExpr::from_vec(self.mem, exprs).unwrap()),
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

    macro_rules! parse {
        ( $s:expr, $e:ident, $mem:ident )  => {
            let scanner = Scanner::new($s);
            let tokens = scanner.scan_tokens().ok().unwrap();
            let mut $mem = Memory::new(100);
            let parser = Parser::new(tokens, &mut $mem);
            let res = parser.parse();
            let $e = SExpr::Nil; //res.unwrap();
        };
    }

    // fn do_parse<'s>(s: &'static str) -> (SExpr<'s>, Mem$mem$mem$memory<'s, )

    // fn parse_err<'a>(s: &'a str) {
    //     let scanner = Scanner::new(s);
    //     let tokens = scanner.scan_tokens().ok().unwrap();
    //     let mem = &mut Memory::new(100);
    //     let parser = Parser::new(tokens, mem);
    //     let res = parser.parse();
    //     assert!(res.is_err());
    // }

    fn f<'s>(x: f64) -> SExpr<'s> { SExpr::Float(x) }
    fn i<'s>(x: i64) -> SExpr<'s> { SExpr::Int(x) }
    fn sy<'s>(x: &'s str) -> SExpr<'s> { SExpr::Sym(x) }
    fn st<'s>(x: &'s str) -> SExpr<'s> { SExpr::Str(x) }
    fn n<'s>() -> SExpr<'s> { SExpr::Nil }

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
        let mut mem = Memory::new(100);
        let scanner = Scanner::new("(+)");
        let tokens = scanner.scan_tokens().unwrap();
        let parser = Parser::new(tokens, &mut mem);
        let res = parser.parse();
        assert_eq!(mem.vec_from_list(res.unwrap()).unwrap(), vec![sy("+")]);
        // assert_eq!(mem.vec_from_list(res).unwrap(), vec![sy("+")]);
        // {
        //     parse!("(+)", x, mem);
        //     assert_eq!(mem.vec_from_list(x).unwrap(), vec![sy("+")])
        // }
        // let (x, mem) = parse!("(+)");
        // assert_eq!(mem.vec_from_list(x).unwrap(), vec![sy("+")]);
        // parse_ok_fn![
        //     ("123", |x, _| assert_eq!(x, i(123))),
        //     ("(+)", |x: SExpr, mem: Memory| assert_eq!(mem.vec_from_list(x).unwrap(), vec![sy("+")])),
        //     ("(+ 1 2)", |x: SExpr, mem: Memory| {
        //         assert_eq!(mem.vec_from_list(x).unwrap(), vec![sy("+"), i(1), i(2)])
        //     }),
        //     ("(+ (+ 1 2) 3)", |x: SExpr, mem: Memory| {
        //         let a = mem.vec_from_list(x).unwrap();
        //         assert_eq!(a[0], sy("+"));
        //         assert_eq!(a[2], i(3));
        //         assert_eq!(mem.vec_from_list(a[1]).unwrap(), vec![sy("+"), i(1), i(2)]);
        //     })
        // ];
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