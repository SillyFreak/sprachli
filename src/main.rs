#[macro_use] extern crate lalrpop_util;

mod ast;
mod fmt;
mod grammar;
mod interpreter;

use std::env;
use std::fs;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Usage error: {0}")]
    Usage(String),
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let filename = match &args[..] {
        [_, filename] => filename,
        _ => Err(Error::Usage("unexpected number of command line arguments (expected one)".to_string())).unwrap(),
    };

    let source = fs::read_to_string(filename).unwrap();

    let parser = grammar::SourceFileParser::new();
    let ast = parser.parse(&source).unwrap();

    let interpreter = interpreter::Interpreter::new();
    let result = interpreter.visit_source_file(&ast).unwrap();

    println!("{result:?}");

    // let exprs = [
    //     "-1 * f(2)",
    //     "if a { 1 } else if b { 2 }",
    //     "{ 1; 2 }",
    // ];
    // for &expr in &exprs {
    //     let expr = grammar::ExpressionParser::new().parse(expr)?;
    //     println!("{:?}", expr);
    //     println!("{:#?}", expr);
    // }

    // let sources = [
    //     "pub fn foo(a, b) { 1 }",
    //     "pub struct Foo;",
    //     "pub struct Bar(a);",
    //     "pub struct Baz { a }",
    // ];
    // for &source in &sources {
    //     let source = grammar::SourceFileParser::new().parse(source)?;
    //     println!("{:?}", source);
    //     println!("{:#?}", source);
    // }
}
