use std::fmt;

use itertools::Itertools;

use super::{Constant, Function, Instruction, InstructionSequence};

#[derive(Clone)]
pub struct FmtConstant<'a, 'b> {
    constants: &'a [Constant<'b>],
    constant: &'a Constant<'b>,
}

impl<'a, 'b> FmtConstant<'a, 'b>
where
    'a: 'b,
{
    pub fn new(constants: &'a [Constant<'b>], constant: &'a Constant<'b>) -> Self {
        Self { constants, constant }
    }
}

impl fmt::Debug for FmtConstant<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Constant::*;

        let constant = &self.constant;

        match constant {
            Number(value) => fmt::Display::fmt(value, f),
            String(value) => value.fmt(f),
            Function(value) => {
                let value = FmtFunction::new(self.constants, value);
                value.fmt(f)
            }
        }
    }
}

#[derive(Clone)]
pub struct FmtFunction<'a, 'b> {
    constants: &'a [Constant<'b>],
    function: &'a Function<'b>,
}

impl<'a, 'b> FmtFunction<'a, 'b>
where
    'a: 'b,
{
    pub fn new(constants: &'a [Constant<'b>], function: &'a Function<'b>) -> Self {
        Self { constants, function }
    }
}

impl fmt::Debug for FmtFunction<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let function = &self.function;

        f.write_str("fn (")?;
        for i in (0..function.arity).map(Some).intersperse(None) {
            match i {
                Some(i) => write!(f, "_{}", i)?,
                None => f.write_str(", ")?,
            }
        }

        if f.alternate() {
            f.write_str(") {\n")?;
            let body = FmtInstructionSequence::new(self.constants, &function.body);
            body.fmt(f)?;
            f.write_str("\n      }")?;
        } else {
            f.write_str(") { ... }")?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct FmtInstructionSequence<'a, 'b> {
    constants: &'a [Constant<'b>],
    instructions: &'a InstructionSequence<'b>,
}

impl<'a, 'b> FmtInstructionSequence<'a, 'b>
where
    'a: 'b,
{
    pub fn new(constants: &'a [Constant<'b>], instructions: &'a InstructionSequence<'b>) -> Self {
        Self { constants, instructions }
    }
}

impl fmt::Debug for FmtInstructionSequence<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let instructions = &self.instructions;

        if f.alternate() {
            for ins in instructions
                .iter()
                .with_offset()
                .map(Some)
                .intersperse_with(|| None)
            {
                if let Some((offset, ins)) = ins {
                    match ins {
                        Ok(ins) => {
                            let ins = FmtInstruction::new(self.constants, ins);
                            write!(f, "      {offset:5} {ins:?}")?
                        }
                        Err(_error) => write!(f, "      {offset:5} ...")?,
                    }
                } else {
                    f.write_str("\n")?;
                }
            }
            Ok(())
        } else {
            instructions.fmt(f)
        }
    }
}

#[derive(Clone)]
pub struct FmtInstruction<'a, 'b> {
    constants: &'a [Constant<'b>],
    instruction: Instruction,
}

impl<'a, 'b> FmtInstruction<'a, 'b>
where
    'a: 'b,
{
    pub fn new(constants: &'a [Constant<'b>], instruction: Instruction) -> Self {
        Self { constants, instruction }
    }
}

impl fmt::Debug for FmtInstruction<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Instruction::*;

        let instruction = &self.instruction;

        match instruction {
            Constant(index) => {
                write!(f, "CONST #{index:<3} -- ")?;
                match self.constants.get(*index) {
                    Some(constant) => write!(f, "{constant:?}")?,
                    _ => write!(f, "illegal constant")?,
                }
                Ok(())
            }
            InlineConstant(value) => write!(f, "CONST {value:?}"),
            Pop => write!(f, "POP"),
            Unary(op) => write!(f, "UNARY {op:?}"),
            Binary(op) => write!(f, "BINARY {op:?}"),
            LoadLocal(local) => write!(f, "LOAD _{local}"),
            LoadNamed(index) => {
                write!(f, "LOAD #{index:<4} -- ")?;
                match self.constants.get(*index) {
                    Some(super::Constant::String(value)) => write!(f, "{value}")?,
                    Some(constant) => write!(f, "{constant:?} (invalid for LOAD)")?,
                    _ => write!(f, "illegal constant")?,
                }
                Ok(())
            }
            Call(arity) => write!(f, "CALL {arity}"),
            Jump(offset) => write!(f, "JUMP {offset:?}"),
            JumpIf(offset) => write!(f, "JUMP_IF {offset:?}"),
        }
    }
}
