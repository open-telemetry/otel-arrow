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

	// Kvlist @@@
	
	// Map @@@
    }

    #[test]
    fn test_key_value() {
	use crate::proto::opentelemetry::common::v1::AnyValue;
	use crate::proto::opentelemetry::common::v1::KeyValue;	

	let k1 = "k1".to_string();
	let v1 = AnyValue::new_string("v1");
	let kv_value = KeyValue{
	    key: k1.clone(),
	    value: Some(v1.clone()),
	};

	assert_eq!(KeyValue::new("k1", v1.clone()), kv_value);
	assert_eq!(KeyValue::new(k1, v1), kv_value);
    }
}
