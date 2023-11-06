use std::fmt::format;
use crate::ast::{LExpr, Literal, LStmt, LType, Op};
use crate::parse::Parser;

pub fn to_lua(parse: &Parser, stmts: &[LStmt]) -> String {
    let mut p = Print::new(parse);
    for s in stmts {
        p.out += &*p.print_stmt(s);
    }
    p.out
}

struct Print<'ast> {
    out: String,
    parse: &'ast Parser
}

impl<'ast> Print<'ast> {
    fn new(parse: &'ast Parser) -> Self {
        Print {
            out: "".to_string(),
            parse,
        }
    }

    fn print_stmt(&self, stmt: &LStmt) -> String {
        let mut out = String::new();
        match stmt {
            LStmt::Local(vars, vals) => {
                out += "local ";
                out += &*comma_join(vars.iter().map(|v| {
                    let info = self.parse.info(*v);
                    format!("{}: {}, ", info.name, self.print_type(&info.ty))
                }));
                out += " = ";
                out += &*comma_join(vals.iter().map(|v| self.print_expr(v)));
            }
            LStmt::Block(stmts) => {
                for s in stmts {
                    out += &*self.print_stmt(s);
                }
                out += "end";
            }
            LStmt::Assign(vars, vals) => {
                out += &*comma_join(vars.iter().map(|v| self.print_expr(v)));
                out += " = ";
                out += &*comma_join(vals.iter().map(|v| self.print_expr(v)));
            }
            LStmt::Expr(expr) => out += &*self.print_expr(expr),
            LStmt::If(branches, els) => {
                let mut branches = branches.iter();
                let first = branches.next().unwrap();
                out += &*format!("if {} then \n{}", self.print_expr(&first.0), self.print_stmt(&first.1));
                for (cond, block) in branches {
                    out += &*format!("elseif {} then \n{}", self.print_expr(cond), self.print_stmt(block));
                }

                if let Some(block) = els {
                    out += &*format!("else \n{}", self.print_stmt(block));
                }
            },
            LStmt::FuncDef(var, func) => {
                let info = self.parse.info(*var);
                // TODO: real function def
                out += &*format!("local {} = {}", info.name, self.print_expr(func));
            },
            LStmt::Return(vals) => {
                out += "return ";
                out += &*comma_join(vals.iter().map(|v| self.print_expr(v)));
            },
            LStmt::NumFor { v, start, stop, body } => {
                let info = self.parse.info(*v);
                out += &*format!("for {}={},{} do\n{}", info.name, self.print_expr(start), self.print_expr(stop), self.print_stmt(body));
            },
            LStmt::MapFor { .. } => todo!(),
        }
        out += "\n";
        out
    }

    fn print_expr(&self, expr: &LExpr) -> String {
        match &expr.op {
            Op::Var(v) => self.parse.info(*v).name.clone(),
            Op::Binary(lhs, op, rhs) => {
                format!("{} {} {}", self.print_expr(lhs), op, self.print_expr(rhs))
            },
            Op::Unary(expr, op) => {
                format!("{} {}", op, self.print_expr(expr))
            }
            Op::StructGet(obj, name) => todo!(),
            Op::Call(func, args) => {
                format!("{}({})", self.print_expr(func), comma_join(args.iter().map(|e| self.print_expr(e))))
            },
            Op::MethodCall(obj, name, args) => {
                format!("{}.{}({})", self.print_expr(obj), name, comma_join(args.iter().map(|e| self.print_expr(e))))
            },
            Op::TableGet(obj, name) => {
                format!("{}[{}]", self.print_expr(obj), self.print_expr(name))
            }
            Op::TableInit(fields) => todo!(),
            Op::Literal(literal) => match literal {
                Literal::Num(v) => format!("{}", v),
                Literal::Bool(v) => format!("{}", v),
                Literal::String(v) => format!("\"{}\"", v),
                Literal::Nil => "nil".to_string()
            }
            Op::FuncDef(args, body) => { 
                let args = comma_join(args.iter().map(|arg| {
                    let info = self.parse.info(arg.name);
                    format!("{}: {}", info.name, self.print_type(&arg.ty))
                }));
                format!("function ({})\n{}", args, self.print_stmt(body))
            },
            Op::RestArgs => "{...}".to_string(),
            Op::Global(s) => s.clone(),
            Op::TypeOf(e) => format!("\"{}\"", self.print_type(&e.ty)),
        }
    }

    fn print_type(&self, ty: &LType) -> String {
        match ty {
            LType::Any => "any".to_string(),
            LType::Nil => "nil".to_string(),
            LType::F64 => "number".to_string(),
            LType::Bool => "boolean".to_string(),
            LType::String => "string".to_string(),
            LType::Table => "table".to_string(),
            LType::LFunction(ret, args) => {
                format!("({}) -> {}", comma_join(args.iter().map(|a| self.print_type(a))), self.print_type(ret))
            },
            LType::Union(_, _) => "union".to_string(),
            LType::CStruct(_) => "struct".to_string(),
            LType::CFunction(_) => "cfunction".to_string(),
        }
    }
}

fn comma_join(entries: impl Iterator<Item=impl ToString>) -> String {
    entries.map(|p| p.to_string()).collect::<Vec<String>>().join(", ")
}