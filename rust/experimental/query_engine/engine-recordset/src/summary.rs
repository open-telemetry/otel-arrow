use std::{cell::RefCell, collections::HashMap, mem::replace, time::SystemTime};

use crate::{
    data_expressions::{SummaryReservoir, SummaryWindow},
    expression::Hasher,
    primitives::AnyValue,
};

pub(crate) struct Summaries {
    lookup: RefCell<HashMap<Box<str>, usize>>,
    summary_data: RefCell<Vec<SummaryState>>,
}

impl Summaries {
    pub fn new() -> Summaries {
        Self {
            lookup: RefCell::new(HashMap::new()),
            summary_data: RefCell::new(Vec::new()),
        }
    }

    pub fn register_summary(&self, mut summary: Summary) -> SummaryInfo {
        let mut lookup = self.lookup.borrow_mut();

        let id = &summary.id;

        let summary_index = lookup.get_mut(id);

        if summary_index.is_some() {
            let index = summary_index.unwrap();
            let mut borrow = self.summary_data.borrow_mut();
            let state = borrow.get_mut(*index).expect("Summary could not be found");

            state.expected_externally_included_record_count += summary.included_count as usize;
            state.summary.total_count += summary.total_count;

            return SummaryInfo {
                summary_index: *index,
                total_count: state.summary.total_count,
                existed: true,
            };
        }

        let mut summary_data = self.summary_data.borrow_mut();

        let index = summary_data.len();

        let old = lookup.insert(id.clone(), index);

        debug_assert!(old.is_none());

        let total_count = summary.total_count;
        let expected_externally_included_record_count = summary.included_count as usize;
        summary.included_count = 0;

        let state = SummaryState::new(
            summary,
            expected_externally_included_record_count,
            Vec::new(),
            Vec::new(),
        );

        summary_data.push(state);

        SummaryInfo {
            summary_index: index,
            total_count,
            existed: false,
        }
    }

    pub fn create_or_update_summary(
        &self,
        data_record_index: usize,
        summary_lookup: SummaryLookup,
    ) -> SummaryResult {
        let mut lookup = self.lookup.borrow_mut();

        let id = summary_lookup.get_id();

        let summary_index = lookup.get_mut(&id);

        if summary_index.is_some() {
            let index = summary_index.unwrap();
            let mut borrow = self.summary_data.borrow_mut();
            let state = borrow.get_mut(*index).expect("Summary could not be found");

            state.summary.total_count += 1;

            let summary_info = SummaryInfo {
                summary_index: *index,
                total_count: state.summary.total_count,
                existed: true,
            };

            let reservoir_size = match &summary_lookup.reservoir_type {
                SummaryReservoir::SimpleReservoir(s) => *s as usize,
            };

            if state.included_data_records.len() < reservoir_size {
                state.included_data_records.push(data_record_index);
                state.summary.included_count += 1;

                return SummaryResult::Include(summary_info);
            } else {
                let r = rand::random_range(0..summary_info.total_count) as usize;
                if r < reservoir_size {
                    let old_data_record_index = replace(
                        &mut state.included_data_records[r as usize],
                        data_record_index,
                    );
                    state
                        .data_records_dropped_after_being_included
                        .push(old_data_record_index);
                    return SummaryResult::Include(summary_info);
                } else {
                    return SummaryResult::Drop(summary_info);
                }
            }
        }

        let mut summary_data = self.summary_data.borrow_mut();

        let reservoir_size = match &summary_lookup.reservoir_type {
            SummaryReservoir::SimpleReservoir(s) => *s,
        };

        let mut state = SummaryState::new(
            Summary::new(
                Some(id.clone()),
                SystemTime::now(),
                summary_lookup.window_type,
                summary_lookup.window_start,
                summary_lookup.window_end,
                summary_lookup.reservoir_type,
                summary_lookup.grouping,
                0,
                1,
            ),
            0,
            Vec::new(),
            Vec::new(),
        );

        let index = summary_data.len();

        let old = lookup.insert(id, index);

        debug_assert!(old.is_none());

        let summary_info = SummaryInfo {
            summary_index: index,
            total_count: 1,
            existed: false,
        };

        let result;
        if reservoir_size > 0 {
            state.included_data_records.push(data_record_index);

            state.summary.included_count = 1;

            result = SummaryResult::Include(summary_info);
        } else {
            result = SummaryResult::Drop(summary_info);
        }

        summary_data.push(state);

        result
    }

    pub fn include_in_summary(&self, summary_index: usize) -> usize {
        let mut borrow = self.summary_data.borrow_mut();
        let state = borrow
            .get_mut(summary_index)
            .expect("Summary could not be found");

        state.externally_included_record_count += 1;

        state.externally_included_record_count
    }

