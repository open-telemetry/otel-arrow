// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Native metric-view parent association and attribute iteration regressions.

#[path = "common/metrics_view.rs"]
mod fixture;

#[global_allocator]
static ALLOCATOR: fixture::allocations::CountingAllocator = fixture::allocations::CountingAllocator;

use arrow::{
    array::{
        ArrayRef, DictionaryArray, Int64Array, RecordBatch, StringArray, UInt8Array, UInt32Array,
    },
    datatypes::UInt8Type,
};
use fixture::{KEYS, Layout};
use otel_arrow_dfe_pdata::views::otap::OtapMetricsView;
use otel_arrow_dfe_pdata::{proto::opentelemetry::arrow::v1::ArrowPayloadType, schema::consts};
use otel_arrow_dfe_pdata_views::views::{
    common::{AnyValueView, AttributeView},
    metrics::*,
};
use std::sync::Arc;

/// Scenario: Gauge and counter attributes are grouped, scattered or appended after grouped rows.
/// Guarantees: Sparse/descending IDs, dictionary parents and ID zero retain exact values and per-point attribute order across resources/scopes.
#[test]
fn numeric_attributes_preserve_parent_associations_and_order() {
    for points in [0, 1, 7, 8192] {
        for layout in [Layout::Grouped, Layout::Scattered, Layout::Mixed] {
            for dictionary in [false, true] {
                let records = fixture::records(points, layout, dictionary);
                let view =
                    OtapMetricsView::try_from(&records).expect("valid synthetic metrics fixture");
                let mut seen = Vec::new();
                for resource in view.resources() {
                    for scope in resource.scopes() {
                        for metric in scope.metrics() {
                            let data = metric.data().expect("valid synthetic metrics fixture");
                            if let Some(gauge) = data.as_gauge() {
                                for point in gauge.data_points() {
                                    check_point(&point, points, &mut seen);
                                }
                            } else {
                                let counter =
                                    data.as_sum().expect("valid synthetic metrics fixture");
                                assert!(counter.is_monotonic());
                                assert_eq!(
                                    counter.aggregation_temporality(),
                                    AggregationTemporality::Cumulative
                                );
                                for point in counter.data_points() {
                                    check_point(&point, points, &mut seen);
                                }
                            }
                        }
                    }
                }
                seen.sort_unstable();
                assert_eq!(
                    seen,
                    (0..points).collect::<Vec<_>>(),
                    "{layout:?}, dictionary={dictionary}"
                );
            }
        }
    }
}

/// Scenario: Attribute parent runs are ascending or grouped in arbitrary order.
/// Guarantees: Both ordered runs and the general fallback retain exact values, parent associations and per-point encounter order.
#[test]
fn numeric_attributes_handle_ascending_and_unordered_parent_runs() {
    for order in [[6, 5, 4, 3, 2, 1, 0], [2, 0, 4, 1, 6, 3, 5]] {
        for dictionary in [false, true] {
            let mut records = fixture::records(7, Layout::Grouped, dictionary);
            let attributes = records
                .get(ArrowPayloadType::NumberDpAttrs)
                .expect("number attributes exist");
            let indices = UInt32Array::from_iter_values(order.into_iter().flat_map(|row| {
                let start = row * KEYS.len() as u32;
                start..start + KEYS.len() as u32
            }));
            let reordered = arrow::compute::take_record_batch(attributes, &indices)
                .expect("valid attribute row permutation");
            records
                .set(ArrowPayloadType::NumberDpAttrs, reordered)
                .expect("valid reordered attributes");
            let view =
                OtapMetricsView::try_from(&records).expect("valid synthetic metrics fixture");
            let mut seen = Vec::new();
            for resource in view.resources() {
                for scope in resource.scopes() {
                    for metric in scope.metrics() {
                        let data = metric.data().expect("numeric metric data exists");
                        if let Some(gauge) = data.as_gauge() {
                            for point in gauge.data_points() {
                                check_point(&point, 7, &mut seen);
                            }
                        } else {
                            for point in data.as_sum().expect("counter data exists").data_points() {
                                check_point(&point, 7, &mut seen);
                            }
                        }
                    }
                }
            }
            seen.sort_unstable();
            assert_eq!(seen, (0..7).collect::<Vec<_>>());
        }
    }
}

