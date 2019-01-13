use std::collections::HashMap;
use std::cell::{RefCell, Cell};

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

    pub fn to_vec(&self) -> Result<Vec<SExpr<'s, 'm>>, ()> {
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

    pub fn from_vec(mem: &'m Memory<'s, 'm>, v: Vec<SExpr<'s, 'm>>) -> Result<Self, ()> {
        let head = match v.get(0) {
            Some(&e) => SExpr::cons(&mem, e, SExpr::Nil),
            None => return Err(()),
        };
        let mut tail = head;
        for &e in &v[1..] {
            let curr = SExpr::cons(&mem, e, SExpr::Nil);
            tail.set_cdr(curr)?;
            tail = curr;
        }
        Ok(head)
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

    pub fn env_get(&self, k: &'s str) -> Result<Self, ()> {
        if let SExpr::Ref(r) = self {
            if let Object::Env(ref env) = *r.borrow() {
                return env.get(k);
            }
        } 
        Err(())
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Object<'s, 'm> {
    Pair(SExpr<'s, 'm>, SExpr<'s, 'm>),
    PrimitiveProcedure(fn(SExpr<'s, 'm>) -> SExpr<'s, 'm>),
    CompoundProcedure(SExpr<'s, 'm>, &'m RefCell<Object<'s, 'm>>),
    Env(Environment<'s, 'm>),
    Empty(Option<usize>),
}


#[derive(PartialEq, Debug, Clone)]
pub struct Environment<'s, 'm> {
    env: HashMap<&'s str, SExpr<'s, 'm>>,
    enclosing: SExpr<'s, 'm>,
}

impl<'s, 'm> Environment<'s, 'm> {
    pub fn new(enclosing: SExpr<'s, 'm>) -> Self {
        Environment {
            env: HashMap::new(),
            enclosing: enclosing,
        }
    }

    pub fn get(&self, k: &'s str) -> Result<SExpr<'s, 'm>, ()> {
        match self.env.get(k) {
            Some(&e) => Ok(e),
            None => self.enclosing.env_get(k),
        }
    }

    pub fn set(&mut self, k: &'s str, v: SExpr<'s, 'm>) {
        self.env.insert(k, v);
    }
}

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

    pub fn alloc(&'m self, o: Object<'s, 'm>) -> SExpr<'s, 'm> {
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

#[cfg(test)]
mod tests {
    use super::*;
    
    fn i<'s, 'm>(i: i64) -> SExpr<'s, 'm> { SExpr::Int(i) }
    fn sy<'s, 'm>(s: &'s str) -> SExpr<'s, 'm> { SExpr::Sym(s) }

    #[test]
    fn test_env() {
        let mem = Memory::new(100);

        let mut env1 = Environment::new(SExpr::Nil);
        env1.set("test", i(1));
        let env1r = SExpr::new_object(&mem, Object::Env(env1));

        assert_eq!(env1r.env_get("test").unwrap(), i(1));

        let env2 = Environment::new(env1r);
        let env2r = SExpr::new_object(&mem, Object::Env(env2));

        assert_eq!(env2r.env_get("test").unwrap(), i(1));

        let mut env3 = Environment::new(env1r);
        env3.set("test", i(2));
        let env3r = SExpr::new_object(&mem, Object::Env(env3));

        assert_eq!(env3r.env_get("test").unwrap(), i(2));   
    }

    #[test]
    fn test_from_vec() {
        {
            let vec = vec![i(1), i(2), i(3)];
            let mem = Memory::new(100);
            let res1 = SExpr::from_vec(&mem, vec).unwrap();
            let res2 = res1.to_vec().unwrap();
            assert_eq!(res2, vec![i(1), i(2), i(3)]);
        }
        {
            let vec = vec![sy("+"), i(1), i(2)];
            let mem = Memory::new(100);
            let res1 = SExpr::from_vec(&mem, vec).unwrap();
            let res2 = res1.to_vec().unwrap();
            assert_eq!(res2, vec![sy("+"), i(1), i(2)]);
        }
    }
}