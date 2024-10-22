use ldpl::{compiler, LDPLResult};
use std::{
    io::{self, Read},
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
    let mut outfile = None;
    let mut includes = vec![];
    let mut ext_includes = vec![];
    let mut ext_flags = vec![];
    let mut stdin = String::new();

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
                outfile = Some(args.remove(0));
            }
            "-i" => {
                if args.is_empty() {
                    error!("filename to include expected.");
                }
                let file = args.remove(0);
                if file.ends_with(".ldpl") || file.ends_with(".lsc") {
                    includes.push(file);
                } else {
                    ext_includes.push(file);
                }
            }
            "-f" => {
                if args.is_empty() {
                    error!("flag expected.");
                }
                ext_flags.push(args.remove(0));
            }
            "-c" => {
                if let Err(error) = io::stdin().read_to_string(&mut stdin) {
                    error!("Error reading STDIN: {}", error);
                }
            }
            "build" => command = "build",
            "run" => command = "run",
            _ if arg.starts_with('-') => error!("Unknown flag {}", arg),
            _ => file = arg,
        }
    }

    quiet = command != "build";

    if stdin.is_empty() {
        if file.is_empty() && !args.is_empty() {
            file = args.remove(0);
        } else if file.is_empty() {
            error!("filename expected.");
        }
    }

    info!("Compiling {}", file);
    let mut compiler = compiler::new();
    if !includes.is_empty() {
        for file in includes {
            compiler.load_and_compile(&file)?;
        }
    }
    for flag in ext_flags {
        compiler.add_flag(flag)?;
    }
    for ext in ext_includes {
        compiler.add_extension(ext)?;
    }
    if stdin.is_empty() {
        compiler.load_and_compile(&file)?;
    } else {
        compiler.compile(&stdin)?;
    }

    if command == "print" {
        println!("{}", compiler);
        return Ok(());
    }

    info!("Building {}", file);
    let bin = compiler.build(&file, outfile)?;
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
