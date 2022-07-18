use std::io::{Result, Write};

use crate::vm::Value;
use crate::vm::ast_module::{AstModule, ConstantTable};
use crate::vm::instruction::{Instruction, InlineConstant, Offset};
use crate::vm::value::{Function, RawValue};
use super::{ConstantType, Number};
use super::instructions;

pub fn write_bytecode<W: Write>(w: &mut W, ast: &AstModule) -> Result<()> {
    header(w);
    constants(w, &ast.constants);

    Ok(())
}

fn header<W: Write>(w: &mut W) -> Result<()> {
    w.write(b"sprachli")?;
    w.write(&0u16.to_be_bytes())?;
    Ok(())
}

fn constants<W: Write>(w: &mut W, constants: &ConstantTable) -> Result<()> {
    w.write(&constants.table.len().to_be_bytes())?;
    for value in &constants.table {
        constant(w, value)?;
    }

    w.write(b"sprachli")?;
    Ok(())
}

fn constant<W: Write>(w: &mut W, value: &Value) -> Result<()> {
    use RawValue::*;

    let value = match value {
        Value::Boxed(value) => value.as_ref(),
        _ => todo!(),
    };

    match value {
        Number(value) => number(w, value)?,
        String(value) => string(w, value)?,
        Function(value) => function(w, value)?,
    }

    Ok(())
}

fn number<W: Write>(w: &mut W, value: &Number) -> Result<()> {
    let value = value.to_string();
    let len = value.len() as u16;
    w.write(&[ConstantType::Number.into()])?;
    w.write(&len.to_be_bytes())?;
    w.write(value.as_bytes())?;
    Ok(())
}

fn string<W: Write>(w: &mut W, value: &str) -> Result<()> {
    let len = value.len() as u16;
    w.write(&[ConstantType::String.into()])?;
    w.write(&len.to_be_bytes())?;
    w.write(value.as_bytes())?;
    Ok(())
}

fn function<W: Write>(w: &mut W, value: &Function) -> Result<()> {
    use InlineConstant as Inl;
    use Instruction as In;
    use instructions::{BinaryOperator, Opcode as Op, UnaryOperator};

    let mut body = Vec::with_capacity(value.body().len());
    for ins in value.body() {
        match ins {
            In::Constant(index) => {
                body.push(Op::Constant.into());
                body.push(index as u8);
            }
            In::InlineConstant(value) => {
                let opcode = match value {
                    Inl::Unit => Op::Unit,
                    Inl::Bool(true) => Op::True,
                    Inl::Bool(false) => Op::False,
                };
                body.push(opcode.into());
            },
            In::Pop => {
                body.push(Op::Pop.into());
            }
            In::Unary(op) => {
                body.push(Op::Unary.into());
                body.push(UnaryOperator::from(op).into());
            }
            In::Binary(op) => {
                body.push(Op::Binary.into());
                body.push(BinaryOperator::from(op).into());
            }
            In::Load(index) => {
                body.push(Op::Load.into());
                body.push(index as u8);
            }
            In::Call(arity) => {
                body.push(Op::Call.into());
                body.push(arity as u8);
            },
            In::Jump(offset) => {
                let (opcode, offset) = match offset {
                    Offset::Forward(offset) => (Op::JumpForward, offset),
                    Offset::Backward(offset) => (Op::JumpBackward, offset),
                };
                body.push(opcode.into());
                body.push(offset as u8);
            },
            In::JumpIf(offset) => {
                let (opcode, offset) = match offset {
                    Offset::Forward(offset) => (Op::JumpForwardIf, offset),
                    Offset::Backward(offset) => (Op::JumpBackwardIf, offset),
                };
                body.push(opcode.into());
                body.push(offset as u8);
            },
            In::Invalid => {
                // TODO check this in a better way
                assert!(Op::try_from(0).is_err());
                body.push(0);
            }
        }

    }

    let arity = value.arity() as u16;
    let len = body.len() as u16;

    w.write(&[ConstantType::Function.into()])?;
    w.write(&arity.to_be_bytes())?;
    // TODO parameters
    w.write(&len.to_be_bytes())?;
    w.write(&body)?;
    Ok(())
}
