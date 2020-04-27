//! The Emitter generates a String of C++ code from parsed LDPL code.

use crate::{parser::Rule, LDPLResult};
use pest::iterators::{Pair, Pairs};

/// Include common C++ functions in our program.
const CPP_HEADER: &'static str = include_str!("../lib/ldpl_header.cpp");

/// Setup the C++ main() function
const MAIN_HEADER: &'static str = r#"
ldpl_list<chText> VAR_ARGV;

int main(int argc, char* argv[]) {
    cout.precision(numeric_limits<ldpl_number>::digits10);
    for(int i = 1; i < argc; ++i) VAR_ARGV.inner_collection.push_back(argv[i]);

"#;
const MAIN_FOOTER: &'static str = r#"
    return 0;
}
"#;

macro_rules! unexpected {
    ($rule:expr) => {
        panic!("Unexpected rule: {:?}", $rule);
    };
}

/// Turns parsed LDPL code into a string of C++ code.
pub fn emit(ast: Pairs<Rule>) -> LDPLResult<String> {
    let mut out = vec![CPP_HEADER.to_string()];
    let mut main = MAIN_HEADER.to_string();

    for pair in ast {
        match pair.as_rule() {
            Rule::cpp_ext_stmt => todo!(),
            Rule::data_section => out.push(emit_data(pair)?),
            Rule::create_stmt_stmt => todo!(),
            Rule::sub_def_stmt => out.push(emit_sub_def_stmt(pair)?),
            Rule::EOI => break,
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
        "number map" | "number vector" => "ldpl_map<chText>",
        "text" => "chText",
        "text list" => "ldpl_list<chText>",
        "text map" | "text vector" => "ldpl_map<chText>",
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
        let ident = parts.next().unwrap();
        let typename = parts.next().unwrap().as_str();
        let mut var = format!("{} {}", type_for(typename), mangle_var(ident.as_str()));

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
        let ident = parts.next().unwrap();
        let typename = parts.next().unwrap().as_str();
        out.push(format!("{}& {}", type_for(typename), emit_var(ident)?));
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
            Rule::ident => name = mangle_sub(node.as_str()),
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
        // control flow
        Rule::call_stmt => emit_call_stmt(pair)?,
        Rule::if_stmt => emit_if_stmt(pair)?,
        Rule::else_stmt => return error!("unexpected ELSE statement"),
        Rule::while_stmt => emit_while_stmt(pair)?,
        // Rule::for_each_stmt => todo!(), //emit_for_each_stmt(pair)?,
        // Rule::for_stmt => todo!(),      //emit_for_stmt(pair)?,
        // Rule::loop_kw_stmt => todo!(),  //emit_loop_kw(pair)?,
        // Rule::return_stmt => todo!(),   //emit_return_stmt(pair)?,
        // Rule::goto_stmt => todo!(),     //emit_goto_stmt(pair)?,
        // Rule::label_stmt => todo!(),    //emit_label_stmt(pair)?,
        // Rule::exit_stmt => todo!(),     //emit_exit_stmt(pair)?,
        // Rule::wait_stmt => todo!(),     //emit_wait_stmt(pair)?,
        Rule::store_quote_stmt => emit_quote_stmt(pair)?,
        Rule::store_stmt => emit_store_stmt(pair)?,

        // math
        Rule::solve_stmt => emit_solve_stmt(pair)?,
        Rule::floor_stmt => emit_floor_stmt(pair)?,
        Rule::modulo_stmt => emit_modulo_stmt(pair)?,
        // Rule::get_rand_stmt => todo!(),
        // Rule::raise_stmt => todo!(),
        // Rule::log_stmt => todo!(),
        // Rule::sin_stmt => todo!(),
        // Rule::cos_stmt => todo!(),
        // Rule::tan_stmt => todo!(),

        // text
        Rule::join_stmt => emit_join_stmt(pair)?,
        Rule::old_join_stmt => emit_old_join_stmt(pair)?,
        Rule::replace_stmt => emit_replace_stmt(pair)?,
        Rule::split_stmt => emit_split_stmt(pair)?,
        Rule::get_char_stmt => emit_get_char_stmt(pair)?,
        Rule::get_ascii_stmt => emit_get_ascii_stmt(pair)?,
        Rule::get_char_code_stmt => emit_get_char_code_stmt(pair)?,
        Rule::get_index_stmt => emit_get_index_stmt(pair)?,
        Rule::count_stmt => emit_count_stmt(pair)?,
        Rule::substr_stmt => emit_substring_stmt(pair)?,
        Rule::trim_stmt => emit_trim_stmt(pair)?,

        // list
        // Rule::push_stmt => todo!(),   // emit_push_stmt(pair)?,
        // Rule::delete_stmt => todo!(), // emit_delete_stmt(pair)?,

        // map
        // Rule::get_keys_count_stmt => todo!(), // emit_get_keys_count_stmt(pair)?,
        // Rule::get_keys_stmt => todo!(),       // emit_get_keys_stmt(pair)?,

        // list + map
        // Rule::clear_stmt => todo!(), // emit_clear_stmt(pair)?,
        // Rule::copy_stmt => todo!(),  // emit_copy_stmt(pair)?,

        // list + text
        Rule::get_length_stmt => emit_get_length_stmt(pair)?,

        // io
        Rule::display_stmt => emit_display_stmt(pair)?,
        Rule::load_stmt => emit_load_stmt(pair)?,
        Rule::write_stmt => emit_write_stmt(pair)?,
        Rule::append_stmt => emit_append_stmt(pair)?,
        Rule::accept_stmt => emit_accept_stmt(pair)?,
        Rule::execute_stmt => emit_execute_stmt(pair)?,

        // create statement
        Rule::user_stmt => {
            panic!("Unexpected user_stmt: {:?}", pair);
        }
        _ => unexpected!(pair),
    });

    Ok(out.join(""))
}

