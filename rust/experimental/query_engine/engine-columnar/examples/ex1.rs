use data_engine_kql_parser::{KqlParser, Parser, ParserOptions};

fn main() {
    // let kql_expr = "logs | where logs.severity_text == \"INFO\"";
    let kql_expr = "logs | where logs.attributes[\"X\"] == \"Y\"";

    let parser_options = ParserOptions::default();
    let pipeline_expr = KqlParser::parse_with_options(kql_expr, parser_options).unwrap();

    println!("{:#?}", pipeline_expr);
    println!("{}", pipeline_expr);
}
