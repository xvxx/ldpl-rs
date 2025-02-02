//! The Compiler generates a String of C++ code from parsed LDPL code.

use crate::{
    parser::{LDPLParser, Parser, Rule},
    LDPLResult, LDPLType, LPM_LOCATION,
};
use pest::iterators::{Pair, Pairs};
use std::{
    collections::HashMap,
    fmt,
    sync::atomic::{AtomicUsize, Ordering},
};

////
// CONSTANTS

/// Track indentation depth.
static DEPTH: AtomicUsize = AtomicUsize::new(0);

/// Include LDPL C++ internal functions in our output.
const CPP_HEADER: &'static str = include_str!("../lib/ldpl_header.cpp");

/// Setup the C++ main() function
const MAIN_HEADER: &'static str = r#"
int main(int argc, char* argv[]) {
    cout.precision(numeric_limits<ldpl_number>::digits10);
    for(int i = 1; i < argc; ++i) VAR_ARGV.inner_collection.push_back(argv[i]);

"#;
const MAIN_FOOTER: &'static str = r#"
    return 0;
}
"#;

////
// DATA

/// State of our LDPL program, including variables and defined
/// sub-procedures. Eventually we'll move this into a Parser so we can
/// have multiple emitters (for different languages).
#[derive(Default)]
pub struct Compiler {
    /// Body of the the main() function. _HEADER and _FOOTER get
    /// inserted automatically when we're done.
    pub main: Vec<String>,

    /// Sub-procedure definitions.
    pub subs: Vec<String>,

    /// Global variable declarations.
    pub vars: Vec<String>,

    /// C++ Extension files to include when building.
    /// Add with `add_extension()`.
    pub exts: Vec<String>,

    /// Compiler flags to build with.
    pub flags: Vec<String>,

    /// Forward function declarations.
    forwards: Vec<String>,

    /// EXTERNAL variables
    extern_vars: HashMap<String, bool>,

    /// Global variables defined in the DATA: section. Used for error
    /// checking.
    globals: HashMap<String, LDPLType>,

    /// Local variables, re-defined for each sub-procedure.
    locals: HashMap<String, LDPLType>,

    /// Sub definitions. name => params
    defs: HashMap<String, Vec<LDPLType>>,

    /// Path of the file we're currently compiling, if any.
    path: Option<String>,

    /// When a sub is called before it's defined, we stick it in this
    /// list. When it's defined we remove it from the list. If the
    /// list isn't empty when we're done, we have an error.
    pub expected_defs: HashMap<String, bool>,

    /// User-defined statements created with CREATE STATEMENT.
    /// The same statement can reference multiple SUBs based on the
    /// param types, so we use a vec.
    user_stmts: HashMap<String, Vec<String>>,

    // in a sub-procedure? RETURN doesn't work outside of one.
    in_sub: bool,

    // in a loop? BREAK/CONTINUE only work in loops. Vec for nesting.
    in_loop: Vec<bool>,

    // counter for tmp variables
    tmp_id: usize,
}

////
// MACROS

/// Call when an unexpected Pair/Rule is encountered.
macro_rules! unexpected {
    ($rule:expr) => {
        return error!("Unexpected rule: {:?}", $rule);
    };
}

/// Produce a single line with indentation. Used to build multi-line responses.
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

/// Produce a line of code at the current indentation level with a
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

////
// FUNCTIONS

/// Turns a string of LDPL code into C++ code.
pub fn compile(code: &str) -> LDPLResult<Compiler> {
    let mut compiler = Compiler::default();
    compiler.compile(code)?;
    Ok(compiler)
}

/// Turns parsed LDPL code into C++ code.
pub fn compile_ast(ast: Pairs<Rule>) -> LDPLResult<Compiler> {
    let mut compiler = Compiler::default();
    compiler.compile_ast(ast)?;
    Ok(compiler)
}

/// Turns LDPL code on disk into C++ code.
pub fn load_and_compile(path: &str) -> LDPLResult<Compiler> {
    let mut compiler = Compiler::default();
    compiler.load_and_compile(path)?;
    Ok(compiler)
}

/// Create a fresh compiler.
pub fn new() -> Compiler {
    Compiler::default()
}

/// Treating the compiler as a string produces the compiled C++.
impl fmt::Display for Compiler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}{}{}{}",
            CPP_HEADER,
            self.forwards.join(""),
            self.vars.join("\n"),
            self.subs.join(""),
            MAIN_HEADER,
            self.main.join(""),
            MAIN_FOOTER
        )
    }
}

impl Compiler {
    /// Add a C++ file to include when building.
    pub fn add_extension(&mut self, ext_file: String) -> LDPLResult<()> {
        self.exts.push(ext_file);
        Ok(())
    }

    /// Add a C++ flag to include when building.
    pub fn add_flag(&mut self, flag: String) -> LDPLResult<()> {
        self.flags.push(flag);
        Ok(())
    }

    /// Load a file from disk, parse it, and generate C++ code.
    pub fn load_and_compile(&mut self, path: &str) -> LDPLResult<()> {
        // info!("Loading {}", path);
        let old_path = self.path.clone();
        self.path = Some(path.to_string());

        let source =
            std::fs::read_to_string(&path).map_err(|err| Err(format!("{}: {}", path, err)))?;
        // info!("Parsing {}", path);
        let out = self.compile(&source);
        self.path = old_path;
        out
    }

    /// Turns a string of LDPL code into C++ code.
    pub fn compile(&mut self, source: &str) -> LDPLResult<()> {
        let ast = LDPLParser::parse(Rule::program, &source)?;
        self.compile_ast(ast)
    }

