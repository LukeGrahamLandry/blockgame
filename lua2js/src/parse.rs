use std::collections::HashMap;
use full_moon::ast::{Ast, BinOp, Block, Call, Expression, Field, FunctionArgs, FunctionBody, FunctionCall, FunctionName, Index, LastStmt, Parameter, Prefix, Stmt, Suffix, UnOp, Value, Var};
use full_moon::tokenizer::{Token, TokenReference, TokenType};
use seesea::ast::Module;
use seesea::scanning::Scanner;
use crate::ast::{Arg, LExpr, Literal, LStmt, LType, LVar, Op};

struct State {
    ctypes: Module,
    next_var: usize,
    locals: Vec<HashMap<String, LVar>>,
    var_names: HashMap<LVar, String>
}

impl State {
    pub fn new() -> Self {
        let scanner = Scanner::new("", "ffi".parse().unwrap());
        State {
            ctypes: scanner.into(),
            next_var: 0,
            locals: vec![HashMap::new()],
            var_names: Default::default(),
        }
    }

    pub fn parse(&mut self, ast: &Ast) -> Vec<LStmt> {
        ast.nodes().stmts().map(|s| self.parse_stmt(s)).collect()
    }

    fn parse_func_body(&mut self, func: &FunctionBody) -> LExpr {
        self.push_scope();
        let args = func.parameters().iter().zip(func.type_specifiers()).map(|(p, t)| {
            let ty = LType::Any;
            match p {
                Parameter::Ellipse(_) => todo!(),
                Parameter::Name(token) => Arg { name: self.new_var(token), ty },
                _ => unreachable!()
            }
        }).collect();

        let body = self.parse_block(func.block()).boxed();
        self.pop_scope();

        Op::FuncDef(args, body).of(LType::LFunction)
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
                let vars = assign.variables().iter().map(|n| self.parse_var(n)).collect();
                let vals = assign.expressions().iter().map(|n| self.parse_expr(n)).collect();
                LStmt::Assign(vars, vals)
            }
            Stmt::Do(_) => todo!(),
            Stmt::FunctionCall(call) => {
                LStmt::Expr(self.parse_call(call))
            },
            Stmt::FunctionDeclaration(func) => {
                LStmt::FuncDef(func.name().to_string(), self.parse_func_body(func.body()))
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
                    (self.parse_expr(iff.condition()), self.parse_block(iff.block()).boxed())
                ];

                if let Some(chain) = iff.else_if() {
                    for choice in chain {
                        branches.push((self.parse_expr(choice.condition()), self.parse_block(choice.block()).boxed()));
                    }
                }

                let el = iff.else_block().map(|b| self.parse_block(b).boxed());
                LStmt::If(branches, el)
            },
            Stmt::LocalAssignment(assign) => {
                let vars = assign.names().iter().map(|n| self.new_var(n)).collect();
                let vals = assign.expressions().iter().map(|n| self.parse_expr(n)).collect();
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
        let func = match call.prefix() {
            Prefix::Expression(e) => self.parse_expr(e),
            Prefix::Name(s) => self.parse_var(&Var::Name(s.clone())),
            _ => unreachable!()
        };

        todo!()
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
            Var::Expression(_) => todo!(),
            Var::Name(s) => {
                let v = self.resolve_var_name(&s.to_string()).unwrap();
                Op::Var(v).any()
            },
            _ => unreachable!()
        }
    }

    fn new_var(&mut self, name: &TokenReference) -> LVar {
        let v = LVar(self.next_var);
        self.next_var += 1;
        self.locals.last_mut().unwrap().insert(name.to_string(), v);
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
            Value::String(val) => lit_str(val),
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
