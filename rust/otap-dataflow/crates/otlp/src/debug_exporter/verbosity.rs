/// Enum that allows the user to specify how much information they want displayeds
#[derive(Debug, Clone, PartialEq)]
pub enum Verbosity {
    /// display the most detailed information available
    Detailed,
    /// display the basic amount of information available and some detail about each request
    Normal,
    /// just display number of logs, metrics, traces, profiles received with some additional detail about samples/datapoints
    Basic,
}
