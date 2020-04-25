pub use pest::Parser;

#[derive(Parser)]
#[grammar = "ldpl.pest"]
pub struct LDPLParser;
