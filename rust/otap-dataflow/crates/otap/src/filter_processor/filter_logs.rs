// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the log filtering logic for the filter processor
//! 

pub struct LogFilter {
    // Include match properties describe logs that should be included in the Collector Service pipeline,
	// all other logs should be dropped from further processing.
	// If both Include and Exclude are specified, Include filtering occurs first.
    include: LogMatchProperties,
    	// Exclude match properties describe logs that should be excluded from the Collector Service pipeline,
	// all other logs should be included.
	// If both Include and Exclude are specified, Include filtering occurs first.
    exclude: LogMatchProperties,
    // LogConditions is a list of OTTL conditions for an ottllog context.
	// If any condition resolves to true, the log event will be dropped.
	// Supports `and`, `or`, and `()`
    log_record: Vec<String>,
}

// LogMatchProperties specifies the set of properties in a log to match against and the
// type of string pattern matching to use.
pub struct LogMatchProperties {
	// LogMatchType specifies the type of matching desired
    match_type: LogMatchType,

	// ResourceAttributes defines a list of possible resource attributes to match logs against.
	// A match occurs if any resource attribute matches all expressions in this given list.
    resource_attributes: Vec<KeyValue>,

	// RecordAttributes defines a list of possible record attributes to match logs against.
	// A match occurs if any record attribute matches at least one expression in this given list.
    record_attributes: Vec<KeyValue>,

	// SeverityTexts is a list of strings that the LogRecord's severity text field must match
	// against.
    severity_texts: Vec<String>,

	// SeverityNumberProperties defines how to match against a log record's SeverityNumber, if defined.
    severity_number: Option<LogServerityNumberMatchProperties>,

	// LogBodies is a list of strings that the LogRecord's body field must match
	// against.
    bodies: Vec<String>,
}

pub struct LogServerityNumberMatchProperties {
	// Min is the minimum severity needed for the log record to match.
	// This corresponds to the short names specified here:
	// https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/data-model.md#displaying-severity
	// this field is case-insensitive ("INFO" == "info")
	min: i32,
	// MatchUndefined lets logs records with "unknown" severity match.
	// If MinSeverity is not set, this field is ignored, as fields are not matched based on severity.
	match_undefined: bool,
}


pub enum LogMatchType {
    Strict, 
    Regexp
}

// ToDo if no filter is set then the filter must allow everything through, boolean array should be initialized to all true

