//! The Compiler wraps your C++ compiler and builds it.

use crate::LDPLResult;
use std::{fs, path::Path, process::Command};

/// Runs the local C++ compiler and builds a binary.
pub fn compile(cpp_code: &str, outfile: Option<&str>) -> LDPLResult<()> {
    let filename = "ldpl-temp.cpp";
    if Path::new(filename).exists() {
        fs::remove_file(filename)?;
    }
    fs::write(filename, cpp_code)?;

    let target = outfile.unwrap_or("ldpl-output-bin");

    let cmd = Command::new("c++")
        .arg("ldpl-temp.cpp")
        .arg("-std=gnu++11")
        .arg("-w")
        .arg("-o")
        .arg(target)
        .output();

    fs::remove_file(filename)?;

    println!("status: {:?}", cmd?.stdout);

    Ok(())
}