/// Scenario: An ordered attribute column contains only two of five data-point parents.
/// Guarantees: Lookups before, between and after the available IDs are empty, while both existing parents retain their attributes in either sort direction.
#[test]
fn numeric_attributes_handle_missing_parents_in_ordered_columns() {
    for order in [[1, 3], [3, 1]] {
        for dictionary in [false, true] {
            let mut records = fixture::records(5, Layout::Grouped, dictionary);
            let indices = UInt32Array::from_iter_values(order.into_iter().flat_map(|row| {
                let start = row * KEYS.len() as u32;
                start..start + KEYS.len() as u32
            }));
            let attributes = arrow::compute::take_record_batch(
                records
                    .get(ArrowPayloadType::NumberDpAttrs)
                    .expect("number attributes exist"),
                &indices,
            )
            .expect("valid subset of attribute rows");
            records
                .set(ArrowPayloadType::NumberDpAttrs, attributes)
                .expect("valid subset of parents");
            let view =
                OtapMetricsView::try_from(&records).expect("valid synthetic metrics fixture");
            let mut seen = Vec::new();
            for resource in view.resources() {
                for scope in resource.scopes() {
                    for metric in scope.metrics() {
                        let data = metric.data().expect("numeric metric data exists");
                        if let Some(gauge) = data.as_gauge() {
                            for point in gauge.data_points() {
                                check_sparse_point(&point, &mut seen);
                            }
                        } else {
                            for point in data.as_sum().expect("counter data exists").data_points() {
                                check_sparse_point(&point, &mut seen);
                            }
                        }
                    }
                }
            }
            seen.sort_unstable();
            assert_eq!(seen, (0..5).collect::<Vec<_>>());
        }
    }
}

/// Scenario: Equivalent 8,192-point metric batches have grouped or scattered attributes.
/// Guarantees: Constructing fresh views for ordered parents makes fewer thread-scoped allocation requests than the general scattered path.
#[test]
fn ordered_attribute_views_request_fewer_allocations() {
    const ITERATIONS: usize = 3;
    let ordered = fixture::records(8192, Layout::Grouped, false);
    let scattered = fixture::records(8192, Layout::Scattered, false);

    for records in [&ordered, &scattered] {
        drop(OtapMetricsView::try_from(records).expect("valid synthetic metrics fixture"));
    }

    let (_, ordered_calls, _) = fixture::allocations::measure(|| {
        for _ in 0..ITERATIONS {
            drop(std::hint::black_box(
                OtapMetricsView::try_from(std::hint::black_box(&ordered))
                    .expect("valid synthetic metrics fixture"),
            ));
        }
    });
    let (_, scattered_calls, _) = fixture::allocations::measure(|| {
        for _ in 0..ITERATIONS {
            drop(std::hint::black_box(
                OtapMetricsView::try_from(std::hint::black_box(&scattered))
                    .expect("valid synthetic metrics fixture"),
            ));
        }
    });

    assert!(
        ordered_calls < scattered_calls,
        "ordered views requested {ordered_calls} allocations; scattered views requested {scattered_calls}"
    );
}

fn check_sparse_point(point: &impl NumberDataPointView, seen: &mut Vec<usize>) {
    let Some(Value::Integer(id)) = point.value() else {
        panic!("missing integer point");
    };
    if matches!(id, 111 | 37) {
        check_point(point, 5, seen);
    } else {
        assert_eq!(point.attributes().count(), 0, "unlinked parent {id}");
        seen.push(5 - usize::try_from(id / 37).expect("valid point ID") - 1);
    }
}

fn check_point(point: &impl NumberDataPointView, points: usize, seen: &mut Vec<usize>) {
    let Some(Value::Integer(id)) = point.value() else {
        panic!("missing integer point");
    };
    let row = points - usize::try_from(id / 37).expect("valid synthetic metrics fixture") - 1;
    seen.push(row);
    let attributes: Vec<_> = point
        .attributes()
        .map(|attr| {
            (
                std::str::from_utf8(attr.key())
                    .expect("valid synthetic metrics fixture")
                    .to_owned(),
                attr.value()
                    .expect("valid synthetic metrics fixture")
                    .as_int64()
                    .expect("valid synthetic metrics fixture"),
            )
        })
        .collect();
    let expected: Vec<_> = KEYS
        .iter()
        .enumerate()
        .map(|(key, name)| ((*name).to_owned(), id * 16 + key as i64))
        .collect();
    assert_eq!(attributes, expected, "point {id}");
}

