// use crate::ast::{SExpr, Object, Memory, Environment, Location};

// pub struct Interpreter<'a> {
//     mem: Memory<'a>,
//     initial_env: Environment<'a>,
//     initial_env_loc: Location,
// }

// impl<'a> Interpreter<'a> { 
//     pub fn new(memsize: usize) -> Self {
//         let mut env = Environment::new(None);
//         let mut mem = Memory::new(memsize);
//         let loc = mem.alloc(Object::PrimitiveProcedure(Interpreter::add));
//         env.set("+", SExpr::Ref(loc));
//         Interpreter { mem: mem, initial_env: env, initial_env_loc: loc }
//     }

//     // fn add(mem: &'b Memory<'a>, mut args: SExpr<'a>) -> SExpr<'a> {
//     //     let mut sum = 0;
//     //     loop {
//     //         match args {
//     //             SExpr::Ref(loc) => match mem.get(loc) {
//     //                 &Object::Cons(car, cdr) => {
//     //                     match car {
//     //                        SExpr::Int(i) => sum += i,
//     //                         _ => panic!("Not a number"),
//     //                     }
//     //                     match cdr {
//     //                         v @ SExpr::Ref(_) | v @ SExpr::Nil => args = v,
//     //                         _ => panic!("Improper argument list"),
//     //                     }
//     //                 }
//     //                 _ => panic!("Improper argument list"),
//     //             }
//     //             SExpr::Nil => break,
//     //             _ => panic!("Improper argument list"),
//     //         }
//     //     }
//     //     SExpr::Int(sum)
//     // }

//     fn setup_intial_env() -> Environment<'a> {
//         unimplemented!()
//     }

//     pub fn eval(&mut self, e: SExpr<'a>) -> Result<SExpr<'a>, &'static str> {
//         self._eval(e, self.initial_env_loc)
//     }

//     fn _eval(&mut self, e: SExpr<'a>, env_loc: Location) -> Result<SExpr<'a>, &'static str> {
//         match e {
//             // values
//             v @ SExpr::Int(_)  => Ok(v),
//             v @ SExpr::Float(_)  => Ok(v),
//             v @ SExpr::Str(_)  => Ok(v),
//             v @ SExpr::Nil  => Ok(v),
//             // variable
//             SExpr::Sym(s) => self.eval_var(s, env_loc),
//             SExpr::Ref(loc) => match self.mem.get(loc) {
//                 // objects
//                 Object::PrimitiveProcedure(_) => Ok(SExpr::Ref(loc)),
//                 Object::CompoundProcedure(_, _) => Ok(SExpr::Ref(loc)),
//                 Object::Env(_) => Ok(SExpr::Ref(loc)),
//                 // special forms
//                 Object::Cons(SExpr::Sym("quote"), _) => unimplemented!(),
//                 Object::Cons(SExpr::Sym("set!"), _) => unimplemented!(),
//                 Object::Cons(SExpr::Sym("define"), _) => unimplemented!(),
//                 Object::Cons(SExpr::Sym("if"), _) => unimplemented!(),
//                 Object::Cons(SExpr::Sym("begin"), _) => unimplemented!(),
//                 Object::Cons(SExpr::Sym("cond"), _) => unimplemented!(),
//                 // application
//                 &Object::Cons(operator, operands) => self.eval_application(operator, operands, env_loc),
//                 // should not occur
//                 Object::Empty(_) => Err("dereferencing empty location"),
//             }
//         }
//     }

//     fn eval_var(&mut self, var: &'a str, env_loc: Location) -> Result<SExpr<'a>, &'static str> {
//         match self.mem.get(env_loc) {
//             Object::Env(env) => match env.get(var, &self.mem) {
//                 Some(sexpr) => Ok(sexpr),
//                 None => Err("Unbound variable"),
//             }
//             _ => Err("Env_loc does not reference an environment"),
//         }
//     }

//     fn eval_application(&mut self, operator: SExpr<'a>, operands: SExpr<'a>, env_loc: Location) -> Result<SExpr<'a>, &'static str> {
//         match self._eval(operator, env_loc)? {
//             SExpr::Ref(r) => match self.mem.get(r) {
//                 Object::PrimitiveProcedure(proc_fn) => {
//                     let operands = self.eval_operands(operands, env_loc)?;
//                     Ok(proc_fn(&self.mem, operands))
//                 },
//                 Object::CompoundProcedure(_, _) => unimplemented!(),
//                 _ => Err("Operator is not procedure"),
//             }
//             _ => Err("Operator is not procedure"),
//         }
//     }

//     fn eval_operands(&mut self, mut operands: SExpr<'a>, env_loc: Location) -> Result<SExpr<'a>, &'static str> {
//         let mut head = SExpr::Nil;
//         let mut tail = None;
//         loop {
//             match operands {
//                 SExpr::Ref(r) => match self.mem.get(r) {
//                     &Object::Cons(car, cdr) => {
//                         let eval_car = self._eval(car, env_loc)?;
//                         let loc = self.mem.alloc(Object::Cons(eval_car, SExpr::Nil));
//                         match tail {
//                             Some(tail_loc) => self.mem.set_cdr(tail_loc, SExpr::Ref(loc)),
//                             None => head = SExpr::Ref(loc),
//                         }
//                         tail = Some(loc);
//                         operands = cdr;
//                     }
//                     _ => return Err("Ill formed list"),
//                 }
//                 SExpr::Nil => break,
//                 _ => return Err("Ill formed list"),
//             }
//         }
//         Ok(head)
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::scanner::Scanner;
//     use crate::parser::Parser;

//     // #[test]
//     // fn test_values() {
//     //     let scanner = Scanner::new("1");
//     //     let tokens = scanner.scan_tokens().ok().unwrap();
//     //     let mut mem = Memory::new(100);
//     //     let parser = Parser::new(tokens, &mut mem);
//     //     let expr = parser.parse().ok().unwrap();
//     //     let mut interpreter = Interpreter::new(mem);
//     //     let environment = Environment::new(None);
//     //     let res = interpreter.eval(expr, environment);
//     //     assert!(res.is_ok());
//     //     assert_eq!(res.ok().unwrap(), SExpr::Int(1));
//     // }
// }
