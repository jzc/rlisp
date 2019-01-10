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
        match self {
            SExpr::Ref(loc) => {
                let mut cell_ref = loc.borrow_mut();
                match *cell_ref {
                    Object::Cons(_, ref mut cdr) => {
                        *cdr = value;
                        Ok(())
                    }
                    _ => Err(())
                }
            }
            _ => Err(())
        }
    }
    // pub fn get_env_then<T>(&self, fun: fn(&Environment<'a, 'b>)->T) -> Option<T> {
    //     match self {
    //         SExpr::Ref(loc) => {
    //             let cell_ref = loc.borrow();
    //             match *cell_ref {
    //                 Object::Env(ref env) => Some(fun(env)),
    //                 _ => None,
    //             }
    //         }
    //         _ => None,
    //     }
    // }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Object<'s, 'm> {
    Cons(SExpr<'s, 'm>, SExpr<'s, 'm>),
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

    // pub fn car(&self, loc: Location) -> SExpr {
    //     match &self.mem[loc] {
    //         Object::Cons(ref car, _) => *car,
    //         _ => panic!("Object is not cons"),
    //     }
    // }

    // pub fn cdr(&self, loc: Location) -> SExpr {
    //     match &self.mem[loc] {
    //         Object::Cons(_, ref cdr) => *cdr,
    //         _ => panic!("Object is not cons"),
    //     }
    // }

    // pub fn set_car(&mut self, loc: Location, e: SExpr<'a>) {
    //     match &mut self.mem[loc] {
    //         Object::Cons(ref mut car, _) => *car = e,
    //         _ => panic!("Object is not cons"),
    //     }
    // }

    // pub fn set_cdr(&mut self, loc: Location, e: SExpr<'a>) {
    //     match &mut self.mem[loc] {
    //         Object::Cons(_, ref mut cdr) => *cdr = e,
    //         _ => panic!("Object is not cons"),
    //     }
    // }
}