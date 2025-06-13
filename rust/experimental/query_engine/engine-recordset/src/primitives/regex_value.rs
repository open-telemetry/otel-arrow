use regex::Regex;

use crate::{error::Error, expression::*};

#[derive(Debug, Clone)]
pub struct RegexValueData {
    regex: Regex,
}

impl RegexValueData {
    pub fn new_from_string(value: &str) -> Result<RegexValueData, Error> {
        let result = Regex::new(value);
        if let Err(error) = result {
            return Err(Error::RegexParseError(error));
        }

        Ok(Self {
            regex: result.unwrap(),
        })
    }

    pub fn new_from_regex(regex: Regex) -> RegexValueData {
        Self { regex }
    }

    pub fn get_regex(&self) -> &regex::Regex {
        &self.regex
    }

    pub fn get_pattern(&self) -> &str {
        self.regex.as_str()
    }

    pub(crate) fn add_hash_bytes(&self, hasher: &mut Hasher) {
        hasher.add_bytes(self.get_pattern().as_bytes());
    }
}
