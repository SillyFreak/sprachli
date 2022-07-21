use std::collections::HashMap;
use std::io::{Result, Write};

use super::instructions;
use super::{ConstantType, Number};
use crate::vm::ast_module::value::{Function, RawValue, Value};
use crate::vm::ast_module::{AstModule, ConstantTable};
use crate::vm::instruction::{InlineConstant, Instruction, Offset};

pub fn write_bytecode<W: Write>(w: &mut W, ast: &AstModule) -> Result<()> {
    header(w)?;
    constants(w, &ast.constants)?;
    globals(w, &ast.global_scope)?;

    Ok(())
}

fn header<W: Write>(w: &mut W) -> Result<()> {
    w.write_all(b"sprachli")?;
    w.write_all(&0u16.to_be_bytes())?;
    Ok(())
}

fn constants<W: Write>(w: &mut W, constants: &ConstantTable) -> Result<()> {
    let len = constants.table.len() as u16;
    w.write_all(&len.to_be_bytes())?;
    for value in &constants.table {
        constant(w, value)?;
    }
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
    w.write_all(&[ConstantType::Number.into()])?;
    w.write_all(&len.to_be_bytes())?;
    w.write_all(value.as_bytes())?;
    Ok(())
}

fn string<W: Write>(w: &mut W, value: &str) -> Result<()> {
    let len = value.len() as u16;
    w.write_all(&[ConstantType::String.into()])?;
    w.write_all(&len.to_be_bytes())?;
    w.write_all(value.as_bytes())?;
    Ok(())
}

fn function<W: Write>(w: &mut W, value: &Function) -> Result<()> {
    use instructions::{BinaryOperator, Opcode as Op, UnaryOperator};
    use InlineConstant as Inl;
    use Instruction as In;

    let mut body = Vec::with_capacity(value.body().len());
    let mut instructions = value.body().iter();
    while let Some(ins) = instructions.next() {
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
            }
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
            In::LoadLocal(index) => {
                body.push(Op::LoadLocal.into());
                body.push(index as u8);
            }
            In::LoadNamed(index) => {
                body.push(Op::LoadNamed.into());
                body.push(index as u8);
            }
            In::Call(arity) => {
                body.push(Op::Call.into());
                body.push(arity as u8);
            }
            In::Jump(offset) => {
                let offset = instructions.byte_offset(offset).unwrap();
                let (opcode, offset) = match offset {
                    Offset::Forward(offset) => (Op::JumpForward, offset),
                    Offset::Backward(offset) => (Op::JumpBackward, offset),
                };
                body.push(opcode.into());
                body.push(offset as u8);
            }
            In::JumpIf(offset) => {
                let offset = instructions.byte_offset(offset).unwrap();
                let (opcode, offset) = match offset {
                    Offset::Forward(offset) => (Op::JumpForwardIf, offset),
                    Offset::Backward(offset) => (Op::JumpBackwardIf, offset),
                };
                body.push(opcode.into());
                body.push(offset as u8);
            }
            In::Invalid => {
                // TODO check this in a better way
                assert!(Op::try_from(0).is_err());
                body.push(0);
            }
        }
    }

    let arity = value.arity() as u16;
    let len = body.len() as u16;

    w.write_all(&[ConstantType::Function.into()])?;
    w.write_all(&arity.to_be_bytes())?;
    w.write_all(&len.to_be_bytes())?;
    // TODO jump offsets must be translated from instruction-wise to byte-wise
    w.write_all(&body)?;
    Ok(())
}

fn globals<W: Write>(w: &mut W, globals: &HashMap<usize, usize>) -> Result<()> {
    let len = globals.len() as u16;
    w.write_all(&len.to_be_bytes())?;
    for (key, value) in globals.iter() {
        let (key, value) = (*key as u16, *value as u16);
        w.write_all(&key.to_be_bytes())?;
        w.write_all(&value.to_be_bytes())?;
    }
    Ok(())
}
