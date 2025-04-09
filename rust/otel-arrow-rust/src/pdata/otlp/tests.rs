// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(test)]
mod tests {
    #[test]
    fn test_any_value() {
	use crate::proto::opentelemetry::common::v1::AnyValue;
	use crate::proto::opentelemetry::common::v1::KeyValue;
	use crate::proto::opentelemetry::common::v1::KeyValueList;
	use crate::proto::opentelemetry::common::v1::ArrayValue;
	use crate::proto::opentelemetry::common::v1::any_value::Value;

	// Primitives
	assert_eq!(AnyValue::new_int(3i64),
		   AnyValue{
		       value: Some(Value::IntValue(3i64)),
		   });
	assert_eq!(AnyValue::new_double(3.123),
		   AnyValue{
		       value: Some(Value::DoubleValue(3.123)),
		   });
	assert_eq!(AnyValue::new_bool(true),
		   AnyValue{
		       value: Some(Value::BoolValue(true)),
		   });

	// String
	let xyz = "xyz".to_string();
	let xyz_value = AnyValue{
	    value: Some(Value::StringValue(xyz.to_string())),
	};
	assert_eq!(AnyValue::new_string("xyz"), xyz_value);
	assert_eq!(AnyValue::new_string(&xyz), xyz_value);
	assert_eq!(AnyValue::new_string(xyz), xyz_value);

	// Bytes
	let hello: Vec<u8> = [104, 101, 108, 108, 111].to_vec();
	let hello_value = AnyValue{
	    value: Some(Value::BytesValue(b"hello".to_vec())),
	};
	assert_eq!(AnyValue::new_bytes(hello.as_slice()), hello_value);
	assert_eq!(AnyValue::new_bytes(&hello), hello_value);
	assert_eq!(AnyValue::new_bytes(hello), hello_value);

	// Kvlist
	let kvs = vec![
	    KeyValue::new("k1", AnyValue::new_string("s1")),
	    KeyValue::new("k2", AnyValue::new_double(2.0)),
	];
	let kvs_value = AnyValue{
	    value: Some(Value::KvlistValue(KeyValueList{
		values: kvs.clone(),
	    })),
	};

	assert_eq!(AnyValue::new_kvlist(&kvs), kvs_value);
	assert_eq!(AnyValue::new_kvlist(&[
	    KeyValue::new("k1", AnyValue::new_string("s1")),
	    KeyValue::new("k2", AnyValue::new_double(2.0)),
	]), kvs_value);
	
	// Array
	let vals = vec![
	    AnyValue::new_string("s1"),
	    AnyValue::new_double(2.0),
	];
	let vals_value = AnyValue{
	    value: Some(Value::ArrayValue(ArrayValue{
		values: vals.clone(),
	    })),
	};

	assert_eq!(AnyValue::new_array(&vals), vals_value);
	assert_eq!(AnyValue::new_array(vals), vals_value);
	assert_eq!(AnyValue::new_array(vec![
	    AnyValue::new_string("s1"),
	    AnyValue::new_double(2.0),
	]), vals_value);
    }

    #[test]
    fn test_key_value() {
	use crate::proto::opentelemetry::common::v1::AnyValue;
	use crate::proto::opentelemetry::common::v1::KeyValue;	
	let k1 = "k1".to_string();
	let k2 = "k2".to_string();
	let v1 = AnyValue::new_string("v1");
	let v2 = AnyValue::new_double(1.23);

	let kv1_value = KeyValue{
	    key: k1.clone(),
	    value: Some(v1.clone()),
	};
	let kv2_value = KeyValue{
	    key: k2.clone(),
	    value: Some(v2.clone()),
	};

	assert_eq!(KeyValue::new("k1", v1.clone()), kv1_value);
	assert_eq!(KeyValue::new(k1.clone(), v1), kv1_value);

	assert_eq!(KeyValue::new("k2", v2.clone()), kv2_value);
	assert_eq!(KeyValue::new(k2, v2), kv2_value);
    }
    
    #[test]
    fn test_log_record_required() {
	use crate::proto::opentelemetry::logs::v1::LogRecord;
	use crate::proto::opentelemetry::logs::v1::SeverityNumber;

	let name = "my_log";
	let ts = 1_000_000_000_000u64;
	let sev = SeverityNumber::Info;
	let lr1_value = {
	    let mut value = LogRecord::default();
	    value.time_unix_nano = ts;
	    value.severity_number = sev as i32;
	    value.event_name = name.into();
	    value
	};
	let lr1 = LogRecord::new(ts, sev, name)
	    .build();

	assert_eq!(lr1, lr1_value);
    }

    #[test]
    fn test_log_record_required_all() {
	use crate::proto::opentelemetry::common::v1::AnyValue;
	use crate::proto::opentelemetry::logs::v1::LogRecord;
	use crate::proto::opentelemetry::logs::v1::LogRecordFlags;
	use crate::proto::opentelemetry::logs::v1::SeverityNumber;

	let name = "my_log";
	let sevtxt = "not on fire";
	let msg = "message in a bottle";
	let ts = 1_000_000_000_000u64;
	let sev = SeverityNumber::Info;
	let flags = LogRecordFlags::TraceFlagsMask;

	let lr1_value = {
	    let mut value = LogRecord::default();
	    value.time_unix_nano = ts;
	    value.severity_number = sev as i32;
	    value.event_name = name.into();
	    value.body = Some(AnyValue::new_string(msg));
	    value.severity_text = sevtxt.into();
	    value.flags = flags as u32;
	    value
	};
	let lr1 = LogRecord::new(ts, sev, name)
	    .body(AnyValue::new_string(msg))
	    .severity_text(sevtxt)
	    .flags(flags)
	    .build();

	assert_eq!(lr1, lr1_value);
    }

    #[test]
    fn test_instrumentation_scope_default() {
	use crate::proto::opentelemetry::common::v1::InstrumentationScope;

	let is1 = InstrumentationScope::new("library").build();
	let mut is1_value = InstrumentationScope::default();
	is1_value.name = "library".into();

	assert_eq!(is1, is1_value);
    }    

    #[test]
    fn test_instrumentation_scope_options() {
	use crate::proto::opentelemetry::common::v1::InstrumentationScope;
	use crate::proto::opentelemetry::common::v1::AnyValue;
	use crate::proto::opentelemetry::common::v1::KeyValue;

	let kv1 = KeyValue::new("k1", AnyValue::new_string("v1"));
	let kv2 = KeyValue::new("k2", AnyValue::new_int(2));
	let is1 = InstrumentationScope::new("library")
	    .version("v1.0")
	    .attributes(&[kv1.clone(), kv2.clone()])
	    .dropped_attributes_count(1u32)
	    .build();
	let mut is1_value = InstrumentationScope::default();
	is1_value.name = "library".into();
	is1_value.attributes = vec![
	    kv1, kv2,
	];
	is1_value.version = "v1.0".into();
	is1_value.dropped_attributes_count = 1u32;
	
	assert_eq!(is1, is1_value);
    }    
}
