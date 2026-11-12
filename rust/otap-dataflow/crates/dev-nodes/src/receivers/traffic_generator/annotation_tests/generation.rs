// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Generation tests: attribute and signal generation, the vendored registry
//! representatives, annotations-disabled behavior, compiled-model parity, and
//! limits.
//!
//! Shared helpers live in the parent module.

use super::*;

// Attribute generation
// ===========================================================================

/// Scenario: Annotated scalar attributes of each type are generated.
/// Guarantees: Generated values have the attribute's type and respect the
/// annotation's values or range.
#[test]
fn attribute_value_scalars() {
    let mut r = rng();
    let int = &attribute(
        "server.port",
        primitive(PrimitiveOrArrayTypeSpec::Int),
        Some(
            "{ range: { min: 1024, max: 2048 }, distribution: { type: normal, mean: 1500, stddev: 300 } }",
        ),
    );
    let double = &attribute(
        "ratio",
        primitive(PrimitiveOrArrayTypeSpec::Double),
        Some("{ range: { min: 0, max: 1 } }"),
    );
    let boolean = &attribute(
        "flag",
        primitive(PrimitiveOrArrayTypeSpec::Boolean),
        Some("{ values: [false] }"),
    );
    for _ in 0..2_000 {
        match kv_scalar(&gen_kv(int, &mut r)) {
            Scalar::Int(i) => assert!((1024..=2048).contains(&i)),
            other => panic!("{other:?}"),
        }
        match kv_scalar(&gen_kv(double, &mut r)) {
            Scalar::Double(d) => assert!((0.0..=1.0).contains(&d)),
            other => panic!("{other:?}"),
        }
        assert_eq!(kv_scalar(&gen_kv(boolean, &mut r)), Scalar::Bool(false));
    }
}

/// Scenario: Array and template attributes are generated from annotations.
/// Guarantees: Array lengths and items stay within their specs and template
/// keys expand to `<prefix>.<key>`.
#[test]
fn attribute_value_arrays_and_templates() {
    let mut r = rng();
    let arr = &attribute(
        "process.command_args",
        primitive(PrimitiveOrArrayTypeSpec::Strings),
        Some("{ length: { range: { min: 1, max: 4 } }, items: { values: [a, b] } }"),
    );
    let tmpl = &attribute(
        "http.request.header",
        AttributeType::Template(TemplateTypeSpec::Strings),
        Some(
            "{ keys: { values: [content-type, accept] }, value: { length: { values: [2] }, items: { values: [x] } } }",
        ),
    );
    let mut keys = HashSet::new();
    for _ in 0..500 {
        let kv = gen_kv(arr, &mut r);
        let Some(AnyValueKind::ArrayValue(a)) = kv.value.as_ref().and_then(|v| v.value.as_ref())
        else {
            panic!("expected array")
        };
        assert!((1..=4).contains(&a.values.len()));
        assert!(
            a.values
                .iter()
                .all(|v| matches!(any_scalar(v), Scalar::String(s) if s == "a" || s == "b"))
        );

        let kv = gen_kv(tmpl, &mut r);
        let _ = keys.insert(kv.key.clone());
        let Some(AnyValueKind::ArrayValue(a)) = kv.value.as_ref().and_then(|v| v.value.as_ref())
        else {
            panic!("expected array")
        };
        assert_eq!(a.values.len(), 2);
    }
    assert_eq!(
        keys,
        HashSet::from([
            "http.request.header.content-type".to_owned(),
            "http.request.header.accept".to_owned()
        ])
    );
}

/// Scenario: Enum attributes are generated with and without annotations.
/// Guarantees: Annotated enums only produce the weighted subset; unannotated
/// enums always produce the first member, matching the behavior before
/// annotations were introduced.
#[test]
fn attribute_value_enums() {
    let mut r = rng();
    let ty = enum_type(&["GET", "POST", "DELETE", "PUT"]);
    let annotated = &attribute(
        "http.request.method",
        ty.clone(),
        Some("{ values: [GET, POST], weights: [9, 1] }"),
    );
    let plain = attribute("m", ty, None);
    let mut a = HashSet::new();
    let mut p = HashSet::new();
    for _ in 0..1_000 {
        let _ = a.insert(format!("{:?}", kv_scalar(&gen_kv(annotated, &mut r))));
        let _ = p.insert(format!("{:?}", kv_scalar(&gen_kv(&plain, &mut r))));
    }
    assert_eq!(a.len(), 2);
    assert_eq!(
        p,
        HashSet::from([format!("{:?}", Scalar::String("GET".into()))]),
        "unannotated enum emits the first member"
    );
}

/// Scenario: An attribute has no annotation but has examples.
/// Guarantees: Values fall back to the examples.
#[test]
fn attribute_value_falls_back_to_examples() {
    let mut r = rng();
    let mut attr = attribute("x", primitive(PrimitiveOrArrayTypeSpec::Int), None);
    attr.examples = Some(Examples::Ints(vec![7, 9]));
    for _ in 0..100 {
        assert!(matches!(
            kv_scalar(&gen_kv(&attr, &mut r)),
            Scalar::Int(7 | 9)
        ));
    }
}

/// Scenario: A string attribute with two `examples` is annotated with
/// `weights: [9, 1]` and no `values`, then compiled and sampled with a seed.
/// Guarantees: Only the example values are generated and their frequencies
/// follow the weights (the first example is drawn roughly 90% of the time).
#[test]
fn attribute_weights_apply_to_examples() {
    let mut r = rng();
    let mut attr = attribute(
        "jvm.gc.action",
        primitive(PrimitiveOrArrayTypeSpec::String),
        Some("{ weights: [9, 1] }"),
    );
    attr.examples = Some(Examples::Strings(vec!["minor".into(), "major".into()]));
    let g = AttributeGen::try_new(&attr).expect("weights over examples compile");
    let mut counts: HashMap<String, usize> = HashMap::new();
    const N: usize = 10_000;
    for _ in 0..N {
        match kv_scalar(&g.sample(&mut r)) {
            Scalar::String(s) => *counts.entry(s).or_default() += 1,
            other => panic!("expected string, got {other:?}"),
        }
    }
    assert_eq!(
        counts.keys().cloned().collect::<HashSet<_>>(),
        HashSet::from(["minor".to_owned(), "major".to_owned()])
    );
    let share = counts["minor"] as f64 / N as f64;
    assert!((0.87..=0.93).contains(&share), "minor share {share}");
}

/// Scenario: An attribute that has examples carries an invalid annotation (a
/// continuous range on a string attribute) and is compiled.
/// Guarantees: Compilation fails with the annotation key path instead of
/// silently falling back to the examples, so an invalid annotation always
/// fails receiver startup.
#[test]
fn attribute_with_invalid_annotation_fails_compilation() {
    let mut attr = attribute(
        "x",
        primitive(PrimitiveOrArrayTypeSpec::String),
        Some("{ range: { min: 0, max: 1 } }"),
    );
    attr.examples = Some(Examples::Strings(vec!["fallback".to_owned()]));
    expect_err(
        AttributeGen::try_new(&attr),
        &format!("{}.range", annotation::NAMESPACE),
        "continuous form is not allowed for string",
    );
}

