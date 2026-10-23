// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

impl NativePdh {
    pub(in super::super) fn open(node: String) -> Result<Self, Error> {
        Self::open_controlled(node, Arc::new(AtomicBool::new(false)))
    }

    pub(in super::super) fn parse_instance_path(path: &str) -> Result<InstanceIdentity, Error> {
        Self::parse_instance_controlled(path, &AtomicBool::new(false))
    }

    pub(in super::super) fn comparison_path_fixture(path: &str) -> Result<String, Error> {
        Self::comparison_path_controlled(path, &AtomicBool::new(false))
    }
}

impl<T: Clone + Default> NativeBuffer<T> {
    fn as_mut_ptr<U>(&mut self) -> *mut U {
        self.storage.as_mut_ptr().cast::<U>()
    }
}

/// Scenario: A native path contains machine, object, parent, numeric instance suffix and counter components.
/// Guarantees: Comparison removes only the machine prefix and normalizes the parsed instance suffix, never object or counter spelling.
#[test]
fn comparison_paths_normalize_only_native_instance_components() {
    for (path, expected) in [
        (
            r"\\LOCAL\Object#00(worker#00)\Counter#00",
            r"\Object#00(worker)\Counter#00",
        ),
        (
            r"\Object(parent/worker#0001)\Counter",
            r"\Object(parent/worker#1)\Counter",
        ),
        (r"\Object(worker#1)\Counter", r"\Object(worker#1)\Counter"),
        (r"\Object\Counter#00", r"\Object\Counter#00"),
    ] {
        assert_eq!(NativePdh::comparison_path_fixture(path).unwrap(), expected);
    }
}
use super::super::super::TEST_PDH_LOCK;
use std::cell::RefCell;
use windows_sys::Win32::System::Performance::PdhFormatFromRawValue;

/// Scenario: Native startup and local object-list refresh run in the calling thread's existing UI language.
/// Guarantees: Refresh returns an object count without changing language or requiring English as a startup prerequisite.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Windows performance counters; run explicitly with `-- --ignored`"]
async fn live_object_refresh_preserves_thread_language() {
    let _serial = TEST_PDH_LOCK.lock().await;
    // SAFETY: This reads the current test thread's native UI language.
    let before = unsafe { GetThreadUILanguage() };
    initialize_worker("object-refresh", &AtomicBool::new(false)).unwrap();
    let mut api = NativePdh::open("object-refresh".to_owned()).unwrap();
    assert!(api.refresh_objects().unwrap() > 0);
    // SAFETY: This reads the same thread after startup and refresh.
    assert_eq!(unsafe { GetThreadUILanguage() }, before);
}

/// Scenario: Object-list sizing succeeds or requests a buffer, and the first fill may require growth.
/// Guarantees: Only sizing refreshes the local list; fill retries without another refresh and returns the object count.
#[test]
fn object_refresh_sizing_and_fill_follow_native_probe_sequence() {
    for sizing_status in [0, PDH_MORE_DATA] {
        for grow in [false, true] {
            let calls = RefCell::new(Vec::new());
            let count = refresh_objects_with(|refresh, size, buffer| {
                calls.borrow_mut().push((refresh, buffer.is_some()));
                match buffer {
                    None => {
                        *size = 8;
                        Ok(sizing_status)
                    }
                    Some(buffer) if grow && calls.borrow().len() == 2 => {
                        assert_eq!(buffer.len(), 8);
                        *size = 24;
                        Ok(PDH_MORE_DATA)
                    }
                    Some(buffer) => {
                        assert_eq!(buffer.len(), if grow { 24 } else { 8 });
                        buffer[..5].copy_from_slice(&[65, 0, 66, 0, 0]);
                        *size = 5;
                        Ok(0)
                    }
                }
            })
            .unwrap();
            assert_eq!(count, 2);
            let mut expected = vec![(true, false), (false, true)];
            if grow {
                expected.push((false, true));
            }
            assert_eq!(calls.into_inner(), expected);
        }
    }
}

/// Scenario: A conditional object-list refresh fails during sizing or buffer fill.
/// Guarantees: Both native stages preserve an explicit typed error for the informational refresh result.
#[test]
fn worker_rejects_failed_object_refresh() {
    let sizing = refresh_objects_with(|_, _, _| Ok(5)).unwrap_err();
    assert!(matches!(
        sizing,
        Error::WorkerObjectRefresh {
            stage: "refresh sizing",
            status: 5,
        }
    ));

    let fill = refresh_objects_with(|_, size, buffer| {
        if buffer.is_none() {
            *size = 8;
            Ok(PDH_MORE_DATA)
        } else {
            Ok(6)
        }
    })
    .unwrap_err();
    assert!(matches!(
        fill,
        Error::WorkerObjectRefresh {
            stage: "enumeration fill",
            status: 6,
        }
    ));
}

