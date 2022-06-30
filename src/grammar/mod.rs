lalrpop_mod!(sprachli, "/grammar/sprachli.rs");

use lalrpop_util::ParseError;
use lalrpop_util::lexer::Token;

pub use sprachli::*;

pub type Error<'a> = ParseError<usize, Token<'a>, &'static str>;
pub type Result<'a, T> = std::result::Result<T, Error<'a>>;

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ast;

    #[test]
    fn test_expr_parser() {
        fn parse(s: &str) -> Result<ast::Expr> {
            ExprParser::new().parse(s)
        }

        assert!(matches!(parse("22"), Ok(ast::Expr::Integer(22))));

        assert!(matches!(parse("a"), Ok(ast::Expr::Identifier(id)) if id == "a"));
        assert!(matches!(parse("b2"), Ok(ast::Expr::Identifier(id)) if id == "b2"));

        assert!(matches!(parse("(22)"), Ok(ast::Expr::Integer(22))));
        assert!(matches!(parse("((22))"), Ok(ast::Expr::Integer(22))));
        assert!(matches!(parse("((22)"), Err(_)));

        assert!(matches!(parse("{ 22 }"), Ok(ast::Expr::Block(_))));
    }

    #[test]
    fn test_stmt_parser() {
        fn parse(s: &str) -> Result<ast::Stmt> {
            StmtParser::new().parse(s)
        }

        assert!(matches!(parse("22;"), Ok(ast::Stmt::Expr(ast::Expr::Integer(22)))));

        assert!(matches!(parse("a;"), Ok(ast::Stmt::Expr(ast::Expr::Identifier(id))) if id == "a"));
        assert!(matches!(parse("b2;"), Ok(ast::Stmt::Expr(ast::Expr::Identifier(id))) if id == "b2"));

        assert!(matches!(parse("(22);"), Ok(ast::Stmt::Expr(ast::Expr::Integer(22)))));
        assert!(matches!(parse("((22));"), Ok(ast::Stmt::Expr(ast::Expr::Integer(22)))));

        assert!(matches!(parse("fn foo() {}"), Ok(ast::Stmt::Declaration(ast::Declaration::Fn(_)))));

        assert!(matches!(parse("22"), Err(_)));
    }
}
