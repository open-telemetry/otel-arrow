pub trait Expression {
    fn get_query_location(&self) -> &QueryLocation;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryLocation {
    start: usize,
    end: usize,
    line_number: usize,
    column_number: usize
}

impl QueryLocation {
    pub fn new(start: usize, end: usize, line_number: usize, column_number: usize) -> QueryLocation {
        Self { start, end, line_number, column_number }
    }

    pub fn get_start_and_end_positions(&self) -> (usize, usize) {
        (self.start, self.end)
    }

    pub fn get_line_and_column_numbers(&self) -> (usize, usize) {
        (self.line_number, self.column_number)
    }
}