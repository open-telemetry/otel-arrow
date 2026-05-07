// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::receivers::host_metrics_receiver::semconv::{attr, metric};
#[cfg(feature = "dev-tools")]
use otap_df_pdata::proto::opentelemetry::common::v1::AnyValue;
use otap_df_pdata::proto::opentelemetry::common::v1::{KeyValue, any_value};
#[cfg(feature = "dev-tools")]
use otap_df_pdata::proto::opentelemetry::metrics::v1::NumberDataPoint;
use otap_df_pdata::proto::opentelemetry::metrics::v1::{
    AggregationTemporality, Metric, MetricsData, metric as otlp_metric, number_data_point,
};
use otap_df_pdata::testing::round_trip::decode_metrics;
use projection::{CounterStarts, counter_key, counter_key_joined, counter_key_matches_joined};
#[cfg(feature = "dev-tools")]
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, ErrorKind};
use std::path::PathBuf;
#[cfg(feature = "dev-tools")]
use weaver_common::{result::WResult, vdir::VirtualDirectoryPath};
#[cfg(feature = "dev-tools")]
use weaver_forge::registry::ResolvedRegistry;
#[cfg(feature = "dev-tools")]
use weaver_resolver::SchemaResolver;
#[cfg(feature = "dev-tools")]
use weaver_semconv::{
    attribute::{
        AttributeType, BasicRequirementLevelSpec, PrimitiveOrArrayTypeSpec, RequirementLevel,
        ValueSpec,
    },
    group::{GroupType, InstrumentSpec},
    registry_repo::RegistryRepo,
};

fn block_on_scrape(source: &mut ProcfsSource, due: ProcfsFamilies) -> io::Result<HostScrape> {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("runtime")
        .block_on(source.scrape_due(due))
}

#[test]
fn projection_uses_expected_metric_shapes() {
    let data = projection_fixture_request();

    let resource_metrics = data.resource_metrics.first().expect("resource metrics");
    let resource = resource_metrics.resource.as_ref().expect("resource");
    assert_has_attr(&resource.attributes, attr::OS_TYPE, "linux");
    assert_has_attr(&resource.attributes, attr::HOST_ID, "host-id");
    assert_has_attr(&resource.attributes, attr::HOST_NAME, "host-name");
    assert_has_attr(&resource.attributes, attr::HOST_ARCH, "amd64");

    let metrics = &resource_metrics.scope_metrics[0].metrics;
    assert_metric_shape(metrics, metric::CPU_TIME, "s", Some(true));
    assert_first_point_attr(metrics, metric::CPU_TIME, attr::CPU_MODE, "user");
    assert_sum_point_attr(metrics, metric::CPU_TIME, attr::CPU_MODE, "iowait");
    assert_metric_shape(metrics, metric::CPU_UTILIZATION, "1", None);
    assert_first_point_attr(metrics, metric::CPU_UTILIZATION, attr::CPU_MODE, "user");
    assert_metric_shape(metrics, metric::CPU_LOGICAL_COUNT, "{cpu}", Some(false));
    assert_metric_shape(metrics, metric::CPU_PHYSICAL_COUNT, "{cpu}", Some(false));
    assert_metric_shape(metrics, metric::CPU_FREQUENCY, "Hz", None);
    assert_first_point_int(metrics, metric::CPU_FREQUENCY, 2_400_000_000);
    assert_first_point_attr_int(metrics, metric::CPU_FREQUENCY, attr::CPU_LOGICAL_NUMBER, 0);
    assert_metric_shape(metrics, metric::MEMORY_USAGE, "By", Some(false));
    assert_first_point_attr(
        metrics,
        metric::MEMORY_USAGE,
        attr::SYSTEM_MEMORY_STATE,
        "used",
    );
    assert_metric_shape(metrics, metric::MEMORY_UTILIZATION, "1", None);
    assert_metric_shape(metrics, metric::MEMORY_LINUX_AVAILABLE, "By", Some(false));
    assert_metric_shape(metrics, metric::MEMORY_LINUX_SLAB_USAGE, "By", Some(false));
    assert_metric_shape(metrics, metric::MEMORY_LIMIT, "By", Some(false));
    assert_metric_shape(metrics, metric::MEMORY_LINUX_SHARED, "By", Some(false));
    assert_metric_shape(
        metrics,
        metric::MEMORY_LINUX_HUGEPAGES_LIMIT,
        "{page}",
        Some(false),
    );
    assert_metric_shape(
        metrics,
        metric::MEMORY_LINUX_HUGEPAGES_PAGE_SIZE,
        "By",
        Some(false),
    );
    assert_metric_shape(
        metrics,
        metric::MEMORY_LINUX_HUGEPAGES_RESERVED,
        "{page}",
        Some(false),
    );
    assert_metric_shape(
        metrics,
        metric::MEMORY_LINUX_HUGEPAGES_SURPLUS,
        "{page}",
        Some(false),
    );
    assert_metric_shape(
        metrics,
        metric::MEMORY_LINUX_HUGEPAGES_USAGE,
        "{page}",
        Some(false),
    );
    assert_first_point_attr(
        metrics,
        metric::MEMORY_LINUX_HUGEPAGES_USAGE,
        attr::SYSTEM_MEMORY_LINUX_HUGEPAGES_STATE,
        "used",
    );
    assert_metric_shape(
        metrics,
        metric::MEMORY_LINUX_HUGEPAGES_UTILIZATION,
        "1",
        None,
    );
    assert_metric_shape(metrics, metric::UPTIME, "s", None);
    assert_metric_shape(metrics, metric::PAGING_FAULTS, "{fault}", Some(true));
    assert_first_point_attr(
        metrics,
        metric::PAGING_FAULTS,
        attr::SYSTEM_PAGING_FAULT_TYPE,
        "minor",
    );
    assert_metric_shape(
        metrics,
        metric::PAGING_OPERATIONS,
        "{operation}",
        Some(true),
    );
    assert_sum_point_attr(
        metrics,
        metric::PAGING_OPERATIONS,
        attr::SYSTEM_PAGING_DIRECTION,
        "in",
    );
    assert_sum_point_attr(
        metrics,
        metric::PAGING_OPERATIONS,
        attr::SYSTEM_PAGING_FAULT_TYPE,
        "minor",
    );
    assert_metric_shape(metrics, metric::PAGING_USAGE, "By", Some(false));
    assert_first_point_attr(
        metrics,
        metric::PAGING_USAGE,
        attr::SYSTEM_DEVICE,
        "/dev/swap",
    );
    assert_metric_shape(metrics, metric::PAGING_UTILIZATION, "1", None);
    assert_metric_shape(metrics, metric::PROCESS_COUNT, "{process}", Some(false));
    assert_sum_point_attr(
        metrics,
        metric::PROCESS_COUNT,
        attr::PROCESS_STATE,
        "running",
    );
    assert_metric_shape(metrics, metric::PROCESS_CREATED, "{process}", Some(true));
    assert_metric_shape(metrics, metric::DISK_IO, "By", Some(true));
    assert_first_point_attr(metrics, metric::DISK_IO, attr::DISK_IO_DIRECTION, "read");
    assert_metric_shape(metrics, metric::DISK_OPERATIONS, "{operation}", Some(true));
    assert_metric_shape(metrics, metric::DISK_IO_TIME, "s", Some(true));
    assert_first_point_attr(metrics, metric::DISK_IO_TIME, attr::SYSTEM_DEVICE, "sda");
    assert_metric_shape(metrics, metric::DISK_OPERATION_TIME, "s", Some(true));
    assert_metric_shape(metrics, metric::DISK_MERGED, "{operation}", Some(true));
    assert_metric_shape(metrics, metric::DISK_LIMIT, "By", Some(false));
    assert_first_point_attr(metrics, metric::DISK_LIMIT, attr::SYSTEM_DEVICE, "sda");
    assert_metric_shape(metrics, metric::FILESYSTEM_USAGE, "By", Some(false));
    assert_first_point_attr(
        metrics,
        metric::FILESYSTEM_USAGE,
        attr::SYSTEM_FILESYSTEM_STATE,
        "used",
    );
    assert_metric_shape(metrics, metric::FILESYSTEM_UTILIZATION, "1", None);
    assert_metric_shape(metrics, metric::FILESYSTEM_LIMIT, "By", Some(false));
    assert_no_first_point_attr(
        metrics,
        metric::FILESYSTEM_LIMIT,
        attr::SYSTEM_FILESYSTEM_STATE,
    );
    assert_metric_shape(metrics, metric::NETWORK_IO, "By", Some(true));
    assert_first_point_attr(
        metrics,
        metric::NETWORK_IO,
        attr::NETWORK_INTERFACE_NAME,
        "eth0",
    );
    assert_metric_shape(
        metrics,
        metric::NETWORK_PACKET_COUNT,
        "{packet}",
        Some(true),
    );
    assert_first_point_attr(
        metrics,
        metric::NETWORK_PACKET_COUNT,
        attr::SYSTEM_DEVICE,
        "eth0",
    );
    assert_metric_shape(
        metrics,
        metric::NETWORK_PACKET_DROPPED,
        "{packet}",
        Some(true),
    );
    assert_first_point_attr(
        metrics,
        metric::NETWORK_PACKET_DROPPED,
        attr::NETWORK_INTERFACE_NAME,
        "eth0",
    );
    assert_metric_shape(metrics, metric::NETWORK_ERRORS, "{error}", Some(true));
}

