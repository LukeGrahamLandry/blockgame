use full_moon::ast::{BinOp, UnOp};
use seesea::ast::{CType, FuncSignature};

pub enum LType {
    Any,
    Nil,
    F64,
    Bool,
    String,
    Table,
    LFunction,
    Union(Box<LType>, Box<LType>),
    CStruct(CType),
    CFunction(FuncSignature),
}

pub enum Op {
    Var(LVar),
    Binary(Box<LExpr>, BinOp, Box<LExpr>),
    Unary(Box<LExpr>, UnOp),
    StructGet(Box<LExpr>, usize),
    Call(Box<LExpr>, Vec<LExpr>),
    MethodCall(Box<LExpr>, String, Vec<LExpr>),
    TableGet(Box<LExpr>, Box<LExpr>),
    TableInit(Vec<(LExpr, LExpr)>),
    Literal(Literal),
    FuncDef(Vec<Arg>, Box<LStmt>),
    RestArgs
}

pub struct LExpr {
    op: Op,
    ty: LType
}

pub enum Literal {
    Num(f64),
    Bool(bool),
    String(String),
    Nil
}

pub enum LStmt {
    Local(Vec<LVar>, Vec<LExpr>),
    Block(Vec<LStmt>),
    Assign(Vec<LExpr>, Vec<LExpr>),
    Expr(LExpr),
    If(Vec<(LExpr, Box<LStmt>)>, Option<Box<LStmt>>),
    FuncDef(String, LExpr),
    Return(Vec<LExpr>),
    NumFor {
        v: LVar,
        start: LExpr,
        stop: LExpr,
        body: Box<LStmt>
    },
    MapFor {
        vars: Vec<LVar>,
        vals: Vec<LExpr>,
        body: Box<LStmt>
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LVar(pub usize);

pub struct Arg {
    pub name: LVar,
    pub ty: LType
}

impl Op {
    pub fn any(self) -> LExpr {
        self.of(LType::Any)
    }

    pub fn of(self, ty: LType) -> LExpr {
        LExpr {
            op: self,
            ty,
        }
    }
}

impl LExpr {
    /// I like post-fix syntax better.
    pub fn boxed(self) -> Box<LExpr> {
        Box::new(self)
    }
}
impl LStmt {
    /// I like post-fix syntax better.
    pub fn boxed(self) -> Box<LStmt> {
        Box::new(self)
    }
}
