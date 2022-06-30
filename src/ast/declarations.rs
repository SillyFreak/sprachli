use super::Block;

/// Declarations are the individual constructs that can go (among other places)
/// directly in a sprachli file. The most typical declarations are [functions](Fn)
/// and [structs](Struct), but there are also others. For example `impl` blocks
/// are a kind of declaration that supplement structs; these blocks don't have
/// their own identity (or visiblity), they just belong to the named struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declaration {
    Use(Use),
    Fn(Fn),
    Struct(Struct),
    Mixin(Mixin),
    Impl(Impl),
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

/// Use declarations make some named declaration available in the current scope,
/// optionally changing the name under which it's available.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Use {
    pub visibility: Visibility,
    pub path: Path,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fn {
    pub visibility: Visibility,
    pub name: String,
    pub formal_parameters: Vec<String>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct {
    pub visibility: Visibility,
    pub name: String,
    pub members: StructMembers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructMembers {
    Empty,
    Positional(Vec<String>),
    Named(Vec<String>),
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
