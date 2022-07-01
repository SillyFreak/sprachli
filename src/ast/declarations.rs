use std::fmt;

use crate::fmt::{FormatterExt, DebugPrefixed};
use super::Block;

/// Declarations are the individual constructs that can go (among other places)
/// directly in a sprachli file. The most typical declarations are [functions](Fn)
/// and [structs](Struct), but there are also others. For example `impl` blocks
/// are a kind of declaration that supplement structs; these blocks don't have
/// their own identity (or visiblity), they just belong to the named struct.
#[derive(Clone, PartialEq, Eq)]
pub enum Declaration {
    Use(Use),
    Fn(Fn),
    Struct(Struct),
    Mixin(Mixin),
    Impl(Impl),
}

impl fmt::Debug for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Use(item) => write!(f, "{item:?}"),
            Self::Fn(item) => write!(f, "{item:?}"),
            Self::Struct(item) => write!(f, "{item:?}"),
            Self::Mixin(item) => write!(f, "{item:?}"),
            Self::Impl(item) => write!(f, "{item:?}"),
        }
    }
}

/// A path is a possibly qualified name for some declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    pub segments: Vec<PathSegment>,
}

/// A path segment is a single part of a path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSegment {
    Root,
    Super,
    Name(String),
}

/// Most constructs have an explicit or implicit visibility that determines
/// whether a construct can be accessed by code in different modules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Public,
}

impl Visibility {
    fn fmt(&self, f: &mut DebugPrefixed<'_, '_>) {
        match self {
            Self::Public => { f.name("pub"); },
            _ => {},
        }
    }
}

/// Use declarations make some named declaration available in the current scope,
/// optionally changing the name under which it's available.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Use {
    pub visibility: Visibility,
    pub path: Path,
    pub name: Option<String>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Fn {
    pub visibility: Visibility,
    pub name: String,
    pub formal_parameters: Vec<String>,
    pub body: Block,
}

impl fmt::Debug for Fn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_prefixed();
        f.name("fn");
        self.visibility.fmt(&mut f);
        f.name(&self.name).names(&self.formal_parameters).item(&self.body).finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Struct {
    pub visibility: Visibility,
    pub name: String,
    pub members: StructMembers,
}

impl fmt::Debug for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_prefixed();
        f.name("struct");
        self.visibility.fmt(&mut f);
        f.name(&self.name);
        self.members.fmt(&mut f);
        f.finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructMembers {
    Empty,
    Positional(Vec<String>),
    Named(Vec<String>),
}

impl StructMembers {
    fn fmt(&self, f: &mut DebugPrefixed<'_, '_>) {
        match self {
            Self::Empty => {},
            Self::Positional(members) => { f.names(members); },
            Self::Named(members) => { f.name("{").names(members).name("}"); },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mixin {
    pub visibility: Visibility,
    pub name: String,
    pub inheritances: Vec<Path>,
    pub methods: Vec<Fn>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Impl {
    pub name: String,
    pub inheritances: Vec<Path>,
    pub methods: Vec<Fn>,
}
