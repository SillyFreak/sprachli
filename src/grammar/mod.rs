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
    fn test_declaration_parser() {
        fn parse(s: &str) -> Result<ast::Declaration> {
            DeclarationParser::new().parse(s)
        }

        assert!(matches!(parse("fn foo() {}"), Ok(ast::Declaration::Fn(_))));
        assert!(matches!(parse("struct Foo();"), Ok(ast::Declaration::Struct(_))));
    }

    #[test]
    fn test_fn_parser() {
        fn parse(s: &str) -> Result<ast::Fn> {
            FnParser::new().parse(s)
        }

        assert!(matches!(
            parse("fn foo() {}"),
            Ok(ast::Fn { 
                visibility: ast::Visibility::Private,
                name,
                formal_parameters,
                ..
            })
            if name == "foo" && formal_parameters.is_empty()
        ));
        assert!(matches!(
            parse("pub fn foo() {}"),
            Ok(ast::Fn { 
                visibility: ast::Visibility::Public,
                ..
            })
        ));
        assert!(matches!(
            parse("fn foo(a) {}"),
            Ok(ast::Fn { 
                formal_parameters,
                ..
            })
            if formal_parameters == &["a"]
        ));
        assert!(matches!(parse("fn foo(a,) {}"), Ok(ast::Fn { .. })));
        assert!(matches!(parse("fn foo(a, b) {}"), Ok(ast::Fn { .. })));
        assert!(matches!(parse("fn foo(a, b,) {}"), Ok(ast::Fn { .. })));
        assert!(matches!(parse("fn foo(a, 1) {}"), Err(_)));
    }

    #[test]
    fn test_struct_parser() {
        fn parse(s: &str) -> Result<ast::Struct> {
            StructParser::new().parse(s)
        }

        assert!(matches!(
            parse("struct Foo;"),
            Ok(ast::Struct { 
                visibility: ast::Visibility::Private,
                name,
                members: ast::StructMembers::Empty,
            })
            if name == "Foo"
        ));

        assert!(matches!(
            parse("pub struct Foo(a);"),
            Ok(ast::Struct { 
                visibility: ast::Visibility::Public,
                members: ast::StructMembers::Positional(members),
                ..
            })
            if members == &["a"]
        ));
        assert!(matches!(parse("struct Foo(a,);"), Ok(ast::Struct { .. })));
        assert!(matches!(parse("struct Foo(a, b);"), Ok(ast::Struct { .. })));
        assert!(matches!(parse("struct Foo(a, b,);"), Ok(ast::Struct { .. })));
        assert!(matches!(parse("struct Foo(a, 1,);"), Err(_)));

        assert!(matches!(
            parse("pub struct Foo { a }"),
            Ok(ast::Struct { 
                visibility: ast::Visibility::Public,
                members: ast::StructMembers::Named(members),
                ..
            })
            if members == &["a"]
        ));
        assert!(matches!(parse("struct Foo { a, }"), Ok(ast::Struct { .. })));
        assert!(matches!(parse("struct Foo { a, b }"), Ok(ast::Struct { .. })));
        assert!(matches!(parse("struct Foo { a, b, }"), Ok(ast::Struct { .. })));
        assert!(matches!(parse("struct Foo { a, 1 }"), Err(_)));
    }

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
