#[macro_use] extern crate lalrpop_util;

mod ast;
mod fmt;
mod grammar;
mod interpreter;

fn main() -> anyhow::Result<()> {
//     let program = "\
// // a
// fn main(a, b) {
//     22
// }
// ";
//     let program = grammar::SourceFileParser::new().parse(program)?;
//     println!("{:?}", program);
//     println!("{:?}", interpreter::Interpreter.visit_program(&program)?);

    let exprs = [
        "-1 * f(2)",
        "if a { 1 }",
    ];
    for &expr in &exprs {
        let expr = grammar::ExpressionParser::new().parse(expr)?;
        println!("{:?}", expr);
        println!("{:#?}", expr);
    }

    Ok(())
}
