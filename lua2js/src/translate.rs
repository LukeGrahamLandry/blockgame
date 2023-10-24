use std::fs;
use std::process::Command;
use full_moon::ast::{Ast, BinOp, Call, Expression, FunctionArgs, FunctionCall, Index, Prefix, Stmt, Suffix, UnOp, Value, Var};

pub fn tojs(ast: &Ast) -> String {
    let mut out = include_str!("runtime.js").to_string();
    for node in ast.nodes().stmts() {
        out += &*stmt2js(node);
    }
    out
}

fn stmt2js(stmt: &Stmt) -> String {
    println!("stmt: {:?}", stmt);
    let mut out = String::from("\n");
    match stmt {
        Stmt::Assignment(assign) => {
            let parts = assign.variables().iter().zip(assign.expressions().iter());
            for (target, value) in parts {
                out += &*format!("{} = {};", var2js(target), expr2js(value));
            }
        }
        Stmt::Do(_) => todo!(),
        Stmt::FunctionCall(call) => {
            out += &*call2js(call);
        },
        Stmt::FunctionDeclaration(_) => todo!(),
        Stmt::GenericFor(_) => todo!(),
        Stmt::If(_) => todo!(),
        Stmt::LocalAssignment(assign) => {
            let parts = assign.names().iter().zip(assign.expressions().iter());
            for (target, value) in parts {
                out += &*format!("let {} = {};", target.token(), expr2js(value));
            }
        },
        Stmt::LocalFunction(_) => todo!(),
        Stmt::NumericFor(_) => todo!(),
        Stmt::Repeat(_) => todo!(),
        Stmt::While(_) => todo!(),
        Stmt::CompoundAssignment(_) => todo!(),
        Stmt::ExportedTypeDeclaration(_) => todo!(),
        Stmt::TypeDeclaration(_) => todo!(),
        _ => todo!()
    }

    out
}

fn expr2js(expr: &Expression) -> String {
    println!("expr: {:?}", expr);
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
                BinOp::TildeEqual(_) => "!=",
                BinOp::TwoDots(_) => "+",  // string concatenation
                BinOp::TwoEqual(_) => "==",
                _ => unimplemented!()
            };
            format!("({} {} {})", lhs, op, rhs)
        },
        Expression::Parentheses { expression, .. } => expr2js(expression),
        Expression::UnaryOperator { unop, expression } => {
            let val = expr2js(expression);
            match unop {
                UnOp::Minus(_) => format!("(-{})", val),
                UnOp::Not(_) => format!("(!{})", val),
                UnOp::Hash(_) => format!("lua.array_len({})", val),
                _ => unimplemented!()
            }
        },
        Expression::Value { value, type_assertion } => value2js(value),
        _ => unimplemented!()
    }
}

fn value2js(value: &Value) -> String {
    match value {
        Value::Function(_) => todo!(),
        Value::FunctionCall(call) => call2js(call),
        Value::IfExpression(_) => todo!(),
        Value::InterpolatedString(_) => todo!(),
        Value::TableConstructor(_) => todo!(),
        Value::Number(num) => {
            let val: f64 = num.to_string().trim().parse().unwrap();
            format!("{}", val)
        },
        Value::ParenthesesExpression(expr) => expr2js(expr),
        Value::String(val) => val.to_string(),
        Value::Symbol(_) => todo!(),
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
                        FunctionArgs::Parentheses { parentheses, arguments } => {
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
                Index::Brackets { .. } => todo!(),
                Index::Dot { dot, name } => {
                    format!("{}.{}", expr, name)
                }
                _ => unimplemented!()
            }
        },
        _ => unimplemented!()
    }
}

fn var2js(var: &Var) -> String {
    match var {
        Var::Expression(expr) => {
            prefix2js(expr.prefix())
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


fn log(src: &str) {
    let ast = full_moon::parse(src);
    println!("{}", tojs(&ast.unwrap()));
}

fn compare(lua_src: &str, name: &str) {
    let ast = full_moon::parse(lua_src).unwrap();
    let js_src = tojs(&ast);
    let lua_out = run_lua(lua_src, name);
    println!("lua says: {}", lua_out);
    let js_out = run_js(&js_src, name
    );
    println!("js says: {}", js_out);
    assert_eq!(lua_out, js_out);
}

fn run_lua(src: &str, name: &str) -> String {
    let path = format!("target/{}.lua", name);
    fs::write(&path, src).unwrap();
    let output = Command::new("luajit").arg(&*path).output().unwrap();
    assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn run_js(src: &str, name: &str) -> String {
    let path = format!("target/{}.js", name);
    fs::write(&path, src).unwrap();
    let output = Command::new("node").arg(&*path).output().unwrap();
    assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn file_output_tests() {
    for file in fs::read_dir("tests").unwrap() {
        let file = file.unwrap();
        let name = file.file_name().to_string_lossy().to_string();
        let content = fs::read_to_string(file.path()).unwrap();
        compare(&content, &name);
    }
}

#[test]
fn demo() {
    compare("print((-1) % 10)", "demo");
}