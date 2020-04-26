use ldpl::{
    compiler, emitter,
    parser::{LDPLParser, Parser, Rule},
};
use pest::iterators::Pairs;
use std::path::Path;

const DEFAULT_COMMAND: &str = "build";

/// Print info message to the console.
macro_rules! info {
    ($msg:expr) => {
        println!("\x1b[93;1m*\x1b[0m {}", $msg)
    };
    ($fmt:expr, $($args:expr),*) => {
        info!(format!($fmt, $($args),*));
    };
}

/// Print info message in green.
macro_rules! success {
    ($msg:expr) => {
        info!("\x1b[92;1m{}\x1b[0m", $msg)
    };
    ($fmt:expr, $($args:expr),*) => {
        success!(format!($fmt, $($args),*));
    };
}

fn main() -> Result<(), std::io::Error> {
    let args = std::env::args().skip(1).collect::<Vec<String>>();

    if args.is_empty() {
        print_usage();
        return Ok(());
    }

    match args[0].as_ref() {
        "-h" | "--help" | "-help" | "help" => {
            print_usage();
            return Ok(());
        }
        "-v" | "--version" | "-version" | "version" => {
            print_version();
            return Ok(());
        }
        _ => {}
    }

    let mut command = DEFAULT_COMMAND;
    let mut path = &args[0];
    if args.len() >= 2 {
        command = &args[0];
        path = &args[1];
    };

    info!("Loading {}", path);
    let source = std::fs::read_to_string(path)?;

    info!("Parsing {}", path);
    let ast = LDPLParser::parse(Rule::program, &source).unwrap();
    if command == "parse" {
        print_ast(ast);
        return Ok(());
    }

    if command == "check" {
        todo!();
    }

    let path = Path::new(path);
    let bin = path
        .file_stem()
        .and_then(|f| Some(format!("{}-bin", f.to_string_lossy())))
        .unwrap_or("ldpl-output-bin".into());
    let bin = format!(
        "{}/{}",
        path.parent()
            .and_then(|d| Some(d.to_string_lossy()))
            .unwrap_or(".".into()),
        bin
    );

    info!("Compiling {}", bin);
    let cpp = emitter::emit(ast)?;
    if command == "emit" {
        println!("{}", cpp);
        return Ok(());
    }

    info!("Building {}", bin);
    compiler::compile(&cpp, None)?;
    info!("Saved as {}", bin);

    success!("File(s) compiled successfully.");
    Ok(())
}

fn print_usage() {
    print!(
        r#"Usage: ldpl-rs [COMMAND] <file.ldpl>

Commands:
  parse       Parse and print syntax tree.
  check       Check for compile errors only.
  emit        Print C++ code.
  build       Compile binary. (default)
"#
    );
}

fn print_version() {
    println!(
        "ldpl-rs v{version} ({built})",
        built = ldpl::BUILD_DATE,
        version = ldpl::VERSION
    );
}

/// Print parsed AST
fn print_ast(ast: Pairs<Rule>) {
    for pair in ast {
        // A pair is a combination of the rule which matched and a span of input
        println!("Rule:    {:?}", pair.as_rule());
        println!("Span:    {:?}", pair.as_span());
        println!("Text:    {}", pair.as_str());
    }
}