/// Scenario: Object membership keeps growing beyond the single allowed fill retry.
/// Guarantees: Refresh returns a typed failure after two fills and never refreshes a second time.
#[test]
fn object_refresh_fill_growth_is_bounded() {
    let mut calls = 0;
    let error = refresh_objects_with(|refresh, size, buffer| {
        calls += 1;
        assert_eq!(refresh, buffer.is_none());
        *size = size.saturating_add(8);
        Ok(PDH_MORE_DATA)
    })
    .unwrap_err();
    assert_eq!(calls, 3);
    assert!(matches!(
        error,
        Error::WorkerObjectRefresh {
            stage: "enumeration fill",
            status: PDH_MORE_DATA
        }
    ));
}

/// Scenario: A stop request arrives between native sizing and fill operations.
/// Guarantees: Object refresh, expansion and path parsing return a typed stop immediately without regrowing buffers.
#[test]
fn native_buffer_calls_propagate_stop_without_retry() {
    let mut calls = 0;
    let error = refresh_objects_with(|_, size, buffer| {
        calls += 1;
        if buffer.is_none() {
            *size = 8;
            Ok(PDH_MORE_DATA)
        } else {
            Err(Error::WorkerStopRequested)
        }
    })
    .unwrap_err();
    assert!(matches!(error, Error::WorkerStopRequested));
    assert_eq!(calls, 2);
    calls = 0;
    let error = NativePdh::expanded_path_buffer_with("fixture", |size, buffer| {
        calls += 1;
        if buffer.is_none() {
            *size = 8;
            Ok(PDH_MORE_DATA)
        } else {
            Err(Error::WorkerStopRequested)
        }
    })
    .unwrap_err();
    assert!(matches!(error, Error::WorkerStopRequested));
    assert_eq!(calls, 2);
    calls = 0;
    let error = NativePdh::parse_instance_identity_with("fixture", |size, buffer| {
        calls += 1;
        if buffer.is_none() {
            *size = u32::try_from(size_of::<PDH_COUNTER_PATH_ELEMENTS_W>()).unwrap();
            Ok(PDH_MORE_DATA)
        } else {
            Err(Error::WorkerStopRequested)
        }
    })
    .unwrap_err();
    assert!(matches!(error, Error::WorkerStopRequested));
    assert_eq!(calls, 2);
    assert!(matches!(
        check_running(&AtomicBool::new(true)),
        Err(Error::WorkerStopRequested)
    ));
}

fn insert_fixture_counter(counters: &mut CounterHandles, path: &str) -> CounterToken {
    let generation = counters.take_generation(path).unwrap();
    counters.insert(generation, std::ptr::dangling_mut())
}

#[derive(Clone, Copy)]
enum BufferApi {
    CounterInfo,
    CounterType,
    ParseInstance,
}

impl BufferApi {
    fn header_size(self) -> u32 {
        u32::try_from(match self {
            Self::CounterInfo | Self::CounterType => size_of::<PDH_COUNTER_INFO_W>(),
            Self::ParseInstance => size_of::<PDH_COUNTER_PATH_ELEMENTS_W>(),
        })
        .unwrap()
    }

    fn size_operation(self) -> PdhOperation {
        match self {
            Self::CounterInfo | Self::CounterType => PdhOperation::GetCounterInfoSize,
            Self::ParseInstance => PdhOperation::ParseCounterPathSize,
        }
    }

    fn fill_operation(self) -> PdhOperation {
        match self {
            Self::CounterInfo | Self::CounterType => PdhOperation::GetCounterInfo,
            Self::ParseInstance => PdhOperation::ParseCounterPath,
        }
    }

    fn malformed_error(self, case: MalformedBuffer) -> Error {
        let header = self.header_size();
        let status = |size: &mut u32, filling: bool| match case {
            MalformedBuffer::NonSizingStatus => {
                *size = header;
                PDH_INVALID_ARGUMENT
            }
            MalformedBuffer::SuccessSizingStatus => {
                *size = header;
                0
            }
            MalformedBuffer::UndersizedRequest => {
                *size = header - 1;
                PDH_MORE_DATA
            }
            MalformedBuffer::UndersizedFill if filling => {
                *size = header - 1;
                0
            }
            MalformedBuffer::OversizedFill if filling => {
                *size = u32::MAX;
                0
            }
            MalformedBuffer::FailedFill if filling => PDH_MORE_DATA,
            MalformedBuffer::UndersizedFill
            | MalformedBuffer::OversizedFill
            | MalformedBuffer::FailedFill => {
                *size = header;
                PDH_MORE_DATA
            }
        };
        match self {
            Self::CounterInfo => fixed_header_buffer_with::<usize>(
                "fixture",
                &COUNTER_INFO_BUFFER,
                |size, buffer| Ok(status(size, buffer.is_some())),
            )
            .map(|_| ()),
            Self::CounterType => NativePdh::counter_type_with("fixture", |size, buffer| {
                Ok(status(size, buffer.is_some()))
            })
            .map(|_| ()),
            Self::ParseInstance => {
                NativePdh::parse_instance_identity_with("fixture", |size, buffer| {
                    Ok(status(size, buffer.is_some()))
                })
                .map(|_| ())
            }
        }
        .unwrap_err()
    }
}