/// Scenario: A registry containing invalid attribute and group annotations is
/// validated.
/// Guarantees: `SemconvModel::compile` reports every invalid annotation with
/// its group, attribute, and key path, so startup fails with a complete list.
#[test]
fn validate_registry_reports_every_error() {
    let mut bad = registry().clone();
    let span = bad
        .groups
        .iter_mut()
        .find(|g| g.r#type == GroupType::Span)
        .expect("a span group");
    let mut ann = span.annotations.clone().unwrap_or_default();
    let _ = ann.insert(
        annotation::NAMESPACE.to_owned(),
        YamlValue(yaml_from_str("{ severity: { values: [INFO] } }")),
    );
    span.annotations = Some(ann);
    let span_id = span.id.clone();
    span.attributes.push(attribute(
        "bad.attr",
        primitive(PrimitiveOrArrayTypeSpec::String),
        Some("{ range: { min: 0, max: 1 } }"),
    ));

    let errors = SemconvModel::compile(&bad).expect_err("invalid annotations");
    assert_eq!(errors.len(), 2, "{}", annotation::format_errors(&errors));
    assert!(errors.iter().any(|e| {
        e.group == span_id
            && e.attribute.is_none()
            && e.error
                .message
                .contains("unknown key \"severity\" for span group")
    }));
    assert!(errors.iter().any(|e| e.group == span_id
        && e.attribute.as_deref() == Some("bad.attr")
        && e.error.path == format!("{}.range", annotation::NAMESPACE)));
    assert!(
        annotation::format_errors(&errors)
            .starts_with("2 invalid traffic-generator annotation(s):")
    );
}

// ===========================================================================
// Signals
// ===========================================================================

/// Scenario: Observations are assigned to OTLP explicit buckets.
/// Guarantees: Buckets are upper-inclusive and values above the last bound
/// land in the overflow bucket at index `bounds.len()`.
#[test]
fn bucket_index_uses_upper_inclusive_bounds() {
    let bounds = [1.0, 2.0];
    let indexes: Vec<usize> = [0.0, 1.0, 1.5, 2.0, 2.0001, 99.0]
        .iter()
        .map(|x| bucket_index(&bounds, *x))
        .collect();
    assert_eq!(indexes, vec![0, 0, 1, 1, 2, 2]);
    assert_eq!(
        bucket_index(HISTOGRAM_BOUNDS, f64::MAX),
        HISTOGRAM_BOUNDS.len()
    );
}

/// Scenario: Metrics are generated from every metric group in the vendored
/// registry, annotated or not.
/// Guarantees: The batch holds exactly the requested number of metrics,
/// counters are non-negative, sums and histograms carry the template's
/// temporality (`Unspecified` unless annotated, with a start time only for
/// delta), every number point is `as_double`, and histograms have consistent
/// buckets, count, sum, min, and max.
#[test]
fn generated_metrics_are_well_formed() {
    let groups = groups_of(GroupType::Metric);
    let requested = groups.len() * 3;
    let mut r = rng();
    let data = semconv_otlp_metrics_with_rng(requested, model(), &mut r);
    let by_name: HashMap<&str, _> = groups
        .iter()
        .map(|g| (g.metric_name.as_deref().unwrap_or_default(), *g))
        .collect();
    let metrics = &data.resource_metrics[0].scope_metrics[0].metrics;
    assert_eq!(metrics.len(), requested);
    let templates: HashMap<&str, _> = model()
        .metrics
        .iter()
        .map(|t| (t.name.as_str(), t))
        .collect();
    for metric in metrics {
        let group = by_name[metric.name.as_str()];
        let temporality = templates[metric.name.as_str()].temporality;
        let delta = temporality == AggregationTemporality::Delta;
        match metric.data.as_ref().expect("data") {
            Data::Sum(sum) => {
                assert_eq!(sum.aggregation_temporality, temporality as i32);
                for p in &sum.data_points {
                    assert_eq!(p.start_time_unix_nano > 0, delta, "{}", metric.name);
                }
                assert_eq!(
                    sum.is_monotonic,
                    group.instrument == Some(InstrumentSpec::Counter)
                );
                for p in &sum.data_points {
                    let v = number_value(p.value.as_ref(), &metric.name);
                    if sum.is_monotonic {
                        assert!(v >= 0.0, "{}: negative counter increment {v}", metric.name);
                    }
                }
            }
            Data::Gauge(g) => {
                assert!(!g.data_points.is_empty());
                for p in &g.data_points {
                    let _ = number_value(p.value.as_ref(), &metric.name);
                }
            }
            Data::Histogram(h) => {
                assert_eq!(h.aggregation_temporality, temporality as i32);
                for p in &h.data_points {
                    assert_histogram_point_consistent(p, &metric.name);
                    assert_eq!(p.start_time_unix_nano > 0, delta, "{}", metric.name);
                }
            }
            other => panic!("unexpected data {other:?}"),
        }
    }
}

/// The value of a number point, which the generator always emits as a double.
fn number_value(v: Option<&number_data_point::Value>, name: &str) -> f64 {
    match v.expect("value set") {
        number_data_point::Value::AsDouble(d) => *d,
        number_data_point::Value::AsInt(i) => panic!("{name}: number point emitted as int {i}"),
    }
}

/// Number of data points in a generated metric.
fn point_count(metric: &Metric) -> usize {
    match metric.data.as_ref().expect("data") {
        Data::Sum(s) => s.data_points.len(),
        Data::Gauge(g) => g.data_points.len(),
        Data::Histogram(h) => h.data_points.len(),
        other => panic!("unexpected data {other:?}"),
    }
}

fn assert_histogram_point_consistent(p: &HistogramDataPoint, name: &str) {
    assert_eq!(p.bucket_counts.len(), p.explicit_bounds.len() + 1, "{name}");
    assert_eq!(p.bucket_counts.iter().sum::<u64>(), p.count, "{name}");
    assert!(p.count >= 1, "{name}");
    let (min, max, sum) = (
        p.min.expect("min"),
        p.max.expect("max"),
        p.sum.expect("sum"),
    );
    assert!(min <= max, "{name}");
    assert!(
        sum >= min * p.count as f64 - 1e-6 && sum <= max * p.count as f64 + 1e-6,
        "{name}"
    );
}

// ===========================================================================
// Vendored registry: representative annotations
// ===========================================================================
//
// The vendored registry carries one representative `traffic-generator`
// annotation per supported annotation kind (see the receiver README). The
// tests below generate signals from those groups only and check that each
// annotation drives the generated values.

/// Span group annotated with `duration` and `status`; its attributes
/// carry the int, double, enum, string, array, and template annotations.
const REPRESENTATIVE_SPAN: &str = "span.azure.cosmosdb.client";
/// Histogram metric annotated with `value`, `bounds`, and `observations`;
/// carries `jvm.gc.action`, whose annotation sets only `weights` over the
/// attribute's `examples`.
const REPRESENTATIVE_HISTOGRAM: &str = "jvm.gc.duration";
/// Counter metric annotated with an exponential `value`.
const REPRESENTATIVE_COUNTER: &str = "jvm.class.loaded";
/// UpDownCounter metric annotated with a uniform `value`; carries the
/// annotated boolean attribute `jvm.thread.daemon`.
const REPRESENTATIVE_UPDOWNCOUNTER: &str = "jvm.thread.count";
/// Gauge metric annotated with a discrete double `value`.
const REPRESENTATIVE_GAUGE: &str = "jvm.cpu.recent_utilization";
/// Event group annotated with `severity`; carries the annotated `any`
/// attribute `feature_flag.result.value`.
const REPRESENTATIVE_EVENT: &str = "feature_flag.evaluation";

/// Group-level and attribute-level representative annotations, as
/// `(group id, attribute name)`; `None` marks a group-level annotation.
const REPRESENTATIVE_ANNOTATIONS: &[(&str, Option<&str>)] = &[
    ("span.azure.cosmosdb.client", None),
    (
        "span.azure.cosmosdb.client",
        Some("azure.cosmosdb.request.body.size"),
    ),
    (
        "span.azure.cosmosdb.client",
        Some("azure.cosmosdb.operation.request_charge"),
    ),
    (
        "span.azure.cosmosdb.client",
        Some("azure.cosmosdb.connection.mode"),
    ),
    (
        "span.azure.cosmosdb.client",
        Some("azure.cosmosdb.operation.contacted_regions"),
    ),
    ("span.azure.cosmosdb.client", Some("db.operation.name")),
    (
        "span.azure.cosmosdb.client",
        Some("db.response.returned_rows"),
    ),
    ("span.azure.cosmosdb.client", Some("db.query.parameter")),
    ("metric.jvm.gc.duration", None),
    ("metric.jvm.gc.duration", Some("jvm.gc.action")),
    ("metric.jvm.class.loaded", None),
    ("metric.jvm.thread.count", None),
    ("metric.jvm.thread.count", Some("jvm.thread.daemon")),
    ("metric.jvm.cpu.recent_utilization", None),
    ("event.feature_flag.evaluation", None),
    (
        "event.feature_flag.evaluation",
        Some("feature_flag.result.value"),
    ),
];

/// Value of attribute `key`, panicking when it is missing.
fn attr_value<'a>(attrs: &'a [KeyValue], key: &str) -> &'a AnyValue {
    attrs
        .iter()
        .find(|kv| kv.key == key)
        .and_then(|kv| kv.value.as_ref())
        .unwrap_or_else(|| panic!("attribute {key} missing"))
}

