use std::fmt::format;
use full_moon::ast::{Ast, BinOp, Block, Call, Expression, Field, FunctionArgs, FunctionBody, FunctionCall, FunctionName, Index, LastStmt, Prefix, Stmt, Suffix, UnOp, Value, Var};
use full_moon::tokenizer::TokenKind;

pub fn tojs(ast: &Ast) -> String {
    let msg = "// This file is @generated from lua. Do not edit manually!";
    let mut out = format!("{0}\n{1}\n{0}\n", msg, include_str!("runtime.js"));
    for node in ast.nodes().stmts() {
        out += &*stmt2js(node);
    }
    out
}

pub fn block2js(block: &Block) -> String {
    let mut out = "{\n".to_string();
    for node in block.stmts() {
        out += &*stmt2js(node);
    }
    if let Some(stmt) = block.last_stmt() {
        match stmt {
            LastStmt::Break(_) => todo!(),
            LastStmt::Continue(_) => todo!(),
            LastStmt::Return(ret) => {
                match ret.returns().len() {
                    0 => out += "return;\n",
                    1 => {
                        out += &*format!("return {};\n", expr2js(ret.returns().iter().next().unwrap()));
                    }
                    _ => {
                        out.push_str("return [");
                        for val in ret.returns() {
                            out += &*format!("{}, ", expr2js(val));
                        }
                        out.push_str("];\n")
                    }
                }

            }
            _ => unimplemented!()
        }
    }
    out.push_str("}\n");
    out
}

fn stmt2js(stmt: &Stmt) -> String {
    println!("stmt: {:?}", stmt);
    let mut out = String::new();
    match stmt {
        Stmt::Assignment(assign) => {
            if assign.variables().len() == assign.expressions().len() {
                let parts = assign.variables().iter().zip(assign.expressions().iter());
                for (target, value) in parts {
                    out += &*format!("{} = {};\n", var2js(target), expr2js(value));
                }
            } else {
                assert_eq!(assign.expressions().len(), 1);  // TODO
                let names: String = assign.variables().iter().map(|s| var2js(s) + ", ").collect();
                let val = expr2js(assign.expressions().iter().next().unwrap());
                out += &*format!("[{}] = {};\n", names, val);
            }
        }
        Stmt::Do(_) => todo!(),
        Stmt::FunctionCall(call) => {
            out += &*call2js(call);
            out.push_str(";\n");
        },
        Stmt::FunctionDeclaration(func) => {
            out += &*format!("{}{}", func_name(func.name()), func_body(func.body()));
        },
        Stmt::GenericFor(block) => {
            out.push_str("for (const [");
            for n in block.names() {
                out += &*format!("{}, ", n)
            }
            out.push_str("] of ");
            assert_eq!(block.expressions().len(), 1);
            out += &*expr2js(block.expressions().iter().next().unwrap());
            out.push(')');
            out += &*block2js(block.block());
        },
        Stmt::If(iff) => {
            out += &*format!("if (LuaHelper.as_bool({})) {}", expr2js(iff.condition()), block2js(iff.block()));

            if let Some(chain) = iff.else_if() {
                for choice in chain {
                    out += &*format!("else if (LuaHelper.as_bool({})) {}", expr2js(choice.condition()), block2js(choice.block()));
                }
            }

            if let Some(block) = iff.else_block() {
                out.push_str("else ");
                out += &*block2js(block);
            }
        },
        Stmt::LocalAssignment(assign) => {
            // TODO: cringe copy-paste
            if assign.names().len() == assign.expressions().len() {
                let parts = assign.names().iter().zip(assign.expressions().iter());
                for (target, value) in parts {
                    out += &*format!("let {} = {};\n", target, expr2js(value));
                }
            } else {
                assert_eq!(assign.expressions().len(), 1);  // TODO
                let names: String = assign.names().iter().map(|s| s.to_string() + ", ").collect();
                let val = expr2js(assign.expressions().iter().next().unwrap());
                out += &*format!("let [{}] = {};\n", names, val);
            }
        },
        Stmt::LocalFunction(_) => todo!(),
        Stmt::NumericFor(_) => todo!(),
        Stmt::Repeat(_) => todo!(),
        Stmt::While(_) => todo!(),
        Stmt::CompoundAssignment(_) => todo!(),
        Stmt::ExportedTypeDeclaration(_) => todo!(),
        Stmt::TypeDeclaration(_) => todo!(),
        _ => unimplemented!()
    }

    out
}

fn func_body(func: &FunctionBody) -> String {
    let mut out = "(".to_string();
    for arg in func.parameters() {
        out += &*format!("{}, ", arg);
    }
    out.push(')');
    out += &*block2js(func.block());
    out.push('\n');

    out
}

fn func_name(name: &FunctionName) -> String {
    format!("function {}", name.names())
}

