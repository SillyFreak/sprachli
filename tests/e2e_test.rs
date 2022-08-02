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

mod operators {
    use super::*;

    #[test]
    fn test_not() {
        let source = "fn main() { !false }";
        run_and_check_result(source, |actual| {
            assert!(actual?.as_bool()?);
            Ok(())
        });

        let source = "fn main() { !true }";
        run_and_check_result(source, |actual| {
            assert!(!actual?.as_bool()?);
            Ok(())
        });
    }

    #[test]
    fn test_neg() {
        let source = "fn main() { -(-42) }";
        run_and_check_result_42(source);
    }

    #[test]
    fn test_mul() {
        let source = "fn main() { 3 * 14 }";
        run_and_check_result_42(source);
    }

    #[test]
    fn test_div() {
        let source = "fn main() { 84 / 2 }";
        run_and_check_result_42(source);
    }

    #[test]
    fn test_mod() {
        let source = "fn main() { 242 % 100 }";
        run_and_check_result_42(source);
    }

    #[test]
    fn test_add() {
        let source = "fn main() { 22 + 20 }";
        run_and_check_result_42(source);
    }

    #[test]
    fn test_sub() {
        let source = "fn main() { 62 - 20 }";
        run_and_check_result_42(source);
    }

    #[test]
    fn test_shr() {
        let source = "fn main() { 168 >> 2 }";
        run_and_check_result_42(source);
    }

    #[test]
    fn test_shl() {
        let source = "fn main() { 21 << 1 }";
        run_and_check_result_42(source);
    }

    #[test]
    fn test_bit_and() {
        // 42 = 0b101010
        //      0b111010 = 58
        //      0b101111 = 47
        let source = "fn main() { 58 & 47 }";
        run_and_check_result_42(source);
    }

    #[test]
    fn test_bit_xor() {
        // 42 = 0b101010
        //      0b111101 = 61
        //      0b010111 = 23
        let source = "fn main() { 61 ^ 23 }";
        run_and_check_result_42(source);
    }

    #[test]
    fn test_bit_or() {
        // 42 = 0b101010
        //      0b001010 = 10
        //      0b101000 = 40
        let source = "fn main() { 10 | 40 }";
        run_and_check_result_42(source);
    }

    #[test]
    fn test_eq() {
        let source = "fn main() { 42 == 42 }";
        run_and_check_result(source, |actual| {
            assert!(actual?.as_bool()?);
            Ok(())
        });

        let source = "fn main() { 42 == 69 }";
        run_and_check_result(source, |actual| {
            assert!(!actual?.as_bool()?);
            Ok(())
        });
    }

    #[test]
    fn test_neq() {
        let source = "fn main() { 42 != 42 }";
        run_and_check_result(source, |actual| {
            assert!(!actual?.as_bool()?);
            Ok(())
        });

        let source = "fn main() { 42 != 69 }";
        run_and_check_result(source, |actual| {
            assert!(actual?.as_bool()?);
            Ok(())
        });
    }

    #[test]
    fn test_gt() {
        let source = "fn main() { 42 > 69 }";
        run_and_check_result(source, |actual| {
            assert!(!actual?.as_bool()?);
            Ok(())
        });

        let source = "fn main() { 42 > 42 }";
        run_and_check_result(source, |actual| {
            assert!(!actual?.as_bool()?);
            Ok(())
        });

        let source = "fn main() { 69 > 42 }";
        run_and_check_result(source, |actual| {
            assert!(actual?.as_bool()?);
            Ok(())
        });
    }

    #[test]
    fn test_lt() {
        let source = "fn main() { 42 < 69 }";
        run_and_check_result(source, |actual| {
            assert!(actual?.as_bool()?);
            Ok(())
        });

        let source = "fn main() { 42 < 42 }";
        run_and_check_result(source, |actual| {
            assert!(!actual?.as_bool()?);
            Ok(())
        });

        let source = "fn main() { 69 < 42 }";
        run_and_check_result(source, |actual| {
            assert!(!actual?.as_bool()?);
            Ok(())
        });
    }

    #[test]
    fn test_gte() {
        let source = "fn main() { 42 >= 69 }";
        run_and_check_result(source, |actual| {
            assert!(!actual?.as_bool()?);
            Ok(())
        });

        let source = "fn main() { 42 >= 42 }";
        run_and_check_result(source, |actual| {
            assert!(actual?.as_bool()?);
            Ok(())
        });

        let source = "fn main() { 69 >= 42 }";
        run_and_check_result(source, |actual| {
            assert!(actual?.as_bool()?);
            Ok(())
        });
    }

    #[test]
    fn test_lte() {
        let source = "fn main() { 42 <= 69 }";
        run_and_check_result(source, |actual| {
            assert!(actual?.as_bool()?);
            Ok(())
        });

        let source = "fn main() { 42 <= 42 }";
        run_and_check_result(source, |actual| {
            assert!(actual?.as_bool()?);
            Ok(())
        });

        let source = "fn main() { 69 <= 42 }";
        run_and_check_result(source, |actual| {
            assert!(!actual?.as_bool()?);
            Ok(())
        });
    }
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
