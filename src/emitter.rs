//! The Emitter generates a String of C++ code from parsed LDPL code.

use crate::{parser::Rule, LDPLResult};
use pest::iterators::{Pair, Pairs};

/// Setup the C++ main() function
const MAIN_HEADER: &'static str = r#"
ldpl_list<chText> VAR_ARGV;
ldpl_number VAR_ERRORCODE = 0;
chText VAR_ERRORTEXT = "";

int main(int argc, char* argv[]) {
    cout.precision(numeric_limits<ldpl_number>::digits10);
    for(int i = 1; i < argc; ++i) VAR_ARGV.inner_collection.push_back(argv[i]);

"#;

const MAIN_FOOTER: &'static str = r#"
    return 0;
}
"#;

/// Turns parsed LDPL code into a string of C++ code.
pub fn emit(ast: Pairs<Rule>) -> LDPLResult<String> {
    let mut out = vec![];
    let mut main = MAIN_HEADER.to_string();

    for pair in ast {
        match pair.as_rule() {
            Rule::cxx_ext_stmt => todo!(),
            Rule::data_section => out.push(emit_data(pair)?),
            Rule::create_stmt_stmt => todo!(),
            Rule::sub_def_stmt => out.push(emit_sub_def_stmt(pair)?),
            Rule::EOI => {}
            _ => main.push_str(&emit_subproc_stmt(pair)?),
        }
    }

    main.push_str(MAIN_FOOTER);
    out.push(main);

    Ok(out.join("\n"))
}

/// LDPL Type => C++ Type
fn type_for(ldpl_type: &str) -> &str {
    match ldpl_type.to_lowercase().as_ref() {
        "number" => "ldpl_number",
        "number list" => "ldpl_list<ldpl_number>",
        "number map" => "ldpl_map<chText>",
        "text" => "chText",
        "text list" => "ldpl_list<chText>",
        "text map" => "ldpl_map<chText>",
        _ => "UNKNOWN_TYPE",
    }
}

/// Mangle a variable name for C++.
fn mangle_var(ident: &str) -> String {
    format!("VAR_{}", mangle(ident))
}

/// Mangle a subprocedure name.
fn mangle_sub(ident: &str) -> String {
    format!("SUB_{}", mangle(ident))
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

    mangled
}

/// Convert `name IS TEXT` into a C++ variable declaration.
/// Used by DATA: and LOCAL DATA: sections.
fn emit_data(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut out = vec![];

    for def in pair.into_inner() {
        assert!(def.as_rule() == Rule::type_def);

        let mut parts = def.into_inner();
        let ident = parts.next().unwrap().as_str();
        let typename = parts.next().unwrap().as_str();
        let mut var = format!("{} {}", type_for(typename), mangle_var(ident));

        if typename == "number" {
            var.push_str(" = 0");
        } else if typename == "text" {
            var.push_str(r#" = """#);
        }

        var.push(';');
        out.push(var);
    }

    Ok(out.join("\n"))
}

/// Convert a param list into a C++ function signature params list.
fn emit_params(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut out = vec![];

    for def in pair.into_inner() {
        assert!(def.as_rule() == Rule::type_def);
        let mut parts = def.into_inner();
        let ident = parts.next().unwrap().as_str();
        let typename = parts.next().unwrap().as_str();
        out.push(format!("{} {}", type_for(typename), mangle_var(ident),));
    }

    Ok(out.join(", "))
}

/// Function definition.
fn emit_sub_def_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut params = String::new();
    let mut name = String::new();
    let mut vars = String::new();
    let mut body: Vec<String> = vec![];

    for node in pair.into_inner() {
        match node.as_rule() {
            Rule::ident => {
                name = mangle_sub(node.as_str());
            }
            Rule::sub_param_section => params = emit_params(node)?,
            Rule::sub_data_section => vars = emit_data(node)?,
            _ => body.push(emit_subproc_stmt(node)?),
        }
    }

    Ok(format!(
        "void {}({}) {{\n{}\n{}\n}}\n",
        name,
        params,
        vars,
        body.join("\n"),
    ))
}

