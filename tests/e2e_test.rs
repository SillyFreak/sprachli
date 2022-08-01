use bigdecimal::BigDecimal;

use sprachli::bytecode::{parser::parse_bytecode, Error as BytecodeError};
use sprachli::compiler::{compile_source_file, Error as CompilerError};
use sprachli::vm::{Error as RuntimeError, Value, Vm};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Compiler Error: {0}")]
    Compiler(#[from] CompilerError),
    #[error("Bytecode Error: {0}")]
    Bytecode(#[from] BytecodeError),
    #[error("Runtime Error: {0}")]
    Runtime(#[from] RuntimeError),
}

fn run_and_check_result<F>(source: &str, f: F)
where
    F: FnOnce(Result<Value, Error>) -> Result<(), Error>,
{
    (|| {
        let mut bytecode = Vec::new();
        match compile_source_file(&mut bytecode, source) {
            Ok(value) => value,
            Err(e) => return f(Err(e.into())),
        };
        let module = match parse_bytecode(&bytecode) {
            Ok(value) => value,
            Err(e) => return f(Err(e.into())),
        };
        let result = match Vm::new(module).run() {
            Ok(value) => value,
            Err(e) => return f(Err(e.into())),
        };
        f(Ok(result))
    })()
    .unwrap()
}

fn run_and_check_result_42(source: &str) {
    run_and_check_result(source, |actual| {
        assert_eq!(actual?.as_number()?, &BigDecimal::from(42));
        Ok(())
    })
}

#[test]
fn test_assign() {
    run_and_check_result_42(include_str!("programs/assign.spr"))
}

#[test]
fn test_assign_immutable() {
    run_and_check_result(include_str!("programs/assign_immutable.spr"), |actual| {
        assert!(matches!(
            actual,
            Err(Error::Compiler(CompilerError::ImmutableVariable)),
        ));
        Ok(())
    })
}

#[test]
fn test_bool() {
    run_and_check_result_42(include_str!("programs/bool.spr"))
}

#[test]
fn test_break() {
    run_and_check_result_42(include_str!("programs/break.spr"))
}

#[test]
fn test_continue() {
    run_and_check_result_42(include_str!("programs/continue.spr"))
}

#[test]
fn test_escape() {
    run_and_check_result(include_str!("programs/escape.spr"), |actual| {
        assert_eq!(actual?.as_string()?, "a\r\nb\"c");
        Ok(())
    })
}

#[test]
fn test_even() {
    run_and_check_result(include_str!("programs/even.spr"), |actual| {
        assert!(actual?.as_bool()?);
        Ok(())
    })
}

#[test]
fn test_fn_expr() {
    run_and_check_result_42(include_str!("programs/fn_expr.spr"))
}

#[test]
fn test_higher_order() {
    run_and_check_result_42(include_str!("programs/higher_order.spr"))
}

#[test]
fn test_loop() {
    run_and_check_result_42(include_str!("programs/loop.spr"))
}

#[test]
fn test_max() {
    run_and_check_result_42(include_str!("programs/max.spr"))
}

#[test]
fn test_return() {
    run_and_check_result_42(include_str!("programs/return.spr"))
}

#[test]
fn test_statement() {
    run_and_check_result_42(include_str!("programs/statement.spr"))
}

#[test]
fn test_variable() {
    run_and_check_result_42(include_str!("programs/variable.spr"))
}