impl LogFilter {
	pub fn filter(&self, logs_payload: OtapArrowRecord) -> Result<OtapArrowRecords, Error>{
		let (include_resource_attr_filter, include_log_record_filter, include_log_attr_filter) = self.include.create_filters(logs_payload, false)?;
		let (exclude_resource_attr_filter, exclude_log_record_filter, exclude_log_attr_filter) = self.exclude.create_filters(logs_payload, true)?;

		// combine the include and exclude filters
		let resource_attr_filter = arrow::compute::or_kleene(&include_resource_attr_filter, &exclude_resource_attr_filter).map_err()?;
		let log_record_filter = arrow::compute::or_kleene(&include_log_record_filter, &exclude_log_record_filter).map_err()?;
		let log_attr_filter = arrow::compute::or_kleene(&include_log_attr_filter, &exclude_log_attr_filter).map_err()?;

		let resource_attrs = logs_payload.get(ArrowPayLoadType::ResourceAttrs)?;
		let log_records = logs_payload.get(ArrowPayLoadType::Logs)?;
		let log_attrs = logs_payload.get(ArrowPayLoadType::LogAttrs)?;


        // HERE WE CLEAN UP THE TABLE WE EXTRACT THE IDS OF RECORDS WE ARE REMOVING AND MAKE SURE TO SYNC THESE REMOVALS WITH THE OTHER TABLES

        // we start with the resource_attr we get the id of the resource_attr that are being removed and use that to update the log_records filter
        // we want to remove the log_records the matching parent_id as the resource_attr that have been removed


        // create filters that we will use to get the rows that are getting removed
        // use these filters to get the ids of rows that are getting removed from the resource_attr
        let id_resource_attr_filter = arrow::compute::not(&resource_attr_filter).map_err()?;
        let resource_id_column = resource_attrs.column_by_name("parent_id").map_err()?;
        // we have a have a array contain all the parent_ids that will be removed

        // get ids that we being kept
        let resource_ids_filtered = arrow::compute::filter(&resource_id_column, resource_attr_filter).map_err()?;
        // get ids being removed
        let resource_ids_removed = arrow::compute::filter(&resource_id_column, id_resource_attr_filter).map_err()?;

        // update the log_records_filter to remove records that contain the parent_ids we found above
        let log_record_resource_id_column = log_records.column_by_name("resource_id").map_err()?;
        // init booleanarray here
        let log_record_resouce_id_filter;


        // TODO: we remove overlapping ids to only get the unique ids that are being removed, 
        //we can move this logic to the get...filter functions to make sure the ids all stick together 
        //that is parent_id with 0 should not both get removed or kept should be xor. In the get...filter functions
        // after we get a filter that extracts all the attributes we should get the ids and use that to build our final filter
        let resource_ids = resource_ids_removed - resource_ids_filtered;

        // build filter
        for id in resource_ids {
            let id_scaler = UInt16Array::new_scaler(id).map_err()?;
            let id_filter = arrow::compute::kernels::eq(&log_record_resource_id_column, &id_scaler);
            log_record_resource_id_filter = arrow::compute::or_kleene(&log_record_resource_id_filter, &id_filter);
        }
        // inverse because these are the ids we want to remove
        log_record_resource_id_filter = arrow::compute::not(&log_record_resource_id_filter);

        // combine filter with log record so now it will remove the log_records that shouldn't belong
        log_record_filter = arrow::compute::and(&log_record_filter, &log_record_resource_id_filter);

        // NOW WE CLEAN UP LOG_ATTR AND SCOPE_ATTR

        // invert filter to get all the removed rows
        let id_log_records_filter = arrow::compute::not(&log_record_filter).map_err()?;

        let log_record_id_column = log_records.column_by_name("id").map_err()?;

        // these are the ids we need to remove from the log_attr table
        let ids = arrow::compute::filter(&log_record_id_column, &id_log_records_filter);
        let log_attr_id_column = log_attr.column_by_name("parent_id");
        let log_attr_parent_id_filter;
        for id in ids() {
            let id_scaler = UInt16Array::new_scaler(id).map_err()?;
            let id_filter = arrow::compute::kernels::eq(&log_attr_id_column, &id_scaler).map_err()?;
            log_attr_parent_id_filter = arrow::compute::or_kleene(&log_attr_parent_id_filter, &id_filter).map_err()?;
        }
        log_attr_parent_id_filter = arrow::compute::not(&log_attr_parent_id_filter);
        log_attr_filter = arrow::compute::and(&log_attr_filter, &log_attr_parent_id_filter);



        let log_record_scope_id_column = log_records.column_by_name("scope_id").map_err()?;

        // here we need to also get the inverse and get the set difference to deal with overlapping scope_ids that are removed and kept
        let scope_ids = arrow::compute::filter(&log_record_scope_id_column, &id_log_records_filter);

        let scope_attr = logs_payload.get(ArrowPayLoadType::ScopeAttrs);
        let scope_attr_id_column = scope_attr.column_by_name("parent_id");
        let scope_attr_filter;
        for id in scope_ids() {
            let id_scaler = UInt16Array::new_scaler(id).map_err()?;
            let id_filter = arrow::compute::kernels::eq(&scope_attr_id_column, &id_scaler).map_err()?;
            scope_attr_filter = arrow::compute::or_kleene(&scope_attr_filter, &id_filter).map_err()?;
        }
        scope_attr_filter = arrow::compute::not(&scope_attr_filter).map_err()?;


        // apply filters to the logs
		let filtered_resource_attrs = arrow::compute::filter_record_batch(resource_attrs, &resource_attr_filter).map_err()?;
		let filtered_log_records = arrow::compute::filter_record_batch(log_record, &log_record_filter).map_err()?;
		let filtered_log_attrs = arrow::compute::filter_record_batch(log_attrs, &log_attr_filter).map_err()?;
        let filtered_scope_attrs = arrow::compute::filter_record_batch(scope_attrs, &scope_attr_filter).map_err()?;

        logs_payload.set(ArrowPayLoadType::ResourceAttrs, filtered_resource_attrs)?;
        logs_payload.set(ArrowPayLoadType::Logs, filtered_log_records)?;
        logs_payload.set(ArrowPayLoadType::LogAttrs, filtered_log_attrs)?;
        logs_payload.set(ArrowPayLoadType::ScopeAttrs, filtered_scope_attrs)?;


		Ok(logs_payload)
	}
}

impl LogMatchProperties {

    // define consts for column names we will use
    const STR: &str = "str";
    const INT: &str = "int";
    const DOUBLE: &str = "double";
    const BOOL: &str = "bool";
    const BYTES: &str = "bytes";
    const SER: &str = "ser";

    const BODY_STR: &str = "body_str";
    const BODY_INT: &str = "body_int";
    const BODY_DOUBLE: &str = "body_double";
    const BODY_BOOL: &str = "body_bool";
    const BODY_BYTES: &str = "body_bytes";
    const BODY_SER: &str = "body_ser";

