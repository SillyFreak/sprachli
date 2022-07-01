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
        use ast::Declaration as Decl;

        fn parse(s: &str) -> Result<Decl> {
            DeclarationParser::new().parse(s)
        }

        assert!(matches!(parse("fn foo() {}"), Ok(Decl::Fn(_))));
        assert!(matches!(parse("struct Foo();"), Ok(Decl::Struct(_))));
    }

    #[test]
    fn test_fn_parser() {
        use ast::Fn;
        use ast::Visibility as Vis;

        fn parse(s: &str) -> Result<Fn> {
            FnParser::new().parse(s)
        }

        assert!(matches!(
            parse("fn foo() {}"),
            Ok(Fn { 
                visibility: Vis::Private,
                name,
                formal_parameters,
                ..
            })
            if name == "foo" && formal_parameters.is_empty()
        ));
        assert!(matches!(
            parse("pub fn foo() {}"),
            Ok(Fn { 
                visibility: Vis::Public,
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
        assert!(matches!(parse("fn foo(a,) {}"), Ok(Fn { .. })));
        assert!(matches!(parse("fn foo(a, b) {}"), Ok(Fn { .. })));
        assert!(matches!(parse("fn foo(a, b,) {}"), Ok(Fn { .. })));
        assert!(matches!(parse("fn foo(a, 1) {}"), Err(_)));
    }

    #[test]
    fn test_struct_parser() {
        use ast::Struct;
        use ast::StructMembers as Members;
        use ast::Visibility as Vis;

        fn parse(s: &str) -> Result<ast::Struct> {
            StructParser::new().parse(s)
        }

        assert!(matches!(
            parse("struct Foo;"),
            Ok(Struct { 
                visibility: Vis::Private,
                name,
                members: Members::Empty,
            })
            if name == "Foo"
        ));

        assert!(matches!(
            parse("pub struct Foo(a);"),
            Ok(Struct { 
                visibility: Vis::Public,
                members: Members::Positional(members),
                ..
            })
            if members == &["a"]
        ));
        assert!(matches!(parse("struct Foo(a,);"), Ok(Struct { .. })));
        assert!(matches!(parse("struct Foo(a, b);"), Ok(Struct { .. })));
        assert!(matches!(parse("struct Foo(a, b,);"), Ok(Struct { .. })));
        assert!(matches!(parse("struct Foo(a, 1,);"), Err(_)));

        assert!(matches!(
            parse("pub struct Foo { a }"),
            Ok(Struct { 
                visibility: Vis::Public,
                members: Members::Named(members),
                ..
            })
            if members == &["a"]
        ));
        assert!(matches!(parse("struct Foo { a, }"), Ok(Struct { .. })));
        assert!(matches!(parse("struct Foo { a, b }"), Ok(Struct { .. })));
        assert!(matches!(parse("struct Foo { a, b, }"), Ok(Struct { .. })));
        assert!(matches!(parse("struct Foo { a, 1 }"), Err(_)));
    }

    #[test]
    fn test_expr_parser() {
        use ast::Expression as Ex;

        fn parse(s: &str) -> Result<Ex> {
            ExpressionParser::new().parse(s)
        }

        assert!(matches!(parse("22"), Ok(Ex::Integer(22))));
        assert!(matches!(parse("a"), Ok(Ex::Identifier(id)) if id == "a"));
        assert!(matches!(parse("(22)"), Ok(Ex::Integer(22))));
        assert!(matches!(parse("((22)"), Err(_)));
        assert!(matches!(parse("{ 22 }"), Ok(Ex::Block(_))));
        assert!(matches!(parse("if a { b } else { c }"), Ok(Ex::If(_))));

        assert!(matches!(parse("-1"), Ok(Ex::Unary(_))));
        assert!(matches!(parse("!true"), Ok(Ex::Unary(_))));

        assert!(matches!(parse("1 + 1"), Ok(Ex::Binary(_))));
        assert!(matches!(parse("1 - 1"), Ok(Ex::Binary(_))));
        assert!(matches!(parse("1 * 1"), Ok(Ex::Binary(_))));
        assert!(matches!(parse("1 / 1"), Ok(Ex::Binary(_))));

        assert!(matches!(parse("1 > 1"), Ok(Ex::Binary(_))));
        assert!(matches!(parse("1 >= 1"), Ok(Ex::Binary(_))));
        assert!(matches!(parse("1 < 1"), Ok(Ex::Binary(_))));
        assert!(matches!(parse("1 <= 1"), Ok(Ex::Binary(_))));

        assert!(matches!(parse("1 == 1"), Ok(Ex::Binary(_))));
        assert!(matches!(parse("1 != 1"), Ok(Ex::Binary(_))));

        assert!(matches!(parse("foo()"), Ok(Ex::Call(_))));
        assert!(matches!(parse("foo(1)"), Ok(Ex::Call(_))));
        assert!(matches!(parse("foo(1,)"), Ok(Ex::Call(_))));
        assert!(matches!(parse("foo(1, 2)"), Ok(Ex::Call(_))));
        assert!(matches!(parse("foo(1, 2,)"), Ok(Ex::Call(_))));
    }

    #[test]
    fn test_stmt_parser() {
        use ast::Expression as Ex;
        use ast::Statement as St;

        fn parse(s: &str) -> Result<St> {
            StatementParser::new().parse(s)
        }

        assert!(matches!(parse("22;"), Ok(St::Expression(Ex::Integer(22)))));

        assert!(matches!(parse("a;"), Ok(St::Expression(Ex::Identifier(id))) if id == "a"));
        assert!(matches!(parse("b2;"), Ok(St::Expression(Ex::Identifier(id))) if id == "b2"));

        assert!(matches!(parse("(22);"), Ok(St::Expression(Ex::Integer(22)))));
        assert!(matches!(parse("((22));"), Ok(St::Expression(Ex::Integer(22)))));

        assert!(matches!(parse("fn foo() {}"), Ok(St::Declaration(ast::Declaration::Fn(_)))));

        assert!(matches!(parse("22"), Err(_)));
    }
}
