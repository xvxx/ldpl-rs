use ldpl::parser::{LDPLParser, Parser, Rule};

// parse whole program
macro_rules! parse {
    ($e:expr) => {
        LDPLParser::parse(Rule::program, &$e).unwrap()
    };
}

// parse into a single node/pair
macro_rules! parse_one {
    ($e:expr) => {
        parse!($e).nth(0).unwrap()
    };
}

// expect an error
macro_rules! parse_err {
    ($e:expr) => {
        LDPLParser::parse(Rule::program, &$e).unwrap_err()
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

    let node = parse_one!("+10213");
    assert_eq!(Rule::number, node.as_rule());
    assert_eq!("+10213", node.as_str());

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

#[test]
fn test_lookup() {
    let node = parse_one!("abc:5");
    assert_eq!(Rule::var, node.as_rule());
    assert_eq!("abc:5", node.as_str());

    let node = parse_one!(r#"person:"name""#);
    assert_eq!(Rule::var, node.as_rule());
    assert_eq!(r#"person:"name""#, node.as_str());

    let node = parse_one!(r#"people:500"#);
    assert_eq!(Rule::var, node.as_rule());
    assert_eq!(r#"people:500"#, node.as_str());

    let node = parse_one!(r#"nested:50:20:30"#);
    assert_eq!(Rule::var, node.as_rule());
    assert_eq!(r#"nested:50:20:30"#, node.as_str());

    let node = parse_one!(r#"nested-map:"ages":20:"year""#);
    assert_eq!(Rule::var, node.as_rule());
    assert_eq!(r#"nested-map:"ages":20:"year""#, node.as_str());
    let mut parts = vec!["nested-map", "\"ages\"", "20", "\"year\""];
    for lookup in node.into_inner() {
        for part in lookup.into_inner() {
            assert_eq!(part.as_str(), parts.remove(0));
        }
    }
}

#[test]
fn test_var() {
    let node = parse_one!(r#"somevar"#);
    assert_eq!(Rule::var, node.as_rule());
    assert_eq!("somevar", node.as_str());

    let node = parse_one!(r#"some-other-var"#);
    assert_eq!(Rule::var, node.as_rule());
    assert_eq!("some-other-var", node.as_str());

    let node = parse_one!(r#"dots.in.var.name"#);
    assert_eq!(Rule::var, node.as_rule());
    assert_eq!("dots.in.var.name", node.as_str());

    let node = parse_one!(r#"_start_with_underbar"#);
    assert_eq!(Rule::var, node.as_rule());
    assert_eq!("_start_with_underbar", node.as_str());

    let err = parse_err!(r#".cant_start_with_dot"#);
    assert_eq!(err.to_string(), " --> 1:1\n  |\n1 | .cant_start_with_dot\n  | ^---\n  |\n  = expected EOI, number, text, or var".to_string());
}
