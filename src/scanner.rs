use crate::ast::{Token, ParseError, AsciiString};

fn is_whitespace(ch: u8) -> bool {
    (ch == b' ') || (ch == b'\n')
}

fn is_numeric(ch: u8) -> bool {
    (ch >= b'0') && (ch <= b'9')
}

const MISSING_QUOTE: &str = "Missing quote '\"'";
const UNEXPECTED_QUOTE: &str = "Unexpected quote '\"'";
const OPEN_PAREN_IN_ATOM: &str = "Found illegal opening paren '(' in atom";

pub struct Scanner {
    source: AsciiString,
    start: usize,
    current: usize,
    line: usize,
    tokens: Vec<Token>
}

impl Scanner {
    pub fn new<'a>(source: &'a [u8]) -> Self {
        Scanner {
            source: source.to_vec(),
            start: 0,
            current: 0,
            line: 1,
            tokens: Vec::new(),
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, ParseError> {
        while !self.at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        return Ok(self.tokens.clone());
    }

    fn parse_err(&self, message: &'static str) -> Result<(), ParseError> {
        Err(ParseError { message: message, line: self.line })
    }

    fn scan_token(&mut self) -> Result<(), ParseError> {
        match self.advance().unwrap() {
            b' ' => Ok(()),
            b'\n' => { self.line += 1; Ok(()) }
            b'(' => { self.tokens.push(Token::OpenParen); Ok(()) }
            b')' => { self.tokens.push(Token::ClosedParen); Ok(()) }
            b'+' | b'-' => if self.is_more_token() { self.int() } else { self.symbol() }
            ch if is_numeric(ch) => self.int(),
            b'.' => if self.is_more_token() { self.float(false) } else { self.symbol() }
            b'"' => self.string(),
            _ => self.symbol(),
        }
    }

    fn int(&mut self) -> Result<(), ParseError> {
        while self.is_more_token() {
            match self.advance().unwrap() {
                ch if is_numeric(ch) => (),
                b'.' => return self.float(false),
                b'e' | b'E' => return match self.peek() {
                    None => self.symbol(),
                    Some(_) => self.float(true),
                },
                b'"' => return self.parse_err(UNEXPECTED_QUOTE),
                b'(' => return self.parse_err(OPEN_PAREN_IN_ATOM),
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
                    b'e' | b'E' => { exponent_consumed = true; break; }
                    b'(' => return self.parse_err(OPEN_PAREN_IN_ATOM),
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
                b'+' | b'-' => { self.advance(); },
                ch if is_numeric(ch) => (),
                _ => return self.symbol(),
            }
        };

        while self.is_more_token() {
            match self.advance().unwrap() {
                ch if is_numeric(ch) => (),
                b'(' => return self.parse_err(OPEN_PAREN_IN_ATOM),
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
                    b'"' => { self.add_string_token(); return Ok(()); }
                    b'\n' => return self.parse_err(MISSING_QUOTE),
                    _ => ()
                }
            }
        }
    }

    fn symbol(&mut self) -> Result<(), ParseError> {
        while self.is_more_token() {
            match self.advance().unwrap() {
                b'(' => return self.parse_err(OPEN_PAREN_IN_ATOM),
                _ => ()
            }
        }

        self.add_symbol_token();
        Ok(())
    }

    fn token_str(&self) -> String {
        let slice = &self.source[self.start..self.current];
        let token_str = String::from_utf8(slice.to_vec()).ok().unwrap();
        token_str
    }

    fn add_int_token(&mut self) {
        let parsed = self.token_str().parse::<i64>().ok().unwrap();
        self.tokens.push(Token::Int(parsed));
    }

    fn add_float_token(&mut self) {
        let parsed = self.token_str().parse::<f64>().ok().unwrap();
        self.tokens.push(Token::Float(parsed));
    }

    fn add_string_token(&mut self) {
        let slice = self.source[self.start+1..self.current-1].to_vec();
        self.tokens.push(Token::Str(slice));
    }

    fn add_symbol_token(&mut self) {
        let slice = self.source[self.start..self.current].to_vec();
        self.tokens.push(Token::Symbol(slice));
    }

    fn at_end(&self) -> bool { 
        self.current >= self.source.len()
    }

    fn is_more_token(&self) -> bool {
        match self.peek() {
            None => false,
            Some(ch) if is_whitespace(ch) || (ch == b')') => false,
            _ => true,
        }
    }

    fn advance(&mut self) -> Option<u8> {
        if self.at_end() { return None; }
        let ch = self.source[self.current];
        self.current += 1;
        Some(ch)
    }

    fn peek(&self) -> Option<u8> {
        if self.at_end() { return None; }
        Some(self.source[self.current])
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn s(x: &'static str) -> Token { Token::Symbol(x.as_bytes().to_vec()) }
    fn st(x: &'static str) -> Token { Token::Str(x.as_bytes().to_vec()) }
    fn i(x: i64) -> Token { Token::Int(x) }
    fn f(x: f64) -> Token { Token::Float(x) }
    fn op() -> Token { Token::OpenParen }
    fn cp() -> Token { Token::ClosedParen }
    
    fn tokens(x: &'static str) -> Result<Vec<Token>, ParseError> { 
        let xstr = x.as_bytes();
        let mut scanner = Scanner::new(xstr);
        scanner.scan_tokens()
    }

    fn scan_ok(x: &'static str, expected: Vec<Token>) {
        let res = tokens(x);
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), expected);
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
            ("(#$%-a)", vec![op(), s("#$%-a"), cp()]),
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
    fn test_advance() {
        let mut scanner = Scanner::new(b"123");
        assert_eq!(scanner.advance(), Some(b'1'));
        assert_eq!(scanner.advance(), Some(b'2'));
        assert_eq!(scanner.advance(), Some(b'3'));
        assert_eq!(scanner.advance(), None);
    }

    #[test]
    fn test_peek() {
        let mut scanner = Scanner::new(b"1");
        assert_eq!(scanner.peek(), Some(b'1'));
        assert_eq!(scanner.advance(), Some(b'1'));
        assert_eq!(scanner.peek(), None);
        assert_eq!(scanner.advance(), None);
    }

    #[test]
    fn test_at_end() {
        let mut scanner = Scanner::new(b"12345");
        for _ in 0..5 { scanner.advance(); }
        assert!(scanner.at_end());
    }
}