#[cfg(feature = "dev-tools")]
#[test]
#[ignore = "dev-only semconv drift check; may access a local or remote semantic-conventions registry"]
fn emitted_phase1_metric_shapes_match_weaver_semconv() {
    let registry = load_semconv_registry();
    let semconv_shapes = semconv_system_metric_shapes(&registry);
    let emitted_shapes = emitted_phase1_metric_shapes();

    for (name, emitted) in emitted_shapes {
        let semconv = semconv_shapes
            .get(&name)
            .unwrap_or_else(|| panic!("missing semconv metric {name}"));

        assert_eq!(emitted.unit, semconv.unit, "unit mismatch for {name}");
        assert_eq!(
            emitted.monotonic, semconv.monotonic,
            "instrument/temporality mismatch for {name}"
        );
        assert_eq!(
            emitted.value_type, semconv.value_type,
            "metric value type mismatch for {name}"
        );

        for attr in &semconv.attributes {
            assert!(
                emitted.attributes.contains(attr),
                "missing semconv attribute {attr} on {name}"
            );
        }
        for attr in &emitted.attributes {
            assert!(
                semconv.all_attributes.contains(attr),
                "unexpected semconv attribute {attr} on {name}"
            );
        }
        for (attr, emitted_kind) in &emitted.attribute_types {
            let Some(semconv_kind) = semconv.attribute_types.get(attr) else {
                continue;
            };
            assert_eq!(
                emitted_kind, semconv_kind,
                "attribute value type mismatch for {attr} on {name}"
            );
        }
        for (attr, values) in &emitted.enum_values {
            let Some(allowed_values) = semconv.enum_values.get(attr) else {
                continue;
            };
            for value in values {
                if is_intentional_semconv_enum_value_gap(name.as_str(), attr.as_str(), value) {
                    continue;
                }
                assert!(
                    allowed_values.contains(value),
                    "unexpected enum value {attr}={value} on {name}"
                );
            }
        }
    }
}

#[test]
fn projection_uses_counter_start_overrides_for_reset_series() {
    let data = decode_metrics(
        HostSnapshot {
            now_unix_nano: 2_000,
            start_time_unix_nano: 1_000,
            counter_starts: CounterStarts {
                entries: vec![(counter_key(metric::PROCESS_CREATED, ""), 1_500)],
            },
            processes: Some(ProcessStats {
                created: 99,
                ..ProcessStats::default()
            }),
            ..HostSnapshot::default()
        }
        .into_otap_records()
        .expect("encode ok"),
    );

    let metrics = &data.resource_metrics[0].scope_metrics[0].metrics;
    assert_first_sum_point_start(metrics, metric::PROCESS_CREATED, 1_500);
}

#[test]
fn counter_tracker_rebaselines_reset_series_only() {
    let mut tracker = CounterTracker::default();
    let disks = vec![DiskStats {
        name: "sda".to_owned(),
        read_bytes: 100,
        write_bytes: 200,
        ..DiskStats::default()
    }];
    let starts = tracker.snapshot(10, 20, None, None, None, Some(&disks), None);

    assert_eq!(starts.get_joined(metric::DISK_IO, "sda", "read", 10), 10);
    assert_eq!(starts.get_joined(metric::DISK_IO, "sda", "write", 10), 10);

    let disks = vec![DiskStats {
        name: "sda".to_owned(),
        read_bytes: 50,
        write_bytes: 250,
        ..DiskStats::default()
    }];
    let starts = tracker.snapshot(10, 30, None, None, None, Some(&disks), None);

    assert_eq!(starts.get_joined(metric::DISK_IO, "sda", "read", 10), 30);
    assert_eq!(starts.get_joined(metric::DISK_IO, "sda", "write", 10), 10);
}

#[test]
fn counter_tracker_rebaselines_paging_operations_by_direction_and_fault_type() {
    let mut tracker = CounterTracker::default();
    let paging = PagingStats {
        swap_in: 100,
        swap_out: 200,
        page_in: 300,
        page_out: 400,
        ..PagingStats::default()
    };
    let starts = tracker.snapshot(10, 20, None, Some(&paging), None, None, None);

    assert_eq!(
        starts.get_joined(metric::PAGING_OPERATIONS, "in", "major", 10),
        10
    );
    assert_eq!(
        starts.get_joined(metric::PAGING_OPERATIONS, "out", "minor", 10),
        10
    );

    let paging = PagingStats {
        swap_in: 50,
        swap_out: 250,
        page_in: 350,
        page_out: 450,
        ..PagingStats::default()
    };
    let starts = tracker.snapshot(10, 30, None, Some(&paging), None, None, None);

    assert_eq!(
        starts.get_joined(metric::PAGING_OPERATIONS, "in", "major", 10),
        30
    );
    assert_eq!(
        starts.get_joined(metric::PAGING_OPERATIONS, "out", "major", 10),
        10
    );
    assert_eq!(
        starts.get_joined(metric::PAGING_OPERATIONS, "in", "minor", 10),
        10
    );
    assert_eq!(
        starts.get_joined(metric::PAGING_OPERATIONS, "out", "minor", 10),
        10
    );
}

#[test]
fn counter_tracker_prunes_disappeared_disk_series_only_when_disk_is_scraped() {
    let mut tracker = CounterTracker::default();
    let disks = vec![DiskStats {
        name: "sda".to_owned(),
        read_bytes: 100,
        write_bytes: 200,
        ..DiskStats::default()
    }];
    let starts = tracker.snapshot(10, 20, None, None, None, Some(&disks), None);
    assert_eq!(starts.get_joined(metric::DISK_IO, "sda", "read", 10), 10);

    let _ = tracker.snapshot(20, 30, None, None, None, None, None);
    let disks = vec![DiskStats {
        name: "sda".to_owned(),
        read_bytes: 150,
        write_bytes: 250,
        ..DiskStats::default()
    }];
    let starts = tracker.snapshot(30, 40, None, None, None, Some(&disks), None);
    assert_eq!(starts.get_joined(metric::DISK_IO, "sda", "read", 30), 30);

    let empty_disks = Vec::new();
    let _ = tracker.snapshot(40, 50, None, None, None, Some(&empty_disks), None);
    let disks = vec![DiskStats {
        name: "sda".to_owned(),
        read_bytes: 200,
        write_bytes: 300,
        ..DiskStats::default()
    }];
    let starts = tracker.snapshot(50, 60, None, None, None, Some(&disks), None);
    assert_eq!(starts.get_joined(metric::DISK_IO, "sda", "read", 50), 50);
}

#[test]
fn counter_tracker_prunes_disappeared_network_series_only_when_network_is_scraped() {
    let mut tracker = CounterTracker::default();
    let networks = vec![NetworkStats {
        name: "veth0".to_owned(),
        rx_bytes: 100,
        tx_bytes: 200,
        ..NetworkStats::default()
    }];
    let starts = tracker.snapshot(10, 20, None, None, None, None, Some(&networks));
    assert_eq!(
        starts.get_joined(metric::NETWORK_IO, "veth0", "receive", 10),
        10
    );

    let _ = tracker.snapshot(20, 30, None, None, None, None, None);
    let networks = vec![NetworkStats {
        name: "veth0".to_owned(),
        rx_bytes: 150,
        tx_bytes: 250,
        ..NetworkStats::default()
    }];
    let starts = tracker.snapshot(30, 40, None, None, None, None, Some(&networks));
    assert_eq!(
        starts.get_joined(metric::NETWORK_IO, "veth0", "receive", 30),
        30
    );

    let empty_networks = Vec::new();
    let _ = tracker.snapshot(40, 50, None, None, None, None, Some(&empty_networks));
    let networks = vec![NetworkStats {
        name: "veth0".to_owned(),
        rx_bytes: 200,
        tx_bytes: 300,
        ..NetworkStats::default()
    }];
    let starts = tracker.snapshot(50, 60, None, None, None, None, Some(&networks));
    assert_eq!(
        starts.get_joined(metric::NETWORK_IO, "veth0", "receive", 50),
        50
    );
}

