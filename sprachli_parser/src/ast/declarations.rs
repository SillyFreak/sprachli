use std::fmt;

use sprachli_fmt::{DebugSexpr, FormatterExt};

use super::FnTrunk;

/// Declarations are the individual constructs that can go (among other places)
/// directly in a sprachli file. The most typical declarations are [functions](Fn)
/// and [structs](Struct), but there are also others. For example `impl` blocks
/// are a kind of declaration that supplement structs; these blocks don't have
/// their own identity (or visiblity), they just belong to the named struct.
#[derive(Clone, PartialEq, Eq)]
pub enum Declaration<'input> {
    Use(Use<'input>),
    Fn(FnDeclaration<'input>),
    Struct(Struct<'input>),
    Mixin(Mixin<'input>),
    Impl(Impl<'input>),
}

impl fmt::Debug for Declaration<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Use(item) => item.fmt(f),
            Self::Fn(item) => item.fmt(f),
            Self::Struct(item) => item.fmt(f),
            Self::Mixin(item) => item.fmt(f),
            Self::Impl(item) => item.fmt(f),
        }
    }
}

/// A path is a possibly qualified name for some declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path<'input> {
    pub segments: Vec<PathSegment<'input>>,
}

/// A path segment is a single part of a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathSegment<'input> {
    Root,
    Super,
    Name(&'input str),
}

/// Most constructs have an explicit or implicit visibility that determines
/// whether a construct can be accessed by code in different modules.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Public,
}

impl Visibility {
    fn fmt(&self, f: &mut DebugSexpr<'_, '_>) {
        match self {
            Self::Public => {
                f.compact_name("pub");
            }
            Self::Private => {}
        }
    }
}

/// Use declarations make some named declaration available in the current scope,
/// optionally changing the name under which it's available.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Use<'input> {
    pub visibility: Visibility,
    pub path: Path<'input>,
    pub name: Option<&'input str>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct FnDeclaration<'input> {
    pub visibility: Visibility,
    pub name: &'input str,
    pub trunk: FnTrunk<'input>,
}

impl<'input> FnDeclaration<'input> {
    pub fn new(visibility: Visibility, name: &'input str, trunk: FnTrunk<'input>) -> Self {
        Self {
            visibility,
            name,
            trunk,
        }
    }
}

impl<'input> From<FnDeclaration<'input>> for Declaration<'input> {
    fn from(value: FnDeclaration<'input>) -> Self {
        Declaration::Fn(value)
    }
}

impl fmt::Debug for FnDeclaration<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_sexpr();
        f.name("fn");
        self.visibility.fmt(&mut f);
        f.compact_name(self.name);
        self.trunk.fmt(&mut f);
        f.finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Struct<'input> {
    pub visibility: Visibility,
    pub name: &'input str,
    pub members: StructMembers<'input>,
}

impl<'input> Struct<'input> {
    pub fn new(visibility: Visibility, name: &'input str, members: StructMembers<'input>) -> Self {
        Self {
            visibility,
            name,
            members,
        }
    }
}

impl<'input> From<Struct<'input>> for Declaration<'input> {
    fn from(value: Struct<'input>) -> Self {
        Declaration::Struct(value)
    }
}

impl fmt::Debug for Struct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_sexpr();
        f.name("struct");
        self.visibility.fmt(&mut f);
        self.members.fmt(&mut f, self.name);
        f.finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructMembers<'input> {
    Empty,
    Positional(Vec<&'input str>),
    Named(Vec<&'input str>),
}

impl StructMembers<'_> {
    fn fmt(&self, f: &mut DebugSexpr<'_, '_>, name: &str) {
        f.compact_name(match self {
            Self::Empty => "empty",
            Self::Positional(_) => "positional",
            Self::Named(_) => "named",
        });
        f.compact_name(name);
        match self {
            Self::Empty => {}
            Self::Positional(fields) => {
                f.compact_names(fields);
            }
            Self::Named(fields) => {
                f.compact_names(fields);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mixin<'input> {
    pub visibility: Visibility,
    pub name: &'input str,
    pub inheritances: Vec<Path<'input>>,
    pub methods: Vec<FnDeclaration<'input>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Impl<'input> {
    pub name: &'input str,
    pub inheritances: Vec<Path<'input>>,
    pub methods: Vec<FnDeclaration<'input>>,
}
