//! The Emitter generates a String of C++ code from parsed LDPL code.

use crate::{parser::Rule, LDPLResult, LDPLType};
use pest::iterators::{Pair, Pairs};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Track indentation depth.
static DEPTH: AtomicUsize = AtomicUsize::new(0);

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

/// Call when an unexpected Pair/Rule is encountered.
macro_rules! unexpected {
    ($rule:expr) => {
        return error!("Unexpected rule: {:?}", $rule);
    };
}

/// Emit a single line with indentation. Used to build multi-line responses.
macro_rules! emit_line {
    ($msg:expr) => {
        format!("{}{}\n", "    ".repeat(DEPTH.load(Ordering::SeqCst)), $msg)
    };
    ($fmt:expr, $($args:expr),*) => {
        emit_line!(format!($fmt, $($args),*))
    };
    ($fmt:expr, $($args:expr,)*) => {
        emit_line!(format!($fmt, $($args,)*))
    };
}

/// Print a line of code at the current indentation level with a
/// trailing newline.
macro_rules! emit {
    ($msg:expr) => {
        Ok(emit_line!($msg))
    };
    ($fmt:expr, $($args:expr),*) => {
        emit!(format!($fmt, $($args),*))
    };
    ($fmt:expr, $($args:expr,)*) => {
        emit!(format!($fmt, $($args,)*))
    };
}

/// Increase indentation level (depth)
macro_rules! indent {
    () => {
        DEPTH.fetch_add(1, Ordering::SeqCst);
    };
}

/// Decrease indentation level
macro_rules! dedent {
    () => {
        if DEPTH.load(Ordering::SeqCst) > 0 {
            DEPTH.fetch_sub(1, Ordering::SeqCst);
        }
    };
}

/// State of our LDPL program, including variables and defined
/// sub-procedures. Eventually we'll move this into a Parser so we can
/// have multiple emitters (for different languages).
pub struct Emitter {
    globals: HashMap<String, LDPLType>,
    locals: HashMap<String, LDPLType>,

    // in a sub-procedure? RETURN doesn't work outside of one.
    in_sub: bool,
    // in a loop? BREAK/CONTINUE only work in loops. Vec for nesting.
    in_loop: Vec<bool>,
}

/// Turns parsed LDPL code into a string of C++ code.
pub fn emit(ast: Pairs<Rule>) -> LDPLResult<String> {
    let mut emitter = Emitter {
        globals: HashMap::default(),
        locals: HashMap::default(),
        in_loop: vec![],
        in_sub: false,
    };
    emitter.emit(ast)
}

impl Emitter {
    /// Turns parsed LDPL code into a string of C++ code.
    pub fn emit(&mut self, ast: Pairs<Rule>) -> LDPLResult<String> {
        let mut out = vec![CPP_HEADER.to_string()];
        let mut main = MAIN_HEADER.to_string();

        // Predeclared vars
        self.globals
            .insert("ARGV".into(), LDPLType::List(Box::new(LDPLType::Text)));

        for pair in ast {
            match pair.as_rule() {
                Rule::cpp_ext_stmt => todo!(),
                Rule::data_section => out.push(self.emit_data(pair, false)?),
                Rule::create_stmt_stmt => todo!(),
                Rule::sub_def_stmt => out.push(self.emit_sub_def_stmt(pair)?),
                Rule::EOI => break,
                _ => {
                    indent!();
                    main.push_str(&self.emit_subproc_stmt(pair)?);
                    dedent!();
                }
            }
        }

        main.push_str(MAIN_FOOTER);
        out.push(main);

        Ok(out.join(""))
    }

