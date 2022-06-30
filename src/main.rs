#[macro_use] extern crate lalrpop_util;

mod ast;
mod grammar;
mod interpreter;

fn main() -> anyhow::Result<()> {
    let program = "\
// a
fn main(a, b) {
    22
}
";
    let program = grammar::SourceFileParser::new().parse(program)?;
    println!("{:?}", program);
    println!("{:?}", interpreter::Interpreter.visit_program(&program)?);

    Ok(())
}