    const KEY: &str = "key";
    const SEVERITY_NUMBER: &str = "severity_number";
    const SEVERITY_TEXT: &str = "severity_text";

    // TODO need to extend the filter creation to support regex match type
	fn get_resource_attr_filter(&self, logs_payload: &OtapArrowRecord) -> Result<BooleanArray, Error> {
		// get resource_attrs record batch
		let resource_attrs = logs_payload.get(ArrowPayLoadType::ResourceAttrs).map_err()?;

		// TODO need to init empty boolean array with right size here
		let mut filter;

		// generate the filter for this record_batch
		for attribute_filter in self.resource_attributes {
			let key_scaler = StringArray::new_scaler(attribute.key).map_err()?;
			let key_column = resource_attrs.column_by_name(self::KEY)?;
			let key_filter = arrow::compute::kernels::cmp::eq(&key_column, &key_scaler).map_err()?;
			let value_filter = match attribute_filter.value {
				MatchValue::String(value) => {
					// get string column
					let string_column = resource_attrs.column_by_name(self::STR)?;
					let value_scaler = StringArray::new_scaler(value).map_err()?;
					arrow::compute::kernels::cmp::eq(&string_column, &value_scaler).map_err()?
				}
				MatchValue::Int(value) => {
					let int_column = resource_attrs.column_by_name(self::INT)?;
					let value_scaler = Int64Array::new_scaler(value).map_err()?;
					arrow::compute::kernels::cmp::eq(&int_column, &value_scaler).map_err()?
				}
				MatchValue::Double(value) => {
					let double_column = resource_attrs.column_by_name(self::DOUBLE)?;
					let value_scaler = Float64Array::new_scaler(value).map_err()?;
					arrow::compute::kernels::cmp::eq(&double_column, &value_scaler).map_err()?
				}
				MatchValue::Boolean(value) => {
					let bool_column = resource_attrs.column_by_name(self::BOOL)?;
					let value_scaler = BooleanArray::new_scaler(value).map_err()?;
					arrow::compute::kernels::cmp::eq(&bool_column, &value_scaler).map_err()?
				}
				_ => {
					// need bytes and array
				}
			};
			// build filter that checks for both matching key and value filter
			let attribute_filter = arrow::compute::and(&key_filter, &value_filter).map_err()?;
			// combine with rest of filters
			filter = arrow::compute::or_kleene(&filter, &attribute_filter).map_err()?;
		}	

		Ok(filter)
	}

	fn get_log_record_filter(&self, logs_payload: &OtapArrowRecord) -> Result<BooleanArray, Error> {
		let log_records = logs_payload.get(ArrowPayLoadType::Logs).map_err()?;

		// create filter for severity texts
		let severity_texts_column = log_records.column_by_name(self::SEVERITY_TEXT);
		let mut severity_texts_filter;
		for severity_text in self.serverity_texts {
			let severity_text_scaler = StringArray::new_scaler(severity_text).map_err()?;
			let severity_text_filter = arrow::compute::kernels::cmp::eq(&severity_texts_column, &severity_text_scaler).map_err()?;
			severity_texts_filter = arrow::compute::or_kleene(&severity_text_filter, &severity_text_filter).map_err()?;
		}

		// create filter for log bodies
		let mut bodies_filter;
		for body in bodies {
			let body_filter = match body {
				MatchValue::String(value) => {
					// get string column
					let string_column = log_attrs.column_by_name(self::BODY_STR)?;
					let value_scaler = StringArray::new_scaler(value).map_err()?;
					arrow::compute::kernels::cmp::eq(&string_column, &value_scaler).map_err()
				}
				MatchValue::Int(value) => {
					let int_column = log_attrs.column_by_name(self::BODY_INT)?;
					let value_scaler = Int64Array::new_scaler(value).map_err()?;
					arrow::compute::kernels::cmp::eq(&int_column, &value_scaler).map_err()
				}
				MatchValue::Double(value) => {
					let double_column = log_attrs.column_by_name(self::BODY_DOUBLE)?;
					let value_scaler = Float64Array::new_scaler(value).map_err()?;
					arrow::compute::kernels::cmp::eq(&double_column, &value_scaler).map_err()
				}
				MatchValue::Boolean(value) => {
					let bool_column = log_attrs.column_by_name(self::BODY_BOOL)?;
					let value_scaler = BooleanArray::new_scaler(value).map_err()?;
					arrow::compute::kernels::cmp::eq(&bool_column, &value_scaler).map_err()
				}
				_ => {
					// need bytes and array
				}
			};
			bodies_filter = arrow::compute::or_kleene(&body_filter, &bodies_filter).map_err()?;
		}
		
		// combine the filters
		let mut filter = arrow::compute::and(&bodies_filter, &severity_texts_filter).map_err()?;

		// if the severity_number field is defined then we create the severity_nubmer filter
		if let Some(severity_number_properties) = self.severity_number {

			let severity_number_column = log_records.column_by_name(self::SEVERITY_NUMBER);
			// TODO make min a string that contains the severity number type and map to the int instead
			let min_severity_number = severity_number_proprties.min;
			let min_severity_scaler = Int32Array::new_scalar(min_severity_number).map_err()?;
			let mut severity_numbers_filter = arrow::compute::kernels::cmp::gt_eq(&severity_number_column, min_severity_scaler).map_err()?;
			// update the filter if we allow unknown 
			if severity_number_proprties.match_undefined {
				let unknown_severity_scaler = Int32Array::new_scalar(0).map_err()?;
				let unknown_severity_number_filter = arrow::compute::kernels::cmp::eq(&severity_number_column, unknown_severity_scaler).map_err()?;
				severity_numbers_filter = arrow::computer::or_kleene(&severity_numbers_filter, &unknown_severity_number_filter).map_err()?;
			}
			// combine severity number filter to the log record filter
			filter = arrow::compute::and(&filter, &serverity_numbers_filter).map()?;
		}

		Ok(filter)
	}

