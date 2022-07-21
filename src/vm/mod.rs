pub mod ast_module;
pub mod bytecode;
mod error;
mod instruction;
mod interpreter;

pub use error::*;
pub use interpreter::Value;

use bytecode::Module;
use interpreter::Interpreter;

#[derive(Debug, Clone)]
pub struct Vm<'b> {
    module: Module<'b>,
}

impl<'b> Vm<'b> {
    pub fn new(module: Module<'b>) -> Self {
        Self { module }
    }

    pub fn run(&self) -> Result<Value<'b>> {
        Interpreter::new(&self.module).main()
    }
}
