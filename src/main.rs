#[macro_use] extern crate lalrpop_util;

mod ast;
mod grammar;
mod interpreter;

fn main() -> Result<(), ()> {
    let program = "\
// a
fn main(a, b) {
    22
}
";
    let program = grammar::ProgramParser::new().parse(program).map_err(|_| ())?;
    println!("{:?}", program);
    println!("{:?}", interpreter::Interpreter.visit_program(&program)?);

    Ok(())
}