fn attr_scalar(attrs: &[KeyValue], key: &str) -> Scalar {
    any_scalar(attr_value(attrs, key))
}

fn attr_int(attrs: &[KeyValue], key: &str) -> i64 {
    match attr_scalar(attrs, key) {
        Scalar::Int(i) => i,
        other => panic!("{key}: expected int, got {other:?}"),
    }
}

fn attr_string(attrs: &[KeyValue], key: &str) -> String {
    match attr_scalar(attrs, key) {
        Scalar::String(s) => s,
        other => panic!("{key}: expected string, got {other:?}"),
    }
}

/// A model restricted to the named span, metric, and event templates.
fn representative_model(spans: &[&str], metrics: &[&str], events: &[&str]) -> SemconvModel {
    let mut model = model().clone();
    model.spans.retain(|t| spans.contains(&t.name.as_str()));
    model.metrics.retain(|t| metrics.contains(&t.name.as_str()));
    model.events.retain(|t| events.contains(&t.name.as_str()));
    assert_eq!(model.spans.len(), spans.len(), "missing span templates");
    assert_eq!(
        model.metrics.len(),
        metrics.len(),
        "missing metric templates"
    );
    assert_eq!(model.events.len(), events.len(), "missing event templates");
    model
}

/// Scenario: The vendored registry is resolved and every representative
/// group and attribute is looked up.
/// Guarantees: Each representative annotation (one per supported annotation
/// kind) is still present, so a registry refresh that drops them fails here
/// with the missing group or attribute instead of silently weakening the
/// generation tests below.
#[test]
fn vendored_registry_keeps_representative_annotations() {
    let has = |a: Option<&BTreeMap<String, YamlValue>>| {
        a.is_some_and(|a| a.contains_key(annotation::NAMESPACE))
    };
    let mut missing = Vec::new();
    for (group_id, attribute) in REPRESENTATIVE_ANNOTATIONS {
        let Some(group) = registry().groups.iter().find(|g| g.id == *group_id) else {
            missing.push(format!("group {group_id} not found"));
            continue;
        };
        let annotated = match attribute {
            None => has(group.annotations.as_ref()),
            Some(name) => group
                .attributes
                .iter()
                .find(|a| a.name == *name)
                .is_some_and(|a| has(a.annotations.as_ref())),
        };
        if !annotated {
            missing.push(format!("{group_id} / {}", attribute.unwrap_or("(group)")));
        }
    }
    assert!(
        missing.is_empty(),
        "missing representative annotations:\n{}",
        missing.join("\n")
    );
}

/// Scenario: Every annotation in the vendored registry is compiled.
/// Guarantees: Every `traffic-generator` annotation parses and validates, so
/// the receiver starts without configuration errors when `registry_path`
/// points at the vendored registry.
#[test]
fn vendored_registry_annotations_all_validate() {
    if let Err(errors) = SemconvModel::compile(registry()) {
        panic!("{}", annotation::format_errors(&errors));
    }
    assert!(!groups_of(GroupType::Metric).is_empty());
    assert!(!groups_of(GroupType::Span).is_empty());
    assert!(!groups_of(GroupType::Event).is_empty());
}