#[derive(Clone, Copy)]
enum MalformedBuffer {
    NonSizingStatus,
    SuccessSizingStatus,
    UndersizedRequest,
    UndersizedFill,
    OversizedFill,
    FailedFill,
}

fn parse_instance_fixture_error(
    filled_size: u32,
    mut configure: impl FnMut(*mut u8, *mut PDH_COUNTER_PATH_ELEMENTS_W),
) -> Error {
    NativePdh::parse_instance_identity_with("fixture", |size, buffer| {
        let Some(buffer) = buffer else {
            *size = filled_size;
            return Ok(PDH_MORE_DATA);
        };
        let base = buffer.as_mut_ptr().cast::<u8>();
        configure(base, base.cast());
        *size = filled_size;
        Ok(0)
    })
    .unwrap_err()
}

/// Scenario: A long-lived query repeatedly adds and removes a fixed peak number of counters.
/// Guarantees: Generational bookkeeping retains storage proportional to peak concurrent ownership, not lifetime additions.
#[test]
fn counter_slots_remain_bounded_under_sustained_churn() {
    let path = "fixture";
    let mut counters = CounterHandles::new();
    let peak = 4;
    for _ in 0..1_024 {
        let tokens = (0..peak)
            .map(|_| insert_fixture_counter(&mut counters, path))
            .collect::<Vec<_>>();
        assert_eq!(counters.slots.len(), peak);
        assert!(counters.free_slots.is_empty());
        for token in tokens {
            counters.remove_with(&token, path, |_| 0).unwrap();
        }
        assert_eq!(counters.slots.len(), peak);
        assert_eq!(counters.free_slots.len(), peak);
    }
}

/// Scenario: A removed counter's slot is reused by a later addition.
/// Guarantees: Generation fencing prevents the stale token from resolving or invoking native removal against the replacement.
#[test]
fn reused_counter_slot_rejects_stale_token() {
    let path = "fixture";
    let mut counters = CounterHandles::new();
    let removed = insert_fixture_counter(&mut counters, path);
    counters.remove_with(&removed, path, |_| 0).unwrap();
    let replacement = insert_fixture_counter(&mut counters, path);

    assert_eq!(removed.index, replacement.index);
    assert_ne!(removed.generation, replacement.generation);
    assert!(matches!(
        counters.handle(&removed, path),
        Err(Error::InvalidCounterToken { .. })
    ));
    let mut native_remove_called = false;
    assert!(matches!(
        counters.remove_with(&removed, path, |_| {
            native_remove_called = true;
            0
        }),
        Err(Error::InvalidCounterToken { .. })
    ));
    assert!(!native_remove_called);
    assert!(counters.handle(&replacement, path).is_ok());
}

/// Scenario: Native removal fails for an owned counter and later succeeds.
/// Guarantees: Failure retains the same live ownership, while only successful removal frees the slot and invalidates the token.
#[test]
fn failed_counter_removal_retains_ownership() {
    let path = "fixture";
    let mut counters = CounterHandles::new();
    let token = insert_fixture_counter(&mut counters, path);
    let handle = counters.handle(&token, path).unwrap();

    let error = counters
        .remove_with(&token, path, |_| PDH_INVALID_ARGUMENT)
        .unwrap_err();
    assert!(matches!(
        error,
        Error::Pdh {
            operation: PdhOperation::RemoveCounter,
            status: PDH_INVALID_ARGUMENT,
            ..
        }
    ));
    assert_eq!(counters.handle(&token, path).unwrap(), handle);
    assert!(counters.free_slots.is_empty());

    counters.remove_with(&token, path, |_| 0).unwrap();
    assert_eq!(counters.free_slots, [token.index]);
    assert!(matches!(
        counters.handle(&token, path),
        Err(Error::InvalidCounterToken { .. })
    ));
}

/// Scenario: A query reaches the end of the counter-token generation space.
/// Guarantees: Exhaustion fails permanently before native ownership is acquired or slot bookkeeping changes.
#[test]
fn counter_generation_exhaustion_is_explicit() {
    let mut counters = CounterHandles::new();
    counters.next_generation = u64::MAX;

    let error = counters.take_generation("fixture").unwrap_err();
    assert!(matches!(
        error,
        Error::CounterSetup {
            permanent: true,
            ..
        }
    ));
    assert!(error.to_string().contains("generation exhausted"));
    assert!(counters.slots.is_empty());
    assert!(counters.free_slots.is_empty());
    assert_eq!(counters.next_generation, u64::MAX);
}

