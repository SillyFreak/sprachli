use std::fs;
use std::path::{PathBuf, Path};

use clap::{ArgGroup, Parser};

use sprachli::bytecode::{parser::parse_bytecode, Error as BytecodeError};
use sprachli::compiler::{write_bytecode, Error as CompilerError, Module};
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
        file: PathBuf,

        /// Name of the bytecode file to output;
        /// defaults to `<file-without-spr-extension>.sprb`
        #[clap(short, long, value_parser)]
        out_file: Option<PathBuf>,
    },
    /// Run a given source or bytecode file;
    /// by default, the kind of file is determined by its extension (`.spr` or `.sprb`)
    #[clap(group(ArgGroup::new("out").args(&["output", "out-file", "bytecode"])))]
    #[clap(group(ArgGroup::new("kind").args(&["source", "bytecode"])))]
    Run {
        /// Name of the source or bytecode file to (compile and) run
        #[clap(value_parser)]
        file: PathBuf,

        /// Force interpreting the given file as a source file
        #[clap(short, long, action)]
        source: bool,

        /// Force interpreting the given file as a bytecode file
        #[clap(short, long, action)]
        bytecode: bool,

        /// Output the generated bytecode to the given file; implies `--source`
        #[clap(short, long, value_parser)]
        out_file: Option<PathBuf>,

        /// Output the generated bytecode to `<file-without-spr-extension>.sprb`; implies `--source`
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
    #[error("Compiler Error: {0}")]
    Compiler(#[from] CompilerError),
    #[error("Bytecode Error: {0}")]
    Bytecode(#[from] BytecodeError),
    #[error("Runtime Error: {0}")]
    Runtime(#[from] RuntimeError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputKind {
    Source,
    Bytecode,
}

fn derive_out_filename<P: AsRef<Path>>(_path: P) -> Result<PathBuf, CompilerError> {
    todo!();
}

fn derive_input_kind<P: AsRef<Path>>(_path: P) -> Option<InputKind> {
    todo!();
}

fn read_source_from_file<P: AsRef<Path>>(path: P) -> Result<String, CompilerError> {
    let source = fs::read_to_string(path)?;
    Ok(source)
}

fn read_bytecode_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, CompilerError> {
    let source = fs::read(path)?;
    Ok(source)
}

fn compile_source(source: &str) -> Result<Module, CompilerError> {
    let ast = parse_source_file(&source).map_err(CompilerError::from)?;
    println!("{ast:#?}");

    let module = Module::new(ast)?;
    println!("{module:#?}");

    Ok(module)
}

fn write_bytecode_to_file<P: AsRef<Path>>(path: P, module: &Module) -> Result<(), CompilerError> {
    let mut file = fs::File::create(path)?;
    write_bytecode(&mut file, module)?;
    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    use Args::*;
    use InputKind::*;

    let args = Args::parse();

    match args {
        Compile { file, out_file } => {
            let out_file = match out_file {
                Some(out_file) => out_file,
                None => derive_out_filename(&file)?,
            };

            let source = read_source_from_file(&file)?;
            let module = compile_source(&source)?;
            write_bytecode_to_file(&out_file, &module)?;
            Ok(())
        }
        Run {
            file,
            source,
            bytecode,
            out_file,
            output,
        } => {
            let kind = if source || output || out_file.is_some() {
                Source
            } else if bytecode {
                Bytecode
            } else {
                derive_input_kind(&file).unwrap()
            };

            let bytecode = match kind {
                Source => {
                    let out_file = match (out_file, output) {
                        (Some(out_file), _) => Some(out_file),
                        (None, true) => Some(derive_out_filename(&file)?),
                        (None, false) => None,
                    };

                    let source = read_source_from_file(&file)?;
                    let module = compile_source(&source)?;

                    if let Some(out_file) = out_file {
                        write_bytecode_to_file(&out_file, &module)?;
                    }

                    let mut bytecode = Vec::new();
                    write_bytecode(&mut bytecode, &module).map_err(CompilerError::from)?;
                    bytecode
                }
                Bytecode => {
                    read_bytecode_from_file(&file)?
                }
            };

            println!("{bytecode:?}");

            let module = parse_bytecode(&bytecode)?;
            println!("{module:#?}");

            let result = Vm::new(module).run()?;

            println!("{result:?}");

            Ok(())
        }
    }
}