/// Scenario: Spans are generated from the representative span group, whose
/// group annotation sets `duration` and `status` and whose attribute
/// definitions are annotated with an int lognormal range, a double normal
/// range, a weighted enum, an unweighted string list, an int zipf range, a
/// string array, and a string template.
/// Guarantees: Durations stay within the annotated range and vary, every
/// annotated status appears, and each attribute value has the attribute's
/// type and stays within its annotated range or value set (zipf favors the
/// range minimum, the enum favors its heavier member, template names expand
/// to `<prefix>.<key>`).
#[test]
fn representative_span_respects_annotations() {
    const N: usize = 2_000;
    let model = representative_model(&[REPRESENTATIVE_SPAN], &[], &[]);
    let traces = semconv_otlp_traces_with_rng(N, &model, &mut rng());
    let spans = &traces.resource_spans[0].scope_spans[0].spans;
    assert_eq!(spans.len(), N);

    let mut durations = HashSet::new();
    let mut codes = HashSet::new();
    let mut modes: HashMap<String, usize> = HashMap::new();
    let mut operations = HashSet::new();
    let mut rows: HashMap<i64, usize> = HashMap::new();
    let mut region_lengths = HashSet::new();
    let mut parameter_keys = HashSet::new();
    for span in spans {
        let d = span.end_time_unix_nano - span.start_time_unix_nano;
        assert!(
            (100_000..=30_000_000_000).contains(&d),
            "duration {d}ns outside [0.1ms, 30000ms]"
        );
        let _ = durations.insert(d);
        let _ = codes.insert(span.status.as_ref().expect("annotated status").code);

        let a = &span.attributes;
        let size = attr_int(a, "azure.cosmosdb.request.body.size");
        assert!((0..=1_048_576).contains(&size), "body size {size}");
        match attr_scalar(a, "azure.cosmosdb.operation.request_charge") {
            Scalar::Double(c) => assert!((1.0..=100.0).contains(&c), "charge {c}"),
            other => panic!("request_charge: expected double, got {other:?}"),
        }
        *modes
            .entry(attr_string(a, "azure.cosmosdb.connection.mode"))
            .or_default() += 1;
        let _ = operations.insert(attr_string(a, "db.operation.name"));
        let n = attr_int(a, "db.response.returned_rows");
        assert!((1..=1000).contains(&n), "returned_rows {n}");
        *rows.entry(n).or_default() += 1;

        let Some(AnyValueKind::ArrayValue(regions)) =
            attr_value(a, "azure.cosmosdb.operation.contacted_regions")
                .value
                .as_ref()
        else {
            panic!("contacted_regions: expected array")
        };
        let _ = region_lengths.insert(regions.values.len());
        for item in &regions.values {
            assert!(
                matches!(any_scalar(item), Scalar::String(s)
                    if ["East US", "West Europe", "Southeast Asia"].contains(&s.as_str())),
                "unexpected region {item:?}"
            );
        }

        let params: Vec<&KeyValue> = a
            .iter()
            .filter(|kv| kv.key.starts_with("db.query.parameter."))
            .collect();
        assert_eq!(params.len(), 1, "one template attribute per span");
        let _ = parameter_keys.insert(params[0].key.clone());
        assert!(
            matches!(kv_scalar(params[0]), Scalar::String(s)
                if ["42", "alice", "100"].contains(&s.as_str())),
            "unexpected parameter value {:?}",
            params[0].value
        );
    }

    assert!(durations.len() > N / 2, "durations should vary");
    assert_eq!(codes, HashSet::from([0, 1, 2]));
    assert_eq!(
        modes.keys().cloned().collect::<HashSet<_>>(),
        HashSet::from(["direct".to_owned(), "gateway".to_owned()])
    );
    assert!(modes["direct"] > modes["gateway"], "weights 9:1 {modes:?}");
    assert_eq!(
        operations,
        HashSet::from([
            "ReadItem".to_owned(),
            "CreateItem".to_owned(),
            "QueryItems".to_owned()
        ])
    );
    let most_frequent_rows = rows.iter().max_by_key(|(_, c)| **c).map(|(v, _)| *v);
    assert_eq!(most_frequent_rows, Some(1), "zipf favors range.min");
    assert_eq!(region_lengths, HashSet::from([1, 2, 3]));
    assert_eq!(
        parameter_keys,
        HashSet::from([
            "db.query.parameter.id".to_owned(),
            "db.query.parameter.name".to_owned(),
            "db.query.parameter.limit".to_owned()
        ])
    );
}

/// Scenario: Metrics are generated from the representative histogram,
/// counter, updowncounter, and gauge groups, and the updowncounter carries
/// an annotated boolean attribute.
/// Guarantees: The histogram uses the annotated `bounds`, per-point
/// observation counts, value range, and delta `temporality` (with a start
/// time no later than the point time), with one data point per metric, and
/// its `jvm.gc.action` attribute (weights only, no `values`) emits only the
/// registry examples, favoring the first by its 9:1 weight; the
/// counter and updowncounter, which have no `temporality` annotation, keep
/// `Unspecified` temporality; the
/// counter emits monotonic increments within its exponential range; the
/// updowncounter emits non-monotonic values within its uniform range with a
/// weighted boolean attribute; the gauge emits only the annotated doubles;
/// the batch holds exactly the requested number of metrics.
#[test]
fn representative_metrics_respect_annotations() {
    const ROUNDS: usize = 200;
    let names = [
        REPRESENTATIVE_HISTOGRAM,
        REPRESENTATIVE_COUNTER,
        REPRESENTATIVE_UPDOWNCOUNTER,
        REPRESENTATIVE_GAUGE,
    ];
    let model = representative_model(&[], &names, &[]);
    let data = semconv_otlp_metrics_with_rng(names.len() * ROUNDS, &model, &mut rng());
    let metrics = &data.resource_metrics[0].scope_metrics[0].metrics;
    assert_eq!(metrics.len(), names.len() * ROUNDS);

    let mut histogram_points = HashSet::new();
    let mut gc_actions: HashMap<String, usize> = HashMap::new();
    let mut daemon: HashMap<bool, usize> = HashMap::new();
    let mut gauge_values = HashSet::new();
    let mut counter_values = HashSet::new();
    for metric in metrics {
        let name = metric.name.as_str();
        match (name, metric.data.as_ref().expect("data")) {
            (REPRESENTATIVE_HISTOGRAM, Data::Histogram(h)) => {
                assert_eq!(
                    h.aggregation_temporality,
                    AggregationTemporality::Delta as i32
                );
                let _ = histogram_points.insert(h.data_points.len());
                for p in &h.data_points {
                    assert!(
                        p.start_time_unix_nano > 0 && p.start_time_unix_nano <= p.time_unix_nano,
                        "delta start time {} vs time {}",
                        p.start_time_unix_nano,
                        p.time_unix_nano
                    );
                    assert_histogram_point_consistent(p, name);
                    assert_eq!(p.explicit_bounds, vec![0.01, 0.1, 1.0, 10.0]);
                    assert!((10..=50).contains(&p.count), "observations {}", p.count);
                    assert!(p.min.expect("min") >= 0.0 && p.max.expect("max") <= 10.0);
                    *gc_actions
                        .entry(attr_string(&p.attributes, "jvm.gc.action"))
                        .or_default() += 1;
                }
            }
            (REPRESENTATIVE_COUNTER, Data::Sum(s)) => {
                assert!(s.is_monotonic);
                assert_eq!(
                    s.aggregation_temporality,
                    AggregationTemporality::Unspecified as i32
                );
                for p in &s.data_points {
                    let v = number_value(p.value.as_ref(), name);
                    assert!((0.0..=100.0).contains(&v), "{name} value {v}");
                    let _ = counter_values.insert(v as i64);
                }
            }
            (REPRESENTATIVE_UPDOWNCOUNTER, Data::Sum(s)) => {
                assert!(!s.is_monotonic);
                assert_eq!(
                    s.aggregation_temporality,
                    AggregationTemporality::Unspecified as i32
                );
                for p in &s.data_points {
                    let v = number_value(p.value.as_ref(), name);
                    assert!((10.0..=200.0).contains(&v), "{name} value {v}");
                    match attr_scalar(&p.attributes, "jvm.thread.daemon") {
                        Scalar::Bool(b) => *daemon.entry(b).or_default() += 1,
                        other => panic!("jvm.thread.daemon: expected bool, got {other:?}"),
                    }
                }
            }
            (REPRESENTATIVE_GAUGE, Data::Gauge(g)) => {
                for p in &g.data_points {
                    let v = number_value(p.value.as_ref(), name);
                    let _ = gauge_values.insert(v.to_bits());
                }
            }
            (_, other) => panic!("{name}: unexpected data {other:?}"),
        }
    }

    assert_eq!(histogram_points, HashSet::from([1]));
    assert_eq!(
        gc_actions.keys().cloned().collect::<HashSet<_>>(),
        HashSet::from(["end of minor GC".to_owned(), "end of major GC".to_owned()]),
        "jvm.gc.action draws only its examples"
    );
    assert!(
        gc_actions["end of minor GC"] > gc_actions["end of major GC"],
        "weights 9:1 favor the first example {gc_actions:?}"
    );
    assert!(counter_values.len() > 5, "counter values should vary");
    assert!(
        daemon[&true] > daemon[&false],
        "weights 3:1 favor true {daemon:?}"
    );
    assert_eq!(
        gauge_values,
        [0.25_f64, 0.5, 0.75].iter().map(|v| v.to_bits()).collect()
    );
}