/// Scenario: Native metadata APIs return malformed sizing and filling results.
/// Guarantees: Counter info, counter type, and parsed instance paths reject non-sizing status, undersized headers, invalid filled sizes, and failed fills.
#[test]
fn native_metadata_buffers_reject_malformed_size_and_fill_contracts() {
    for api in [
        BufferApi::CounterInfo,
        BufferApi::CounterType,
        BufferApi::ParseInstance,
    ] {
        for case in [
            MalformedBuffer::NonSizingStatus,
            MalformedBuffer::SuccessSizingStatus,
            MalformedBuffer::UndersizedRequest,
            MalformedBuffer::UndersizedFill,
            MalformedBuffer::OversizedFill,
            MalformedBuffer::FailedFill,
        ] {
            let error = api.malformed_error(case);
            let expected = match case {
                MalformedBuffer::NonSizingStatus => {
                    Some((api.size_operation(), PDH_INVALID_ARGUMENT))
                }
                MalformedBuffer::SuccessSizingStatus => Some((api.size_operation(), 0)),
                MalformedBuffer::FailedFill => Some((api.fill_operation(), PDH_MORE_DATA)),
                _ => None,
            };
            match expected {
                Some((expected_operation, expected_status)) => assert!(matches!(
                    error,
                    Error::Pdh { operation, status, path }
                        if operation == expected_operation
                            && status == expected_status
                            && path == "fixture"
                )),
                None => assert!(matches!(error, Error::InvalidSample(_))),
            }
        }
    }
}

/// Scenario: Metadata sizing rounds storage up, while fills report exactly the header or the full allocation.
/// Guarantees: The production helper preserves the reported fill input and accepts both valid filled-size boundaries.
#[test]
fn fixed_header_buffer_preserves_fill_size_boundaries() {
    let header = COUNTER_INFO_BUFFER.minimum_size;
    let requested = u32::try_from(header + 1).unwrap();
    let capacity = (requested as usize).div_ceil(size_of::<usize>()) * size_of::<usize>();
    for filled in [header, capacity] {
        let mut calls = 0;
        let (buffer, filled_size) =
            fixed_header_buffer_with::<usize>("fixture", &COUNTER_INFO_BUFFER, |size, buffer| {
                calls += 1;
                let Some(buffer) = buffer else {
                    assert_eq!(*size, 0);
                    *size = requested;
                    return Ok(PDH_MORE_DATA);
                };
                assert_eq!(*size, requested);
                assert_eq!(size_of_val(buffer), capacity);
                *size = u32::try_from(filled).unwrap();
                Ok(0)
            })
            .unwrap();
        assert_eq!(calls, 2);
        assert_eq!(buffer.byte_capacity(), capacity);
        assert_eq!(filled_size, filled);
    }
}

/// Scenario: Native sizing exceeds metadata capacity or returns an unaligned string.
/// Guarantees: Size violations fail before allocation and string decoding checks UTF-16 alignment.
#[test]
fn native_size_overflow_and_pointer_alignment_are_checked() {
    assert!(matches!(
        NativeBuffer::<usize>::with_byte_capacity(u32::MAX, "overflow"),
        Err(Error::InvalidSample("overflow"))
    ));
    let buffer = NativeBuffer::<usize>::with_byte_capacity(32, "fixture").unwrap();
    let pointer = buffer.as_ptr::<u8>().wrapping_add(1).cast::<u16>();
    assert!(matches!(
        NativePdh::wide_string(pointer, &buffer, 32),
        Err(Error::InvalidSample(_))
    ));
}

/// Scenario: Native metadata strings point outside filled bytes, start in allocation slack, or omit a terminator.
/// Guarantees: Instance and direct string decoding never reads beyond the native filled-size boundary.
#[test]
fn native_strings_are_bounded_by_reported_fill_size() {
    let header = size_of::<PDH_COUNTER_PATH_ELEMENTS_W>();
    let filled = u32::try_from(header + 4 * size_of::<u16>()).unwrap();
    let error = parse_instance_fixture_error(filled, |base, elements| {
        // SAFETY: The fixture buffer is aligned for the header, and the
        // pointer intentionally targets the first byte outside filled data.
        unsafe {
            (*elements).szInstanceName = base.add(filled as usize).cast();
        }
    });
    assert!(matches!(error, Error::InvalidSample(_)));

    let error = parse_instance_fixture_error(filled, |base, elements| {
        let name = unsafe { base.add(header).cast::<u16>() };
        // SAFETY: Four UTF-16 units fit in the reported fixture bytes.
        unsafe {
            (*elements).szInstanceName = name;
            for index in 0..4 {
                *name.add(index) = b'a' as u16;
            }
        }
    });
    assert!(matches!(error, Error::InvalidSample(_)));

    let mut buffer = NativeBuffer::<usize>::with_byte_capacity(
        4 * u32::try_from(size_of::<u16>()).unwrap(),
        "fixture",
    )
    .unwrap();
    let pointer = buffer.as_mut_ptr::<u16>();
    // SAFETY: The fixture buffer has room for four UTF-16 units.
    unsafe {
        *pointer = b'a' as u16;
        *pointer.add(1) = b'b' as u16;
        *pointer.add(2) = 0;
    }

    assert!(matches!(
        NativePdh::wide_string(pointer.wrapping_add(2), &buffer, 2 * size_of::<u16>()),
        Err(Error::InvalidSample(_))
    ));
    assert!(matches!(
        NativePdh::wide_string(pointer, &buffer, 2 * size_of::<u16>()),
        Err(Error::InvalidSample(_))
    ));
}

