use std::collections::HashMap;
use std::cell::{RefCell, Cell};

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Token<'a> {
    OpenParen,
    ClosedParen,
    Int(i64),
    Float(f64),
    Str(&'a str),
    Symbol(&'a str),
}

#[derive(Debug)]
pub struct ParseError {
    pub message: &'static str,
    pub line: usize,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum SExpr<'s, 'm> {
    Nil,
    Int(i64),
    Float(f64),
    Str(&'s str),
    Sym(&'s str),
    Ref(&'m RefCell<Object<'s, 'm>>),
}

impl<'s, 'm> SExpr<'s, 'm> {
    pub fn set_cdr(&self, value: SExpr<'s, 'm>) -> Result<(), ()> {
       if let SExpr::Ref(loc) = self {
           if let Object::Pair(_, ref mut cdr) = *loc.borrow_mut() {
               *cdr = value;
               return Ok(())
           }
       } 
       Err(())
    }

    pub fn list_to_vec(&self) -> Result<Vec<SExpr<'s, 'm>>, ()> {
        let mut vec = Vec::new();
        let mut curr = *self;
        loop {
            match curr {
                SExpr::Ref(r) => match *r.borrow() {
                    Object::Pair(car, cdr) => { vec.push(car); curr = cdr; }
                    _ => return Err(()),
                }
                SExpr::Nil => break,
                _ => return Err(())
            }
        }
        Ok(vec)
    }

    pub fn from_vec(v: Vec<SExpr<'s, 'm>>, mem: &'m Memory<'s, 'm>) -> Self {
        let head = mem.alloc(Object::Pair(v[0], SExpr::Nil));
        let mut tail = head;
        for e in v.iter().skip(1) {
            match e {
                SExpr::Ref(r) => unimplemented!(),
                _ => unimplemented!()
            }
        }
        unimplemented!()
    }
    
    
    pub fn new_object(mem: &'m Memory<'s, 'm>, o: Object<'s, 'm>) -> Self {
        match mem.first.get() {
            Some(idx) => {
                let ref_ = &mem.mem[idx];
                {
                    let cell_ref = ref_.borrow();
                    match *cell_ref {
                        Object::Empty(next) => mem.first.set(next),
                        _ => panic!("Head of free list is not an empty object"),
                    }
                }
                {
                    let mut cell_ref = ref_.borrow_mut(); 
                    *cell_ref = o;
                }
                SExpr::Ref(ref_)
            }
            None => panic!("Out of memory"),
        }
    }

    pub fn cons(mem: &'m Memory<'s, 'm>, left: Self, right: Self) -> Self {
        SExpr::new_object(mem, Object::Pair(left, right))
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Object<'s, 'm> {
    Pair(SExpr<'s, 'm>, SExpr<'s, 'm>),
    PrimitiveProcedure,
    CompoundProcedure(SExpr<'s, 'm>, &'m RefCell<Object<'s, 'm>>),
    Env(Environment<'s, 'm>),
    Empty(Option<usize>),
}


#[derive(PartialEq, Debug, Clone)]
pub struct Environment<'s, 'm> {
    env: HashMap<&'s str, SExpr<'s, 'm>>,
    enclosing: Option<&'m RefCell<Object<'s, 'm>>>,
}

// impl<'s, 'm>  Environment<'s, 'm>  {
//     pub fn new(enclosing: Option<&'m RefCell<Object<'s, 'm>>>) -> Self {
//         Environment {
//             env: HashMap::new(),
//             enclosing: enclosing,
//         }
//     }

//     // pub fn get(&self, k: &'a str) -> Result<SExpr<'a, 'b>, ()> {
//     //     match self.env.get(k) {
//     //         Some(&e) => Some(e),
//     //         None => match self. {
//     //             Some(enc_loc) => if let Object::Env(enc) = mem.get(enc_loc) {
//     //                 enc.get(k, mem)
//     //             } else {
//     //                 panic!()
//     //             }
//     //             None => None,
//     //         }
//     //     }
//     //     unimplemented!()
//     // }

//     // pub fn set(&mut self, k: &'a str, v: SExpr<'a>) {
//     //     self.env.insert(k, v);
//     // }
// }

#[derive(PartialEq, Debug)]
pub struct Memory<'s, 'm>  {
    mem: Vec<RefCell<Object<'s, 'm>>>,
    first: Cell<Option<usize>>,
}

impl<'s, 'm>  Memory<'s, 'm>  {
    pub fn new(size: usize) -> Self {
        let obj = Memory { mem: vec![RefCell::new(Object::Empty(None)); size], first: Cell::new(Some(0)) };
        for i in 0..size-1 {
            let mut cell_ref = obj.mem[i].borrow_mut();
            *cell_ref = Object::Empty(Some(i+1));
        }
        obj
    }

    pub fn alloc(&'m self, o: Object<'s, 'm>) -> SExpr<'s, 'm> { //&'m RefCell<Object<'s, 'm>> {
        match self.first.get() {
            Some(idx) => {
                let ref_ = &self.mem[idx];
                {
                    let cell_ref = ref_.borrow();
                    match *cell_ref {
                        Object::Empty(next) => self.first.set(next),
                        _ => panic!("Head of free list is not an empty object"),
                    }
                }
                {
                    let mut cell_ref = ref_.borrow_mut(); 
                    *cell_ref = o;
                }
                SExpr::Ref(ref_)
            }
            None => panic!("Out of memory"),
        }
    }
}