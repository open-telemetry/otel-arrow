use crate::proto::opentelemetry::common::v1::{AnyValue, KeyValue, any_value::Value};

#[test]
fn debug_keyvalue_size_components() {
    // Create test KeyValue
    let key_value = KeyValue {
        key: "test_key".to_string(),
        value: Some(AnyValue {
            value: Some(Value::StringValue("test_value".to_string())),
        }),
    };

    // Get Prost reference size
    let prost_size = ::prost::Message::encoded_len(&key_value);

    // Calculate our pdata size using the generated method
    let pdata_size = key_value.pdata_size();

    println!("=== SIZE BREAKDOWN ===");
    println!("Prost reference: {} bytes", prost_size);
    println!("Pdata calculated: {} bytes", pdata_size);

    // Test individual components
    println!("\n=== COMPONENT BREAKDOWN ===");

    // Test key alone (should be 10: tag(1) + len(1) + 8 bytes)
    println!("Key: '{}'", key_value.key);
    println!("Key length: {} bytes", key_value.key.len());
    println!(
        "Key expected: tag(1) + len(1) + {} = {} bytes",
        key_value.key.len(),
        1 + 1 + key_value.key.len()
    );

    // Test AnyValue alone
    if let Some(any_value) = &key_value.value {
        let any_value_prost = ::prost::Message::encoded_len(any_value);
        let any_value_pdata = any_value.pdata_size();
        println!(
            "AnyValue - prost: {}, pdata: {}",
            any_value_prost, any_value_pdata
        );

        if let Some(Value::StringValue(s)) = &any_value.value {
            println!("String value: '{}'", s);
            println!("String length: {} bytes", s.len());
            println!(
                "String expected: tag(1) + len(1) + {} = {} bytes",
                s.len(),
                1 + 1 + s.len()
            );
        }
    }

    // Test a KeyValue with no value
    let key_only = KeyValue {
        key: "test_key".to_string(),
        value: None,
    };
    println!(
        "KeyValue (key only) - prost: {}, pdata: {}",
        ::prost::Message::encoded_len(&key_only),
        key_only.pdata_size()
    );

    println!("\n=== EXPECTED CALCULATION ===");
    println!("Key field: tag(1) + len(1) + data(8) = 10 bytes");
    println!("Value field: tag(2) + len(1) + AnyValue(12) = 14 bytes");
    println!("Expected total: 24 bytes");

    assert_eq!(
        prost_size, pdata_size,
        "pdata_size and prost encoded_len differ for KeyValue"
    );
}
