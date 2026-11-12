// TEMPORARY: compression investigation, do not commit.
#![cfg(test)]

use super::compression_ratio_tests::load_semconv_registry;
use super::semconv_signal::{semconv_otlp_logs, semconv_otlp_metrics, semconv_otlp_traces};
use otel_arrow_dfe_pdata::OtlpProtoBytes;
use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use otel_arrow_dfe_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use otel_arrow_dfe_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
use prost::Message;

const N: usize = 10;
const BATCH: usize = 1000;

/// Dumps DFE-encoded payloads and checks they decode to the same message as
/// the prost (canonical) encoding, i.e. reordering is semantically lossless.
#[test]
#[ignore]
fn dump_payloads() {
    let out = std::env::var("DUMP_DIR").expect("DUMP_DIR");
    let tag = std::env::var("DUMP_TAG").unwrap_or_else(|_| "dfe".into());
    let registry = load_semconv_registry();
    let mut totals = std::collections::BTreeMap::<&str, (usize, usize)>::new();
    for i in 0..N {
        for sig in ["logs", "traces", "metrics"] {
            // REUSE=1 reads previously dumped prost inputs so encoder variants see identical bytes.
            let prost_path = format!("{out}/{sig}_prost_{i}.pb");
            let prost_bytes: prost::bytes::Bytes = if std::env::var("REUSE").is_ok() {
                std::fs::read(&prost_path).unwrap().into()
            } else {
                let b = match sig {
                    "logs" => semconv_otlp_logs(BATCH, &registry).encode_to_vec(),
                    "traces" => semconv_otlp_traces(BATCH, &registry).encode_to_vec(),
                    _ => semconv_otlp_metrics(BATCH, &registry).encode_to_vec(),
                };
                std::fs::write(&prost_path, &b).unwrap();
                b.into()
            };
            let prost_otlp = match sig {
                "logs" => OtlpProtoBytes::ExportLogsRequest(prost_bytes.clone()),
                "traces" => OtlpProtoBytes::ExportTracesRequest(prost_bytes.clone()),
                _ => OtlpProtoBytes::ExportMetricsRequest(prost_bytes.clone()),
            };
            // Force the engine re-encode path: OTLP bytes -> OTAP records -> OTLP bytes.
            let records: OtapArrowRecords =
                otel_arrow_dfe_pdata::TryIntoWithOptions::try_into_with_default(prost_otlp)
                    .expect("to otap");
            let otlp: OtlpProtoBytes =
                otel_arrow_dfe_pdata::TryIntoWithOptions::try_into_with_default(records)
                    .expect("to otlp");
            let (dfe, canonical) = match otlp {
                OtlpProtoBytes::ExportLogsRequest(b) => {
                    let p = ExportLogsServiceRequest::decode(b.as_ref()).unwrap().encode_to_vec();
                    (b, p)
                }
                OtlpProtoBytes::ExportTracesRequest(b) => {
                    let p = ExportTraceServiceRequest::decode(b.as_ref()).unwrap().encode_to_vec();
                    (b, p)
                }
                OtlpProtoBytes::ExportMetricsRequest(b) => {
                    let p = ExportMetricsServiceRequest::decode(b.as_ref()).unwrap().encode_to_vec();
                    (b, p)
                }
            };
            
            std::fs::write(format!("{out}/{sig}_{tag}_{i}.pb"), &dfe).unwrap();
            let z = zstd::bulk::compress(dfe.as_ref(), 3).unwrap().len();
            let t = totals.entry(sig).or_default();
            t.0 += dfe.len();
            t.1 += z;
            println!("{sig} {i}: {tag}={} canonical(decode+prost)={} prost_orig={} identical_to_canonical={}", dfe.len(), canonical.len(), prost_bytes.len(), dfe.as_ref() == canonical.as_slice());
        }
    }
    for (sig, (raw, z)) in totals {
        println!("TOTAL {tag} {sig}: raw={raw} zstd3={z}");
    }
}