    /// Convert `name IS TEXT` into a C++ variable declaration.
    /// Used by DATA: and LOCAL DATA: sections.
    fn emit_data(&mut self, pair: Pair<Rule>, local: bool) -> LDPLResult<String> {
        let mut out = vec![];

        for def in pair.into_inner() {
            assert!(def.as_rule() == Rule::type_def);

            let mut parts = def.into_inner();
            let ident = parts.next().unwrap().as_str();
            let typename = parts.next().unwrap().as_str();
            let mut var = format!("{} {}", emit_type(typename), mangle_var(ident));

            if typename == "number" {
                var.push_str(" = 0");
            } else if typename == "text" {
                var.push_str(r#" = """#);
            }

            let varname = ident.to_string().to_uppercase();
            let ldpltype = LDPLType::from(typename);
            if local {
                if self.locals.contains_key(&varname) {
                    return error!("Duplicate declaration for variable: {}", ident);
                }
                self.locals.insert(varname, ldpltype);
            } else {
                if self.globals.contains_key(&varname) {
                    return error!("Duplicate declaration for variable: {}", ident);
                }
                self.globals.insert(varname, ldpltype);
            };

            var.push(';');
            out.push(var);
        }

        Ok(format!("{}\n\n", out.join("\n")))
    }

    /// Convert a param list into a C++ function signature params list.
    fn emit_params(&mut self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut out = vec![];

        for def in pair.into_inner() {
            assert!(def.as_rule() == Rule::type_def);
            let mut parts = def.into_inner();
            let ident = parts.next().unwrap().as_str();
            let typename = parts.next().unwrap().as_str();
            self.locals
                .insert(ident.to_string(), LDPLType::from(typename));
            out.push(format!("{}& {}", emit_type(typename), mangle_var(ident)));
        }

        Ok(out.join(", "))
    }

    /// Function definition.
    fn emit_sub_def_stmt(&mut self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut params = String::new();
        let mut name = String::new();
        let mut vars = String::new();
        let mut body: Vec<String> = vec![];

        self.in_sub = true;
        self.locals.clear();
        indent!();
        for node in pair.into_inner() {
            match node.as_rule() {
                Rule::ident => name = mangle_sub(node.as_str()),
                Rule::sub_param_section => params = self.emit_params(node)?,
                Rule::sub_data_section => vars = self.emit_data(node, true)?,
                _ => body.push(self.emit_subproc_stmt(node)?),
            }
        }
        dedent!();
        self.in_sub = false;

        emit!("void {}({}) {{{}\n{}}}", name, params, vars, body.join(""),)
    }

    /// Emit a stmt from the PROCEDURE: section of a file or function.
    fn emit_subproc_stmt(&mut self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut out = vec![];

        out.push(match pair.as_rule() {
            // control flow
            Rule::call_stmt => self.emit_call_stmt(pair)?,
            Rule::if_stmt => self.emit_if_stmt(pair)?,
            Rule::else_stmt => return error!("unexpected ELSE statement"),
            Rule::while_stmt => self.emit_while_stmt(pair)?,
            Rule::for_each_stmt => self.emit_for_each_stmt(pair)?,
            Rule::for_stmt => self.emit_for_stmt(pair)?,
            Rule::loop_kw_stmt => self.emit_loop_kw_stmt(pair)?,
            Rule::return_stmt => self.emit_return_stmt(pair)?,
            Rule::goto_stmt => self.emit_goto_stmt(pair)?,
            Rule::label_stmt => self.emit_label_stmt(pair)?,
            Rule::exit_stmt => self.emit_exit_stmt(pair)?,
            Rule::wait_stmt => self.emit_wait_stmt(pair)?,
            Rule::store_quote_stmt => self.emit_quote_stmt(pair)?,
            Rule::store_stmt => self.emit_store_stmt(pair)?,

            // math
            Rule::solve_stmt => self.emit_solve_stmt(pair)?,
            Rule::floor_stmt => self.emit_floor_stmt(pair)?,
            Rule::modulo_stmt => self.emit_modulo_stmt(pair)?,

            // text
            Rule::join_stmt => self.emit_join_stmt(pair)?,
            Rule::old_join_stmt => self.emit_old_join_stmt(pair)?,
            Rule::replace_stmt => self.emit_replace_stmt(pair)?,
            Rule::split_stmt => self.emit_split_stmt(pair)?,
            Rule::get_char_stmt => self.emit_get_char_stmt(pair)?,
            Rule::get_ascii_stmt => self.emit_get_ascii_stmt(pair)?,
            Rule::get_char_code_stmt => self.emit_get_char_code_stmt(pair)?,
            Rule::get_index_stmt => self.emit_get_index_stmt(pair)?,
            Rule::count_stmt => self.emit_count_stmt(pair)?,
            Rule::substr_stmt => self.emit_substring_stmt(pair)?,
            Rule::trim_stmt => self.emit_trim_stmt(pair)?,

            // list
            Rule::push_stmt => self.emit_push_stmt(pair)?,
            Rule::delete_stmt => self.emit_delete_stmt(pair)?,

            // map
            Rule::get_keys_count_stmt => self.emit_get_keys_count_stmt(pair)?,
            Rule::get_keys_stmt => self.emit_get_keys_stmt(pair)?,

            // list + map
            Rule::clear_stmt => self.emit_clear_stmt(pair)?,
            Rule::copy_stmt => self.emit_copy_stmt(pair)?,

            // list + text
            Rule::get_length_stmt => self.emit_get_length_stmt(pair)?,

            // io
            Rule::display_stmt => self.emit_display_stmt(pair)?,
            Rule::load_stmt => self.emit_load_stmt(pair)?,
            Rule::write_stmt => self.emit_write_stmt(pair)?,
            Rule::append_stmt => self.emit_append_stmt(pair)?,
            Rule::accept_stmt => self.emit_accept_stmt(pair)?,
            Rule::execute_stmt => self.emit_execute_stmt(pair)?,

            // create statement
            Rule::user_stmt => {
                panic!("Unexpected user_stmt: {:?}", pair);
            }
            _ => unexpected!(pair),
        });

        Ok(out.join(""))
    }

    ////
    // CONTROL FLOW

    /// STORE _ IN _
    fn emit_store_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();

        let expr = iter.next().unwrap();
        let var = iter.next().unwrap();
        let val = self.emit_expr_for_type(expr, self.type_of_var(var.clone())?)?;

        emit!("{} = {};", self.emit_var(var)?, val)
    }

