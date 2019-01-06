#![allow(dead_code)]

mod ast;
mod scanner;
mod parser;
mod interpreter;

fn main() {
    let mut a = 1;
    let r1 = &mut a;
    let r2 = &mut a;
    *r1 = 5;
    *r2 = 6;
}