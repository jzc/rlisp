use std::collections::HashMap;
use crate::ast::Expr;

struct Interpreter<'a> {
    environment: HashMap<&'a [u8], &'a Expr>,
    expr: Option<&'a Expr>,
}

struct Memory {
    mem: Vec<Expr>
}

impl<'a> Interpreter<'a> { 
    pub fn new() -> Self {
        Interpreter { environment: HashMap::new(), expr: None }
    }

    pub fn eval(&mut self, e: &'a Expr) {
        self.expr = Some(e);
    }
}