    pub fn get_summary_index(&self, summary_id: &str) -> Option<usize> {
        self.lookup.borrow().get(summary_id).copied()
    }

    pub fn get_summary<F>(&self, summary_index: usize, action: F)
    where
        F: FnOnce(Option<&Summary>),
    {
        let borrow = self.summary_data.borrow();

        let state = borrow.get(summary_index);

        if let Some(summary) = state {
            action(Some(&summary.summary))
        } else {
            action(None)
        }
    }

    pub fn get_summaries(self) -> Vec<SummaryState> {
        self.summary_data.take()
    }
}

pub(crate) enum SummaryResult {
    Include(SummaryInfo),
    Drop(SummaryInfo),
}

#[derive(Debug)]
pub(crate) struct SummaryInfo {
    summary_index: usize,
    total_count: u32,
    existed: bool,
}

impl SummaryInfo {
    pub fn get_summary_index(&self) -> usize {
        self.summary_index
    }

    pub fn get_existed(&self) -> bool {
        self.existed
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct SummaryLookup {
    window_type: SummaryWindow,
    window_start: i64,
    window_end: i64,
    reservoir_type: SummaryReservoir,
    grouping: Vec<SummaryGroupKeyValue>,
}

impl SummaryLookup {
    pub fn new(
        window_type: SummaryWindow,
        window_start: i64,
        window_end: i64,
        reservoir_type: SummaryReservoir,
        grouping: Vec<SummaryGroupKeyValue>,
    ) -> SummaryLookup {
        Self {
            window_type,
            window_start,
            window_end,
            reservoir_type,
            grouping,
        }
    }

    pub fn get_id(&self) -> Box<str> {
        Summary::generate_id(
            &self.window_type,
            self.window_start,
            self.window_end,
            &self.grouping,
        )
    }
}

#[derive(Debug, Clone)]
pub struct Summary {
    id: Box<str>,
    observed_timestamp: SystemTime,
    window_type: SummaryWindow,
    window_start: i64,
    window_end: i64,
    reservoir_type: SummaryReservoir,
    grouping: Vec<SummaryGroupKeyValue>,
    included_count: u32,
    total_count: u32,
}

impl Summary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Option<Box<str>>,
        observed_timestamp: SystemTime,
        window_type: SummaryWindow,
        window_start: i64,
        window_end: i64,
        reservoir_type: SummaryReservoir,
        grouping: Vec<SummaryGroupKeyValue>,
        included_count: u32,
        total_count: u32,
    ) -> Summary {
        let id_value = if let Some(v) = id {
            v
        } else {
            Summary::generate_id(&window_type, window_start, window_end, &grouping)
        };

        Self {
            id: id_value,
            observed_timestamp,
            window_type,
            window_start,
            window_end,
            reservoir_type,
            grouping,
            included_count,
            total_count,
        }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_observed_timestamp(&self) -> &SystemTime {
        &self.observed_timestamp
    }

    pub fn get_window_type(&self) -> &SummaryWindow {
        &self.window_type
    }

    pub fn get_window_start(&self) -> i64 {
        self.window_start
    }

    pub fn get_window_end(&self) -> i64 {
        self.window_end
    }

    pub fn get_reservoir_type(&self) -> &SummaryReservoir {
        &self.reservoir_type
    }

    pub fn get_grouping(&self) -> &Vec<SummaryGroupKeyValue> {
        &self.grouping
    }

    pub fn get_included_count(&self) -> u32 {
        self.included_count
    }

    pub fn get_total_count(&self) -> u32 {
        self.total_count
    }

    pub(crate) fn add_externally_included_record_count(
        &mut self,
        externally_included_record_count: usize,
    ) {
        self.included_count += externally_included_record_count as u32;
    }

    pub(crate) fn generate_id(
        window_type: &SummaryWindow,
        window_start: i64,
        window_end: i64,
        grouping: &Vec<SummaryGroupKeyValue>,
    ) -> Box<str> {
        let mut hasher = Hasher::new();

        window_type.add_hash_bytes(&mut hasher);

        hasher.add_bytes(&window_start.to_le_bytes());
        hasher.add_bytes(&window_end.to_le_bytes());

        for group in grouping {
            if group.key.name.is_none() {
                hasher.add_bytes(&[0]);
            } else {
                hasher.add_bytes(group.key.name.as_ref().unwrap().as_bytes());
            }
            hasher.add_bytes(group.key.path.as_bytes());

            match &group.value {
                SummaryGroupValue::StringValue(string_value) => {
                    hasher.add_bytes(&[1]);
                    hasher.add_bytes(string_value.as_bytes());
                }
                SummaryGroupValue::LongValue(long_value) => {
                    hasher.add_bytes(&[2]);
                    hasher.add_bytes(long_value);
                }
                SummaryGroupValue::DoubleValue(double_value) => {
                    hasher.add_bytes(&[3]);
                    hasher.add_bytes(double_value);
                }
                SummaryGroupValue::NullValue => hasher.add_bytes(&[0]),
            }
        }

        hasher.into()
    }
}

pub(crate) struct SummaryState {
    summary: Summary,
    expected_externally_included_record_count: usize,
    externally_included_record_count: usize,
    included_data_records: Vec<usize>,
    data_records_dropped_after_being_included: Vec<usize>,
}

impl SummaryState {
    pub fn get_summary(&self) -> &Summary {
        &self.summary
    }