    /// Turns parsed LDPL code into C++ code.
    pub fn compile_ast(&mut self, ast: Pairs<Rule>) -> LDPLResult<()> {
        // Predeclared vars
        if self.globals.is_empty() {
            self.vars.push("ldpl_list<chText> VAR_ARGV;".into());
            self.globals
                .insert("ARGV".into(), LDPLType::List(Box::new(LDPLType::Text)));
            self.globals.insert("ERRORCODE".into(), LDPLType::Number);
            self.globals.insert("ERRORTEXT".into(), LDPLType::Text);
        }

        for pair in ast {
            match pair.as_rule() {
                Rule::header_stmt => self.compile_header(pair)?,
                Rule::data_section => {
                    let data = self.compile_data(pair, false)?;
                    self.vars.push(data);
                }
                Rule::EOI => break,

                Rule::procedure_section => {
                    for proc_stmt in pair.into_inner() {
                        match proc_stmt.as_rule() {
                            Rule::create_stmt_stmt => self.add_user_stmt(proc_stmt)?,
                            Rule::sub_def_stmt => {
                                let sub = self.compile_sub_def_stmt(proc_stmt)?;
                                self.subs.push(sub);
                            }
                            _ => {
                                indent!();
                                let stmt = self.compile_subproc_stmt(proc_stmt)?;
                                self.main.push(stmt);
                                dedent!();
                            }
                        }
                    }
                }

                _ => unexpected!(pair),
            }
        }

        Ok(())
    }

    /// Process INCLUDE, EXTENSION, and FLAGs in the header above
    /// the DATA: and PROCEDURE: sections.
    fn compile_header(&mut self, pair: Pair<Rule>) -> LDPLResult<()> {
        let stmt = pair.into_inner().next().unwrap();
        match stmt.as_rule() {
            Rule::include_stmt => {
                let file = stmt.into_inner().next().unwrap().as_str();
                self.load_and_compile(&self.expand_path(unquote(file)))?;
            }
            Rule::extension_stmt => {
                let ext_file = unquote(stmt.into_inner().next().unwrap().as_str());
                self.add_extension(self.expand_path(ext_file))?;
            }
            Rule::flag_stmt => {
                let flag = unquote(stmt.into_inner().next().unwrap().as_str());
                self.add_flag(flag.into())?;
            }
            Rule::using_stmt => {
                let name = stmt.into_inner().next().unwrap().as_str().to_lowercase();
                let mut path = format!("{}{}/{}.ldpl", LPM_LOCATION, name, name);
                if path.contains('~') {
                    path = path.replace("~", env!("HOME"));
                }
                self.load_and_compile(&path)?;
            }
            _ => unexpected!(stmt),
        }
        Ok(())
    }