/// Scenario: Logs are generated from the representative event group, whose
/// group annotation sets `severity` and whose `any` attribute definition is
/// annotated with mixed string, boolean, int, and double values.
/// Guarantees: Severity number and text agree and cover exactly the annotated
/// names, and the `any` attribute emits every annotated value with its own
/// scalar type.
#[test]
fn representative_event_respects_annotations() {
    const N: usize = 1_000;
    let model = representative_model(&[], &[], &[REPRESENTATIVE_EVENT]);
    let logs = semconv_otlp_logs_with_rng(N, &model, &mut rng());
    let records = &logs.resource_logs[0].scope_logs[0].log_records;
    assert_eq!(records.len(), N);

    let mut severities = HashSet::new();
    let mut values = HashSet::new();
    for rec in records {
        assert_eq!(
            annotation::severity(&rec.severity_text).map(|(n, _)| n),
            Some(rec.severity_number)
        );
        let _ = severities.insert(rec.severity_text.clone());
        let _ = values.insert(format!(
            "{:?}",
            attr_scalar(&rec.attributes, "feature_flag.result.value")
        ));
    }
    assert_eq!(
        severities,
        HashSet::from(["INFO".to_owned(), "DEBUG".to_owned(), "WARN".to_owned()])
    );
    assert_eq!(
        values,
        [
            Scalar::String("#ff0000".into()),
            Scalar::Bool(true),
            Scalar::Int(3),
            Scalar::Double(0.5),
        ]
        .iter()
        .map(|s| format!("{s:?}"))
        .collect()
    );
}

/// Scenario: A semantic-conventions config is compiled through `semconv_model`.
/// Guarantees: It returns the validated model compiled from the vendored
/// registry (same group counts as the shared test model).
#[test]
fn config_registry_paths() {
    let config = Config::new(
        TrafficConfig::new(None, None, 10, 1, 1, 1),
        vendored_registry_path(),
    )
    .with_annotations(true);
    let compiled = config.semconv_model().expect("valid");
    assert_eq!(compiled.spans.len(), model().spans.len());
}

// ===========================================================================
// Annotations disabled
// ===========================================================================

/// Compile the vendored registry with annotations disabled.
fn unannotated_model() -> SemconvModel {
    SemconvModel::compile_without_annotations(registry())
}

/// Scenario: The vendored registry is compiled with annotations enabled and,
/// separately, with annotations disabled.
/// Guarantees: `annotation_count` is non-zero when annotations are read (so the
/// receiver can tell whether `annotations: true` found anything) and zero when
/// they are ignored, which is the signal the startup "none found" warning uses.
#[test]
fn annotation_count_reflects_registry() {
    assert!(
        model().annotation_count() > 0,
        "vendored registry should carry representative annotations"
    );
    assert_eq!(
        unannotated_model().annotation_count(),
        0,
        "disabled compilation must not count annotations"
    );
}

/// Scenario: `semconv_model` is called on the vendored, annotated
/// registry with `annotations` disabled (the default) and enabled.
/// Guarantees: Both produce the same groups, but only the enabled model
/// carries annotation-driven samplers, so the flag alone selects whether
/// annotations drive generation.
#[test]
fn config_annotations_flag_selects_compilation() {
    let config = Config::new(
        TrafficConfig::new(None, None, 10, 1, 1, 1),
        vendored_registry_path(),
    );
    assert!(!config.annotations());
    let disabled = config.clone().semconv_model().expect("valid");
    let enabled = config
        .with_annotations(true)
        .semconv_model()
        .expect("valid");

    assert_eq!(disabled.spans.len(), enabled.spans.len());
    assert_eq!(disabled.metrics.len(), enabled.metrics.len());
    assert_eq!(disabled.events.len(), enabled.events.len());
    assert!(enabled.spans.iter().any(|s| s.status.is_some()));
    assert!(enabled.metrics.iter().any(|m| m.value.is_some()));
    assert!(enabled.events.iter().any(|e| e.severity.is_some()));
    assert!(disabled.spans.iter().all(|s| s.status.is_none()));
    assert!(disabled.spans.iter().all(|s| s.duration_ms.is_none()));
    assert!(disabled.metrics.iter().all(|m| m.value.is_none()));
    assert!(disabled.metrics.iter().all(|m| m.observations.is_none()));
    assert!(disabled.events.iter().all(|e| e.severity.is_none()));
}

