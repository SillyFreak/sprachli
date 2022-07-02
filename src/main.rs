#[macro_use] extern crate lalrpop_util;

mod ast;
mod fmt;
mod grammar;
mod interpreter;

fn main() -> anyhow::Result<()> {
    let source = "\
// a
fn main() {
    22
}
";
    let source = grammar::SourceFileParser::new().parse(source)?;
    println!("{:?}", source);
    println!("{:?}", interpreter::Interpreter.visit_source_file(&source)?);

    // let exprs = [
    //     "-1 * f(2)",
    //     "if a { 1 } else if b { 2 }",
    //     "{ 1; 2 }",
    // ];
    // for &expr in &exprs {
    //     let expr = grammar::ExpressionParser::new().parse(expr)?;
    //     println!("{:?}", expr);
    //     println!("{:#?}", expr);
    // }

    // let sources = [
    //     "pub fn foo(a, b) { 1 }",
    //     "pub struct Foo;",
    //     "pub struct Bar(a);",
    //     "pub struct Baz { a }",
    // ];
    // for &source in &sources {
    //     let source = grammar::SourceFileParser::new().parse(source)?;
    //     println!("{:?}", source);
    //     println!("{:#?}", source);
    // }

    Ok(())
}