/// STORE _ IN _
fn emit_store_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let val = emit_expr(iter.next().unwrap())?;
    let var = emit_var(iter.next().unwrap())?;
    Ok(format!("{} = {};\n", var, val))
}

/// STORE QUOTE IN _
fn emit_quote_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let var = emit_var(iter.next().unwrap())?;
    let txt = iter.next().unwrap().as_str();
    // remove extra preceeding \n from txt. parser limitation.
    if !txt.is_empty() {
        Ok(format!("{} = \"{}\";\n", var, &txt[1..]))
    } else {
        Ok(format!("{} = \"\";\n", var))
    }
}

/// CALL _ WITH _ ...
fn emit_call_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let ident = iter.next().unwrap();

    let mut var_id = 0;
    let mut prefix = vec![];

    let mut params = vec![];
    if let Some(expr_list) = iter.next() {
        for param in expr_list.into_inner() {
            match param.as_rule() {
                Rule::number => {
                    let var = format!("LPVAR_{}", var_id);
                    prefix.push(format!("ldpl_number {} = {};", var, emit_expr(param)?));
                    params.push(var);
                    var_id += 1;
                }
                Rule::text | Rule::linefeed => {
                    let var = format!("LPVAR_{}", var_id);
                    prefix.push(format!("chText {} = {};", var, emit_expr(param)?));
                    params.push(var);
                    var_id += 1;
                }
                Rule::var => params.push(emit_expr(param)?),
                _ => unexpected!(param),
            }
        }
    }

    Ok(format!(
        "{}\n{}({});\n",
        prefix.join("\n"),
        mangle_sub(ident.as_str()),
        params.join(", ")
    ))
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
            _ => unexpected!(test),
        }
    }
    Ok(out.join("\n"))
}

/// Single test expression. Use _stmt for expressions with OR / AND.
fn emit_test_expr(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let left = emit_expr(iter.next().unwrap())?;
    let mid = iter.next().unwrap();
    let sign = match mid.as_rule() {
        Rule::equal_expr => "==",
        Rule::not_equal_expr => "!=",
        Rule::gt_expr => ">",
        Rule::lt_expr => "<",
        Rule::gte_expr => ">=",
        Rule::lte_expr => "<=",
        _ => unexpected!(mid),
    };
    let right = emit_expr(iter.next().unwrap())?;
    Ok(format!("({} {} {})", left, sign, right))
}

/// Variable, Number, or Text
fn emit_expr(pair: Pair<Rule>) -> LDPLResult<String> {
    Ok(match pair.as_rule() {
        Rule::var => emit_var(pair)?,
        Rule::ident => mangle_var(pair.as_str()),
        Rule::number => pair.as_str().to_string(),
        Rule::text => pair.as_str().to_string(),
        Rule::linefeed => "\"\\n\"".to_string(),
        _ => panic!("UNIMPLEMENTED: {:?}", pair),
    })
}

/// Turn an ident/lookup pair into a C++ friendly ident.
fn emit_var(pair: Pair<Rule>) -> LDPLResult<String> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::ident => Ok(mangle_var(inner.as_str())),
        Rule::lookup => {
            let mut iter = inner.into_inner();
            let mut parts = vec![emit_expr(iter.next().unwrap())?];
            for part in iter {
                parts.push(format!("[{}]", emit_expr(part)?));
            }
            Ok(parts.join(""))
        }
        _ => unexpected!(inner),
    }
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
            _ => unexpected!(node),
        }
    }

    if test.is_some() {
        Ok(format!("}} else if {} {{\n", test.unwrap()))
    } else {
        Ok(format!("}} else {{\n"))
    }
}