/// Scenario: A registry whose span group and attribute carry invalid
/// `traffic-generator` annotations is compiled with and without annotations.
/// Guarantees: With annotations disabled, compilation never reads the
/// annotations and succeeds with the invalid attribute kept as a fallback
/// generator; with annotations enabled, the same registry still fails.
#[test]
fn compile_without_annotations_ignores_invalid_annotations() {
    let mut bad = registry().clone();
    let span = bad
        .groups
        .iter_mut()
        .find(|g| g.r#type == GroupType::Span)
        .expect("a span group");
    let mut ann = span.annotations.clone().unwrap_or_default();
    let _ = ann.insert(
        annotation::NAMESPACE.to_owned(),
        YamlValue(yaml_from_str("{ severity: { values: [INFO] } }")),
    );
    span.annotations = Some(ann);
    let span_id = span.id.clone();
    let attr_count = span.attributes.len() + 1;
    span.attributes.push(attribute(
        "bad.attr",
        primitive(PrimitiveOrArrayTypeSpec::String),
        Some("{ range: { min: 0, max: 1 } }"),
    ));
    // A non-signal group with an annotation is rejected only when enabled.
    let attr_group = bad
        .groups
        .iter_mut()
        .find(|g| g.r#type == GroupType::AttributeGroup)
        .expect("an attribute group");
    let mut ann = attr_group.annotations.clone().unwrap_or_default();
    let _ = ann.insert(
        annotation::NAMESPACE.to_owned(),
        YamlValue(yaml_from_str("{ value: { values: [1] } }")),
    );
    attr_group.annotations = Some(ann);

    assert!(SemconvModel::compile(&bad).is_err());
    let model = SemconvModel::compile_without_annotations(&bad);
    let span = model
        .spans
        .iter()
        .find(|s| s.name == span_id)
        .expect("span compiled");
    assert_eq!(span.attributes.len(), attr_count);
    let kv = span.attributes.last().expect("bad.attr").sample(&mut rng());
    assert_eq!(kv.key, "bad.attr");
    assert_eq!(kv_scalar(&kv), Scalar::String("value".into()));
}

/// Scenario: Spans, metrics, and logs are generated from the vendored,
/// annotated registry with annotations disabled.
/// Guarantees: Output matches the pre-annotation generator: spans last the
/// fixed default delay with no status, logs are INFO with no severity text,
/// number points are 1.0 with `Unspecified` temporality and no start time,
/// histogram points carry no buckets, bounds, count, sum, min, or max, and
/// annotated attributes use their non-annotated fallback values.
#[test]
fn generation_without_annotations_matches_unannotated_output() {
    use crate::receivers::traffic_generator::fake_data::delay;
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::SeverityNumber;

    let model = unannotated_model();
    let mut r = rng();

    let n_spans = model.spans.len() * 2;
    let traces = semconv_otlp_traces_with_rng(n_spans, &model, &mut r);
    let spans = &traces.resource_spans[0].scope_spans[0].spans;
    assert_eq!(spans.len(), n_spans);
    for span in spans {
        assert_eq!(
            span.end_time_unix_nano - span.start_time_unix_nano,
            delay(),
            "{}",
            span.name
        );
        assert!(span.status.is_none(), "{}", span.name);
    }

    let n_logs = model.events.len() * 2;
    let logs = semconv_otlp_logs_with_rng(n_logs, &model, &mut r);
    let records = &logs.resource_logs[0].scope_logs[0].log_records;
    assert_eq!(records.len(), n_logs);
    for rec in records {
        assert_eq!(rec.severity_number, SeverityNumber::Info as i32);
        assert!(rec.severity_text.is_empty());
    }

    let n_metrics = model.metrics.len() * 2;
    let data = semconv_otlp_metrics_with_rng(n_metrics, &model, &mut r);
    let metrics = &data.resource_metrics[0].scope_metrics[0].metrics;
    assert_eq!(metrics.len(), n_metrics);
    let mut saw_histogram = false;
    let mut daemon_values = HashSet::new();
    for metric in metrics {
        let number_points = match metric.data.as_ref().expect("data") {
            Data::Sum(s) => {
                assert_eq!(
                    s.aggregation_temporality,
                    AggregationTemporality::Unspecified as i32
                );
                &s.data_points
            }
            Data::Gauge(g) => &g.data_points,
            Data::Histogram(h) => {
                saw_histogram = true;
                assert_eq!(
                    h.aggregation_temporality,
                    AggregationTemporality::Unspecified as i32
                );
                for p in &h.data_points {
                    assert!(p.bucket_counts.is_empty(), "{}", metric.name);
                    assert!(p.explicit_bounds.is_empty(), "{}", metric.name);
                    assert_eq!(p.count, 0, "{}", metric.name);
                    assert!(p.sum.is_none() && p.min.is_none() && p.max.is_none());
                    assert_eq!(p.start_time_unix_nano, 0);
                }
                continue;
            }
            other => panic!("{}: unexpected data {other:?}", metric.name),
        };
        for p in number_points {
            assert_eq!(number_value(p.value.as_ref(), &metric.name), 1.0);
            assert_eq!(p.start_time_unix_nano, 0);
        }
        if metric.name == REPRESENTATIVE_UPDOWNCOUNTER {
            for p in number_points {
                let _ = daemon_values.insert(format!(
                    "{:?}",
                    attr_scalar(&p.attributes, "jvm.thread.daemon")
                ));
            }
        }
    }
    assert!(saw_histogram, "vendored registry has a histogram");
    // `jvm.thread.daemon` is annotated in the registry; disabled, it uses the
    // boolean type default only.
    assert_eq!(
        daemon_values,
        HashSet::from([format!("{:?}", Scalar::Bool(true))])
    );
}

// ===========================================================================
// Compiled model and samplers
// ===========================================================================

/// Scenario: A precompiled `ValueSampler` and a `ValueSampler` rebuilt for
/// every draw are driven by identically seeded RNGs for discrete, weighted,
/// and every continuous distribution (int and double output).
/// Guarantees: Compiling a sampler once and reusing it draws exactly the same
/// value sequence as rebuilding it for every draw, so the optimization does
/// not change generated values.
#[test]
fn compiled_value_sampler_matches_per_call_sampling() {
    let mut specs = vec![
        (
            ValueSpec::Discrete {
                values: vec![Scalar::Int(1), Scalar::Int(2), Scalar::Int(3)],
                weights: None,
            },
            true,
        ),
        (
            ValueSpec::Discrete {
                values: vec![Scalar::String("a".into()), Scalar::String("b".into())],
                weights: Some(vec![9.0, 1.0]),
            },
            false,
        ),
    ];
    for (_, distribution, (min, max)) in non_uniform_cases() {
        for integer in [false, true] {
            if matches!(distribution, Distribution::Zipf { .. }) && !integer {
                continue;
            }
            specs.push((
                ValueSpec::Continuous {
                    range: Range { min, max },
                    distribution,
                },
                integer,
            ));
        }
    }
    specs.push((
        ValueSpec::Continuous {
            range: Range {
                min: -5.0,
                max: 5.0,
            },
            distribution: Distribution::Uniform,
        },
        false,
    ));
    for (spec, integer) in specs {
        let compiled = ValueSampler::new(&spec, integer);
        let (mut a, mut b) = (rng(), rng());
        for _ in 0..500 {
            assert_eq!(
                compiled.sample_value(&mut a),
                ValueSampler::new(&spec, integer).sample_value(&mut b),
                "{spec:?} integer={integer}"
            );
        }
    }
}

/// Scenario: The vendored registry is compiled into a `SemconvModel`.
/// Guarantees: The model has one template per span group, per metric group
/// with an instrument, and per event group (in registry order), and one
/// attribute generator per group attribute, so generation cycles over exactly
/// the groups that can produce a signal.
#[test]
fn compiled_model_mirrors_registry_groups() {
    let m = model();
    let spans = groups_of(GroupType::Span);
    let metrics: Vec<_> = groups_of(GroupType::Metric)
        .into_iter()
        .filter(|g| g.instrument.is_some())
        .collect();
    let events = groups_of(GroupType::Event);
    assert_eq!(m.spans.len(), spans.len());
    assert_eq!(m.metrics.len(), metrics.len());
    assert_eq!(m.events.len(), events.len());
    for (t, g) in m.spans.iter().zip(&spans) {
        assert_eq!(t.name, g.id);
        assert_eq!(t.attributes.len(), g.attributes.len());
    }
    for (t, g) in m.metrics.iter().zip(&metrics) {
        assert_eq!(Some(&t.name), g.metric_name.as_ref());
        assert_eq!(Some(&t.instrument), g.instrument.as_ref());
        assert_eq!(t.attributes.len(), g.attributes.len());
    }
    for (t, g) in m.events.iter().zip(&events) {
        assert_eq!(Some(&t.name), g.name.as_ref());
        assert_eq!(t.attributes.len(), g.attributes.len());
    }
}

/// Scenario: A registry with an invalid event-group annotation and an invalid
/// attribute annotation on the same group is compiled.
/// Guarantees: `SemconvModel::compile` reports both errors, the group error
/// first (without an attribute) and then the attribute error, so the startup
/// error report is complete and stably ordered.
#[test]
fn compile_reports_same_errors_as_validation() {
    let mut bad = registry().clone();
    let event = bad
        .groups
        .iter_mut()
        .find(|g| g.r#type == GroupType::Event)
        .expect("an event group");
    let mut ann = event.annotations.clone().unwrap_or_default();
    let _ = ann.insert(
        annotation::NAMESPACE.to_owned(),
        YamlValue(yaml_from_str("{ severity: { values: [LOUD] } }")),
    );
    event.annotations = Some(ann);
    event.attributes.push(attribute(
        "bad.attr",
        primitive(PrimitiveOrArrayTypeSpec::Int),
        Some("{ weights: [1] }"),
    ));
    let event_id = event.id.clone();

    let compiled = SemconvModel::compile(&bad).expect_err("invalid annotations");
    assert_eq!(compiled.len(), 2);
    assert_eq!(compiled[0].group, event_id);
    assert!(compiled[0].attribute.is_none());
    assert_eq!(compiled[1].attribute.as_deref(), Some("bad.attr"));
}

/// Scenario: Non-annotated attributes are compiled into fallback generators
/// for enums, example lists, single examples, and type defaults.
/// Guarantees: Compiled fallbacks produce exactly the values the
/// non-annotated rules allow: the first enum member (as on main), every
/// example, or the type default.
#[test]
fn compiled_fallbacks_cover_expected_values() {
    let mut r = rng();
    let collect = |attr: &Attribute, r: &mut StdRng| -> HashSet<String> {
        let g = AttributeGen::try_new(attr).expect("unannotated attribute compiles");
        (0..500)
            .map(|_| format!("{:?}", kv_scalar(&g.sample(r))))
            .collect()
    };
    let e = attribute("e", enum_type(&["a", "b", "c"]), None);
    assert_eq!(
        collect(&e, &mut r),
        HashSet::from([format!("{:?}", Scalar::String("a".into()))])
    );

    let mut ex = attribute("x", primitive(PrimitiveOrArrayTypeSpec::String), None);
    ex.examples = Some(Examples::Strings(vec!["p".into(), "q".into()]));
    assert_eq!(
        collect(&ex, &mut r),
        HashSet::from([
            format!("{:?}", Scalar::String("p".into())),
            format!("{:?}", Scalar::String("q".into()))
        ])
    );

    let d = attribute("d", primitive(PrimitiveOrArrayTypeSpec::Double), None);
    assert_eq!(
        collect(&d, &mut r),
        HashSet::from([format!("{:?}", Scalar::Double(3.13))])
    );
    let b = attribute("b", primitive(PrimitiveOrArrayTypeSpec::Boolean), None);
    assert_eq!(
        collect(&b, &mut r),
        HashSet::from([format!("{:?}", Scalar::Bool(true))])
    );
}

/// Scenario: Signals are generated from the same compiled model twice with
/// identically seeded RNGs.
/// Guarantees: The compiled model is reused without mutation, so generation
/// is reproducible for a seed and repeated batches do not depend on prior
/// generation.
#[test]
fn compiled_model_generation_is_reproducible() {
    let (mut a, mut b) = (rng(), rng());
    let strip = |mut d: otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogsData| {
        for rl in &mut d.resource_logs {
            for sl in &mut rl.scope_logs {
                for rec in &mut sl.log_records {
                    rec.time_unix_nano = 0;
                    rec.observed_time_unix_nano = 0;
                }
            }
        }
        d
    };
    let n = groups_of(GroupType::Event).len() * 2;
    assert_eq!(
        strip(semconv_otlp_logs_with_rng(n, model(), &mut a)),
        strip(semconv_otlp_logs_with_rng(n, model(), &mut b))
    );
}

// ===========================================================================
// Limits: one data point per metric, count caps, numeric safety
// ===========================================================================

/// Scenario: Metrics are generated from every metric group in the vendored
/// registry (gauges, sums, and histograms), cycling three times.
/// Guarantees: The batch holds exactly the requested number of metrics and
/// every metric carries exactly one data point, so payload item counts
/// (data points) equal the metric count the producer requested.
#[test]
fn every_metric_carries_one_data_point() {
    let requested = model().metrics.len() * 3;
    let data = semconv_otlp_metrics_with_rng(requested, model(), &mut rng());
    let metrics = &data.resource_metrics[0].scope_metrics[0].metrics;
    assert_eq!(metrics.len(), requested);
    for metric in metrics {
        assert_eq!(point_count(metric), 1, "{}", metric.name);
    }
}

/// Scenario: Metrics are requested from a model without metric templates.
/// Guarantees: Generation returns an empty batch instead of looping forever.
#[test]
fn metric_generation_without_metrics_is_empty() {
    let data = semconv_otlp_metrics_with_rng(10, &SemconvModel::default(), &mut rng());
    assert!(data.resource_metrics[0].scope_metrics[0].metrics.is_empty());
}

/// Scenario: Count specs (`observations`, array `length`) are annotated
/// with values above their caps, in both discrete and continuous form.
/// Guarantees: Each is rejected at parse time with the offending key path, so
/// a registry cannot request allocations or sampling work large enough to
/// panic or stall the receiver.
#[test]
fn parse_rejects_counts_above_caps() {
    let ns = annotation::NAMESPACE;
    expect_err(
        annotation::parse_metric_spec(
            &yaml_map("observations: { values: [1, 51] }"),
            &InstrumentSpec::Histogram,
        ),
        &format!("{ns}.observations.values[1]"),
        "must be in 1..=50",
    );
    expect_err(
        annotation::parse_metric_spec(
            &yaml_map("observations: { range: { min: 1, max: 1000000000000000000 } }"),
            &InstrumentSpec::Histogram,
        ),
        &format!("{ns}.observations.range.max"),
        "must be within",
    );
    expect_err(
        annotation::parse_metric_spec(
            &yaml_map("observations: { range: { min: 1, max: 51 } }"),
            &InstrumentSpec::Histogram,
        ),
        &format!("{ns}.observations.range.max"),
        "must be in 1..=50",
    );
    expect_err(
        parse_attr(
            "length: { values: [33] }\nitems: { values: [1] }",
            &primitive(PrimitiveOrArrayTypeSpec::Ints),
        ),
        &format!("{ns}.length.values[0]"),
        "must be in 0..=32",
    );
    let _ = annotation::parse_metric_spec(
        &yaml_map(&format!(
            "observations: {{ range: {{ min: 1, max: {} }} }}",
            annotation::MAX_OBSERVATIONS
        )),
        &InstrumentSpec::Histogram,
    )
    .expect("counts at the caps are valid");
    let _ = parse_attr(
        &format!(
            "length: {{ values: [0, {}] }}\nitems: {{ values: [1] }}",
            annotation::MAX_ARRAY_LENGTH
        ),
        &primitive(PrimitiveOrArrayTypeSpec::Ints),
    )
    .expect("lengths at the caps are valid");
}

/// Scenario: A discrete `values` list and a histogram `bounds` list each exceed
/// their size caps (`MAX_DISCRETE_VALUES`, `MAX_HISTOGRAM_BOUNDS`), while lists
/// exactly at the caps are also parsed.
/// Guarantees: Oversized lists are rejected at parse time with the offending
/// key path (bounding the memory a single compiled sampler or metric retains),
/// and lists at the caps are accepted.
#[test]
fn parse_rejects_lists_above_caps() {
    let ns = annotation::NAMESPACE;

    // One more discrete value than allowed.
    let too_many: Vec<String> = (0..=annotation::MAX_DISCRETE_VALUES)
        .map(|i| i.to_string())
        .collect();
    expect_err(
        parse_scalar(
            &format!("values: [{}]", too_many.join(", ")),
            ScalarKind::Int,
        ),
        &format!("{ns}.values"),
        "at most",
    );
    // Exactly at the cap parses.
    let at_cap: Vec<String> = (0..annotation::MAX_DISCRETE_VALUES)
        .map(|i| i.to_string())
        .collect();
    let _ = parse_scalar(&format!("values: [{}]", at_cap.join(", ")), ScalarKind::Int)
        .expect("values at the cap are valid");

    // One more histogram bound than allowed (strictly increasing).
    let too_many_bounds: Vec<String> = (0..=annotation::MAX_HISTOGRAM_BOUNDS)
        .map(|i| i.to_string())
        .collect();
    expect_err(
        annotation::parse_metric_spec(
            &yaml_map(&format!("bounds: [{}]", too_many_bounds.join(", "))),
            &InstrumentSpec::Histogram,
        ),
        &format!("{ns}.bounds"),
        "at most",
    );
    let at_cap_bounds: Vec<String> = (0..annotation::MAX_HISTOGRAM_BOUNDS)
        .map(|i| i.to_string())
        .collect();
    let _ = annotation::parse_metric_spec(
        &yaml_map(&format!("bounds: [{}]", at_cap_bounds.join(", "))),
        &InstrumentSpec::Histogram,
    )
    .expect("bounds at the cap are valid");
}

/// Scenario: Int ranges are annotated with bounds beyond +/-(2^52 - 1): one
/// past the limit (+/-2^52), 2^53 + 1 on each sign (which rounds to +/-2^53
/// when converted to f64), a degenerate min == max == 2^53 + 1, and a zipf
/// range spanning nearly the whole i64 domain.
/// Guarantees: They are rejected at parse time -- validating the converted
/// bound is sound because f64 rounding never moves an out-of-limit value back
/// into range -- so int sampling never loses precision, never emits a value
/// outside the configured range, and zipf never computes an overflowing rank
/// count; bounds at exactly +/-(2^52 - 1) are accepted, compile into a
/// sampler, and only ever yield values within the range.
#[test]
fn parse_rejects_int_ranges_beyond_safe_integers() {
    let ns = annotation::NAMESPACE;
    // One past the limit (2^52) on each sign.
    expect_err(
        parse_scalar("range: { min: 0, max: 4503599627370496 }", ScalarKind::Int),
        &format!("{ns}.range.max"),
        "must be within",
    );
    expect_err(
        parse_scalar("range: { min: -4503599627370496, max: 0 }", ScalarKind::Int),
        &format!("{ns}.range.min"),
        "must be within",
    );
    // 2^53 + 1 rounds to 2^53 as an f64; validation on the converted bound
    // still rejects it because 2^53 is far above the limit.
    expect_err(
        parse_scalar("range: { min: 0, max: 9007199254740993 }", ScalarKind::Int),
        &format!("{ns}.range.max"),
        "must be within",
    );
    expect_err(
        parse_scalar("range: { min: -9007199254740993, max: 0 }", ScalarKind::Int),
        &format!("{ns}.range.min"),
        "must be within",
    );
    // Degenerate range at 2^53 + 1: rejected rather than compiling and always
    // emitting 2^53, which would be below the configured min.
    expect_err(
        parse_scalar(
            "range: { min: 9007199254740993, max: 9007199254740993 }",
            ScalarKind::Int,
        ),
        &format!("{ns}.range.min"),
        "must be within",
    );
    expect_err(
        parse_scalar(
            "range: { min: -9223372036854775808, max: 9223372036854775807 }\n\
             distribution: { type: zipf, exponent: 1.1 }",
            ScalarKind::Int,
        ),
        &format!("{ns}.range.min"),
        "must be within",
    );
    // Bounds exactly at the limit compile; zipf over the widest accepted range
    // must never produce a value outside [min, max].
    let min = -4_503_599_627_370_495.0;
    let max = 4_503_599_627_370_495.0;
    let spec = parse_scalar(
        "range: { min: -4503599627370495, max: 4503599627370495 }\n\
         distribution: { type: zipf, exponent: 1.1 }",
        ScalarKind::Int,
    )
    .expect("bounds at 2^52 - 1 are valid");
    let sampler = ValueSampler::new(&spec, true);
    let mut r = rng();
    for _ in 0..100 {
        let x = sampler.sample_f64(&mut r);
        assert!((min..=max).contains(&x), "{x}");
    }
}

/// Scenario: Weights are individually finite but their sum overflows to
/// infinity.
/// Guarantees: The spec is rejected at parse time instead of producing an
/// infinite cumulative total that would select arbitrary values.
#[test]
fn parse_rejects_weights_with_overflowing_sum() {
    expect_err(
        parse_scalar(
            "values: [1, 2, 3]\nweights: [1.0e308, 1.0e308, 0]",
            ScalarKind::Int,
        ),
        &format!("{}.weights", annotation::NAMESPACE),
        "sum must be finite",
    );
}

/// An RNG whose every draw is the maximum value, which makes `random::<f64>()`
/// return the largest value below 1.0.
struct MaxRng;

impl rand::TryRng for MaxRng {
    type Error = std::convert::Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok(u32::MAX)
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        Ok(u64::MAX)
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        dst.fill(u8::MAX);
        Ok(())
    }
}

/// Scenario: A weighted sampler whose trailing items have zero weight is
/// drawn with the largest possible uniform value. With a subnormal total,
/// `u * total` rounds up to `total`, so the target lands on or past every
/// cumulative weight.
/// Guarantees: The last positive-weight index is returned; zero-weight items
/// are never selected, even when rounding pushes the target to the total.
#[test]
fn index_sampler_never_selects_trailing_zero_weights() {
    let sampler = IndexSampler::new(4, Some(&[0.1, 0.2, 0.0, 0.0]));
    assert_eq!(sampler.sample(&mut MaxRng), 1);
    let smallest_subnormal = f64::from_bits(1);
    let sampler = IndexSampler::new(3, Some(&[0.0, smallest_subnormal, 0.0]));
    assert_eq!(sampler.sample(&mut MaxRng), 1);
}

/// Scenario: A span is annotated with a duration so large that the start
/// timestamp plus the duration in nanoseconds overflows `u64`.
/// Guarantees: Generation does not panic and the end timestamp saturates at
/// `u64::MAX`, never wrapping before the start time.
#[test]
fn huge_span_duration_saturates_end_time() {
    let mut model = representative_model(&[REPRESENTATIVE_SPAN], &[], &[]);
    let spec = parse_scalar("values: [1.0e300]", ScalarKind::Double).expect("valid");
    model.spans[0].duration_ms = Some(ValueSampler::new(&spec, false));
    let traces = semconv_otlp_traces_with_rng(1, &model, &mut rng());
    let span = &traces.resource_spans[0].scope_spans[0].spans[0];
    assert_eq!(span.end_time_unix_nano, u64::MAX);
    assert!(span.end_time_unix_nano >= span.start_time_unix_nano);
}

/// Scenario: A double range spans almost the whole finite `f64` domain, so
/// its width `max - min` overflows to infinity.
/// Guarantees: Uniform samples stay finite and inside the range and are
/// spread across it, instead of every sample collapsing to `max`.
#[test]
fn uniform_sampling_handles_ranges_wider_than_f64_max() {
    let sampler = number_sampler((-1.0e308, 1.0e308), Distribution::Uniform, false);
    let xs = draw(&sampler, 1_000);
    assert!(
        xs.iter()
            .all(|x| x.is_finite() && (-1.0e308..=1.0e308).contains(x)),
        "samples must be finite and in range"
    );
    assert!(xs.iter().any(|x| *x < 0.0), "negative half is reachable");
    assert!(xs.iter().any(|x| *x > 0.0), "positive half is reachable");
    assert!(
        xs.iter().all(|x| *x < 1.0e308),
        "samples must not pin to max"
    );
}
