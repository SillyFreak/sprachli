use std::fs;

use clap::{ArgGroup, Parser};

use sprachli::bytecode::{parser::parse_bytecode, Error as BytecodeError};
use sprachli::compiler::write_bytecode;
use sprachli::compiler::{Error as CompilerError, Module};
use sprachli::parser::parse_source_file;
use sprachli::vm::{Error as RuntimeError, Vm};

/// Sprachli compiler and interpreter
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
enum Args {
    /// Compiles a given source file
    Compile {
        /// Name of the source file to compile
        #[clap(value_parser)]
        file: String,

        /// Name of the bytecode file to output;
        /// defaults to `<file-without-extension>.sprb`
        #[clap(short, long, value_parser)]
        out_file: Option<String>,
    },
    /// Run a given source or bytecode file;
    /// by default, the kind of file is determined by its extension (`.spr` or `.sprb`)
    #[clap(group(ArgGroup::new("out").args(&["output", "out-file", "bytecode"])))]
    #[clap(group(ArgGroup::new("kind").args(&["source", "bytecode"])))]
    Run {
        /// Name of the source or bytecode file to (compile and) run
        #[clap(value_parser)]
        file: String,

        /// Force interpreting the given file as a source file
        #[clap(short, long, action)]
        source: bool,

        /// Force interpreting the given file as a bytecode file
        #[clap(short, long, action)]
        bytecode: bool,

        /// Output the generated bytecode to the given file; implies `--source`
        #[clap(short, long, value_parser)]
        out_file: Option<String>,

        /// Output the generated bytecode to `<file-without-extension>.sprb`; implies `--source`
        #[clap(long = "out", value_parser)]
        output: bool,
    },
}

#[test]
fn verify_args() {
    use clap::CommandFactory;
    Args::command().debug_assert()
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
    use Args::*;

    let args = Args::parse();

    match args {
        Compile { .. } => todo!(),
        Run { file, .. } => {
            let source = fs::read_to_string(file).map_err(CompilerError::from)?;

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
    }
}
