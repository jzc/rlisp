// use std::collections::HashMap;
// use std::str;
// use crate::ast::{Expr, AsciiString};

// struct Memory {
//     mem: Vec<Expr>
// }

// impl Memory {
//     pub fn new() -> Self {
//         Memory { mem: Vec::new() }
//     }

//     pub fn alloc(&mut self, e: Expr) -> usize {
//         self.mem.push(e);
//         self.mem.len()-1
//     }

//     pub fn get(&self, i: usize) -> Expr { self.mem[i].clone() }
// }

// struct Environment<'a> {
//     env: HashMap<AsciiString, Expr>,
//     enclosing: Option<&'a Environment<'a>>,
// }

// impl<'a> Environment<'a> {
//     pub fn new(enclosing: Option<&'a Environment<'a>>) -> Self {
//         Environment {
//             env: HashMap::new(),
//             enclosing: enclosing,
//         }
//     }

//     pub fn get(&self, k: &AsciiString) -> Option<Expr> {
//         match self.env.get(k) {
//             Some(e) => Some(e.clone()),
//             None => match self.enclosing {
//                 Some(enc) => enc.get(k),
//                 None => None,
//             }
//         }
//     }
// }

// struct Interpreter {
//     environment: HashMap<AsciiString, Expr>,
//     mem: Memory,
// }

// impl Interpreter { 
//     pub fn new() -> Self {
//         Interpreter { environment: HashMap::new(), mem: Memory::new(), }
//     }

//     pub fn eval<'a>(&mut self, e: Expr, env: Environment<'a>) -> Result<Expr, &'static str> {
//         match e {
//             v @ Expr::Int(_)  => Ok(v),
//             v @ Expr::Float(_)  => Ok(v),
//             v @ Expr::Str(_)  => Ok(v),
//             v @ Expr::Nil  => Ok(v),
//             v @ Expr::Ref(_) => Ok(v),
//             Expr::Symbol(s) => match env.get(&s) {
//                 Some(e) => Ok(e),
//                 None => Err("Unbound variable"),
//             }
//             Expr::List(exprs) => if let Some(Expr::Symbol(s)) = exprs.get(0) { 
//                 match s.as_slice() {
//                     b"define" => unimplemented!(),
//                     b"lambda" => unimplemented!(),
//                     b"set!" => unimplemented!(),
//                     b"if" => unimplemented!(),

//                     // primitive functions
//                     b"+" => unimplemented!(),
//                     b"-" => unimplemented!(),
//                     b"*" => unimplemented!(),
//                     b"/" => unimplemented!(),
//                     b"cons" => unimplemented!(),
//                     b"car" => unimplemented!(),
//                     b"cdr" => unimplemented!(),

//                     s => unimplemented!(),
//                 }
//             } else { Err("expected symbol") }
//         }
//     }
// }
