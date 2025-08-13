// SPDX-License-Identifier: Apache-2.0

//! Multivariate metrics for the Perf Exporter

use crate::descriptor::{MetricsDescriptor, MetricsField, MetricsKind};
const METRIC_SET: &[MetricsField] = &[
    MetricsField {
        name: "bytes.total",
        unit: "bytes",
        kind: MetricsKind::Counter,
    },
    MetricsField {
        name: "pdata.messages",
        unit: "count",
        kind: MetricsKind::Counter,
    },
    MetricsField {
        name: "logs",
        unit: "count",
        kind: MetricsKind::Counter,
    },
    MetricsField {
        name: "spans",
        unit: "count",
        kind: MetricsKind::Counter,
    },
    MetricsField {
        name: "metrics",
        unit: "count",
        kind: MetricsKind::Counter,
    },
];

const MULTIVARIATE_METRICS: MetricsDescriptor = MetricsDescriptor {
    name: "otap_perf_exporter",
    fields: METRIC_SET,
};
