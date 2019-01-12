#![allow(dead_code)]

mod sexpr;
mod scanner;
mod parser;
mod interpreter;

fn main() {
    let mut v = vec![5; 5];
    let r = &v[0];
    v.push(5);
    *r;
}