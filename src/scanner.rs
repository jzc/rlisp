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

    #[test]
    fn test_scan_tokens() {
        let mut res;

        res = tokens("()");
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [op(), cp()]);
        
        res = tokens("(ab( 1 2)");
        assert!(res.is_err());
        assert_eq!(res.err().unwrap().line, 1);

        res = tokens("(1 2\nab( 3)");
        assert!(res.is_err());
        assert_eq!(res.err().unwrap().line, 2);

        let sources = vec![
            "(+ abc def)",
            "( + abc def)",
            "(+ abc def )",
            "(+  abc    def)",
            "    (+ abc def)  ",
            "(+\nabc\ndef)",
        ];
        for si in sources {
            res = tokens(si);
            assert!(res.is_ok());
            assert_eq!(res.ok().unwrap(), [op(), s("+"), s("abc"), s("def"), cp()]);
        }


        res = tokens("(() (1 2 3))");
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [Token::OpenParen, 
            Token::OpenParen, Token::ClosedParen,
            Token::OpenParen, i(1), i(2), i(3), Token::ClosedParen,
            Token::ClosedParen]);

        res = tokens("(#$%-a)");
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [op(), s("#$%-a"), cp()]);

        res = tokens("(() () ())");
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [op(), op(), cp(), op(), cp(), op(), cp(), cp()])
    }

    #[test]
    fn test_scan_int() {
        let mut res;

        res = tokens("123");
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [i(123)]);

        res = tokens("(1");
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [op(), i(1)]);

        let sources = vec![
            " 123",
            "0123",
            " 0123",
            " +0123",
            "+0123",
        ];
        for si in sources {
            res = tokens(si);
            assert!(res.is_ok());
            assert_eq!(res.ok().unwrap(), [i(123)]);
        }


        res = tokens(" +01123(3");
        assert!(res.is_err());

        res = tokens("(+011233)");
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [op(), i(11233), cp()]);
    }

    #[test]
    fn test_scan_string() {
        let mut res;

        res = tokens(r#" ("abc") "#);
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [op(), st("abc"), cp()]);

        res = tokens(r#" "(abc))())(" "#);
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [st("(abc))())(")]);

        res = tokens(" \"a\nb\"  ");
        assert!(res.is_err());

        res = tokens("(\")");
        assert!(res.is_err());

        res = tokens(r#"  ("abc" "def" ("ijk")) "#);
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [op(), st("abc"), st("def"), op(), st("ijk"), cp(), cp()]);
    }

    #[test]
    fn test_scan_float() {
        let mut res;
        let mut sources;

        sources = vec![
            "1.0",
            "01.0",
            "+1.0",
            "+01.0",
            "+01.",
            "01.",
            "1.",
            "1.00",
            "+1.00",
            "+01.00",
        ];
        for si in sources {
            res = tokens(si);
            assert!(res.is_ok());
            assert_eq!(res.ok().unwrap(), [f(1.0)]);
        }

        sources = vec![
            "1e1",
            "+1e1",
            "+1e+1",
            "1e+01",
            "001e01",
            "001.0e+01",
            "1.e+1",
            "1.e+001",
            "001.e01",
        ];
        
        for si in sources {
            res = tokens(si);
            assert!(res.is_ok());
            assert_eq!(res.ok().unwrap(), [f(1e1)]);
        }

        sources = vec![
            "1(e1",
            "1e1(",
            "1.e1(",
            "1.e(1",
        ];

        for si in sources {
            res = tokens(si);
            assert!(res.is_err());
        }

        res = tokens("1.1.");
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [s("1.1.")]);

        res = tokens("1.e15.");
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [s("1.e15.")]);

        res = tokens("1.e");
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [s("1.e")]);

        res = tokens(".1");
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [f(0.1)]);

        res = tokens(".");
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [s(".")]);

        res = tokens("3.14156e-03");
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [f(3.14156e-03)]);
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