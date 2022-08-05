lalrpop_mod!(
    #[allow(clippy::all)]
    pub grammar,
    "/parser/sprachli.rs"
);
mod string_literal;

use lalrpop_util::lexer::Token;
use lalrpop_util::ParseError;

use crate::ast::SourceFile;
use grammar::SourceFileParser;

pub use string_literal::{string_from_literal, ParseStringError};

pub type Error<'a> = ParseError<usize, Token<'a>, &'static str>;
pub type Result<'a, T> = std::result::Result<T, Error<'a>>;

pub fn parse_source_file(source: &str) -> Result<SourceFile> {
    let parser = SourceFileParser::new();
    parser.parse(source)
}

#[cfg(test)]
mod tests {
    use std::fmt;

    use super::grammar::*;
    use super::*;

    trait ParsingFn<'input> {
        type Ast: 'input + fmt::Debug;

        fn call<'a>(&'a self, input: &'input str) -> Result<Self::Ast>
        where
            'input: 'a;
    }

    impl<'input, Ast, F> ParsingFn<'input> for F
    where
        F: Fn(&'input str) -> Result<Ast>,
        Ast: 'input + fmt::Debug,
    {
        type Ast = Ast;

        fn call<'a>(&'a self, input: &'input str) -> Result<Self::Ast>
        where
            'input: 'a,
        {
            self(input)
        }
    }

    struct TestParser<F> {
        parser: F,
    }

    impl<F> TestParser<F>
    where
        F: for<'input> ParsingFn<'input>,
    {
        pub fn new(parser: F) -> Self {
            Self { parser }
        }

        pub fn parse(&self, s: &str, expected: &str) {
            let actual = self.parser.call(s).unwrap();
            assert_eq!(format!("{actual:?}"), expected);
        }

        pub fn parse_eq(&self, a: &str, b: &str) {
            let a = self.parser.call(a).unwrap();
            let b = self.parser.call(b).unwrap();
            assert_eq!(format!("{a:?}"), format!("{b:?}"));
        }

        pub fn parse_err(&self, s: &str) {
            self.parser.call(s).unwrap_err();
        }
    }

    #[test]
    fn test_declaration_parser() {
        fn parse<'input>(input: &'input str) -> Result<crate::ast::Declaration<'input>> {
            DeclarationParser::new().parse(input)
        }

        let test = TestParser::new(parse);

        test.parse("fn foo() {}", "(fn foo (block ()))");
        test.parse("struct Foo;", "(struct empty Foo)");
    }

    #[test]
    fn test_fn_declaration_parser() {
        fn parse<'input>(input: &'input str) -> Result<crate::ast::FnDeclaration<'input>> {
            FnDeclarationParser::new().parse(input)
        }

        let test = TestParser::new(parse);

        test.parse("fn foo() {}", "(fn foo (block ()))");
        test.parse("pub fn foo() {}", "(fn pub foo (block ()))");
        test.parse("fn foo(a) {}", "(fn foo (a) (block ()))");
        test.parse("fn foo(a,) {}", "(fn foo (a) (block ()))");
        test.parse("fn foo(a, b) {}", "(fn foo (a) (b) (block ()))");
        test.parse("fn foo(a, b,) {}", "(fn foo (a) (b) (block ()))");
        test.parse_err("fn foo(a, 1) {}");
    }

    #[test]
    fn test_struct_parser() {
        fn parse<'input>(input: &'input str) -> Result<crate::ast::Struct<'input>> {
            StructParser::new().parse(input)
        }

        let test = TestParser::new(parse);

        test.parse("struct Foo;", "(struct empty Foo)");
        test.parse("pub struct Foo(a);", "(struct pub positional Foo a)");
        test.parse("struct Foo(a,);", "(struct positional Foo a)");
        test.parse("struct Foo(a, b);", "(struct positional Foo a b)");
        test.parse("struct Foo(a, b,);", "(struct positional Foo a b)");
        test.parse_err("struct Foo(a, 1,);");
        test.parse("struct Foo { a }", "(struct named Foo a)");
        test.parse("struct Foo { a, }", "(struct named Foo a)");
        test.parse("struct Foo { a, b }", "(struct named Foo a b)");
        test.parse("struct Foo { a, b, }", "(struct named Foo a b)");
        test.parse_err("struct Foo { a, 1 }");
    }

    #[test]
    fn test_expr_parser() {
        fn parse<'input>(input: &'input str) -> Result<crate::ast::Expression<'input>> {
            ExpressionParser::new().parse(input)
        }

        let test = TestParser::new(parse);

        test.parse("22", "22");
        test.parse("a", "a");
        test.parse("(22)", "22");
        test.parse_err("((22)");
        test.parse("{ 22 }", "(block 22)");
        test.parse("if a { b } else { c }", "(if a (block b) else (block c))");
        test.parse(
            "if a { b } else if c { d }",
            "(if a (block b) if c (block d))",
        );

        test.parse("-1", "(- 1)");
        test.parse("!true", "(! true)");

        test.parse("1 + 1", "(+ 1 1)");
        test.parse("1 - 1", "(- 1 1)");
        test.parse("1 * 1", "(* 1 1)");
        test.parse("1 / 1", "(/ 1 1)");

        test.parse("1 > 1", "(> 1 1)");
        test.parse("1 >= 1", "(>= 1 1)");
        test.parse("1 < 1", "(< 1 1)");
        test.parse("1 <= 1", "(<= 1 1)");

        test.parse("1 == 1", "(== 1 1)");
        test.parse("1 != 1", "(!= 1 1)");

        test.parse("foo()", "(call foo)");
        test.parse("foo(1)", "(call foo 1)");
        test.parse("foo(1,)", "(call foo 1)");
        test.parse("foo(1, 2)", "(call foo 1 2)");
        test.parse("foo(1, 2,)", "(call foo 1 2)");

        test.parse_eq("-f()", "-(f())");
        test.parse_eq("--a", "-(-a)");
        test.parse_eq("-a * b", "(-a) * b");
        test.parse_eq("a * b * c", "(a * b) * c");
        test.parse_eq("a + b * c", "a + (b * c)");
        test.parse_eq("a * b + c", "(a * b) + c");
        test.parse_eq("a + b + c", "(a + b) + c");
        test.parse_eq("a >= b + c", "a >= (b + c)");
        test.parse_eq("a + b >= c", "(a + b) >= c");
        // test.parse_eq("a >= b >= c", "???");
        // test.parse_eq("a == b >= c", "???");
        // test.parse_eq("a >= b == c", "???");
        // test.parse_eq("a == b == c", "???");
    }

    #[test]
    fn test_stmt_parser() {
        fn parse<'input>(input: &'input str) -> Result<crate::ast::Statement<'input>> {
            StatementParser::new().parse(input)
        }

        let test = TestParser::new(parse);

        test.parse("22;", "22");

        test.parse("a;", "a");
        test.parse("b2;", "b2");

        test.parse("(22);", "22");
        test.parse("((22));", "22");

        test.parse("fn foo() {}", "(fn foo (block ()))");

        test.parse_err("22");
    }
}
