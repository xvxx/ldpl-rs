pub use pest::Parser;

#[derive(Parser)]
#[grammar = "parser/ldpl.pest"]
pub struct LDPLParser;