/// Scenario: Orphan parents and dictionary null values interrupt a point's attribute rows.
/// Guarantees: Unlinked rows are skipped without losing later attributes, and ID zero remains a valid parent.
#[test]
fn numeric_attributes_skip_nulls_without_losing_later_rows() {
    for (dictionary, null_first) in [(false, false), (true, false), (true, true)] {
        let mut records = fixture::records(3, Layout::Grouped, false);
        let parents: ArrayRef = if dictionary {
            Arc::new(DictionaryArray::<UInt8Type>::new(
                UInt8Array::from(vec![0, 4, 0, 1, 3, 2, 0]),
                Arc::new(UInt32Array::from(vec![
                    Some(74),
                    Some(37),
                    Some(0),
                    None,
                    Some(2222),
                ])),
            ))
        } else {
            Arc::new(UInt32Array::from(vec![74, 2222, 74, 37, 3333, 0, 74]))
        };
        let attributes = RecordBatch::try_from_iter([
            (consts::PARENT_ID, parents),
            (
                consts::ATTRIBUTE_KEY,
                Arc::new(StringArray::from(vec![
                    "first", "orphan", "again", "other", "unlinked", "zero", "last",
                ])),
            ),
            (
                consts::ATTRIBUTE_TYPE,
                Arc::new(UInt8Array::from(vec![2; 7])),
            ),
            (
                consts::ATTRIBUTE_INT,
                Arc::new(Int64Array::from_iter_values(10..17)),
            ),
        ])
        .expect("valid synthetic metrics fixture");
        let attributes = if null_first {
            arrow::compute::take_record_batch(
                &attributes,
                &UInt32Array::from(vec![4, 0, 1, 2, 3, 5, 6]),
            )
            .expect("move logical null parent to the first row")
        } else {
            attributes
        };
        records
            .set(ArrowPayloadType::NumberDpAttrs, attributes)
            .expect("valid synthetic metrics fixture");
        let view = OtapMetricsView::try_from(&records).expect("valid synthetic metrics fixture");
        let mut seen = 0;
        for resource in view.resources() {
            for scope in resource.scopes() {
                for metric in scope.metrics() {
                    let data = metric.data().expect("valid synthetic metrics fixture");
                    if let Some(gauge) = data.as_gauge() {
                        for point in gauge.data_points() {
                            check_nullable_point(&point);
                            seen += 1;
                        }
                    } else {
                        for point in data
                            .as_sum()
                            .expect("valid synthetic metrics fixture")
                            .data_points()
                        {
                            check_nullable_point(&point);
                            seen += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(seen, 3);
    }
}

/// Scenario: Attributes are absent/empty, or a number point has a null ID while another point has ID zero.
/// Guarantees: Unlinked points expose no attributes and never inherit the attributes of point zero.
#[test]
fn numeric_attributes_handle_absent_payloads_and_null_point_ids() {
    for case in 0..3 {
        let mut records = fixture::records(3, Layout::Grouped, false);
        match case {
            0 => {
                let _ = records.remove(ArrowPayloadType::NumberDpAttrs);
            }
            1 => {
                let empty = records
                    .get(ArrowPayloadType::NumberDpAttrs)
                    .expect("valid synthetic metrics fixture")
                    .slice(0, 0);
                records
                    .set(ArrowPayloadType::NumberDpAttrs, empty)
                    .expect("valid synthetic metrics fixture");
            }
            _ => {
                let numbers = records
                    .get(ArrowPayloadType::NumberDataPoints)
                    .expect("valid synthetic metrics fixture");
                let ids: ArrayRef = Arc::new(UInt32Array::from(vec![Some(74), None, Some(0)]));
                let replacement = RecordBatch::try_from_iter(
                    numbers.schema().fields().iter().zip(numbers.columns()).map(
                        |(field, column)| {
                            (
                                field.name().as_str(),
                                if field.name() == consts::ID {
                                    Arc::clone(&ids)
                                } else {
                                    Arc::clone(column)
                                },
                            )
                        },
                    ),
                )
                .expect("valid synthetic metrics fixture");
                records
                    .set(ArrowPayloadType::NumberDataPoints, replacement)
                    .expect("valid synthetic metrics fixture");
            }
        }
        let view = OtapMetricsView::try_from(&records).expect("valid synthetic metrics fixture");
        let mut seen = 0;
        for resource in view.resources() {
            for scope in resource.scopes() {
                for metric in scope.metrics() {
                    let data = metric.data().expect("valid synthetic metrics fixture");
                    if let Some(gauge) = data.as_gauge() {
                        for point in gauge.data_points() {
                            assert_eq!(
                                point.attributes().count(),
                                if case == 2 { KEYS.len() } else { 0 }
                            );
                            seen += 1;
                        }
                    } else {
                        for point in data
                            .as_sum()
                            .expect("valid synthetic metrics fixture")
                            .data_points()
                        {
                            assert_eq!(point.value(), Some(Value::Integer(37)));
                            assert_eq!(point.attributes().count(), 0);
                            seen += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(seen, 3);
    }
}

fn check_nullable_point(point: &impl NumberDataPointView) {
    let expected: &[(&[u8], i64)] = match point.value().expect("valid synthetic metrics fixture") {
        Value::Integer(74) => &[(b"first", 10), (b"again", 12), (b"last", 16)],
        Value::Integer(37) => &[(b"other", 13)],
        Value::Integer(0) => &[(b"zero", 15)],
        _ => panic!("unexpected point"),
    };
    let actual: Vec<_> = point
        .attributes()
        .map(|attr| {
            (
                attr.key().to_vec(),
                attr.value()
                    .expect("valid synthetic metrics fixture")
                    .as_int64()
                    .expect("valid synthetic metrics fixture"),
            )
        })
        .collect();
    let expected: Vec<_> = expected
        .iter()
        .map(|(key, value)| (key.to_vec(), *value))
        .collect();
    assert_eq!(actual, expected);
}
