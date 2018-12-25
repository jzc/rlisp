type AsciiString = Vec<u8>;

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    OpenParen,
    ClosedParen,
    Int(i64),
    Float(f64),
    String(AsciiString),
    Symbol(AsciiString),
}

pub struct ParseError {
    message: String,
    line: usize,
}

pub struct Scanner {
    source: AsciiString,
    start: usize,
    current: usize,
    line: usize,
    tokens: Vec<Token>
}

fn is_whitespace(ch: u8) -> bool {
    (ch == b' ') || (ch == b'\n')
}

fn is_numeric(ch: u8) -> bool {
    (ch >= b'0') && (ch <= b'9')
}

const missing_quote: &str = "Missing quote '\"'";
const unexpected_quote: &str = "Unexpected quote '\"'";
const open_paren_in_atom: &str = "Found illegal opening paren '(' in atom";

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
        Err(ParseError { message: message.to_string(), line: self.line })
    }

    fn scan_token(&mut self) -> Result<(), ParseError> {
        let ch = self.advance();
        match ch.unwrap() {
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
                b'"' => return self.parse_err(unexpected_quote),
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
            Some(ch) if (ch == b'+') || (ch == b'-') => { self.advance(); },
            _ => return self.symbol(),
        }

        while self.is_more_token() {
            match self.advance().unwrap() {
                ch if is_numeric(ch) => (),
                _ => return self.symbol(),
            }
        }
        
        self.add_float_token();
        Ok(())
    }
    
    fn string(&mut self) -> Result<(), ParseError> {
        loop {
            match self.advance() {
                None => return self.parse_err(missing_quote),
                Some(ch) => match ch {
                    b'"' => { self.add_string_token(); return Ok(()); }
                    b'\n' => return self.parse_err(missing_quote),
                    _ => ()
                }
            }
        }
    }

    fn symbol(&mut self) -> Result<(), ParseError> {
        while self.is_more_token() {
            match self.advance().unwrap() {
                b'(' => return self.parse_err(open_paren_in_atom),
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
        self.tokens.push(Token::String(slice));
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
        if self.at_end() {
            return None;
        }
        let ch = self.source[self.current];
        self.current += 1;
        Some(ch)
    }

    fn peek(&self) -> Option<u8> {
        if self.at_end() {
            return None;
        }
        Some(self.source[self.current])
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_tokens() {
        let mut scanner;
        let mut res;
        let s = |x: &[u8]| Token::Symbol(x.to_vec()); 
        let i = |x: i64| Token::Int(x);

        scanner = Scanner::new(b"()");
        res = scanner.scan_tokens();
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [Token::OpenParen, Token::ClosedParen]);
        
        scanner = Scanner::new(b"(ab( 1 2)");
        res = scanner.scan_tokens();
        assert!(res.is_err());
        assert_eq!(res.err().unwrap().line, 1);

        scanner = Scanner::new(b"(1 2\nab( 3)");
        res = scanner.scan_tokens();
        assert!(res.is_err());
        assert_eq!(res.err().unwrap().line, 2);

        let sources: Vec<&[u8]> = vec![
            b"(+ abc def)",
            b"( + abc def)",
            b"(+ abc def )",
            b"(+  abc    def)",
            b"    (+ abc def)  ",
            b"(+\nabc\ndef)",
        ];
        for si in sources {
            scanner = Scanner::new(si);
            res = scanner.scan_tokens();
            assert!(res.is_ok());
            assert_eq!(res.ok().unwrap(), [Token::OpenParen, s(b"+"), s(b"abc"), s(b"def"), Token::ClosedParen]);
        }

        scanner = Scanner::new(b"(() (1 2 3))");
        res = scanner.scan_tokens();
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [Token::OpenParen, 
            Token::OpenParen, Token::ClosedParen,
            Token::OpenParen, i(1), i(2), i(3), Token::ClosedParen,
            Token::ClosedParen]);

        scanner = Scanner::new(b"(#$%-a)");
        res = scanner.scan_tokens();
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), [Token::OpenParen, s(b"#$%-a"), Token::ClosedParen])
        
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