/// Scenario: Native formatting is requested for an exact large integer and a double above 100.
/// Guarantees: PDH display scaling is disabled, doubles are not capped, and the matching union field preserves each value.
#[test]
fn formatting_requests_unscaled_uncapped_values() {
    for (format, flags, number) in [
        (
            ValueFormat::Integer,
            PDH_FMT_LARGE | PDH_FMT_NOSCALE,
            Number::Integer(i64::MAX),
        ),
        (
            ValueFormat::Double,
            PDH_FMT_DOUBLE | PDH_FMT_NOSCALE | PDH_FMT_NOCAP100,
            Number::Double(250.0),
        ),
    ] {
        let result = NativePdh::formatted_with("fixture", format, |requested, value| {
            assert_eq!(requested, flags);
            value.CStatus = PDH_CSTATUS_VALID_DATA;
            match number {
                Number::Integer(number) => value.Anonymous.largeValue = number,
                Number::Double(number) => value.Anonymous.doubleValue = number,
            }
            0
        })
        .unwrap();
        assert_eq!(result, Some(number));
    }
}

/// Scenario: Windows formats successive rate, delta, timer, fraction and average samples.
/// Guarantees: Native formulas, rather than receiver approximations, preserve provider semantics and uncapped values.
#[test]
fn native_formulas_from_raw_fixtures() {
    for (native_type, first, second, previous_first, previous_second, time_base, expected) in [
        (0x1041_0400, 160, 12, 100, 10, 10, 300.0),
        (0x0040_0500, 200, 1, 100, 1, 1, 100.0),
        (0x2051_0500, 150, 200, 100, 180, 10_000_000, 250.0),
        (0x2002_0400, 25, 100, 0, 0, 1, 25.0),
        (0x20C2_0400, 160, 200, 100, 100, 1, 60.0),
        (0x3002_0400, 200, 5, 100, 3, 100, 0.5),
        (0x4002_0500, 200, 5, 100, 3, 1, 50.0),
    ] {
        let current = PDH_RAW_COUNTER {
            CStatus: PDH_CSTATUS_VALID_DATA,
            FirstValue: first,
            SecondValue: second,
            ..Default::default()
        };
        let previous = PDH_RAW_COUNTER {
            CStatus: PDH_CSTATUS_VALID_DATA,
            FirstValue: previous_first,
            SecondValue: previous_second,
            ..Default::default()
        };
        let value = NativePdh::formatted_with("fixture", ValueFormat::Double, |flags, value| {
            // SAFETY: All pointers reference initialized fixture values and writable output.
            unsafe {
                PdhFormatFromRawValue(native_type, flags, &time_base, &current, &previous, value)
            }
        })
        .unwrap();
        let Some(Number::Double(actual)) = value else {
            panic!("successful native double formatting must return a double");
        };
        assert!((actual - expected).abs() < 1e-9);
    }
}

/// Scenario: Windows formats a raw-fraction fixture whose current denominator is zero.
/// Guarantees: The receiver preserves PDH's defined zero result instead of rejecting a valid native observation.
#[test]
fn native_raw_fraction_zero_base_formats_as_zero() {
    let current = PDH_RAW_COUNTER {
        CStatus: PDH_CSTATUS_VALID_DATA,
        FirstValue: 25,
        SecondValue: 0,
        ..Default::default()
    };
    let time_base = 1;
    let value = NativePdh::formatted_with("fixture", ValueFormat::Double, |flags, value| {
        // SAFETY: Current and output point to initialized storage; raw fractions need no older sample.
        unsafe { PdhFormatFromRawValue(0x2002_0400, flags, &time_base, &current, null(), value) }
    })
    .unwrap();
    assert_eq!(value, Some(Number::Double(0.0)));
}

/// Scenario: Windows formats elapsed time from one raw sample and no previous sample.
/// Guarantees: PERF_ELAPSED_TIME uses the current object timestamp and frequency without two-sample warm-up.
#[test]
fn native_elapsed_formula_from_single_raw_fixture() {
    let current = PDH_RAW_COUNTER {
        CStatus: PDH_CSTATUS_VALID_DATA,
        FirstValue: 100,
        SecondValue: 250,
        ..Default::default()
    };
    let time_base = 10;
    let value = NativePdh::formatted_with("fixture", ValueFormat::Double, |flags, value| {
        // SAFETY: Current and output point to initialized storage; elapsed time needs no older raw sample.
        unsafe { PdhFormatFromRawValue(0x3024_0500, flags, &time_base, &current, null(), value) }
    })
    .unwrap();
    assert_eq!(value, Some(Number::Double(15.0)));
}

