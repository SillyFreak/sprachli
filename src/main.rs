use std::env;
use std::fs;

use sprachli::grammar::SourceFileParser;
use sprachli::interpreter::Interpreter;

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

    let parser = SourceFileParser::new();
    let ast = parser.parse(&source).unwrap();

    let interpreter = Interpreter::new();
    let result = interpreter.visit_source_file(&ast).unwrap();

    println!("{result:?}");
}
