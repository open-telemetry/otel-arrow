use std::collections::HashSet;

pub struct ParserState {
    pub default_source_map_key: Option<Box<str>>,
    pub attached_data_names: HashSet<Box<str>>,
    pub variable_names: HashSet<Box<str>>,
}

impl ParserState {
    pub fn new() -> ParserState {
        Self {
            default_source_map_key: None,
            attached_data_names: HashSet::new(),
            variable_names: HashSet::new(),
        }
    }

    pub fn with_default_source_map_key_name(mut self, name: &str) -> ParserState {
        if !name.is_empty() {
            self.default_source_map_key = Some(name.into());
        }

        self
    }

    pub fn with_attached_data_names(mut self, names: &[&str]) -> ParserState {
        for name in names {
            self.attached_data_names.insert((*name).into());
        }

        self
    }

    pub fn push_variable_name(&mut self, name: &str) {
        self.variable_names.insert(name.into());
    }
}