/// Scenario: Windows formats all four normal/inverse system-clock and 100-nanosecond multi-timer SDK types.
/// Guarantees: Fixture coverage protects every supported formula even when the local provider exposes only a subset.
#[test]
fn native_multi_timer_formulas_from_raw_fixtures() {
    for (
        native_type,
        first,
        second,
        previous_first,
        previous_second,
        time_base,
        multi_count,
        expected,
    ) in [
        (0x2241_0500, 160, 12, 100, 10, 10, 4, 750.0),
        (0x2341_0500, 102, 20, 100, 10, 10, 4, 380.0),
        (0x2251_0500, 150, 200, 100, 180, 10_000_000, 4, 62.5),
        (0x2351_0500, 150, 200, 100, 180, 10_000_000, 4, 150.0),
    ] {
        let current = PDH_RAW_COUNTER {
            CStatus: PDH_CSTATUS_VALID_DATA,
            FirstValue: first,
            SecondValue: second,
            MultiCount: multi_count,
            ..Default::default()
        };
        let previous = PDH_RAW_COUNTER {
            CStatus: PDH_CSTATUS_VALID_DATA,
            FirstValue: previous_first,
            SecondValue: previous_second,
            MultiCount: multi_count,
            ..Default::default()
        };
        let value = NativePdh::formatted_with("fixture", ValueFormat::Double, |flags, value| {
            // SAFETY: All pointers reference initialized fixture values and writable output.
            unsafe {
                PdhFormatFromRawValue(native_type, flags, &time_base, &current, &previous, value)
            }
        })
        .unwrap();
        let Some(Number::Double(actual)) = value else {
            panic!("successful native double formatting must return a double");
        };
        assert!((actual - expected).abs() < 1e-9);
    }
}

/// Scenario: Query collection returns each documented transient status or a hard failure.
/// Guarantees: Transients become no observations while unrelated failures remain errors.
#[test]
fn collection_statuses_distinguish_omissions_from_failures() {
    for status in [
        PDH_INVALID_DATA,
        PDH_NO_DATA,
        PDH_CSTATUS_NO_INSTANCE,
        PDH_CALC_NEGATIVE_VALUE,
        PDH_CALC_NEGATIVE_DENOMINATOR,
        PDH_CALC_NEGATIVE_TIMEBASE,
    ] {
        assert!(!NativePdh::collect_status(status).unwrap());
    }
    assert!(NativePdh::collect_status(PDH_INVALID_ARGUMENT).is_err());
    assert!(NativePdh::collect_status(0).unwrap());
}

/// Scenario: Raw reads return transient API statuses, transient data statuses, and valid bases.
/// Guarantees: Only checked valid data crosses the native boundary and all omissions return None.
#[test]
fn raw_statuses_return_checked_optional_bases() {
    for status in [
        PDH_INVALID_DATA,
        PDH_NO_DATA,
        PDH_CSTATUS_NO_INSTANCE,
        PDH_CALC_NEGATIVE_VALUE,
        PDH_CALC_NEGATIVE_DENOMINATOR,
        PDH_CALC_NEGATIVE_TIMEBASE,
    ] {
        assert_eq!(NativePdh::raw_with("fixture", |_| status).unwrap(), None);
    }
    for status in [
        PDH_CSTATUS_INVALID_DATA,
        PDH_NO_DATA,
        PDH_CSTATUS_NO_INSTANCE,
        PDH_CALC_NEGATIVE_VALUE,
        PDH_CALC_NEGATIVE_DENOMINATOR,
        PDH_CALC_NEGATIVE_TIMEBASE,
    ] {
        assert_eq!(
            NativePdh::raw_with("fixture", |raw| {
                raw.CStatus = status;
                0
            })
            .unwrap(),
            None
        );
    }
    assert_eq!(
        NativePdh::raw_with("fixture", |raw| {
            raw.CStatus = PDH_CSTATUS_VALID_DATA;
            raw.SecondValue = 42;
            raw.MultiCount = 3;
            0
        })
        .unwrap(),
        Some(RawSample {
            second_value: 42,
            multi_count: 3,
        })
    );
    assert!(NativePdh::raw_with("fixture", |_| PDH_INVALID_ARGUMENT).is_err());
}

