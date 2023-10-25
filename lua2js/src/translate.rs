use full_moon::ast::{Ast, BinOp, Block, Call, Expression, Field, FunctionArgs, FunctionBody, FunctionCall, FunctionName, Index, LastStmt, Prefix, Stmt, Suffix, UnOp, Value, Var};

pub fn tojs(ast: &Ast) -> String {
    let mut out = "// This file is @generated from lua. Do not edit manually!\n\n".to_string();
    for node in ast.nodes().stmts() {
        out += &*stmt2js(node);
    }
    out
}

// TODO: make this configurable
/// Enables explicit runtime type checking.
/// - Arithmetic is on numbers or coercible strings.
const SAFE: bool = true;

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
                        out += &*comma_join(ret.returns().iter().map(expr2js));
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

const JS_KEYWORDS: [&str; 19] = ["new", "abstract", "arguments", "await", "const", "let", "var", "try", "catch", "null", "typeof", "throw", "delete", "debugger", "class", "instanceof", "finally", "case", "yield"];

fn stmt2js(stmt: &Stmt) -> String {
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
                let names: String = comma_join(assign.variables().iter().map(var2js));
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
            out += &* comma_join(block.names().iter());
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
                let names: String = comma_join(assign.names().iter().map(ToString::to_string).map(unkeyword));
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
    // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/rest_parameters
    out += &*comma_join(func.parameters().iter().map(|p| p.to_string()).map(|s| if s == "..." { s + "arguments" } else {s}));
    out.push(')');
    out += &*block2js(func.block());
    out.push('\n');

    out
}

// TODO: it would be clever to make this a trait implemented for those iterators so I could called it with postfix syntax.
fn comma_join(entries: impl Iterator<Item=impl ToString>) -> String {
    entries.map(|p| p.to_string()).collect::<Vec<String>>().join(", ")
}

fn func_name(name: &FunctionName) -> String {
    format!("function {}", unkeyword(name.names().to_string()))
}

// TODO: hash set?
fn unkeyword(s: String) -> String {
    let keyword = JS_KEYWORDS.iter().any(|ss| **ss == s);
    if keyword {
        s + "__"  // TODO: assumes you didn't name something this, but like why
    } else {
        s
    }
}

fn expr2js(expr: &Expression) -> String {
    match expr {
        Expression::BinaryOperator { lhs, binop, rhs } => {
            let lhs = expr2js(lhs);
            let rhs = expr2js(rhs);
            let require_numbers = matches!(binop, BinOp::GreaterThan(_) | BinOp::GreaterThanEqual(_) | BinOp::LessThan(_) | BinOp::LessThanEqual(_) | BinOp::Minus(_) | BinOp::Percent(_) | BinOp::Plus(_) | BinOp::Slash(_) | BinOp::Star(_));
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
            if SAFE && require_numbers {
                format!("(LuaHelper.require_number({}) {} LuaHelper.require_number({}))", lhs, op, rhs)

            } else {
                format!("({} {} {})", lhs, op, rhs)
            }
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
        Value::Function((_, func)) => {
            format!("function{}",func_body(func))
        },
        Value::FunctionCall(call) => call2js(call),
        Value::IfExpression(_) => todo!(),
        Value::InterpolatedString(_) => todo!(),
        Value::TableConstructor(table) => {
            let mut out = String::new();

            // TODO: this is a bit deranged.
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/rest_parameters
            if table.fields().len() == 1 {
                if let Field::NoKey(val) = table.fields().iter().next().unwrap() {
                    if let Expression::Value { value, .. } = val {
                        if let Value::Symbol(sym) = value.as_ref() {
                            if sym.to_string() == "..." {
                                return "LuaHelper.array_to_table(arguments)".to_string();
                            }
                        }
                    }
                }
            }

            for field in table.fields() {
                match field {
                    Field::ExpressionKey { key, value, .. } => {
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
                // Not using null because failed table lookups return nil in lua and undefined in js
                "undefined".to_string()
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
    } else if func == "setmetatable" {
        func = "LuaHelper.setmetatable".to_string();
    } else if func == "error" {
        func = "throw".to_string();
    } else {
        func = unkeyword(func);
    }

    for suffix in call.suffixes() {
        func = apply_suffix(func, suffix);
    }
    func
}

fn apply_suffix(expr: String, suffix: &Suffix) -> String {
    match suffix {
        Suffix::Call(call) => {
            match call {
                Call::AnonymousCall(args) => {
                    format!("{}({})", expr, args2js(args))
                }
                // TODO: this evaluates the receiver expression twice which is wrong. Can't just use the this keyword like js because lua has being a method as a property of the call not the function definition.
                // TODO: also a trailing comma if method has no extra arguments which is fine but imperfect
                Call::MethodCall(method) => format!("LuaHelper.method_call({0}, \"{1}\", {2})", expr, method.name(), args2js(method.args())),
                _ => unimplemented!()
            }
        }
        Suffix::Index(index) => {
            match index {
                Index::Brackets { expression, .. } => format!("{}[{}]", expr, expr2js(expression)),
                Index::Dot { name, .. } => format!("{}.{}", expr, name),
                _ => unimplemented!()
            }
        },
        _ => unimplemented!()
    }
}

fn args2js(args: &FunctionArgs) -> String {
    match args {
        FunctionArgs::Parentheses { arguments, .. } => {
            comma_join(arguments.iter().map(expr2js))
        }
        FunctionArgs::String(s) => {
            // I recognise this doesn't matter and you can still run arbitrary code, there's no inner sandbox here and its only me on my own website anyway but still, out of principle.
            assert!(!s.to_string().contains("${"), "XSS this asshole. https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals \n{:?}", s);
            format!("`{}`", s.to_string().trim().strip_prefix("[[\n").unwrap().strip_suffix("]]").unwrap())
        },
        FunctionArgs::TableConstructor(_) => todo!(),
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
            unkeyword(name.token().to_string())
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
            unkeyword(name.token().to_string())
        }
        _ => unimplemented!()
    }
}
