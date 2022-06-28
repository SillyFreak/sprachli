lalrpop_mod!(sprachli, "/grammar/sprachli.rs");

pub use sprachli::*;

#[test]
fn sprachli() {
    assert!(ExprParser::new().parse("22").is_ok());
    assert!(ExprParser::new().parse("(22)").is_ok());
    assert!(ExprParser::new().parse("((((22))))").is_ok());
    assert!(ExprParser::new().parse("((22)").is_err());
}
