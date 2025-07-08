#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Array,
    Boolean,
    Integer,
    DateTime,
    Double,
    Map,
    Regex,
    String,
}
