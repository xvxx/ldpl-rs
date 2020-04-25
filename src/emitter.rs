//! The Emitter generates a String of C++ code from parsed LDPL code.

use crate::{parser::Rule, LDPLResult};
use pest::iterators::Pairs;

struct Emitter<'i> {
    out: String,
    ast: Pairs<'i, Rule>,
}

pub fn emit(ast: Pairs<Rule>) -> LDPLResult<String> {
    let emitter = Emitter::new(ast);
    emitter.run()?;
    Ok(emitter.out)
}

impl<'i> Emitter<'i> {
    fn new(ast: Pairs<Rule>) -> Emitter {
        Emitter {
            ast,
            out: String::new(),
        }
    }

    fn run(&self) -> LDPLResult<()> {
        Ok(())
    }
}
