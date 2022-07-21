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
    Ok((i, Module::new(constants)))
}

fn header(i: &[u8]) -> IResult<u16> {
    let (i, _magic) = tag(b"sprachli")(i)?;
    let (i, version) = be_u16(i)?;
    Ok((i, version))
}

fn constants(i: &[u8]) -> IResult<Vec<Constant>> {
    let (mut input, len) = be_u16(i)?;

    let mut constants = Vec::with_capacity(len as usize);
    for _ in 0..len {
        let (i, constant) = constant(input, &constants)?;
        constants.push(constant);
        input = i;
    }

    Ok((input, constants))
}

fn constant<'b>(i: &'b [u8], constants: &[Constant<'b>]) -> IResult<'b, Constant<'b>> {
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
            let (i, constant) = function(i, constants)?;
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

fn function<'b>(i: &'b [u8], constants: &[Constant<'b>]) -> IResult<'b, Function<'b>> {
    let (i, arity) = be_u16(i)?;
    let (i, formal_parameters) = count(
        |i| {
            let (i, index) = be_u8(i)?;
            let constant = &constants[index as usize];
            let name = match *constant {
                Constant::String(name) => name,
                _ => todo!(),
            };
            Ok((i, name))
        },
        arity as usize,
    )(i)?;

    let (i, len) = be_u16(i)?;
    let (i, bytes) = take(len as usize)(i)?;
    let body = InstructionSequence::new(bytes);

    Ok((i, Function::new(formal_parameters, body)))
}
