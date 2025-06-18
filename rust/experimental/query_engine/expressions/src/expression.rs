use std::hash::{Hash, Hasher};

pub trait Expression {
    fn get_query_location(&self) -> &QueryLocation;
}

#[derive(Debug, Clone, Eq)]
pub struct QueryLocation {
    start: usize,
    end: usize,
    line_number: usize,
    column_number: usize,
    fake: bool,
}

impl QueryLocation {
    pub fn new(
        start: usize,
        end: usize,
        line_number: usize,
        column_number: usize,
    ) -> QueryLocation {
        Self {
            start,
            end,
            line_number,
            column_number,
            fake: false,
        }
    }

    pub fn new_fake() -> QueryLocation {
        Self {
            start: 0,
            end: 0,
            line_number: 1,
            column_number: 1,
            fake: true,
        }
    }

    pub fn get_start_and_end_positions(&self) -> (usize, usize) {
        (self.start, self.end)
    }

    pub fn get_line_and_column_numbers(&self) -> (usize, usize) {
        (self.line_number, self.column_number)
    }
}

impl Hash for QueryLocation {
    fn hash<H: Hasher>(&self, _: &mut H) {}
}

impl PartialEq for QueryLocation {
    fn eq(&self, other: &Self) -> bool {
        if self.fake || other.fake {
            return true;
        }

        self.start == other.start
            && self.end == other.end
            && self.line_number == other.line_number
            && self.column_number == other.column_number
    }
}