/// Emit a stmt from the PROCEDURE: section of a file or function.
fn emit_subproc_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut out = vec![];

    out.push(match pair.as_rule() {
        Rule::call_stmt => emit_call_stmt(pair)?,
        Rule::display_stmt => emit_display_stmt(pair)?,
        Rule::store_stmt => emit_store_stmt(pair)?,
        Rule::solve_stmt => "".to_string(),
        Rule::if_stmt => emit_if_stmt(pair)?,
        Rule::else_stmt => "".to_string(),
        Rule::while_stmt => emit_while_stmt(pair)?,
        Rule::user_stmt => {
            panic!("Unexpected user_stmt: {:?}", pair);
        }
        _ => "".to_string(),
    });

    Ok(out.join(""))
}

/// STORE _ IN _
fn emit_store_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let val = iter.next().unwrap().as_str();
    let ident = iter.next().unwrap().as_str();
    Ok(format!("{} = {};\n", mangle_var(ident), val))
}

/// CALL _ WITH _ ...
fn emit_call_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    println!("CALL: {:?}", pair);
    Ok("TODO".to_string())
}

/// DISPLAY _ ...
fn emit_display_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut parts = vec!["cout".to_string()];
    let expr_list = pair.into_inner().next().unwrap();
    for node in expr_list.into_inner() {
        parts.push(emit_expr(node)?);
    }
    parts.push("flush".into());
    Ok(format!("{};\n", parts.join(" << ")))
}

/// IF and WHILE test statement
fn emit_test_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut out = vec![];
    for test in pair.into_inner() {
        match test.as_rule() {
            Rule::or_test_expr => {
                let mut iter = test.into_inner();
                let left = emit_test_expr(iter.next().unwrap())?;
                let right = emit_test_expr(iter.next().unwrap())?;
                out.push(format!("({} || {})", left, right));
            }
            Rule::and_test_expr => {
                let mut iter = test.into_inner();
                let left = emit_test_expr(iter.next().unwrap())?;
                let right = emit_test_expr(iter.next().unwrap())?;
                out.push(format!("({} && {})", left, right));
            }
            Rule::one_test_expr => out.push(emit_test_expr(test)?),
            _ => unimplemented!(),
        }
    }
    Ok(out.join("\n"))
}

/// Single test expression. Use _stmt for expressions with OR / AND.
fn emit_test_expr(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let left = emit_expr(iter.next().unwrap())?;
    let sign = match iter.next().unwrap().as_rule() {
        Rule::equal_expr => "==",
        Rule::not_equal_expr => "!=",
        Rule::gt_expr => ">",
        Rule::lt_expr => "<",
        Rule::gte_expr => ">=",
        Rule::lte_expr => "<=",
        _ => unimplemented!(),
    };
    let right = emit_expr(iter.next().unwrap())?;
    Ok(format!("({} {} {})", left, sign, right))
}

/// Variable, Number, or Text
fn emit_expr(pair: Pair<Rule>) -> LDPLResult<String> {
    Ok(match pair.as_rule() {
        Rule::var => mangle_var(pair.as_str()),
        Rule::number => pair.as_str().to_string(),
        Rule::text => pair.as_str().to_string(),
        Rule::linefeed => "\"\\n\"".to_string(),
        _ => {
            unimplemented!();
        }
    })
}

/// WHILE _ DO / REPEAT
fn emit_while_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let test = iter.next().unwrap();
    let test = emit_test_stmt(test)?;

    let mut body = vec![];
    for node in iter {
        body.push(emit_subproc_stmt(node)?);
    }
    Ok(format!("while {} {{\n{}\n}}\n", test, body.join("\n")))
}

/// IF _ THEN / END IF
fn emit_if_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let test = iter.next().unwrap();
    let test = emit_test_stmt(test)?;

    let mut body = vec![];
    for node in iter {
        match node.as_rule() {
            Rule::else_stmt => body.push(emit_else_stmt(node)?),
            _ => body.push(emit_subproc_stmt(node)?),
        }
    }

    Ok(format!("if {} {{\n{}\n}}\n", test, body.join("\n")))
}

/// ELSE IF _ THEN
fn emit_else_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut test = None;
    for node in pair.into_inner() {
        match node.as_rule() {
            Rule::test_expr => test = Some(emit_test_stmt(node)?),
            _ => unimplemented!(),
        }
    }

    if test.is_some() {
        Ok(format!("}} else if {} {{\n", test.unwrap()))
    } else {
        Ok(format!("}} else {{\n"))
    }
}
