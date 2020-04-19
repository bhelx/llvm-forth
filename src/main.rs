#![feature(box_patterns)]
#[macro_use]
extern crate lalrpop_util;
use std::io;
pub mod ast;
mod compiler;

use crate::compiler::Compiler;
use inkwell::context::Context;

fn main() -> io::Result<()> {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);
    let code = "
    : add-one 1 + ;
    : zero-test 0= if 100 else -1 then ;
    -1 add-one zero-test
    ";
    // prints the module LLVM Code
    // and executes the code
    let result = compiler.compile_and_run(code);
    // prints 100
    println!("Top of stack = {}", result);

    Ok(())
}
