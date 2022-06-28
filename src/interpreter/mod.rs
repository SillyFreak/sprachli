use crate::ast;

pub struct Interpreter;

impl Interpreter {
    pub fn visit_program(&self, node: &ast::Program) -> Result<i32, ()> {
        let main = node.declarations.iter().find_map(|decl| {
            if let ast::Declaration::Fn(node) = decl {
                if node.name == "main" {
                    return Some(node)
                }
            }
            None
        });
        let main = main.ok_or(())?;
        self.visit_fn(main)
    }

    pub fn visit_fn(&self, node: &ast::Fn) -> Result<i32, ()> {
        let node = node.body.expr.as_ref().ok_or(())?;
        self.visit_expr(node)
    }

    pub fn visit_expr(&self, node: &ast::Expr) -> Result<i32, ()> {
        let result = if let ast::Expr::Integer(value) = node {
            Some(value)
        } else {
            None
        };
        let result = *result.ok_or(())?;
        Ok(result)
    }
}