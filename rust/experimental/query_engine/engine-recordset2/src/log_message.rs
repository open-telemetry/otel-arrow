use data_engine_expressions::Expression;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum LogLevel {
    Verbose = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl LogLevel {
    pub fn get_name(&self) -> &str {
        match self {
            LogLevel::Verbose => "Verbose",
            LogLevel::Info => "Info",
            LogLevel::Warn => "Warn",
            LogLevel::Error => "Error",
        }
    }
}

#[derive(Debug)]
pub struct LogMessage<'a> {
    log_level: LogLevel,
    expression: &'a dyn Expression,
    message: String,
}

impl<'a> LogMessage<'a> {
    pub fn new(
        log_level: LogLevel,
        expression: &'a dyn Expression,
        message: String,
    ) -> LogMessage<'a> {
        Self {
            log_level,
            expression,
            message,
        }
    }

    pub fn get_log_level(&self) -> LogLevel {
        self.log_level.clone()
    }

    pub fn get_expression(&self) -> &dyn Expression {
        self.expression
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }
}
