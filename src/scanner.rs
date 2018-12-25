type AsciiString = Vec<u8>;

#[derive(PartialEq, Debug)]
pub enum Token {
    OpenParen,
    ClosedParen,
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
}

fn is_whitespace(ch: u8) -> bool {
    (ch == b' ') || (ch == b'\n') || ch == (b'\r')
}

impl Scanner {
    pub fn new<'a>(source: &'a [u8]) -> Self {
        Scanner {
            source: source.to_vec(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();
        while let Some(c) = self.advance() {
            match c {
                b' ' => continue,
                b'\n' => self.line += 1,
                b'(' => tokens.push(Token::OpenParen),
                b')' => tokens.push(Token::ClosedParen),
                ch => {
                    let mut symbol = vec![ch];
                    loop {
                        match self.peek() {
                            None => break,
                            Some(ch) if is_whitespace(ch) | (ch == b')') => break,
                            Some(ch) if ch == b'(' => {
                                return Err(ParseError {
                                    message: "Illegal character '(' in symbol".to_string(),
                                    line: self.line
                                })
                            }
                            Some(ch) => symbol.push(ch),
                        }
                        self.advance();
                    }
                    tokens.push(Token::Symbol(symbol));
                }
            }
        }
        Ok(tokens)
    }   

    fn at_end(&self) -> bool { 
        self.current >= self.source.len()
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
            Token::OpenParen, s(b"1"), s(b"2"), s(b"3"), Token::ClosedParen,
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