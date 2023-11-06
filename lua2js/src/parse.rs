use std::any::TypeId;
use std::collections::HashMap;
use full_moon::ast::{Ast, BinOp, Block, Call, Expression, Field, FunctionArgs, FunctionBody, FunctionCall, FunctionName, Index, LastStmt, Parameter, Prefix, Stmt, Suffix, UnOp, Value, Var, VarExpression};
use full_moon::ast::types::{TypeInfo, TypeSpecifier};
use full_moon::node::Node;
use full_moon::tokenizer::{Token, TokenReference, TokenType};
use seesea::ast::Module;
use seesea::scanning::Scanner;
use crate::ast::{Arg, LExpr, Literal, LStmt, LType, LVar, Op};

pub struct Parser {
    ctypes: Module,
    next_var: usize,
    locals: Vec<HashMap<String, LVar>>,
    var_names: HashMap<LVar, VarInfo>,
}

pub struct VarInfo {
    pub name: String,
    pub ty: LType
}

impl Parser {
    pub fn new() -> Self {
        let scanner = Scanner::new("", "ffi".parse().unwrap());
        Parser {
            ctypes: scanner.into(),
            next_var: 0,
            locals: vec![HashMap::new()],
            var_names: Default::default(),
        }
    }

    pub fn info(&self, var: LVar) -> &VarInfo {
        self.var_names.get(&var).unwrap()
    }

    pub fn info_mut(&mut self, var: LVar) -> &mut VarInfo {
        self.var_names.get_mut(&var).unwrap()
    }

    pub fn parse(&mut self, ast: &Ast) -> Vec<LStmt> {
        ast.nodes().stmts().map(|s| self.parse_stmt(s)).collect()
    }

    fn parse_func_body(&mut self, func: &FunctionBody) -> LExpr {
        self.push_scope();
        let args: Vec<_> = func.parameters().iter().zip(func.type_specifiers()).map(|(p, t)| {
            let ty = t.map(|t| self.parse_type(t.type_info())).unwrap_or(LType::Any);
            match p {
                Parameter::Ellipse(_) => todo!(),
                Parameter::Name(token) => {
                    let name = self.new_var(token);
                    self.info_mut(name).ty = ty.clone();
                    Arg { name, ty }
                },
                _ => unreachable!()
            }
        }).collect();

        let body = self.parse_block(func.block()).boxed();
        self.pop_scope();

        let ret = func.return_type().map(|t| self.parse_type(t.type_info())).unwrap_or(LType::Any);
        let arg_t = args.iter().map(|arg| arg.ty.clone()).collect();

        Op::FuncDef(args, body).of(LType::LFunction(Box::new(ret), arg_t))
    }

    fn parse_type(&mut self, ty: &TypeInfo) -> LType {
        match ty {
            TypeInfo::Array { .. } => todo!(),
            TypeInfo::Basic(ty) => {
                let s = ty.to_string();
                if s == "string" {
                    LType::String
                } else if s == "number" {
                    LType::F64
                } else {
                    LType::Any
                }
            },
            TypeInfo::String(_) => LType::String,
            TypeInfo::Boolean(_) => LType::Bool,
            TypeInfo::Callback { .. } => todo!(),
            TypeInfo::Generic { .. } => todo!(),
            TypeInfo::GenericPack { .. } => todo!(),
            TypeInfo::Intersection { .. } => todo!(),
            TypeInfo::Module { .. } => todo!(),
            TypeInfo::Optional { .. } => todo!(),
            TypeInfo::Table { .. } => todo!(),
            TypeInfo::Typeof { .. } => todo!(),
            TypeInfo::Tuple { .. } => todo!(),
            TypeInfo::Union { .. } => todo!(),
            TypeInfo::Variadic { .. } => todo!(),
            TypeInfo::VariadicPack { .. } => todo!(),
            _ => unreachable!()
        }
    }

    fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.locals.pop().unwrap();
    }

    pub fn parse_stmt(&mut self, stmt: &Stmt) -> LStmt {
        match stmt {
            Stmt::Assignment(assign) => {
                let vars: Vec<_> = assign.variables().iter().map(|n| self.parse_var(n)).collect();
                let vals: Vec<_> = assign.expressions().iter().map(|n| self.parse_expr(n)).collect();

                LStmt::Assign(vars, vals)
            }
            Stmt::Do(_) => todo!(),
            Stmt::FunctionCall(call) => {
                LStmt::Expr(self.parse_call(call))
            },
            Stmt::FunctionDeclaration(func) => {
                // TODO: this is wrong.
                let var = self.new_var(func.name().tokens().next().unwrap());
                let expr = self.parse_func_body(func.body());
                self.info_mut(var).ty = expr.ty.clone();
                LStmt::FuncDef(var, expr)
            },
            Stmt::GenericFor(block) => {
                let vars = block.names().iter().map(|s| self.new_var(s)).collect();
                let vals = block.expressions().iter().map(|s| self.parse_expr(s)).collect();
                LStmt::MapFor {
                    vars,
                    vals,
                    body: self.parse_block(block.block()).boxed(),
                }
            },
            Stmt::If(iff) => {
                let mut branches = vec![
                    (self.parse_expr(iff.condition()), self.parse_block(iff.block()))
                ];

                if let Some(chain) = iff.else_if() {
                    for choice in chain {
                        branches.push((self.parse_expr(choice.condition()), self.parse_block(choice.block())));
                    }
                }

                let el = iff.else_block().map(|b| self.parse_block(b).boxed());
                LStmt::If(branches, el)
            },
            Stmt::LocalAssignment(assign) => {
                let vars: Vec<_> = assign.names().iter().map(|n| self.new_var(n)).collect();
                let vals: Vec<_> = assign.expressions().iter().map(|n| self.parse_expr(n)).collect();
                if vars.len() == vals.len() {
                    for (var, val) in vars.iter().zip(vals.iter()) {
                        let ty = &mut self.info_mut(*var).ty;
                        *ty = val.ty.clone();
                    }
                }
                LStmt::Local(vars, vals)
            },
            Stmt::LocalFunction(_) => todo!(),
            Stmt::NumericFor(node) => {
                assert!(node.step().is_none()); // TODO: explicit step value defaults to one.
                LStmt::NumFor {
                    v: self.new_var(node.index_variable()),
                    start: self.parse_expr(node.start()),
                    stop: self.parse_expr(node.end()),
                    body: self.parse_block(node.block()).boxed(),
                }
            },
            Stmt::Repeat(_) => todo!(),
            Stmt::While(_) => todo!(),
            Stmt::CompoundAssignment(_) => todo!(),
            Stmt::ExportedTypeDeclaration(_) => todo!(),
            Stmt::TypeDeclaration(_) => todo!(),
            _ => unimplemented!()
        }
    }

    fn parse_call(&mut self, call: &FunctionCall) -> LExpr {
        let mut expr = self.parse_prefix(call.prefix());
        for suf in call.suffixes() {
            expr = self.parse_suffix(expr, suf);
        }

        if let Op::Call(func, args) = &expr.op {
            if let Op::Global(name) = &func.op {
                if name == "type" {
                    assert_eq!(args.len(), 1);
                    expr = Op::TypeOf(args.first().unwrap().clone().boxed()).of(LType::String)
                }
            }
        }

        expr
    }

    fn parse_prefix(&mut self, pre: &Prefix) -> LExpr {
        match pre {
            Prefix::Expression(e) => self.parse_expr(e),
            Prefix::Name(s) => self.parse_var(&Var::Name(s.clone())),
            _ => unreachable!()
        }
    }

    fn resolve_var_name(&self, name: &str) -> Option<LVar> {
        for scope in self.locals.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(*v);
            }
        }
        None
    }

    /// Var can be a direct variable access or an expression accessing fields of a table.
    fn parse_var(&mut self, name: &Var) -> LExpr {
        match name {
            Var::Expression(var) => {
                let mut expr = self.parse_prefix(var.prefix());
                for suf in var.suffixes() {
                    expr = self.parse_suffix(expr, suf);
                }
                expr
            }
            Var::Name(s) => {
                let name = s.to_string().trim().to_string();
                let v = self.resolve_var_name(&name);
                match v {
                    None => Op::Global(name).any(),
                    Some(v) => {
                        let info = self.info(v);
                        Op::Var(v).of(info.ty.clone())
                    },
                }
            },
            _ => unreachable!()
        }
    }

    fn new_var(&mut self, name: &TokenReference) -> LVar {
        let v = LVar(self.next_var);
        self.next_var += 1;
        let name = name.to_string().trim().to_string();
        self.locals.last_mut().unwrap().insert(name.clone(), v);
        self.var_names.insert(v, VarInfo {
            name,
            ty: LType::Any,
        });
        v
    }

    pub fn parse_expr(&mut self, expr: &Expression) -> LExpr {
        match expr {
            Expression::BinaryOperator { lhs, binop, rhs } => {
                Op::Binary(self.parse_expr(lhs).boxed(), binop.clone(), self.parse_expr(rhs).boxed()).any()
            },
            Expression::Parentheses { expression, .. } => self.parse_expr(expression),
            Expression::UnaryOperator { unop, expression } => {
                Op::Unary(self.parse_expr(expression).boxed(), unop.clone()).any()
            },
            Expression::Value { value, .. } => self.parse_value(value),
            _ => unimplemented!()
        }
    }

    pub fn parse_value(&mut self, value: &Value) -> LExpr {
        match value {
            Value::Function((_, func)) => {
                self.parse_func_body(func)
            },
            Value::FunctionCall(call) => self.parse_call(call),
            Value::IfExpression(_) => todo!(),
            Value::InterpolatedString(_) => todo!(),
            Value::TableConstructor(table) => {
                // TODO: this is a bit deranged.
                // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/rest_parameters
                if table.fields().len() == 1 {
                    if let Field::NoKey(val) = table.fields().iter().next().unwrap() {
                        if let Expression::Value { value, .. } = val {
                            if let Value::Symbol(sym) = value.as_ref() {
                                if sym.to_string() == "..." {
                                    return Op::RestArgs.any()
                                }
                            }
                        }
                    }
                }

                let fields = table.fields().iter().map(|field| {
                    match field {
                        Field::ExpressionKey { key, value, .. } => {
                            (self.parse_expr(key), self.parse_expr(value))
                        },
                        Field::NameKey { key, value, .. } => {
                            let key = Op::Literal(Literal::String(key.to_string().trim().to_string())).of(LType::String);
                            (key, self.parse_expr(value))
                        },
                        Field::NoKey(_) => todo!(),
                        _ => unimplemented!()
                    }
                }).collect();

                Op::TableInit(fields).of(LType::Table)
            },
            Value::Number(num) => {
                let val: f64 = num.token().to_string().trim().parse().unwrap_or_else(|e| panic!("Failed to parse '{}' as number with {:?}", num, e));
                Op::Literal(Literal::Num(val)).of(LType::F64)
            },
            Value::ParenthesesExpression(expr) => self.parse_expr(expr),
            Value::String(val) => {
                let mut s = val.to_string().trim().to_string();
                assert!(s.len() > 2);
                s.truncate(s.len() - 1);
                s.remove(0);
                lit_str(s)
            },
            Value::Symbol(sym) => {
                let sym = sym.token().to_string();
                let sym = sym.trim();
                if sym == "true" {
                    Op::Literal(Literal::Bool(true)).of(LType::Bool)
                } else if sym == "false" {
                    Op::Literal(Literal::Bool(false)).of(LType::Bool)
                } else if sym == "nil" {
                    Op::Literal(Literal::Nil).of(LType::Nil)
                } else {
                    panic!("Unknown symbol {}", sym)
                }
            },
            Value::Var(var) => self.parse_var(var),
            _ => unimplemented!()
        }
    }

    pub fn parse_block(&mut self, block: &Block) -> LStmt {
        let mut stmts = vec![];
        for node in block.stmts() {
            stmts.push(self.parse_stmt(node));
        }
        if let Some(stmt) = block.last_stmt() {
            match stmt {
                LastStmt::Break(_) => todo!(),
                LastStmt::Continue(_) => todo!(),
                LastStmt::Return(ret) => {
                    stmts.push(LStmt::Return(ret.returns().iter().map(|e| self.parse_expr(e)).collect()))
                }
                _ => unimplemented!()
            }
        }

        LStmt::Block(stmts)
    }

    fn parse_suffix(&mut self, expr: LExpr, suffix: &Suffix) -> LExpr {
        match suffix {
            Suffix::Call(call) => {
                match call {
                    Call::AnonymousCall(args) => Op::Call(expr.boxed(), self.parse_args(args)).any(),
                    // TODO: this evaluates the receiver expression twice which is wrong. Can't just use the this keyword like js because lua has being a method as a property of the call not the function definition.
                    // TODO: also a trailing comma if method has no extra arguments which is fine but imperfect
                    Call::MethodCall(method) => {
                        Op::MethodCall(expr.boxed(), method.name().to_string(), self.parse_args(method.args())).any()
                    },
                    _ => unimplemented!()
                }
            }
            Suffix::Index(index) => {
                match index {
                    Index::Brackets { expression, .. } => Op::TableGet(expr.boxed(), self.parse_expr(expression).boxed()).any(),
                    Index::Dot { name, .. } => Op::TableGet(expr.boxed(), lit_str(name).boxed()).any(),
                    _ => unimplemented!()
                }
            },
            _ => unimplemented!()
        }
    }

    fn parse_args(&mut self, args: &FunctionArgs) -> Vec<LExpr> {
        match args {
            FunctionArgs::Parentheses { arguments, .. } => {
                arguments.iter().map(|s| self.parse_expr(s)).collect()
            }
            FunctionArgs::String(s) => {
                // I recognise this doesn't matter and you can still run arbitrary code, there's no inner sandbox here and its only me on my own website anyway but still, out of principle.
                assert!(!s.to_string().contains("${"), "XSS this asshole. https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals \n{:?}", s);
                let s = s.to_string();
                let s = s.trim().strip_prefix("[[\n").unwrap().strip_suffix("]]").unwrap();
                vec![lit_str(s)]
            },
            FunctionArgs::TableConstructor(_) => todo!(),
            _ => unimplemented!()
        }
    }
}

fn lit_str(s: impl ToString) -> LExpr {
    Op::Literal(Literal::String(s.to_string())).of(LType::String)
}