    /// STORE QUOTE IN _
    fn emit_quote_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let var = self.emit_var(iter.next().unwrap())?;
        let txt = iter.next().unwrap().as_str();
        // remove extra preceeding \n from txt. parser limitation.
        if !txt.is_empty() {
            emit!("{} = {:?};", var, &txt[1..])
        } else {
            emit!("{} = \"\";", var)
        }
    }

    /// RETURN
    fn emit_return_stmt(&self, _pair: Pair<Rule>) -> LDPLResult<String> {
        if !self.in_sub {
            return error!("RETURN can't be used outside of SUB-PROCEDURE");
        }
        emit!("return;")
    }

    /// BREAK / CONTINUE
    fn emit_loop_kw_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        if self.in_loop.is_empty() {
            return error!("{} can't be used without FOR/WHILE loop", pair.as_str());
        }
        emit!("{};", pair.as_str())
    }

    /// GOTO _
    fn emit_goto_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let label = pair.into_inner().next().unwrap();
        emit!("goto label_{};", mangle(label.as_str()))
    }

    /// LABEL _
    fn emit_label_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let label = pair.into_inner().next().unwrap();
        emit!("label_{}:", mangle(label.as_str()))
    }

    /// WAIT _ MILLISECONDS
    fn emit_wait_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let count = self.emit_expr(pair.into_inner().next().unwrap())?;
        emit!(
            "std::this_thread::sleep_for(std::chrono::milliseconds((long int){}));",
            count
        )
    }

    /// EXIT
    fn emit_exit_stmt(&self, _pair: Pair<Rule>) -> LDPLResult<String> {
        emit!("exit(0);")
    }

    /// CALL _ WITH _ ...
    fn emit_call_stmt(&mut self, pair: Pair<Rule>) -> LDPLResult<String> {
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
                        prefix.push(emit_line!(
                            "ldpl_number {} = {};",
                            var,
                            self.emit_expr(param)?
                        ));
                        params.push(var);
                        var_id += 1;
                    }
                    Rule::text | Rule::linefeed => {
                        let var = format!("LPVAR_{}", var_id);
                        prefix.push(emit_line!("chText {} = {};", var, self.emit_expr(param)?));
                        params.push(var);
                        var_id += 1;
                    }
                    Rule::var => params.push(self.emit_expr(param)?),
                    _ => unexpected!(param),
                }
            }
        }

        Ok(format!(
            "{}{}",
            prefix.join(""),
            emit_line!("{}({});", mangle_sub(ident.as_str()), params.join(", "))
        ))
    }

    /// IF and WHILE test statement
    fn emit_test_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut out = vec![];
        for test in pair.into_inner() {
            match test.as_rule() {
                Rule::or_test_expr => {
                    let mut iter = test.into_inner();
                    let left = self.emit_test_expr(iter.next().unwrap())?;
                    let right = self.emit_test_expr(iter.next().unwrap())?;
                    out.push(format!("({} || {})", left, right));
                }
                Rule::and_test_expr => {
                    let mut iter = test.into_inner();
                    let left = self.emit_test_expr(iter.next().unwrap())?;
                    let right = self.emit_test_expr(iter.next().unwrap())?;
                    out.push(format!("({} && {})", left, right));
                }
                Rule::one_test_expr => out.push(self.emit_test_expr(test)?),
                _ => unexpected!(test),
            }
        }
        Ok(out.join("\n"))
    }

    /// Single test expression. Use _stmt for expressions with OR / AND.
    fn emit_test_expr(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let left = self.emit_expr(iter.next().unwrap())?;
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
        let right = self.emit_expr(iter.next().unwrap())?;
        Ok(format!("({} {} {})", left, sign, right))
    }

    /// Coerce Number -> Text and Text -> Number.
    fn emit_expr_for_type(&self, expr: Pair<Rule>, typename: &LDPLType) -> LDPLResult<String> {
        let expr_type = self.type_of_expr(expr.clone())?;

        if typename.is_text() && expr.as_rule() == Rule::number {
            // 45 => "45"
            Ok(format!(r#""{}""#, self.emit_expr(expr)?))
        } else if typename.is_number() && (expr_type.is_text() || expr_type.is_text_collection()) {
            // "123" => to_number("123")
            Ok(format!("to_number({})", self.emit_expr(expr)?))
        } else if typename.is_text() && (expr_type.is_number() || expr_type.is_number_collection())
        {
            // txt_var => to_ldpl_string(txt_var)
            Ok(format!("to_ldpl_string({})", self.emit_expr(expr)?))
        } else {
            self.emit_expr(expr)
        }
    }

    /// Variable, Number, or Text
    fn emit_expr(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        Ok(match pair.as_rule() {
            Rule::var => self.emit_var(pair)?,
            Rule::ident => mangle_var(pair.as_str()),
            Rule::number => {
                // trim leading 0s from numbers
                let num = pair.as_str().trim_start_matches('0');
                if num.is_empty() { "0" } else { num }.to_string()
            }
            Rule::text => pair.as_str().to_string(),
            Rule::linefeed => "\"\\n\"".to_string(),
            _ => panic!("UNIMPLEMENTED: {:?}", pair),
        })
    }

    /// Turn an ident/lookup pair into a C++ friendly ident.
    fn emit_var(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        assert!(
            pair.as_rule() == Rule::var,
            "Expected Rule::var, got {:?}",
            pair.as_rule()
        );

        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::ident => Ok(mangle_var(inner.as_str())),
            Rule::lookup => {
                let mut iter = inner.into_inner();
                let basevar = iter.next().unwrap();
                let mut parts = vec![self.emit_expr(basevar)?];
                for part in iter {
                    parts.push(format!("[{}]", self.emit_expr(part)?));
                }
                Ok(parts.join(""))
            }
            _ => unexpected!(inner),
        }
    }

    /// WHILE _ DO / REPEAT
    fn emit_while_stmt(&mut self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let test = iter.next().unwrap();
        let test = self.emit_test_stmt(test)?;

        self.in_loop.push(true);
        let mut body = vec![];
        indent!();
        for node in iter {
            body.push(self.emit_subproc_stmt(node)?);
        }
        dedent!();
        self.in_loop.pop();

        Ok(format!(
            "{}{}{}",
            emit_line!("while {} {{", test),
            body.join(""),
            emit_line!("}")
        ))
    }

    /// IF _ THEN / END IF
    fn emit_if_stmt(&mut self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let test = iter.next().unwrap();
        let test = self.emit_test_stmt(test)?;

        let mut body = vec![];
        indent!();
        for node in iter {
            match node.as_rule() {
                Rule::else_stmt => body.push(self.emit_else_stmt(node)?),
                _ => body.push(self.emit_subproc_stmt(node)?),
            }
        }
        dedent!();

        Ok(format!(
            "{}{}{}",
            emit_line!("if {} {{", test),
            body.join(""),
            emit_line!("}")
        ))
    }

    /// ELSE IF _ THEN
    fn emit_else_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut test = None;
        for node in pair.into_inner() {
            match node.as_rule() {
                Rule::test_expr => test = Some(self.emit_test_stmt(node)?),
                _ => unexpected!(node),
            }
        }

        dedent!();
        let out = if test.is_some() {
            emit!("}} else if {} {{", test.unwrap())
        } else {
            emit!("} else {")
        };
        indent!();
        out
    }

    /// FOR _ IN _ TO _ STEP _ DO / REPEAT
    fn emit_for_stmt(&mut self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let var = mangle_var(iter.next().unwrap().as_str());
        let from = self.emit_expr(iter.next().unwrap())?;
        let to = self.emit_expr(iter.next().unwrap())?;
        let step = self.emit_expr(iter.next().unwrap())?;

        self.in_loop.push(true);
        indent!();
        let mut body = vec![];
        for node in iter {
            body.push(self.emit_subproc_stmt(node)?);
        }
        dedent!();
        self.in_loop.pop();

        let init = format!("{} = {}", var, from);
        let test = format!(
            "{step} >= 0 ? {var} < {to} : {var} > {to}",
            step = step,
            var = var,
            to = to
        );
        let incr = format!("{} += {}", var, step);

        Ok(format!(
            "{}{}{}",
            emit_line!("for({}; {}; {}) {{", init, test, incr),
            body.join(""),
            emit_line!("}")
        ))
    }

    /// FOR EACH _ IN _ DO / REPEAT
    fn emit_for_each_stmt(&mut self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let ident = mangle_var(iter.next().unwrap().as_str());
        let collection = self.emit_expr(iter.next().unwrap())?;
        let range_var = "RVAR_0"; // TODO: not really...

        self.in_loop.push(true);
        indent!();
        let mut body = vec![];
        for node in iter {
            body.push(self.emit_subproc_stmt(node)?);
        }
        dedent!();
        self.in_loop.pop();

        Ok(format!(
            "{}{}{}{}",
            emit_line!(
                "for (auto& {} : {}.inner_collection) {{",
                range_var,
                collection
            ),
            emit_line!("{} = {};", ident, range_var),
            body.join(""),
            emit_line!("}")
        ))
    }

    ////
    // ARITHMETIC

    /// MODULO _ BY _ IN _
    fn emit_modulo_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let base = self.emit_expr(iter.next().unwrap())?;
        let by = self.emit_expr(iter.next().unwrap())?;
        let var = self.emit_var(iter.next().unwrap())?;

        emit!("{} = modulo({}, {});", var, base, by)
    }

    /// FLOOR _
    /// FLOOR _ IN _
    /// TODO: only FLOOR _ in 4.4
    fn emit_floor_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let stmt = pair.into_inner().next().unwrap();
        let rule = stmt.as_rule();
        let mut iter = stmt.into_inner();
        let left = self.emit_expr(iter.next().unwrap())?;
        let mut right = left.clone();
        match rule {
            Rule::floor_in_stmt => right = self.emit_var(iter.next().unwrap())?,
            Rule::floor_mut_stmt => {}
            _ => unexpected!(rule),
        }

        emit!("{} = floor({});", left, right)
    }

    /// IN _ SOLVE X
    fn emit_solve_stmt(&mut self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let ident = iter.next().unwrap();

        emit!(
            "{} = {};",
            self.emit_var(ident)?,
            self.emit_solve_expr(iter.next().unwrap())?
        )
    }

    // Math expression part of a SOLVE statement
    fn emit_solve_expr(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut parts = vec![];

        for part in pair.into_inner() {
            match part.as_rule() {
                Rule::var | Rule::number | Rule::text => parts.push(self.emit_expr(part)?),
                Rule::solve_expr => parts.push(self.emit_solve_expr(part)?),
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
    fn emit_split_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let text = self.emit_expr(iter.next().unwrap())?;
        let splitter = self.emit_expr(iter.next().unwrap())?;
        let var = self.emit_var(iter.next().unwrap())?;
        emit!("{} = utf8_split_list({}, {});", var, text, splitter)
    }

    /// REPLACE _ FROM _ WITH _ IN _
    /// replace_stmt = { ^"REPLACE" ~ expr ~ ^"FROM" ~ expr ~ ^"WITH" ~ expr ~ ^"IN" ~ var }
    fn emit_replace_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let search = self.emit_expr(iter.next().unwrap())?;
        let text = self.emit_expr(iter.next().unwrap())?;
        let replacement = self.emit_expr(iter.next().unwrap())?;
        let var = self.emit_var(iter.next().unwrap())?;

        emit!("{} = str_replace(((chText){}).str_rep(), ((chText){}).str_rep(), ((chText){}).str_rep());",
            var, text, search, replacement)
    }

    /// IN _ JOIN _ _...
    fn emit_join_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let var = self.emit_var(iter.next().unwrap())?;

        let mut out = vec![emit_line!(r#"joinvar = "";"#)];
        for expr in iter {
            out.push(emit_line!(
                "join(joinvar, {}, joinvar);",
                self.emit_expr_for_type(expr, &LDPLType::Text)?
            ));
        }
        out.push(emit_line!("{} = joinvar;", var));

        Ok(format!("{}", out.join("")))
    }

    /// JOIN _ AND _ IN _
    fn emit_old_join_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let left = self.emit_expr_for_type(iter.next().unwrap(), &LDPLType::Text)?;
        let right = self.emit_expr_for_type(iter.next().unwrap(), &LDPLType::Text)?;
        let var = self.emit_var(iter.next().unwrap())?;

        emit!("join({}, {}, {});", left, right, var)
    }

    /// TRIM _ IN _
    fn emit_trim_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let expr = self.emit_expr(iter.next().unwrap())?;
        let var = self.emit_var(iter.next().unwrap())?;
        emit!("{} = trimCopy({});", var, expr)
    }

    /// COUNT _ FROM _ IN _
    fn emit_count_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let search = self.emit_expr(iter.next().unwrap())?;
        let text = self.emit_expr(iter.next().unwrap())?;
        let var = self.emit_var(iter.next().unwrap())?;
        emit!("{} = utf8Count({}, {});", var, text, search)
    }

    /// SUBSTRING _ FROM _ LENGTH _ IN _
    fn emit_substring_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let text = self.emit_expr(iter.next().unwrap())?;
        let search = self.emit_expr(iter.next().unwrap())?;
        let length = self.emit_expr(iter.next().unwrap())?;
        let var = self.emit_var(iter.next().unwrap())?;

        Ok(format!(
            "{}{}",
            emit_line!("joinvar = {};", text),
            emit_line!("{} = joinvar.substr({}, {});", var, search, length)
        ))
    }

    /// GET INDEX OF _ FROM _ IN _
    fn emit_get_index_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let search = self.emit_expr(iter.next().unwrap())?;
        let text = self.emit_expr(iter.next().unwrap())?;
        let var = self.emit_var(iter.next().unwrap())?;
        emit!("{} = utf8GetIndexOf({}, {});", var, text, search)
    }

    /// GET CHARACTER CODE OF _ IN _
    fn emit_get_char_code_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let expr = self.emit_expr(iter.next().unwrap())?;
        let var = self.emit_var(iter.next().unwrap())?;
        emit!("{} = get_char_num({});", var, expr)
    }

    /// GET ASCII CHARACTER _ IN _
    fn emit_get_ascii_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let chr = self.emit_expr(iter.next().unwrap())?;
        let var = self.emit_var(iter.next().unwrap())?;
        emit!("{} = (char)({});", var, chr)
    }

    /// GET CHARACTER AT _ FROM _ IN _
    fn emit_get_char_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let at = self.emit_expr(iter.next().unwrap())?;
        let from = self.emit_expr(iter.next().unwrap())?;
        let var = self.emit_var(iter.next().unwrap())?;
        emit!("{} = charat({}, {});", var, from, at)
    }

    ////
    // LIST + TEXT

    // GET LENGTH OF _ IN _
    fn emit_get_length_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let expr = iter.next().unwrap();
        let var = self.emit_var(iter.next().unwrap())?;
        let expr_type = self.type_of_expr(expr.clone())?;
        let expr = self.emit_expr(expr)?;

        if expr_type.is_text() {
            emit!("{} = ((chText){}).size();", var, expr)
        } else if expr_type.is_list() {
            emit!("{} = {}.inner_collection.size();", var, expr)
        } else {
            unexpected!(expr_type)
        }
    }

    ////
    // LIST

    /// PUSH _ TO _
    fn emit_push_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let expr = self.emit_expr(iter.next().unwrap())?;
        let list = self.emit_var(iter.next().unwrap())?;
        emit!("{}.inner_collection.push_back({});", list, expr)
    }

    /// DELETE LAST ELEMENT OF _
    fn emit_delete_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let list = self.emit_var(iter.next().unwrap())?;
        emit!(format!(
            "if({list}.inner_collection.size() > 0) {list}.inner_collection.pop_back();",
            list = list
        ))
    }

    ////
    // MAP

    /// GET KEYS COUNT OF _ IN _
    fn emit_get_keys_count_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let map = self.emit_expr(iter.next().unwrap())?;
        let var = self.emit_var(iter.next().unwrap())?;
        emit!("{} = {}.inner_collection.size();", var, map)
    }

    /// GET KEYS OF _ IN _
    fn emit_get_keys_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let map = self.emit_expr(iter.next().unwrap())?;
        let var = self.emit_var(iter.next().unwrap())?;
        emit!("get_indices({}, {});", var, map)
    }

    ////
    // MAP + LIST

    /// COPY _ TO _
    fn emit_copy_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let from = self.emit_expr(iter.next().unwrap())?;
        let to = self.emit_var(iter.next().unwrap())?;
        emit!("{}.inner_collection = {}.inner_collection;", to, from)
    }

    /// CLEAR _
    fn emit_clear_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let collection = self.emit_var(iter.next().unwrap())?;
        emit!("{}.inner_collection.clear();", collection)
    }

    ////
    // IO

    /// DISPLAY _...
    fn emit_display_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut parts = vec!["cout".to_string()];
        for node in pair.into_inner() {
            parts.push(self.emit_expr(node)?);
        }
        parts.push("flush".into());
        emit!("{};", parts.join(" << "))
    }

    /// ACCEPT _
    /// ACCEPT _ UNTIL EOF
    fn emit_accept_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let stmt = pair.into_inner().next().unwrap();

        let eof = stmt.as_rule() == Rule::accept_eof_stmt;
        let ident = stmt.into_inner().next().unwrap();

        let fun = if eof {
            "input_until_eof()"
        } else {
            // TODO check for text vs number
            "input_number()"
        };

        emit!("{} = {};", self.emit_var(ident)?, fun)
    }

    /// LOAD FILE _ IN _
    fn emit_load_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let path = self.emit_expr(iter.next().unwrap())?;
        let var = self.emit_var(iter.next().unwrap())?;
        emit!("load_file({}, {});", path, var)
    }

    /// WRITE _ TO FILE _
    fn emit_write_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let expr = self.emit_expr(iter.next().unwrap())?;
        let path = self.emit_expr(iter.next().unwrap())?;

        Ok(format!("{}{}{}",
            emit_line!("file_writing_stream.open(expandHomeDirectory(((chText){}).str_rep()), ios_base::out);", path),
            emit_line!("file_writing_stream << {};", expr),
            emit_line!("file_writing_stream.close();")
        ))
    }

    /// APPEND _ TO FILE _
    fn emit_append_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let expr = self.emit_expr(iter.next().unwrap())?;
        let path = self.emit_expr(iter.next().unwrap())?;

        Ok(format!("{}{}{}",
            emit_line!("file_writing_stream.open(expandHomeDirectory(((chText){}).str_rep()), ios_base::app);", path),
            emit_line!("file_writing_stream << {};", expr),
            emit_line!("file_writing_stream.close();")
        ))
    }

    /// EXECUTE _
    /// EXECUTE _ AND STORE EXIT CODE IN _
    /// EXECUTE _ AND STORE OUTPUT IN _
    fn emit_execute_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let pair = pair.into_inner().next().unwrap();
        let rule = pair.as_rule();
        let mut iter = pair.into_inner();
        match rule {
            Rule::execute_expr_stmt => emit!("system({});", self.emit_expr(iter.next().unwrap())?),
            Rule::execute_output_stmt => {
                let expr = self.emit_expr(iter.next().unwrap())?;
                let var = self.emit_var(iter.next().unwrap())?;
                emit!("{} = exec({});", var, expr)
            }
            Rule::execute_exit_code_stmt => {
                let expr = self.emit_expr(iter.next().unwrap())?;
                let var = self.emit_var(iter.next().unwrap())?;
                emit!(
                    "{} = (system({}) >> 8) & 0xff;", //shift wait() val and get lowest 2
                    var,
                    expr
                )
            }
            _ => unexpected!(rule),
        }
    }
}

