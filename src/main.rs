use ldpl::{builder, compiler, LDPLResult};
use std::{
    path::Path,
    process::{Command, Stdio},
};

const DEFAULT_COMMAND: &str = "build";

/// Print error message to the console.
macro_rules! error {
        ($msg:expr) => {{
            eprintln!("\x1b[91;1mLDPL Error: \x1b[0m{}", $msg.to_string().replace("Error: ", "").trim());
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
                println!("\x1b[95;1m*\x1b[0m {}", $msg);
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
    let mut file = String::new();
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
    let mut includes = vec![];

    while !args.is_empty() {
        let arg = args.remove(0);
        match arg.as_ref() {
            "-h" | "--help" | "-help" | "help" => {
                print_usage();
                return Ok(());
            }
            "-v" | "--version" | "-version" | "version" => {
                print_version();
                return Ok(());
            }
            "print" | "-r" => command = "print",
            "-o" => {
                if args.is_empty() {
                    error!("binary name expected.");
                }
                bin = args.remove(0);
            }
            "-i" => includes.push(args.remove(0)),
            "-f" | "-c" => todo!(),
            "build" => command = "build",
            "run" => command = "run",
            _ if arg.starts_with('-') => error!("Unknown flag {}", arg),
            _ => file = arg,
        }
    }

    quiet = command != "build";

    if file.is_empty() && !args.is_empty() {
        file = args.remove(0);
    } else if file.is_empty() {
        error!("filename expected.");
    }

    info!("Loading {}", file);
    let path = Path::new(&file);
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
    let mut compiler = compiler::new();
    if !includes.is_empty() {
        for file in includes {
            compiler.load_and_compile(&file)?;
        }
    }
    compiler.load_and_compile(&file)?;

    if command == "print" {
        println!("{}", compiler);
        return Ok(());
    }

    info!("Building {}", bin);
    builder::build(&compiler.to_string(), Some(&bin))?;
    info!("Saved as {}", bin);
    success!("File(s) compiled successfully.");

    if command == "run" {
        info!("Running {}", bin);
        let bin = if bin.starts_with('/') || bin.starts_with('.') {
            bin
        } else {
            format!("./{}", bin)
        };
        let mut cmd = Command::new(bin)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .spawn()?;
        cmd.wait()?;
    }

    Ok(())
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
    print       Print compiled C++ code. (same as -r)
    build       Compile binary. (default)
    run         Run binary after building.
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
