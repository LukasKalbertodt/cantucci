extern crate lalrpop_util;
#[macro_use] extern crate log;
extern crate term_painter;

mod ast;
pub mod lex;
pub mod base;
mod diag;

use std::path::Path;


pub struct Shape {
    name: String,
}

impl Shape {
    pub fn load<P: AsRef<Path>>(file: P) -> Result<Self, ()> {
        unimplemented!()
    }
}
