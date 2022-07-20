mod ast_module;
pub mod bytecode;
mod error;
mod instruction;
mod interpreter;
mod value;

pub use ast_module::AstModule;
pub use error::*;
pub use value::Value;

use crate::ast;
use interpreter::Interpreter;

#[derive(Debug, Clone)]
pub struct Vm {
    module: AstModule,
}

impl<'input> TryFrom<ast::SourceFile<'input>> for Vm {
    type Error = Error;

    fn try_from(value: ast::SourceFile) -> Result<Self> {
        let module = value.try_into()?;
        Ok(Vm::new(module))
    }
}

impl Vm {
    pub fn new(module: AstModule) -> Self {
        Self { module }
    }

    pub fn run(&self) -> Result<Value> {
        Interpreter::new(&self.module).main()
    }
}