#[test]
fn counter_keys_do_not_collide_with_pipe_in_series_values() {
    let metric = metric::DISK_IO;
    let device = "read|write";
    let joined = counter_key_joined(metric, device, "read");
    assert!(!counter_key_matches_joined(
        &joined,
        metric,
        "read",
        "write|read"
    ));
    assert!(counter_key_matches_joined(&joined, metric, device, "read"));
}

#[test]
fn scrape_due_emits_successful_families_after_partial_read_error() {
    let root = tempfile::tempdir().expect("tempdir");
    let proc = root.path().join("proc");
    std::fs::create_dir(&proc).expect("proc dir");
    std::fs::write(
        proc.join("meminfo"),
        "MemTotal: 1000 kB\nMemFree: 100 kB\nMemAvailable: 200 kB\n",
    )
    .expect("meminfo");
    // Cumulative metrics read /proc/stat once to cache boot time. Provide
    // btime here so this test only exercises the missing diskstats error.
    std::fs::write(proc.join("stat"), "btime 1700000000\n").expect("stat");
    let mut source = ProcfsSource::new(
        Some(root.path()),
        ProcfsConfig {
            cpu: false,
            memory: true,
            paging: false,
            system: false,
            disk: true,
            filesystem: false,
            network: false,
            processes: false,
            cpu_utilization: false,
            memory_limit: false,
            memory_shared: false,
            memory_hugepages: false,
            disk_limit: false,
            filesystem_include_virtual: false,
            filesystem_include_remote: false,
            filesystem_limit: false,
            filesystem_include_devices: None,
            filesystem_exclude_devices: None,
            filesystem_include_fs_types: None,
            filesystem_exclude_fs_types: None,
            filesystem_include_mount_points: None,
            filesystem_exclude_mount_points: None,
            disk_include: None,
            disk_exclude: None,
            network_include: None,
            network_exclude: None,
            validation: HostViewValidationMode::None,
        },
    )
    .expect("source");

    let scrape = block_on_scrape(
        &mut source,
        ProcfsFamilies {
            memory: true,
            disk: true,
            ..ProcfsFamilies::default()
        },
    )
    .expect("partial scrape");

    assert_eq!(scrape.partial_errors, 1);
    assert!(scrape.snapshot.memory.is_some());
    assert!(scrape.snapshot.disks.is_empty());
}

#[test]
fn scrape_due_preserves_disk_counter_state_after_diskstats_read_error() {
    let root = tempfile::tempdir().expect("tempdir");
    let proc = root.path().join("proc");
    std::fs::create_dir_all(&proc).expect("proc dir");
    std::fs::write(proc.join("stat"), "btime 1700000000\n").expect("stat");
    std::fs::write(
        proc.join("meminfo"),
        "MemTotal: 1000 kB\nMemFree: 100 kB\nMemAvailable: 200 kB\n",
    )
    .expect("meminfo");
    std::fs::write(
        proc.join("diskstats"),
        "8 0 sda 1 0 100 0 2 0 200 0 0 0 0 0 0 0 0\n",
    )
    .expect("diskstats");
    let mut source = ProcfsSource::new(
        Some(root.path()),
        ProcfsConfig {
            cpu: false,
            memory: true,
            paging: false,
            system: false,
            disk: true,
            filesystem: false,
            network: false,
            processes: false,
            cpu_utilization: false,
            memory_limit: false,
            memory_shared: false,
            memory_hugepages: false,
            disk_limit: false,
            filesystem_include_virtual: false,
            filesystem_include_remote: false,
            filesystem_limit: false,
            filesystem_include_devices: None,
            filesystem_exclude_devices: None,
            filesystem_include_fs_types: None,
            filesystem_exclude_fs_types: None,
            filesystem_include_mount_points: None,
            filesystem_exclude_mount_points: None,
            disk_include: None,
            disk_exclude: None,
            network_include: None,
            network_exclude: None,
            validation: HostViewValidationMode::None,
        },
    )
    .expect("source");

    let first = block_on_scrape(
        &mut source,
        ProcfsFamilies {
            disk: true,
            ..ProcfsFamilies::default()
        },
    )
    .expect("first disk scrape");
    let first_start = first
        .snapshot
        .counter_starts
        .get_joined(metric::DISK_IO, "sda", "read", 0);

    std::fs::remove_file(proc.join("diskstats")).expect("remove diskstats");
    let partial = block_on_scrape(
        &mut source,
        ProcfsFamilies {
            memory: true,
            disk: true,
            ..ProcfsFamilies::default()
        },
    )
    .expect("partial scrape");
    assert_eq!(partial.partial_errors, 1);
    assert!(partial.snapshot.disks.is_empty());

    std::fs::write(
        proc.join("diskstats"),
        "8 0 sda 1 0 50 0 2 0 100 0 0 0 0 0 0 0 0\n",
    )
    .expect("diskstats after reset");
    let after_error = block_on_scrape(
        &mut source,
        ProcfsFamilies {
            disk: true,
            ..ProcfsFamilies::default()
        },
    )
    .expect("disk scrape after read error");
    let reset_start =
        after_error
            .snapshot
            .counter_starts
            .get_joined(metric::DISK_IO, "sda", "read", 0);
    assert!(
        reset_start > first_start,
        "disk counter state should survive read errors so the later reset is detected"
    );
}

#[test]
fn scrape_due_uses_stable_fallback_start_time_when_stat_is_unavailable() {
    let root = tempfile::tempdir().expect("tempdir");
    let proc = root.path().join("proc");
    std::fs::create_dir(&proc).expect("proc dir");
    std::fs::write(
        proc.join("diskstats"),
        "8 0 sda 1 0 100 0 2 0 200 0 0 0 0 0 0 0 0\n",
    )
    .expect("diskstats");
    let mut source = ProcfsSource::new(
        Some(root.path()),
        ProcfsConfig {
            cpu: false,
            memory: false,
            paging: false,
            system: false,
            disk: true,
            filesystem: false,
            network: false,
            processes: false,
            cpu_utilization: false,
            memory_limit: false,
            memory_shared: false,
            memory_hugepages: false,
            disk_limit: false,
            filesystem_include_virtual: false,
            filesystem_include_remote: false,
            filesystem_limit: false,
            filesystem_include_devices: None,
            filesystem_exclude_devices: None,
            filesystem_include_fs_types: None,
            filesystem_exclude_fs_types: None,
            filesystem_include_mount_points: None,
            filesystem_exclude_mount_points: None,
            disk_include: None,
            disk_exclude: None,
            network_include: None,
            network_exclude: None,
            validation: HostViewValidationMode::None,
        },
    )
    .expect("source");

    let first = block_on_scrape(
        &mut source,
        ProcfsFamilies {
            disk: true,
            ..ProcfsFamilies::default()
        },
    )
    .expect("first disk scrape");
    std::thread::sleep(Duration::from_millis(1));
    let second = block_on_scrape(
        &mut source,
        ProcfsFamilies {
            disk: true,
            ..ProcfsFamilies::default()
        },
    )
    .expect("second disk scrape");

    assert_eq!(first.partial_errors, 1);
    assert_eq!(second.partial_errors, 1);
    assert_eq!(
        first.snapshot.start_time_unix_nano,
        second.snapshot.start_time_unix_nano
    );
}

#[test]
fn validation_requires_stat_for_cumulative_families() {
    let root = tempfile::tempdir().expect("tempdir");
    let proc = root.path().join("proc");
    std::fs::create_dir(&proc).expect("proc dir");
    std::fs::write(
        proc.join("diskstats"),
        "8 0 sda 1 0 100 0 2 0 200 0 0 0 0 0 0 0 0\n",
    )
    .expect("diskstats");

    let err = match ProcfsSource::new(
        Some(root.path()),
        ProcfsConfig {
            cpu: false,
            memory: false,
            paging: false,
            system: false,
            disk: true,
            filesystem: false,
            network: false,
            processes: false,
            cpu_utilization: false,
            memory_limit: false,
            memory_shared: false,
            memory_hugepages: false,
            disk_limit: false,
            filesystem_include_virtual: false,
            filesystem_include_remote: false,
            filesystem_limit: false,
            filesystem_include_devices: None,
            filesystem_exclude_devices: None,
            filesystem_include_fs_types: None,
            filesystem_exclude_fs_types: None,
            filesystem_include_mount_points: None,
            filesystem_exclude_mount_points: None,
            disk_include: None,
            disk_exclude: None,
            network_include: None,
            network_exclude: None,
            validation: HostViewValidationMode::FailSelected,
        },
    ) {
        Ok(_) => panic!("missing stat should fail validation for cumulative disk metrics"),
        Err(err) => err,
    };

    assert_eq!(err.kind(), ErrorKind::NotFound);
}

