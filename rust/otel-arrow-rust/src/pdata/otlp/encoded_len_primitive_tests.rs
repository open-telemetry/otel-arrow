#[cfg(test)]
mod encoded_len_primitive_tests {
    use prost::Message;
    use crate::proto::opentelemetry::common::v1::AnyValue;
    use crate::proto::opentelemetry::common::v1::KeyValue;
    use crate::proto::opentelemetry::common::v1::any_value::Value;

    #[test]
    fn test_encoded_len_i32() {
        let value: i32 = 42;
        // prost encodes primitives as fields in messages, so we use AnyValue
        let av = AnyValue { value: Some(Value::IntValue(value as i64)) };
        let prost_len = av.encoded_len();
        let pdata_len = av.pdata_size();
        assert_eq!(prost_len, pdata_len, "pdata_size and prost encoded_len differ for i32");
    }

    #[test]
    fn test_encoded_len_f64() {
        let value: f64 = 3.1415;
        let av = AnyValue { value: Some(Value::DoubleValue(value)) };
        let prost_len = av.encoded_len();
        let pdata_len = av.pdata_size();
        assert_eq!(prost_len, pdata_len, "pdata_size and prost encoded_len differ for f64");
    }

    #[test]
    fn test_encoded_len_string() {
        let value = "hello world".to_string();
        let av = AnyValue { value: Some(Value::StringValue(value.clone())) };
        let prost_len = av.encoded_len();
        let pdata_len = av.pdata_size();
        assert_eq!(prost_len, pdata_len, "pdata_size and prost encoded_len differ for String");
    }
}