    /// Convert `name IS TEXT` into a C++ variable declaration.
    /// Used by DATA: and LOCAL DATA: sections.
    fn compile_data(&mut self, pair: Pair<Rule>, local: bool) -> LDPLResult<String> {
        let mut out = vec![];

        for def in pair.into_inner() {
            let is_extern = def.as_rule() == Rule::external_type_def;

            let mut parts = def.into_inner();
            let ident = parts.next().unwrap().as_str();
            let typename = parts.next().unwrap().as_str();
            let varname = ident.to_uppercase();
            let mut var: String;

            if is_extern {
                self.extern_vars.insert(varname.clone(), true);
                var = format!("extern {} {}", compile_type(typename), mangle_extern(ident));
            } else {
                var = format!("{} {}", compile_type(typename), mangle_var(ident));
                if typename == "number" {
                    var.push_str(" = 0");
                } else if typename == "text" {
                    var.push_str(r#" = """#);
                }
            }

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
            out.push(emit_line!(var));
        }

        Ok(format!("{}\n", out.join("")))
    }

    /// Convert a param list into a vector of param types and a C++
    /// function signature params list.
    fn compile_params(&mut self, pair: Pair<Rule>) -> LDPLResult<(Vec<LDPLType>, String)> {
        let mut out = vec![];
        let mut types = vec![];

        for def in pair.into_inner() {
            assert!(def.as_rule() == Rule::type_def);
            let mut parts = def.into_inner();
            let ident = parts.next().unwrap().as_str();
            let typename = parts.next().unwrap().as_str();
            let typetype = LDPLType::from(typename);
            types.push(typetype.clone());
            self.locals.insert(ident.to_uppercase(), typetype);
            out.push(format!("{}& {}", compile_type(typename), mangle_var(ident)));
        }

        Ok((types, out.join(", ")))
    }

    /// Function definition.
    fn compile_sub_def_stmt(&mut self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let mut params = String::new();
        let mut param_types = vec![];
        let mut vars = String::new();
        let mut body: Vec<String> = vec![];
        let mut is_extern = false;
        let ident;

        self.locals.clear();
        self.in_sub = true;
        indent!();

        let first = iter.next().unwrap();
        if first.as_rule() == Rule::external {
            is_extern = true;
            ident = iter.next().unwrap().as_str();
        } else {
            ident = first.as_str();
        }

        let ident_upper = ident.to_uppercase();

        if self.defs.contains_key(&ident_upper) {
            return error!("Redefining existing SUB-PROCEDURE: {}", ident);
        }

        if self.expected_defs.contains_key(&ident_upper) {
            self.expected_defs.remove(&ident_upper);
        }

        let mut node = iter.next().unwrap();

        if node.as_rule() == Rule::sub_param_section {
            let (types, string) = self.compile_params(node)?;
            params = string;
            param_types = types;
            node = iter.next().unwrap();
        }

        if node.as_rule() == Rule::sub_data_section {
            vars = self.compile_data(node, true)?;
            node = iter.next().unwrap();
        }

        // done with the header, register this SUB so we
        // can call it recursively in the body.
        self.defs.insert(ident.to_uppercase(), param_types);

        loop {
            body.push(self.compile_subproc_stmt(node)?);
            let node_opt = iter.next();
            if node_opt.is_none() {
                break;
            } else {
                node = node_opt.unwrap();
            }
        }
        dedent!();
        self.in_sub = false;

        let mangled = if is_extern {
            mangle_extern(ident)
        } else {
            mangle_sub(ident)
        };

        emit!(
            "void {}({}) {{\n{}{}}}\n",
            mangled,
            params,
            vars,
            body.join(""),
        )
    }

    /// Read CREATE STATEMENT and add mapping as a user_stmt
    /// CREATE STATEMENT <text> EXECUTING <ident>
    fn add_user_stmt(&mut self, pair: Pair<Rule>) -> LDPLResult<()> {
        let mut iter = pair.into_inner();
        let stmt = unquote(iter.next().unwrap().as_str()).to_uppercase();
        let ident = iter.next().unwrap().as_str().to_uppercase();

        if !self.defs.contains_key(&ident) {
            return error!(
                "CREATE STATEMENT used with unknown sub-procedure: {}",
                ident
            );
        }

        if let Some(subs) = self.user_stmts.get_mut(&stmt) {
            subs.push(ident);
        } else {
            self.user_stmts.insert(stmt, vec![ident]);
        }

        Ok(())
    }

    /// Translate a user-defined STATEMENT into a SUB call.
    fn compile_user_stmt(&mut self, pair: Pair<Rule>) -> LDPLResult<String> {
        let iter = pair.into_inner();

        // we can't just take pair.as_str() because that returns the
        // trailing comments. maybe a pest bug?
        let stmt = iter
            .clone()
            .map(|part| part.as_str())
            .collect::<Vec<_>>()
            .join(" ")
            .to_uppercase();

        let types_iter = iter.clone(); // for inferring types of stmt parts
        let call_parts: Vec<_> = stmt.split(" ").map(|p| p.to_uppercase()).collect();
        let mut matched = false;
        let mut sub_name = String::new();

        // args is list of (index, type)
        let mut args: Vec<(usize, LDPLType)> = vec![];

        'outer: for (pattern, subs) in &self.user_stmts {
            let mut def_parts: Vec<_> = pattern.split(" ").collect();

            // don't bother if the patterns aren't the same length
            if def_parts.len() != call_parts.len() {
                continue;
            }

            let mut types_iter = types_iter.clone(); // re-use each loop
                                                     // compare each word in the pattern
            for (i, call_part) in call_parts.iter().enumerate() {
                let node = types_iter.next().unwrap();
                let def_part = def_parts.remove(0); // safe - we checked size
                if def_part == "$" {
                    args.push((i, self.scalar_type_of_expr(node)?.clone()));
                } else if *call_part != def_part {
                    continue 'outer;
                }
            }

            // if we got here, we may have found a match.
            // now we need to compare arity and param types to find
            // the specific sub-procedure to call.
            let call_params: Vec<LDPLType> = args.iter().map(|t| t.1.clone()).collect();
            for sub in subs {
                if let Some(sub_params) = self.defs.get(sub) {
                    if *sub_params == call_params {
                        sub_name = sub.clone();
                        matched = true;
                        break 'outer;
                    }
                }
            }

            // if we're here, we didn't find a match
            return error!(
                "Statement arguments didn't match any sub-procedures: {}",
                stmt
            );
        }

        if matched {
            let iter = iter
                .enumerate()
                .filter(|(i, _rule)| args.iter().any(|(idx, _)| idx == i))
                .map(|(_, rule)| rule);
            let (prefix, args) = self.compile_arg_list(iter)?;
            return Ok(format!(
                "{}\n{}",
                prefix,
                emit_line!("{}({});", mangle_sub(&sub_name), args)
            ));
        }

        error!("Unknown statement: {}", stmt)
    }

    /// Used in CALL and when calling user-defined statements.
    /// Wants either Pairs<Rule> or an Iterator you built of
    /// Rule::expr items exclusively. Like from an expr_list.
    ///
    /// Returns a tuple of (PREFIX, ARGS) where:
    /// - PREFIX is the setup code that should be inserted before a
    ///   function call is made.
    /// - ARGS are the list of args to be used in the function call,
    ///   as prepared by PREFIX.
    fn compile_arg_list<'p, I: Iterator<Item = Pair<'p, Rule>>>(
        &mut self,
        mut iter: I,
    ) -> LDPLResult<(String, String)> {
        let mut prefix = vec![];
        let mut args = vec![];

        while let Some(arg) = iter.next() {
            match arg.as_rule() {
                Rule::number => {
                    let var = format!("LPVAR_{}", self.tmp_id);
                    self.tmp_id += 1;
                    prefix.push(emit_line!(
                        "ldpl_number {} = {};",
                        var,
                        self.compile_expr(arg)?
                    ));
                    args.push(var);
                }
                Rule::text | Rule::linefeed => {
                    let var = format!("LPVAR_{}", self.tmp_id);
                    self.tmp_id += 1;
                    prefix.push(emit_line!("chText {} = {};", var, self.compile_expr(arg)?));
                    args.push(var);
                }
                Rule::var => args.push(self.compile_expr(arg)?),
                _ => unexpected!(arg),
            }
        }

        Ok((prefix.join(""), args.join(", ")))
    }

