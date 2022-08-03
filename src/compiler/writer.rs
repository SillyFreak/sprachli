use std::collections::BTreeMap;
use std::io::{Result, Write};

use super::constant::{Constant, Function};
use super::{Module, Struct};
use crate::bytecode::instruction;
use crate::bytecode::{ConstantType, Number, StructType};

pub fn write_bytecode<W: Write>(w: &mut W, module: &Module) -> Result<()> {
    header(w)?;
    constants(w, module.constants())?;
    globals(w, module.globals())?;
    structs(w, module.structs())?;

    Ok(())
}

fn header<W: Write>(w: &mut W) -> Result<()> {
    w.write_all(b"sprachli")?;
    w.write_all(&0u16.to_be_bytes())?;
    Ok(())
}

fn constants<W: Write>(w: &mut W, constants: &[Constant]) -> Result<()> {
    let len = constants.len() as u16;
    w.write_all(&len.to_be_bytes())?;
    for value in constants {
        constant(w, value)?;
    }
    Ok(())
}

fn constant<W: Write>(w: &mut W, value: &Constant) -> Result<()> {
    use Constant::*;

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
    use instruction::InlineConstant as Const;
    use instruction::Instruction as In;
    use instruction::Offset::*;
    use instruction::Opcode as Op;

    let mut body = Vec::with_capacity(value.body().len());
    for ins in value.body() {
        match *ins {
            In::Constant(index) => {
                body.push(Op::Constant.into());
                body.push(index as u8);
            }
            In::InlineConstant(value) => {
                let opcode = match value {
                    Const::Unit => Op::Unit,
                    Const::Bool(true) => Op::True,
                    Const::Bool(false) => Op::False,
                };
                body.push(opcode.into());
            }
            In::Pop => {
                body.push(Op::Pop.into());
            }
            In::Unary(op) => {
                body.push(Op::Unary.into());
                body.push(op.into());
            }
            In::Binary(op) => {
                body.push(Op::Binary.into());
                body.push(op.into());
            }
            In::LoadLocal(index) => {
                body.push(Op::LoadLocal.into());
                body.push(index as u8);
            }
            In::LoadNamed(index) => {
                body.push(Op::LoadNamed.into());
                body.push(index as u8);
            }
            In::StoreLocal(index) => {
                body.push(Op::StoreLocal.into());
                body.push(index as u8);
            }
            In::StoreNamed(index) => {
                body.push(Op::StoreNamed.into());
                body.push(index as u8);
            }
            In::PopScope(depth) => {
                body.push(Op::PopScope.into());
                body.push(depth as u8);
            }
            In::Call(arity) => {
                body.push(Op::Call.into());
                body.push(arity as u8);
            }
            In::Return => {
                body.push(Op::Return.into());
            }
            In::Jump(offset) => {
                let (opcode, offset) = match offset {
                    Forward(offset) => (Op::JumpForward, offset),
                    Backward(offset) => (Op::JumpBackward, offset),
                };
                body.push(opcode.into());
                body.push(offset as u8);
            }
            In::JumpIf(offset) => {
                let (opcode, offset) = match offset {
                    Forward(offset) => (Op::JumpForwardIf, offset),
                    Backward(offset) => (Op::JumpBackwardIf, offset),
                };
                body.push(opcode.into());
                body.push(offset as u8);
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

fn globals<W: Write>(w: &mut W, globals: &BTreeMap<usize, usize>) -> Result<()> {
    let len = globals.len() as u16;
    w.write_all(&len.to_be_bytes())?;
    for (key, value) in globals.iter() {
        let (key, value) = (*key as u16, *value as u16);
        w.write_all(&key.to_be_bytes())?;
        w.write_all(&value.to_be_bytes())?;
    }
    Ok(())
}

fn structs<W: Write>(w: &mut W, structs: &BTreeMap<usize, Struct>) -> Result<()> {
    let len = structs.len() as u16;
    w.write_all(&len.to_be_bytes())?;
    for (name, decl) in structs {
        strucct(w, *name, decl)?;
    }
    Ok(())
}

fn strucct<W: Write>(w: &mut W, name: usize, decl: &Struct) -> Result<()> {
    use Struct::*;

    let name = name as u16;
    w.write_all(&name.to_be_bytes())?;

    match decl {
        Empty => {
            w.write_all(&[StructType::Empty.into()])?;
            Ok(())
        }
        Positional(count) => {
            let count = *count as u16;
            w.write_all(&[StructType::Positional.into()])?;
            w.write_all(&count.to_be_bytes())?;
            Ok(())
        }
        Named(members) => {
            let len = members.len() as u16;
            w.write_all(&[StructType::Named.into()])?;
            w.write_all(&len.to_be_bytes())?;
            for member in members {
                let member = *member as u16;
                w.write_all(&member.to_be_bytes())?;
            }
            Ok(())
        }
    }
}
