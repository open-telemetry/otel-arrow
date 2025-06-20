use std::collections::HashSet;

use data_engine_expressions::QueryLocation;

pub struct ParserState<'a> {
    query: &'a str,
    default_source_map_key: Option<Box<str>>,
    attached_data_names: HashSet<Box<str>>,
    variable_names: HashSet<Box<str>>,
}

impl<'a> ParserState<'a> {
    pub fn new(query: &'a str) -> ParserState<'a> {
        Self {
            query,
            default_source_map_key: None,
            attached_data_names: HashSet::new(),
            variable_names: HashSet::new(),
        }
    }

    pub fn with_default_source_map_key_name(mut self, name: &str) -> ParserState<'a> {
        if !name.is_empty() {
            self.default_source_map_key = Some(name.into());
        }

        self
    }

    pub fn with_attached_data_names(mut self, names: &[&str]) -> ParserState<'a> {
        for name in names {
            self.attached_data_names.insert((*name).into());
        }

        self
    }

    pub fn get_query(&self) -> &str {
        self.query
    }

    pub fn get_query_slice(&self, query_location: &QueryLocation) -> &str {
        let (start, end) = query_location.get_start_and_end_positions();

        &self.query[start..end]
    }

    pub fn get_default_source_map_key(&self) -> Option<&str> {
        self.default_source_map_key.as_ref().map(|f| f.as_ref())
    }

    pub fn is_well_defined_identifier(&self, name: &str) -> bool {
        name == "source" || self.is_attached_data_defined(name) || self.is_variable_defined(name)
    }

    pub fn is_attached_data_defined(&self, name: &str) -> bool {
        self.attached_data_names.contains(name)
    }

    pub fn is_variable_defined(&self, name: &str) -> bool {
        self.variable_names.contains(name)
    }

    pub fn push_variable_name(&mut self, name: &str) {
        self.variable_names.insert(name.into());
    }
}
