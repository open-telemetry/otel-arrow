// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Scenario: The wall clock moves backward while a cumulative metric query remains open.
/// Guarantees: The next point begins a new cumulative sequence that later scrapes retain.
#[test]
fn cumulative_start_time_resets_after_clock_rollback() {
    let path = r"\Memory\Available Bytes";
    let config = CounterConfig {
        metric_kind: MetricKind::Sum,
        ..counter(path, "memory")
    };
    let (state, mut query) = fake_query(
        vec![config.clone()],
        vec![fake_counter(path, Number::Integer(42))],
    );
    for (time, has_data, expected_start) in [
        (120, true, 100),
        (120, true, 100),
        (115, false, 115),
        (116, true, 115),
    ] {
        {
            let mut state = state.borrow_mut();
            state.timestamp = time;
            state.collect_has_data = has_data;
            state.counters[0].formatted = Some(Number::Integer(1));
        }
        let sample = query.collect().unwrap();
        assert_eq!(query.start_time_unix_nano, expected_start);
        assert_eq!(sample.timestamp_unix_nano, time);
        assert_eq!(sample.points[0].timestamp_unix_nano, time);
        if has_data {
            assert_eq!(sample.points[0].start_time_unix_nano, Some(expected_start));
            assert!(
                into_otap(std::slice::from_ref(&config), sample)
                    .unwrap()
                    .is_some()
            );
        } else {
            assert_eq!(sample.points[0].value, SampleValue::NoObservation);
        }
    }
    state.borrow_mut().collect_error = true;
    assert!(query.collect().is_err());
    state.borrow_mut().collect_error = false;
    state.borrow_mut().timestamp = 130;
    assert_eq!(
        query.collect().unwrap().points[0].start_time_unix_nano,
        Some(115)
    );
}

/// Scenario: A calculated counter sees a smaller valid base, a formatted omission, a hard collect error, and a timestamp error.
/// Guarantees: A decrease becomes the next baseline without warming, while every failed sample clears readiness and forces warm-up.
#[test]
fn calculated_readiness_distinguishes_base_decrease_from_failures() {
    let path = r"\PhysicalDisk(_Total)\Avg. Disk sec/Transfer";
    let (state, mut query) = fake_query(
        vec![counter(path, "windows.disk.average_latency")],
        vec![FakeCounter {
            path: path.to_owned(),
            native_type: 0x4002_0500,
            raw: Some(10),
            multi_count: 1,
            formatted: Some(Number::Double(4.0)),
            raw_error: false,
            formatted_error: false,
        }],
    );

    assert_eq!(
        query.collect().unwrap().points[0].value,
        SampleValue::Warming
    );
    state.borrow_mut().counters[0].raw = Some(12);
    assert!(matches!(
        query.collect().unwrap().points[0].value,
        SampleValue::Value(Number::Double(4.0))
    ));

    state.borrow_mut().counters[0].raw = Some(5);
    assert_eq!(
        query.collect().unwrap().points[0].value,
        SampleValue::NoObservation
    );
    state.borrow_mut().counters[0].raw = Some(6);
    assert!(matches!(
        query.collect().unwrap().points[0].value,
        SampleValue::Value(Number::Double(4.0))
    ));

    {
        let mut state = state.borrow_mut();
        state.counters[0].raw = Some(14);
        state.counters[0].formatted = None;
    }
    assert_eq!(
        query.collect().unwrap().points[0].value,
        SampleValue::NoObservation
    );
    state.borrow_mut().counters[0].raw = Some(16);
    assert_eq!(
        query.collect().unwrap().points[0].value,
        SampleValue::Warming
    );

    {
        let mut state = state.borrow_mut();
        state.counters[0].raw = Some(18);
        state.collect_error = true;
    }
    assert!(query.collect().is_err());
    {
        let mut state = state.borrow_mut();
        state.collect_error = false;
        state.counters[0].raw = Some(20);
    }
    assert_eq!(
        query.collect().unwrap().points[0].value,
        SampleValue::Warming
    );

    {
        let mut state = state.borrow_mut();
        state.counters[0].raw = Some(22);
        state.timestamp_error = true;
    }
    assert!(query.collect().is_err());
    {
        let mut state = state.borrow_mut();
        state.timestamp_error = false;
        state.counters[0].raw = Some(24);
    }
    assert_eq!(
        query.collect().unwrap().points[0].value,
        SampleValue::Warming
    );
}

/// Scenario: Elapsed-time, raw-fraction, and large raw-fraction counters each have one valid collected sample.
/// Guarantees: Every formatted single-sample formula emits immediately, accepts a zero native base through PDH formatting, and performs no raw read.
#[test]
fn elapsed_time_emits_from_first_sample() {
    let elapsed_path = r"\System\System Up Time";
    let fraction_path = r"\Memory\% Committed Bytes In Use";
    let large_fraction_path = r"\Memory\Large Fraction";
    let (state, mut query) = fake_query(
        vec![
            counter(elapsed_path, "system.uptime"),
            counter(fraction_path, "system.memory.committed"),
            counter(large_fraction_path, "system.memory.large_fraction"),
        ],
        vec![
            FakeCounter {
                path: elapsed_path.to_owned(),
                native_type: 0x3024_0500,
                raw: None,
                multi_count: 1,
                formatted: Some(Number::Double(42.5)),
                raw_error: false,
                formatted_error: false,
            },
            FakeCounter {
                path: fraction_path.to_owned(),
                native_type: 0x2002_0400,
                raw: Some(0),
                multi_count: 0,
                formatted: Some(Number::Double(50.0)),
                raw_error: true,
                formatted_error: false,
            },
            FakeCounter {
                path: large_fraction_path.to_owned(),
                native_type: 0x2002_0500,
                raw: Some(0),
                multi_count: 0,
                formatted: Some(Number::Double(25.0)),
                raw_error: true,
                formatted_error: false,
            },
        ],
    );

    let sample = query.collect().unwrap();
    assert_eq!(
        sample
            .points
            .iter()
            .map(|point| point.value)
            .collect::<Vec<_>>(),
        [
            SampleValue::Value(Number::Double(42.5)),
            SampleValue::Value(Number::Double(50.0)),
            SampleValue::Value(Number::Double(25.0))
        ]
    );
    assert_eq!(state.borrow().calls.raw, 0);
}

