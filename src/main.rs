use ldpl::{
    compiler, emitter,
    parser::{LDPLParser, Parser, Rule},
    LDPLResult,
};
use pest::iterators::Pairs;
use std::path::Path;

const DEFAULT_COMMAND: &str = "build";

/// Print error message to the console.
macro_rules! error {
        ($msg:expr) => {{
            eprintln!("\x1b[91;1mLDPL Error: {}\x1b[0m", $msg.to_string().replace("Error: ", ""));
            std::process::exit(1);
        }};
        ($fmt:expr, $($args:expr),*) => {
            error!(format!($fmt, $($args),*));
        };
    }

fn main() {
    if let Err(e) = run() {
        error!(e);
    }
}

fn run() -> LDPLResult<()> {
    let quiet: bool;
    let args = std::env::args().skip(1).collect::<Vec<String>>();

    if args.is_empty() {
        print_usage();
        return Ok(());
    }

    /// Print info message to the console.
    macro_rules! info {
        ($msg:expr) => {
            if !quiet {
                println!("\x1b[93;1m*\x1b[0m {}", $msg);
            }
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

    let mut command = DEFAULT_COMMAND;
    let mut path = "";
    let mut bin = String::new();

    // split args on = so -o=file is the same as -o file
    let mut new_args = vec![];
    for arg in args {
        if arg.contains('=') {
            for part in arg.split("=") {
                new_args.push(part.to_string());
            }
        } else {
            new_args.push(arg);
        }
    }
    let mut args = new_args;

    match args[0].as_ref() {
        "-h" | "--help" | "-help" | "help" => {
            print_usage();
            return Ok(());
        }
        "-v" | "--version" | "-version" | "version" => {
            print_version();
            return Ok(());
        }
        "emit" | "-r" => command = "emit",
        "-o" => {
            args.remove(0);
            if args.is_empty() {
                error!("binary name expected.");
            }
            bin = args[0].clone();
        }
        "parse" => command = "parse",
        "check" => command = "check",
        _ => path = &args[0],
    }

    quiet = command != "build";

    if path.is_empty() {
        path = &args.get(1).unwrap_or_else(|| error!("filename expected."));
    }

    info!("Loading {}", path);
    let source = std::fs::read_to_string(path)?;

    info!("Parsing {}", path);
    let ast = LDPLParser::parse(Rule::program, &source)
        .unwrap_or_else(|e| error!(format!("{} failed to parse.\n\x1b[0m{}", path, e)));
    if command == "parse" {
        print_ast(ast);
        return Ok(());
    }

    if command == "check" {
        todo!();
    }

    let path = Path::new(path);
    let bin = if bin.is_empty() {
        format!(
            "{}/{}",
            path.parent()
                .and_then(|d| Some(d.to_string_lossy()))
                .unwrap_or(".".into()),
            path.file_stem()
                .and_then(|f| Some(format!("{}-bin", f.to_string_lossy())))
                .unwrap_or("ldpl-output-bin".into())
        )
        .trim_matches('/')
        .into()
    } else {
        bin
    };

    info!("Compiling {}", bin);
    let cpp = emitter::emit(ast)?;
    if command == "emit" {
        println!("{}", cpp);
        return Ok(());
    }

    info!("Building {}", bin);
    compiler::compile(&cpp, Some(&bin))?;
    info!("Saved as {}", bin);

    success!("File(s) compiled successfully.");
    Ok(())
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

fn print_version() {
    println!(
        "ldpl-rs v{version} ({built})",
        built = ldpl::BUILD_DATE,
        version = ldpl::VERSION
    );
}

fn print_usage() {
    print!("\n\x1b[95;1mUsage:\x1b[0m");
    println!(
        r#"
    ldpl-rs [options] <command> <file.ldpl>
    ldpl-rs [-i='<included file>']... <source file>|-c
            [-o='<output name>'|-r] [-f='<c++ flag>']... [-n]
    ldpl-rs [-v|-h]
"#
    );
    print!("\x1b[95;1mCommands:\x1b[0m");
    println!(
        r#"
    parse       Parse and print syntax tree.
    check       Check for errors only.
    emit        Print C++ code. (same as -r)
    build       Compile binary. (default)
"#
    );
    print!("\x1b[95;1mOptions:\x1b[0m");
    print!(
        r#"
    -v --version             Display LDPL version information
    -h --help                Display this information
    -r                       Display generated C++ code
    -o=<name>                Set output file for compiled binary
    -i=<file>                Include file in current compilation
    -f=<flag>                Pass a flag to the C++ compiler
    -c                       Compile from standard input
"#,
    );
    println!(
        "
  Documentation for LDPL can be found at \x1b[96;1mhttps://docs.ldpl-lang.org\x1b[0m
  "
    );
}
