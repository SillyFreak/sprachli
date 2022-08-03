use std::collections::HashMap;

use nom::bytes::complete::{tag, take};
use nom::multi::count;
use nom::number::complete::{be_u16, be_u8};
use nom::Finish;

use super::{
    Constant, ConstantType, Error, Function, InstructionSequence, Module, Number, Struct,
    StructType,
};

pub type Input<'a> = &'a [u8];

pub type IResult<'a, O, E = Error> = nom::IResult<Input<'a>, O, E>;

pub fn parse_bytecode(i: &[u8]) -> Result<Module, Error> {
    bytecode(i).finish().map(|(_, bytecode)| bytecode)
}

fn bytecode(i: &[u8]) -> IResult<Module> {
    let (i, _version) = header(i)?;
    let (i, constants) = constants(i)?;
    let (i, globals) = globals(i, &constants)?;
    let (i, structs) = structs(i, &constants)?;
    Ok((i, Module::new(constants, globals, structs)))
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
    let t = ConstantType::try_from(t).map_err(|_| nom::Err::Error(Error::InvalidConstantType))?;

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
        Number::parse_bytes(bytes, 10).ok_or(nom::Err::Error(Error::InvalidNumberConstant))?;
    Ok((i, value))
}

fn string(i: &[u8]) -> IResult<&str> {
    let (i, len) = be_u16(i)?;
    let (i, bytes) = take(len as usize)(i)?;
    let value =
        std::str::from_utf8(bytes).map_err(|_| nom::Err::Error(Error::InvalidStringConstant))?;
    Ok((i, value))
}

fn function(i: &[u8]) -> IResult<Function> {
    let (i, arity) = be_u16(i)?;
    let (i, len) = be_u16(i)?;
    let (i, bytes) = take(len as usize)(i)?;
    let body = InstructionSequence::new(bytes);

    Ok((i, Function::new(arity as usize, body)))
}

fn get_constant<'a, 'b>(
    constants: &'a [Constant<'b>],
    index: usize,
) -> Result<&'a Constant<'b>, Error> {
    constants
        .get(index)
        .ok_or(Error::InvalidConstantRef(index, constants.len()))
}

fn get_string_constant<'b>(constants: &[Constant<'b>], index: usize) -> Result<&'b str, Error> {
    let constant = get_constant(constants, index)?;
    match constant {
        Constant::String(value) => Ok(*value),
        _ => Err(Error::InvalidConstantRefType(index, "string")),
    }
}

fn globals<'b>(i: &'b [u8], constants: &[Constant<'b>]) -> IResult<'b, HashMap<&'b str, usize>> {
    let (i, len) = be_u16(i)?;
    let (i, globals) = count(|i| global(i, constants), len as usize)(i)?;
    Ok((i, HashMap::from_iter(globals)))
}

fn global<'b>(i: &'b [u8], constants: &[Constant<'b>]) -> IResult<'b, (&'b str, usize)> {
    let (i, name) = be_u16(i)?;
    let (i, value) = be_u16(i)?;

    let name = get_string_constant(constants, name as usize).map_err(nom::Err::Error)?;
    Ok((i, (name, value as usize)))
}

fn structs<'b>(
    i: &'b [u8],
    constants: &[Constant<'b>],
) -> IResult<'b, HashMap<&'b str, Struct<'b>>> {
    let (i, len) = be_u16(i)?;
    let (i, structs) = count(|i| strucct(i, constants), len as usize)(i)?;
    Ok((i, HashMap::from_iter(structs)))
}

fn strucct<'b>(i: &'b [u8], constants: &[Constant<'b>]) -> IResult<'b, (&'b str, Struct<'b>)> {
    use StructType::*;

    let (i, name) = be_u16(i)?;
    let name = get_string_constant(constants, name as usize).map_err(nom::Err::Error)?;

    let (i, t) = be_u8(i)?;
    let t = StructType::try_from(t).map_err(|_| nom::Err::Error(Error::InvalidStructType))?;

    match t {
        Empty => Ok((i, (name, Struct::Empty))),
        Positional => {
            let (i, count) = be_u16(i)?;
            Ok((i, (name, Struct::Positional(count as usize))))
        }
        Named => {
            let (i, len) = be_u16(i)?;
            let (i, members) = count(
                |i| {
                    let (i, member) = be_u16(i)?;
                    let member =
                        get_string_constant(constants, member as usize).map_err(nom::Err::Error)?;
                    Ok((i, member))
                },
                len as usize,
            )(i)?;
            Ok((i, (name, Struct::Named(members))))
        }
    }
}