    pub fn get_summary_mut(&mut self) -> &mut Summary {
        &mut self.summary
    }

    pub fn get_expected_externally_included_record_count(&self) -> usize {
        self.expected_externally_included_record_count
    }

    pub fn get_externally_included_record_count(&self) -> usize {
        self.externally_included_record_count
    }

    pub fn get_included_data_record_count(&self) -> usize {
        self.included_data_records.len()
    }

    pub fn get_included_data_records(&self) -> &Vec<usize> {
        &self.included_data_records
    }

    pub fn get_data_records_dropped_after_being_included(&self) -> &Vec<usize> {
        &self.data_records_dropped_after_being_included
    }
}

impl From<SummaryState> for Summary {
    fn from(val: SummaryState) -> Self {
        val.summary
    }
}

impl SummaryState {
    pub fn new(
        summary: Summary,
        expected_externally_included_record_count: usize,
        included_data_records: Vec<usize>,
        data_records_dropped_after_being_included: Vec<usize>,
    ) -> SummaryState {
        Self {
            summary,
            expected_externally_included_record_count,
            externally_included_record_count: 0,
            included_data_records,
            data_records_dropped_after_being_included,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SummaryGroupKeyValue {
    key: SummaryGroupKey,
    value: SummaryGroupValue,
}

impl SummaryGroupKeyValue {
    pub fn new(key: SummaryGroupKey, value: SummaryGroupValue) -> SummaryGroupKeyValue {
        Self { key, value }
    }

    pub fn get_key(&self) -> &SummaryGroupKey {
        &self.key
    }

    pub fn get_value(&self) -> &SummaryGroupValue {
        &self.value
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SummaryGroupKey {
    name: Option<Box<str>>,
    path: Box<str>,
}

impl SummaryGroupKey {
    pub fn new(name: Option<&str>, path: &str) -> SummaryGroupKey {
        Self {
            name: either!(name.is_none() => None; Some(name.unwrap().into())),
            path: path.into(),
        }
    }

    pub fn get_name(&self) -> Option<&str> {
        self.name.as_ref().map(|v| v.as_ref())
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SummaryGroupValue {
    StringValue(Box<str>),
    LongValue([u8; 8]),
    DoubleValue([u8; 8]),
    NullValue,
}

impl SummaryGroupValue {
    pub fn new_from_any_value(any_value: &AnyValue) -> SummaryGroupValue {
        if let AnyValue::LongValue(long_value) = any_value {
            SummaryGroupValue::LongValue(long_value.get_value().to_le_bytes())
        } else if let AnyValue::DoubleValue(double_value) = any_value {
            return SummaryGroupValue::DoubleValue(double_value.get_value().to_le_bytes());
        } else if let AnyValue::StringValue(string_value) = any_value {
            return SummaryGroupValue::StringValue(string_value.get_value().into());
        } else {
            let mut group = SummaryGroupValue::NullValue;

            any_value.as_string_value(|r| {
                if let Some(string_value) = r {
                    group = SummaryGroupValue::StringValue(string_value.into())
                }
            });

            return group;
        }
    }

    pub fn to_string_value(&self) -> Option<&str> {
        if let SummaryGroupValue::StringValue(s) = self {
            return Some(s);
        }

        None
    }

    pub fn to_long_value(&self) -> Option<i64> {
        if let SummaryGroupValue::LongValue(b) = self {
            return Some(i64::from_le_bytes(*b));
        }

        None
    }

    pub fn to_double_value(&self) -> Option<f64> {
        if let SummaryGroupValue::DoubleValue(b) = self {
            return Some(f64::from_le_bytes(*b));
        }

        None
    }

    pub fn is_null_value(&self) -> bool {
        if let SummaryGroupValue::NullValue = self {
            return true;
        }

        false
    }
}