	fn get_log_attr_filter(&self, logs_payload: &OtapArrowRecord) -> Result<BooleanArray, Error>{
		// get log_attrs record batch
		let log_attrs = logs_payload.get(ArrowPayLoadType::LogAttrs).map_err()?;

		// TODO need to init empty boolean array with right size here
		let mut filter;

		// generate the filter for this record_batch
		for attribute_filter in self.record_attributes {
			let key_scaler = StringArray::new_scaler(attribute.key).map_err()?;
			let key_column = log_attrs.column_by_name(self::KEY)?;
			let key_filter = arrow::compute::kernels::cmp::eq(&key_column, &key_scaler).map_err()?;
			let value_filter = match attribute_filter.value {
				MatchValue::String(value) => {
					// get string column
					let string_column = log_attrs.column_by_name(self::STR)?;
					let value_scaler = StringArray::new_scaler(value).map_err()?;
					arrow::compute::kernels::cmp::eq(&string_column, &value_scaler).map_err()?
				}
				MatchValue::Int(value) => {
					let int_column = log_attrs.column_by_name(self::INT)?;
					let value_scaler = Int64Array::new_scaler(value).map_err()?;
					arrow::compute::kernels::cmp::eq(&int_column, &value_scaler).map_err()?
				}
				MatchValue::Double(value) => {
					let double_column = log_attrs.column_by_name(self::DOUBLE)?;
					let value_scaler = Float64Array::new_scaler(value).map_err()?;
					arrow::compute::kernels::cmp::eq(&double_column, &value_scaler).map_err()?
				}
				MatchValue::Boolean(value) => {
					let bool_column = log_attrs.column_by_name(self::BOOL)?;
					let value_scaler = BooleanArray::new_scaler(value).map_err()?;
					arrow::compute::kernels::cmp::eq(&bool_column, &value_scaler).map_err()?
				}
				_ => {
					// need bytes and array
				}
			};
			// build filter that checks for both matching key and value filter
			let attribute_filter = arrow::compute::and(&key_filter, &value_filter).map_err()?;
			// combine with rest of filters
			filter = arrow::compute::or_kleene(&filter, &attribute_filter).map_err()?;
		}	

		Ok(filter)
	}

    // fn build_boolean_array(&self, column, )

	pub fn create_filters(&self, logs_payload: &OtapArrowRecord, invert: bool) -> Result<(BooleanArray, BooleanArray, BooleanArray), Error> {
       let (mut resource_attr_filter, mut log_record_filter, mut log_attr_filter) =  match self.match_type {
            LogMatchType::Strict => {
                (self.get_log_record_filter(logs_payload)?, self.get_log_record_filter(logs_payload)?, self.get_log_attr_filter(logs_payload)?)
            },
            LogMatchType::Regexp => {
                // todo replace these functions with regexp filter function
                (self.get_log_record_filter(logs_payload)?, self.get_log_record_filter(logs_payload)?, self.get_log_attr_filter(logs_payload)?)
            }
        };

        if invert {
            resource_attr_filter = arrow::compute::not(&esource_attr_filter).map_err()?;
            log_record_filter = arrow::compute::not(&log_record_filter).map_err()?;
            log_attr_filter = arrow::compute::not(&log_attr_filter).map_err()?;
        }

        Ok((resource_attr_filter, log_record_filter, log_attr_filter))
	}
}