use crate::ast::{Token, ParseError, SExpr, Object, Memory};

pub struct Parser<'a, 'b> {
    tokens: Vec<Token<'a>>,
    start: usize,
    current: usize,
    mem: &'b mut Memory<'a>,
}

impl<'a, 'b> Parser<'a, 'b> {
    pub fn new(tokens: Vec<Token<'a>>, mem: &'b mut Memory<'a>) -> Self {
        Parser {
            tokens: tokens,
            start: 0,
            current: 0,
            mem: mem,
        }
    }

    // sexpr ::= int | float | symbol | string | '(' ')' | '(' sexpr (sexpr)* ')'

    pub fn parse(mut self) -> Result<SExpr<'a>, ParseError> {
        self.expr()
    }

    fn expr(&mut self) -> Result<SExpr<'a>, ParseError> {
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
                        let mut prev_ref = None;
                        let mut head_ref = None;
                        loop {
                            match self.expr() {
                                Ok(e) => {
                                    let curr_ref = self.mem.alloc(Object::Cons(e, SExpr::Nil));
                                    if let Some(loc) = prev_ref {
                                        self.mem.set_cdr(loc, SExpr::Ref(curr_ref));
                                    } else {
                                        head_ref = Some(SExpr::Ref(curr_ref));
                                    }
                                    prev_ref = Some(curr_ref)
                                },
                                _ => break,
                            }
                        }
                        match self.advance() {
                            Some(Token::ClosedParen) => Ok(head_ref.unwrap()),
                            _ => Err(ParseError { message: "Missing closing parenthesis", line: 0}),
                        }
                    }
                }
                Token::ClosedParen => Err(ParseError { message: "Unexpected closing parenthesis", line: 0}),
            }
        }
    }    

    fn advance(&mut self) -> Option<Token<'a>> {
        match self.tokens.get(self.current) {
            Some(&token) => {
                self.current += 1;
                Some(token)
            }
            None => None
        }
    }

    fn peek(&self) -> Option<Token<'a>> {
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

    // fn parse((x, mem): (&'static str, &'static mut Memory)) -> (Result<SExpr<'static>, ParseError>, &'static Memory<'static>) {
    //     let scanner = Scanner::new(x);
    //     let tokens = scanner.scan_tokens().ok().unwrap();
    //     let mut parser = Parser::new(tokens, mem);
    //     (parser.parse(), &parser.mem)
    // }

    fn parse_ok<'a>(s: &'a str, mut init_mem: Memory<'a>, sexpr: SExpr<'static>, mem: Memory<'static>) {
        let scanner = Scanner::new(s);
        let tokens = scanner.scan_tokens().ok().unwrap();
        let parser = Parser::new(tokens, &mut init_mem);
        let res = parser.parse();
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), sexpr);
        assert_eq!(init_mem, mem);
        
    }

    // fn parse_ok((x, mem): (&'static str, &'static mut Memory), (ex_sexpr, ex_mem): (SExpr<'static>, &'static Memory)) {
    //     let (res, res_mem) = parse((x, mem));
    //     assert!(res.is_ok());
    //     assert_eq!(res.ok().unwrap(), ex_sexpr);
    //     assert_eq!(res_mem, ex_mem);
    // }
    
    // fn parse_err(x: &'static str) {
    //     let res = parse(x);
    //     assert!(res.is_err());
    // }

    fn f(x: f64) -> SExpr<'static> { SExpr::Float(x) }
    fn i(x: i64) -> SExpr<'static> { SExpr::Int(x) }
    fn sy(x: &'static str) -> SExpr<'static> { SExpr::Sym(x) }
    fn st(x: &'static str) -> SExpr<'static> { SExpr::Str(x) }
    fn n() -> SExpr<'static> { SExpr::Nil }
    
    #[test]
    fn test_values() {
        let empty = || Memory::new(1);
        let tests = vec![
            ("123", empty(), i(123), empty()),
            ("1.0", empty(), f(1.0), empty()),
            ("abc", empty(), sy("abc"), empty()),
            ("\"abc\"", empty(), st("abc"), empty()),
            ("()", empty(), n(), empty()),
        ];
        for (a, b, c, d) in tests { parse_ok(a, b, c, d); }
    }

    // #[test]
    // fn test_exprs() {
    //     let tests = vec![
    //         ("(+ 1 2)", l(vec![sy("+"), i(1), i(2)])),
    //         ("(+ 1 2 3 4 5)", l(vec![sy("+"), i(1), i(2), i(3), i(4), i(5)])),
    //         ("(- 3 4)", l(vec![sy("-"), i(3), i(4)])),
    //         ("(* (+ 5 2) (- 5 3))", l(vec![sy("*"), l(vec![sy("+"), i(5), i(2)]), l(vec![sy("-"), i(5), i(3)])])),
    //         ("(())", l(vec![n()])),
    //         ("(() () ())", l(vec![n(), n(), n()])),
    //         ("(cons 1 (cons 2 (cons 3 ())))", l(vec![sy("cons"), i(1), l(vec![sy("cons"), i(2), l(vec![sy("cons"), i(3), n()])])]))
    //     ];
    //     for (x, y) in tests { parse_ok(x, y); }
    // }

    // #[test]
    // fn test_err() {
    //     let tests = vec![
    //         ")", "(+ 1", "", "(()"
    //     ];
    //     for x in tests { parse_err(x); }
    // }
}