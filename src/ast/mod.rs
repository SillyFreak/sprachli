#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declaration {
    Fn(Fn),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fn {
    pub name: String,
    pub formal_parameters: Vec<String>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Declaration(Declaration),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Integer(i32),
    Identifier(String),
    Block(Box<Block>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub expr: Option<Expr>,
}
