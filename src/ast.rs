pub type AsciiString = Vec<u8>;

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    OpenParen,
    ClosedParen,
    Int(i64),
    Float(f64),
    Str(AsciiString),
    Symbol(AsciiString),
}

pub struct ParseError {
    pub message: &'static str,
    pub line: usize,
}

#[derive(PartialEq, Debug)]
pub enum Expr {
    Null,
    Int(i64),
    Float(f64),
    Str(AsciiString),
    Symbol(AsciiString),
    List(Vec<Box<Expr>>),
}