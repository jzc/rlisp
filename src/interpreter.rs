use crate::sexpr::{SExpr, Memory, Object, Environment};
use crate::scanner::Scanner;
use crate::parser::Parser;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Primitive {
    Add, Sub, Mul, Div, 
    Eql, Gt, Gte, Lt, Lte,
}

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
        env.insert("+", self.mem.alloc(Object::PrimitiveProcedure(Primitive::Add)));
        env.insert("-", self.mem.alloc(Object::PrimitiveProcedure(Primitive::Sub)));
        env.insert("*", self.mem.alloc(Object::PrimitiveProcedure(Primitive::Mul)));
        env.insert("/", self.mem.alloc(Object::PrimitiveProcedure(Primitive::Div)));
        env.insert("=",  self.mem.alloc(Object::PrimitiveProcedure(Primitive::Eql)));
        env.insert("<",  self.mem.alloc(Object::PrimitiveProcedure(Primitive::Lt)));
        env.insert("<=", self.mem.alloc(Object::PrimitiveProcedure(Primitive::Lte)));
        env.insert(">",  self.mem.alloc(Object::PrimitiveProcedure(Primitive::Gt)));
        env.insert(">=", self.mem.alloc(Object::PrimitiveProcedure(Primitive::Gte)));
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
            v @ SExpr::Bool(_) => Ok(v),
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
                &Object::Pair(SExpr::Sym("set!"), e) => self.eval_set(e, env),
                &Object::Pair(SExpr::Sym("define"), e) => self.eval_define(e, env),
                &Object::Pair(SExpr::Sym("if"), e) => self.eval_if(e, env),
                &Object::Pair(SExpr::Sym("begin"), e) => {
                    let seq = self.eval_sequence(e, env)?;
                    Ok(seq[seq.len()-1])
                }
                &Object::Pair(SExpr::Sym("lambda"), e) => self.eval_lambda(e, env),
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
        
        macro_rules! comparison_fold {
            ( $operands:expr, $op:tt) => {
                {
                    let operands = $operands;
                    if operands.len() == 0 {
                        Ok(SExpr::Bool(true))
                    } else {
                        let mut prev = operands[0];
                        let mut acc = Ok(true);
                        for &e in &operands[1..] {
                            match acc {
                                Ok(true) => (),
                                Ok(false) => break,
                                Err(_) => break,
                            }
                            let (accp, prevp) = match (prev, e) {
                                (SExpr::Int(prev), SExpr::Int(x)) => (Ok(prev $op x), SExpr::Int(x)),
                                (SExpr::Int(prev), SExpr::Float(x)) => (Ok((prev as f64) $op x), SExpr::Float(x)),
                                (SExpr::Float(prev), SExpr::Int(x)) => (Ok(prev $op x as f64), SExpr::Int(x)),
                                (SExpr::Float(prev), SExpr::Float(x)) => (Ok(prev $op x), SExpr::Float(x)),
                                _ => (Err("Type error"), prev),
                            };
                            acc = accp;
                            prev = prevp;
                        }
                        match acc {
                            Ok(res) => Ok(SExpr::Bool(res)),
                            Err(s) => Err(s),
                        }
                    }
                }
            };
        }

        match procd {
            Primitive::Add => afold1!(operands, SExpr::Int(0), +),
            Primitive::Sub => afold2!(operands, -),
            Primitive::Mul => afold1!(operands, SExpr::Int(1), *),
            Primitive::Div => afold2!(operands, /),
            Primitive::Eql => comparison_fold!(operands, ==),
            Primitive::Lt  => comparison_fold!(operands, <),
            Primitive::Lte => comparison_fold!(operands, <=),
            Primitive::Gt  => comparison_fold!(operands, >),
            Primitive::Gte => comparison_fold!(operands, >=),
        }
    }

    fn eval_lambda(&mut self, form: SExpr<'s>, env: SExpr<'s>) -> Result<SExpr<'s>, &'static str> {
        let form_vec = self.mem.vec_from_list(form).or(Err("ill formed"))?;
        if form_vec.len() < 1 { 
            Err("ill formed")
        } else {
            let params = form_vec[0];
            let body = self.mem.get_nth_ref(form, 1).unwrap();
            self.make_procedure(params, body, env)
        }
    }

    fn make_procedure(&mut self, params: SExpr<'s>, body: SExpr<'s>, env: SExpr<'s>) -> Result<SExpr<'s>, &'static str> {
        let param_vec = self.mem.vec_from_list(params).or(Err("err"))?;
        if param_vec.iter().any(|e| if let SExpr::Sym(_) = e { false } else { true }) {
            Err("ill formed")
        } else {
            let list = self.mem.list_from_vec(vec![params, body, env]).unwrap();
            let r = self.mem.alloc(Object::CompoundProcedure(list));
            Ok(r)
        }
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
                envp.insert(s, arg)
            }
        }
        let renvp = self.mem.alloc(Object::Env(envp));
        let res = self.eval_sequence(body, renvp)?;
        Ok(res[res.len()-1])
    }

    fn eval_define(&mut self, form: SExpr<'s>, env: SExpr<'s>) -> Result<SExpr<'s>, &'static str> {
        let vec = self.mem.vec_from_list(form).or(Err("ill formed form"))?;
        if vec.len() < 2 {
            Err("ill formed define")
        } else {
            match vec[0] {
                SExpr::Sym(ident) => if vec.len() == 2 {
                    let evaled = self._eval(vec[1], env)?;
                    self.mem.env_insert(env, ident, evaled).or(Err("err"))?;
                    Ok(SExpr::Sym(ident))
                } else {
                    Err("ill formed")
                }
                l @ SExpr::Ref(_) => {
                    let fn_vec = self.mem.vec_from_list(l).or(Err("err"))?;
                    if fn_vec.len() == 0 {
                        Err("ill formed")
                    } else {
                        let ident = if let SExpr::Sym(s) = fn_vec[0] { s } else { unreachable!() };
                        let params = self.mem.get_nth_ref(l, 1).unwrap();
                        let body = self.mem.get_nth_ref(form, 1).unwrap();
                        let procd = self.make_procedure(params, body, env)?;
                        self.mem.env_insert(env, ident, procd).or(Err("err"))?;
                        Ok(SExpr::Sym(ident))
                    }
                }
                _ => Err("ill formed"),
            }
        }
    }

    fn eval_set(&mut self, form: SExpr<'s>, env: SExpr<'s>) -> Result<SExpr<'s>, &'static str> {
        let vec = self.mem.vec_from_list(form).or(Err("ill formed"))?;
        if vec.len() == 2 {
            if let SExpr::Sym(ident) = vec[0] {
                let evaled = self._eval(vec[1], env)?;
                self.mem.env_set(env, ident, evaled).or(Err("err"))?;
                Ok(SExpr::Nil)
            } else {
                Err("Expected symbol")
            }
        } else {
            Err("ill formed")
        }
    }

    fn eval_if(&mut self, form: SExpr<'s>, env: SExpr<'s>) -> Result<SExpr<'s>, &'static str> {
        let form_vec = self.mem.vec_from_list(form).or(Err("ill formed"))?;
        if form_vec.len() == 3 {
            let cond = self._eval(form_vec[0], env)?;
            if cond.as_bool() {
                self._eval(form_vec[1], env)
            } else {
                self._eval(form_vec[2], env)
            }
        } else {
            Err("ill formed")
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    fn f<'s>(x: f64) -> SExpr<'s> { SExpr::Float(x) }
    fn i<'s>(x: i64) -> SExpr<'s> { SExpr::Int(x) }
    fn b<'s>(x: bool) -> SExpr<'s> { SExpr::Bool(x) }
    fn sy<'s>(x: &'s str) -> SExpr<'s> { SExpr::Sym(x) }
    fn st<'s>(x: &'s str) -> SExpr<'s> { SExpr::Str(x) }
    fn n<'s>() -> SExpr<'s> { SExpr::Nil }

    macro_rules! eval_ok {
        ($s:expr, $ex:expr) => {
            let mut interpreter = Interpreter::new(500);
            let res = interpreter.eval_string($s).expect("err");
            assert_eq!(res, $ex);
        };
    }

    #[test]
    fn test_values() {
        eval_ok!("1", i(1));
        eval_ok!("2", i(2));
        eval_ok!("1.0", f(1.0));
        eval_ok!("()", n());
        eval_ok!("\"abc\"", st("abc"));
        eval_ok!("#t", b(true));
        eval_ok!("#f", b(false));
    }

    #[test]
    fn test_arithmetic() {
        eval_ok!("(+ 1 2)", i(3));
        eval_ok!("(+)", i(0));
        eval_ok!("(+ (+ 1 2) (+ 3 4))", i(10));
        eval_ok!("(+ 1 2 3 4 5 6 7 8)", i((1..9).fold(0, |acc, x| acc+x)));
        eval_ok!("(+ 1.0 0.5)", f(1.5));
        eval_ok!("(+ 1 0.5)", f(1.5));
        eval_ok!("(+ 1.0 2)", f(3.0));

        eval_ok!("(- 5 3)", i(2));
        eval_ok!("(- 10 1 2 3)", i(4));

        eval_ok!("(* 5 3)", i(15));
        eval_ok!("(*)", i(1));

        eval_ok!("(/ 5.0 2)", f(2.5));
    }

    #[test]
    fn test_lambda() {
        eval_ok!("((lambda (a) a) 5)", i(5));
        eval_ok!("((lambda (a b) (+ a b)) 5 3)", i(8));
        eval_ok!("((lambda () 5))", i(5));
        eval_ok!("((lambda () ()))", SExpr::Nil);
        eval_ok!("(((lambda (a) (lambda () a)) 1e3))", f(1e3));
    }

    #[test]
    fn test_begin() {
        eval_ok!("(begin 1 2 3 4 5)", i(5));
    }

    #[test]
    fn test_define() {
        eval_ok!("(begin (define a 1) a)", i(1));
        eval_ok!("(begin (define a 1) (define b a) b)", i(1));
        eval_ok!("(begin (define a 1) (define b 2) (+ a b))", i(3));
        eval_ok!("(begin (define a 1) (define b 2) (+ a b))", i(3));
        eval_ok!("(begin (define (fn) 5) (fn))", i(5));
        eval_ok!("(begin (define (fn a) a) (fn 5))", i(5));
        eval_ok!("(begin (define a 5) (define (fn) a) (fn))", i(5));
        eval_ok!("(begin (define a 5) (define (fn a) a) (fn 6))", i(6));
    }

    #[test]
    fn test_set() {
        eval_ok!("(begin (define a 1) (set! a 2) a)", i(2));
        eval_ok!("(begin (define a 1) (set! a (+ a 1)) a)", i(2));
        eval_ok!("(begin (define a 1) (define (inc) (set! a (+ a 1))) (inc) (inc) a)", i(3));
        eval_ok!("
            (begin
                (define a 1)
                (define (inc1) (set! a (+ a 1)))
                (define (inc2) (set! a (+ a 2)))
                (inc1)
                (inc2)
                (inc1)
                a
            )
        ", i(5));
    }

    #[test]
    fn test_comparison() {
        eval_ok!("(= 1 1)", b(true));
        eval_ok!("(= 1 1 2)", b(false));
        eval_ok!("(=)", b(true));
        eval_ok!("(<)", b(true));
        eval_ok!("(<=)", b(true));
        eval_ok!("(>)", b(true));
        eval_ok!("(>=)", b(true));
        eval_ok!("(= 1)", b(true));
        eval_ok!("(< 1 2)", b(true));
        eval_ok!("(< 1 2.0)", b(true));
        eval_ok!("(< 1.0 2.0)", b(true));
        eval_ok!("(< 1.0 2)", b(true));
        eval_ok!("(< 1.0 2 3.0)", b(true));
        eval_ok!("(< 1.0 2 2)", b(false));
        eval_ok!("(< 1.0 2 2.1)", b(true));
        eval_ok!("(<= 1 1 1)", b(true));
        eval_ok!("(<= 1 2 2)", b(true));
        eval_ok!("(<= 1 2 3)", b(true));
        eval_ok!("(<= 1 2 )", b(true));
        eval_ok!("(> 2 1 )", b(true));
        eval_ok!("(> 2 1 1)", b(false));
        eval_ok!("(>= 2 1 1)", b(true));
        eval_ok!("(>= 2 1)", b(true));
        eval_ok!("(>= 2)", b(true));
    }

    #[test]
    fn test_if() {
        eval_ok!("(if #t 1 2)", i(1));
        eval_ok!("(if #f 1 2)", i(2));
        eval_ok!("(if (= 1 1) 1 2)", i(1));
        eval_ok!("(if (= 1 2) 1 2)", i(2));
    }
}
