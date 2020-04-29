//! The Builder wraps your C++ compiler and builds the final program.

use crate::{compiler::Compiler, LDPLResult};
use std::{fs, path::Path, process::Command, str};

impl Compiler {
    /// Run the local C++ compiler and build a binary.
    /// Returns the name of the built binary.
    pub fn build(&self, path: &str, outfile: Option<String>) -> LDPLResult<String> {
        let path = Path::new(&path);
        let target = if outfile.is_none() {
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
            .to_string()
        } else {
            outfile.unwrap().to_string()
        };

        let filename = "ldpl-temp.cpp";
        if Path::new(filename).exists() {
            fs::remove_file(filename)?;
        }
        fs::write(filename, self.to_string())?;

        let mut cmd = Command::new("c++");
        let mut cmd = cmd
            .arg("ldpl-temp.cpp")
            .arg("-std=gnu++11")
            .arg("-w")
            .arg("-o")
            .arg(&target);

        if !self.exts.is_empty() {
            for ext in &self.exts {
                cmd = cmd.arg(ext);
            }
        }

        // run command
        let cmd = cmd.output();

        fs::remove_file(filename)?;

        let output = cmd?;
        if !output.stderr.is_empty() {
            return error!(
                "C++ Error compiling {}: \n{}",
                filename,
                str::from_utf8(&output.stderr).unwrap_or("UTF-8 Error in C++ output")
            );
        }

        Ok(target)
    }
}