////
// ARITHMETIC

/// MODULO _ BY _ IN _
fn emit_modulo_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let base = emit_expr(iter.next().unwrap())?;
    let by = emit_expr(iter.next().unwrap())?;
    let var = emit_var(iter.next().unwrap())?;

    Ok(format!("{} = modulo({}, {});", var, base, by))
}

/// FLOOR _
/// FLOOR _ IN _
/// TODO: only FLOOR _ in 4.4
fn emit_floor_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let stmt = pair.into_inner().next().unwrap();
    let rule = stmt.as_rule();
    let mut iter = stmt.into_inner();
    let left = emit_expr(iter.next().unwrap())?;
    let mut right = left.clone();
    match rule {
        Rule::floor_in_stmt => right = emit_var(iter.next().unwrap())?,
        Rule::floor_mut_stmt => {}
        _ => unexpected!(rule),
    }

    Ok(format!("{} = floor({});", left, right))
}

/// IN _ SOLVE X
fn emit_solve_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let ident = iter.next().unwrap();

    Ok(format!(
        "{} = {};\n",
        emit_var(ident)?,
        emit_solve_expr(iter.next().unwrap())?
    ))
}

// Math expression part of a SOLVE statement
fn emit_solve_expr(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut parts = vec![];

    for part in pair.into_inner() {
        match part.as_rule() {
            Rule::var | Rule::number | Rule::text => parts.push(emit_expr(part)?),
            Rule::solve_expr => parts.push(emit_solve_expr(part)?),
            Rule::math_op => parts.push(part.as_str().to_string()),
            _ => {
                panic!("unexpected rule: {:?}", part);
            }
        }
    }

    Ok(parts.join(" "))
}

////
// TEXT

/// SPLIT _ BY _ IN _
fn emit_split_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let text = emit_expr(iter.next().unwrap())?;
    let splitter = emit_expr(iter.next().unwrap())?;
    let var = emit_var(iter.next().unwrap())?;
    Ok(format!(
        "{} = utf8_split_list({}, {});\n",
        var, text, splitter
    ))
}

/// REPLACE _ FROM _ WITH _ IN _
/// replace_stmt = { ^"REPLACE" ~ expr ~ ^"FROM" ~ expr ~ ^"WITH" ~ expr ~ ^"IN" ~ var }
fn emit_replace_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let search = emit_expr(iter.next().unwrap())?;
    let text = emit_expr(iter.next().unwrap())?;
    let replacement = emit_expr(iter.next().unwrap())?;
    let var = emit_var(iter.next().unwrap())?;

    Ok(format!("{} = str_replace(((chText){}).str_rep(), ((chText){}).str_rep() ((chText){}).str_rep());\n",
    var, text, search, replacement))
}

/// IN _ JOIN _ _...
fn emit_join_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let var = emit_var(iter.next().unwrap())?;

    let mut out = vec![r#"joinvar = "";"#.to_string()];
    for expr in iter {
        out.push(format!("join(joinvar, {}, joinvar);", emit_expr(expr)?));
    }
    out.push(format!("{} = joinvar;", var));

    Ok(out.join("\n"))
}

/// JOIN _ AND _ IN _
fn emit_old_join_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let left = emit_expr(iter.next().unwrap())?;
    let right = emit_expr(iter.next().unwrap())?;
    let var = emit_var(iter.next().unwrap())?;

    Ok(format!("join({}, {}, {});\n", left, right, var))
}

/// TRIM _ IN _
fn emit_trim_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let expr = emit_expr(iter.next().unwrap())?;
    let var = emit_var(iter.next().unwrap())?;
    Ok(format!("{} = trimCopy({});", var, expr))
}

/// COUNT _ FROM _ IN _
fn emit_count_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let search = emit_expr(iter.next().unwrap())?;
    let text = emit_expr(iter.next().unwrap())?;
    let var = emit_var(iter.next().unwrap())?;
    Ok(format!("{} = utf8Count({}, {});\n", var, text, search))
}

/// SUBSTRING _ FROM _ LENGTH _ IN _
fn emit_substring_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let text = emit_expr(iter.next().unwrap())?;
    let search = emit_expr(iter.next().unwrap())?;
    let length = emit_expr(iter.next().unwrap())?;
    let var = emit_var(iter.next().unwrap())?;
    Ok(format!(
        "joinvar = {};\n{} = joinvar.substr({}, {});",
        text, var, search, length
    ))
}

