use crate::fake_signal_receiver::config::{AttributeValue, EventConfig, LinkConfig, DatapointConfig, DatapointType};
use std::collections::HashMap;

const EVENT_NAMES: [&str; 5] = [
    "unknown",
    "message-receive",
    "message-send",
    "http-receive",
    "http-send",
];


const SPAN_NAMES: [&str; 6] = [
    "dns-lookup",
    "message-send",
    "http-close",
    "unknown",
    "http-send",
    "http-close",
];

const TRACE_STATES: [&str; 3] = ["started", "ended", "unknown"];

pub fn default_event_names() -> Vec<String> {
    EVENT_NAMES.iter().map(|s| s.to_string()).collect()
}

pub fn default_trace_states() -> Vec<String> {
    TRACE_STATES.iter().map(|s| s.to_string()).collect()
}

pub fn default_span_names() -> Vec<String> {
    SPAN_NAMES.iter().map(|s| s.to_string()).collect()
}

pub fn default_event_config() -> EventConfig {
    EventConfig::new(0, vec![], HashMap::new())
}

pub fn default_link_config() -> LinkConfig {
    LinkConfig::new(0, vec![], HashMap::new())
}

pub fn default_datapoint_config() -> DatapointConfig {
    DatapointConfig::new(0, vec![], HashMap::new(), 0.0, 0.0)
}

pub fn default_top_value() -> f64 {
    10.0
}

pub fn default_bottom_value() -> f64 {
    0.0
}

pub fn default_datapoint_type() -> Vec<DatapointType> {
    vec![DatapointType::Gauge]
}

pub fn default_metric_attributes() -> HashMap<String, Vec<AttributeValue>> {
    HashMap::from([
        ("cpu".to_string(), vec![AttributeValue::Int(0)]),
        (
            "cpu_id".to_string(),
            vec![
                AttributeValue::String("cpu-0".to_string()),
                AttributeValue::String("cpu-1".to_string()),
                AttributeValue::String("cpu-2".to_string()),
                AttributeValue::String("cpu-3".to_string()),
            ],
        ),
        (
            "cpu_arch".to_string(),
            vec![AttributeValue::String("x86-64".to_string())],
        ),
        (
            "cpu_vendor".to_string(),
            vec![AttributeValue::String("intel".to_string())],
        ),
        (
            "cpu_model".to_string(),
            vec![
                AttributeValue::String("i7".to_string()),
                AttributeValue::String("i5".to_string()),
            ],
        ),
        (
            "cpu_mhz".to_string(),
            vec![
                AttributeValue::String("2.4".to_string()),
                AttributeValue::String("4.2".to_string()),
            ],
        ),
        (
            "cpu_cores".to_string(),
            vec![
                AttributeValue::String("4".to_string()),
                AttributeValue::String("8".to_string()),
                AttributeValue::String("16".to_string()),
            ],
        ),
        (
            "cpu_logical_processors".to_string(),
            vec![AttributeValue::String("8".to_string())],
        ),
    ])
}

pub fn default_attributes() -> HashMap<String, Vec<AttributeValue>> {
    HashMap::from([
        (
            "version".to_string(),
            vec![
                AttributeValue::String("2.0.0".to_string()),
                AttributeValue::String("1.0.0".to_string()),
                AttributeValue::String("1.5.2".to_string()),
            ],
        ),
        (
            "status".to_string(),
            vec![
                AttributeValue::Int(400),
                AttributeValue::Int(404),
                AttributeValue::Int(200),
            ],
        ),
        (
            "hostname".to_string(),
            vec![
                AttributeValue::String("host3.thedomain.edu".to_string()),
                AttributeValue::String("host1.mydomain.com".to_string()),
                AttributeValue::String("host4.gov".to_string()),
                AttributeValue::String("host2.org".to_string()),
            ],
        ),
        (
            "up".to_string(),
            vec![AttributeValue::Bool(true), AttributeValue::Bool(false)],
        ),
    ])
}