#[test]
fn filesystem_stat_worker_reports_disconnect_as_broken_pipe() {
    let worker = FilesystemStatWorker::disconnected_for_test();
    match worker.statvfs(PathBuf::from("/"), Duration::from_millis(1)) {
        Ok(_) => panic!("worker is disconnected"),
        Err(err) => assert_eq!(err.kind(), ErrorKind::BrokenPipe),
    }
}

#[test]
fn scrape_due_fails_when_all_due_families_fail() {
    let root = tempfile::tempdir().expect("tempdir");
    let mut source = ProcfsSource::new(
        Some(root.path()),
        ProcfsConfig {
            cpu: false,
            memory: true,
            paging: false,
            system: false,
            disk: false,
            filesystem: false,
            network: false,
            processes: false,
            cpu_utilization: false,
            memory_limit: false,
            memory_shared: false,
            memory_hugepages: false,
            disk_limit: false,
            filesystem_include_virtual: false,
            filesystem_include_remote: false,
            filesystem_limit: false,
            filesystem_include_devices: None,
            filesystem_exclude_devices: None,
            filesystem_include_fs_types: None,
            filesystem_exclude_fs_types: None,
            filesystem_include_mount_points: None,
            filesystem_exclude_mount_points: None,
            disk_include: None,
            disk_exclude: None,
            network_include: None,
            network_exclude: None,
            validation: HostViewValidationMode::None,
        },
    )
    .expect("source");

    assert!(
        block_on_scrape(
            &mut source,
            ProcfsFamilies {
                memory: true,
                ..ProcfsFamilies::default()
            },
        )
        .is_err()
    );
}

#[test]
fn scrape_due_reads_opt_in_disk_limit_from_sysfs() {
    let root = tempfile::tempdir().expect("tempdir");
    let proc = root.path().join("proc");
    let sys_sda = root.path().join("sys/block/sda");
    std::fs::create_dir(&proc).expect("proc dir");
    std::fs::create_dir_all(&sys_sda).expect("sys block dir");
    std::fs::write(
        proc.join("diskstats"),
        "8 0 sda 1 0 2 3 4 0 5 6 0 0 0 0 0 0 0 0\n",
    )
    .expect("diskstats");
    std::fs::write(sys_sda.join("size"), "4096\n").expect("disk size");
    let mut source = ProcfsSource::new(
        Some(root.path()),
        ProcfsConfig {
            cpu: false,
            memory: false,
            paging: false,
            system: false,
            disk: true,
            filesystem: false,
            network: false,
            processes: false,
            cpu_utilization: false,
            memory_limit: false,
            memory_shared: false,
            memory_hugepages: false,
            disk_limit: true,
            filesystem_include_virtual: false,
            filesystem_include_remote: false,
            filesystem_limit: false,
            filesystem_include_devices: None,
            filesystem_exclude_devices: None,
            filesystem_include_fs_types: None,
            filesystem_exclude_fs_types: None,
            filesystem_include_mount_points: None,
            filesystem_exclude_mount_points: None,
            disk_include: None,
            disk_exclude: None,
            network_include: None,
            network_exclude: None,
            validation: HostViewValidationMode::None,
        },
    )
    .expect("source");

    let scrape = block_on_scrape(
        &mut source,
        ProcfsFamilies {
            disk: true,
            ..ProcfsFamilies::default()
        },
    )
    .expect("disk scrape");

    assert_eq!(scrape.snapshot.disks.len(), 1);
    assert_eq!(
        scrape.snapshot.disks[0].limit_bytes,
        Some(4096 * DISKSTAT_SECTOR_BYTES)
    );
}

#[test]
fn scrape_due_uses_boot_time_for_counter_only_family_ticks() {
    let root = tempfile::tempdir().expect("tempdir");
    let proc = root.path().join("proc");
    let proc_one = proc.join("1");
    std::fs::create_dir_all(proc_one.join("net")).expect("proc dirs");
    std::fs::write(proc.join("stat"), "btime 123\n").expect("stat");
    std::fs::write(
        proc.join("diskstats"),
        "8 0 sda 1 0 2 3 4 0 5 6 0 0 0 0 0 0 0 0\n",
    )
    .expect("diskstats");
    std::fs::write(
        proc_one.join("net/dev"),
        "Inter-|   Receive                                                |  Transmit\n\
          face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed\n\
          eth0: 10 1 0 0 0 0 0 0 20 2 0 0 0 0 0 0\n",
    )
    .expect("netdev");
    std::fs::write(
        proc.join("vmstat"),
        "pgfault 10\npgmajfault 1\npgpgin 2\npgpgout 3\npswpin 4\npswpout 5\n",
    )
    .expect("vmstat");
    std::fs::write(proc.join("swaps"), "Filename Type Size Used Priority\n").expect("swaps");

    let mut source = ProcfsSource::new(
        Some(root.path()),
        ProcfsConfig {
            cpu: false,
            memory: false,
            paging: true,
            system: false,
            disk: true,
            filesystem: false,
            network: true,
            processes: false,
            cpu_utilization: false,
            memory_limit: false,
            memory_shared: false,
            memory_hugepages: false,
            disk_limit: false,
            filesystem_include_virtual: false,
            filesystem_include_remote: false,
            filesystem_limit: false,
            filesystem_include_devices: None,
            filesystem_exclude_devices: None,
            filesystem_include_fs_types: None,
            filesystem_exclude_fs_types: None,
            filesystem_include_mount_points: None,
            filesystem_exclude_mount_points: None,
            disk_include: None,
            disk_exclude: None,
            network_include: None,
            network_exclude: None,
            validation: HostViewValidationMode::None,
        },
    )
    .expect("source");

    let expected_start = 123 * NANOS_PER_SEC;
    let disk_scrape = block_on_scrape(
        &mut source,
        ProcfsFamilies {
            disk: true,
            ..ProcfsFamilies::default()
        },
    )
    .expect("disk scrape");
    assert_eq!(disk_scrape.snapshot.start_time_unix_nano, expected_start);
    assert_eq!(disk_scrape.snapshot.disks.len(), 1);

    std::fs::remove_file(proc.join("stat")).expect("remove stat after cache");

    let network_scrape = block_on_scrape(
        &mut source,
        ProcfsFamilies {
            network: true,
            ..ProcfsFamilies::default()
        },
    )
    .expect("network scrape");
    assert_eq!(network_scrape.snapshot.start_time_unix_nano, expected_start);
    assert_eq!(network_scrape.snapshot.networks.len(), 1);

    let paging_scrape = block_on_scrape(
        &mut source,
        ProcfsFamilies {
            paging: true,
            ..ProcfsFamilies::default()
        },
    )
    .expect("paging scrape");
    assert_eq!(paging_scrape.snapshot.start_time_unix_nano, expected_start);
    assert!(paging_scrape.snapshot.paging.is_some());
}

