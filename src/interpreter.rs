use crate::ast::{SExpr, Memory, Environment};

struct Interpreter<'a> {
    mem: Memory<'a>,
}

impl<'a> Interpreter<'a> { 
    pub fn new() -> Self {
        Interpreter { mem: mem }
    }

    pub fn eval(&mut self, e: SExpr<'a>, env: Environment<'a>) -> Result<SExpr, &'static str> {
        match e {
            // values
            v @ SExpr::Int(_)  => Ok(v),
            v @ SExpr::Float(_)  => Ok(v),
            v @ SExpr::Str(_)  => Ok(v),
            v @ SExpr::Nil  => Ok(v),
            // variables
            SExpr::Sym(s) => match env.get(s, &self.mem) {
                Some(sexpr) => Ok(sexpr),
                None => Err("variable not found"),
            }
            v @ SExpr::Ref(_) => Ok(v),
            // Expr::Symbol(s) => match env.get(&s, ) {
            //     Some(e) => Ok(e),
            //     None => Err("Unbound variable"),
            // }
            // Expr::List(exprs) => if let Some(Expr::Symbol(s)) = exprs.get(0) { 
            //     match s.as_slice() {
            //         b"define" => unimplemented!(),
            //         b"lambda" => unimplemented!(),
            //         b"set!" => unimplemented!(),
            //         b"if" => unimplemented!(),

            //         // primitive functions
            //         b"+" => unimplemented!(),
            //         b"-" => unimplemented!(),
            //         b"*" => unimplemented!(),
            //         b"/" => unimplemented!(),
            //         b"cons" => unimplemented!(),
            //         b"car" => unimplemented!(),
            //         b"cdr" => unimplemented!(),

            //         s => unimplemented!(),
            //     }
            // } else { Err("expected symbol") }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;
    use crate::parser::Parser;

    #[test]
    fn test_values() {
        let scanner = Scanner::new("1");
        let tokens = scanner.scan_tokens().ok().unwrap();
        let mut mem = Memory::new(100);
        let parser = Parser::new(tokens, &mut mem);
        let expr = parser.parse().ok().unwrap();
        let mut interpreter = Interpreter::new(mem);
        let environment = Environment::new(None);
        let res = interpreter.eval(expr, environment);
        assert!(res.is_ok());
        assert_eq!(res.ok().unwrap(), SExpr::Int(1));
    }
}
