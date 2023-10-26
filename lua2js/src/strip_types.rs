use full_moon::ast::types::{GenericDeclaration, GenericParameterInfo, TypeArgument, TypeAssertion, TypeDeclaration, TypeInfo, TypeSpecifier};
use full_moon::node::{Node, Tokens};
use full_moon::{Error, parse, print, ShortString};
use full_moon::ast::{Ast, FunctionBody, FunctionDeclaration, FunctionName};
use full_moon::tokenizer::{Token, TokenReference, TokenType};
use full_moon::visitors::VisitorMut;

pub struct StripTypes();

pub fn strip_types(src: &str) -> Result<String, Box<Error>> {
    let ast = parse(src)?;
    let ast = StripTypes().visit_ast(ast);
    Ok(print(&ast))
}

impl VisitorMut for StripTypes {
    fn visit_type_argument(&mut self, node: TypeArgument) -> TypeArgument {
        TypeArgument::new(notype())
    }

    fn visit_type_specifier(&mut self, node: TypeSpecifier) -> TypeSpecifier {
        if has_newline(node.tokens()) {
            TypeSpecifier::new(newline_notype()).with_punctuation(empty())
        } else {
            TypeSpecifier::new(notype()).with_punctuation(empty())
        }
    }

    fn visit_type_declaration(&mut self, node: TypeDeclaration) -> TypeDeclaration {
        TypeDeclaration::new(empty(), notype()).with_equal_token(empty()).with_type_token(empty())
    }

    fn visit_type_assertion(&mut self, node: TypeAssertion) -> TypeAssertion {
        if has_newline(node.tokens()) {
            TypeAssertion::new(newline_notype())
        } else {
            TypeAssertion::new(notype())
        }.with_assertion_op(empty())
    }

    fn visit_function_body(&mut self, node: FunctionBody) -> FunctionBody {
        node.with_generics(None)
    }
}

fn notype() -> TypeInfo {
    TypeInfo::Basic(empty())
}

fn newline_notype() -> TypeInfo {
    TypeInfo::Basic(newline())
}

fn empty() -> TokenReference {
    TokenReference::new(vec![], Token::new(TokenType::Whitespace { characters: Default::default() }), vec![])
}
fn newline() -> TokenReference {
    TokenReference::new(vec![], Token::new(TokenType::Whitespace { characters: ShortString::new("\n") }), vec![])
}

fn has_newline(t: Tokens) -> bool {
    let s = t.last().unwrap().to_string();
    s.ends_with('\n')
}