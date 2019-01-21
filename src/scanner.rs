// use crate::ast::{Token, ParseError};
use std::str::Chars;
use std::iter::Peekable;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Token<'a> {
    OpenParen,
    ClosedParen,
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(&'a str),
    Symbol(&'a str),
}

#[derive(Debug)]
pub struct ParseError {
    pub message: &'static str,
    pub line: usize,
}

fn is_whitespace(ch: char) -> bool {
    (ch == ' ') || (ch == '\n') || (ch == '\r') || (ch == '\t')
}

fn is_numeric(ch: char) -> bool {
    (ch >= '0') && (ch <= '9')
}

const MISSING_QUOTE: &str = "Missing quote '\"'";
const UNEXPECTED_QUOTE: &str = "Unexpected quote '\"'";
const OPEN_PAREN_IN_ATOM: &str = "Found illegal opening paren '(' in atom";

pub struct Scanner<'a> {
    source: &'a str,
    iter: Peekable<Chars<'a>>,
    start: usize,
    current: usize,
    line: usize,
    tokens: Vec<Token<'a>>
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source,
            iter: source.chars().peekable(),
            start: 0,
            current: 0,
            line: 1,
            tokens: Vec::new(),
        }
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token<'a>>, ParseError> {
        while !self.at_end() {
            self.start = self.current;
            self.token()?;
        }
        Ok(self.tokens)
    }

    fn parse_err(&self, message: &'static str) -> Result<(), ParseError> {
        Err(ParseError { message: message, line: self.line })
    }

    fn token(&mut self) -> Result<(), ParseError> {
        match self.advance().unwrap() {
            ' ' => Ok(()),
            '\n' => { self.line += 1; Ok(()) }
            '(' => { self.tokens.push(Token::OpenParen); Ok(()) }
            ')' => { self.tokens.push(Token::ClosedParen); Ok(()) }
            '+' | '-' => if self.is_more_token() { self.int() } else { self.symbol() }
            ch if is_numeric(ch) => self.int(),
            '.' => if self.is_more_token() { self.float(false) } else { self.symbol() }
            '"' => self.string(),
            '#' => self.bool_(),
            _ => self.symbol(),
        }
    }

    fn int(&mut self) -> Result<(), ParseError> {
        while self.is_more_token() {
            match self.advance().unwrap() {
                ch if is_numeric(ch) => (),
                '.' => return self.float(false),
                'e' | 'E' => return match self.peek() {
                    None => self.symbol(),
                    Some(_) => self.float(true),
                },
                '"' => return self.parse_err(UNEXPECTED_QUOTE),
                '(' => return self.parse_err(OPEN_PAREN_IN_ATOM),
                _ => return self.symbol(),
            }
        }
        self.add_int_token();
        Ok(())
    }

    fn float(&mut self, mut exponent_consumed: bool) -> Result<(), ParseError> {
        if !exponent_consumed {
            while self.is_more_token() {
                match self.advance().unwrap() {
                    ch if is_numeric(ch) => (),
                    'e' | 'E' => { exponent_consumed = true; break; }
                    '(' => return self.parse_err(OPEN_PAREN_IN_ATOM),
                    _ => return self.symbol(),
                }
            }
            if !self.is_more_token() {
                if exponent_consumed {
                    return self.symbol();
                } else {
                    self.add_float_token();
                    return Ok(());
                }
            }
        }
        match self.peek() {
            None => (),
            Some(ch) => match ch {
                '+' | '-' => { self.advance(); },
                ch if is_numeric(ch) => (),
                _ => return self.symbol(),
            }
        };

        while self.is_more_token() {
            match self.advance().unwrap() {
                ch if is_numeric(ch) => (),
                '(' => return self.parse_err(OPEN_PAREN_IN_ATOM),
                _ => return self.symbol(),
            }
        }
        
        self.add_float_token();
        Ok(())
    }
    
    fn string(&mut self) -> Result<(), ParseError> {
        loop {
            match self.advance() {
                None => return self.parse_err(MISSING_QUOTE),
                Some(ch) => match ch {
                    '"' => { self.add_string_token(); return Ok(()); }
                    '\n' => return self.parse_err(MISSING_QUOTE),
                    _ => ()
                }
            }
        }
    }

    fn symbol(&mut self) -> Result<(), ParseError> {
        while self.is_more_token() {
            match self.advance().unwrap() {
                '(' => return self.parse_err(OPEN_PAREN_IN_ATOM),
                _ => ()
            }
        }

        self.add_symbol_token();
        Ok(())
    }

    fn bool_(&mut self) -> Result<(), ParseError> {
        match self.advance() {
            None => self.parse_err("expected char"),
            Some(ch) => match ch {
                't' | 'f' => if self.is_more_token() {
                    self.parse_err("unexpected char after '#' 1")
                } else {
                    self.tokens.push(Token::Bool(ch == 't'));
                    Ok(())
                }
                _ => self.parse_err("unexpected char after '#' 2"),
            }
        }
    }

    fn add_int_token(&mut self) {
        let token_str = self.source.get(self.start..self.current).unwrap();
        let parsed = token_str.parse::<i64>().ok().unwrap();
        self.tokens.push(Token::Int(parsed));
    }

    fn add_float_token(&mut self) {
        let token_str = self.source.get(self.start..self.current).unwrap();
        let parsed = token_str.parse::<f64>().ok().unwrap();
        self.tokens.push(Token::Float(parsed));
    }

    fn add_string_token(&mut self) {
        let slice = self.source.get(self.start+1..self.current-1).unwrap();
        self.tokens.push(Token::Str(slice));
    }

    fn add_symbol_token(&mut self) {
        let slice = self.source.get(self.start..self.current).unwrap();
        self.tokens.push(Token::Symbol(slice));
    }

    fn at_end(&self) -> bool { 
        self.current >= self.source.len()
    }

    fn is_more_token(&mut self) -> bool {
        match self.peek() {
            None => false,
            Some(ch) if is_whitespace(ch) || (ch == ')') => false,
            _ => true,
        }
    }

    fn advance(&mut self) -> Option<char> {
        match self.iter.next() {
            Some(ch) => {
                self.current += ch.len_utf8();
                Some(ch)
            }
            None => None,
        }
    }

    fn peek(&mut self) -> Option<char> {
        match self.iter.peek() {
            Some(&ch) => Some(ch),
            None => None,
        }      
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn s(x: &'static str) -> Token { Token::Symbol(x) }
    fn st(x: &'static str) -> Token { Token::Str(x) }
    fn i(x: i64) -> Token<'static> { Token::Int(x) }
    fn f(x: f64) -> Token<'static> { Token::Float(x) }
    fn op() -> Token<'static> { Token::OpenParen }
    fn cp() -> Token<'static> { Token::ClosedParen }
    
    fn tokens(x: &'static str) -> Result<Vec<Token>, ParseError> { 
        let scanner = Scanner::new(x);
        scanner.scan_tokens()
    }

    fn scan_ok(x: &'static str, expected: Vec<Token>) {
        let res = tokens(x).expect("err");
        assert_eq!(res, expected);
    }

    fn scan_err(x: &'static str) {
        let res = tokens(x);
        assert!(res.is_err());
    }


    #[test]
    fn test_scan_tokens() {
        let mut tests = vec![
            ("()", vec![op(), cp()]),
            ("(() (1 2 3))", vec![op(), op(), cp(), op(), i(1), i(2), i(3), cp(), cp()]),
            ("($%-a)", vec![op(), s("$%-a"), cp()]),
            ("(() () ())", vec![op(), op(), cp(), op(), cp(), op(), cp(), cp()]),
        ];
        let same = vec![
            "(+ abc def)", "( + abc def)", "(+ abc def )",
            "(+  abc    def)", "    (+ abc def)  ", "(+\nabc\ndef)",
        ];
        for i in same { tests.push((i, vec![op(), s("+"), s("abc"), s("def"), cp()])); }

        for (x, y) in tests { scan_ok(x, y); }

        let errs = vec![
            "(ab( 1 2)", "(1 2\nab( 3)",
        ];

        for x in errs { scan_err(x); }
    }

    #[test]
    fn test_scan_int() {
        let mut tests = vec![
            ("123", vec![i(123)]),
            ("(1", vec![op(), i(1)]),
            ("(+011233)", vec![op(), i(11233), cp()]),
            ("(-011233)", vec![op(), i(-11233), cp()]),
            ("-55123", vec![i(-55123)]),
            ("--5123", vec![s("--5123")]),
            ("-+5123", vec![s("-+5123")]),
            ("5-5", vec![s("5-5")]),
        ];

        let same = vec![
            " 123", "0123"," 0123",
            " +0123", "+0123",
        ];
        for x in same { tests.push((x, vec![i(123)])); }

        for (x, y) in tests { scan_ok(x, y) }

        scan_err(" +01123(3");
    }

    #[test]
    fn test_scan_string() {
        let tests = vec! [
            (r#" ("abc") "#, vec![op(), st("abc"), cp()]),
            (r#" "(abc))())(" "#, vec![st("(abc))())(")]),
            (r#"  ("abc" "def" ("ijk")) "#, vec![op(), st("abc"), st("def"), op(), st("ijk"), cp(), cp()]),
        ];

        for (x, y) in tests { scan_ok(x, y); }

        let errs = vec![
            " \"a\nb\"  ", "(\")"
        ];

        for x in errs { scan_err(x); }
    }

    #[test]
    fn test_scan_float() {
        let mut tests = vec![
            ("1.1.", vec![s("1.1.")]),
            ("1.e15.", vec![s("1.e15.")]),
            ("1.e", vec![s("1.e")]),
            (".1", vec![f(0.1)]),
            (".", vec![s(".")]),
            ("3.14156e-03", vec![f(3.14156e-03)])
        ];

        let same1 = vec![
            "1.0", "01.0", "+1.0", "+01.0", "+01.",
            "01.", "1.", "1.00", "+1.00", "+01.00",
        ];
        for i in same1 { tests.push((i, vec![f(1.0)])); }

        let same2 = vec![
            "1e1", "+1e1", "+1e+1",
            "1e+01", "001e01", "001.0e+01",
            "1.e+1", "1.e+001", "001.e01",
        ];
        for i in same2 { tests.push((i, vec![f(1e1)])); }

        let errs = vec![
            "1(e1", "1e1(", "1.e1(", "1.e(1",
        ];

        for x in errs { scan_err(x); }       
    }

    #[test]
    fn test_bool() {
        scan_ok("#t", vec![Token::Bool(true)]);
        scan_ok("#f", vec![Token::Bool(false)]);
        scan_err("#");
        scan_err("#ta");
        scan_err("#fa");
        scan_err("#a");
        scan_err("#b");
    }

    #[test]
    fn test_advance() {
        let mut scanner = Scanner::new("123");
        assert_eq!(scanner.advance(), Some('1'));
        assert_eq!(scanner.advance(), Some('2'));
        assert_eq!(scanner.advance(), Some('3'));
        assert_eq!(scanner.advance(), None);
    }

    #[test]
    fn test_peek() {
        let mut scanner = Scanner::new("1");
        assert_eq!(scanner.peek(), Some('1'));
        assert_eq!(scanner.advance(), Some('1'));
        assert_eq!(scanner.peek(), None);
        assert_eq!(scanner.advance(), None);
    }

    #[test]
    fn test_at_end() {
        let mut scanner = Scanner::new("12345");
        for _ in 0..5 { scanner.advance(); }
        assert!(scanner.at_end());
    }
}