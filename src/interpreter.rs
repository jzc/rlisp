// use crate::sexpr::{SExpr, Memory, Object, Environment};
// use crate::scanner::Scanner;
// use crate::parser::Parser;

// pub struct Interpreter<'s, 'm> {
//     mem: Memory<'s, 'm>,
//     initial_env: SExpr<'s, 'm>,
// }

// impl<'s, 'm> Interpreter<'s, 'm> {
//     pub fn new(memsize: usize) -> Self {
//         Interpreter { mem: Memory::new(memsize), initial_env: SExpr::Nil }
//     }

//     pub fn setup_intial_env(&'m mut self) {
//         let mut env = Environment::new(SExpr::Nil);

//         env.set("+", SExpr::new_object(&self.mem, Object::PrimitiveProcedure(Interpreter::add)));

//         self.initial_env = SExpr::new_object(&self.mem, Object::Env(env));
//     }

//     fn add(x: SExpr<'s, 'm>) -> SExpr<'s, 'm> {
//         unimplemented!()
//     }

//     pub fn eval_string(&'m self, s: &'s str) -> Result<SExpr<'s, 'm>, &'static str> {
//         let scanner = Scanner::new(s);
//         let tokens = scanner.scan_tokens().expect("scan err");
//         let mut parser = Parser::new(tokens, &self.mem);
//         let expr = parser.parse().expect("parse err");
//         self.eval(expr)
//     }

//     pub fn eval(&self, e: SExpr<'s, 'm>) -> Result<SExpr<'s, 'm>, &'static str> {
//         self._eval(e, self.initial_env)
//     }

//     fn _eval(&self, e: SExpr<'s, 'm>, env: SExpr<'s, 'm>) -> Result<SExpr<'s, 'm>, &'static str> {
//         match e {
//             // values
//             v @ SExpr::Int(_)  => Ok(v),
//             v @ SExpr::Float(_)  => Ok(v),
//             v @ SExpr::Str(_)  => Ok(v),
//             v @ SExpr::Nil  => Ok(v),
//             // variable
//             SExpr::Sym(s) => self.eval_var(s, env),
//             SExpr::Ref(r) => match *r.borrow() {
//                 // objects
//                 Object::PrimitiveProcedure(_) => Ok(SExpr::Ref(r)),
//                 Object::CompoundProcedure(_, _) => Ok(SExpr::Ref(r)),
//                 Object::Env(_) => Ok(SExpr::Ref(r)),
//                 // special forms
//                 Object::Pair(SExpr::Sym("quote"), _) => unimplemented!(),
//                 Object::Pair(SExpr::Sym("set!"), _) => unimplemented!(),
//                 Object::Pair(SExpr::Sym("define"), _) => unimplemented!(),
//                 Object::Pair(SExpr::Sym("if"), _) => unimplemented!(),
//                 Object::Pair(SExpr::Sym("begin"), _) => unimplemented!(),
//                 Object::Pair(SExpr::Sym("cond"), _) => unimplemented!(),
//                 // application
//                 Object::Pair(operator, operands) => self.eval_application(operator, operands, env),
//                 // should not occur
//                 Object::Empty(_) => Err("dereferencing empty location"),
//             }
//         }
//     }

//     fn eval_var(&self, k: &'s str, env: SExpr<'s, 'm>) -> Result<SExpr<'s, 'm>, &'static str> {
//         match env.env_get(k) {
//             Ok(e) => Ok(e),
//             Err(_) => Err("Unbound variable"),
//         }
//     }

//     fn eval_application(&self, operator: SExpr<'s, 'm>, operands: SExpr<'s, 'm>, env: SExpr<'s, 'm>) -> Result<SExpr<'s, 'm>, &'static str> {
//        unimplemented!() 
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_values() {
//         let interpreter = Interpreter::new(500);
//         let res = interpreter.eval_string("1");
//         assert!(res.is_ok());
//         assert_eq!(res.ok().unwrap(), SExpr::Int(1));
//     }
// }
