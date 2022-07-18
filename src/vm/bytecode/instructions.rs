use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::ast;
use crate::vm::{InternalError, Result};
use crate::vm::instruction::{InlineConstant, Instruction, Offset};

#[derive(Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Opcode {
    Constant = 1,
    // InlineConstant
    Unit,
    True,
    False,

    Pop,
    Unary,
    Binary,
    Load,
    Call,
    // Jump & JumpIf
    JumpForward,
    JumpBackward,
    JumpForwardIf,
    JumpBackwardIf,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum BinaryOperator {
    // equality
    Equals,
    NotEquals,
    // comparison
    Greater,
    GreaterEquals,
    Less,
    LessEquals,
    // additive
    Add,
    Subtract,
    // multiplicative
    Multiply,
    Divide,
}

impl From<BinaryOperator> for ast::BinaryOperator {
    fn from(op: BinaryOperator) -> Self {
        use BinaryOperator as Op;
        use ast::BinaryOperator as Ast;

        match op {
            Op::Equals => Ast::Equals,
            Op::NotEquals => Ast::NotEquals,
            Op::Greater => Ast::Greater,
            Op::GreaterEquals => Ast::GreaterEquals,
            Op::Less => Ast::Less,
            Op::LessEquals => Ast::LessEquals,
            Op::Add => Ast::Add,
            Op::Subtract => Ast::Subtract,
            Op::Multiply => Ast::Multiply,
            Op::Divide => Ast::Divide,
        }
    }
}

impl From<ast::BinaryOperator> for BinaryOperator {
    fn from(op: ast::BinaryOperator) -> Self {
        use BinaryOperator as Op;
        use ast::BinaryOperator as Ast;

        match op {
            Ast::Equals => Op::Equals,
            Ast::NotEquals => Op::NotEquals,
            Ast::Greater => Op::Greater,
            Ast::GreaterEquals => Op::GreaterEquals,
            Ast::Less => Op::Less,
            Ast::LessEquals => Op::LessEquals,
            Ast::Add => Op::Add,
            Ast::Subtract => Op::Subtract,
            Ast::Multiply => Op::Multiply,
            Ast::Divide => Op::Divide,
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum UnaryOperator {
    // negation
    Negate,
    // logical inverse
    Not,
}

impl From<UnaryOperator> for ast::UnaryOperator {
    fn from(op: UnaryOperator) -> Self {
        use UnaryOperator as Op;
        use ast::UnaryOperator as Ast;

        match op {
            Op::Negate => Ast::Negate,
            Op::Not => Ast::Not,
        }
    }
}

impl From<ast::UnaryOperator> for UnaryOperator {
    fn from(op: ast::UnaryOperator) -> Self {
        use UnaryOperator as Op;
        use ast::UnaryOperator as Ast;

        match op {
            Ast::Negate => Op::Negate,
            Ast::Not => Op::Not,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstructionSequence<'b>(&'b [u8]);

impl<'b> InstructionSequence<'b> {
    pub fn new(instructions: &'b [u8]) -> Self {
        Self(instructions)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(self)
    }
}

impl<'a> IntoIterator for &'a InstructionSequence<'_> {
    type Item = Result<Instruction>;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, Clone)]
pub struct Iter<'a>(std::slice::Iter<'a, u8>);

impl<'a> Iter<'a> {
    fn new(instructions: &'a InstructionSequence) -> Self {
        Self(instructions.0.iter())
    }

    fn opcode(&mut self) -> Option<Result<Opcode>> {
        self.0.next().copied().map(|opcode| -> Result<Opcode> {
            let opcode = opcode.try_into()
                .map_err(|_| InternalError::InvalidBytecode)?;

            Ok(opcode)
        })
    }

    fn parameter(&mut self) -> Result<u8> {
        let parameter = *self.0.next().ok_or(InternalError::InvalidBytecode)?;
        Ok(parameter)
    }

    pub fn jump(&mut self, offset: Offset) -> Result<()> {
        use InternalError::*;
        use Offset::*;

        match offset {
            Forward(offset) => {
                if offset > 0 {
                    self.0.nth(offset - 1).ok_or(InvalidJump)?;
                }
            }
            Backward(offset) => {
                if offset > 0 {
                    self.0.nth_back(offset - 1).ok_or(InvalidJump)?;
                }
            }
        }

        Ok(())
    }
}

impl Iterator for Iter<'_> {
    type Item = Result<Instruction>;

    fn next(&mut self) -> Option<Self::Item> {
        use Opcode as Op;
        use Instruction as In;
        use InlineConstant as Inl;

        self.opcode().map(|opcode| {
            opcode.and_then(|opcode| {
                let ins = match opcode {
                    Op::Constant => {
                        let constant = self.parameter()?;
                        In::Constant(constant as usize)
                    }
                    Op::Unit => In::InlineConstant(Inl::Unit),
                    Op::True => In::InlineConstant(Inl::Bool(true)),
                    Op::False => In::InlineConstant(Inl::Bool(false)),
                    Op::Pop => In::Pop,
                    Op::Unary => {
                        let op = self.parameter()?;
                        let op: UnaryOperator = op.try_into()
                            .map_err(|_| InternalError::InvalidBytecode)?;
                        In::Unary(op.into())
                    }
                    Op::Binary => {
                        let op = self.parameter()?;
                        let op: BinaryOperator = op.try_into()
                            .map_err(|_| InternalError::InvalidBytecode)?;
                        In::Binary(op.into())
                    }
                    Op::Load => {
                        let constant = self.parameter()?;
                        In::Load(constant as usize)
                    }
                    Op::Call => {
                        let arity = self.parameter()?;
                        In::Call(arity as usize)
                    }
                    Op::JumpForward => {
                        let offset = self.parameter()?;
                        In::Jump(Offset::Forward(offset as usize))
                    }
                    Op::JumpBackward => {
                        let offset = self.parameter()?;
                        In::Jump(Offset::Backward(offset as usize))
                    }
                    Op::JumpForwardIf => {
                        let offset = self.parameter()?;
                        In::JumpIf(Offset::Forward(offset as usize))
                    }
                    Op::JumpBackwardIf => {
                        let offset = self.parameter()?;
                        In::JumpIf(Offset::Backward(offset as usize))
                    }
                };
        
                Ok(ins)
            })
        })
    }
}