/// Scenario: Each multi-timer SDK family warms, receives a zero native MultiCount, then recovers.
/// Guarantees: The only receiver-side denominator guard covers every multi-timer and preserves the next valid interval.
#[test]
fn all_multi_timer_invalid_bases_preserve_readiness() {
    for native_type in [0x2241_0500, 0x2341_0500, 0x2251_0500, 0x2351_0500] {
        let path = format!(r"\Fixture\Multi Timer {native_type:08X}");
        let (state, mut query) = fake_query(
            vec![counter(&path, "system.cpu.utility")],
            vec![FakeCounter {
                path: path.clone(),
                native_type,
                raw: Some(10),
                multi_count: 4,
                formatted: Some(Number::Double(25.0)),
                raw_error: false,
                formatted_error: false,
            }],
        );

        assert_eq!(
            query.collect().unwrap().points[0].value,
            SampleValue::Warming
        );
        {
            let mut state = state.borrow_mut();
            state.counters[0].raw = Some(20);
            state.counters[0].multi_count = 0;
        }
        let sample = query.collect().unwrap();
        assert!(sample.points.is_empty());
        assert_eq!(sample.metadata.failures.len(), 1);
        assert!(
            sample.metadata.failures[0]
                .error
                .contains("multi-counter base denominator must be positive"),
            "native type 0x{native_type:08X}"
        );

        {
            let mut state = state.borrow_mut();
            state.counters[0].raw = Some(30);
            state.counters[0].multi_count = 4;
        }
        assert_eq!(
            query.collect().unwrap().points[0].value,
            SampleValue::Value(Number::Double(25.0)),
            "native type 0x{native_type:08X}"
        );
    }
}

/// Scenario: A formatted transient interrupts a ready native delta count.
/// Guarantees: Native readiness resets and one valid sample re-warms before emission with the unchanged cumulative start.
#[test]
fn native_delta_transient_restarts_readiness() {
    let path = r"\Provider\Operations Delta";
    let (state, mut query) = fake_query(
        vec![cumulative_sum(path, "windows.operations", "{operation}")],
        vec![FakeCounter {
            path: path.to_owned(),
            native_type: 0x0040_0500,
            raw: Some(10),
            multi_count: 1,
            formatted: Some(Number::Double(2.0)),
            raw_error: false,
            formatted_error: false,
        }],
    );
    assert_eq!(
        query.collect().unwrap().points[0].value,
        SampleValue::Warming
    );

    {
        let mut state = state.borrow_mut();
        state.timestamp = 110;
        state.counters[0].raw = Some(20);
        state.counters[0].formatted = None;
    }
    assert_eq!(
        query.collect().unwrap().points[0].value,
        SampleValue::NoObservation
    );

    {
        let mut state = state.borrow_mut();
        state.timestamp = 120;
        state.counters[0].raw = Some(30);
        state.counters[0].formatted = Some(Number::Double(3.0));
    }
    assert_eq!(
        query.collect().unwrap().points[0].value,
        SampleValue::Warming
    );

    {
        let mut state = state.borrow_mut();
        state.timestamp = 130;
        state.counters[0].raw = Some(40);
        state.counters[0].formatted = Some(Number::Double(4.0));
    }
    let sample = query.collect().unwrap();
    assert_eq!(sample.points[0].start_time_unix_nano, Some(100));
}

/// Scenario: Query collection and timestamp failures interrupt a ready native delta count.
/// Guarantees: Each gap clears native readiness so the next valid sample warms while retaining the cumulative start.
#[test]
fn native_delta_collection_gaps_restart_readiness() {
    let path = r"\Provider\Operations Delta";
    let (state, mut query) = fake_query(
        vec![cumulative_sum(path, "windows.operations", "{operation}")],
        vec![FakeCounter {
            path: path.to_owned(),
            native_type: 0x0040_0500,
            raw: Some(10),
            multi_count: 1,
            formatted: Some(Number::Double(2.0)),
            raw_error: false,
            formatted_error: false,
        }],
    );
    assert_eq!(
        query.collect().unwrap().points[0].value,
        SampleValue::Warming
    );

    {
        let mut state = state.borrow_mut();
        state.collect_error = true;
    }
    assert!(query.collect().is_err());
    {
        let mut state = state.borrow_mut();
        state.collect_error = false;
        state.timestamp = 120;
        state.counters[0].raw = Some(20);
    }
    assert_eq!(
        query.collect().unwrap().points[0].value,
        SampleValue::Warming
    );

    {
        let mut state = state.borrow_mut();
        state.timestamp_error = true;
        state.counters[0].raw = Some(30);
    }
    assert!(query.collect().is_err());
    {
        let mut state = state.borrow_mut();
        state.timestamp_error = false;
        state.timestamp = 140;
        state.counters[0].raw = Some(40);
    }
    assert_eq!(
        query.collect().unwrap().points[0].value,
        SampleValue::Warming
    );

    {
        let mut state = state.borrow_mut();
        state.timestamp = 150;
        state.counters[0].raw = Some(50);
        state.counters[0].formatted = Some(Number::Double(5.0));
    }
    let sample = query.collect().unwrap();
    assert_eq!(sample.points[0].start_time_unix_nano, Some(100));
}