#[test]
fn scrape_due_reads_filesystem_usage_from_mountinfo() {
    let root = tempfile::tempdir().expect("tempdir");
    let proc_one = root.path().join("proc/1");
    std::fs::create_dir_all(&proc_one).expect("proc one dir");
    std::fs::write(
        proc_one.join("mountinfo"),
        "36 25 8:1 / / rw,relatime - ext4 /dev/sda1 rw\n",
    )
    .expect("mountinfo");
    let mut source = ProcfsSource::new(
        Some(root.path()),
        ProcfsConfig {
            cpu: false,
            memory: false,
            paging: false,
            system: false,
            disk: false,
            filesystem: true,
            network: false,
            processes: false,
            cpu_utilization: false,
            memory_limit: false,
            memory_shared: false,
            memory_hugepages: false,
            disk_limit: false,
            filesystem_include_virtual: false,
            filesystem_include_remote: false,
            filesystem_limit: true,
            filesystem_include_devices: None,
            filesystem_exclude_devices: None,
            filesystem_include_fs_types: None,
            filesystem_exclude_fs_types: None,
            filesystem_include_mount_points: None,
            filesystem_exclude_mount_points: None,
            disk_include: None,
            disk_exclude: None,
            network_include: None,
            network_exclude: None,
            validation: HostViewValidationMode::None,
        },
    )
    .expect("source");

    let scrape = block_on_scrape(
        &mut source,
        ProcfsFamilies {
            filesystem: true,
            ..ProcfsFamilies::default()
        },
    )
    .expect("filesystem scrape");

    assert_eq!(scrape.snapshot.filesystems.len(), 1);
    assert_eq!(scrape.snapshot.filesystems[0].device, "/dev/sda1");
    assert_eq!(scrape.snapshot.filesystems[0].mountpoint, "/");
    assert_eq!(scrape.snapshot.filesystems[0].fs_type, "ext4");
    assert!(scrape.snapshot.filesystems[0].limit_bytes.is_some());
}

#[test]
fn cpu_parser_accepts_missing_newer_fields() {
    let cpu = parse_cpu_total("10 20 30 40", 10.0).expect("cpu row");
    assert_eq!(cpu.user, 1.0);
    assert_eq!(cpu.nice, 2.0);
    assert_eq!(cpu.system, 3.0);
    assert_eq!(cpu.idle, 4.0);
    assert_eq!(cpu.steal, 0.0);
}

#[test]
fn cpu_parser_removes_guest_from_user_and_nice() {
    let cpu = parse_cpu_total("100 50 30 40 5 2 3 7 10 4", 10.0).expect("cpu row");
    assert_eq!(cpu.user, 9.0);
    assert_eq!(cpu.nice, 4.6);
    assert_eq!(cpu.interrupt, 0.5);
}

#[test]
fn cpu_utilization_uses_counter_deltas() {
    let utilization = cpu_utilization(
        CpuTimes {
            user: 1.0,
            idle: 1.0,
            ..CpuTimes::default()
        },
        CpuTimes {
            user: 3.0,
            idle: 2.0,
            ..CpuTimes::default()
        },
    )
    .expect("utilization");

    assert_eq!(utilization.user, 2.0 / 3.0);
    assert_eq!(utilization.idle, 1.0 / 3.0);
}

#[test]
fn cpu_utilization_skips_counter_resets() {
    assert!(
        cpu_utilization(
            CpuTimes {
                user: 2.0,
                ..CpuTimes::default()
            },
            CpuTimes {
                user: 1.0,
                ..CpuTimes::default()
            },
        )
        .is_none()
    );
}

#[test]
fn clock_ticks_per_second_uses_positive_system_value() {
    assert!(clock_ticks_per_second() > 0.0);
}

#[test]
fn memavailable_fallback_uses_free_buffers_cached() {
    let memory =
        parse_meminfo("MemTotal: 1000 kB\nMemFree: 100 kB\nBuffers: 20 kB\nCached: 30 kB\n")
            .expect("memory");
    assert!(!memory.has_available);
    assert_eq!(memory.available, 150 * BYTES_PER_KIB);
    assert_eq!(memory.used, 850 * BYTES_PER_KIB);
}

#[test]
fn meminfo_parser_reads_shared_memory() {
    let memory =
        parse_meminfo("MemTotal: 1000 kB\nMemFree: 100 kB\nShmem: 12 kB\n").expect("memory");
    assert_eq!(memory.shared, 12 * BYTES_PER_KIB);
}

#[test]
fn meminfo_parser_reads_hugepage_stats() {
    let memory = parse_meminfo(
        "MemTotal: 1000 kB\n\
         MemFree: 100 kB\n\
         HugePages_Total: 8\n\
         HugePages_Free: 3\n\
         HugePages_Rsvd: 2\n\
         HugePages_Surp: 1\n\
         Hugepagesize: 2048 kB\n",
    )
    .expect("memory");

    assert_eq!(memory.hugepages.total, 8);
    assert_eq!(memory.hugepages.free, 3);
    assert_eq!(memory.hugepages.reserved, 2);
    assert_eq!(memory.hugepages.surplus, 1);
    assert_eq!(memory.hugepages.page_size_bytes, 2048 * BYTES_PER_KIB);
}

#[test]
fn uptime_parser_reads_first_field() {
    assert_eq!(parse_uptime("123.45 67.89"), Some(123.45));
}

#[test]
fn vmstat_parser_derives_minor_faults() {
    let paging =
        parse_vmstat("pgfault 100\npgmajfault 7\npgpgin 5\npgpgout 6\npswpin 3\npswpout 4\n");
    assert_eq!(paging.minor_faults, 93);
    assert_eq!(paging.major_faults, 7);
    assert_eq!(paging.page_in, 5);
    assert_eq!(paging.page_out, 6);
    assert_eq!(paging.swap_in, 3);
    assert_eq!(paging.swap_out, 4);
}

#[test]
fn swaps_parser_reads_device_usage() {
    let swaps = parse_swaps("Filename Type Size Used Priority\n/dev/sda2 partition 200 50 -2\n");
    assert_eq!(swaps.len(), 1);
    assert_eq!(swaps[0].name, "/dev/sda2");
    assert_eq!(swaps[0].used, 50 * BYTES_PER_KIB);
    assert_eq!(swaps[0].free, 150 * BYTES_PER_KIB);
}

#[test]
fn diskstats_parser_accepts_flush_columns() {
    let disks = parse_diskstats("8 0 sda 1 0 2 3 4 0 5 6 0 0 0 0 0 0 0 0\n", None, None);
    assert_eq!(disks.len(), 1);
    assert_eq!(disks[0].name, "sda");
    assert_eq!(disks[0].read_bytes, 1024);
    assert_eq!(disks[0].write_bytes, 2560);
}

#[test]
fn diskstats_parser_applies_filters_before_parsing_values() {
    let exclude = CompiledFilter::compile(
        crate::receivers::host_metrics_receiver::MatchType::Glob,
        vec!["loop*".to_owned()],
    )
    .expect("valid")
    .expect("filter");
    let disks = parse_diskstats(
        "7 0 loop0 broken row\n8 0 sda 1 0 2 3 4 0 5 6 0 0 0 0 0 0 0 0\n",
        None,
        Some(&exclude),
    );

    assert_eq!(disks.len(), 1);
    assert_eq!(disks[0].name, "sda");
}

#[test]
fn mountinfo_parser_skips_virtual_and_remote_filesystems_by_default() {
    let mounts = parse_mountinfo(
        "36 25 8:1 / / rw,relatime - ext4 /dev/sda1 rw\n\
         37 25 0:32 / /proc rw,nosuid,nodev,noexec,relatime - proc proc rw\n\
         38 25 0:33 / /mnt/fuse rw,relatime - fuse.sshfs sshfs rw\n\
         39 25 0:34 / /mnt/fuseblk rw,relatime - fuseblk /dev/fuse rw\n\
         40 25 0:35 / /mnt/nfs rw,relatime - nfs server:/export rw\n\
         41 25 0:36 / /dev/pts rw,nosuid,noexec,relatime - devpts devpts rw\n\
         42 25 0:37 / /sys/kernel/security rw,nosuid,nodev,noexec,relatime - securityfs securityfs rw\n",
        false,
        false,
        true,
        FilesystemFilters::default(),
    );

    assert_eq!(mounts.len(), 1);
    assert_eq!(mounts[0].device, "/dev/sda1");
    assert_eq!(mounts[0].mountpoint, "/");
    assert_eq!(mounts[0].fs_type, "ext4");
    assert_eq!(mounts[0].mode, "rw");
    assert!(mounts[0].emit_limit);
}