    /// Emit a stmt from the PROCEDURE: section of a file or function.
    fn compile_subproc_stmt(&mut self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut out = vec![];

        out.push(match pair.as_rule() {
            // control flow
            Rule::call_stmt => self.compile_call_stmt(pair)?,
            Rule::if_stmt => self.compile_if_stmt(pair)?,
            Rule::else_stmt => return error!("unexpected ELSE statement"),
            Rule::while_stmt => self.compile_while_stmt(pair)?,
            Rule::for_each_stmt => self.compile_for_each_stmt(pair)?,
            Rule::for_stmt => self.compile_for_stmt(pair)?,
            Rule::loop_kw_stmt => self.compile_loop_kw_stmt(pair)?,
            Rule::return_stmt => self.compile_return_stmt(pair)?,
            Rule::goto_stmt => self.compile_goto_stmt(pair)?,
            Rule::label_stmt => self.compile_label_stmt(pair)?,
            Rule::exit_stmt => self.compile_exit_stmt(pair)?,
            Rule::wait_stmt => self.compile_wait_stmt(pair)?,
            Rule::store_quote_stmt => self.compile_store_quote_stmt(pair)?,
            Rule::store_stmt => self.compile_store_stmt(pair)?,

            // math
            Rule::solve_stmt => self.compile_solve_stmt(pair)?,
            Rule::floor_stmt => self.compile_floor_stmt(pair)?,
            Rule::modulo_stmt => self.compile_modulo_stmt(pair)?,

            // text
            Rule::join_stmt => self.compile_join_stmt(pair)?,
            Rule::old_join_stmt => self.compile_old_join_stmt(pair)?,
            Rule::replace_stmt => self.compile_replace_stmt(pair)?,
            Rule::split_stmt => self.compile_split_stmt(pair)?,
            Rule::get_char_stmt => self.compile_get_char_stmt(pair)?,
            Rule::get_ascii_stmt => self.compile_get_ascii_stmt(pair)?,
            Rule::get_char_code_stmt => self.compile_get_char_code_stmt(pair)?,
            Rule::get_index_stmt => self.compile_get_index_stmt(pair)?,
            Rule::count_stmt => self.compile_count_stmt(pair)?,
            Rule::substr_stmt => self.compile_substring_stmt(pair)?,
            Rule::trim_stmt => self.compile_trim_stmt(pair)?,

            // list
            Rule::push_stmt => self.compile_push_stmt(pair)?,
            Rule::delete_stmt => self.compile_delete_stmt(pair)?,

            // map
            Rule::get_keys_count_stmt => self.compile_get_keys_count_stmt(pair)?,
            Rule::get_keys_stmt => self.compile_get_keys_stmt(pair)?,

            // list + map
            Rule::clear_stmt => self.compile_clear_stmt(pair)?,
            Rule::copy_stmt => self.compile_copy_stmt(pair)?,

            // list + text
            Rule::get_length_stmt => self.compile_get_length_stmt(pair)?,

            // io
            Rule::display_stmt => self.compile_display_stmt(pair)?,
            Rule::load_stmt => self.compile_load_stmt(pair)?,
            Rule::write_stmt => self.compile_write_stmt(pair)?,
            Rule::append_stmt => self.compile_append_stmt(pair)?,
            Rule::accept_stmt => self.compile_accept_stmt(pair)?,
            Rule::execute_stmt => self.compile_execute_stmt(pair)?,

            // user-defined statement (made via CREATE STATEMENT)
            Rule::user_stmt => self.compile_user_stmt(pair)?,

            _ => unexpected!(pair),
        });

        Ok(out.join(""))
    }

    ////
    // CONTROL FLOW

    /// STORE _ IN _
    fn compile_store_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();

        let expr = iter.next().unwrap();
        let var = iter.next().unwrap();
        let val = self.compile_expr_for_type(expr, self.type_of_var(var.clone())?)?;

