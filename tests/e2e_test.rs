use bigdecimal::BigDecimal;

use sprachli::bytecode::parse_bytecode;
use sprachli::compiler::compile_source_file;
use sprachli::vm::{Value, Vm};

fn run_and_check_result<F>(source: &str, f: F)
where
    F: FnOnce(Value),
{
    let mut bytecode = Vec::new();
    compile_source_file(&mut bytecode, source).expect("compiler error");
    let module = parse_bytecode(&bytecode).expect("bytecode error");
    let result = Vm::new(module).run().expect("runtime error");
    f(result)
}

#[test]
fn test_break() {
    run_and_check_result(include_str!("programs/break.spr"), |actual| {
        let expected = &BigDecimal::from(42);
        assert_eq!(actual.as_number().ok(), Some(expected));
    })
}

#[test]
fn test_escape() {
    run_and_check_result(include_str!("programs/escape.spr"), |actual| {
        let expected = "a\r\nb\"c";
        assert_eq!(actual.as_string().ok(), Some(expected));
    })
}

#[test]
fn test_even() {
    run_and_check_result(include_str!("programs/even.spr"), |actual| {
        let expected = true;
        assert_eq!(actual.as_bool().ok(), Some(expected));
    })
}

#[test]
fn test_loop() {
    run_and_check_result(include_str!("programs/loop.spr"), |actual| {
        let expected = &BigDecimal::from(42);
        assert_eq!(actual.as_number().ok(), Some(expected));
    })
}

#[test]
fn test_max() {
    run_and_check_result(include_str!("programs/max.spr"), |actual| {
        let expected = &BigDecimal::from(42);
        assert_eq!(actual.as_number().ok(), Some(expected));
    })
}

#[test]
fn test_return() {
    run_and_check_result(include_str!("programs/return.spr"), |actual| {
        let expected = &BigDecimal::from(42);
        assert_eq!(actual.as_number().ok(), Some(expected));
    })
}

#[test]
fn test_statement() {
    run_and_check_result(include_str!("programs/statement.spr"), |actual| {
        let expected = &BigDecimal::from(42);
        assert_eq!(actual.as_number().ok(), Some(expected));
    })
}

#[test]
fn test_variable() {
    run_and_check_result(include_str!("programs/variable.spr"), |actual| {
        let expected = &BigDecimal::from(42);
        assert_eq!(actual.as_number().ok(), Some(expected));
    })
}
