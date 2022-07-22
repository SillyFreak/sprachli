use std::env;
use std::fs;

use sprachli::bytecode::{parser::parse_bytecode, Error as BytecodeError};
use sprachli::compiler::{compile_source_file, Error as CompilerError};
use sprachli::vm::{Vm, Error as RuntimeError};

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

fn main() {
    let result = || -> Result<(), Error> {
        let args: Vec<String> = env::args().collect();
    
        let filename = match &args[..] {
            [_, filename] => filename,
            _ => {
                let msg = "unexpected number of command line arguments (expected one)";
                Err(Error::Usage(msg.to_string()))?
            }
        };
    
        let source = fs::read_to_string(filename).map_err(CompilerError::from)?;
    
        let mut bytes = Vec::new();
        compile_source_file(&mut bytes, &source)?;
        println!("{bytes:?}");
    
        let module = parse_bytecode(&bytes)?;
        println!("{module:#?}");
    
        let result = Vm::new(module).run()?;
    
        println!("{result:?}");
    
        Ok(())
    }();

    if let Err(error) = result {
        println!("{error}");
    }
}
