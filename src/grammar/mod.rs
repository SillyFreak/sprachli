lalrpop_mod!(sprachli, "/grammar/sprachli.rs");

use lalrpop_util::ParseError;
use lalrpop_util::lexer::Token;

pub use sprachli::*;

pub type Error<'a> = ParseError<usize, Token<'a>, &'static str>;
pub type Result<'a, T> = std::result::Result<T, Error<'a>>;

#[cfg(test)]
mod tests {
    use std::fmt;

    use super::*;

    use crate::ast;

    struct TestParser<F> {
        parser: F,
    }

    impl<F, T> TestParser<F>
    where
        F: Fn(&str) -> Result<T>,
        T: fmt::Debug,
    {
        pub fn new(parser: F) -> Self {
            Self { parser }
        }

        pub fn parse(&self, s: &str, expected: &str) {
            let actual = (self.parser)(s).unwrap();
            assert_eq!(format!("{actual:?}"), expected);
        }

        pub fn parse_eq(&self, a: &str, b: &str) {
            let a = (self.parser)(a).unwrap();
            let b = (self.parser)(b).unwrap();
            assert_eq!(format!("{a:?}"), format!("{b:?}"));
        }
    
        pub fn parse_err(&self, s: &str) {
            (self.parser)(s).unwrap_err();
        }
    }

    #[test]
    fn test_declaration_parser() {
        let test = TestParser::new(|s|  DeclarationParser::new().parse(s));

        test.parse("fn foo() {}", "(fn foo (block ()))");
        test.parse("struct Foo;", "(struct empty Foo)");
    }

    #[test]
    fn test_fn_parser() {
        let test = TestParser::new(|s|  FnParser::new().parse(s));

        test.parse("fn foo() {}", "(fn foo (block ()))");
        test.parse("pub fn foo() {}", "(fn pub foo (block ()))");
        test.parse("fn foo(a) {}", "(fn foo a (block ()))");
        test.parse("fn foo(a,) {}", "(fn foo a (block ()))");
        test.parse("fn foo(a, b) {}", "(fn foo a b (block ()))");
        test.parse("fn foo(a, b,) {}", "(fn foo a b (block ()))");
        test.parse_err("fn foo(a, 1) {}");
    }

    #[test]
    fn test_struct_parser() {
        let test = TestParser::new(|s|  StructParser::new().parse(s));

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
        let test = TestParser::new(|s|  ExpressionParser::new().parse(s));

        test.parse("22", "22");
        test.parse("a", "a");
        test.parse("(22)", "22");
        test.parse_err("((22)");
        test.parse("{ 22 }", "(block 22)");
        test.parse("if a { b } else { c }", "(if a (block b) else (block c))");
        test.parse("if a { b } else if c { d }", "(if a (block b) if c (block d))");

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

        test.parse_eq("a + b * c", "a + (b * c)");
        test.parse_eq("a * b + c", "(a * b) + c");
    }

    #[test]
    fn test_stmt_parser() {
        let test = TestParser::new(|s|  StatementParser::new().parse(s));

        test.parse("22;", "22");

        test.parse("a;", "a");
        test.parse("b2;", "b2");

        test.parse("(22);", "22");
        test.parse("((22));", "22");

        test.parse("fn foo() {}", "(fn foo (block ()))");

        test.parse_err("22");
    }
}