/// Scenario: Formatted reads combine an API status with a more specific counter data status.
/// Guarantees: Transient data omits points, specific hard data failures are preserved, and valid values decode.
#[test]
fn formatted_statuses_preserve_specific_counter_failures() {
    for status in [
        PDH_CSTATUS_INVALID_DATA,
        PDH_NO_DATA,
        PDH_CSTATUS_NO_INSTANCE,
        PDH_CALC_NEGATIVE_VALUE,
        PDH_CALC_NEGATIVE_DENOMINATOR,
        PDH_CALC_NEGATIVE_TIMEBASE,
    ] {
        assert_eq!(
            NativePdh::formatted_with("fixture", ValueFormat::Integer, |_, value| {
                value.CStatus = status;
                PDH_INVALID_DATA
            })
            .unwrap(),
            None
        );
    }
    let error = NativePdh::formatted_with("fixture", ValueFormat::Integer, |_, value| {
        value.CStatus = PDH_INVALID_ARGUMENT;
        PDH_INVALID_DATA
    })
    .unwrap_err();
    assert!(matches!(
        error,
        Error::CounterData {
            operation: PdhOperation::FormattedCounterDataStatus,
            status: PDH_INVALID_ARGUMENT,
            ..
        }
    ));
    assert_eq!(
        NativePdh::formatted_with("fixture", ValueFormat::Integer, |_, value| {
            value.CStatus = PDH_CSTATUS_NEW_DATA;
            value.Anonymous.largeValue = 42;
            0
        })
        .unwrap(),
        Some(Number::Integer(42))
    );
}

/// Scenario: Wildcard expansion returns a valid multi-string or malformed termination.
/// Guarantees: Every concrete path is decoded exactly and malformed native buffers fail.
#[test]
fn parses_expanded_path_multi_string() {
    let mut buffer = Vec::new();
    for path in [
        r"\Process(worker)\Private Bytes",
        r"\Process(worker#1)\Private Bytes",
    ] {
        buffer.extend(path.encode_utf16());
        buffer.push(0);
    }
    buffer.push(0);
    assert_eq!(
        NativePdh::parse_multi_sz(&buffer).unwrap(),
        vec![
            r"\Process(worker)\Private Bytes",
            r"\Process(worker#1)\Private Bytes"
        ]
    );
    assert!(NativePdh::parse_multi_sz(&[b'a' as u16]).is_err());
}

/// Scenario: Wildcard membership grows after PDH's size measurement and exceeds the initial headroom.
/// Guarantees: Expansion grows geometrically from the reported fill size and returns the later complete multi-string.
#[test]
fn wildcard_expansion_retries_growing_membership() {
    let mut calls = 0;
    let buffer = NativePdh::expanded_path_buffer_with("fixture", |size, buffer| {
        calls += 1;
        Ok(match buffer {
            None => {
                *size = 4;
                PDH_MORE_DATA
            }
            Some(_) if calls == 2 => {
                *size = 128;
                PDH_MORE_DATA
            }
            Some(buffer) => {
                buffer[..3].copy_from_slice(&[b'a' as u16, 0, 0]);
                *size = 3;
                0
            }
        })
    })
    .unwrap();

    assert_eq!(calls, 3);
    assert_eq!(buffer, [b'a' as u16, 0, 0]);
}

/// Scenario: Wildcard sizing or a growth response after several smaller growth reports exceeds the buffer budget.
/// Guarantees: Expansion fails before allocating the oversized buffer or issuing another fill call.
#[test]
fn wildcard_expansion_rejects_oversized_sizing_and_repeated_growth() {
    let oversized = u32::try_from(64 * 1024 * 1024 / size_of::<u16>() + 1).unwrap();
    for oversized_sizing in [false, true] {
        let mut calls = 0;
        let error = NativePdh::expanded_path_buffer_with("fixture", |size, buffer| {
            calls += 1;
            assert!(calls <= if oversized_sizing { 1 } else { 5 });
            if buffer.is_none() && !oversized_sizing {
                *size = 4;
            } else if !oversized_sizing && calls < 5 {
                *size = u32::try_from(buffer.unwrap().len() + 1).unwrap();
            } else {
                assert_eq!(buffer.is_none(), oversized_sizing);
                *size = oversized;
            }
            Ok(PDH_MORE_DATA)
        })
        .unwrap_err();
        assert!(matches!(error, Error::DiscoveryBufferLimit));
        assert_eq!(error.reason(), "discovery_buffer_limit");
        assert_eq!(
            error.to_string(),
            "wildcard path list exceeds 64 MiB discovery buffer limit"
        );
        assert_eq!(calls, if oversized_sizing { 1 } else { 5 });
    }
}

/// Scenario: Repeated native growth fits the peak budget but exceeds it if freed buffers accumulate.
/// Guarantees: Superseded buffers do not consume the next attempt's budget, and decoding releases the final raw capacity.
#[test]
fn wildcard_growth_releases_superseded_buffers() {
    let mut budget = super::super::expansion::DiscoveryBudget::limited(700);
    let mut calls = 0;
    let buffer =
        NativePdh::expanded_path_buffer_budgeted_with("fixture", &mut budget, |size, buffer| {
            calls += 1;
            assert!(calls <= 4);
            if calls == 4 {
                buffer.unwrap()[..3].copy_from_slice(&[b'a' as u16, 0, 0]);
                *size = 3;
                return Ok(0);
            }
            *size = if let Some(buffer) = buffer {
                u32::try_from(buffer.len() + 1).unwrap()
            } else {
                4
            };
            Ok(PDH_MORE_DATA)
        })
        .unwrap();
    assert_eq!(calls, 4);
    assert_eq!(
        NativePdh::parse_discovery_paths(buffer, &mut budget, || Ok(())).unwrap(),
        ["a"]
    );
    budget.reserve(699).unwrap();
}

