use std::collections::HashMap;

use nom::bytes::complete::{tag, take};
use nom::multi::count;
use nom::number::complete::{be_u16, be_u8};
use nom::Finish;

use super::error::ParseError;
use super::{Constant, ConstantType, Function, InstructionSequence, Module, Number};

pub type Input<'a> = &'a [u8];

pub type IResult<'a, O, E = ParseError<Input<'a>>> = nom::IResult<Input<'a>, O, E>;

pub fn parse_bytecode(i: &[u8]) -> Result<Module, ParseError<Input<'_>>> {
    bytecode(i).finish().map(|(_, bytecode)| bytecode)
}

fn bytecode(i: &[u8]) -> IResult<Module> {
    let (i, _version) = header(i)?;
    let (i, constants) = constants(i)?;
    let (i, globals) = globals(i, &constants)?;
    Ok((i, Module::new(constants, globals)))
}

fn header(i: &[u8]) -> IResult<u16> {
    let (i, _magic) = tag(b"sprachli")(i)?;
    let (i, version) = be_u16(i)?;
    Ok((i, version))
}

fn constants(i: &[u8]) -> IResult<Vec<Constant>> {
    let (i, len) = be_u16(i)?;
    let (i, constants) = count(constant, len as usize)(i)?;
    Ok((i, constants))
}

fn constant(i: &[u8]) -> IResult<Constant> {
    use ConstantType::*;

    let (i, t) = be_u8(i)?;
    let t =
        ConstantType::try_from(t).map_err(|_| nom::Err::Error(ParseError::InvalidConstantType))?;

    match t {
        Number => {
            let (i, constant) = number(i)?;
            Ok((i, Constant::Number(constant)))
        }
        String => {
            let (i, constant) = string(i)?;
            Ok((i, Constant::String(constant)))
        }
        Function => {
            let (i, constant) = function(i)?;
            Ok((i, Constant::Function(constant)))
        }
    }
}

fn number(i: &[u8]) -> IResult<Number> {
    let (i, len) = be_u16(i)?;
    let (i, bytes) = take(len as usize)(i)?;
    let value =
        Number::parse_bytes(bytes, 10).ok_or(nom::Err::Error(ParseError::InvalidNumberConstant))?;
    Ok((i, value))
}

fn string(i: &[u8]) -> IResult<&str> {
    let (i, len) = be_u16(i)?;
    let (i, bytes) = take(len as usize)(i)?;
    let value = std::str::from_utf8(bytes)
        .map_err(|_| nom::Err::Error(ParseError::InvalidStringConstant))?;
    Ok((i, value))
}

fn function(i: &[u8]) -> IResult<Function> {
    let (i, arity) = be_u16(i)?;
    let (i, len) = be_u16(i)?;
    let (i, bytes) = take(len as usize)(i)?;
    let body = InstructionSequence::new(bytes);

    Ok((i, Function::new(arity as usize, body)))
}

fn globals<'b>(i: &'b [u8], constants: &[Constant<'b>]) -> IResult<'b, HashMap<&'b str, usize>> {
    let (i, len) = be_u16(i)?;
    let (i, globals) = count(|i| global(i, constants), len as usize)(i)?;
    Ok((i, HashMap::from_iter(globals)))
}

fn global<'b>(i: &'b [u8], constants: &[Constant<'b>]) -> IResult<'b, (&'b str, usize)> {
    let (i, name) = be_u16(i)?;
    let (i, value) = be_u16(i)?;

    let index = name as usize;
    let name = constants
        .get(index)
        .ok_or(nom::Err::Error(ParseError::InvalidConstantRef(
            index,
            constants.len(),
        )))?;
    let name = match name {
        Constant::String(name) => *name,
        _ => {
            let error = ParseError::InvalidConstantRefType(index, "string");
            Err(nom::Err::Error(error))?
        }
    };
    Ok((i, (name, value as usize)))
}
