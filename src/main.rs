use std::env;
use std::fs;

use sprachli::compiler::{self, write_bytecode};
use sprachli::parser::parse_source_file;
use sprachli::vm::bytecode::parser::parse_bytecode;
use sprachli::vm::Vm;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Usage error: {0}")]
    Usage(String),
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let filename = match &args[..] {
        [_, filename] => filename,
        _ => {
            let msg = "unexpected number of command line arguments (expected one)";
            Err(Error::Usage(msg.to_string())).unwrap()
        }
    };

    let source = fs::read_to_string(filename).unwrap();

    let ast = parse_source_file(&source).unwrap();
    let module = compiler::Module::new(ast).unwrap();

    let mut bytes = Vec::new();
    write_bytecode(&mut bytes, &module).unwrap();
    println!("{bytes:?}");

    let module = parse_bytecode(&bytes).unwrap();
    println!("{module:#?}");

    let result = Vm::new(module).run().expect("execution error");

    println!("{result:?}");
}
