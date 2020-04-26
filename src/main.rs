use ldpl::{
    compiler, emitter,
    parser::{LDPLParser, Parser, Rule},
};
use pest::iterators::Pairs;

const DEFAULT_COMMAND: &str = "build";

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

    let source = std::fs::read_to_string(path)?;

    let ast = LDPLParser::parse(Rule::program, &source).unwrap();
    if command == "parse" {
        print_ast(ast);
        return Ok(());
    }

    if command == "check" {
        todo!();
    }

    let cpp = emitter::emit(ast)?;
    if command == "emit" {
        println!("{}", cpp);
        return Ok(());
    }

    compiler::compile(&cpp, None)?;
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
