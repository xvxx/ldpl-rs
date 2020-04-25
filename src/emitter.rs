//! The Emitter generates a String of C++ code from parsed LDPL code.

use crate::{parser::Rule, LDPLResult};
use pest::iterators::{Pair, Pairs};

pub fn emit(ast: Pairs<Rule>) -> LDPLResult<String> {
    let mut out = String::new();
    for pair in ast {
        match pair.as_rule() {
            Rule::cxx_ext_stmt => todo!(),
            Rule::data_section => out.push_str(&emit_data(pair)?),
            Rule::procedure_section => {}
            Rule::EOI => {}
            _ => {
                return error!("Unexpected Rule: {:?}", pair);
            }
        }
    }
    Ok(out)
}

/// Convert an ident to a C++-friendly ident by stripping illegal
/// characters and whatnot.
/// https://docs.ldpl-lang.org/naming/
/// Based on `fix_identifier()` in ldpl.cpp
fn mangle(ident: &str) -> String {
    let mut mangled = String::with_capacity(ident.len() + 10);

    for c in ident.to_uppercase().chars() {
        if c.is_alphanumeric() {
            mangled.push(c);
        } else {
            mangled.push_str(&format!("c{}_", c as u16));
        }
    }

    format!("VAR_{}", mangled)
}

/// Convert `name IS TEXT` into a C++ variable declaration.
/// Used by DATA: and LOCAL DATA: sections.
fn emit_data(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut out = vec![];

    for def in pair.into_inner() {
        assert!(def.as_rule() == Rule::type_def);
        let mut parts = def.into_inner();
        let ident = parts.next().unwrap();
        let r#type = parts.next().unwrap();
        match r#type.as_str().to_lowercase().as_ref() {
            "number" => {
                out.push(format!(r#"ldpl_number {} = 0;"#, mangle(ident.as_str())));
            }
            "text" => {
                out.push(format!(r#"chText {} = "";"#, mangle(ident.as_str())));
            }
            "number list" => {
                out.push(format!(
                    r#"ldpl_list<ldpl_number> {};"#,
                    mangle(ident.as_str())
                ));
            }
            "number map" => {
                out.push(format!(
                    r#"ldpl_map<ldpl_number> {};"#,
                    mangle(ident.as_str())
                ));
            }
            "text list" => {
                out.push(format!(r#"ldpl_list<chText> {};"#, mangle(ident.as_str())));
            }
            "text map" => {
                out.push(format!(r#"ldpl_map<chText> {};"#, mangle(ident.as_str())));
            }
            _ => todo!(),
        }
    }

    Ok(out.join("\n"))
}
