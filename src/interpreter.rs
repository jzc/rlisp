use crate::sexpr::{SExpr, Memory, Object, Primitive, Environment};
use crate::scanner::Scanner;
use crate::parser::Parser;

pub struct Interpreter<'s> {
    mem: Memory<'s>,
    initial_env: SExpr<'s>,
}

impl<'s> Interpreter<'s> {
    pub fn new(memsize: usize) -> Self {
        let mut obj = Interpreter { mem: Memory::new(memsize), initial_env: SExpr::Nil };
        obj.setup_intial_env();
        obj
    }

    pub fn setup_intial_env(&mut self) {
        let mut env = Environment::new(SExpr::Nil);
        env.set("+", self.mem.alloc(Object::PrimitiveProcedure(Primitive::Add)));
        env.set("-", self.mem.alloc(Object::PrimitiveProcedure(Primitive::Sub)));
        env.set("*", self.mem.alloc(Object::PrimitiveProcedure(Primitive::Mul)));
        env.set("/", self.mem.alloc(Object::PrimitiveProcedure(Primitive::Div)));
        self.initial_env = self.mem.alloc(Object::Env(env));
    }

    pub fn eval_string(&mut self, s: &'s str) -> Result<SExpr<'s>, &'static str> {
        let scanner = Scanner::new(s);
        let tokens = scanner.scan_tokens().expect("scan err");
        let parser = Parser::new(tokens, &mut self.mem);
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
                &Object::Pair(SExpr::Sym("define"), e) => self.eval_define(e, env),
                Object::Pair(SExpr::Sym("if"), _) => unimplemented!(),
                &Object::Pair(SExpr::Sym("begin"), e) => {
                    let seq = self.eval_sequence(e, env)?;
                    Ok(seq[seq.len()-1])
                }
                &Object::Pair(SExpr::Sym("lambda"), e) => self.make_procedure(e, env),
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
                &Object::PrimitiveProcedure(procd) => {
                    let ops = self.eval_sequence(operands, env)?;
                    self.eval_primitive(procd, ops)
                }
                &Object::CompoundProcedure(procd) => {
                    let ops = self.eval_sequence(operands, env)?;
                    self.eval_compound(procd, ops)
                },
                _ => Err("Applying non procedure"),
            }
            _ => Err("Applying non procedure"),
        }
    }

    fn eval_sequence(&mut self, operands: SExpr<'s>, env: SExpr<'s>) -> Result<Vec<SExpr<'s>>, &'static str> {
        let vec_op = self.mem.vec_from_list(operands).or(Err("Ill formed list"))?;
        let vec_op_evalr: Result<Vec<SExpr<'s>>, &'static str> = vec_op.iter().map(|&e| self._eval(e, env)).collect();
        vec_op_evalr
        // self.mem.list_from_vec(vec_op_evalr?).or(Ok(SExpr::Nil))
    }    

    fn eval_primitive(&self, procd: Primitive, operands: Vec<SExpr<'s>>) -> Result<SExpr<'s>, &'static str> {
        macro_rules! arithmetic_fold { 
            ( $op_iter:expr, $initial:expr, $op:tt) => {
                $op_iter.try_fold($initial, |acc, &e| match (acc, e) {
                    (SExpr::Int(acc), SExpr::Int(x)) => Ok(SExpr::Int(acc $op x)),
                    (SExpr::Int(acc), SExpr::Float(x)) => Ok(SExpr::Float(acc as f64 $op x)),
                    (SExpr::Float(acc), SExpr::Int(x)) => Ok(SExpr::Float(acc $op x as f64)),
                    (SExpr::Float(acc), SExpr::Float(x)) => Ok(SExpr::Float(acc $op x)),
                    _ => Err("Type error")
                })
            };
        }

        macro_rules! afold1 {
            ( $operands:expr, $initial:expr, $op:tt ) => {
                arithmetic_fold!($operands.iter(), $initial, $op)
            };
        }

        macro_rules! afold2 {
            ( $operands:expr, $op:tt ) => {
                {
                    let operands = $operands;
                    if operands.len() >= 1 {
                        let mut iter = operands.iter();
                        let first = *iter.next().unwrap();
                        arithmetic_fold!(iter, first, $op)
                    } else {
                        Err("Not enough operands")
                    }
                }
            };
        }

        match procd {
            Primitive::Add => afold1!(operands, SExpr::Int(0), +),
            Primitive::Sub => afold2!(operands, -),
            Primitive::Mul => afold1!(operands, SExpr::Int(1), *),
            Primitive::Div => afold2!(operands, /),
        }
    }

    fn make_procedure(&mut self, e: SExpr<'s>, env: SExpr<'s>) -> Result<SExpr<'s>, &'static str> {
        let form = dbg!(self.mem.vec_from_list(e).or(Err("Ill formed special form")))?;
        if form.len() < 1 { return Err("Ill formed special form"); }
        // if let SExpr::Ref(_) = form[0] {} else { return Err("Ill formed special form"); }
        let params = form[0];
        let body = self.mem.get_nth_ref(e, 1).unwrap();
        let list = self.mem.list_from_vec(vec![params, body, env]).unwrap();
        let r = self.mem.alloc(Object::CompoundProcedure(list));
        Ok(r)
    }

    fn eval_compound(&mut self, procd: SExpr<'s>, operands: Vec<SExpr<'s>>) -> Result<SExpr<'s>, &'static str> {
        let vec = self.mem.vec_from_list(procd).or(Err("Ill formed procedure"))?;
        let params = vec[0];
        let body = vec[1];
        let env = vec[2];
        let param_vec = self.mem.vec_from_list(params).or(Err("something"))?;
        let mut envp = Environment::new(env);
        for (param, arg) in param_vec.iter().zip(operands) {
            if let &SExpr::Sym(s) = param {
                envp.set(s, arg)
            }
        }
        let renvp = self.mem.alloc(Object::Env(envp));
        let res = self.eval_sequence(body, renvp)?;
        Ok(res[res.len()-1])
    }

    fn eval_define(&mut self, form: SExpr<'s>, env: SExpr<'s>) -> Result<SExpr<'s>, &'static str> {
        let vec = self.mem.vec_from_list(form).or(Err("ill formed form"))?;
        match vec.len() {
            l if l < 2 => Err("ill formed define"),
            // variable definition
            2 => if let SExpr::Sym(ident) = vec[0] {
                let evaled = self._eval(vec[1], env)?;
                self.mem.env_set(env, ident, evaled).or(Err("err"))?;
                Ok(SExpr::Sym(ident))
            } else {
                Err("expected symbol")
            }
            // unction definition
            _ => {
                unimplemented!()
            },
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
    fn test_arithmetic() {
        let test = |a, b| {
            let mut interpreter = Interpreter::new(500);
            let res = interpreter.eval_string(a);
            assert!(res.is_ok(), "{:?}", res.err().unwrap());
            assert_eq!(res.ok().unwrap(), b);
        };
        test("(+ 1 2)", i(3));
        test("(+)", i(0));
        test("(+ (+ 1 2) (+ 3 4))", i(10));
        test("(+ 1 2 3 4 5 6 7 8)", i((1..9).fold(0, |acc, x| acc+x)));
        test("(+ 1.0 0.5)", f(1.5));
        test("(+ 1 0.5)", f(1.5));
        test("(+ 1.0 2)", f(3.0));

        test("(- 5 3)", i(2));
        test("(- 10 1 2 3)", i(4));

        test("(* 5 3)", i(15));
        test("(*)", i(1));

        test("(/ 5.0 2)", f(2.5));
    }

    #[test]
    fn test_lambda() {
        let test = |a, b| {
            let mut interpreter = Interpreter::new(500);
            let res = interpreter.eval_string(a).expect("err");
            // assert!(res.is_ok(), "{:?}", res.err().unwrap());
            assert_eq!(res, b);
        };

        test("((lambda (a) a) 5)", i(5));
        test("((lambda (a b) (+ a b)) 5 3)", i(8));
        test("((lambda () 5))", i(5));
        test("((lambda () ()))", SExpr::Nil);
        test("(((lambda (a) (lambda () a)) 1e3))", f(1e3));
        test("(begin 1 2 3)", i(3));
    }

    #[test]
    fn test_begin() {
        let test = |a, b| {
            let mut interpreter = Interpreter::new(500);
            let res = interpreter.eval_string(a).expect("err");
            // assert!(res.is_ok(), "{:?}", res.err().unwrap());
            assert_eq!(res, b);
        };
        test("(begin (define a 1) a)", i(1));
        test("(begin (define a 1) (define b a) b)", i(1));
        test("(begin (define a 1) (define b 2) (+ a b))", i(3));
        test("(begin (define a 1) (define b 2) (+ a b))", i(3));
        test("(begin (define a (lambda () 5)) (a))", i(5));
        test("(begin (define a 5) (define f (lambda (a) a)) (f a))", i(5));
    }
}