fn expr2js(expr: &Expression) -> String {
    match expr {
        Expression::BinaryOperator { lhs, binop, rhs } => {
            let lhs = expr2js(lhs);
            let rhs = expr2js(rhs);
            let op = match binop {
                BinOp::And(_) => "&&",
                BinOp::Caret(_) => todo!(),
                BinOp::GreaterThan(_) => ">",
                BinOp::GreaterThanEqual(_) => "+>",
                BinOp::LessThan(_) => "<",
                BinOp::LessThanEqual(_) => "<=",
                BinOp::Minus(_) => "-",
                BinOp::Or(_) => "||",
                BinOp::Percent(_) => {
                    // https://www.lua.org/manual/5.1/manual.html#2.2.1
                    return format!("LuaHelper.mod({}, {})", lhs, rhs)
                },
                BinOp::Plus(_) => "+",
                BinOp::Slash(_) => "/",
                BinOp::Star(_) => "*",
                BinOp::TildeEqual(_) => "!==",
                BinOp::TwoDots(_) => "+",  // string concatenation
                BinOp::TwoEqual(_) => "===",
                _ => unimplemented!()
            };
            format!("({} {} {})", lhs, op, rhs)
        },
        Expression::Parentheses { expression, .. } => expr2js(expression),
        Expression::UnaryOperator { unop, expression } => {
            let val = expr2js(expression);
            match unop {
                UnOp::Minus(_) => format!("(-{})", val),
                UnOp::Not(_) => format!("!LuaHelper.as_bool({})", val),
                UnOp::Hash(_) => format!("LuaHelper.array_len({})", val),
                _ => unimplemented!()
            }
        },
        Expression::Value { value, .. } => value2js(value),
        _ => unimplemented!()
    }
}

fn value2js(value: &Value) -> String {
    match value {
        Value::Function(_) => todo!(),
        Value::FunctionCall(call) => call2js(call),
        Value::IfExpression(_) => todo!(),
        Value::InterpolatedString(_) => todo!(),
        Value::TableConstructor(table) => {
            let mut out = String::new();
            for field in table.fields() {
                match field {
                    Field::ExpressionKey { brackets, key, equal, value } => {
                        out += &*format!("[{}]: {}, ", expr2js(key), expr2js(value));

                    },
                    Field::NameKey { key, value, .. } => {
                        out += &*format!("{}: {}, ", key.to_string().trim(), expr2js(value));
                    },
                    Field::NoKey(_) => todo!(),
                    _ => unimplemented!()
                }
            }
            format!("{{ {} }}", out)
        },
        Value::Number(num) => {
            let val: f64 = num.to_string().trim().parse().unwrap();
            format!("{}", val)
        },
        Value::ParenthesesExpression(expr) => expr2js(expr),
        Value::String(val) => val.to_string(),
        Value::Symbol(sym) => {
            let sym = sym.to_string();
            let sym = sym.trim();
            if sym == "true" {
                "true".to_string()
            } else if sym == "false" {
                "false".to_string()
            } else if sym == "nil" {
                "null".to_string()
            } else {
                panic!("Unknown symbol {}", sym)
            }
        },
        Value::Var(var) => var2js(var),
        _ => unimplemented!()
    }
}

fn call2js(call: &FunctionCall) -> String {
    let mut func = prefix2js(call.prefix());
    if func == "print" {
        func = "console.log".to_string();
    } else if func == "require" {
        func = "LuaHelper.require".to_string();
    } else if func == "ipairs" {
        func = "LuaHelper.ipairs".to_string();
    } else if func == "pairs" {
        func = "LuaHelper.pairs".to_string();
    }
    call.suffixes().enumerate().for_each(|(i, s)| println!("call suffix {}: {:?}", i, s));
    for suffix in call.suffixes() {
        func = apply_suffix(func, suffix);
    }
    func
}

fn apply_suffix(expr: String, suffix: &Suffix) -> String {
    let mut out = String::new();
    match suffix {
        Suffix::Call(call) => {
            match call {
                Call::AnonymousCall(args) => {
                    match args {
                        FunctionArgs::Parentheses { arguments, .. } => {
                            for arg in arguments.iter() {
                                out += &*format!("{}, ", expr2js(arg));
                            }
                            format!("{}({})", expr, out)
                        }
                        FunctionArgs::String(_) => todo!(),
                        FunctionArgs::TableConstructor(_) => todo!(),
                        _ => unimplemented!()
                    }
                }
                Call::MethodCall(_) => todo!(),
                _ => unimplemented!()
            }
        }
        Suffix::Index(index) => {
            println!("index: {:?}", index);
            match index {
                Index::Brackets { expression, .. } => format!("{}[{}]", expr, expr2js(expression)),
                Index::Dot { name, .. } => format!("{}.{}", expr, name),
                _ => unimplemented!()
            }
        },
        _ => unimplemented!()
    }
}

fn var2js(var: &Var) -> String {
    match var {
        Var::Expression(expr) => {
            let mut out = prefix2js(expr.prefix());
            for suf in expr.suffixes() {
                out = apply_suffix(out, suf);
            }
            out
        }
        Var::Name(name) => {
            name.token().to_string()
        }
        _ => unimplemented!()
    }
}
fn prefix2js(pref: &Prefix) -> String {
    match pref {
        Prefix::Expression(expr) => {
            expr2js(expr)
        }
        Prefix::Name(name) => {
            name.token().to_string()
        }
        _ => unimplemented!()
    }
}