////
// HELPERS

impl Emitter {
    /// Find the type for an expression.
    fn type_of_expr(&self, expr: Pair<Rule>) -> LDPLResult<&LDPLType> {
        match expr.as_rule() {
            Rule::var => self.type_of_var(expr),
            Rule::ident => self.type_of_var(expr),
            Rule::number => Ok(&LDPLType::Number),
            Rule::text | Rule::linefeed => Ok(&LDPLType::Text),
            _ => unexpected!(expr),
        }
    }

    /// Find the LDPLType for a variable, local or global.
    fn type_of_var(&self, var: Pair<Rule>) -> LDPLResult<&LDPLType> {
        match var.as_rule() {
            Rule::var => self.type_of_var(var.into_inner().next().unwrap()),
            Rule::ident => {
                if let Some(t) = self.locals.get(&var.as_str().to_uppercase()) {
                    Ok(t)
                } else if let Some(t) = self.globals.get(&var.as_str().to_uppercase()) {
                    Ok(t)
                } else {
                    error!("No type found for {:?}", var)
                }
            }
            Rule::lookup => {
                let mut iter = var.into_inner();
                let base = iter.next().unwrap();
                let out = self.type_of_var(base);
                out
            }
            _ => unexpected!(var),
        }
    }
}

/// LDPL Type => C++ Type
fn emit_type(ldpl_type: &str) -> &str {
    match ldpl_type.to_lowercase().as_ref() {
        "number" => "ldpl_number",
        "number list" => "ldpl_list<ldpl_number>",
        "number map" | "number vector" => "ldpl_map<ldpl_number>",
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
