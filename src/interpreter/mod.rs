use crate::ast;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Symbol not known: {0}")]
    UnknownSymbol(String),
    #[error("Unsupported language construct: {0}")]
    Unsupported(&'static str),
}

pub struct Interpreter;

impl Interpreter {
    pub fn visit_program(&self, node: &ast::Program) -> Result<i32, Error> {
        let main = node.declarations.iter().find_map(|decl| {
            if let ast::Declaration::Fn(node) = decl {
                if node.name == "main" {
                    return Some(node)
                }
            }
            None
        });
        let main = main.ok_or(Error::UnknownSymbol("main".to_string()))?;
        self.visit_fn(main)
    }

    pub fn visit_fn(&self, node: &ast::Fn) -> Result<i32, Error> {
        let node = node.body.expr.as_ref().ok_or(Error::Unsupported("function without a final expression"))?;
        self.visit_expr(node)
    }

    pub fn visit_expr(&self, node: &ast::Expr) -> Result<i32, Error> {
        let result = if let ast::Expr::Integer(value) = node {
            Some(value)
        } else {
            None
        };
        let result = *result.ok_or(Error::Unsupported("expression that is not an integer literal"))?;
        Ok(result)
    }
}