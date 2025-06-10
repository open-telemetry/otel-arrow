use core::fmt;
use std::fmt::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

use sha2::{Digest, Sha256};

use crate::execution_context::ExecutionContext;

pub(crate) fn get_next_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);

    return COUNTER.fetch_add(1, Ordering::Relaxed);
}

pub(crate) struct Hasher {
    hasher: Sha256,
}

impl Hasher {
    pub fn new() -> Hasher {
        Self {
            hasher: Sha256::new(),
        }
    }

    pub fn add_bytes(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }
}

impl Into<ExpressionHash> for Hasher {
    fn into(self) -> ExpressionHash {
        let hash = self.hasher.finalize();

        let bytes = &hash[..];

        return ExpressionHash {
            bytes: bytes.into(),
            hex: hex::encode(bytes).into(),
        };
    }
}

impl Into<Box<str>> for Hasher {
    fn into(self) -> Box<str> {
        let hash = self.hasher.finalize();

        let bytes = &hash[..];

        return hex::encode(bytes).into();
    }
}

#[derive(Debug)]
pub(crate) struct ExpressionHash {
    bytes: Box<[u8]>,
    hex: Box<str>,
}

impl ExpressionHash {
    pub fn new<F>(build: F) -> ExpressionHash
    where
        F: FnOnce(&mut Hasher),
    {
        let mut hasher = Hasher::new();

        build(&mut hasher);

        hasher.into()
    }

    pub fn get_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn get_hex(&self) -> &str {
        &self.hex
    }
}

pub(crate) trait Expression: fmt::Debug {
    fn get_id(&self) -> usize;

    fn get_hash(&self) -> &ExpressionHash;

    fn write_debug(
        &self,
        execution_context: &dyn ExecutionContext,
        heading: &'static str,
        level: i32,
        output: &mut String,
    );
}

#[derive(Debug)]
pub(crate) enum ExpressionMessage {
    Info(ExpressionMessageData),
    Warn(ExpressionMessageData),
    Err(ExpressionMessageData),
}

#[derive(Debug)]
pub(crate) struct ExpressionMessageData {
    scope: Option<String>,
    message: String,
}

impl ExpressionMessage {
    pub fn add_scope(&mut self, scope: &str) {
        match self {
            ExpressionMessage::Info(expression_message_data) => {
                expression_message_data.scope = Some(scope.to_string())
            }
            ExpressionMessage::Warn(expression_message_data) => {
                expression_message_data.scope = Some(scope.to_string())
            }
            ExpressionMessage::Err(expression_message_data) => {
                expression_message_data.scope = Some(scope.to_string())
            }
        }
    }

    pub fn info(message: String) -> ExpressionMessage {
        ExpressionMessage::Info(ExpressionMessageData {
            scope: None,
            message,
        })
    }

    pub fn warn(message: String) -> ExpressionMessage {
        ExpressionMessage::Warn(ExpressionMessageData {
            scope: None,
            message,
        })
    }

    pub fn err(message: String) -> ExpressionMessage {
        ExpressionMessage::Err(ExpressionMessageData {
            scope: None,
            message,
        })
    }

    pub fn write_debug_comment(&self, output: &mut String) {
        let result = match self {
            ExpressionMessage::Info(m) => ("Info", &m.scope, &m.message),
            ExpressionMessage::Warn(m) => ("Warn", &m.scope, &m.message),
            ExpressionMessage::Err(m) => ("Error", &m.scope, &m.message),
        };

        if !result.1.is_none() {
            write!(
                output,
                "// [{}] {}: {}\n",
                result.1.as_ref().unwrap(),
                result.0,
                result.2
            );
        } else {
            write!(output, "// {}: {}\n", result.0, result.2);
        }
    }
}
