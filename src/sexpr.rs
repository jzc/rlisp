use std::collections::HashMap;
// use std::cell::{RefCell, Cell};
// use crate::parser::Parser;
// use crate::scanner::{Scanner, ParseError};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum SExpr<'s> {
    Nil,
    Int(i64),
    Float(f64),
    Str(&'s str),
    Sym(&'s str),
    Ref(usize),
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Primitive {
    Add, Sub, Mul, Div
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
            enclosing,
        }
    }

    pub fn get<'m>(&self, k: &'s str, mem: &'m Memory<'s>) -> Result<SExpr<'s>, ()> {
        match self.env.get(k) {
            Some(&e) => Ok(e),
            None => mem.env_get(k, self.enclosing),
        }
    }

    pub fn set(&mut self, k: &'s str, e: SExpr<'s>) {
        self.env.insert(k, e);
    }
}

#[derive(PartialEq, Debug)]
pub struct Memory<'s>  {
    mem: Vec<Object<'s>>,
    first: usize,
}

impl<'s> Memory<'s>  {
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
        Err(()) // type error
    }

    pub fn list_from_vec(&mut self, vec: Vec<SExpr<'s>>) -> Result<SExpr<'s>, ()> {
        let head = match vec.get(0) {
            Some(&e) => self.cons(e, SExpr::Nil),
            None => return Err(()), // input vec has length 0
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
                    _ => return Err(()), // type error
                },
                SExpr::Nil => break,
                _ => return Err(()), // type error/ill formed list
            }
        }
        Ok(vec)
    }

    pub fn env_get(&self,  k: &'s str, e: SExpr<'s>) -> Result<SExpr<'s>, ()> {
        match e {
            SExpr::Ref(addr) => match &self.mem[addr] {
                &Object::Env(ref env) => env.get(k, self),
                _ => Err(()), // type error
            }
            SExpr::Nil => Err(()), // not found
            _ => Err(()), // type error
        }
    }

    pub fn get(&self, addr: usize) -> &Object<'s> {
        &self.mem[addr]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn i<'s>(i: i64) -> SExpr<'s> { SExpr::Int(i) }
    fn sy<'s>(s: &'s str) -> SExpr<'s> { SExpr::Sym(s) }

    #[test]
    fn test_env() {
        let mut mem = Memory::new(100);

        let mut env1 = Environment::new(SExpr::Nil);
        env1.set("test", i(1));
        let env1r = mem.alloc(Object::Env(env1));

        assert_eq!(mem.env_get("test", env1r).unwrap(), i(1));

        let env2 = Environment::new(env1r);
        let env2r = mem.alloc(Object::Env(env2));

        assert_eq!(mem.env_get("test", env2r).unwrap(), i(1));

        let mut env3 = Environment::new(env1r);
        env3.set("test", i(2));
        let env3r = mem.alloc(Object::Env(env3));

        assert_eq!(mem.env_get("test", env3r).unwrap(), i(2));   
    }
}