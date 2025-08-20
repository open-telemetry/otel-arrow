#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Array,
    Boolean,
    DateTime,
    Double,
    Integer,
    Map,
    Null,
    Regex,
    String,
    TimeSpan,
}
