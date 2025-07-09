use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "ottl.pest"]
#[allow(dead_code)]
pub(crate) struct OttlPestParser;
