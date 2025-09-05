// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::proto::opentelemetry::common::v1::{AnyValue, KeyValue, any_value::Value};
use crate::pdata::otlp::PrecomputedSizes;

pub fn debug_keyvalue_visitor() {
    // Create test data
    let key_value = KeyValue {
        key: "test_key".to_string(),
        value: Some(AnyValue {
            value: Some(Value::StringValue("test_value".to_string())),
        }),
    };

    println!("=== DETAILED KEYVALUE VISITOR DEBUG ===");
    
    // Test individual components
    let just_key = KeyValue {
        key: "test_key".to_string(),
        value: None,
    };
    
    let just_value = AnyValue {
        value: Some(Value::StringValue("test_value".to_string())),
    };
    
    println!("Key only: prost={}, pdata={}", 
        ::prost::Message::encoded_len(&just_key), 
        just_key.pdata_size());
        
    println!("Value only: prost={}, pdata={}", 
        ::prost::Message::encoded_len(&just_value), 
        just_value.pdata_size());
        
    println!("Combined: prost={}, pdata={}", 
        ::prost::Message::encoded_len(&key_value), 
        key_value.pdata_size());
    
    // Now let's trace through the visitor step by step
    println!("\n=== VISITOR STEP-BY-STEP ===");
    
    // Step 1: Create sizes accumulator
    let mut sizes = PrecomputedSizes::default();
    
    // Step 2: Create KeyValue visitor manually and trace its execution
    use crate::proto::opentelemetry::common::v1::{KeyValueEncodedLen, KeyValueMessageAdapter};
    
    let mut visitor = KeyValueEncodedLen::new(0); // Use tag 0 for top-level
    let adapter = KeyValueMessageAdapter::new(&key_value);
    
    println!("Starting visitor with empty sizes: len={}", sizes.len());
    
    // This should replicate what the pdata_size() method does
    let (final_sizes, total_size) = KeyValueEncodedLen::children_encoded_size(sizes, &adapter);
    
    println!("Final total size: {}", total_size);
    println!("Final sizes vec: {:?}", final_sizes.debug_sizes());
}
