use std::collections::HashMap;
use std::cell::{RefCell, Cell};
use crate::parser::Parser;
use crate::scanner::{Scanner, ParseError};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum SExpr<'s> {
    Nil,
    Int(i64),
    Float(f64),
    Str(&'s str),
    Sym(&'s str),
    Ref(usize),
}

// impl<'s> SExpr<'s> {
    // pub fn set_cdr(&self, value: SExpr<'s, 'm>) -> Result<(), ()> {
    //    if let SExpr::Ref(loc) = self {
    //        if let Object::Pair(_, ref mut cdr) = *loc.borrow_mut() {
    //            *cdr = value;
    //            return Ok(())
    //        }
    //    } 
    //    Err(())
    // }

//     pub fn to_vec(&self) -> Result<Vec<SExpr<'s, 'm>>, ()> {
//         let mut vec = Vec::new();
//         let mut curr = *self;
//         loop {
//             match curr {
//                 SExpr::Ref(r) => match *r.borrow() {
//                     Object::Pair(car, cdr) => { vec.push(car); curr = cdr; }
//                     _ => return Err(()),
//                 }
//                 SExpr::Nil => break,
//                 _ => return Err(())
//             }
//         }
//         Ok(vec)
//     }

//     pub fn from_vec(mem: &'m Memory<'s, 'm>, v: Vec<SExpr<'s, 'm>>) -> Result<Self, ()> {
//         let head = match v.get(0) {
//             Some(&e) => SExpr::cons(&mem, e, SExpr::Nil),
//             None => return Err(()),
//         };
//         let mut tail = head;
//         for &e in &v[1..] {
//             let curr = SExpr::cons(&mem, e, SExpr::Nil);
//             tail.set_cdr(curr)?;
//             tail = curr;
//         }
//         Ok(head)
//     }
    
    
//     pub fn new_object(mem: &'m Memory<'s, 'm>, o: Object<'s, 'm>) -> Self {
//         match mem.first.get() {
//             Some(idx) => {
//                 let ref_ = &mem.mem[idx];
//                 {
//                     let cell_ref = ref_.borrow();
//                     match *cell_ref {
//                         Object::Empty(next) => mem.first.set(next),
//                         _ => panic!("Head of free list is not an empty object"),
//                     }
//                 }
//                 {
//                     let mut cell_ref = ref_.borrow_mut(); 
//                     *cell_ref = o;
//                 }
//                 SExpr::Ref(ref_)
//             }
//             None => panic!("Out of memory"),
//         }
//     }

//     pub fn cons(mem: &'m Memory<'s, 'm>, left: Self, right: Self) -> Self {
//         SExpr::new_object(mem, Object::Pair(left, right))
//     }

//     pub fn env_get(&self, k: &'s str) -> Result<Self, ()> {
//         if let SExpr::Ref(r) = self {
//             if let Object::Env(ref env) = *r.borrow() {
//                 return env.get(k);
//             }
//         } 
//         Err(())
//     }
// }

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Primitive {
    Add, Sub
}

#[derive(PartialEq, Debug, Clone)]
pub enum Object<'s> {
    Pair(SExpr<'s>, SExpr<'s>),
    PrimitiveProcedure(Primitive),
    CompoundProcedure(SExpr<'s>),
    Env(Environment<'s>),
    Empty(Option<usize>),
}


#[derive(PartialEq, Debug, Clone)]
pub struct Environment<'s> {
    env: HashMap<&'s str, SExpr<'s>>,
    enclosing: SExpr<'s>,
}

impl<'s> Environment<'s> {
    pub fn new(enclosing: SExpr<'s>) -> Self {
        Environment {
            env: HashMap::new(),
            enclosing: enclosing,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Memory<'s>  {
    mem: Vec<Object<'s>>,
    first: usize,
}

impl<'s>  Memory<'s>  {
    pub fn new(size: usize) -> Self {
        let mut mem = Vec::with_capacity(size);
        for i in 0..size-1 {
            mem.push(Object::Empty(Some(i+1)));
        }
        mem.push(Object::Empty(None));
        Memory { mem, first: 0 }
    }

    pub fn alloc(&mut self, obj: Object<'s>) -> SExpr<'s> {
        match self.mem[self.first] {
            Object::Empty(Some(next)) => {
                self.mem[self.first] = obj;
                let r = self.first;
                self.first = next;
                SExpr::Ref(r)
            }
            Object::Empty(None) => unimplemented!(),
            _ => panic!("Head of free list is not an empty object"),
        }
    }

    pub fn cons(&mut self, left: SExpr<'s>, right: SExpr<'s>) -> SExpr<'s> {
        self.alloc(Object::Pair(left, right))
    }

    pub fn set_cdr(&mut self, pair: SExpr<'s>, value: SExpr<'s>) -> Result<(), ()> {
        if let SExpr::Ref(addr) = pair {
            if let Some(Object::Pair(_, ref mut cdr)) = self.mem.get_mut(addr) {
                *cdr = value;
                return Ok(());
            }
        }
        Err(())
    }

    pub fn list_from_vec(&mut self, vec: Vec<SExpr<'s>>) -> Result<SExpr<'s>, ()> {
        let head = match vec.get(0) {
            Some(&e) => self.cons(e, SExpr::Nil),
            None => return Err(()),
        };
        let mut tail = head;
        for &e in &vec[1..] {
            let curr = self.cons(e, SExpr::Nil);
            self.set_cdr(tail, curr)?;
            tail = curr;
        }
        Ok(head)
    }

    pub fn vec_from_list(&self, list: SExpr<'s>) -> Result<Vec<SExpr<'s>>, ()> {
        let mut vec = Vec::new();
        let mut curr = list;
        loop {
            match curr {
                SExpr::Ref(addr) => match &self.mem[addr] {
                    &Object::Pair(left, right) => { vec.push(left); curr = right; }
                    _ => return Err(()),
                },
                SExpr::Nil => break,
                _ => return Err(()),
            }
        }
        Ok(vec)
    }

    pub fn parse(&mut self, s: &'s str) -> Result<SExpr<'s>, ParseError> {
        let scanner = Scanner::new(s);
        let tokens = scanner.scan_tokens()?;
        let parser = Parser::new(tokens, self);
        parser.parse()
    }

//     pub fn to_vec(&self) -> Result<Vec<SExpr<'s, 'm>>, ()> {
//         let mut vec = Vec::new();
//         let mut curr = *self;
//         loop {
//             match curr {
//                 SExpr::Ref(r) => match *r.borrow() {
//                     Object::Pair(car, cdr) => { vec.push(car); curr = cdr; }
//                     _ => return Err(()),
//                 }
//                 SExpr::Nil => break,
//                 _ => return Err(())
//             }
//         }
//         Ok(vec)
//     }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
    
//     fn i<'s, 'm>(i: i64) -> SExpr<'s, 'm> { SExpr::Int(i) }
//     fn sy<'s, 'm>(s: &'s str) -> SExpr<'s, 'm> { SExpr::Sym(s) }

//     #[test]
//     fn test_env() {
//         let mem = Memory::new(100);

//         let mut env1 = Environment::new(SExpr::Nil);
//         env1.set("test", i(1));
//         let env1r = SExpr::new_object(&mem, Object::Env(env1));

//         assert_eq!(env1r.env_get("test").unwrap(), i(1));

//         let env2 = Environment::new(env1r);
//         let env2r = SExpr::new_object(&mem, Object::Env(env2));

//         assert_eq!(env2r.env_get("test").unwrap(), i(1));

//         let mut env3 = Environment::new(env1r);
//         env3.set("test", i(2));
//         let env3r = SExpr::new_object(&mem, Object::Env(env3));

//         assert_eq!(env3r.env_get("test").unwrap(), i(2));   
//     }

//     #[test]
//     fn test_from_vec() {
//         {
//             let vec = vec![i(1), i(2), i(3)];
//             let mem = Memory::new(100);
//             let res1 = SExpr::from_vec(&mem, vec).unwrap();
//             let res2 = res1.to_vec().unwrap();
//             assert_eq!(res2, vec![i(1), i(2), i(3)]);
//         }
//         {
//             let vec = vec![sy("+"), i(1), i(2)];
//             let mem = Memory::new(100);
//             let res1 = SExpr::from_vec(&mem, vec).unwrap();
//             let res2 = res1.to_vec().unwrap();
//             assert_eq!(res2, vec![sy("+"), i(1), i(2)]);
//         }
//     }
// }