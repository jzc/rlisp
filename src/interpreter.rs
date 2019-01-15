use crate::sexpr::{SExpr, Memory, Object, Primitive, Environment};
use crate::scanner::Scanner;
use crate::parser::Parser;

pub struct Interpreter<'s> {
    mem: Memory<'s>,
    initial_env: SExpr<'s>,
}

impl<'s> Interpreter<'s> {
    pub fn new(memsize: usize) -> Self {
        let mut mem = Memory::new(memsize);
        let mut env = Environment::new(SExpr::Nil);
        env.set("+", mem.alloc(Object::PrimitiveProcedure(Primitive::Add)));
        let initial_env = mem.alloc(Object::Env(env));
        Interpreter { mem, initial_env }
    }

//     pub fn setup_intial_env(&'m mut self) {
//         let mut env = Environment::new(SExpr::Nil);

//         env.set("+", SExpr::new_object(&self.mem, Object::PrimitiveProcedure(Interpreter::add)));

//         self.initial_env = SExpr::new_object(&self.mem, Object::Env(env));
//     }

//     fn add(x: SExpr<'s, 'm>) -> SExpr<'s, 'm> {
//         unimplemented!()
//     }

    pub fn eval_string(&mut self, s: &'s str) -> Result<SExpr<'s>, &'static str> {
        let scanner = Scanner::new(s);
        let tokens = scanner.scan_tokens().expect("scan err");
        let mut parser = Parser::new(tokens, &mut self.mem);
        let expr = parser.parse().expect("parse err");
        self.eval(expr)
    }

    pub fn eval(&mut self, e: SExpr<'s>) -> Result<SExpr<'s>, &'static str> {
        self._eval(e, self.initial_env)
    }

    fn _eval(&mut self, e: SExpr<'s>, env: SExpr<'s>) -> Result<SExpr<'s>, &'static str> {
        match e {
            // values
            v @ SExpr::Int(_) => Ok(v),
            v @ SExpr::Float(_) => Ok(v),
            v @ SExpr::Str(_) => Ok(v),
            v @ SExpr::Nil => Ok(v),
            // variable
            SExpr::Sym(s) => self.eval_var(s, env),
            SExpr::Ref(addr) => match self.mem.get(addr) {
                // objects
                Object::PrimitiveProcedure(_) => Ok(SExpr::Ref(addr)),
                Object::CompoundProcedure(_) => Ok(SExpr::Ref(addr)),
                Object::Env(_) => Ok(SExpr::Ref(addr)),
                // special forms
                Object::Pair(SExpr::Sym("quote"), _) => unimplemented!(),
                Object::Pair(SExpr::Sym("set!"), _) => unimplemented!(),
                Object::Pair(SExpr::Sym("define"), _) => unimplemented!(),
                Object::Pair(SExpr::Sym("if"), _) => unimplemented!(),
                Object::Pair(SExpr::Sym("begin"), _) => unimplemented!(),
                Object::Pair(SExpr::Sym("cond"), _) => unimplemented!(),
                // application
                &Object::Pair(operator, operands) => self.eval_application(operator, operands, env),
                // should not occur
                Object::Empty(_) => Err("dereferencing empty location"),
            }
        }
    }

    fn eval_var(&self, k: &'s str, env: SExpr<'s>) -> Result<SExpr<'s>, &'static str> {
        match self.mem.env_get(k, env) {
            Ok(e) => Ok(e),
            Err(_) => Err("Unbound variable"),
        }
    }

    fn eval_application(&mut self, operator: SExpr<'s>, operands: SExpr<'s>, env: SExpr<'s>) -> Result<SExpr<'s>, &'static str> {
        match self._eval(operator, env)? {
            SExpr::Ref(addr) => match self.mem.get(addr) {
                &Object::PrimitiveProcedure(p) => {
                    let ops = self.eval_operands(operands, env)?;
                    self.eval_primitive(p, ops)
                }
                Object::CompoundProcedure(l) => unimplemented!(),
                _ => Err("Applying non procedure"),
            }
            _ => Err("Applying non procedure"),
        }
    }

    fn eval_operands(&mut self, operands: SExpr<'s>, env: SExpr<'s>) -> Result<Vec<SExpr<'s>>, &'static str> {
        let vec_op = self.mem.vec_from_list(operands).or(Err("Ill formed list"))?;
        let vec_op_evalr: Result<Vec<SExpr<'s>>, &'static str> = vec_op.iter().map(|&e| self._eval(e, env)).collect();
        vec_op_evalr
        // self.mem.list_from_vec(vec_op_evalr?).or(Ok(SExpr::Nil))
    }

    fn eval_primitive(&self, p: Primitive, operands: Vec<SExpr<'s>>) -> Result<SExpr<'s>, &'static str> {
        match p {
            Primitive::Add => operands.iter().try_fold(SExpr::Int(0), |acc, &e| match (acc, e) {
                (SExpr::Int(acc), SExpr::Int(x)) => Ok(SExpr::Int(acc+x)),
                (SExpr::Int(acc), SExpr::Float(x)) => Ok(SExpr::Float(acc as f64+x)),
                (SExpr::Float(acc), SExpr::Int(x)) => Ok(SExpr::Float(acc+x as f64)),
                (SExpr::Float(acc), SExpr::Float(x)) => Ok(SExpr::Float(acc+x)),
                _ => Err("Type error")
            }),
            Primitive::Sub => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f<'s>(x: f64) -> SExpr<'s> { SExpr::Float(x) }
    fn i<'s>(x: i64) -> SExpr<'s> { SExpr::Int(x) }
    fn sy<'s>(x: &'s str) -> SExpr<'s> { SExpr::Sym(x) }
    fn st<'s>(x: &'s str) -> SExpr<'s> { SExpr::Str(x) }
    fn n<'s>() -> SExpr<'s> { SExpr::Nil }

    #[test]
    fn test_values() {
        let mut interpreter = Interpreter::new(500);
        let mut test = |a, b| {
            let res = interpreter.eval_string(a);
            assert!(res.is_ok());
            assert_eq!(res.ok().unwrap(), b);
        };
        test("1", i(1));
        test("2", i(2));
        test("1.0", f(1.0));
        test("()", n());
        test("\"abc\"", st("abc"));
    }

    #[test]
    fn test_add() {
        let mut interpreter = Interpreter::new(500);
        let mut test = |a, b| {
            let res = interpreter.eval_string(a);
            assert!(res.is_ok(), "{:?}", res.err().unwrap());
            assert_eq!(res.ok().unwrap(), b);
        };
        test("(+ 1 2)", i(3));
        test("(+)", i(0));
        test("(+ (+ 1 2) (+ 3 4))", i(10));
    }
}
