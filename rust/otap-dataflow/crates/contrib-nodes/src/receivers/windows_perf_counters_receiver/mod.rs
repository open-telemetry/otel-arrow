// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Windows performance-counter receiver.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = runtime::WINDOWS_PERF_COUNTERS_RECEIVER_URN,
    target = "otel.receiver.windows_perf_counters",
);

mod config;
mod metrics;
mod otap_builder;
mod pdh;
mod runtime;

#[cfg(test)]
mod test_utils {
    use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otel_arrow_dfe_pdata::{OtlpProtoBytes, TryIntoWithOptions};
    use otel_arrow_dfe_pdata_codec::OtapPayload;
    use prost::Message;

    pub(super) fn decode(records: OtapArrowRecords) -> ExportMetricsServiceRequest {
        let bytes: OtlpProtoBytes = OtapPayload::from_otap(records)
            .try_into_with_default()
            .unwrap();
        ExportMetricsServiceRequest::decode(bytes.as_bytes()).unwrap()
    }

    /// Owned child processes sharing one executable name, killed on drop.
    pub(super) struct Children(Vec<std::process::Child>, std::path::PathBuf);

    impl Drop for Children {
        fn drop(&mut self) {
            for child in &mut self.0 {
                let _ = child.kill();
                let _ = child.wait();
            }
            let _ = std::fs::remove_dir_all(&self.1);
        }
    }

    /// Starts two long-running copies of `ping.exe` named `name.exe`, and waits until PDH lists both.
    pub(super) fn spawn_children(name: &str) -> Children {
        let dir = std::env::temp_dir().join(name);
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join(format!("{name}.exe"));
        let _ = std::fs::copy(r"C:\Windows\System32\PING.EXE", &exe).unwrap();
        let mut children = Children(Vec::new(), dir);
        for _ in 0..2 {
            children.0.push(
                std::process::Command::new(&exe)
                    .args(["-n", "3600", "127.0.0.1"])
                    .stdout(std::process::Stdio::null())
                    .spawn()
                    .unwrap(),
            );
        }
        await_listed(name, &children.0);
        children
    }

    /// Polls `\Process(*)\ID Process` until both children are listed under `name`, or panics
    /// after 30 s with the last observed state.
    fn await_listed(name: &str, children: &[std::process::Child]) {
        use super::pdh::{NativePdh, Pdh};
        let mut query = NativePdh::open().unwrap();
        let counter = query.add(r"\Process(*)\ID Process").unwrap();
        let pids: Vec<f64> = children.iter().map(|child| f64::from(child.id())).collect();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let mut items = Vec::new();
        loop {
            let read = query
                .collect()
                .and_then(|()| query.array(&counter, &mut items));
            let listed: Vec<_> = items
                .iter()
                .filter(|(instance, _)| instance.split('#').next() == Some(name))
                .map(|(instance, value)| (instance.clone(), value.value))
                .collect();
            if read.is_ok() && pids.iter().all(|pid| listed.iter().any(|(_, v)| v == pid)) {
                return;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "PDH did not list both {name} children within 30 s: read {read:?}, listed \
                 {listed:?}, children {pids:?}"
            );
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