        emit!("{} = {};", self.compile_var(var)?, val)
    }

    /// STORE QUOTE IN _
    fn compile_store_quote_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let var = self.compile_var(iter.next().unwrap())?;
        let txt = iter.next().unwrap().as_str();
        // remove extra preceeding \n from txt. parser limitation.
        if !txt.is_empty() {
            emit!(
                r#"{} = "{}";"#,
                var,
                &txt[1..].replace("\n", "\\\n\\n").replace("\"", "\\\"")
            )
        } else {
            emit!("{} = \"\";", var)
        }
    }

    /// RETURN
    fn compile_return_stmt(&self, _pair: Pair<Rule>) -> LDPLResult<String> {
        if !self.in_sub {
            return error!("RETURN can't be used outside of SUB-PROCEDURE");
        }
        emit!("return;")
    }

    /// BREAK / CONTINUE
    fn compile_loop_kw_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        if self.in_loop.is_empty() {
            return error!("{} can't be used without FOR/WHILE loop", pair.as_str());
        }
        emit!("{};", pair.as_str())
    }

    /// GOTO _
    fn compile_goto_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let label = pair.into_inner().next().unwrap();
        emit!("goto label_{};", mangle(label.as_str()))
    }

    /// LABEL _
    fn compile_label_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let label = pair.into_inner().next().unwrap();
        // no indenting
        Ok(format!("label_{}:\n", mangle(label.as_str())))
    }

    /// WAIT _ MILLISECONDS
    fn compile_wait_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let count = self.compile_expr(pair.into_inner().next().unwrap())?;
        emit!(
            "std::this_thread::sleep_for(std::chrono::milliseconds((long int){}));",
            count
        )
    }

    /// EXIT
    fn compile_exit_stmt(&self, _pair: Pair<Rule>) -> LDPLResult<String> {
        emit!("exit(0);")
    }

    /// CALL _ WITH _ ...
    fn compile_call_stmt(&mut self, pair: Pair<Rule>) -> LDPLResult<String> {
        let call_stmt = pair.into_inner().next().unwrap();
        let is_extern = call_stmt.as_rule() == Rule::call_external_stmt;
        let mut iter = call_stmt.into_inner();
        let ident = iter.next().unwrap().as_str();

        if !is_extern && !self.defs.contains_key(&ident.to_uppercase()) {
            self.expected_defs.insert(ident.to_uppercase(), true);
        }

        let (prefix, params) = self.compile_arg_list(iter)?;

        let mangled = if is_extern {
            mangle_extern(ident)
        } else {
            mangle_sub(ident)
        };

        let fwd_decl = format!("void {}();", mangled);
        if !self.forwards.contains(&fwd_decl) {
            self.forwards.push(format!("void {}();\n", mangled));
        }

        Ok(format!(
            "{}{}",
            prefix,
            emit_line!("{}({});", mangled, params)
        ))
    }

    /// IF and WHILE test statement
    fn compile_test_stmt(&self, test: Pair<Rule>) -> LDPLResult<String> {
        let mut out = vec![];
        match test.as_rule() {
            Rule::test_expr => {
                for t in test.into_inner() {
                    out.push(self.compile_test_stmt(t)?);
                }
            }
            Rule::or_test_expr => {
                let mut iter = test.into_inner();
                let left = self.compile_test_stmt(iter.next().unwrap())?;
                let right = self.compile_test_stmt(iter.next().unwrap())?;
                out.push(format!("({} || {})", left, right));
            }
            Rule::and_test_expr => {
                let mut iter = test.into_inner();
                let left = self.compile_test_stmt(iter.next().unwrap())?;
                let right = self.compile_test_stmt(iter.next().unwrap())?;
                out.push(format!("({} && {})", left, right));
            }
            Rule::one_test_expr => out.push(self.compile_test_expr(test)?),
            _ => unexpected!(test),
        }
        Ok(out.join(" "))
    }

    /// Single test expression. Use _stmt for expressions with OR / AND.
    fn compile_test_expr(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let left = self.compile_expr(iter.next().unwrap())?;
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
        let right = self.compile_expr(iter.next().unwrap())?;
        Ok(format!("({} {} {})", left, sign, right))
    }

    /// Coerce Number -> Text and Text -> Number.
    fn compile_expr_for_type(&self, expr: Pair<Rule>, typename: &LDPLType) -> LDPLResult<String> {
        let expr_type = self.type_of_expr(expr.clone())?;

        if typename.is_text() && expr.as_rule() == Rule::number {
            // 45 => "45"
            Ok(format!(r#""{}""#, self.compile_expr(expr)?))
        } else if typename.is_number() && (expr_type.is_text() || expr_type.is_text_collection()) {
            // "123" => to_number("123")
            Ok(format!("to_number({})", self.compile_expr(expr)?))
        } else if typename.is_text() && (expr_type.is_number() || expr_type.is_number_collection())
        {
            // txt_var => to_ldpl_string(txt_var)
            Ok(format!("to_ldpl_string({})", self.compile_expr(expr)?))
        } else {
            self.compile_expr(expr)
        }
    }

    /// Variable, Number, or Text
    fn compile_expr(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        Ok(match pair.as_rule() {
            Rule::var => self.compile_var(pair)?,
            Rule::number => self.compile_number(pair)?,
            Rule::text => pair.as_str().to_string(),
            Rule::linefeed => "\"\\n\"".to_string(),
            Rule::ident => self.mangle_var(pair.as_str()),
            _ => return error!("UNIMPLEMENTED: {:?}", pair),
        })
    }

    /// Normalize a number literal.
    /// Ex: -000.0 => 0
    fn compile_number(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let num = pair.as_str();
        if let Ok(parsed) = num.parse::<f64>() {
            Ok(parsed.to_string())
        } else {
            error!("Can't parse number: {}", num)
        }
    }

    /// Turn an ident/lookup pair into a C++ friendly ident.
    fn compile_var(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        assert!(
            pair.as_rule() == Rule::var,
            "Expected Rule::var, got {:?}",
            pair.as_rule()
        );

        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::ident => Ok(self.mangle_var(inner.as_str())),
            Rule::lookup => self.compile_lookup_from_iter(inner.into_inner()),
            _ => unexpected!(inner),
        }
    }

    /// Expects the iterator from an IDENT.into_inner() call.
    /// Knows the difference between a:b:1 where b is a container
    /// (a[b[1]]) and where b is a scalar (a[b][1]).
    fn compile_lookup_from_iter(&self, mut iter: Pairs<Rule>) -> LDPLResult<String> {
        let basevar = iter.next().unwrap();
        let mut parts = vec![self.compile_expr(basevar)?];
        let mut copy = iter.clone();
        while let Some(part) = iter.next() {
            // If it's an ident AND a variable AND a
            // container, then end this lookup and nest the
            // new one
            if part.as_rule() == Rule::ident {
                if let Ok(t) = self.type_of_var(part.clone()) {
                    if t.is_collection() {
                        parts.push(format!("[{}]", self.compile_lookup_from_iter(copy)?));
                        break;
                    }
                }
            }
            copy.next(); // copy should be 1 step behind iter, to
                         // capture the current variable

            // otherwise just keep adding index operations
            parts.push(format!("[{}]", self.compile_expr(part)?));
        }
        Ok(parts.join(""))
    }

    /// WHILE _ DO / REPEAT
    fn compile_while_stmt(&mut self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let test = iter.next().unwrap();
        let test = self.compile_test_stmt(test)?;

        self.in_loop.push(true);
        let mut body = vec![];
        indent!();
        for node in iter {
            body.push(self.compile_subproc_stmt(node)?);
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
    fn compile_if_stmt(&mut self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let test = iter.next().unwrap();
        let test = self.compile_test_stmt(test)?;

        let mut body = vec![];
        indent!();
        for node in iter {
            match node.as_rule() {
                Rule::else_stmt => body.push(self.compile_else_stmt(node)?),
                _ => body.push(self.compile_subproc_stmt(node)?),
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
    fn compile_else_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();

        let test = if let Some(test_expr) = iter.next() {
            Some(self.compile_test_stmt(test_expr)?)
        } else {
            None
        };

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
    fn compile_for_stmt(&mut self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let var = mangle_var(iter.next().unwrap().as_str());
        let from = self.compile_expr(iter.next().unwrap())?;
        let to = self.compile_expr(iter.next().unwrap())?;
        let step = self.compile_expr(iter.next().unwrap())?;

        self.in_loop.push(true);
        indent!();
        let mut body = vec![];
        for node in iter {
            body.push(self.compile_subproc_stmt(node)?);
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
    fn compile_for_each_stmt(&mut self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let ident = mangle_var(iter.next().unwrap().as_str());
        let collection = iter.next().unwrap();

        let range_var = format!("RVAR_{}", self.tmp_id);
        self.tmp_id += 1;

        let method = if self.type_of_expr(collection.clone())?.is_map() {
            ".second"
        } else {
            ""
        };

        self.in_loop.push(true);
        indent!();
        let mut body = vec![emit_line!("{} = {}{};", ident, range_var, method)];
        for node in iter {
            body.push(self.compile_subproc_stmt(node)?);
        }
        dedent!();
        self.in_loop.pop();

        Ok(format!(
            "{}{}{}",
            emit_line!(
                "for (auto& {} : {}.inner_collection) {{",
                range_var,
                self.compile_expr(collection)?
            ),
            body.join(""),
            emit_line!("}")
        ))
    }

    ////
    // ARITHMETIC

    /// MODULO _ BY _ IN _
    fn compile_modulo_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let base = self.compile_expr(iter.next().unwrap())?;
        let by = self.compile_expr(iter.next().unwrap())?;
        let var = self.compile_var(iter.next().unwrap())?;

        emit!("{} = modulo({}, {});", var, base, by)
    }

    /// FLOOR _
    /// FLOOR _ IN _
    /// TODO: only FLOOR _ in 4.4
    fn compile_floor_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let stmt = pair.into_inner().next().unwrap();
        let rule = stmt.as_rule();
        let mut iter = stmt.into_inner();
        let left = self.compile_expr(iter.next().unwrap())?;
        let mut right = left.clone();
        match rule {
            Rule::floor_in_stmt => right = self.compile_var(iter.next().unwrap())?,
            Rule::floor_mut_stmt => {}
            _ => unexpected!(rule),
        }

        emit!("{} = floor({});", left, right)
    }

    /// IN _ SOLVE X
    fn compile_solve_stmt(&mut self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let ident = iter.next().unwrap();

        emit!(
            "{} = {};",
            self.compile_var(ident)?,
            self.compile_solve_expr(iter.next().unwrap())?
        )
    }

    // Math expression part of a SOLVE statement
    fn compile_solve_expr(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut parts = vec![];

        for part in pair.into_inner() {
            match part.as_rule() {
                Rule::var | Rule::number | Rule::text => parts.push(self.compile_expr(part)?),
                Rule::solve_expr => parts.push(self.compile_solve_expr(part)?),
                Rule::math_op => parts.push(part.as_str().to_string()),
                _ => return error!("unexpected rule: {:?}", part),
            }
        }

        Ok(parts.join(" "))
    }

    ////
    // TEXT

    /// SPLIT _ BY _ IN _
    fn compile_split_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let text = self.compile_expr(iter.next().unwrap())?;
        let splitter = self.compile_expr(iter.next().unwrap())?;
        let var = self.compile_var(iter.next().unwrap())?;
        emit!("{} = utf8_split_list({}, {});", var, text, splitter)
    }

    /// REPLACE _ FROM _ WITH _ IN _
    /// replace_stmt = { ^"REPLACE" ~ expr ~ ^"FROM" ~ expr ~ ^"WITH" ~ expr ~ ^"IN" ~ var }
    fn compile_replace_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let search = self.compile_expr(iter.next().unwrap())?;
        let text = self.compile_expr(iter.next().unwrap())?;
        let replacement = self.compile_expr(iter.next().unwrap())?;
        let var = self.compile_var(iter.next().unwrap())?;

        emit!("{} = str_replace(((chText){}).str_rep(), ((chText){}).str_rep(), ((chText){}).str_rep());",
            var, text, search, replacement)
    }

    /// IN _ JOIN _ _...
    fn compile_join_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let var = self.compile_var(iter.next().unwrap())?;

        let mut out = vec![emit_line!(r#"joinvar = "";"#)];
        for expr in iter {
            out.push(emit_line!(
                "join(joinvar, {}, joinvar);",
                self.compile_expr_for_type(expr, &LDPLType::Text)?
            ));
        }
        out.push(emit_line!("{} = joinvar;", var));

        Ok(format!("{}", out.join("")))
    }

    /// JOIN _ AND _ IN _
    fn compile_old_join_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let left = self.compile_expr_for_type(iter.next().unwrap(), &LDPLType::Text)?;
        let right = self.compile_expr_for_type(iter.next().unwrap(), &LDPLType::Text)?;
        let var = self.compile_var(iter.next().unwrap())?;

        emit!("join({}, {}, {});", left, right, var)
    }

    /// TRIM _ IN _
    fn compile_trim_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let expr = self.compile_expr(iter.next().unwrap())?;
        let var = self.compile_var(iter.next().unwrap())?;
        emit!("{} = trimCopy({});", var, expr)
    }

    /// COUNT _ FROM _ IN _
    fn compile_count_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let search = self.compile_expr(iter.next().unwrap())?;
        let text = self.compile_expr(iter.next().unwrap())?;
        let var = self.compile_var(iter.next().unwrap())?;
        emit!("{} = utf8Count({}, {});", var, text, search)
    }

    /// SUBSTRING _ FROM _ LENGTH _ IN _
    fn compile_substring_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let text = self.compile_expr(iter.next().unwrap())?;
        let search = self.compile_expr(iter.next().unwrap())?;
        let length = self.compile_expr(iter.next().unwrap())?;
        let var = self.compile_var(iter.next().unwrap())?;

        Ok(format!(
            "{}{}",
            emit_line!("joinvar = {};", text),
            emit_line!("{} = joinvar.substr({}, {});", var, search, length)
        ))
    }

    /// GET INDEX OF _ FROM _ IN _
    fn compile_get_index_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let search = self.compile_expr(iter.next().unwrap())?;
        let text = self.compile_expr(iter.next().unwrap())?;
        let var = self.compile_var(iter.next().unwrap())?;
        emit!("{} = utf8GetIndexOf({}, {});", var, text, search)
    }

    /// GET CHARACTER CODE OF _ IN _
    fn compile_get_char_code_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let expr = self.compile_expr(iter.next().unwrap())?;
        let var = self.compile_var(iter.next().unwrap())?;
        emit!("{} = get_char_num({});", var, expr)
    }

    /// GET ASCII CHARACTER _ IN _
    fn compile_get_ascii_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let chr = self.compile_expr(iter.next().unwrap())?;
        let var = self.compile_var(iter.next().unwrap())?;
        emit!("{} = (char)({});", var, chr)
    }

    /// GET CHARACTER AT _ FROM _ IN _
    fn compile_get_char_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let at = self.compile_expr(iter.next().unwrap())?;
        let from = self.compile_expr(iter.next().unwrap())?;
        let var = self.compile_var(iter.next().unwrap())?;
        emit!("{} = charat({}, {});", var, from, at)
    }

    ////
    // LIST + TEXT

    // GET LENGTH OF _ IN _
    fn compile_get_length_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let expr = iter.next().unwrap();
        let var = self.compile_var(iter.next().unwrap())?;
        let expr_type = self.type_of_expr(expr.clone())?;
        let expr = self.compile_expr(expr)?;

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
    fn compile_push_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let expr = self.compile_expr(iter.next().unwrap())?;
        let list = self.compile_var(iter.next().unwrap())?;
        emit!("{}.inner_collection.push_back({});", list, expr)
    }

    /// DELETE LAST ELEMENT OF _
    fn compile_delete_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let list = self.compile_var(iter.next().unwrap())?;
        emit!(format!(
            "if({list}.inner_collection.size() > 0) {list}.inner_collection.pop_back();",
            list = list
        ))
    }

    ////
    // MAP

    /// GET KEYS COUNT OF _ IN _
    fn compile_get_keys_count_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let map = self.compile_expr(iter.next().unwrap())?;
        let var = self.compile_var(iter.next().unwrap())?;
        emit!("{} = {}.inner_collection.size();", var, map)
    }

    /// GET KEYS OF _ IN _
    fn compile_get_keys_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let map = self.compile_expr(iter.next().unwrap())?;
        let var = self.compile_var(iter.next().unwrap())?;
        emit!("get_indices({}, {});", var, map)
    }

    ////
    // MAP + LIST

    /// COPY _ TO _
    fn compile_copy_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let from = self.compile_expr(iter.next().unwrap())?;
        let to = self.compile_var(iter.next().unwrap())?;
        emit!("{}.inner_collection = {}.inner_collection;", to, from)
    }

    /// CLEAR _
    fn compile_clear_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let collection = self.compile_var(iter.next().unwrap())?;
        emit!("{}.inner_collection.clear();", collection)
    }

    ////
    // IO

    /// DISPLAY _...
    fn compile_display_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut parts = vec!["cout".to_string()];
        for node in pair.into_inner() {
            parts.push(self.compile_expr(node)?);
        }
        parts.push("flush".into());
        emit!("{};", parts.join(" << "))
    }

    /// ACCEPT _
    /// ACCEPT _ UNTIL EOF
    fn compile_accept_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let stmt = pair.into_inner().next().unwrap();

        let eof = stmt.as_rule() == Rule::accept_eof_stmt;
        let ident = stmt.into_inner().next().unwrap();
        let vartype = self.type_of_var(ident.clone())?;

        let fun = if eof {
            "input_until_eof()"
        } else if vartype.is_text() {
            "input_string()"
        } else if vartype.is_number() {
            "input_number()"
        } else {
            unexpected!(ident);
        };

        emit!("{} = {};", self.compile_var(ident)?, fun)
    }

    /// LOAD FILE _ IN _
    fn compile_load_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let path = self.compile_expr(iter.next().unwrap())?;
        let var = self.compile_var(iter.next().unwrap())?;
        emit!("load_file({}, {});", path, var)
    }

    /// WRITE _ TO FILE _
    fn compile_write_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let expr = self.compile_expr(iter.next().unwrap())?;
        let path = self.compile_expr(iter.next().unwrap())?;

        Ok(format!("{}{}{}",
            emit_line!("file_writing_stream.open(expandHomeDirectory(((chText){}).str_rep()), ios_base::out);", path),
            emit_line!("file_writing_stream << {};", expr),
            emit_line!("file_writing_stream.close();")
        ))
    }

    /// APPEND _ TO FILE _
    fn compile_append_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let mut iter = pair.into_inner();
        let expr = self.compile_expr(iter.next().unwrap())?;
        let path = self.compile_expr(iter.next().unwrap())?;

        Ok(format!("{}{}{}",
            emit_line!("file_writing_stream.open(expandHomeDirectory(((chText){}).str_rep()), ios_base::app);", path),
            emit_line!("file_writing_stream << {};", expr),
            emit_line!("file_writing_stream.close();")
        ))
    }

    /// EXECUTE _
    /// EXECUTE _ AND STORE EXIT CODE IN _
    /// EXECUTE _ AND STORE OUTPUT IN _
    fn compile_execute_stmt(&self, pair: Pair<Rule>) -> LDPLResult<String> {
        let pair = pair.into_inner().next().unwrap();
        let rule = pair.as_rule();
        let mut iter = pair.into_inner();
        match rule {
            Rule::execute_expr_stmt => emit!(
                "system({});",
                self.compile_c_char_array(iter.next().unwrap())?
            ),
            Rule::execute_output_stmt => {
                let expr = self.compile_c_char_array(iter.next().unwrap())?;
                let var = self.compile_var(iter.next().unwrap())?;
                emit!("{} = exec({});", var, expr)
            }
            Rule::execute_exit_code_stmt => {
                let expr = self.compile_c_char_array(iter.next().unwrap())?;
                let var = self.compile_var(iter.next().unwrap())?;
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

impl Compiler {
    /// Find the scalar type for an expression. Map(List(Text)) will
    /// just give us Text.
    fn scalar_type_of_expr(&self, expr: Pair<Rule>) -> LDPLResult<LDPLType> {
        if let Ok(t) = self.type_of_expr(expr.clone()) {
            self.scalar_type_of_collection(t.clone())
        } else {
            error!("Can't infer scalar type for expression: {:?}", expr)
        }
    }

    /// Map(List(Text)) => Text
    fn scalar_type_of_collection<'a>(&self, t: LDPLType) -> LDPLResult<LDPLType> {
        Ok(match t {
            LDPLType::List(inner) | LDPLType::Map(inner) => {
                return self.scalar_type_of_collection(*inner);
            }
            _ => t,
        })
    }

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
                    error!("No type found for {}", var.as_str())
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

    /// Expand a relative file path into a full one, based on the
    /// current file we're compiling.
    fn expand_path(&self, file: &str) -> String {
        if let Some(current) = &self.path {
            if let Some(cwd) = std::path::Path::new(current).parent() {
                let root = cwd.to_string_lossy();
                if !root.is_empty() {
                    return format!("{}/{}", root, file);
                }
            }
        }

        file.to_string()
    }

    /// Like the freestanding mangle_var(), but also works with
    /// external variables. Use this when you want to reference a
    /// variable that can be either global, local, or external.
    fn mangle_var(&self, ident: &str) -> String {
        let ident = ident.to_uppercase();
        if self.extern_vars.contains_key(&ident) {
            mangle_extern(&ident)
        } else {
            mangle_var(&ident)
        }
    }

    /// Compile TEXT as a c char array, mostly for EXECUTE and friends.
    fn compile_c_char_array(&self, node: Pair<Rule>) -> LDPLResult<String> {
        Ok(match node.as_rule() {
            Rule::var => format!("{}.str_rep().c_str()", mangle_var(node.as_str())),
            Rule::text => node.as_str().to_string(),
            _ => unexpected!(node),
        })
    }
}

/// LDPL Type => C++ Type
fn compile_type(ldpl_type: &str) -> &str {
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
    format!("SUBPR_{}", mangle(ident))
}

/// Convert an ident to a C++-friendly ident by stripping illegal
/// characters and whatnot.
/// https://docs.ldpl-lang.org/naming/
/// Based on `fix_identifier()` in ldpl.cpp
fn mangle(ident: &str) -> String {
    let mut mangled = String::with_capacity(ident.len() + 10);

    for c in ident.to_uppercase().chars() {
        if c.is_alphanumeric() || c == '_' {
            mangled.push(c);
        } else {
            mangled.push_str(&format!("c{}_", c as u16));
        }
    }

    mangled
}

/// External functions have simpler conversion rules.
/// http://docs.ldpl-lang.org/naming/#external-identifier-naming-schemes
fn mangle_extern(ident: &str) -> String {
    let mut mangled = String::with_capacity(ident.len() + 10);

    for c in ident.to_uppercase().chars() {
        if c.is_alphanumeric() || c == '_' {
            mangled.push(c);
        } else {
            mangled.push('_');
        }
    }

    mangled.to_uppercase()
}

/// Remove "quotes" from a literal text string.
fn unquote(text: &str) -> &str {
    &text[1..text.len() - 1]
}
