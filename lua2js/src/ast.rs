use full_moon::ast::{BinOp, UnOp};
use seesea::ast::{CType, FuncSignature};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LType {
    Any,
    Nil,
    F64,
    Bool,
    String,
    Table,
    LFunction(Box<LType>, Vec<LType>),
    Union(Box<LType>, Box<LType>),
    CStruct(CType),
    CFunction(FuncSignature),
}

#[derive(Clone, Debug)]
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
    RestArgs,
    Global(String),
    TypeOf(Box<LExpr>)
}

#[derive(Clone, Debug)]
pub struct LExpr {
    pub op: Op,
    pub ty: LType
}

#[derive(Clone, Debug)]
pub enum Literal {
    Num(f64),
    Bool(bool),
    String(String),
    Nil
}

#[derive(Clone, Debug)]
pub enum LStmt {
    Local(Vec<LVar>, Vec<LExpr>),
    Block(Vec<LStmt>),
    Assign(Vec<LExpr>, Vec<LExpr>),
    Expr(LExpr),
    If(Vec<(LExpr, LStmt)>, Option<Box<LStmt>>),
    FuncDef(LVar, LExpr),
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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct LVar(pub usize);

#[derive(Clone, Debug)]
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

impl LType {
    // Is it ok to say: self = new
    // YES: local a: any = 10;
    // NO: local b: any; local c: number = b;
    pub fn can_assign(&self, new: &LType) -> bool {
        self == &LType::Any || self == new
    }
}