#[test]
fn mountinfo_parser_keeps_remote_filesystems_separate_from_virtual_filesystems() {
    let mountinfo = "36 25 8:1 / / rw,relatime - ext4 /dev/sda1 rw\n\
         37 25 0:32 / /run rw,nosuid,nodev - tmpfs tmpfs rw\n\
         38 25 0:33 / /mnt/fuse rw,relatime - fuse.sshfs sshfs rw\n\
         39 25 0:34 / /mnt/fuseblk rw,relatime - fuseblk /dev/fuse rw\n\
         40 25 0:35 / /mnt/nfs rw,relatime - nfs server:/export rw\n\
         41 25 0:36 / /mnt/cifs rw,relatime - cifs //server/share rw\n\
         42 25 0:37 / /mnt/9p rw,relatime - 9p hostshare rw\n";

    let virtual_only = parse_mountinfo(mountinfo, true, false, false, FilesystemFilters::default());
    assert_eq!(virtual_only.len(), 2);
    assert_eq!(virtual_only[0].fs_type, "ext4");
    assert_eq!(virtual_only[1].fs_type, "tmpfs");

    let remote_only = parse_mountinfo(mountinfo, false, true, false, FilesystemFilters::default());
    assert_eq!(remote_only.len(), 6);
    assert_eq!(remote_only[0].fs_type, "ext4");
    assert_eq!(remote_only[1].fs_type, "fuse.sshfs");
    assert_eq!(remote_only[2].fs_type, "fuseblk");
    assert_eq!(remote_only[3].fs_type, "nfs");
    assert_eq!(remote_only[4].fs_type, "cifs");
    assert_eq!(remote_only[5].fs_type, "9p");

    let all_included = parse_mountinfo(mountinfo, true, true, false, FilesystemFilters::default());
    assert_eq!(all_included.len(), 7);
}

#[test]
fn mountinfo_parser_applies_filters_after_remote_filesystem_opt_in() {
    let include_fs_types = CompiledFilter::compile(
        crate::receivers::host_metrics_receiver::MatchType::Strict,
        vec!["nfs".to_owned()],
    )
    .expect("valid")
    .expect("filter");
    let mounts = parse_mountinfo(
        "36 25 8:1 / / rw,relatime - ext4 /dev/sda1 rw\n\
         37 25 0:35 / /mnt/nfs rw,relatime - nfs server:/export rw\n\
         38 25 0:36 / /mnt/cifs rw,relatime - cifs //server/share rw\n",
        false,
        true,
        false,
        FilesystemFilters {
            include_fs_types: Some(&include_fs_types),
            ..FilesystemFilters::default()
        },
    );

    assert_eq!(mounts.len(), 1);
    assert_eq!(mounts[0].fs_type, "nfs");
}

#[test]
fn mountinfo_parser_unescapes_paths() {
    let mounts = parse_mountinfo(
        "36 25 8:1 / /mnt/data\\040disk rw,relatime - ext4 /dev/disk\\040one rw\n",
        false,
        false,
        false,
        FilesystemFilters::default(),
    );

    assert_eq!(mounts.len(), 1);
    assert_eq!(mounts[0].device, "/dev/disk one");
    assert_eq!(mounts[0].mountpoint, "/mnt/data disk");
}

#[test]
fn mountinfo_parser_preserves_utf8_while_unescaping_paths() {
    let mounts = parse_mountinfo(
        "36 25 8:1 / /mnt/caf\u{00e9}\\040disk rw,relatime - ext4 /dev/disk\\040\u{00e9} rw\n",
        false,
        false,
        false,
        FilesystemFilters::default(),
    );

    assert_eq!(mounts.len(), 1);
    assert_eq!(mounts[0].device, "/dev/disk \u{00e9}");
    assert_eq!(mounts[0].mountpoint, "/mnt/caf\u{00e9} disk");
}

#[test]
fn mountinfo_parser_applies_filesystem_filters() {
    let include_mounts = CompiledFilter::compile(
        crate::receivers::host_metrics_receiver::MatchType::Glob,
        vec!["/data*".to_owned()],
    )
    .expect("valid")
    .expect("filter");
    let exclude_fs_types = CompiledFilter::compile(
        crate::receivers::host_metrics_receiver::MatchType::Strict,
        vec!["xfs".to_owned()],
    )
    .expect("valid")
    .expect("filter");
    let mounts = parse_mountinfo(
        "36 25 8:1 / / rw,relatime - ext4 /dev/sda1 rw\n37 25 8:2 / /data rw,relatime - ext4 /dev/sdb1 rw\n38 25 8:3 / /data2 rw,relatime - xfs /dev/sdc1 rw\n",
        false,
        false,
        false,
        FilesystemFilters {
            include_mount_points: Some(&include_mounts),
            exclude_fs_types: Some(&exclude_fs_types),
            ..FilesystemFilters::default()
        },
    );

    assert_eq!(mounts.len(), 1);
    assert_eq!(mounts[0].device, "/dev/sdb1");
    assert_eq!(mounts[0].mountpoint, "/data");
}

#[test]
fn netdev_parser_reads_device_counters() {
    let interfaces = parse_netdev(
        "Inter-| Receive | Transmit\n face |bytes packets errs drop fifo frame compressed multicast|bytes packets errs drop fifo colls carrier compressed\n eth0: 10 2 0 0 0 0 0 0 30 4 0 0 0 0 0 0\n",
        None,
        None,
    );
    assert_eq!(interfaces.len(), 1);
    assert_eq!(interfaces[0].name, "eth0");
    assert_eq!(interfaces[0].rx_bytes, 10);
    assert_eq!(interfaces[0].tx_packets, 4);
}

#[test]
fn netdev_parser_applies_interface_filters() {
    let include = CompiledFilter::compile(
        crate::receivers::host_metrics_receiver::MatchType::Strict,
        vec!["eth0".to_owned()],
    )
    .expect("valid")
    .expect("filter");
    let interfaces = parse_netdev(
        "Inter-| Receive | Transmit\n face |bytes packets errs drop fifo frame compressed multicast|bytes packets errs drop fifo colls carrier compressed\n lo: 1 1 0 0 0 0 0 0 1 1 0 0 0 0 0 0\n eth0: 10 2 3 4 0 0 0 0 30 4 5 6 0 0 0 0\n",
        Some(&include),
        None,
    );

    assert_eq!(interfaces.len(), 1);
    assert_eq!(interfaces[0].name, "eth0");
    assert_eq!(interfaces[0].rx_errors, 3);
    assert_eq!(interfaces[0].tx_dropped, 6);
}

#[test]
fn root_path_uses_host_pid_one_netdev() {
    let paths = ProcfsPaths::new(Some(Path::new("/host")));
    assert_eq!(paths.net_dev, PathBuf::from("/host/proc/1/net/dev"));
    assert_eq!(paths.mountinfo, PathBuf::from("/host/proc/1/mountinfo"));
}

#[test]
fn root_slash_uses_current_proc_netdev() {
    let paths = ProcfsPaths::new(Some(Path::new("/")));
    assert_eq!(paths.net_dev, PathBuf::from("/proc/net/dev"));
    assert_eq!(paths.mountinfo, PathBuf::from("/proc/self/mountinfo"));
}

#[test]
fn host_arch_uses_semconv_values() {
    if let Some(arch) = host_arch() {
        assert!(matches!(
            arch,
            "amd64" | "arm32" | "arm64" | "ppc32" | "ppc64" | "x86"
        ));
    }
}

#[cfg(feature = "dev-tools")]
#[derive(Debug)]
struct MetricShape {
    unit: String,
    monotonic: Option<bool>,
    attributes: BTreeSet<String>,
    all_attributes: BTreeSet<String>,
    attribute_types: BTreeMap<String, AttributeValueKind>,
    enum_values: BTreeMap<String, BTreeSet<String>>,
    value_type: Option<MetricValueKind>,
}

#[cfg(feature = "dev-tools")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MetricValueKind {
    Int,
    Double,
}

#[cfg(feature = "dev-tools")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AttributeValueKind {
    Int,
    Double,
    String,
    Bool,
}

#[cfg(feature = "dev-tools")]
fn load_semconv_registry() -> ResolvedRegistry {
    let registry_path = std::env::var("OTAP_HOST_METRICS_SEMCONV_REGISTRY")
        .map(|path| {
            path.parse::<VirtualDirectoryPath>()
                .expect("valid OTAP_HOST_METRICS_SEMCONV_REGISTRY")
        })
        .unwrap_or_else(|_| VirtualDirectoryPath::GitRepo {
            url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
            sub_folder: Some("model".to_owned()),
            refspec: Some(format!(
                "v{}",
                crate::receivers::host_metrics_receiver::semconv::VERSION
            )),
        });

    let registry_repo =
        RegistryRepo::try_new("main", &registry_path).expect("semantic convention registry");
    let registry = match SchemaResolver::load_semconv_repository(registry_repo, false) {
        WResult::Ok(registry) | WResult::OkWithNFEs(registry, _) => registry,
        WResult::FatalErr(err) => panic!("failed to load semantic convention registry: {err}"),
    };
    let resolved_schema = match SchemaResolver::resolve(registry, true) {
        WResult::Ok(schema) | WResult::OkWithNFEs(schema, _) => schema,
        WResult::FatalErr(err) => {
            panic!("failed to resolve semantic convention registry: {err}");
        }
    };

    ResolvedRegistry::try_from_resolved_registry(
        &resolved_schema.registry,
        resolved_schema.catalog(),
    )
    .expect("resolved semantic convention registry")
}

