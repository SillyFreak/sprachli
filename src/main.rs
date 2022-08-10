use std::fs;

use clap::Parser;

use sprachli::bytecode::{parser::parse_bytecode, Error as BytecodeError};
use sprachli::compiler::write_bytecode;
use sprachli::compiler::{Error as CompilerError, Module};
use sprachli::parser::parse_source_file;
use sprachli::vm::{Error as RuntimeError, Vm};

/// Sprachli compiler and interpreter
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
   /// Name of the source file to compile and run
   #[clap(value_parser)]
   filename: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Usage error: {0}")]
    Usage(String),
    #[error("Compiler Error: {0}")]
    Compiler(#[from] CompilerError),
    #[error("Bytecode Error: {0}")]
    Bytecode(#[from] BytecodeError),
    #[error("Runtime Error: {0}")]
    Runtime(#[from] RuntimeError),
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let filename = &args.filename;

    let source = fs::read_to_string(filename).map_err(CompilerError::from)?;

    let ast = parse_source_file(&source).map_err(CompilerError::from)?;
    println!("{ast:#?}");

    let module = Module::new(ast)?;
    println!("{module:#?}");

    let mut bytes = Vec::new();
    write_bytecode(&mut bytes, &module).map_err(CompilerError::from)?;
    println!("{bytes:?}");

    let module = parse_bytecode(&bytes)?;
    println!("{module:#?}");

    let result = Vm::new(module).run()?;

    println!("{result:?}");

    Ok(())
}
