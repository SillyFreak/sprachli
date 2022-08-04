use std::collections::BTreeMap;
use std::io::{Result, Write};

use super::constant::{Constant, Function};
use super::{Module, StructType};
use crate::bytecode::instruction;
use crate::bytecode::{ConstantKind, Number, StructTypeKind};

pub fn write_bytecode<W: Write>(w: &mut W, module: &Module) -> Result<()> {
    header(w)?;
    constants(w, module.constants())?;
    globals(w, module.globals())?;
    struct_types(w, module.struct_types())?;

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
    w.write_all(&[ConstantKind::Number.into()])?;
    w.write_all(&len.to_be_bytes())?;
    w.write_all(value.as_bytes())?;
    Ok(())
}

fn string<W: Write>(w: &mut W, value: &str) -> Result<()> {
    let len = value.len() as u16;
    w.write_all(&[ConstantKind::String.into()])?;
    w.write_all(&len.to_be_bytes())?;
    w.write_all(value.as_bytes())?;
    Ok(())
}

fn function<W: Write>(w: &mut W, value: &Function) -> Result<()> {
    use instruction::InlineConstant as Const;
    use instruction::Instruction as In;
    use instruction::Offset::*;
    use instruction::Opcode as Op;

    fn push_opcode(body: &mut Vec<u8>, opcode: Op) {
        body.push(opcode.into());
    }

    fn push_opcode_u8(body: &mut Vec<u8>, opcode: Op, param: u8) {
        body.push(opcode.into());
        body.push(param);
    }

    let mut body = Vec::with_capacity(value.body().len());
    for ins in value.body() {
        match *ins {
            In::Constant(index) => push_opcode_u8(&mut body, Op::Constant, index as u8),
            In::InlineConstant(value) => {
                let opcode = match value {
                    Const::Unit => Op::Unit,
                    Const::Bool(true) => Op::True,
                    Const::Bool(false) => Op::False,
                };
                push_opcode(&mut body, opcode);
            }
            In::Unary(op) => push_opcode_u8(&mut body, Op::Unary, op.into()),
            In::Binary(op) => push_opcode_u8(&mut body, Op::Binary, op.into()),
            In::LoadLocal(index) => push_opcode_u8(&mut body, Op::LoadLocal, index as u8),
            In::StoreLocal(index) => push_opcode_u8(&mut body, Op::StoreLocal, index as u8),
            In::LoadNamed(index) => push_opcode_u8(&mut body, Op::LoadNamed, index as u8),
            In::StoreNamed(index) => push_opcode_u8(&mut body, Op::StoreNamed, index as u8),
            In::LoadPositionalField(index) => {
                push_opcode_u8(&mut body, Op::LoadPositionalField, index as u8)
            }
            In::StorePositionalField(index) => {
                push_opcode_u8(&mut body, Op::StorePositionalField, index as u8)
            }
            In::LoadNamedField(index) => push_opcode_u8(&mut body, Op::LoadNamedField, index as u8),
            In::StoreNamedField(index) => {
                push_opcode_u8(&mut body, Op::StoreNamedField, index as u8)
            }
            In::Pop => push_opcode(&mut body, Op::Pop),
            In::PopScope(depth) => push_opcode_u8(&mut body, Op::PopScope, depth as u8),
            In::Call(arity) => push_opcode_u8(&mut body, Op::Call, arity as u8),
            In::Return => push_opcode(&mut body, Op::Return),
            In::Jump(offset) => {
                let (opcode, offset) = match offset {
                    Forward(offset) => (Op::JumpForward, offset),
                    Backward(offset) => (Op::JumpBackward, offset),
                };
                push_opcode_u8(&mut body, opcode, offset as u8);
            }
            In::JumpIf(offset) => {
                let (opcode, offset) = match offset {
                    Forward(offset) => (Op::JumpForwardIf, offset),
                    Backward(offset) => (Op::JumpBackwardIf, offset),
                };
                push_opcode_u8(&mut body, opcode, offset as u8);
            }
        }
    }

    let arity = value.arity() as u16;
    let len = body.len() as u16;

    w.write_all(&[ConstantKind::Function.into()])?;
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

fn struct_types<W: Write>(w: &mut W, structs: &BTreeMap<usize, StructType>) -> Result<()> {
    let len = structs.len() as u16;
    w.write_all(&len.to_be_bytes())?;
    for (name, decl) in structs {
        struct_type(w, *name, decl)?;
    }
    Ok(())
}

fn struct_type<W: Write>(w: &mut W, name: usize, decl: &StructType) -> Result<()> {
    use StructType::*;

    let name = name as u16;
    w.write_all(&name.to_be_bytes())?;

    match decl {
        Empty => {
            w.write_all(&[StructTypeKind::Empty.into()])?;
            Ok(())
        }
        Positional(count) => {
            let count = *count as u16;
            w.write_all(&[StructTypeKind::Positional.into()])?;
            w.write_all(&count.to_be_bytes())?;
            Ok(())
        }
        Named(members) => {
            let len = members.len() as u16;
            w.write_all(&[StructTypeKind::Named.into()])?;
            w.write_all(&len.to_be_bytes())?;
            for member in members {
                let member = *member as u16;
                w.write_all(&member.to_be_bytes())?;
            }
            Ok(())
        }
    }
}