#[cfg(feature = "dev-tools")]
fn semconv_system_metric_shapes(registry: &ResolvedRegistry) -> BTreeMap<String, MetricShape> {
    registry
        .groups
        .iter()
        .filter(|group| group.r#type == GroupType::Metric)
        .filter_map(|group| {
            let name = group.metric_name.as_ref()?;
            if !name.starts_with("system.") {
                return None;
            }

            let monotonic = match group.instrument.as_ref()? {
                InstrumentSpec::Counter => Some(true),
                InstrumentSpec::UpDownCounter => Some(false),
                InstrumentSpec::Gauge | InstrumentSpec::Histogram => None,
            };
            let attributes = group
                .attributes
                .iter()
                .filter(|attr| !is_opt_in_requirement(&attr.requirement_level))
                .map(|attr| attr.name.clone())
                .collect();
            let all_attributes = group
                .attributes
                .iter()
                .map(|attr| attr.name.clone())
                .collect();
            let enum_values = group
                .attributes
                .iter()
                .filter_map(|attr| match &attr.r#type {
                    AttributeType::Enum { members } => Some((
                        attr.name.clone(),
                        members
                            .iter()
                            .map(|member| value_spec_string(&member.value))
                            .collect(),
                    )),
                    _ => None,
                })
                .collect();
            let attribute_types = group
                .attributes
                .iter()
                .filter_map(|attr| {
                    attribute_value_kind(&attr.r#type).map(|kind| (attr.name.clone(), kind))
                })
                .collect();

            Some((
                name.clone(),
                MetricShape {
                    unit: group.unit.clone().unwrap_or_default(),
                    monotonic,
                    attributes,
                    all_attributes,
                    attribute_types,
                    enum_values,
                    value_type: semconv_metric_value_type(group.annotations.as_ref()),
                },
            ))
        })
        .collect()
}

#[cfg(feature = "dev-tools")]
fn semconv_metric_value_type(
    annotations: Option<&BTreeMap<String, weaver_semconv::YamlValue>>,
) -> Option<MetricValueKind> {
    let code_generation = annotations?.get("code_generation")?.0.as_mapping()?;
    let value_type = code_generation.iter().find_map(|(key, value)| {
        (key.as_str() == Some("metric_value_type")).then(|| value.as_str())?
    })?;
    match value_type {
        "int" => Some(MetricValueKind::Int),
        "double" => Some(MetricValueKind::Double),
        _ => None,
    }
}

#[cfg(feature = "dev-tools")]
fn value_spec_string(value: &ValueSpec) -> String {
    match value {
        ValueSpec::Int(value) => value.to_string(),
        ValueSpec::Double(value) => value.to_string(),
        ValueSpec::String(value) => value.clone(),
        ValueSpec::Bool(value) => value.to_string(),
    }
}

#[cfg(feature = "dev-tools")]
fn attribute_value_kind(attribute_type: &AttributeType) -> Option<AttributeValueKind> {
    match attribute_type {
        AttributeType::PrimitiveOrArray(PrimitiveOrArrayTypeSpec::Int) => {
            Some(AttributeValueKind::Int)
        }
        AttributeType::PrimitiveOrArray(PrimitiveOrArrayTypeSpec::Double) => {
            Some(AttributeValueKind::Double)
        }
        AttributeType::PrimitiveOrArray(PrimitiveOrArrayTypeSpec::String) => {
            Some(AttributeValueKind::String)
        }
        AttributeType::PrimitiveOrArray(PrimitiveOrArrayTypeSpec::Boolean) => {
            Some(AttributeValueKind::Bool)
        }
        AttributeType::Enum { members } => {
            members.first().map(|member| value_spec_kind(&member.value))
        }
        _ => None,
    }
}

#[cfg(feature = "dev-tools")]
fn value_spec_kind(value: &ValueSpec) -> AttributeValueKind {
    match value {
        ValueSpec::Int(_) => AttributeValueKind::Int,
        ValueSpec::Double(_) => AttributeValueKind::Double,
        ValueSpec::String(_) => AttributeValueKind::String,
        ValueSpec::Bool(_) => AttributeValueKind::Bool,
    }
}

#[cfg(feature = "dev-tools")]
fn is_intentional_semconv_enum_value_gap(_name: &str, _attr: &str, _value: &str) -> bool {
    false
}

#[cfg(feature = "dev-tools")]
fn is_opt_in_requirement(requirement_level: &RequirementLevel) -> bool {
    matches!(
        requirement_level,
        RequirementLevel::Basic(BasicRequirementLevelSpec::OptIn) | RequirementLevel::OptIn { .. }
    )
}

#[cfg(feature = "dev-tools")]
fn emitted_phase1_metric_shapes() -> BTreeMap<String, MetricShape> {
    let metrics = projection_fixture_metrics();
    let mut shapes = BTreeMap::new();
    for metric in &metrics {
        let (monotonic, points) = match metric.data.as_ref().expect("metric data") {
            otlp_metric::Data::Sum(sum) => (Some(sum.is_monotonic), &sum.data_points),
            otlp_metric::Data::Gauge(gauge) => (None, &gauge.data_points),
            _ => panic!("unsupported metric data for {}", metric.name),
        };
        let value_type = metric_value_type(points);
        let shape = shapes
            .entry(metric.name.clone())
            .or_insert_with(|| MetricShape {
                unit: metric.unit.clone(),
                monotonic,
                attributes: BTreeSet::new(),
                all_attributes: BTreeSet::new(),
                attribute_types: BTreeMap::new(),
                enum_values: BTreeMap::new(),
                value_type,
            });
        assert_eq!(
            shape.unit, metric.unit,
            "unit mismatch across {}",
            metric.name
        );
        assert_eq!(
            shape.monotonic, monotonic,
            "instrument/temporality mismatch across {}",
            metric.name
        );
        assert_eq!(
            shape.value_type, value_type,
            "value type mismatch across {}",
            metric.name
        );
        for attr in points.iter().flat_map(|point| point.attributes.iter()) {
            let _ = shape.attributes.insert(attr.key.clone());
            if let Some(value) = any_value_string(attr.value.as_ref()) {
                let _ = shape
                    .enum_values
                    .entry(attr.key.clone())
                    .or_default()
                    .insert(value);
            }
            if let Some(kind) = any_value_kind(attr.value.as_ref()) {
                let previous = shape.attribute_types.insert(attr.key.clone(), kind);
                assert!(
                    previous.is_none() || previous == Some(kind),
                    "mixed attribute value types for {} on {}",
                    attr.key,
                    metric.name
                );
            }
        }
    }
    shapes
}

#[cfg(feature = "dev-tools")]
fn metric_value_type(points: &[NumberDataPoint]) -> Option<MetricValueKind> {
    let mut value_type = None;
    for point in points {
        let point_value_type = match point.value {
            Some(number_data_point::Value::AsInt(_)) => MetricValueKind::Int,
            Some(number_data_point::Value::AsDouble(_)) => MetricValueKind::Double,
            None => continue,
        };
        if value_type
            .replace(point_value_type)
            .is_some_and(|current| current != point_value_type)
        {
            panic!("mixed int/double data points");
        }
    }
    value_type
}

#[cfg(feature = "dev-tools")]
fn any_value_string(value: Option<&AnyValue>) -> Option<String> {
    match value?.value.as_ref()? {
        any_value::Value::StringValue(value) => Some(value.clone()),
        any_value::Value::IntValue(value) => Some(value.to_string()),
        any_value::Value::DoubleValue(value) => Some(value.to_string()),
        any_value::Value::BoolValue(value) => Some(value.to_string()),
        _ => None,
    }
}

#[cfg(feature = "dev-tools")]
fn any_value_kind(value: Option<&AnyValue>) -> Option<AttributeValueKind> {
    match value?.value.as_ref()? {
        any_value::Value::StringValue(_) => Some(AttributeValueKind::String),
        any_value::Value::IntValue(_) => Some(AttributeValueKind::Int),
        any_value::Value::DoubleValue(_) => Some(AttributeValueKind::Double),
        any_value::Value::BoolValue(_) => Some(AttributeValueKind::Bool),
        _ => None,
    }
}

fn projection_fixture_request() -> MetricsData {
    decode_metrics(
        HostSnapshot {
            now_unix_nano: 2_000,
            start_time_unix_nano: 1_000,
            counter_starts: CounterStarts::default(),
            memory_limit: true,
            memory_shared: true,
            memory_hugepages: true,
            cpu: Some(CpuTimes {
                user: 1.0,
                nice: 2.0,
                system: 3.0,
                idle: 4.0,
                wait: 5.0,
                interrupt: 6.0,
                steal: 7.0,
            }),
            cpu_utilization: Some(CpuTimes {
                user: 0.1,
                nice: 0.1,
                system: 0.2,
                idle: 0.3,
                wait: 0.1,
                interrupt: 0.1,
                steal: 0.1,
            }),
            cpuinfo: CpuInfo {
                logical_count: 2,
                physical_count: 1,
                frequencies_hz: vec![2_400_000_000.0],
            },
            memory: Some(MemoryStats {
                total: 100,
                used: 80,
                free: 10,
                available: 20,
                has_available: true,
                cached: 5,
                buffered: 5,
                shared: 7,
                slab_reclaimable: 3,
                slab_unreclaimable: 2,
                hugepages: HugepageStats {
                    total: 10,
                    free: 4,
                    reserved: 2,
                    surplus: 1,
                    page_size_bytes: 2 * BYTES_PER_KIB,
                },
            }),
            uptime_seconds: Some(42.0),
            paging: Some(PagingStats {
                minor_faults: 9,
                major_faults: 1,
                page_in: 4,
                page_out: 5,
                swap_in: 2,
                swap_out: 3,
            }),
            swaps: vec![SwapStats {
                name: "/dev/swap".to_owned(),
                size: 100,
                used: 25,
                free: 75,
            }],
            processes: Some(ProcessStats {
                running: 4,
                blocked: 1,
                created: 99,
            }),
            disks: vec![DiskStats {
                name: "sda".to_owned(),
                limit_bytes: Some(123),
                read_bytes: 10,
                write_bytes: 20,
                read_ops: 1,
                write_ops: 2,
                read_merged: 3,
                write_merged: 4,
                read_time_seconds: 0.5,
                write_time_seconds: 0.6,
                io_time_seconds: 0.7,
            }],
            filesystems: vec![FilesystemStats {
                device: "/dev/sda1".to_owned(),
                mountpoint: "/".to_owned(),
                fs_type: "ext4".to_owned(),
                mode: "rw",
                used: 60,
                free: 30,
                reserved: 10,
                limit_bytes: Some(100),
            }],
            networks: vec![NetworkStats {
                name: "eth0".to_owned(),
                rx_bytes: 10,
                tx_bytes: 20,
                rx_packets: 1,
                tx_packets: 2,
                rx_errors: 3,
                tx_errors: 4,
                rx_dropped: 5,
                tx_dropped: 6,
            }],
            resource: HostResource {
                host_id: Some("host-id".to_owned()),
                host_name: Some("host-name".to_owned()),
                host_arch: Some("amd64"),
            },
        }
        .into_otap_records()
        .expect("encode ok"),
    )
}

#[cfg(feature = "dev-tools")]
fn projection_fixture_metrics() -> Vec<Metric> {
    projection_fixture_request()
        .resource_metrics
        .into_iter()
        .next()
        .expect("resource metrics")
        .scope_metrics
        .into_iter()
        .next()
        .expect("scope metrics")
        .metrics
}

fn assert_metric_shape(
    metrics: &[Metric],
    name: &'static str,
    unit: &'static str,
    monotonic_sum: Option<bool>,
) {
    let metric = metric_by_name(metrics, name);
    assert_eq!(metric.unit, unit);
    match metric.data.as_ref().expect("metric data") {
        otlp_metric::Data::Sum(sum) => {
            let expected_monotonic =
                monotonic_sum.unwrap_or_else(|| panic!("{name} should be a gauge"));
            assert_eq!(
                sum.aggregation_temporality,
                AggregationTemporality::Cumulative as i32
            );
            assert_eq!(sum.is_monotonic, expected_monotonic);
            assert!(
                sum.data_points
                    .iter()
                    .all(|point| point.start_time_unix_nano == 1_000)
            );
        }
        otlp_metric::Data::Gauge(gauge) => {
            assert!(monotonic_sum.is_none(), "{name} should be a cumulative sum");
            assert!(
                gauge
                    .data_points
                    .iter()
                    .all(|point| point.start_time_unix_nano == 0)
            );
        }
        _ => panic!("unexpected data kind for {name}"),
    }
}

fn assert_first_point_attr(
    metrics: &[Metric],
    name: &'static str,
    key: &'static str,
    value: &'static str,
) {
    let metric = metric_by_name(metrics, name);
    let point = match metric.data.as_ref().expect("metric data") {
        otlp_metric::Data::Sum(sum) => sum.data_points.first(),
        otlp_metric::Data::Gauge(gauge) => gauge.data_points.first(),
        _ => None,
    }
    .expect("data point");
    assert_has_attr(&point.attributes, key, value);
}

fn assert_sum_point_attr(
    metrics: &[Metric],
    name: &'static str,
    key: &'static str,
    value: &'static str,
) {
    let metric = metric_by_name(metrics, name);
    let otlp_metric::Data::Sum(sum) = metric.data.as_ref().expect("metric data") else {
        panic!("{name} should be a cumulative sum");
    };
    assert!(
        sum.data_points
            .iter()
            .any(|point| has_attr(&point.attributes, key, value)),
        "missing point attribute {key}={value}"
    );
}

fn assert_first_point_int(metrics: &[Metric], name: &'static str, expected: i64) {
    let metric = metric_by_name(metrics, name);
    let point = match metric.data.as_ref().expect("metric data") {
        otlp_metric::Data::Sum(sum) => sum.data_points.first(),
        otlp_metric::Data::Gauge(gauge) => gauge.data_points.first(),
        _ => None,
    }
    .expect("data point");
    assert_eq!(
        point.value,
        Some(number_data_point::Value::AsInt(expected)),
        "{name} first point should be int"
    );
}

fn assert_first_point_attr_int(
    metrics: &[Metric],
    name: &'static str,
    key: &'static str,
    expected: i64,
) {
    let metric = metric_by_name(metrics, name);
    let point = match metric.data.as_ref().expect("metric data") {
        otlp_metric::Data::Sum(sum) => sum.data_points.first(),
        otlp_metric::Data::Gauge(gauge) => gauge.data_points.first(),
        _ => None,
    }
    .expect("data point");
    assert!(
        point.attributes.iter().any(|attr| {
            attr.key == key
                && matches!(
                    attr.value.as_ref().and_then(|value| value.value.as_ref()),
                    Some(any_value::Value::IntValue(actual)) if *actual == expected
                )
        }),
        "missing int attribute {key}={expected}"
    );
}

fn assert_no_first_point_attr(metrics: &[Metric], name: &'static str, key: &'static str) {
    let metric = metric_by_name(metrics, name);
    let point = match metric.data.as_ref().expect("metric data") {
        otlp_metric::Data::Sum(sum) => sum.data_points.first(),
        otlp_metric::Data::Gauge(gauge) => gauge.data_points.first(),
        _ => None,
    }
    .expect("data point");
    assert!(
        !point.attributes.iter().any(|attr| attr.key == key),
        "unexpected attribute {key}"
    );
}

fn assert_first_sum_point_start(metrics: &[Metric], name: &'static str, expected_start: u64) {
    let metric = metric_by_name(metrics, name);
    let otlp_metric::Data::Sum(sum) = metric.data.as_ref().expect("metric data") else {
        panic!("{name} should be a cumulative sum");
    };
    let point = sum.data_points.first().expect("data point");
    assert_eq!(point.start_time_unix_nano, expected_start);
}

fn metric_by_name<'a>(metrics: &'a [Metric], name: &'static str) -> &'a Metric {
    metrics
        .iter()
        .find(|metric| metric.name == name)
        .unwrap_or_else(|| panic!("missing metric {name}"))
}

fn assert_has_attr(attributes: &[KeyValue], key: &'static str, value: &'static str) {
    assert!(
        has_attr(attributes, key, value),
        "missing attribute {key}={value}"
    );
}

fn has_attr(attributes: &[KeyValue], key: &'static str, value: &'static str) -> bool {
    attributes.iter().any(|attr| {
        attr.key == key
            && matches!(
                attr.value.as_ref().and_then(|value| value.value.as_ref()),
                Some(any_value::Value::StringValue(actual)) if actual == value
            )
    })
}
