use ldpl::parser::{LDPLParser, Parser, Rule};

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        eprintln!("ERROR: Please provide an LDPL file.");
        std::process::exit(1);
    }
    let file = args.get(1).unwrap();
    let code = std::fs::read_to_string(file).unwrap();

    let pairs = LDPLParser::parse(Rule::program, &code).unwrap();
    for pair in pairs {
        // A pair is a combination of the rule which matched and a span of input
        println!("Rule:    {:?}", pair.as_rule());
        println!("Span:    {:?}", pair.as_span());
        println!("Text:    {}", pair.as_str());
    }
}
