use std::collections::HashMap;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Token<'a> {
    OpenParen,
    ClosedParen,
    Int(i64),
    Float(f64),
    Str(&'a str),
    Symbol(&'a str),
}

pub struct ParseError {
    pub message: &'static str,
    pub line: usize,
}

type Location = usize;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum SExpr<'a> {
    Nil,
    Int(i64),
    Float(f64),
    Str(&'a str),
    Sym(&'a str),
    Ref(Location),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Object<'a> {
    Cons(SExpr<'a>, SExpr<'a>),
    Procedure(SExpr<'a>, Location),
    PrimitiveProcedure(fn(SExpr<'a>) -> SExpr<'a>),
    Env(Environment<'a>),
    Empty(Option<Location>),
}

#[derive(PartialEq, Debug, Clone)]
pub struct Environment<'a> {
    env: HashMap<&'a str, SExpr<'a>>,
    enclosing: Option<Location>,
}

impl<'a> Environment<'a> {
    pub fn new(enclosing: Option<Location>) -> Self {
        Environment {
            env: HashMap::new(),
            enclosing: enclosing,
        }
    }

    pub fn get<'b>(&self, k: &'a str, mem: &'b Memory<'a>) -> Option<SExpr<'a>> {
        match self.env.get(k) {
            Some(&e) => Some(e),
            None => match self.enclosing {
                Some(enc_loc) => if let Object::Env(enc) = mem.get(enc_loc) {
                    enc.get(k, mem)
                } else {
                    panic!()
                }
                None => None,
            }
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Memory<'a> {
    mem: Vec<Object<'a>>,
    first: Option<Location>,
}

impl<'a> Memory<'a> {
    pub fn new(size: usize) -> Self {
        let mut mem = vec![Object::Empty(None); size];
        for i in 0..size-1 {
            mem[i] = Object::Empty(Some(i+1));
        }
        Memory { mem: mem, first: Some(0) }
    }

    pub fn alloc(&mut self, o: Object<'a>) -> Location {
        match self.first {
            Some(loc) => {
                if let Object::Empty(next) = self.mem[loc] {
                    self.first = next;
                    self.mem[loc] = o;
                    return loc;
                } else {
                    panic!("Head of free list is not a location");
                }
            }
            None => panic!("Out of memory"),
        }
    }

    pub fn get(&self, loc: Location) -> &Object<'a> {
        &self.mem[loc]
    }

    pub fn car(&self, loc: Location) -> SExpr {
        match &self.mem[loc] {
            Object::Cons(ref car, _) => *car,
            _ => panic!("Object is not cons"),
        }
    }

    pub fn cdr(&self, loc: Location) -> SExpr {
        match &self.mem[loc] {
            Object::Cons(_, ref cdr) => *cdr,
            _ => panic!("Object is not cons"),
        }
    }

    pub fn set_car(&mut self, loc: Location, e: SExpr<'a>) {
        match &mut self.mem[loc] {
            Object::Cons(ref mut car, _) => *car = e,
            _ => panic!("Object is not cons"),
        }
    }

    pub fn set_cdr(&mut self, loc: Location, e: SExpr<'a>) {
        match &mut self.mem[loc] {
            Object::Cons(_, ref mut cdr) => *cdr = e,
            _ => panic!("Object is not cons"),
        }
    }
}