/// GET INDEX OF _ FROM _ IN _
fn emit_get_index_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let search = emit_expr(iter.next().unwrap())?;
    let text = emit_expr(iter.next().unwrap())?;
    let var = emit_var(iter.next().unwrap())?;
    Ok(format!("{} = utf8GetIndexOf({}, {});\n", var, text, search))
}

/// GET CHARACTER CODE OF _ IN _
fn emit_get_char_code_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let expr = emit_expr(iter.next().unwrap())?;
    let var = emit_var(iter.next().unwrap())?;
    Ok(format!("{} = get_char_num({});\n", var, expr))
}

/// GET ASCII CHARACTER _ IN _
fn emit_get_ascii_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let chr = emit_expr(iter.next().unwrap())?;
    let var = emit_var(iter.next().unwrap())?;
    Ok(format!("{} = (char)({});\n", var, chr))
}

/// GET CHARACTER AT _ FROM _ IN _
fn emit_get_char_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let at = emit_expr(iter.next().unwrap())?;
    let from = emit_expr(iter.next().unwrap())?;
    let var = emit_var(iter.next().unwrap())?;
    Ok(format!("{} = charat({}, {});", var, from, at))
}

////
// LIST + TEXT

// GET LENGTH OF _ IN _
fn emit_get_length_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let it = emit_expr(iter.next().unwrap())?;
    let var = emit_var(iter.next().unwrap())?;

    Ok(format!("{} = ((chText){}).size();", var, it))

    /*
    if self.is_text(it) {
        Ok(format!("{} = ((chText){}).size();", var, it))
    } else if self.is_list(it) {
        Ok(format!("{} = {}.inner_collection.size();", var, it))
    } else {
        unimplemented!()
    }
    */
}

////
// IO

/// DISPLAY _...
fn emit_display_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut parts = vec!["cout".to_string()];
    for node in pair.into_inner() {
        parts.push(emit_expr(node)?);
    }
    parts.push("flush".into());
    Ok(format!("{};\n", parts.join(" << ")))
}

/// ACCEPT _
/// ACCEPT _ UNTIL EOF
fn emit_accept_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let stmt = pair.into_inner().next().unwrap();

    let eof = stmt.as_rule() == Rule::accept_eof_stmt;
    let ident = stmt.into_inner().next().unwrap();

    let fun = if eof {
        "input_until_eof()"
    } else {
        // TODO check for text vs number
        "input_number()"
    };

    Ok(format!("{} = {};\n", emit_var(ident)?, fun))
}

/// LOAD FILE _ IN _
fn emit_load_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let path = emit_expr(iter.next().unwrap())?;
    let var = emit_var(iter.next().unwrap())?;
    Ok(format!("load_file({}, {});\n", path, var))
}

/// WRITE _ TO FILE _
fn emit_write_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let expr = emit_expr(iter.next().unwrap())?;
    let path = emit_expr(iter.next().unwrap())?;
    Ok(format!(
        r#"
file_writing_stream.open(expandHomeDirectory(((chText){}).str_rep()), ios_base::out);
file_writing_stream << {};
file_writing_stream.close();
"#,
        path, expr
    ))
}

/// APPEND _ TO FILE _
fn emit_append_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let mut iter = pair.into_inner();
    let expr = emit_expr(iter.next().unwrap())?;
    let path = emit_expr(iter.next().unwrap())?;
    Ok(format!(
        r#"
file_writing_stream.open(expandHomeDirectory(((chText){}).str_rep()), ios_base::app);
file_writing_stream << {};
file_writing_stream.close();
"#,
        path, expr
    ))
}

/// EXECUTE _
/// EXECUTE _ AND STORE EXIT CODE IN _
/// EXECUTE _ AND STORE OUTPUT IN _
fn emit_execute_stmt(pair: Pair<Rule>) -> LDPLResult<String> {
    let pair = pair.into_inner().next().unwrap();
    let rule = pair.as_rule();
    let mut iter = pair.into_inner();
    Ok(match rule {
        Rule::execute_expr_stmt => format!("system({});", emit_expr(iter.next().unwrap())?),
        Rule::execute_output_stmt => {
            let var = emit_var(iter.next().unwrap())?;
            format!("{} = exec({});", var, emit_expr(iter.next().unwrap())?)
        }
        Rule::execute_exit_code_stmt => {
            let var = emit_var(iter.next().unwrap())?;
            format!(
                "{} = (system({}) >> 8) & 0xff;", //shift wait() val and get lowest 2
                var,
                emit_expr(iter.next().unwrap())?
            )
        }
        _ => unexpected!(rule),
    })
}