/// Same as dump_payloads for logs, but also runs the OTAP wire round trip
/// (Producer -> Consumer) that the end-to-end benchmark goes through, and
/// reports zstd3 for the DFE (Go order) output and the prost (forward order)
/// re-encode of it, with and without the wire round trip.
#[test]
#[ignore]
fn dump_wire_round_trip() {
    use otel_arrow_dfe_pdata::otap::{Logs, from_record_messages};
    use otel_arrow_dfe_pdata::{Consumer, Producer};
    let out = std::env::var("DUMP_DIR").expect("DUMP_DIR");
    let mut tot = [0usize; 6];
    for i in 0..N {
        let prost_bytes: prost::bytes::Bytes =
            std::fs::read(format!("{out}/logs_prost_{i}.pb")).unwrap().into();
        let to_otap = |b: prost::bytes::Bytes| -> OtapArrowRecords {
            otel_arrow_dfe_pdata::TryIntoWithOptions::try_into_with_default(
                OtlpProtoBytes::ExportLogsRequest(b),
            )
            .expect("to otap")
        };
        let to_otlp = |r: OtapArrowRecords| -> Vec<u8> {
            let o: OtlpProtoBytes =
                otel_arrow_dfe_pdata::TryIntoWithOptions::try_into_with_default(r).expect("to otlp");
            o.as_bytes().to_vec()
        };
        let z = |b: &[u8]| zstd::bulk::compress(b, 3).unwrap().len();
        let canon = |b: &[u8]| ExportLogsServiceRequest::decode(b).unwrap().encode_to_vec();

        // In-memory path (what dump_payloads measures).
        let mem = to_otlp(to_otap(prost_bytes.clone()));
        // Wire path: OTAP producer -> consumer, as loadgen exporter -> engine receiver.
        let mut recs = to_otap(prost_bytes.clone());
        let mut bar = Producer::new().produce_bar(&mut recs).expect("produce");
        let msgs = Consumer::default().consume_bar(&mut bar).expect("consume");
        let wire_recs = OtapArrowRecords::Logs(from_record_messages::<Logs>(msgs).expect("store"));
        let wire = to_otlp(wire_recs);
        let tag = std::env::var("DUMP_TAG").unwrap_or_else(|_| "dfe".into());
        std::fs::write(format!("{out}/memlogs_{tag}_{i}.pb"), &mem).unwrap();
        std::fs::write(format!("{out}/wirelogs_{tag}_{i}.pb"), &wire).unwrap();

        let v = [
            z(&prost_bytes),
            z(&mem),
            z(&canon(&mem)),
            z(&wire),
            z(&canon(&wire)),
            wire.len(),
        ];
        for (t, x) in tot.iter_mut().zip(v) {
            *t += x;
        }
    }
    println!(
        "WIRE logs zstd3: prost_orig={} mem_go={} mem_fwd={} wire_go={} wire_fwd={} wire_raw={}",
        tot[0], tot[1], tot[2], tot[3], tot[4], tot[5]
    );
}

/// Describe how the OTAP wire round trip changes the OTLP layout for batch 0.
#[test]
#[ignore]
fn diff_wire_layout() {
    use otel_arrow_dfe_pdata::otap::{Logs, from_record_messages};
    use otel_arrow_dfe_pdata::{Consumer, Producer};
    let out = std::env::var("DUMP_DIR").expect("DUMP_DIR");
    let prost_bytes: prost::bytes::Bytes =
        std::fs::read(format!("{out}/logs_prost_0.pb")).unwrap().into();
    let to_otap = |b: prost::bytes::Bytes| -> OtapArrowRecords {
        otel_arrow_dfe_pdata::TryIntoWithOptions::try_into_with_default(
            OtlpProtoBytes::ExportLogsRequest(b),
        )
        .expect("to otap")
    };
    let to_req = |r: OtapArrowRecords| -> ExportLogsServiceRequest {
        let o: OtlpProtoBytes =
            otel_arrow_dfe_pdata::TryIntoWithOptions::try_into_with_default(r).expect("to otlp");
        ExportLogsServiceRequest::decode(o.as_bytes()).unwrap()
    };
    let mem = to_req(to_otap(prost_bytes.clone()));
    let mut recs = to_otap(prost_bytes.clone());
    let mut bar = Producer::new().produce_bar(&mut recs).expect("produce");
    let msgs = Consumer::default().consume_bar(&mut bar).expect("consume");
    let wire = to_req(OtapArrowRecords::Logs(from_record_messages::<Logs>(msgs).expect("store")));
    for (name, req) in [("mem", &mem), ("wire", &wire)] {
        let rl = &req.resource_logs;
        let sl: usize = rl.iter().map(|r| r.scope_logs.len()).sum();
        println!("{name}: resource_logs={} scope_logs={}", rl.len(), sl);
        let recs: Vec<_> = rl.iter().flat_map(|r| r.scope_logs.iter()).flat_map(|s| s.log_records.iter()).collect();
        for lr in recs.iter().take(4) {
            let keys: Vec<_> = lr.attributes.iter().map(|kv| kv.key.as_str()).collect();
            let body = format!("{:?}", lr.body).chars().take(60).collect::<String>();
            println!("  t={} sev={} event={:?} keys={:?} body={}", lr.time_unix_nano, lr.severity_number, lr.event_name, keys, body);
        }
    }
}