/// Scenario: One selector's decoded paths remain live while a later selector decodes another native list.
/// Guarantees: Retained UTF-8 bytes and the current raw buffer share the budget, and a failed selector refunds its allocations.
#[test]
fn decoded_discovery_paths_share_budget_across_selectors() {
    let mut budget = super::super::expansion::DiscoveryBudget::limited(27);
    let buffer: Vec<u16> = "member\0\0".encode_utf16().collect();
    let retained =
        NativePdh::parse_discovery_paths(buffer.clone(), &mut budget, || Ok(())).unwrap();
    let error = NativePdh::parse_discovery_paths(buffer, &mut budget, || Ok(())).unwrap_err();
    assert!(matches!(error, Error::DiscoveryMemoryLimit));
    assert_eq!(retained, ["member"]);
    budget.reserve(21).unwrap();
}

/// Scenario: Native paths contain multibyte or invalid UTF-16, or a later list entry is malformed.
/// Guarantees: Charges match lossy UTF-8 output bytes exactly, and discarded partial results leave the budget unchanged.
#[test]
fn decoded_discovery_budget_matches_utf8_and_refunds_malformed_lists() {
    let mut budget = super::super::expansion::DiscoveryBudget::limited(64);
    let paths =
        NativePdh::parse_discovery_paths(vec![0xd800, 0x00e9, 0, 0], &mut budget, || Ok(()))
            .unwrap();
    assert_eq!(paths, ["\u{fffd}\u{e9}"]);
    budget.reserve(59).unwrap();
    assert!(matches!(
        budget.reserve(1),
        Err(Error::DiscoveryMemoryLimit)
    ));

    let mut budget = super::super::expansion::DiscoveryBudget::limited(64);
    let error =
        NativePdh::parse_discovery_paths(vec![b'a' as u16, 0, b'b' as u16], &mut budget, || Ok(()))
            .unwrap_err();
    assert!(matches!(error, Error::InvalidSample(_)));
    budget.reserve(64).unwrap();
}

/// Scenario: Twenty selectors each expand to 2,000 process paths, including multibyte counter names.
/// Guarantees: All 40,000 decoded members fit the default refresh limit and only their actual UTF-8 bytes remain charged.
#[test]
fn discovery_budget_accepts_twenty_selectors_with_two_thousand_instances() {
    let mut budget = super::super::expansion::DiscoveryBudget::new();
    let mut retained = Vec::new();
    let mut decoded_bytes = 0;
    for selector in 0..20 {
        let expected: Vec<_> = (0..2000)
            .map(|member| {
                format!(
                    r"\Process(worker-{member:04})\Counter-{selector}-{}",
                    "V\u{00e4}lue"
                )
            })
            .collect();
        let native: Vec<_> = expected
            .iter()
            .flat_map(|path| path.encode_utf16().chain(Some(0)))
            .chain(Some(0))
            .collect();
        let buffer = NativePdh::expanded_path_buffer_budgeted_with(
            "fixture",
            &mut budget,
            |size, buffer| {
                *size = u32::try_from(native.len()).unwrap();
                if let Some(buffer) = buffer {
                    buffer[..native.len()].copy_from_slice(&native);
                    Ok(0)
                } else {
                    Ok(PDH_MORE_DATA)
                }
            },
        )
        .unwrap();
        let paths = NativePdh::parse_discovery_paths(buffer, &mut budget, || Ok(())).unwrap();
        assert_eq!(paths, expected);
        decoded_bytes += paths.iter().map(String::len).sum::<usize>();
        retained.push(paths);
    }
    assert_eq!(retained.iter().map(Vec::len).sum::<usize>(), 40_000);
    budget
        .reserve(super::super::expansion::DISCOVERY_REFRESH_BYTES - decoded_bytes)
        .unwrap();
    assert!(matches!(
        budget.reserve(1),
        Err(Error::DiscoveryMemoryLimit)
    ));
}

/// Scenario: PDH reports that a valid wildcard currently has no matching instances.
/// Guarantees: Both documented no-instance status and successful zero-size responses become an empty membership set.
#[test]
fn wildcard_expansion_accepts_zero_matches() {
    for (status, reported) in [
        (0, 0),
        (PDH_CSTATUS_NO_INSTANCE, 0),
        (PDH_CSTATUS_NO_INSTANCE, 37),
    ] {
        let mut calls = 0;
        let buffer = NativePdh::expanded_path_buffer_with("fixture", |size, buffer| {
            calls += 1;
            assert!(buffer.is_none());
            *size = reported;
            Ok(status)
        })
        .unwrap();
        assert!(buffer.is_empty());
        assert_eq!(calls, 1);
    }
}
