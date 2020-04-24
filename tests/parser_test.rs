use ldpl::parser::{LDPLParser, Parser, Rule};

macro_rules! parse {
    ($e:expr) => {
        LDPLParser::parse(Rule::program, &$e).unwrap()
    };
}

macro_rules! parse_one {
    ($e:expr) => {
        parse!($e).nth(0).unwrap()
    };
}

#[test]
fn test_number() {
    let node = parse_one!("3.14");
    assert_eq!(Rule::number, node.as_rule());
    assert_eq!("3.14", node.as_str());

    let node = parse_one!("314");
    assert_eq!(Rule::number, node.as_rule());
    assert_eq!("314", node.as_str());

    let node = parse_one!("-12051205.0325035");
    assert_eq!(Rule::number, node.as_rule());
    assert_eq!("-12051205.0325035", node.as_str());
}

#[test]
fn test_text() {
    let node = parse_one!(r#""hiya""#);
    assert_eq!(Rule::text, node.as_rule());
    assert_eq!(r#""hiya""#, node.as_str());

    let node = parse_one!(r#""spaces, too""#);
    assert_eq!(Rule::text, node.as_rule());
    assert_eq!(r#""spaces, too""#, node.as_str());

    let node = parse_one!(r#""and \"inner strings\" too""#);
    assert_eq!(Rule::text, node.as_rule());
    assert_eq!(r#""and \"inner strings\" too""#, node.as_str());
}
