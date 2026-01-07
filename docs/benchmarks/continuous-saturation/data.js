window.BENCHMARK_DATA = {
  "lastUpdate": 1767822707693,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3b6b188e013d94a185e56ab21d57f48a4667974b",
          "message": "Add perf test to saturate the df engine (#1612)\n\nTrying to work towards\nhttps://github.com/open-telemetry/otel-arrow/issues/1531\nThis adds a continuous tests, matching the existing back pressure ones.\nThere is only one difference - the load generator is not capped at 100K\nRPS - it goes as much as it can go.\n\n\nNeed to try tweaking few settings to get it right. The engine and\nload-generator is now running in just 1 core, we might need to run\nload_generator on more than one core to fully saturate the engine..\nAlso, in future, we want to run engine of multiple cores to see if we\ntruly scale with number of cores..\n\nAdd as continuous to begin with, once things are stabilized, we can move\nthis to nightly runs.",
          "timestamp": "2025-12-12T19:52:23Z",
          "tree_id": "a5761259a6d63ecf008583c74a872c1c4d7f95e9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3b6b188e013d94a185e56ab21d57f48a4667974b"
        },
        "date": 1765570627680,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 1001000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.50298309326172,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05009308118185862,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06937203408210689,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.05009308118185862,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.06937203408210689,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.01028645833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.1640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1006000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16766.343914546313,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.33172919754628,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001155,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10681.333568296035,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 657.114062629953,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 361000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.997342586517334,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 54.947276736753395,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 64.9796264029055,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 54.947276736753395,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.9796264029055,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.40078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.53515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9031000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 8670000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 150513.796870273,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 144497.24491919688,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001144,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2820683.117041999,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2703093.384457995,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lalit_fin@yahoo.com",
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5d1c03befc813876f6e85afb2cf7436c8cad1821",
          "message": "feat: Add mTLS client CA hot-reload with file watching (#1577)\n\n## What This Does\n\nAdds automatic hot-reload of mTLS **client CA certificates** for\nOTLP/OTAP receivers. When the CA file changes, certificates reload\nautomatically without restarting the service or dropping connections.\n\n**Important:** This PR is specifically for **client CA reload (mTLS)**.\nServer certificate reload uses a different mechanism (covered\nseparately). And it follows the pattern used in Go based otel collector.\n\n  ## Reload Mechanisms Summary\n\n| Certificate Type | Configuration | Reload Method | When Checked |\n\n|------------------|------------------------|--------------------|---------------------------------------------------|\n| Server Cert | `reload_interval` | Lazy polling | During TLS handshake\n(if interval expired) |\n| Client CA | `watch_client_ca: true`| File watching | Immediate\n(~50-500ms after file change) |\n| Client CA | `watch_client_ca: false`| Background polling | Every\n`reload_interval` (dedicated thread) |\n\n- **Client CA**: Uses `watch_client_ca` to choose between file watcher\n(notify) or interval polling\n- **Server Cert**: Always uses lazy polling on handshake, ignores\n`watch_client_ca` flag\n\n  ## Why We Need This\n\n  Modern cloud deployments rotate certificates frequently:\n  - **SPIFFE/SPIRE** issues 1-hour certificates\n  - **cert-manager** rotates daily/weekly\n  - **Zero-trust** architectures require continuous rotation\n\nWithout this, every cert rotation means restarting receivers and losing\ntelemetry data.\n\n  ## How It Works\n\n  ### Hot Path (TLS Handshakes)\nThe verification path is wait-free - just an atomic pointer load. No\nfilesystem access, no blocking.\n\n  ```rust\n  fn verify_client_cert(...) -> Result<...> {\n      let verifier = self.inner.load();  // Fast atomic load\n      verifier.verify_client_cert(...)    // Delegate\n  }\n\n```\n\n## Reload Path (File Changes)\n\n  Two reload modes are supported for client CA certificates:\n\n  ### File Watching Mode (watch_client_ca: true):\n  - Uses OS-native file notifications (inotify/kqueue)\n  - Detects changes within 50-500ms\n  - No polling overhead\n\n  ### Polling Mode (watch_client_ca: false):\n  - Checks file periodically based on reload_interval\n  - More compatible with exotic filesystems\n  - Less responsive but simpler\n\n  **When a change is detected:**\n\n  1. **Filter** - Check if it's actually our CA file\n  2. **Debounce** - Wait 1 second to avoid reload storms\n  3. **Verify** - Check file identity changed (inode)\n  4. **Reload** - Load new CA and atomic swap\n  5. **Graceful fallback** - Keep old CA if reload fails\n\n  The reload happens in the file watcher's OS thread, not the async runtime, so it doesn't block other connections.\n\n  ## Design Decision: Why Not Use Async Channels?\n\n  We could bridge the file watcher to a tokio worker task via channels, which would make all I/O async. However:\n\n  - The blocking I/O (checking file metadata) happens in the watcher's dedicated OS thread, not the tokio runtime\n  - Certificate reloads are rare (minutes/hours apart)\n  - The hot path (TLS handshakes) is already wait-free\n  - A channel adds complexity for negligible benefit here\n\n  We documented this trade-off in the code for future reference. If we need to refactor later, the channel pattern is a viable option. Also the current\n\n ## Configuration\n\n  ### File Watching Mode (Recommended for Client CA)\n\n  Immediate reload when CA file changes (50-500ms detection):\n\n```yaml\n  receivers:\n    otlp:\n      protocols:\n        grpc:\n          tls:\ncert_file: /etc/certs/server.crt # Server cert (lazy polling)\n            key_file: /etc/certs/server.key\nclient_ca_file: /etc/certs/ca.crt # Client CA (file watching)\nwatch_client_ca: true # Enable OS file watching for CA\nreload_interval: 5m # Used for server cert polling\n```\n\n  ### Polling Mode (Fallback for Client CA)\n\n  Periodic checks based on interval (better filesystem compatibility):\n\n```yaml\n  receivers:\n    otlp:\n      protocols:\n        grpc:\n          tls:\ncert_file: /etc/certs/server.crt # Server cert (lazy polling)\n            key_file: /etc/certs/server.key\n            client_ca_file: /etc/certs/ca.crt      # Client CA (polling)\n            watch_client_ca: false                  # Use polling for CA\nreload_interval: 1m # Check CA every 1 min (default: 5m)\n```\n\n  **Clarifications:**\n  - `watch_client_ca` only affects client CA reload, not server certificates\n  - `reload_interval` is used for:\n    - Server cert lazy polling (always)\n    - Client CA polling (only when `watch_client_ca: false`)\n\n\n  ## Performance Impact\n\n  \n**Hot Path (TLS Handshakes):**\n- Hot-reload overhead: ~3-5 nanoseconds (single atomic pointer load)\n- No file I/O or blocking operations during handshake\n- Lock-free - no contention between connections\n- Total verification time dominated by crypto operations, not reload mechanism\n\n**Memory:**\n- ReloadableClientCaVerifier struct: ~100 bytes\n- Inner CA store: Depends on number of CA certificates loaded\n- Shared across all connections - not duplicated per connection\n\n**Threads:**\n- +1 OS thread per receiver (notify crate's watcher thread)\n  - `watch_client_ca: true` → blocked in epoll/kqueue (near-zero CPU)\n  - `watch_client_ca: false` → polling on interval (minimal CPU)\n\n**CPU:**\n- File watcher: Event-driven (inotify/kqueue) - idles when no changes\n- Reload: ~1-10ms when CA changes (file read + PEM parse + cert validation)\n- Frequency: Typically hours/days between reloads\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2025-12-12T20:49:12Z",
          "tree_id": "a370e04a9f7c5963235197ee02f6b219783ae13b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5d1c03befc813876f6e85afb2cf7436c8cad1821"
        },
        "date": 1765574053533,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 1001000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.50298309326172,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04755504203927825,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.0747282057600493,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.04755504203927825,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.0747282057600493,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.111328125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.33203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1006000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16766.346429449866,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.33174169706692,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001146,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10859.746536542745,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 668.8743958652327,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 356000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.058830261230469,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 52.82106317074512,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 65.00135694070289,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.82106317074512,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.00135694070289,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.926171875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.03515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 8771000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 8415000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 146180.34638158893,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 140247.13428355614,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001226,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2728370.7453854093,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2622030.901048639,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "88e7730221f0576665e4771357bbbdc92be0162c",
          "message": "expose `ExporterPDataMetrics` as public (#1615)\n\nProjects may wish to define custom `Exporter` implementations while\ntaking advantage of the common set of exporter metrics defined in the\n`otap_df_otap::metrics` module. This makes the struct for accumulating\nthose exporter metrics public.",
          "timestamp": "2025-12-12T21:59:59Z",
          "tree_id": "aaa3599a88e7b6dc4a79aee87befcdd729e03827",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/88e7730221f0576665e4771357bbbdc92be0162c"
        },
        "date": 1765578282463,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 351000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.1508989334106445,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 49.93534114391213,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 50.67835724796059,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 49.93534114391213,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 50.67835724796059,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.084635416666664,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.42578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 8456000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 8105000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 140930.1905900832,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 135080.3210421741,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001338,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2584375.571877569,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2485714.4774699016,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1001000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.50298309326172,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.045954153242868195,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07057941981898352,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.045954153242868195,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.07057941981898352,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.083072916666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.24609375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1006000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16766.386668009312,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.33194168990711,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001002,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10704.10280271895,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 660.53348361334,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "l.querel@f5.com",
            "name": "Laurent Quérel",
            "username": "lquerel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e954e1f6dcce9808f3c1a44ebd49a7891c3460cf",
          "message": "Telemetry: Support f64 metrics, fix gauge accumulation, enrich pipeline-oriented monitoring (#1606)\n\nPR summary:\n- [most important/interesting change, all the rest is relatively\nmechanical] New pipeline-oriented metrics, see the comments in the\n`pipeline_metrics.rs` for more details. **It's now possible to measure\nthe cpu and memory usage per pipeline instance!**\n- Use jemalloc instead of mimalloc to capture per-thread memory metrics,\nI didn't observe any performance regression under load. When jemalloc is\nnot used the memory-oriented metrics are not reported.\n- Telemetry core now supports f64 values end‑to‑end (all metric types).\n- Registry accumulation fixed to respect instrument semantics (fixed bug\nfor Gauge).\n\nFuture work: add pipeline-oriented `network_io_received` and\n`network_io_sent` metrics.",
          "timestamp": "2025-12-12T23:27:51Z",
          "tree_id": "cbd8a420d9a54332697ff30942be32b34de7f088",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e954e1f6dcce9808f3c1a44ebd49a7891c3460cf"
        },
        "date": 1765584558295,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 446000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.604583740234375,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 61.44141125476907,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 65.28620171247194,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 61.44141125476907,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.28620171247194,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.82890625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.6015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9686000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9240000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 161430.14508796786,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 153996.95856006845,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001185,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3078883.2107977127,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2951185.892897875,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1001000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.50298309326172,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.060305042806208464,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.09226186575299797,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.060305042806208464,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.09226186575299797,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.902734375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.91015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1006000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16766.244995605026,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.33123755270888,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001509,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16627.121656831292,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 734.2691119818851,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "AaronRM@users.noreply.github.com",
            "name": "Aaron Marten",
            "username": "AaronRM"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee",
          "message": "[otap-df-quiver] WAL Refactoring (#1616)\n\n- Updated terminology from \"checkpoint\" to \"cursor\" throughout the WAL\nwriter implementation for clarity.\n- Update WAL Header and Cursor (formerly 'checkpoint') file formats to\nuse variable length headers for better forward compatibility",
          "timestamp": "2025-12-13T00:30:33Z",
          "tree_id": "73eec344dcb7019fefe48690d3790436c9d4f737",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee"
        },
        "date": 1765587327005,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 1001000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.50298309326172,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05373761855178161,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.11208545847483792,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.05373761855178161,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.11208545847483792,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.11888020833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.046875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1006000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16766.379961569324,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.33190835770041,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001026,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16527.684836103806,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722.8927444921957,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 376000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.587603569030762,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 50.191060466261796,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 51.454272959630906,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 50.191060466261796,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 51.454272959630906,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.034765625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.08203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 8196000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7820000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 136597.14967281016,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 130330.61376785938,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001252,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2541358.4662806997,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2432301.6205243235,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3336a652af1be4f6259e1e8b430e36134be23d40",
          "message": "chore(deps): update rust crate flume to 0.12.0 (#1624)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [flume](https://redirect.github.com/zesterer/flume) |\nworkspace.dependencies | minor | `0.11.1` -> `0.12.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>zesterer/flume (flume)</summary>\n\n###\n[`v0.12.0`](https://redirect.github.com/zesterer/flume/blob/HEAD/CHANGELOG.md#0120---2025-12-08)\n\n##### Added\n\n- `fastrand` feature\n\n##### Removed\n\n- `nanorand` feature\n\n##### Fixed\n\n- Panic when using `Duration::MAX` as timeout\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi40Mi4yIiwidXBkYXRlZEluVmVyIjoiNDIuNDIuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-15T16:41:10Z",
          "tree_id": "b99c61cc57503b229ee0e2d922a60bed940e86fd",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3336a652af1be4f6259e1e8b430e36134be23d40"
        },
        "date": 1765818948577,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 381000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.333977699279785,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 54.55417318901481,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 64.85693106026525,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 54.55417318901481,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.85693106026525,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.278385416666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.1015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 8791000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 8410000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 146509.77338182906,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 140160.0721352727,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002823,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2748081.4542484316,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2626822.3774140547,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1001000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.50298309326172,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05917818587130297,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07869258900422914,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.05917818587130297,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.07869258900422914,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.830989583333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.8515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1006000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16766.308426543284,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.3315528158215,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001282,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16803.484282782076,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 735.8589632490136,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "97e0a808d80282ee49b135fdef4e9ec1e9e7be42",
          "message": "columnar query engine support filtering by datetimes and additional int types (#1627)\n\npart of #1508 \n\nAdds support for filtering by integer types other than int64 and support\nfor filtering by timestamp to the columnar query engine.\n\nBefore this change, every int field we encountered we assumed was int64,\nwhich worked when filtering by attribute values. However, there are a\nhandful of fields in the OTel/OTAP data model that are not of this type\nso now we must check if we're applying a filter to one of these fields\nand, if so, create a literal for the binary expression with the correct\ninteger type.\n\nThis PR also adds support for datetime literal - for example:\n```kql\nlogs | where time_unix_nano > datetime(1970-01-01 00:00:01.1)\n```",
          "timestamp": "2025-12-15T17:12:59Z",
          "tree_id": "a8d79d372f3c88d92c7a04dda3e2254485bda66d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/97e0a808d80282ee49b135fdef4e9ec1e9e7be42"
        },
        "date": 1765820295412,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 371000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.455921173095703,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 51.398190466809915,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 65.42916487925696,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 51.398190466809915,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.42916487925696,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.895442708333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.1796875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 8326000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7955000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 138764.40018146372,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 132581.16784092528,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00098,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2581415.2382121985,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2466262.493249511,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1001000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.50298309326172,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.059392771389484914,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.0994395414407436,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.059392771389484914,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.0994395414407436,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.645572916666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.83984375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1006000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16766.33078117335,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.33166392233275,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001202,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16798.64293679258,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 734.8500195618732,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "0056fdc3c745dd9647bde074dd9d9e27b605375e",
          "message": "Try speed up benchmark prep steps (#1619)\n\n3.5 minutes spend on preparing the dedicated machine. Attempting to\noptimize this.\nIf this helps, I'll move this to a common script and re-use in all\nworkflows. (we have 4 now, but likely to add more)",
          "timestamp": "2025-12-15T17:54:37Z",
          "tree_id": "51a5fb33db6e2102c3ff64307fe06d39ffa12071",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0056fdc3c745dd9647bde074dd9d9e27b605375e"
        },
        "date": 1765822507978,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 371000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.450575828552246,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 50.8856094705236,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 64.67207446120523,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 50.8856094705236,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.67207446120523,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.071614583333336,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.765625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 8336000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7965000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 138928.24855943606,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 132745.1415278201,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002196,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2581146.2968089725,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2468882.7015583315,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1001000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.50298309326172,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04686596930437921,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07369513580437978,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.04686596930437921,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.07369513580437978,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.58203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.64453125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1006000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16766.391418407547,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.33196530023632,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000985,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16793.89930887011,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 734.0814590002054,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "102d2d694a9089a3f96a203be0532024658e0fe0",
          "message": "chore(deps): update rust crate rcgen to 0.14 (#1625)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [rcgen](https://redirect.github.com/rustls/rcgen) |\nworkspace.dependencies | minor | `0.13` -> `0.14` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>rustls/rcgen (rcgen)</summary>\n\n###\n[`v0.14.6`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.6):\n0.14.6\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.5...v0.14.6)\n\n#### What's Changed\n\n- Use private cfg for docs.rs-like builds by\n[@&#8203;ctz](https://redirect.github.com/ctz) in\n[#&#8203;384](https://redirect.github.com/rustls/rcgen/pull/384)\n- Expand rustdoc for CertificateSigningRequestParams::from\\_der by\n[@&#8203;dwhjames](https://redirect.github.com/dwhjames) in\n[#&#8203;386](https://redirect.github.com/rustls/rcgen/pull/386)\n- Group imports by\n[@&#8203;iamjpotts](https://redirect.github.com/iamjpotts) in\n[#&#8203;381](https://redirect.github.com/rustls/rcgen/pull/381)\n- examples: add signing new cert using existing ca pem files by\n[@&#8203;iamjpotts](https://redirect.github.com/iamjpotts) in\n[#&#8203;379](https://redirect.github.com/rustls/rcgen/pull/379)\n- Tweak CSR parsing errors/documentation by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;390](https://redirect.github.com/rustls/rcgen/pull/390)\n- Rename invalid CSR signature error variant by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;393](https://redirect.github.com/rustls/rcgen/pull/393)\n- chore: fix some typos in comments by\n[@&#8203;black5box](https://redirect.github.com/black5box) in\n[#&#8203;395](https://redirect.github.com/rustls/rcgen/pull/395)\n- ci: sync cargo-check-external-types nightly by\n[@&#8203;cpu](https://redirect.github.com/cpu) in\n[#&#8203;399](https://redirect.github.com/rustls/rcgen/pull/399)\n- Forward selected crypto backend to x509-parser by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;398](https://redirect.github.com/rustls/rcgen/pull/398)\n\n###\n[`v0.14.5`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.5):\n0.14.5\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.4...v0.14.5)\n\nImplement SigningKey for `&impl SigningKey` to make `Issuer` more\nbroadly useful.\n\n#### What's Changed\n\n- Forward signing and public key data through references by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;380](https://redirect.github.com/rustls/rcgen/pull/380)\n\n###\n[`v0.14.4`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.4):\n0.14.4\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.3...v0.14.4)\n\n#### What's Changed\n\n- Upgrade botan to 0.12 by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;377](https://redirect.github.com/rustls/rcgen/pull/377)\n- Upgrade x509-parser to 0.18 by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;376](https://redirect.github.com/rustls/rcgen/pull/376)\n- Add unstable support for ML-DSA algorithms by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;374](https://redirect.github.com/rustls/rcgen/pull/374)\n\n###\n[`v0.14.3`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.3):\n0.14.3\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.2...v0.14.3)\n\n#### What's Changed\n\n- docs: fix typo in `PKCS_RSA_SHA384` doc comment by\n[@&#8203;Bravo555](https://redirect.github.com/Bravo555) in\n[#&#8203;367](https://redirect.github.com/rustls/rcgen/pull/367)\n- Fix regression in key usage purpose encoding by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;369](https://redirect.github.com/rustls/rcgen/pull/369)\n\n###\n[`v0.14.2`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.2):\n0.14.2\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.1...v0.14.2)\n\n- Add a `CertifiedIssuer` type (see\n[#&#8203;363](https://redirect.github.com/rustls/rcgen/issues/363))\n\n#### What's changed\n\n- Add a CertifiedIssuer by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;363](https://redirect.github.com/rustls/rcgen/pull/363)\n- Provide a non-owning constructor for `Issuer` by\n[@&#8203;p-avital](https://redirect.github.com/p-avital) in\n[#&#8203;362](https://redirect.github.com/rustls/rcgen/pull/362)\n- Allow access to the CertifiedIssuer's Certificate by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;364](https://redirect.github.com/rustls/rcgen/pull/364)\n\n###\n[`v0.14.1`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.1):\n0.14.1\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.0...v0.14.1)\n\nDeclare 1.71 `rust-version` and check MSRV in CI.\n\n#### What's Changed\n\n- Check MSRV in CI by [@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;361](https://redirect.github.com/rustls/rcgen/pull/361)\n\n###\n[`v0.14.0`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.0):\n0.14.0\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.13.3...v0.14.0)\n\n0.14.0 contains a number of potentially breaking API changes, though\nhopefully the rate of API change should slow down after this. Here is a\nsummary of the most noticeable changes you might run into:\n\n- `signed_by()` methods now take a reference to an `&Issuer` type that\ncontains both the issuer's relevant certificate parameters and the\nsigning key (see\n[#&#8203;356](https://redirect.github.com/rustls/rcgen/issues/356)). The\n`from_ca_cert_der()` and `from_ca_cert_pem()` constructors that were\npreviously attached to `CertificateParams` are now attached to `Issuer`\ninstead, removing a number of documented caveats.\n- The `RemoteKeyPair` trait is now called `SigningKey` and instead of\n`KeyPair` being an enum that contains a `Remote` variant, that variant\nhas been removed in favor of `KeyPair` implementing the trait (see\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/issues/328)). To\nalign with this change, the `CertifiedKey::key_pair` field is now called\n`signing_key`, and `CertifiedKey` is generic over the signing key type.\n- The `KeyPair::public_key_der()` method has moved to\n`PublicKeyData::subject_public_key_info()` (see\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/issues/328)).\n- Output types like `Certificate` no longer contain their originating\n`CertificateParams`. Instead, `signed_by()` and `self_signed()` now take\n`&self`, allowing the caller to retain access to the input parameters\n(see\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/issues/328)). In\norder to make this possible, `Certificate::key_identifier()` can now be\naccessed via `CertificateParams` directly.\n- String types have been moved into a module (see\n[#&#8203;329](https://redirect.github.com/rustls/rcgen/issues/329)).\n\n#### What's Changed\n\n- Revert impl AsRef issuer by\n[@&#8203;audunhalland](https://redirect.github.com/audunhalland) in\n[#&#8203;325](https://redirect.github.com/rustls/rcgen/pull/325)\n- Move string types to separate module by\n[@&#8203;est31](https://redirect.github.com/est31) in\n[#&#8203;329](https://redirect.github.com/rustls/rcgen/pull/329)\n- Unbundle params from output types by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/pull/328)\n- Deduplicate Issuer construction by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;332](https://redirect.github.com/rustls/rcgen/pull/332)\n- Extract write\\_extensions() method, reducing rightward drift by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;333](https://redirect.github.com/rustls/rcgen/pull/333)\n- Update 0.12-to-0.13.md by\n[@&#8203;Alirexaa](https://redirect.github.com/Alirexaa) in\n[#&#8203;338](https://redirect.github.com/rustls/rcgen/pull/338)\n- Distribute methods for parsing params elements from x509 by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;336](https://redirect.github.com/rustls/rcgen/pull/336)\n- Eagerly derive Clone, Copy, where possible by\n[@&#8203;lvkv](https://redirect.github.com/lvkv) in\n[#&#8203;341](https://redirect.github.com/rustls/rcgen/pull/341)\n- Updated `.gitignore` to be more specific by\n[@&#8203;Rynibami](https://redirect.github.com/Rynibami) in\n[#&#8203;342](https://redirect.github.com/rustls/rcgen/pull/342)\n- Eagerly implemented `Debug` trait by\n[@&#8203;Rynibami](https://redirect.github.com/Rynibami) in\n[#&#8203;343](https://redirect.github.com/rustls/rcgen/pull/343)\n- Minor tweaks to Debug impls and other style improvements by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;348](https://redirect.github.com/rustls/rcgen/pull/348)\n- tests: only test against openssl on Unix by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;350](https://redirect.github.com/rustls/rcgen/pull/350)\n- Eagerly implemented `PartialEq` and `Eq` traits by\n[@&#8203;Rynibami](https://redirect.github.com/Rynibami) in\n[#&#8203;344](https://redirect.github.com/rustls/rcgen/pull/344)\n- Use Issuer directly in the public API by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;356](https://redirect.github.com/rustls/rcgen/pull/356)\n- Tweak docstring for PublicKeyData::subject\\_public\\_key\\_info() by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;358](https://redirect.github.com/rustls/rcgen/pull/358)\n\n###\n[`v0.13.3`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.13.3):\n0.13.3\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.13.2...v0.13.3)\n\nThis release was yanked due to\n[#&#8203;324](https://redirect.github.com/rustls/rcgen/issues/324)\n\n#### What's Changed\n\n- Update dependencies by [@&#8203;djc](https://redirect.github.com/djc)\nin [#&#8203;305](https://redirect.github.com/rustls/rcgen/pull/305)\n- Add link to GitHub releases by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;304](https://redirect.github.com/rustls/rcgen/pull/304)\n- change signature of signed\\_by to accept \\&impl\nAsRef<CertificateParams> issuer by\n[@&#8203;audunhalland](https://redirect.github.com/audunhalland) in\n[#&#8203;307](https://redirect.github.com/rustls/rcgen/pull/307)\n- Clarify CertificateParams::signed\\_by() docs by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;308](https://redirect.github.com/rustls/rcgen/pull/308)\n- refactor: Generalize csr/crl signed\\_by to take \\&impl AsRef issuer by\n[@&#8203;audunhalland](https://redirect.github.com/audunhalland) in\n[#&#8203;312](https://redirect.github.com/rustls/rcgen/pull/312)\n- Fix: mark SAN as critical when subject is empty by\n[@&#8203;howardjohn](https://redirect.github.com/howardjohn) in\n[#&#8203;311](https://redirect.github.com/rustls/rcgen/pull/311)\n- Elide private key in KeyPair Debug impl by\n[@&#8203;lvkv](https://redirect.github.com/lvkv) in\n[#&#8203;314](https://redirect.github.com/rustls/rcgen/pull/314)\n- derive Debug for non-sensitive struct types by\n[@&#8203;cpu](https://redirect.github.com/cpu) in\n[#&#8203;316](https://redirect.github.com/rustls/rcgen/pull/316)\n- update LICENSE by\n[@&#8203;jasmyhigh](https://redirect.github.com/jasmyhigh) in\n[#&#8203;318](https://redirect.github.com/rustls/rcgen/pull/318)\n- Make `Certificate` cloneable (derive `Clone`) by\n[@&#8203;MadLittleMods](https://redirect.github.com/MadLittleMods) in\n[#&#8203;319](https://redirect.github.com/rustls/rcgen/pull/319)\n- Update dependencies by [@&#8203;djc](https://redirect.github.com/djc)\nin [#&#8203;321](https://redirect.github.com/rustls/rcgen/pull/321)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi40Mi4yIiwidXBkYXRlZEluVmVyIjoiNDIuNDIuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-15T18:32:00Z",
          "tree_id": "ee7b422f2fad980d5e5c4a5638c17ab0563d5220",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/102d2d694a9089a3f96a203be0532024658e0fe0"
        },
        "date": 1765824837909,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 1001000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.50298309326172,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05887996954806016,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.10470789075759912,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.05887996954806016,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.10470789075759912,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.609375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.6875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1006000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16766.274335847207,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.33138337896226,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001404,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16538.4040168244,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 725.7135565158992,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 371000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.828294277191162,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 61.15459126792053,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 64.94627185955186,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 61.15459126792053,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.94627185955186,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.34596354166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.671875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9691000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9320000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 161512.68268716038,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 155329.50187228716,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00148,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3074416.8943924094,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2942430.280351235,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "76450334+andborja@users.noreply.github.com",
            "name": "Andres Borja",
            "username": "andborja"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "70e2d4934f8e0acf880b5429dcfe98433323d14d",
          "message": "feat: Declarative configuration for logs and Console exporter (#1630)\n\nDeclarative configuration for logs and Console exporter.",
          "timestamp": "2025-12-16T15:39:48Z",
          "tree_id": "8443dc498d3c266c7f705e4c57bf45f39494395e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/70e2d4934f8e0acf880b5429dcfe98433323d14d"
        },
        "date": 1765900970448,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 1001000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.50298309326172,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05383520569941176,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.08224101555847976,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.05383520569941176,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.08224101555847976,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.557552083333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.70703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1006000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16766.303396759737,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.3315278168973,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0013,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16801.638148009297,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 734.0863681961725,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 371000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.2589826583862305,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 53.77815426135254,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 64.891366265284,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.77815426135254,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.891366265284,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.059375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.328125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 8711000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 8340000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 145180.3377790305,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 138997.13202584253,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001238,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2719648.6851215926,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2601137.5100851255,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "76450334+andborja@users.noreply.github.com",
            "name": "Andres Borja",
            "username": "andborja"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6dbec074678d2265b953223691466e41ffa0ce4b",
          "message": "feat: Add declarative configuration for otlp logs exporter (#1635)\n\nAdd declarative configuration for otlp logs exporter\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2025-12-16T23:11:15Z",
          "tree_id": "7be994590179b95a7b9329eef5c46ba06d2f5c6e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6dbec074678d2265b953223691466e41ffa0ce4b"
        },
        "date": 1765928998361,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 47000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.44315582513809204,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.72321560784508,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.90293688613478,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.72321560784508,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.90293688613478,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.864322916666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.55078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 10605750,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 10558750,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 176759.5127642343,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 175976.19266901058,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001014,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4592208.8573152665,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4395046.919079983,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "865730028aeacfbe679d94d531d15cb2257aeb44",
          "message": "Adds OTAP Dataflow processor that uses columnar query engine (#1638)\n\ncloses #1628 \n\nAdds a processor implementation to OTAP Dataflow that uses the columnar\nquery engine to transform telemetry data.\n\nExample config:\n```yaml\nnodes:\n  transform:\n    kind: processor\n    plugin_urn: urn:otel:transform:processor\n    out_ports:\n      out_port:\n        destinations:\n        - exporter\n        dispatch_strategy: round_robin\n    config:\n      query: logs | where event_name == \"gen_ai.system.message\"\n```\n\n**I'm flexible on the name of this plugin and open to suggestions**. I\nhesitated between some alternative names:\n- \"\\<_language_\\> processor\" - where \"_language_\" = KQL/OTTL/ anything\nthat can be transformed into our expression AST. I was thinking this\ncould eventually accept different ways to express the transform so\npinning it to one language seemed too specific.\n- \"transform processor\" - this is a good name that expresses the purpose\nof the processor, but it's not really API compatible with Go collector's\n[transform\nprocessor](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/processor/transformprocessor)\nand I didn't want to cause confusion.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2025-12-17T00:44:50Z",
          "tree_id": "b37a55ffc0a360721985d05a9d975e8c6a27adab",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/865730028aeacfbe679d94d531d15cb2257aeb44"
        },
        "date": 1765935669395,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 112000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.048370122909546,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.75645488446861,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.90458218779051,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.75645488446861,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.90458218779051,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.57981770833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 10683250,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 10571250,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 178051.4454470754,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 176184.80730886164,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000917,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4593592.822512953,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4397350.74614913,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "geukhanuslu@gmail.com",
            "name": "Gokhan Uslu",
            "username": "gouslu"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "05cf0061fdc89f8b4ea52ba600ca8f80b80a4116",
          "message": "use logs view for better performance (#1631)",
          "timestamp": "2025-12-17T18:16:05Z",
          "tree_id": "af8dff66652159f7c01e1ce63ba7c1639f70ab50",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/05cf0061fdc89f8b4ea52ba600ca8f80b80a4116"
        },
        "date": 1765997180445,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 525600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 6.40350866317749,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.28170647263808,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.39505906119287,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.28170647263808,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.39505906119287,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.62721354166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.6328125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 8208000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7682400,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 136797.47380665038,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 128037.63557166311,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001108,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3815101.0453808624,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2063051.1061781263,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 83520,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.7874550223350525,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.73197731498593,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.85533863369396,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.73197731498593,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.85533863369396,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.446614583333336,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.22265625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 10606320,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 10522800,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 176768.58542015828,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 175376.6123084389,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001159,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4579428.783839543,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4383792.269080309,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "225df9867d5a4e6b09a877a66023bc7af2b69397",
          "message": "PipelinePerfTest - reduce batch size for Saturation test (#1647)\n\nhttps://open-telemetry.github.io/otel-arrow/benchmarks/continuous-saturation/\nWe are still losing small % of logs, so trying to reduce the batch size\nbit more.",
          "timestamp": "2025-12-17T22:18:07Z",
          "tree_id": "87edca11b7b03c0dee735d27fc985f62edc4ea46",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/225df9867d5a4e6b09a877a66023bc7af2b69397"
        },
        "date": 1766011102821,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 404400,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.8425798416137695,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 91.3552763863365,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.42539907782763,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 91.3552763863365,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.42539907782763,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.670833333333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.5390625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6921600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6517200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 115357.01225338264,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108617.1868148615,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001554,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3470048.599895534,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2001187.195541589,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.66658089810072,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.88443141361256,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.66658089810072,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.88443141361256,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.11927083333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9853600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9853600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 164223.74622104637,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 164223.74622104637,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001067,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4600794.82965031,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4408126.990896199,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "AaronRM@users.noreply.github.com",
            "name": "Aaron Marten",
            "username": "AaronRM"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "ac99143900169732ae2a7c90ed7a61cc4bd37471",
          "message": "[otap-df-quiver] Segment file reader/writer for Quiver durable storage layer. (#1643)\n\nImplements Segment file reader/writer for Quiver durable storage layer",
          "timestamp": "2025-12-17T23:42:12Z",
          "tree_id": "e6cb549d81b18cbeef6cb72bdd3b86560acd7f72",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac99143900169732ae2a7c90ed7a61cc4bd37471"
        },
        "date": 1766016571553,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 371800,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.122481822967529,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.14794446425924,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.25685842256586,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.14794446425924,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.25685842256586,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.834375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.7578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 7258200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6886400,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 120967.03832367838,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 114770.52336835286,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001469,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3643207.341711209,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2110656.9809521334,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 236600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.461352825164795,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.59222227386213,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.73662707472425,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.59222227386213,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.73662707472425,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.09231770833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.61328125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9612600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9376000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 160207.4233306081,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 156264.1534181992,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000965,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4564614.335472325,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4372940.285068589,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "l.querel@f5.com",
            "name": "Laurent Quérel",
            "username": "lquerel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "17d4378b7e7e992edf4f77f8436f17c8958e0c82",
          "message": "Pipeline metrics - Follow up of #1606 (#1632)\n\nSo far, we had hot-path support for **Counter**, **UpDownCounter**, and\n**Gauge** metrics. These metrics are defined using macros that specify\nthe name, description, and unit, which lets us automatically generate a\nsemantic convention registry for our engine and for all pipeline\ncomponents (receiver, processor, exporter).\n\nSome of these metrics are fundamentally **cumulative counters** or\n**cumulative up-down counters** (for example, memory usage). In this PR,\nI therefore split our `Counter` and `UpDownCounter` types into:\n\n* **DeltaCounter / DeltaUpDownCounter**\n* **ObserveCounter / ObserveUpDownCounter**\n\nWith the following API:\n\n* delta counters and delta up-down counters expose an `add` method\n* observe counters and observe up-down counters expose an `observe`\nmethod\n* gauges expose a `set` method\n\nThis, I believe, aligns with the OTel intrumentation API model.\n\nNow I'm hitting a problem: I haven't found a way to automatically\nrepresent these **cumulative** metrics without going through an SDK\nexporter configuration file. I put a temporary workaround in\n`dispatcher.rs`, but I’d like a better and more accurate solution to\nconnect the Rust Client SDK with these internal metrics.",
          "timestamp": "2025-12-18T18:39:21Z",
          "tree_id": "5208a8ea7b4b724b81e8a1c21a4ff1501ac0a0ee",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/17d4378b7e7e992edf4f77f8436f17c8958e0c82"
        },
        "date": 1766084606186,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 15600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.16298216581344604,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.70542332042436,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.80942269936757,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.70542332042436,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.80942269936757,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.743359375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.51171875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9571600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9556000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 159523.80853176382,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 159263.81319001367,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001075,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4558655.386781808,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4368141.348046745,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 429600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.955748081207275,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.19686998557307,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.39588919647004,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.19686998557307,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.39588919647004,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.331119791666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.671875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 7213200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6783600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 120217.4874545122,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 113057.6370953847,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001254,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3631618.044246772,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2102261.9726730553,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "AaronRM@users.noreply.github.com",
            "name": "Aaron Marten",
            "username": "AaronRM"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "97ea2748e5b570547b180273cc06afaa4d09f72e",
          "message": "[otap-quiver-df] Enhance safety of memory mapping in SegmentReader by using copy-on-write mapping (#1649)\n\nEnhance safety of memory mapping in SegmentReader by using copy-on-write\nmapping. Follow up to [previous PR\ndiscussion](https://github.com/open-telemetry/otel-arrow/pull/1643#discussion_r2629027444).",
          "timestamp": "2025-12-18T18:53:57Z",
          "tree_id": "ccb19cc7846ca6102faf9c758a6c10c6e093f457",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/97ea2748e5b570547b180273cc06afaa4d09f72e"
        },
        "date": 1766085488103,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 419800,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.742503643035889,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.30865290561827,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.47367739534883,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.30865290561827,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.47367739534883,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.166927083333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.5625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 7310400,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6890600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 121837.48811711998,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 114840.9656954239,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001237,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3691616.052506627,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2134043.1898230426,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 11600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.11934646964073181,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.72423437165179,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.87435869885879,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.72423437165179,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.87435869885879,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.20494791666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.59765625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9719600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9708000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 161990.49310002098,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 161797.163156406,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001052,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4556099.43007094,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4365464.4442736935,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4c67cc6a9e1e5d72cbc2dc409b17db6bf4c6ce06",
          "message": "Support renaming attributes in columnar query engine (#1648)\n\npart of #1639\n\nAdd support for renaming attributes in columnar query engine. This is\ndone by using the `otap_df_pdata::otap::transform::transform_attributes`\nfunction to service some variants of the `Move` and `RenameMapKeys`\n`TranformExpression` from our pipeline AST.\n\nFor example, we now support queries like this to rename attributes:\n```kql\nlogs | project-rename attributes[\"x\"] = attributes[\"y\"], attributes[\"a\"] = attributes[\"b\"]\n```\n\n---------\n\nCo-authored-by: Cijo Thomas <cithomas@microsoft.com>",
          "timestamp": "2025-12-18T19:33:12Z",
          "tree_id": "a66dd64873e5ff7e3f58bd207c58978ebd4b6ba0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4c67cc6a9e1e5d72cbc2dc409b17db6bf4c6ce06"
        },
        "date": 1766087905986,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 10600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.10868896543979645,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.67171857582969,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.79052669347355,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.67171857582969,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.79052669347355,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.73580729166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.29296875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9752600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9742000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 162540.22068810716,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 162363.55740454237,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001149,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4575510.489927904,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4384117.288298649,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 316200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.191521644592285,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.23177639815253,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.36232459371614,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.23177639815253,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.36232459371614,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.557291666666664,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.734375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 7543800,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7227600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 125727.6761334528,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 120457.77353881909,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001109,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3755806.9234341825,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2174387.9115429386,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "beb5ea01e7da7c95a5719ca5cf9760001430520f",
          "message": "Ignore known failed test in windows (#1657)\n\nTrying to get the CI to green.\nhttps://github.com/open-telemetry/otel-arrow/issues/1614 is already\nopened to track actually addressing this.",
          "timestamp": "2025-12-18T22:02:53Z",
          "tree_id": "0f6e8e22e29c41c7a5f412b774213e22833990a9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/beb5ea01e7da7c95a5719ca5cf9760001430520f"
        },
        "date": 1766098559367,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 377000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.25142765045166,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.16682029659565,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.28622215613383,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.16682029659565,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.28622215613383,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.60950520833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.39453125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 7179000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6802000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 119647.40963358142,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 113364.21233146968,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001299,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3641617.8019456123,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2107707.153515728,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 15600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.16394132375717163,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.62608148576332,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.8025825555038,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.62608148576332,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.8025825555038,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.18919270833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.6015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9515600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9500000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 158590.34390535072,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 158330.34880625832,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001131,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4558764.304356112,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4368805.033608324,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "816a562af2c08ed862088664f62555d582995011",
          "message": "Fix perf result uploading (#1659)\n\nhttps://github.com/open-telemetry/otel-arrow/pull/1651 didn't work well,\nas the GH Action is adding the commit info late, and we don't have a way\nto interfere easily. Will check for some other option, but until then,\nthis PR must be merged to get back broken perf uploading!",
          "timestamp": "2025-12-19T00:19:50Z",
          "tree_id": "69708dea0b9e114188b49bcb4e0931a332770fe7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/816a562af2c08ed862088664f62555d582995011"
        },
        "date": 1766104845717,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 8600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.08977410942316055,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.6522901728889,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.82484940229173,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.6522901728889,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.82484940229173,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.375390625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.828125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9579600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9571000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 159656.98248303108,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 159513.65185864654,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001134,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4564075.538695022,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4375656.825896565,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 531200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 7.264571666717529,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.23172673927635,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.42421524851662,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.23172673927635,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.42421524851662,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.53333333333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.05078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 7312200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6781000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 121867.94449400286,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 113014.76048437315,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001012,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3627309.9282957255,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2100013.8616251694,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "890710c0f3e93527e251c373954a1b9a4a649f12",
          "message": "Add CI check for macos (arm) and linux(arm) (#1660)\n\nFix https://github.com/open-telemetry/otel-arrow/issues/1578",
          "timestamp": "2025-12-19T00:34:13Z",
          "tree_id": "7ed21f3f8c6f3f552ec8bd095fc83fdad71af189",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/890710c0f3e93527e251c373954a1b9a4a649f12"
        },
        "date": 1766106090472,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 357000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.680616855621338,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.28053179972628,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.45157081055973,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.28053179972628,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.45157081055973,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.59361979166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.56640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 7627200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7270200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 127117.69493246522,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 121167.80282384212,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001088,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3805871.3628528,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2203891.6818979755,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 12600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.13072438538074493,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.6608610831,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.87486475011605,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.6608610831,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.87486475011605,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.335546875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.75,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9638600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9626000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 160639.07104331497,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 160429.07661516714,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001592,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4589499.177900847,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4398216.410082782,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lalit_fin@yahoo.com",
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "a7819cb53c5bc9498b599e187aabecd9f5ce9179",
          "message": "feat:  Add TLS/mTLS support for OTLP and OTAP exporters (#1626)\n\nAdds client-side TLS and mTLS support for OTLP and OTAP exporters,\nenabling secure communication with downstream collectors.\n\n## Changes:\n\n- Extended existing `experimental-tls` feature flag to support exporters\n(previously only covered receivers)\n- Implemented `build_endpoint_with_tls()` in `GrpcClientSettings` to\nhandle client-side TLS configuration\n- “Added `load_client_tls_config()` in `tls_utils.rs` to construct\nclient TLS/mTLS config (trust roots + client identity) with Go-aligned\n`insecure` semantics; `insecure_skip_verify=true` is rejected\n(fail-fast) as of now.”\n  - Updated both OTLP and OTAP exporters to build TLS-enabled endpoints\n\n ## Configuration\n\nTLS is scheme-driven by default: `https://` enables TLS (system roots),\n`http://` is plaintext when `tls` is omitted. The optional `tls` block\nadds trust roots and client identity (mTLS). Note: `insecure: true` with\nno custom CA results in no explicit TLS config; the endpoint scheme\nstill decides TLS vs plaintext.\n\n```yaml\n  exporter:\n    config:\n      grpc_endpoint: \"https://backend:4317\"\n      tls:\n        ca_file: \"/path/to/ca.crt\"              # Server trust\n        cert_file: \"/path/to/client.crt\"       # Client identity (mTLS)\n        key_file: \"/path/to/client.key\"\n        include_system_ca_certs_pool: true     # Default: true\n```\n\n ## Testing\n\n  - Added integration tests in `crates/otap/tests/otlp_exporter_tls.rs`\n\n## Known Limitations\n\nCertificate hot reload is not included in this PR. This applies to:\n\n- Client Identity: Client certificate and key (mTLS).\n- Trust Anchors: Custom CA certificates (ca_file) and System CA\ncertificates.\n\nIf any of these files change (e.g., certificate rotation or updating the\nCA bundle), the process must be restarted to pick up the changes.\n\nWhile the Go OpenTelemetry Collector's exporters support hot reload for\nclient certificates via periodic polling, implementing this for our Rust\nexporters is complex:\n- Requires either recreating the gRPC channel (may disrupt in-flight\nrequests) or implementing a custom TLS connector with lazy certificate\nloading.\n- Unlike receivers which use LazyReloadableCertResolver, exporters would\nneed significant integration work with tonic's transport layer.\n\nSee detailed explanation in `crates/otap/src/tls_utils.rs:115-130`. Will\ncreate a follow-up issue to track hot reload support, and can be\nimplemented if this becomes an operational requirement.\n\n## Client-side TLS config (OTLP exporter) — Go vs Rust\n\n| Field | Config key | Go behavior | Rust behavior | Notes |\n|-------|------------|-------------|---------------|-------|\n| Plaintext vs TLS | (scheme-driven) | `https://` → TLS, `http://` →\nplaintext | Same | gRPC dialing; scheme decides |\n| `insecure` | `insecure` | If true + no CA, TLS config is nil; scheme\nstill decides | Same | Go's intended semantics — \"insecure\" isn't a\nguaranteed plaintext switch |\n| `insecure` + custom CA | `insecure` + `ca_file`/`ca_pem` | TLS still\nbuilt when CA exists | Same | Surprises people |\n| Skip cert verification | `insecure_skip_verify` | Supported | Not\nsupported (fails fast) | **Major difference** |\n| SNI override | `server_name_override` | Sets TLS ServerName | Same\n(stored as `server_name` internally) | Same key |\n| Custom root CAs | `ca_file`, `ca_pem` | Adds trust roots | Same | — |\n| Client cert (mTLS) | `cert_file`, `key_file` | Presents client cert |\nSame | — |\n| No TLS block | — | Scheme-driven | Same | gRPC dialing; `https://` →\nTLS, `http://` → plaintext |\n\n**TL;DR:** Behavior matches except `insecure_skip_verify` — Go supports\nit, Rust currently fails fast if set.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2025-12-19T00:50:29Z",
          "tree_id": "74a0e5093a1b4656c1fa7a2b30edf3bb7d07dafd",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a7819cb53c5bc9498b599e187aabecd9f5ce9179"
        },
        "date": 1766107001405,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 21600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.22204861044883728,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.60525449753081,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.74660927459936,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.60525449753081,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.74660927459936,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.889583333333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.51171875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9727600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9706000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 162123.80247948953,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 161763.80883937716,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00106,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4566997.747218725,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4376375.007107353,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 351600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.634243011474609,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.34674680781588,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.51972891782945,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.34674680781588,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.51972891782945,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.532421875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.91015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 7587000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7235400,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 126443.91804754193,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 120584.19989998481,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002886,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3774527.5145520503,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2184717.656739004,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "fe53f5c91e2ec0a005f31629c46bc91ee90f613e",
          "message": "[PipelinePerfTest] - Collect logs from the engine (#1656)\n\nAdds automatic collection and display of Docker container logs during\nperf test.\nLogs are collected and printed at the end, separated by each container\nto avoid interfering with test log itself.\n\nThe previous behavior of printing logs in \"--debug\" mode during\ncontainer shutdown is retained. (I'll send a follow up to remove --debug\noption from the CI runs to avoid too much noise)\n\nAlso, enabled engine configs in perf test to do INFO level logs. (as of\ntoday, engine logs are OFF by default)\n\nExample from local run:\n\n```text\n================================================================================\nComponent: df-engine\nContainer ID: dd2985efabfe53324ffb410f7bbd18a6584ccb804701464e9ff47543ced149e8\n================================================================================\nStarting pipeline on core ID set 1-2\n2025-12-18T20:59:42.948630Z  INFO pipeline-core-1 pipeline_thread{core.id=1}: otap-df-otap:  name=\"Exporter.Start\" grpc_endpoint=\"http://backend-service:1235\" Starting OTLP Exporter\n2025-12-18T20:59:42.948629Z  INFO pipeline-core-2 pipeline_thread{core.id=2}: otap-df-otap:  name=\"Exporter.Start\" grpc_endpoint=\"http://backend-service:1235\" Starting OTLP Exporter\n2025-12-18T20:59:42.948744Z  INFO pipeline-core-2 pipeline_thread{core.id=2}: otap-df-otap:  name=\"Receiver.Start\" Starting OTLP Receiver\n2025-12-18T20:59:42.948744Z  INFO pipeline-core-1 pipeline_thread{core.id=1}: otap-df-otap:  name=\"Receiver.Start\" Starting OTLP Receiver\n\n================================================================================\n```\n\n\nWe don't really have much logs in the engine, but we are adding them,\nand they are badly needed to investigate data loss issues like\nhttps://github.com/open-telemetry/otel-arrow/issues/1637",
          "timestamp": "2025-12-19T00:58:32Z",
          "tree_id": "3de14daca034baa8f2b76a00bec1ace2359ed98b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fe53f5c91e2ec0a005f31629c46bc91ee90f613e"
        },
        "date": 1766107529637,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 12600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.1302110254764557,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.67876148969576,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.86108101422387,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.67876148969576,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.86108101422387,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.228515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.6484375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9676600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9664000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 161273.64547370814,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 161063.64940763442,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001124,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4570081.706515179,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4377743.520188732,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 472400,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 6.298330307006836,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.30809073467617,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.46370745387453,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.30809073467617,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.46370745387453,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.24622395833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.70703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 7500400,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7028000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 125003.70407888001,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 117130.5573391244,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001422,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3730202.436460172,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2156509.8074536645,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "l.querel@f5.com",
            "name": "Laurent Quérel",
            "username": "lquerel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "5380f8cd34f24e4d5daf11d156761ef6b793d1ca",
          "message": "Tokio metrics (#1661)\n\nThis PR adds Tokio-related multivariate metrics. Some of these metrics\nare only reported when the DF engine is compiled with `RUSTFLAGS=\"--cfg\ntokio_unstable\"`.\n\nIt is also now possible to enable or disable pipeline-related metrics\nper pipeline configuration. By default, all of these metrics are\nenabled.",
          "timestamp": "2025-12-19T01:19:57Z",
          "tree_id": "c1b0d0a01fbd3c6da4c66dacd2cbbef074ee5b6f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5380f8cd34f24e4d5daf11d156761ef6b793d1ca"
        },
        "date": 1766109148843,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 415000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.6277289390563965,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.19853866706335,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.30436585539007,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.19853866706335,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.30436585539007,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.45078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.5703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 7374200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6959200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 122900.95929646959,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 115984.42623416657,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001159,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3725253.0680348626,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2153129.179516317,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.6520231795028,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.78546019513706,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.6520231795028,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.78546019513706,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.40807291666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.78125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9593600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9593600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 159890.10355324156,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 159890.10355324156,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001212,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4567403.030711949,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4374156.075618883,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "2c5e35748dc82b3cee511b767e9ca66ee66df7ff",
          "message": "Support deleting attributes in columnar query engine (#1654)\n\ncloses #1639 \n\nAdds the ability to delete attributes to the columnar query engine.\n\nWe now support queries like:\n```kql\nlogs | project-away attributes[\"x\"], attributes[\"y\"]\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-12-19T03:20:40Z",
          "tree_id": "98129481f316ac1e088724154180fcfb2603f905",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2c5e35748dc82b3cee511b767e9ca66ee66df7ff"
        },
        "date": 1766115874231,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 98600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.0248841047286987,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.60253020518658,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.7333653787234,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.60253020518658,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.7333653787234,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.594791666666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.2421875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9620600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9522000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 160339.68827775316,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 158696.39230201498,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001364,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4516934.371079972,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4325050.129062723,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 375000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.056633949279785,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.26765094608027,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.38359161999072,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.26765094608027,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.38359161999072,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.52877604166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.76171875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 7416000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7041000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 123597.26026073088,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 117347.3987993266,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00133,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3743850.812859274,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2164156.222048318,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lalit_fin@yahoo.com",
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3ce308b4e37971040b2966959db4f2fdfc1a4310",
          "message": "[Geneva Exporter] Implement concurrent uploads and add metrics (#1653)\n\nRefactor GenevaExporter to support concurrent batch uploads, to improve\nthroughput for high-volume telemetry - allowing multiple in-flight\nupload requests per exporter instance (controlled by\nmax_concurrent_uploads).\n\nIt also introduces internal metrics to track the health and performance\nof the export process and ensures proper Ack/Nack propagation. Also\nadded tests to ensure that the exporter correctly parses the user's YAML\nconfiguration, ensuring all fields and authentication methods (like\ncertificates or managed identities) are deserialized properly.",
          "timestamp": "2025-12-19T16:58:21Z",
          "tree_id": "147ad677e7e50343a3006a8ccee37997e194b58a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3ce308b4e37971040b2966959db4f2fdfc1a4310"
        },
        "date": 1766164984698,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 408600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.871364593505859,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 90.17298909276509,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.5665547547082,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.17298909276509,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.5665547547082,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.9015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.05859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6959200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6550600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 115983.48871907577,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 109173.67530796323,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001644,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3479397.995787409,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2004331.467878761,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.68128942966837,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.8300987784727,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.68128942966837,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.8300987784727,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.17578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.61328125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9663600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9663600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 161056.81912782224,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 161056.81912782224,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001185,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4606653.715777088,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4411109.73307477,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "61b11bbbede0abc1775496a64359ea1a12e321bf",
          "message": "Another windows and macOS test to ignore (#1665)\n\nHopefully, this'd make it back to green!",
          "timestamp": "2025-12-19T20:08:57Z",
          "tree_id": "41841d1978267604964c06abbc95e660f25cef1d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/61b11bbbede0abc1775496a64359ea1a12e321bf"
        },
        "date": 1766176385305,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 240600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.4998440742492676,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.58355642449733,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.7709605577072,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.58355642449733,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.7709605577072,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.24752604166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.7734375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9624600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9384000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 160403.11335966643,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 156393.2855149419,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002576,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4574632.903717753,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4380109.901526833,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 429400,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.761747360229492,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.29489176539722,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.48581665036106,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.29489176539722,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.48581665036106,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.17044270833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.5625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 7452600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7023200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 124207.637984751,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 117051.10741144074,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001141,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3698096.822460484,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2136011.9329081825,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1efee73c2ef92e2d7c2957928945dd5ece06820d",
          "message": "Add memory info to SystemInformation log (#1666)",
          "timestamp": "2025-12-20T00:00:55Z",
          "tree_id": "d794e99c2bed155a7108e4a58621a6dc2c9ec111",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1efee73c2ef92e2d7c2957928945dd5ece06820d"
        },
        "date": 1766190918528,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.63054377057439,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.73857469894476,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.63054377057439,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.73857469894476,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.495703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.734375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9766600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9766600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 162774.20606325168,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 162774.20606325168,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000907,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4572373.903059193,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4377717.198279814,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 420000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.628819465637207,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.25785059817423,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.37080790827393,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.25785059817423,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.37080790827393,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.052083333333336,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.796875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 7461600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7041600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 124356.86827953382,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 117357.04455842786,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001511,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3701567.1124706217,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2138623.909999909,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4396b28f6e9ca9920fefd4cf9f0862c63f8ef136",
          "message": "fix(deps): update module google.golang.org/protobuf to v1.36.11 (#1622)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[google.golang.org/protobuf](https://redirect.github.com/protocolbuffers/protobuf-go)\n| `v1.36.10` -> `v1.36.11` |\n![age](https://developer.mend.io/api/mc/badges/age/go/google.golang.org%2fprotobuf/v1.36.11?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/google.golang.org%2fprotobuf/v1.36.10/v1.36.11?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>protocolbuffers/protobuf-go\n(google.golang.org/protobuf)</summary>\n\n###\n[`v1.36.11`](https://redirect.github.com/protocolbuffers/protobuf-go/releases/tag/v1.36.11)\n\n[Compare\nSource](https://redirect.github.com/protocolbuffers/protobuf-go/compare/v1.36.10...v1.36.11)\n\n**Full Changelog**:\n<https://github.com/protocolbuffers/protobuf-go/compare/v1.36.10...v1.36.11>\n\nUser-visible changes:\n[CL/726780](https://go-review.googlesource.com/c/protobuf/+/726780):\nencoding/prototext: Support URL chars in type URLs in text-format.\n\nBug fixes:\n[CL/728680](https://go-review.googlesource.com/c/protobuf/+/728680):\ninternal/impl: check recursion limit in lazy decoding validation\n[CL/711015](https://go-review.googlesource.com/c/protobuf/+/711015):\nreflect/protodesc: fix handling of import options in dynamic builds\n\nMaintenance:\n[CL/728681](https://go-review.googlesource.com/c/protobuf/+/728681):\nreflect/protodesc: add support for edition unstable\n[CL/727960](https://go-review.googlesource.com/c/protobuf/+/727960):\nall: add EDITION\\_UNSTABLE support\n[CL/727940](https://go-review.googlesource.com/c/protobuf/+/727940):\ntypes: regenerate using latest protobuf v33.2 release\n[CL/727140](https://go-review.googlesource.com/c/protobuf/+/727140):\ninternal/testprotos/lazy: convert .proto files to editions\n[CL/723440](https://go-review.googlesource.com/c/protobuf/+/723440):\ncmd/protoc-gen-go: add missing annotations for few generated protobuf\nsymbols.\n[CL/720980](https://go-review.googlesource.com/c/protobuf/+/720980):\ninternal/filedesc: remove duplicative Message.unmarshalOptions\n[CL/716360](https://go-review.googlesource.com/c/protobuf/+/716360):\ninternal/encoding/tag: use proto3 defaults if proto3\n[CL/716520](https://go-review.googlesource.com/c/protobuf/+/716520):\nproto: un-flake TestHasExtensionNoAlloc\n[CL/713342](https://go-review.googlesource.com/c/protobuf/+/713342):\ncompiler/protogen: properly filter option dependencies in go-protobuf\nplugin.\n[CL/711200](https://go-review.googlesource.com/c/protobuf/+/711200):\nproto: add test for oneofs containing messages with required fields\n[CL/710855](https://go-review.googlesource.com/c/protobuf/+/710855):\nproto: add explicit test for a non-nil but empty byte slice\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi40Mi4yIiwidXBkYXRlZEluVmVyIjoiNDIuNDIuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2025-12-22T18:04:22Z",
          "tree_id": "805ef27817fea8cbab75dc258bcddcc658ae5fdc",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4396b28f6e9ca9920fefd4cf9f0862c63f8ef136"
        },
        "date": 1766427912322,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.67203053999685,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.80894131859577,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.67203053999685,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.80894131859577,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.77591145833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.62109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9817600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9817600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 163624.21503051146,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 163624.21503051146,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000899,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4596250.751440575,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4400943.310765471,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 418000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.620999336242676,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.28939654171856,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.47225954784763,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.28939654171856,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.47225954784763,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.753255208333336,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.8515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 7436400,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7018400,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 123937.74846423624,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 116971.20835638152,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00109,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3694308.256874815,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2134586.547055078,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "69ef3efb9ed59b57e063f4a52de94dd089ce56a0",
          "message": "DF Engine binary size tracking (#1673)\n\nhttps://github.com/open-telemetry/otel-arrow/issues/1575",
          "timestamp": "2025-12-22T19:27:53Z",
          "tree_id": "1c7029d4c86fc15341a69d631747973a9a0971a4",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/69ef3efb9ed59b57e063f4a52de94dd089ce56a0"
        },
        "date": 1766432936669,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 89600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.9134840965270996,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.56176216270465,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.7341489505503,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.56176216270465,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.7341489505503,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.6953125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.26171875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9808600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 9719000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 163472.90951429633,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 161979.61050195197,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001379,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4591877.469078604,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4398104.893350472,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 392800,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.56926155090332,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 88.4154861357598,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.39785557412566,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 88.4154861357598,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.39785557412566,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.139973958333336,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.046875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 7053000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6660200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 117546.8908847361,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 111000.39737282282,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001587,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3450805.4268709724,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1981604.638717779,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "76450334+andborja@users.noreply.github.com",
            "name": "Andres Borja",
            "username": "andborja"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0b67b7783d5c2621fbffb5cbb4b74200a834d2c9",
          "message": "fix: Small (but anoying) fixes to tests and code (#1683)\n\nSmall (but anoying) fixes to tests and code",
          "timestamp": "2025-12-22T20:21:22Z",
          "tree_id": "1b9e5e73c9fb1a08053124aa5b872d896f0df229",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0b67b7783d5c2621fbffb5cbb4b74200a834d2c9"
        },
        "date": 1766436318179,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 382000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.75799036026001,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.06903734006941,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.22957002938901,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.06903734006941,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.22957002938901,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.243229166666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.66015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 8028600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7646600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 133806.9068303371,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 127440.38733637941,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001387,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4012243.261132849,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2317469.3780412087,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.46433752157981,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.5764883781908,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.46433752157981,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.5764883781908,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.51705729166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.80859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11780600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11780600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 196339.40327294447,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 196339.40327294447,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001201,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5553347.582322238,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5319853.217951251,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9892ed711f59b7276e28cccc8b4e8ad0f9ebf884",
          "message": "Upgrade Collector dependencies to 0.142.0 and misc Go modules (#1682)\n\nSupersedes #1677, #1676, and #1675.\n\nTouched manually since already upgrading Collector dependencies.",
          "timestamp": "2025-12-22T21:24:59Z",
          "tree_id": "4cfc39e849fc77a92a7c312ac633335ea4ffcd33",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9892ed711f59b7276e28cccc8b4e8ad0f9ebf884"
        },
        "date": 1766439965133,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.41450425185407,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.57273035424868,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.41450425185407,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.57273035424868,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.5484375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.4140625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11672600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11672600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 194539.56250814672,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 194539.56250814672,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001163,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5520385.4416324,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5290195.593711678,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 375200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.63484525680542,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.22472718551496,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.34885609365372,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.22472718551496,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.34885609365372,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.865885416666664,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.30078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 8095200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7720000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 134916.43820603137,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 128663.26995633982,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001584,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4044479.281279714,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2336417.7170229857,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6b7aa726b6a0efd22b948a145e2a324826c89969",
          "message": "Update Makefile builder command to replace in build.yaml and Dockerfile (#1685)\n\nFollowing up on\nhttps://github.com/open-telemetry/otel-arrow/pull/1682#discussion_r2640864598\n\nIn 99% of cases, the same version should be used for:\n* Builder\n* Collector/Collector-Contrib references in `otelarrowcol-build.yaml`\n* Main `Dockerfile`\n\nThis addition to `builder` command would be invoked every time\n`genotelarrowcol` is invoked but could be brittle if more references are\nadded elsewhere in the repo.\n\n#1482 is probably the correct long-term approach to stop building our\nown collector in this repo.",
          "timestamp": "2025-12-23T17:48:29Z",
          "tree_id": "c99886fa0d7653b3474a0865753829a51dd9230c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6b7aa726b6a0efd22b948a145e2a324826c89969"
        },
        "date": 1766513356257,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 379200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.594247341156006,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.22110160272402,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.38611016576297,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.22110160272402,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.38611016576297,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.64296875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.7734375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 8253800,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7874600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 137560.94894355163,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 131241.05848831954,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00104,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4138313.0741246217,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2389962.7815291192,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.48322107633379,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.62198440882695,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.48322107633379,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.62198440882695,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.03372395833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.70703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11604600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11604600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 193406.48644882953,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 193406.48644882953,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00109,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5514665.362335316,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5284364.927221274,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a7bf084d7427d9b688d92afa20cf9f02d82de1f2",
          "message": "Use MB for binary size (#1689)\n\nhttps://open-telemetry.github.io/otel-arrow/benchmarks/binary-size/ is\nlive now. It's tracked in bytes. I think its better to use MB for binary\nsize.",
          "timestamp": "2025-12-23T18:01:43Z",
          "tree_id": "180ee459b80d685de6671303eecc20a2407fe485",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a7bf084d7427d9b688d92afa20cf9f02d82de1f2"
        },
        "date": 1766514160284,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.47541682545076,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.62986712105956,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.47541682545076,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.62986712105956,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.1421875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.1953125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11593600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11593600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 193222.72492307823,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 193222.72492307823,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001224,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5507983.909663176,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5277917.556855057,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 497000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 6.208309650421143,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.05183152642762,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.1537793826586,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.05183152642762,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.1537793826586,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.79466145833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.921875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 8005400,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7508400,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 133420.02229311343,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 125136.89451940102,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001489,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4030945.9577923277,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2328026.9075384545,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "da5ebefaa9ab841ce1abc75213f5696fed1a0457",
          "message": "Fix null handling in columnar query engine filtering code (#1688)\n\ncloses #1680 \n\nFixes the handling of null values in our filtering code. We expect the\nselection vectors we combine for each filter criteria not to contain\nnulls. This corrects the behaviour.\n\nWhen comparing values using binary operators like\n[`eq`](https://arrow.apache.org/rust/arrow/compute/kernels/cmp/fn.eq.html),\narrow's compute kernels will produce `null` in the resulting boolean\narray. `nulls` cause problems because the `nulls` would propagate when\nwe combine the selection vectors using `and`, `or` and `not`. For\nexample, arrow's\n[`or`](https://arrow.apache.org/rust/arrow/compute/fn.or.html) treats\n`true | null` as null.\n\nWe'd like to treat expect `null == \"val\"` as `false` and we expect to\nproduce selection vectors that have no nulls. The solution is to combine\nthe null buffer that results from executing the physical expression with\nthe values buffer, to remove the nulls.",
          "timestamp": "2025-12-23T18:28:55Z",
          "tree_id": "fcdece15d7ccdaaeb70ce002236300e20ee32c11",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/da5ebefaa9ab841ce1abc75213f5696fed1a0457"
        },
        "date": 1766515998217,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 380000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.703203201293945,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.10211124538958,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.21660665429808,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.10211124538958,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.21660665429808,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.303515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.34765625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 8079600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7699600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 134656.99266049726,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 128323.80076844951,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00134,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4043126.6705671544,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2334745.416256388,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.45274566693352,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.63370350324374,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.45274566693352,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.63370350324374,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.297265625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.93359375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11724600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11724600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 195405.12464214017,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 195405.12464214017,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001497,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5528549.861737861,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5299054.272729989,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "abffa4b3884e48bea960028496dd3759a710b231",
          "message": "Fix bugs transforming attributes when all attrs deleted (#1687)\n\ncloses #1681 \n\n- Fixes a bug where calling `transform_attributes` would produce an\nerror when attribute keys are dictionary encoded and all attributes are\ndeleted.\n- Adds proper handling of the situation where this is called (in\n`AttributesProcessor` and `AttributesTransformFilterStage`), which is\nthat the `RecordBatch` for the attribute's payload type should be\nremoved from the OTAP batch.\n- Consolidates the transform attributes into a single function. Before\nthis, we had two functions `transform_attributes` and\n`transform_attributes_with_stats` that had almost identical logic (the\ndifferences just being stats returned, and comments)\n- Fixes the to_string method on various `otap_df_pdata::error::Error`\nvariants that wrapped `ArrowError` but didn't expose the underlying\nerror in their error message.",
          "timestamp": "2025-12-23T20:12:51Z",
          "tree_id": "0bab64a09de36d2c42a0aeeb90b535ea61e59320",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/abffa4b3884e48bea960028496dd3759a710b231"
        },
        "date": 1766522219290,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 414200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.207835674285889,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.06717503622468,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.22153173876166,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.06717503622468,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.22153173876166,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.2109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.6875,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 7953400,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7539200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 132553.3152770121,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 125650.15647854371,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001517,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3971132.317161049,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2294411.9565927065,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.51089525039242,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.64668522491884,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.51089525039242,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.64668522491884,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.44453125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11646600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11646600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 194077.66989482334,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 194077.66989482334,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009995,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5491168.314609689,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5261001.244270333,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3ea5b2096108070f8674c450b149a3cd4c832190",
          "message": "PipelinePerfTest - remove running with debug flags (#1690)\n\nhttps://github.com/open-telemetry/otel-arrow/pull/1656 added ability to\nshow logs from the containers. The \"--debug\" flag is also doing it,\nwhich feels quite noisy. This PR removes the \"--debug\" flag from all\nperf tests.",
          "timestamp": "2025-12-23T20:53:01Z",
          "tree_id": "626718281b05bb0c2be1e33c50974cf6cd563b91",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3ea5b2096108070f8674c450b149a3cd4c832190"
        },
        "date": 1766524477923,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 502600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 6.404831886291504,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 97.85009235012707,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 97.99166620775193,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.85009235012707,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.99166620775193,
            "unit": "%",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.711848958333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.234375,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 7847200,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 7344600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 130784.60244969135,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 122408.06799266019,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000947,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3944662.2741343062,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2283124.6670611845,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 76600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.6621944308280945,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.4434492121747,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.61991956676465,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.4434492121747,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.61991956676465,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.10520833333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.8203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11567600,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11491000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 192790.38364046364,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 191513.73650649813,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000918,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5489323.915688353,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5259732.719432628,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "dd02f5bde29751989cb16f1e8b8d94d64915c2e0",
          "message": "PipelinePerfTest - Try scaling tests for engine (#1670)",
          "timestamp": "2025-12-23T22:20:45Z",
          "tree_id": "09cdc76be6695bcd8d43f9b947573d78e50063f7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dd02f5bde29751989cb16f1e8b8d94d64915c2e0"
        },
        "date": 1766530091297,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 654000,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6857234239578247,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 370.1385088760202,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 370.71845072959377,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 92.53462721900505,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 92.67961268239844,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 79.70833333333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 83.703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 38796400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 38142400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 646594.7154410096,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 635694.9169056192,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001109,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18276598.7355169,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17512496.047322705,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 5039600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.957571983337402,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 1136.8265028156277,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 1143.5203116289838,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 71.05165642597673,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 71.47001947681149,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 273.13684895833336,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 289.0703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 101654600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 96615000,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1694189.8816425675,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1610199.1982152963,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001893,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 47557623.75632008,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 45480692.610974275,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 385200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.8303461074829102,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 196.4699383682423,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 196.77295938376213,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.23496918412116,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.38647969188106,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.483723958333336,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.3359375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 21045200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 20660000,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 350745.1492798501,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 344325.2990763549,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0014,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9897819.063781554,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9476539.247250348,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 2226400,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.178537368774414,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 670.2985128145601,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 672.9475616751035,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.78731410182002,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 84.11844520938794,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 140.01484375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 147.3515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 70044800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 67818400,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1167373.66208505,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1130268.2563837538,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002039,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33087811.324470345,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 31690757.064902972,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.52096049890758,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.6526093081373,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.52096049890758,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.6526093081373,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.25559895833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.8359375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11134600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11134600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 185573.18098375053,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 185573.18098375053,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001127,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5282703.501371073,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5059180.0748552475,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b639b30ee6fb2e1453a80c71a0b29c018c233980",
          "message": "Columnar query engine support conditionally executing pipeline stages (#1684)\n\npart of #1667 \n\nAdds a new `ConditionalDataExpression` to the transformation expression\nAST for applying transformation `PipelineStages` to some subset of rows\nthat match a condition. This is used to implement a\n`ConditionalPipelineStage`, which operates like `if/else if/else`\ncontrol flow.\n\nFor example, imagine we had a hypothetical syntax like:\n```kql\nlogs |\n  if (severity_text == \"ERROR\") {\n     set attributes[\"important\"] = \"very\" | set attributes[\"triggers_alarm\"] = true\n  } else if (severity_text == \"WARN) {\n     set attributes[\"important\"] = \"somewhat\"\n  } else {\n     set attributes[\"important\"] = \"no\"\n  }\n```\n\nThis could be modeled using our conditional expression like:\n```rs\n// this is pesudocode to illustrate what each field represents\nConditional {\n  branches: [\n     ConditionalBranch {\n       condition: \"severity_text == \\\"ERROR\\\"\",\n       expressions: [ \n         \"set attributes[\\\"important\\\"] = \\\"very\\\"\",\n         \"set attributes[\\\"triggers_alarm\\\"] = true\"  \n      ],\n     },\n     ConditionalBranch {\n       condition: \"severity_text == \\\"WARN\\\"\",\n       expressions: [\n        \"set attributes[\\\"important\\\"] = \\\"somewhat\\\"\n      ],\n     },\n  ],\n  default_branch: Some([\n    \"set attributes[\"important\"] = \\\"no\\\"\"\n  ])\n}\n```\n\nNote there is currently no parser support for a language syntax that\ncreates this variant of `DataExpression`. That will happen in a future\nPR\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-12-23T23:56:20Z",
          "tree_id": "5b528ed718099c799e52accd3b2cf680103cf427",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b639b30ee6fb2e1453a80c71a0b29c018c233980"
        },
        "date": 1766537197504,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 714000,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.966432809829712,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 345.4754598267543,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 347.5979719396985,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.36886495668857,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.89949298492462,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 83.892578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 86.4921875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 36309400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 35595400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 605087.7975905042,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 593189.1518546997,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006829,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17003809.432974514,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16289382.596590826,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 3730400,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.6927852630615234,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 1138.5961210338958,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 1146.828885142372,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 71.16225756461849,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 71.67680532139825,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 276.93984375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 290.23828125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 101018600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 97288200,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1683572.735516624,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1621402.0092090806,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002516,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 47346936.479660116,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 45268530.47882856,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 217800,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.1876847743988037,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 170.76410441561865,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 171.20624211715545,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.38205220780932,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.60312105857773,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.76197916666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.5078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 18338200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 18120400,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 305628.8578493486,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 301998.950593479,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001533,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 8543236.666862972,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8180094.214274252,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1107000,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6450008153915405,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 638.9331181627359,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 640.3919880123839,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 79.86663977034199,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 80.04899850154798,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 131.88359375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 139.0078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 67294800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 66187800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1121546.9891336197,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1103097.5321626365,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001766,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 31687544.037479248,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 30368694.82125974,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 8600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.07519060373306274,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.53035623597343,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.65323233067575,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.53035623597343,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.65323233067575,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.38138020833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.7109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11437600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11429000,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 190623.71835315615,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 190480.3872366774,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000928,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5387164.535815555,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5161541.219286042,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9e2e53ae578e852cd2bec269b1b9b84ba3b20453",
          "message": "fix(deps): update golang.org/x/exp digest to 944ab1f (#1585)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [golang.org/x/exp](https://pkg.go.dev/golang.org/x/exp) | require |\ndigest | `87e1e73` -> `944ab1f` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - At any time (no schedule defined),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi40Mi4yIiwidXBkYXRlZEluVmVyIjoiNDIuNTkuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2025-12-26T19:50:46Z",
          "tree_id": "a776071811573fd9652cd1301987760426258307",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9e2e53ae578e852cd2bec269b1b9b84ba3b20453"
        },
        "date": 1766780245198,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 715000,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.26005482673645,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 295.66655750060335,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 297.2209663881837,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 73.91663937515084,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 74.30524159704592,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 79.31236979166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 82.0703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 31636400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 30921400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 527257.4189469048,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 515341.1119541042,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001811,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14683136.58868352,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14048425.941364696,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 5417200,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 6.4610772132873535,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 884.0827892103167,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 892.5573184660404,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.255174325644795,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.78483240412753,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 265.1528645833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 280.39453125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 83843600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 78426400,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1397320.0438970309,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1307038.1125176651,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003147,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 38432491.1424789,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 36719815.03651394,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 564000,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.379025936126709,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 151.32954553938276,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 151.7577930401055,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 75.66477276969138,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 75.87889652005275,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.70651041666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 16691200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 16127200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 278180.51424096,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 268780.7221330288,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001327,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7732347.513073725,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7397936.39152558,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 2399600,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.6703028678894043,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 620.6524348933981,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 622.2518267171679,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.58155436167476,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 77.78147833964599,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 129.679296875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 134.93359375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 65378800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 62979200,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1089579.7846275535,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1049588.9060676491,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003683,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 30690214.54303211,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 29409724.55245542,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 3600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.03154684603214264,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.4575298998566,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.60037998297345,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.4575298998566,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.60037998297345,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.61510416666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.51171875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11411600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11408000,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 190190.01451758,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 190130.01556456173,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001047,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5383031.681759169,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5156924.220144415,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mblanchard@macrosssoftware.com",
            "name": "Mikel Blanchard",
            "username": "CodeBlanch"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1",
          "message": "[query-engine] KQL function parsing + type conversion (#1668)\n\nRelates to #1479\n\n## Changes\n\n* Adds support for function definition and invocation in KQL parser\n* Implements automatic type conversion for functions in RecordSet engine",
          "timestamp": "2025-12-26T22:23:48Z",
          "tree_id": "08d702f979db333718ff43aeffdbdcf538bcdf19",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1"
        },
        "date": 1766789625314,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 591800,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.902563452720642,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 294.9274710005992,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 295.4277771578785,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 73.7318677501498,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 73.85694428946962,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 70.309765625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 71.6015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 31105400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 30513600,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 518412.2911515318,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 508549.16790272365,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001278,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14376106.420975411,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13784462.982384417,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 3177000,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.0469677448272705,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 1170.0071832131846,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 1177.2311579880018,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 73.12544895082404,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 73.57694737425011,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 275.42552083333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 292.296875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 104267600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 101090600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1737737.03065354,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1684788.7461779572,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001944,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 48809575.13001397,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 46692553.26367446,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 474200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.5889649391174316,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 171.21226741906662,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 171.69166274277705,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.60613370953331,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.84583137138853,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 51.762109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.83203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 18316200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 17842000,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 305264.84611184814,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 297361.64621087315,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001013,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 8543223.308844093,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8176017.151026512,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 936800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.3109285831451416,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 691.3088020592602,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 693.6477839938128,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.41360025740752,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.7059729992266,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 137.25989583333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 146.5859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 71460800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 70524000,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1190951.5229492923,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1175338.999905905,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003114,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33731560.113807395,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32333921.806573056,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 13600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.12317055463790894,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.57938751518826,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.69782747405917,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.57938751518826,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.69782747405917,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.93567708333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.88671875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11041600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11028000,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 184022.7653840405,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 183796.10352260532,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001272,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5239467.622262513,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5019259.976831293,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "66b4c7e30dca8c44340dace1056ed5a5887366ae",
          "message": "chore(deps): update dependency psutil to v7.2.1 (#1698)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [psutil](https://redirect.github.com/giampaolo/psutil) | `==7.1.3` ->\n`==7.2.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/psutil/7.2.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/psutil/7.1.3/7.2.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>giampaolo/psutil (psutil)</summary>\n\n###\n[`v7.2.1`](https://redirect.github.com/giampaolo/psutil/blob/HEAD/HISTORY.rst#721)\n\n[Compare\nSource](https://redirect.github.com/giampaolo/psutil/compare/release-7.2.0...release-7.2.1)\n\n\\=====\n\n2025-12-29\n\n**Bug fixes**\n\n- 2699\\_, \\[FreeBSD], \\[NetBSD]: `heap_info()`\\_ does not detect small\nallocations\n(<= 1K). In order to fix that, we now flush internal jemalloc cache\nbefore\n  fetching the metrics.\n\n###\n[`v7.2.0`](https://redirect.github.com/giampaolo/psutil/blob/HEAD/HISTORY.rst#720)\n\n[Compare\nSource](https://redirect.github.com/giampaolo/psutil/compare/release-7.1.3...release-7.2.0)\n\n\\=====\n\n2025-12-23\n\n**Enhancements**\n\n- 1275\\_: new `heap_info()`\\_ and `heap_trim()`\\_ functions, providing\ndirect\n  access to the platform's native C heap allocator (glibc, mimalloc,\n  libmalloc). Useful to create tools to detect memory leaks.\n- 2403\\_, \\[Linux]: publish wheels for Linux musl.\n- 2680\\_: unit tests are no longer installed / part of the distribution.\nThey\n  now live under `tests/` instead of `psutil/tests`.\n\n**Bug fixes**\n\n- 2684\\_, \\[FreeBSD], \\[critical]: compilation fails on FreeBSD 14 due\nto missing\n  include.\n- 2691\\_, \\[Windows]: fix memory leak in `net_if_stats()`\\_ due to\nmissing\n  `Py_CLEAR`.\n\n**Compatibility notes**\n\n- 2680\\_: `import psutil.tests` no longer works (but it was never\ndocumented to\n  begin with).\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi41OS4wIiwidXBkYXRlZEluVmVyIjoiNDIuNTkuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-29T19:06:30Z",
          "tree_id": "56cded756d16689177f25411efffa6cea4ba33b1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/66b4c7e30dca8c44340dace1056ed5a5887366ae"
        },
        "date": 1767036809053,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 243000,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.6399553418159485,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 369.3983452606088,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 370.2434495167286,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 92.3495863151522,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 92.56086237918215,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 85.27122395833334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 88.1015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 37971400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 37728400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 632829.2862528815,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 628779.4614753003,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002596,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18003031.56792393,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17251201.29617255,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 4868800,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.2041778564453125,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 1023.7959859268933,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 1038.871058622123,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 63.98724912043083,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.92944116388269,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 248.92291666666668,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 262.30859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 93555600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 88686800,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1559165.3846472416,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1478023.6419319953,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003641,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 43394977.87322254,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 41465928.98062025,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 120200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.5811198949813843,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 196.7130694176396,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 197.06522622657516,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.3565347088198,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.53261311328758,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.429296875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.73046875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 20684200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 20564000,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 344727.4854246382,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 342724.2054453283,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001598,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9735801.011073364,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9317930.944816295,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1108000,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.7216250896453857,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 613.6858470591059,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 617.2806276799381,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 76.71073088238823,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 77.16007845999226,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 141.2703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 150.7421875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 64357800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 63249800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1072607.8685243127,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1054141.5828786732,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001238,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 30131737.97361213,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 28854808.234428067,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 178600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.5929930210113525,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.58309820557439,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.777710113927,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.58309820557439,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.777710113927,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.4578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.78515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11211600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11033000,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 186857.27188383052,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 183880.64867586267,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000876,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5276952.410423273,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5054837.177361567,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5256452a56cfff9e2e552d627108e23bceb28ac5",
          "message": "Skip tls_reload_integration on Windows (#1705)\n\nFailing on unrelated PR:\nhttps://github.com/open-telemetry/otel-arrow/actions/runs/20586507089/job/59123800890?pr=1697\n\nSeems to be similar to other tests in this part of the code being\nunreliable on `macos` and `windows`. Already have issue #1614 tracking\neventual fix.\n\nFor now, see if this gets CI back to stable.",
          "timestamp": "2025-12-30T23:13:45Z",
          "tree_id": "933d382876cee0b3ce4ea000cf80f253cf252caa",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5256452a56cfff9e2e552d627108e23bceb28ac5"
        },
        "date": 1767138234575,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 570800,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6672313213348389,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 328.7651603657615,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 329.9114536434588,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 82.19129009144038,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 82.4778634108647,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 80.70169270833334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 83.88671875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 34236400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 33665600,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 570594.6461394547,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 561081.5132161216,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001264,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16145702.630846115,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15466572.017282363,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 4725200,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.3454270362854,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 1197.9157559979492,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 1205.7748315226083,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 74.86973474987182,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 75.36092697016302,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 257.96692708333336,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 271.86328125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 108739600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 104014400,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1812230.7392528688,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1733481.5743753295,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003176,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 51400804.61383424,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 49209244.77089487,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 227800,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.250535249710083,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 171.16334649363543,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 171.60047607521474,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.58167324681772,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.80023803760737,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.205989583333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 18216200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 17988400,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 303598.72875261394,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 299802.1196678517,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00091,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 8561243.885193039,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8192816.74858431,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 2303400,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.347050428390503,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 658.354215511014,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 661.2053888373522,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 82.29427693887675,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 82.65067360466902,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 141.408984375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 151.31640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 68818800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 66515400,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1146927.356034358,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1108539.1180544812,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002754,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 32437311.970975623,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 31051767.63087777,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 2600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.02306886948645115,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.60461095192456,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.84541471409831,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.60461095192456,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.84541471409831,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.104817708333336,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.8984375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11270600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11268000,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 187840.45937430492,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 187797.12670396143,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000918,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5310347.193163014,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5087193.96966952,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "l.querel@f5.com",
            "name": "Laurent Quérel",
            "username": "lquerel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "acdbd4c4615bd9a6fb8e98ef2938092120ac3d99",
          "message": "Channel metrics (#1697)\n\nThis PR introduces channel sender/receiver metric sets (send/recv counts\nand error counts, plus capacity) and a consolidated ChannelAttributeSet\nincluding channel kind/mode/type/impl and node URN. A new\n`TelemetrySettings.channel_metrics` flag gates registration/reporting to\navoid overhead when disabled.\n\nI also added few additional `otel_debug!` to help diagnose pipeline\ninitialization and creation.\n\nI didn't observe any performance regression.\n\nChannel attributes:\n```rust\n/// Channel endpoint attributes (sender or receiver).\n#[attribute_set(name = \"channel.attrs\")]\n#[derive(Debug, Clone, Default, Hash)]\npub struct ChannelAttributeSet {\n    /// Node attributes.\n    #[compose]\n    pub node_attrs: NodeAttributeSet,\n\n    /// Unique channel identifier (in scope of the pipeline).\n    #[attribute(key = \"channel.id\")]\n    pub channel_id: Cow<'static, str>,\n    /// Channel payload kind (\"control\" or \"pdata\").\n    #[attribute(key = \"channel.kind\")]\n    pub channel_kind: Cow<'static, str>,\n    /// Concurrency mode of the channel (\"local\" or \"shared\").\n    #[attribute(key = \"channel.mode\")]\n    pub channel_mode: Cow<'static, str>,\n    /// Channel type (\"mpsc\", \"mpmc\", \"spsc\", \"spmc\").\n    #[attribute(key = \"channel.type\")]\n    pub channel_type: Cow<'static, str>,\n    /// Channel implementation (\"tokio\", \"flume\", \"internal\").\n    #[attribute(key = \"channel.impl\")]\n    pub channel_impl: Cow<'static, str>,\n}\n```\n\nChannel metrics:\n```rust\n#[metric_set(name = \"channel.sender\")]\n#[derive(Debug, Default, Clone)]\npub struct ChannelSenderMetrics {\n    /// Count of messages successfully sent to the channel.\n    #[metric(name = \"send.count\", unit = \"{message}\")]\n    pub send_count: Counter<u64>,\n    /// Count of send failures due to a full channel.\n    #[metric(name = \"send.error_full\", unit = \"{1}\")]\n    pub send_error_full: Counter<u64>,\n    /// Count of send failures due to a closed channel.\n    #[metric(name = \"send.error_closed\", unit = \"{1}\")]\n    pub send_error_closed: Counter<u64>,\n    // Total bytes successfully sent (when message size is known).\n    // TODO: Populate in a future PR when message sizes are tracked.\n    // #[metric(name = \"send.bytes\", unit = \"{By}\")]\n    // pub send_bytes: Counter<u64>,\n}\n\n#[metric_set(name = \"channel.receiver\")]\n#[derive(Debug, Default, Clone)]\npub struct ChannelReceiverMetrics {\n    /// Count of messages successfully received from the channel.\n    #[metric(name = \"recv.count\", unit = \"{message}\")]\n    pub recv_count: Counter<u64>,\n    /// Count of receive attempts when the channel was empty.\n    #[metric(name = \"recv.error_empty\", unit = \"{1}\")]\n    pub recv_error_empty: Counter<u64>,\n    /// Count of receive attempts after the channel was closed.\n    #[metric(name = \"recv.error_closed\", unit = \"{1}\")]\n    pub recv_error_closed: Counter<u64>,\n    // Total bytes successfully received (when message size is known).\n    // TODO: Populate in a future PR when message sizes are tracked.\n    // #[metric(name = \"recv.bytes\", unit = \"{By}\")]\n    // pub recv_bytes: Counter<u64>,\n    // Current number of buffered messages.\n    // TODO: Populate in a future PR when queue depth is tracked.\n    // #[metric(name = \"queue.depth\", unit = \"{message}\")]\n    // pub queue_depth: Gauge<u64>,\n    /// Maximum channel capacity (buffer size).\n    #[metric(name = \"capacity\", unit = \"{message}\")]\n    pub capacity: Gauge<u64>,\n}\n```\n\nPS: I will introduce latency metrics once we have a support for\nhistograms.",
          "timestamp": "2025-12-31T00:06:23Z",
          "tree_id": "1fe13dd3214ced0d7783460e2761e96a9df29250",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/acdbd4c4615bd9a6fb8e98ef2938092120ac3d99"
        },
        "date": 1767141449132,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 192800,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.5011906027793884,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 369.01516446058287,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 369.71765931364234,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 92.25379111514572,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 92.42941482841059,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 81.18020833333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 83.859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 38468400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 38275600,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 641097.4097620782,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 637884.2898870085,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003986,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18132214.841283277,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17312192.203698777,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 4910800,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.687838554382324,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 910.9645388292553,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 914.7849963157487,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 56.935283676828455,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 57.17406226973429,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 273.43359375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 287.23828125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 86338600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 81427800,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1438825.9976042842,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1356987.9007503265,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006283,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 39434223.943515494,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 37496410.4013034,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 196.54914310709967,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 196.88344111351685,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.27457155354983,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.44172055675843,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.80442708333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.43359375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 20730200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 20730200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 345489.2834358069,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 345489.2834358069,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00244,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9771619.860100567,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9322054.442726603,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1326400,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.9976565837860107,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 639.1968164299109,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 644.3841994342685,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 79.89960205373886,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 80.54802492928356,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 137.40013020833334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 140.70703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 66397800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 65071400,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1106595.1238103479,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1084489.1538501678,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001891,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 31587400.567448806,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 30135220.83270623,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 168600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.483815312385559,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.5679301385437,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.73470673677699,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.5679301385437,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.73470673677699,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.94088541666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.8125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11362600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11194000,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 189373.10645226535,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 186563.15927927222,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001128,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5369795.27171073,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5127401.889541508,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1af4d28d2119f57dae33f166f635ef16d69223a5",
          "message": "feat: Initial Condense processor implementation (#1695)\n\nRelated to #1435\n\nFixes #1693\n\nThis is a basic implementation of the `Condense` behavior from the above\nissue that works for `LogAttrs` payload types.\n\nThis iteration currently builds an entirely new `RecordBatch` during\nexecution. As mentioned in comments, once #1035 is completed, working\nin-place on the existing `RecordBatch` would be more efficient\nespecially with respect to persisted attributes.\n\nWith a debug pipeline configuration composed of:\n* `syslog_cef_receiver`\n* `attributes_processor` (doing various renames and deletes)\n* `condense_attributes_processor`\n\nSending the CEF message:\n> <134>Dec 29 17:28:13 securityhost\nCEF:0|Security|threatmanager|1.0|100|worm successfully\nstopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp\nact=blocked vendorspecificext1=value1 vendorspecificext2=value2\n\nResults in the following LogRecord:\n```\nLogRecord #0:\n   -> ObservedTimestamp: 1767029293753398998\n   -> Timestamp: 1767029293000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Body: <134>Dec 29 17:28:13 securityhost CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp act=blocked vendorspecificext1=value1 vendorspecificext2=value2\n   -> Attributes:\n      -> AdditionalExtensions: vendorspecificext1=value1|vendorspecificext2=value2\n      -> Computer: securityhost\n      -> DeviceVendor: Security\n      -> DeviceProduct: threatmanager\n      -> DeviceVersion: 1.0\n      -> DeviceEventClassId: 100\n      -> Activity: worm successfully stopped\n      -> LogSeverity: 10\n      -> SourceIP: 10.0.0.1\n      -> DestinationIP: 2.1.2.2\n      -> SourcePort: 1232\n      -> DestinationPort: 80\n      -> Protocol: tcp\n      -> DeviceAction: blocked\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0 \n```",
          "timestamp": "2025-12-31T00:42:40Z",
          "tree_id": "4e06011fad634e5a9c778e0ec4f753ff9f27fb48",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1af4d28d2119f57dae33f166f635ef16d69223a5"
        },
        "date": 1767143642827,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 545600,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.5109639167785645,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 348.15529890696286,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 348.9992156804459,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 87.03882472674071,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.24980392011148,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 78.32018229166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 80.171875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 36109400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 35563800,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 601811.3372273446,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 592718.1851508426,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001196,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16995803.285509285,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16225289.253711915,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 4711000,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.333372592926025,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 949.1938645068284,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 956.3545998133748,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 59.324616531676774,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.77216248833592,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 267.95221354166665,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 280.58203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 88330600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 83619600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1472093.7877864144,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1393581.5413592216,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003378,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 40673601.17332933,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 38740815.27573193,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 237000,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.4422023296356201,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 152.1274873666919,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 152.44913718187462,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 76.06374368334595,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 76.22456859093731,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.38255208333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.8515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 16433200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 16196200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 273880.88321534946,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 269930.9666244215,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001267,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7631532.5268965,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7271880.071193661,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 2369200,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.6555070877075195,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 624.1944959288353,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 625.8790639039132,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 78.02431199110441,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 78.23488298798915,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 144.19674479166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 149.51953125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 64811800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 62442600,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1080172.2187687817,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1040686.4457967768,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001358,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 30449000.113249715,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 29057565.060675815,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 168600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.5113486051559448,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.57373320768103,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.72535833256137,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.57373320768103,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.72535833256137,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.938151041666664,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.41015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11155600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 10987000,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 185923.62991404475,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 183113.67580996177,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00098,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5264929.778317039,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5026960.285787073,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "76450334+andborja@users.noreply.github.com",
            "name": "Andres Borja",
            "username": "andborja"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2584e4b429b131115964742db4115782e6b19781",
          "message": "feat: Add internal telemetry prometheus exporter (#1691)\n\nAdd internal telemetry configurable prometheus exporter.",
          "timestamp": "2026-01-03T22:00:15Z",
          "tree_id": "7709c84d086c2cdb0dfe11d1658864c8ee41e547",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2584e4b429b131115964742db4115782e6b19781"
        },
        "date": 1767479467555,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 392.3482907499826,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 393.4445361958452,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.08707268749565,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.3611340489613,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 76.76302083333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 82.26953125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 39766400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 39766400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 662732.6525940083,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 662732.6525940083,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003683,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18811397.797286753,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17971774.434023794,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 4792600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.757272243499756,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 1145.2135446251561,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 1151.2815162816728,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 71.57584653907226,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 71.95509476760455,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 264.7057291666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 277.16015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 100742600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 95950000,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1678810.8739876486,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1598945.266045495,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008308,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 47292979.28484309,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 45088679.5759164,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 108800,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.6200419068336487,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 171.7921513061347,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 172.24641156269303,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.89607565306736,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.12320578134651,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.430859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.43359375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 17547200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 17438400,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 292448.07414213335,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 290634.77341799135,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001079,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 8214399.109306201,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7825539.56548926,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1750200,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.7662923336029053,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 616.2910286721166,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 617.295277393794,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.03637858401457,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 77.16190967422425,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 134.04127604166666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 137.53515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 63268800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 61518600,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1054438.2090989794,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1025269.3651574942,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002378,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 29652631.051393826,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 28302948.214795973,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 165600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.4796812534332275,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.62890750564263,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.77267174730974,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.62890750564263,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.77267174730974,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.008723958333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.12109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11191600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11026000,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 186523.8408304781,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 183763.8826438446,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000909,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5292946.937766211,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5052097.18037349,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "32a6fbbcd5edc9ab869be6a125042fa549e45c86",
          "message": "Add gix-date deny exclusion in otap-dataflow (#1719)\n\nResolve active CI blocker",
          "timestamp": "2026-01-05T17:40:23Z",
          "tree_id": "695fe660eb337d83e42c68d7110c3df06011b3bd",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32a6fbbcd5edc9ab869be6a125042fa549e45c86"
        },
        "date": 1767637834652,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 397000,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.0817025899887085,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 351.12080392748277,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 351.5964905940594,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 87.78020098187069,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.89912264851485,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 77.56783854166666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 83.00390625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 36701400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 36304400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 611593.398822656,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 604977.7770934415,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009477,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17287310.083063465,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16501091.44177863,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 5445400,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 6.219717502593994,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 927.2502306132445,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 933.816427409126,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 57.95313941332778,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.36352671307038,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 262.54895833333336,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 273.6328125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 87550600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 82105200,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1458888.6091107978,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1368149.8588103778,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.011847,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 40186536.42217822,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 38224300.58518063,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 314000,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.9260022640228271,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 148.05022837259884,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 148.5552107070551,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 74.02511418629942,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 74.27760535352755,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 51.88229166666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.12109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 16303200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 15989200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 271714.55212322995,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 266481.32371612615,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001203,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7588089.324633012,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7222030.927240619,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1404400,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.047743797302246,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 652.4678974840358,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 655.852675887159,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 81.55848718550448,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.98158448589487,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 135.05755208333332,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 141.765625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 68582800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 67178400,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1142981.2500397894,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1119575.9229380107,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003434,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 32347945.37827798,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 30878869.00960952,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 162600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.4354320764541626,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.53159016841224,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.6647658054029,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.53159016841224,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.6647658054029,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.40846354166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.30078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11327600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11165000,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 188789.4757350458,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 186079.53110824767,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001226,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5353462.95476784,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5110317.973579701,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "5eb861da646507ef5c4b6bcdd3e1ef655e827700",
          "message": "Upgrade arrow-go to v18.5.0 (#1702)\n\nSupersedes #1674\n\nIn-place upgrade from Renovate failed some tests due to a change in\n`v18.4.1`:\n> fix(arrow/array): update timestamp json format by\n[@​zeroshade](https://redirect.github.com/zeroshade) in\n[#​450](https://redirect.github.com/apache/arrow-go/pull/450)\n\nUpdated a number of tests to expect format like\n`1970-01-01T00:00:00.000000004Z` instead of `1970-01-01\n00:00:00.000000004Z`.",
          "timestamp": "2026-01-05T18:09:45Z",
          "tree_id": "35a937cf8692cb7e435d7b9a52e3895508919ee6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5eb861da646507ef5c4b6bcdd3e1ef655e827700"
        },
        "date": 1767638772892,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 668200,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.8402947187423706,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 344.35806119775737,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 345.32162840808735,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.08951529943934,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.33040710202184,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 83.45755208333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 85.578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 36309400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 35641200,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 605137.7661971025,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 594001.4473547942,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001874,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17067415.867344555,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16281921.322545022,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 5794200,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.658862590789795,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 1118.4699599795867,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 1124.7786110259578,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.90437249872417,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 70.29866318912237,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 276.2942708333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 287.96875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 102391600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 96597400,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1706327.9932106573,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1609769.235868637,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006986,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 47802722.09239269,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 45557266.01205432,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 278200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.3527050018310547,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 196.60672107661205,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 196.9598312044472,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.30336053830602,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.4799156022236,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.991015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.4609375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 20566200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 20288000,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 342764.16729641985,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 338127.5795290217,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001021,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9698341.834254093,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9252356.004308872,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1302800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.8227912187576294,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 685.3200188636523,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 688.4247932446359,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.66500235795654,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.05309915557949,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 136.52643229166668,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 147.3515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 71472800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 70170000,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1191167.9299824021,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1169455.424257412,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002287,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33904529.50601799,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32357886.89288706,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 15600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.1372079998254776,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.55853795132057,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.67173032875444,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.55853795132057,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.67173032875444,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.85299479166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.5234375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11369600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11354000,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 189490.19095433332,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 189230.1952659285,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000995,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5377851.698495436,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5135255.437297435,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "813c4858716e4cdd27ef86f4c9a8fedda6f5c0bd",
          "message": "chore(deps): update github workflow dependencies (#1709)\n\n> **Note:** This PR body was truncated due to platform limits.\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[actions/create-github-app-token](https://redirect.github.com/actions/create-github-app-token)\n| action | patch | `v2.2.0` → `v2.2.1` |\n| [actions/setup-node](https://redirect.github.com/actions/setup-node) |\naction | minor | `v6.0.0` → `v6.1.0` |\n|\n[actions/setup-python](https://redirect.github.com/actions/setup-python)\n| action | minor | `v5` → `v5.6.0` |\n|\n[codecov/codecov-action](https://redirect.github.com/codecov/codecov-action)\n| action | patch | `v5.5.1` → `v5.5.2` |\n|\n[docker/setup-buildx-action](https://redirect.github.com/docker/setup-buildx-action)\n| action | minor | `v3.7.1` → `v3.12.0` |\n| dtolnay/rust-toolchain | action | digest | `6d653ac` → `f7ccc83` |\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | patch | `v4.31.6` → `v4.31.9` |\n| [go](https://redirect.github.com/actions/go-versions) | uses-with |\npatch | `1.25.4` → `1.25.5` |\n| [python](https://redirect.github.com/actions/python-versions) |\nuses-with | minor | `3.11` → `3.14` |\n|\n[step-security/harden-runner](https://redirect.github.com/step-security/harden-runner)\n| action | minor | `v2.13.2` → `v2.14.0` |\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | minor | `v2.62.60` → `v2.65.13` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>actions/create-github-app-token\n(actions/create-github-app-token)</summary>\n\n###\n[`v2.2.1`](https://redirect.github.com/actions/create-github-app-token/releases/tag/v2.2.1)\n\n[Compare\nSource](https://redirect.github.com/actions/create-github-app-token/compare/v2.2.0...v2.2.1)\n\n##### Bug Fixes\n\n- **deps:** bump the production-dependencies group with 2 updates\n([#&#8203;311](https://redirect.github.com/actions/create-github-app-token/issues/311))\n([b212e6a](https://redirect.github.com/actions/create-github-app-token/commit/b212e6a739dec02d8488610fbaf8f049f82ee999))\n\n</details>\n\n<details>\n<summary>actions/setup-node (actions/setup-node)</summary>\n\n###\n[`v6.1.0`](https://redirect.github.com/actions/setup-node/releases/tag/v6.1.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-node/compare/v6.0.0...v6.1.0)\n\n#### What's Changed\n\n##### Enhancement:\n\n- Remove always-auth configuration handling by\n[@&#8203;priyagupta108](https://redirect.github.com/priyagupta108) in\n[#&#8203;1436](https://redirect.github.com/actions/setup-node/pull/1436)\n\n##### Dependency updates:\n\n- Upgrade\n[@&#8203;actions/cache](https://redirect.github.com/actions/cache) from\n4.0.3 to 4.1.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;1384](https://redirect.github.com/actions/setup-node/pull/1384)\n- Upgrade actions/checkout from 5 to 6 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;1439](https://redirect.github.com/actions/setup-node/pull/1439)\n- Upgrade js-yaml from 3.14.1 to 3.14.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;1435](https://redirect.github.com/actions/setup-node/pull/1435)\n\n##### Documentation update:\n\n- Add example for restore-only cache in documentation by\n[@&#8203;aparnajyothi-y](https://redirect.github.com/aparnajyothi-y) in\n[#&#8203;1419](https://redirect.github.com/actions/setup-node/pull/1419)\n\n**Full Changelog**:\n<https://github.com/actions/setup-node/compare/v6...v6.1.0>\n\n</details>\n\n<details>\n<summary>actions/setup-python (actions/setup-python)</summary>\n\n###\n[`v5.6.0`](https://redirect.github.com/actions/setup-python/releases/tag/v5.6.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-python/compare/v5.5.0...v5.6.0)\n\n#### What's Changed\n\n- Workflow updates related to Ubuntu 20.04 by\n[@&#8203;aparnajyothi-y](https://redirect.github.com/aparnajyothi-y) in\n[#&#8203;1065](https://redirect.github.com/actions/setup-python/pull/1065)\n- Fix for Candidate Not Iterable Error by\n[@&#8203;aparnajyothi-y](https://redirect.github.com/aparnajyothi-y) in\n[#&#8203;1082](https://redirect.github.com/actions/setup-python/pull/1082)\n- Upgrade semver and\n[@&#8203;types/semver](https://redirect.github.com/types/semver) by\n[@&#8203;dependabot](https://redirect.github.com/dependabot) in\n[#&#8203;1091](https://redirect.github.com/actions/setup-python/pull/1091)\n- Upgrade prettier from 2.8.8 to 3.5.3 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot) in\n[#&#8203;1046](https://redirect.github.com/actions/setup-python/pull/1046)\n- Upgrade ts-jest from 29.1.2 to 29.3.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot) in\n[#&#8203;1081](https://redirect.github.com/actions/setup-python/pull/1081)\n\n**Full Changelog**:\n<https://github.com/actions/setup-python/compare/v5...v5.6.0>\n\n###\n[`v5.5.0`](https://redirect.github.com/actions/setup-python/releases/tag/v5.5.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-python/compare/v5.4.0...v5.5.0)\n\n#### What's Changed\n\n##### Enhancements:\n\n- Support free threaded Python versions like '3.13t' by\n[@&#8203;colesbury](https://redirect.github.com/colesbury) in\n[#&#8203;973](https://redirect.github.com/actions/setup-python/pull/973)\n- Enhance Workflows: Include ubuntu-arm runners, Add e2e Testing for\nfree threaded and Upgrade\n[@&#8203;action/cache](https://redirect.github.com/action/cache) from\n4.0.0 to 4.0.3 by\n[@&#8203;priya-kinthali](https://redirect.github.com/priya-kinthali) in\n[#&#8203;1056](https://redirect.github.com/actions/setup-python/pull/1056)\n- Add support for .tool-versions file in setup-python by\n[@&#8203;mahabaleshwars](https://redirect.github.com/mahabaleshwars) in\n[#&#8203;1043](https://redirect.github.com/actions/setup-python/pull/1043)\n\n##### Bug fixes:\n\n- Fix architecture for pypy on Linux ARM64 by\n[@&#8203;mayeut](https://redirect.github.com/mayeut) in\n[#&#8203;1011](https://redirect.github.com/actions/setup-python/pull/1011)\n  This update maps arm64 to aarch64 for Linux ARM64 PyPy installations.\n\n##### Dependency updates:\n\n- Upgrade [@&#8203;vercel/ncc](https://redirect.github.com/vercel/ncc)\nfrom 0.38.1 to 0.38.3 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot) in\n[#&#8203;1016](https://redirect.github.com/actions/setup-python/pull/1016)\n- Upgrade\n[@&#8203;actions/glob](https://redirect.github.com/actions/glob) from\n0.4.0 to 0.5.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot) in\n[#&#8203;1015](https://redirect.github.com/actions/setup-python/pull/1015)\n\n#### New Contributors\n\n- [@&#8203;colesbury](https://redirect.github.com/colesbury) made their\nfirst contribution in\n[#&#8203;973](https://redirect.github.com/actions/setup-python/pull/973)\n- [@&#8203;mahabaleshwars](https://redirect.github.com/mahabaleshwars)\nmade their first contribution in\n[#&#8203;1043](https://redirect.github.com/actions/setup-python/pull/1043)\n\n**Full Changelog**:\n<https://github.com/actions/setup-python/compare/v5...v5.5.0>\n\n###\n[`v5.4.0`](https://redirect.github.com/actions/setup-python/releases/tag/v5.4.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-python/compare/v5.3.0...v5.4.0)\n\n#### What's Changed\n\n##### Enhancements:\n\n- Update cache error message by\n[@&#8203;aparnajyothi-y](https://redirect.github.com/aparnajyothi-y) in\n[#&#8203;968](https://redirect.github.com/actions/setup-python/pull/968)\n- Enhance Workflows: Add Ubuntu-24, Remove Python 3.8 by\n[@&#8203;priya-kinthali](https://redirect.github.com/priya-kinthali) in\n[#&#8203;985](https://redirect.github.com/actions/setup-python/pull/985)\n- Configure Dependabot settings by\n[@&#8203;HarithaVattikuti](https://redirect.github.com/HarithaVattikuti)\nin\n[#&#8203;1008](https://redirect.github.com/actions/setup-python/pull/1008)\n\n##### Documentation changes:\n\n- Readme update - recommended permissions by\n[@&#8203;benwells](https://redirect.github.com/benwells) in\n[#&#8203;1009](https://redirect.github.com/actions/setup-python/pull/1009)\n- Improve Advanced Usage examples by\n[@&#8203;lrq3000](https://redirect.github.com/lrq3000) in\n[#&#8203;645](https://redirect.github.com/actions/setup-python/pull/645)\n\n##### Dependency updates:\n\n- Upgrade `undici` from 5.28.4 to 5.28.5 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot) in\n[#&#8203;1012](https://redirect.github.com/actions/setup-python/pull/1012)\n- Upgrade `urllib3` from 1.25.9 to 1.26.19 in /**tests**/data by\n[@&#8203;dependabot](https://redirect.github.com/dependabot) in\n[#&#8203;895](https://redirect.github.com/actions/setup-python/pull/895)\n- Upgrade `actions/publish-immutable-action` from 0.0.3 to 0.0.4 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot) in\n[#&#8203;1014](https://redirect.github.com/actions/setup-python/pull/1014)\n- Upgrade `@actions/http-client` from 2.2.1 to 2.2.3 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot) in\n[#&#8203;1020](https://redirect.github.com/actions/setup-python/pull/1020)\n- Upgrade `requests` from 2.24.0 to 2.32.2 in /**tests**/data by\n[@&#8203;dependabot](https://redirect.github.com/dependabot) in\n[#&#8203;1019](https://redirect.github.com/actions/setup-python/pull/1019)\n- Upgrade `@actions/cache` to `^4.0.0` by\n[@&#8203;priyagupta108](https://redirect.github.com/priyagupta108) in\n[#&#8203;1007](https://redirect.github.com/actions/setup-python/pull/1007)\n\n#### New Contributors\n\n- [@&#8203;benwells](https://redirect.github.com/benwells) made their\nfirst contribution in\n[#&#8203;1009](https://redirect.github.com/actions/setup-python/pull/1009)\n-\n[@&#8203;HarithaVattikuti](https://redirect.github.com/HarithaVattikuti)\nmade their first contribution in\n[#&#8203;1008](https://redirect.github.com/actions/setup-python/pull/1008)\n- [@&#8203;lrq3000](https://redirect.github.com/lrq3000) made their\nfirst contribution in\n[#&#8203;645](https://redirect.github.com/actions/setup-python/pull/645)\n\n**Full Changelog**:\n<https://github.com/actions/setup-python/compare/v5...v5.4.0>\n\n###\n[`v5.3.0`](https://redirect.github.com/actions/setup-python/releases/tag/v5.3.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-python/compare/v5.2.0...v5.3.0)\n\n#### What's Changed\n\n- Add workflow file for publishing releases to immutable action package\nby [@&#8203;Jcambass](https://redirect.github.com/Jcambass) in\n[#&#8203;941](https://redirect.github.com/actions/setup-python/pull/941)\n- Upgrade IA publish by\n[@&#8203;Jcambass](https://redirect.github.com/Jcambass) in\n[#&#8203;943](https://redirect.github.com/actions/setup-python/pull/943)\n\n##### Bug Fixes:\n\n- Normalise Line Endings to Ensure Cross-Platform Consistency by\n[@&#8203;priya-kinthali](https://redirect.github.com/priya-kinthali) in\n[#&#8203;938](https://redirect.github.com/actions/setup-python/pull/938)\n- Revise `isGhes` logic by\n[@&#8203;jww3](https://redirect.github.com/jww3) in\n[#&#8203;963](https://redirect.github.com/actions/setup-python/pull/963)\n- Bump pillow from 7.2 to 10.2.0 by\n[@&#8203;aparnajyothi-y](https://redirect.github.com/aparnajyothi-y) in\n[#&#8203;956](https://redirect.github.com/actions/setup-python/pull/956)\n\n##### Enhancements:\n\n- Enhance workflows and documentation updates by\n[@&#8203;priya-kinthali](https://redirect.github.com/priya-kinthali) in\n[#&#8203;965](https://redirect.github.com/actions/setup-python/pull/965)\n- Bump default versions to latest by\n[@&#8203;jeffwidman](https://redirect.github.com/jeffwidman) in\n[#&#8203;905](https://redirect.github.com/actions/setup-python/pull/905)\n\n#### New Contributors\n\n- [@&#8203;Jcambass](https://redirect.github.com/Jcambass) made their\nfirst contribution in\n[#&#8203;941](https://redirect.github.com/actions/setup-python/pull/941)\n- [@&#8203;jww3](https://redirect.github.com/jww3) made their first\ncontribution in\n[#&#8203;963](https://redirect.github.com/actions/setup-python/pull/963)\n\n**Full Changelog**:\n<https://github.com/actions/setup-python/compare/v5...v5.3.0>\n\n###\n[`v5.2.0`](https://redirect.github.com/actions/setup-python/releases/tag/v5.2.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-python/compare/v5.1.1...v5.2.0)\n\n#### What's Changed\n\n##### Bug fixes:\n\n- Add `.zip` extension to Windows package downloads for `Expand-Archive`\nCompatibility by\n[@&#8203;priyagupta108](https://redirect.github.com/priyagupta108) in\n[#&#8203;916](https://redirect.github.com/actions/setup-python/pull/916)\nThis addresses compatibility issues on Windows self-hosted runners by\nensuring that the filenames for Python and PyPy package downloads\nexplicitly include the .zip extension, allowing the Expand-Archive\ncommand to function correctly.\n- Add arch to cache key by\n[@&#8203;Zxilly](https://redirect.github.com/Zxilly) in\n[#&#8203;896](https://redirect.github.com/actions/setup-python/pull/896)\nThis addresses issues with caching by adding the architecture (arch) to\nthe cache key, ensuring that cache keys are accurate to prevent\nconflicts.\nNote: This change may break previous cache keys as they will no longer\nbe compatible with the new format.\n\n##### Documentation changes:\n\n- Fix display of emojis in contributors doc by\n[@&#8203;sciencewhiz](https://redirect.github.com/sciencewhiz) in\n[#&#8203;899](https://redirect.github.com/actions/setup-python/pull/899)\n- Documentation update for caching poetry dependencies by\n[@&#8203;gowridurgad](https://redirect.github.com/gowridurgad) in\n[#&#8203;908](https://redirect.github.com/actions/setup-python/pull/908)\n\n##### Dependency updates:\n\n- Bump [@&#8203;iarna/toml](https://redirect.github.com/iarna/toml)\nversion from 2.2.5 to 3.0.0 by\n[@&#8203;priya-kinthali](https://redirect.github.com/priya-kinthali) in\n[#&#8203;912](https://redirect.github.com/actions/setup-python/pull/912)\n- Bump pyinstaller from 3.6 to 5.13.1 by\n[@&#8203;aparnajyothi-y](https://redirect.github.com/aparnajyothi-y) in\n[#&#8203;923](https://redirect.github.com/actions/setup-python/pull/923)\n\n#### New Contributors\n\n- [@&#8203;sciencewhiz](https://redirect.github.com/sciencewhiz) made\ntheir first contribution in\n[#&#8203;899](https://redirect.github.com/actions/setup-python/pull/899)\n- [@&#8203;priyagupta108](https://redirect.github.com/priyagupta108)\nmade their first contribution in\n[#&#8203;916](https://redirect.github.com/actions/setup-python/pull/916)\n- [@&#8203;Zxilly](https://redirect.github.com/Zxilly) made their first\ncontribution in\n[#&#8203;896](https://redirect.github.com/actions/setup-python/pull/896)\n- [@&#8203;aparnajyothi-y](https://redirect.github.com/aparnajyothi-y)\nmade their first contribution in\n[#&#8203;923](https://redirect.github.com/actions/setup-python/pull/923)\n\n**Full Changelog**:\n<https://github.com/actions/setup-python/compare/v5...v5.2.0>\n\n###\n[`v5.1.1`](https://redirect.github.com/actions/setup-python/releases/tag/v5.1.1)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-python/compare/v5.1.0...v5.1.1)\n\n#### What's Changed\n\n##### Bug fixes:\n\n- fix(ci): update all failing workflows by\n[@&#8203;mayeut](https://redirect.github.com/mayeut) in\n[#&#8203;863](https://redirect.github.com/actions/setup-python/pull/863)\nThis update ensures compatibility and optimal performance of workflows\non the latest macOS version.\n\n##### Documentation changes:\n\n- Documentation update for cache by\n[@&#8203;gowridurgad](https://redirect.github.com/gowridurgad) in\n[#&#8203;873](https://redirect.github.com/actions/setup-python/pull/873)\n\n##### Dependency updates:\n\n- Bump braces from 3.0.2 to 3.0.3 and undici from 5.28.3 to 5.28.4 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot) in\n[#&#8203;893](https://redirect.github.com/actions/setup-python/pull/893)\n\n#### New Contributors\n\n- [@&#8203;gowridurgad](https://redirect.github.com/gowridurgad) made\ntheir first contribution in\n[#&#8203;873](https://redirect.github.com/actions/setup-python/pull/873)\n\n**Full Changelog**:\n<https://github.com/actions/setup-python/compare/v5...v5.1.1>\n\n###\n[`v5.1.0`](https://redirect.github.com/actions/setup-python/releases/tag/v5.1.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-python/compare/v5...v5.1.0)\n\n#### What's Changed\n\n- Leveraging the raw API to retrieve the version-manifest, as it does\nnot impose a rate limit and hence facilitates unrestricted consumption\nwithout the need for a token for Github Enterprise Servers by\n[@&#8203;Shegox](https://redirect.github.com/Shegox) in\n[#&#8203;766](https://redirect.github.com/actions/setup-python/pull/766).\n- Dependency updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot) and\n[@&#8203;HarithaVattikuti](https://redirect.github.com/HarithaVattikuti)\nin\n[#&#8203;817](https://redirect.github.com/actions/setup-python/pull/817)\n- Documentation changes for version in README by\n[@&#8203;basnijholt](https://redirect.github.com/basnijholt) in\n[#&#8203;776](https://redirect.github.com/actions/setup-python/pull/776)\n- Documentation changes for link in README by\n[@&#8203;ukd1](https://redirect.github.com/ukd1) in\n[#&#8203;793](https://redirect.github.com/actions/setup-python/pull/793)\n- Documentation changes for link in Advanced Usage by\n[@&#8203;Jamim](https://redirect.github.com/Jamim) in\n[#&#8203;782](https://redirect.github.com/actions/setup-python/pull/782)\n- Documentation changes for avoiding rate limit issues on GHES by\n[@&#8203;priya-kinthali](https://redirect.github.com/priya-kinthali) in\n[#&#8203;835](https://redirect.github.com/actions/setup-python/pull/835)\n\n#### New Contributors\n\n- [@&#8203;basnijholt](https://redirect.github.com/basnijholt) made\ntheir first contribution in\n[#&#8203;776](https://redirect.github.com/actions/setup-python/pull/776)\n- [@&#8203;ukd1](https://redirect.github.com/ukd1) made their first\ncontribution in\n[#&#8203;793](https://redirect.github.com/actions/setup-python/pull/793)\n- [@&#8203;Jamim](https://redirect.github.com/Jamim) made their first\ncontribution in\n[#&#8203;782](https://redirect.github.com/actions/setup-python/pull/782)\n- [@&#8203;Shegox](https://redirect.github.com/Shegox) made their first\ncontribution in\n[#&#8203;766](https://redirect.github.com/actions/setup-python/pull/766)\n- [@&#8203;priya-kinthali](https://redirect.github.com/priya-kinthali)\nmade their first contribution in\n[#&#8203;835](https://redirect.github.com/actions/setup-python/pull/835)\n\n**Full Changelog**:\n<https://github.com/actions/setup-python/compare/v5.0.0...v5.1.0>\n\n</details>\n\n<details>\n<summary>codecov/codecov-action (codecov/codecov-action)</summary>\n\n###\n[`v5.5.2`](https://redirect.github.com/codecov/codecov-action/blob/HEAD/CHANGELOG.md#v552)\n\n[Compare\nSource](https://redirect.github.com/codecov/codecov-action/compare/v5.5.1...v5.5.2)\n\n##### What's Changed\n\n**Full Changelog**:\n<https://github.com/codecov/codecov-action/compare/v5.5.1..v5.5.2>\n\n</details>\n\n<details>\n<summary>docker/setup-buildx-action\n(docker/setup-buildx-action)</summary>\n\n###\n[`v3.12.0`](https://redirect.github.com/docker/setup-buildx-action/releases/tag/v3.12.0)\n\n[Compare\nSource](https://redirect.github.com/docker/setup-buildx-action/compare/v3.11.1...v3.12.0)\n\n- Deprecate `install` input by\n[@&#8203;crazy-max](https://redirect.github.com/crazy-max) in\n[#&#8203;455](https://redirect.github.com/docker/setup-buildx-action/pull/455)\n- Bump\n[@&#8203;docker/actions-toolkit](https://redirect.github.com/docker/actions-toolkit)\nfrom 0.62.1 to 0.63.0 in\n[#&#8203;434](https://redirect.github.com/docker/setup-buildx-action/pull/434)\n- Bump brace-expansion from 1.1.11 to 1.1.12 in\n[#&#8203;436](https://redirect.github.com/docker/setup-buildx-action/pull/436)\n- Bump form-data from 2.5.1 to 2.5.5 in\n[#&#8203;432](https://redirect.github.com/docker/setup-buildx-action/pull/432)\n- Bump undici from 5.28.4 to 5.29.0 in\n[#&#8203;435](https://redirect.github.com/docker/setup-buildx-action/pull/435)\n\n**Full Changelog**:\n<https://github.com/docker/setup-buildx-action/compare/v3.11.1...v3.12.0>\n\n###\n[`v3.11.1`](https://redirect.github.com/docker/setup-buildx-action/releases/tag/v3.11.1)\n\n[Compare\nSource](https://redirect.github.com/docker/setup-buildx-action/compare/v3.11.0...v3.11.1)\n\n- Fix `keep-state` not being respected by\n[@&#8203;crazy-max](https://redirect.github.com/crazy-max) in\n[#&#8203;429](https://redirect.github.com/docker/setup-buildx-action/pull/429)\n\n**Full Changelog**:\n<https://github.com/docker/setup-buildx-action/compare/v3.11.0...v3.11.1>\n\n###\n[`v3.11.0`](https://redirect.github.com/docker/setup-buildx-action/releases/tag/v3.11.0)\n\n[Compare\nSource](https://redirect.github.com/docker/setup-buildx-action/compare/v3.10.0...v3.11.0)\n\n- Keep BuildKit state support by\n[@&#8203;crazy-max](https://redirect.github.com/crazy-max) in\n[#&#8203;427](https://redirect.github.com/docker/setup-buildx-action/pull/427)\n- Remove aliases created when installing by default by\n[@&#8203;hashhar](https://redirect.github.com/hashhar) in\n[#&#8203;139](https://redirect.github.com/docker/setup-buildx-action/pull/139)\n- Bump\n[@&#8203;docker/actions-toolkit](https://redirect.github.com/docker/actions-toolkit)\nfrom 0.56.0 to 0.62.1 in\n[#&#8203;422](https://redirect.github.com/docker/setup-buildx-action/pull/422)\n[#&#8203;425](https://redirect.github.com/docker/setup-buildx-action/pull/425)\n\n**Full Changelog**:\n<https://github.com/docker/setup-buildx-action/compare/v3.10.0...v3.11.0>\n\n###\n[`v3.10.0`](https://redirect.github.com/docker/setup-buildx-action/releases/tag/v3.10.0)\n\n[Compare\nSource](https://redirect.github.com/docker/setup-buildx-action/compare/v3.9.0...v3.10.0)\n\n- Bump\n[@&#8203;docker/actions-toolkit](https://redirect.github.com/docker/actions-toolkit)\nfrom 0.54.0 to 0.56.0 in\n[#&#8203;408](https://redirect.github.com/docker/setup-buildx-action/pull/408)\n\n**Full Changelog**:\n<https://github.com/docker/setup-buildx-action/compare/v3.9.0...v3.10.0>\n\n###\n[`v3.9.0`](https://redirect.github.com/docker/setup-buildx-action/releases/tag/v3.9.0)\n\n[Compare\nSource](https://redirect.github.com/docker/setup-buildx-action/compare/v3.8.0...v3.9.0)\n\n- Bump\n[@&#8203;docker/actions-toolkit](https://redirect.github.com/docker/actions-toolkit)\nfrom 0.48.0 to 0.54.0 in\n[#&#8203;402](https://redirect.github.com/docker/setup-buildx-action/pull/402)\n[#&#8203;404](https://redirect.github.com/docker/setup-buildx-action/pull/404)\n\n**Full Changelog**:\n<https://github.com/docker/setup-buildx-action/compare/v3.8.0...v3.9.0>\n\n###\n[`v3.8.0`](https://redirect.github.com/docker/setup-buildx-action/releases/tag/v3.8.0)\n\n[Compare\nSource](https://redirect.github.com/docker/setup-buildx-action/compare/v3.7.1...v3.8.0)\n\n- Make cloud prefix optional to download buildx if driver is cloud by\n[@&#8203;crazy-max](https://redirect.github.com/crazy-max) in\n[#&#8203;390](https://redirect.github.com/docker/setup-buildx-action/pull/390)\n- Bump [@&#8203;actions/core](https://redirect.github.com/actions/core)\nfrom 1.10.1 to 1.11.1 in\n[#&#8203;370](https://redirect.github.com/docker/setup-buildx-action/pull/370)\n- Bump\n[@&#8203;docker/actions-toolkit](https://redirect.github.com/docker/actions-toolkit)\nfrom 0.39.0 to 0.48.0 in\n[#&#8203;389](https://redirect.github.com/docker/setup-buildx-action/pull/389)\n- Bump cross-spawn from 7.0.3 to 7.0.6 in\n[#&#8203;382](https://redirect.github.com/docker/setup-buildx-action/pull/382)\n\n**Full Changelog**:\n<https://github.com/docker/setup-buildx-action/compare/v3.7.1...v3.8.0>\n\n</details>\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v4.31.9`](https://redirect.github.com/github/codeql-action/compare/v4.31.8...v4.31.9)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.31.8...v4.31.9)\n\n###\n[`v4.31.8`](https://redirect.github.com/github/codeql-action/releases/tag/v4.31.8)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.31.7...v4.31.8)\n\n##### CodeQL Action Changelog\n\nSee the [releases\npage](https://redirect.github.com/github/codeql-action/releases) for the\nrelevant changes to the CodeQL CLI and language packs.\n\n##### 4.31.8 - 11 Dec 2025\n\n- Update default CodeQL bundle version to 2.23.8.\n[#&#8203;3354](https://redirect.github.com/github/codeql-action/pull/3354)\n\nSee the full\n[CHANGELOG.md](https://redirect.github.com/github/codeql-action/blob/v4.31.8/CHANGELOG.md)\nfor more information.\n\n###\n[`v4.31.7`](https://redirect.github.com/github/codeql-action/releases/tag/v4.31.7)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.31.6...v4.31.7)\n\n##### CodeQL Action Changelog\n\nSee the [releases\npage](https://redirect.github.com/github/codeql-action/releases) for the\nrelevant changes to the CodeQL CLI and language packs.\n\n##### 4.31.7 - 05 Dec 2025\n\n- Update default CodeQL bundle version to 2.23.7.\n[#&#8203;3343](https://redirect.github.com/github/codeql-action/pull/3343)\n\nSee the full\n[CHANGELOG.md](https://redirect.github.com/github/codeql-action/blob/v4.31.7/CHANGELOG.md)\nfor more information.\n\n</details>\n\n<details>\n<summary>actions/go-versions (go)</summary>\n\n###\n[`v1.25.5`](https://redirect.github.com/actions/go-versions/releases/tag/1.25.5-19880500865):\n1.25.5\n\n[Compare\nSource](https://redirect.github.com/actions/go-versions/compare/1.25.4-19122936812...1.25.5-19880500865)\n\nGo 1.25.5\n\n</details>\n\n<details>\n<summary>actions/python-versions (python)</summary>\n\n###\n[`v3.14.2`](https://redirect.github.com/actions/python-versions/releases/tag/3.14.2-20014991423):\n3.14.2\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.14.1-19879739908...3.14.2-20014991423)\n\nPython 3.14.2\n\n###\n[`v3.14.1`](https://redirect.github.com/actions/python-versions/releases/tag/3.14.1-19879739908):\n3.14.1\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.14.0-18313368925...3.14.1-19879739908)\n\nPython 3.14.1\n\n###\n[`v3.14.0`](https://redirect.github.com/actions/python-versions/releases/tag/3.14.0-18313368925):\n3.14.0\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.11-20014977833...3.14.0-18313368925)\n\nPython 3.14.0\n\n###\n[`v3.13.11`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.11-20014977833):\n3.13.11\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.10-19879712315...3.13.11-20014977833)\n\nPython 3.13.11\n\n###\n[`v3.13.10`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.10-19879712315):\n3.13.10\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.9-18515951191...3.13.10-19879712315)\n\nPython 3.13.10\n\n###\n[`v3.13.9`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.9-18515951191):\n3.13.9\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.8-18331000654...3.13.9-18515951191)\n\nPython 3.13.9\n\n###\n[`v3.13.8`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.8-18331000654):\n3.13.8\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.7-16980743123...3.13.8-18331000654)\n\nPython 3.13.8\n\n###\n[`v3.13.7`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.7-16980743123):\n3.13.7\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.6-16792117939...3.13.7-16980743123)\n\nPython 3.13.7\n\n###\n[`v3.13.6`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.6-16792117939):\n3.13.6\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.5-15601068749...3.13.6-16792117939)\n\nPython 3.13.6\n\n###\n[`v3.13.5`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.5-15601068749):\n3.13.5\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.4-15433317575...3.13.5-15601068749)\n\nPython 3.13.5\n\n###\n[`v3.13.4`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.4-15433317575):\n3.13.4\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.3-14344076652...3.13.4-15433317575)\n\nPython 3.13.4\n\n###\n[`v3.13.3`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.3-14344076652):\n3.13.3\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.2-13708744326...3.13.3-14344076652)\n\nPython 3.13.3\n\n###\n[`v3.13.2`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.2-13708744326):\n3.13.2\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.1-13437882550...3.13.2-13708744326)\n\nPython 3.13.2\n\n###\n[`v3.13.1`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.1-13437882550):\n3.13.1\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.0-13707372259...3.13.1-13437882550)\n\nPython 3.13.1\n\n###\n[`v3.13.0`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.0-13707372259):\n3.13.0\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.12-18393146713...3.13.0-13707372259)\n\nPython 3.13.0\n\n###\n[`v3.12.12`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.12-18393146713):\n3.12.12\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.11-15433310049...3.12.12-18393146713)\n\nPython 3.12.12\n\n###\n[`v3.12.11`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.11-15433310049):\n3.12.11\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.10-14343898437...3.12.11-15433310049)\n\nPython 3.12.11\n\n###\n[`v3.12.10`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.10-14343898437):\n3.12.10\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.9-13149478207...3.12.10-14343898437)\n\nPython 3.12.10\n\n###\n[`v3.12.9`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.9-13149478207):\n3.12.9\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.8-12154062663...3.12.9-13149478207)\n\nPython 3.12.9\n\n###\n[`v3.12.8`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.8-12154062663):\n3.12.8\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.7-11128208086...3.12.8-12154062663)\n\nPython 3.12.8\n\n###\n[`v3.12.7`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.7-11128208086):\n3.12.7\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.6-10765725458...3.12.7-11128208086)\n\nPython 3.12.7\n\n###\n[`v3.12.6`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.6-10765725458):\n3.12.6\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.5-10375840348...3.12.6-10765725458)\n\nPython 3.12.6\n\n###\n[`v3.12.5`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.5-10375840348):\n3.12.5\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.4-9947065640...3.12.5-10375840348)\n\nPython 3.12.5\n\n###\n[`v3.12.4`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.4-9947065640):\n3.12.4\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.3-11057844995...3.12.4-9947065640)\n\nPython 3.12.4\n\n###\n[`v3.12.3`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.3-11057844995):\n3.12.3\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.2-11057786931...3.12.3-11057844995)\n\nPython 3.12.3\n\n###\n[`v3.12.2`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.2-11057786931):\n3.12.2\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.1-11057762749...3.12.2-11057786931)\n\nPython 3.12.2\n\n###\n[`v3.12.1`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.1-11057762749):\n3.12.1\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.0-11057302691...3.12.1-11057762749)\n\nPython 3.12.1\n\n###\n[`v3.12.0`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.0-11057302691):\n3.12.0\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.11.14-18393181605...3.12.0-11057302691)\n\nPython 3.12.0\n\n</details>\n\n<details>\n<summary>step-security/harden-runner\n(step-security/harden-runner)</summary>\n\n###\n[`v2.14.0`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.14.0)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.13.3...v2.14.0)\n\n##### What's Changed\n\n- Selective installation: Harden-Runner now skips installation on\nGitHub-hosted runners when the repository has a custom property\nskip\\_harden\\_runner, allowing organizations to opt out specific repos.\n- Avoid double install: The action no longer installs Harden-Runner if\nit’s already present on a GitHub-hosted runner, which could happen when\na composite action also installs it.\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.13.3...v2.14.0>\n\n###\n[`v2.13.3`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.13.3)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.13.2...v2.13.3)\n\n##### What's Changed\n\n- Fixed an issue where process events were not uploaded in certain edge\ncases.\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.13.2...v2.13.3>\n\n</details>\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.65.13`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.65.12...v2.65.13)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.13...HEAD\n\n[2.65.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.12...v2.65.13\n\n[2.65.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.11...v2.65.12\n\n[2.65.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.10...v2.65.11\n\n[2.65.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.9...v2.65.10\n\n[2.65.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.8...v2.65.9\n\n[2.65.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.7...v2.65.8\n\n[2.65.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.6...v2.65.7\n\n[2.65.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.5...v2.65.6\n\n[2.65.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.4...v2.65.5\n\n[2.65.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.3...v2.65.4\n\n[2.65.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.2...v2.65.3\n\n[2.65.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.1...v2.65.2\n\n[2.65.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.0...v2.65.1\n\n[2.65.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.2...v2.65.0\n\n[2.64.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.1...v2.64.2\n\n[2.64.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.0...v2.64.1\n\n[2.64.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.3...v2.64.0\n\n[2.63.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.2...v2.63.3\n\n[2.63.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.1...v2.63.2\n\n[2.63.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.0...v2.63.1\n\n[2.63.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.67...v2.63.0\n\n[2.62.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.66...v2.62.67\n\n[2.62.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.65...v2.62.66\n\n[2.62.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.64...v2.62.65\n\n[2.62.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.63...v2.62.64\n\n[2.62.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.62...v2.62.63\n\n[2.62.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.61...v2.62.62\n\n[2.62.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.60...v2.62.61\n\n[2.62.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.59...v2.62.60\n\n[2.62.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.58...v2.62.59\n\n[2.62.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.57...v2.62.58\n\n[2.62.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.56...v2.62.57\n\n[2.62.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.55...v2.62.56\n\n[2.62.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.54...v2.62.55\n\n[2.62.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.53...v2.62.54\n\n[2.62.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.52...v2.62.53\n\n[2.62.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.51...v2.62.52\n\n[2.62.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.50...v2.62.51\n\n[2.62.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.49...v2.62.50\n\n[2.62.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.48...v2.62.49\n\n[2.62.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.47...v2.62.48\n\n[2.62.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.46...v2.62.47\n\n[2.62.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.45...v2.62.46\n\n[2.62.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.44...v2.62.45\n\n[2.62.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.43...v2.62.44\n\n[2.62.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.42...v2.62.43\n\n[2.62.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.41...v2.62.42\n\n[2.62.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.40...v2.62.41\n\n[2.62.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.39...v2.62.40\n\n[2.62.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.38...v2.62.39\n\n[2.62.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.37...v2.62.38\n\n[2.62.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.36...v2.62.37\n\n[2.62.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.35...v2.62.36\n\n[2.62.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.34...v2.62.35\n\n[2.62.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.33...v2.62.34\n\n[2.62.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.32...v2.62.33\n\n[2.62.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.31...v2.62.32\n\n[2.62.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.30...v2.62.31\n\n[2.62.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.29...v2.62.30\n\n[2.62.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.28...v2.62.29\n\n[2.62.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.27...v2.62.28\n\n[2.62.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.26...v2.62.27\n\n[2.62.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.25...v2.62.26\n\n[2.62.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.24...v2.62.25\n\n[2.62.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.23...v2.62.24\n\n[2.62.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.22...v2.62.23\n\n[2.62.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.21...v2.62.22\n\n[2.62.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.20...v2.62.21\n\n[2.62.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.19...v2.62.20\n\n[2.62.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.18...v2.62.19\n\n[2.62.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.17...v2.62.18\n\n[2.62.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.16...v2.62.17\n\n[2.62.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.15...v2.62.16\n\n[2.62.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.14...v2.62.15\n\n[2.62.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.13...v2.62.14\n\n[2.62.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.12...v2.62.13\n\n[2.62.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.11...v2.62.12\n\n[2.62.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.10...v2.62.11\n\n[2.62.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.9...v2.62.10\n\n[2.62.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.8...v2.62.9\n\n[2.62.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.7...v2.62.8\n\n[2.62.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.6...v2.62.7\n\n[2.62.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.5...v2.62.6\n\n[2.62.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.4...v2.62.5\n\n[2.62.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.3...v2.62.4\n\n[2.62.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.2...v2.62.3\n\n[2.62.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.1...v2.62.2\n\n[2.62.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.0...v2.62.1\n\n[2.62.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.13...v2.62.0\n\n[2.61.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.12...v2.61.13\n\n[2.61.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.11...v2.61.12\n\n[2.61.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.10...v2.61.11\n\n[2.61.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.9...v2.61.10\n\n[2.61.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.8...v2.61.9\n\n[2.61.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.7...v2.61.8\n\n[2.61.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.6...v2.61.7\n\n[2.61.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.5...v2.61.6\n\n[2.61.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.4...v2.61.5\n\n[2.61.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.3...v2.61.4\n\n[2.61.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.2...v2.61.3\n\n[2.61.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.1...v2.61.2\n\n[2.61.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.0...v2.61.1\n\n[2.61.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.60.0...v2.61.0\n\n[2.60.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.1...v2.60.0\n\n[2.59.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.0...v2.59.1\n\n[2.59.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.33...v2.59.0\n\n[2.58.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.32...v2.58.33\n\n[2.58.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.31...v2.58.32\n\n[2.58.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.30...v2.58.31\n\n[2.58.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.29...v2.58.30\n\n[2.58.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.28...v2.58.29\n\n[2.58.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.27...v2.58.28\n\n[2.58.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.26...v2.58.27\n\n[2.58.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.25...v2.58.26\n\n[2.58.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.24...v2.58.25\n\n[2.58.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.23...v2.58.24\n\n[2.58.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.22...v2.58.23\n\n[2.58.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.21...v2.58.22\n\n[2.58.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.20...v2.58.21\n\n[2.58.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.19...v2.58.20\n\n[2.58.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.18...v2.58.19\n\n[2.58.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.17...v2.58.18\n\n[2.58.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.16...v2.58.17\n\n[2.58.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.15...v2.58.16\n\n[2.58.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.14...v2.58.15\n\n[2.58.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.13...v2.58.14\n\n[2.58.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.12...v2.58.13\n\n[2.58.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.11...v2.58.12\n\n[2.58.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.10...v2.58.11\n\n[2.58.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.9...v2.58.10\n\n[2.58.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.8...v2.58.9\n\n[2.58.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.7...v2.58.8\n\n[2.58.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.6...v2.58.7\n\n[2.58.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.5...v2.58.6\n\n[2.58.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.4...v2.58.5\n\n[2.58.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.3...v2.58.4\n\n[2.58.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.2...v2.58.3\n\n[2.58.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.1...v2.58.2\n\n[2.58.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.0...v2.58.1\n\n[2.58.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.8...v2.58.0\n\n[2.57.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.7...v2.57.8\n\n[2.57.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.6...v2.57.7\n\n[2.57.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.5...v2.57.6\n\n[2.57.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.4...v2.57.5\n\n[2.57.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.3...v2.57.4\n\n[2.57.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.2...v2.57.3\n\n[2.57.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.1...v2.57.2\n\n[2.57.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.0...v2.57.1\n\n[2.57.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.24...v2.57.0\n\n[2.56.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.23...v2.56.24\n\n[2.56.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.22...v2.56.23\n\n[2.56.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.21...v2.56.22\n\n[2.56.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.20...v2.56.21\n\n[2.56.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.19...v2.56.20\n\n[2.56.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.18...v2.56.19\n\n[2.56.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.17...v2.56.18\n\n[2.56.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.16...v2.56.17\n\n[2.56.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.15...v2.56.16\n\n[2.56.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.14...v2.56.15\n\n[2.56.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.13...v2.56.14\n\n[2.56.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.12...v2.56.13\n\n[2.56.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.11...v2.56.12\n\n[2.56.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.10...v2.56.11\n\n[2.56.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.9...v2.56.10\n\n[2.56.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.8...v2.56.9\n\n[2.56.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.7...v2.56.8\n\n[2.56.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.6...v2.56.7\n\n[2.56.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.5...v2.56.6\n\n[2.56.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.4...v2.56.5\n\n[2.56.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.3...v2.56.4\n\n[2.56.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.2...v2.56.3\n\n[2.56.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.1...v2.56.2\n\n[2.56.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.0...v2.56.1\n\n[2.56.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.4...v2.56.0\n\n[2.55.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.3...v2.55.4\n\n[2.55.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.2...v2.55.3\n\n[2.55.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.1...v2.55.2\n\n[2.55.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.0...v2.55.1\n\n[2.55.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.3...v2.55.0\n\n[2.54.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.2...v2.54.3\n\n[2.54.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.1...v2.54.2\n\n[2.54.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.0...v2.54.1\n\n[2.54.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.2...v2.54.0\n\n[2.53.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.1...v2.53.2\n\n[2.53.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.0...v2.53.1\n\n[2.53.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.8...v2.53.0\n\n[2.52.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.7...v2.52.8\n\n[2.52.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.6...v2.52.7\n\n[2.52.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.5...v2.52.6\n\n[2.52.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.4...v2.52.5\n\n[2.52.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.3...v2.52.4\n\n[2.52.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.2...v2.52.3\n\n[2.52.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.1...v2.52.2\n\n[2.52.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.0...v2.52.1\n\n[2.52.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.3...v2.52.0\n\n[2.51.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.2...v2.51.3\n\n[2.51.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.1...v2.51.2\n\n[2.51.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.0...v2.51.1\n\n[2.51.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.10...v2.51.0\n\n[2.50.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.9...v2.50.10\n\n[2.50.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.8...v2.50.9\n\n[2.50.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.7...v2.50.8\n\n[2.50.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.6...v2.50.7\n\n[2.50.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.5...v2.50.6\n\n[2.50.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.4...v2.50.5\n\n[2.50.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.3...v2.50.4\n\n[2.50.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.2...v2.50.3\n\n[2.50.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.1...v2.50.2\n\n[2.50.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.0...v2.50.1\n\n[2.50.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.50...v2.50.0\n\n[2.49.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.49...v2.49.50\n\n[2.49.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.48...v2.49.49\n\n[2.49.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.47...v2.49.48\n\n[2.49.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.46...v2.49.47\n\n[2.49.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.45...v2.49.46\n\n[2.49.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.44...v2.49.45\n\n[2.49.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.43...v2.49.44\n\n[2.49.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.42...v2.49.43\n\n[2.49.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.41...v2.49.42\n\n[2.49.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.40...v2.49.41\n\n[2.49.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.39...v2.49.40\n\n[2.49.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.38...v2.49.39\n\n[2.49.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.37...v2.49.38\n\n[2.49.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.36...v2.49.37\n\n[2.49.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.35...v2.49.36\n\n[2.49.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.34...v2.49.35\n\n[2.49.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.33...v2.49.34\n\n[2.49.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.32...v2.49.33\n\n[2.49.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.31...v2.49.32\n\n[2.49.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.30...v2.49.31\n\n[2.49.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.29...v2.49.30\n\n[2.49.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.28...v2.49.29\n\n[2.49.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.27...v2.49.28\n\n[2.49.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.26...v2.49.27\n\n[2.49.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.25...v2.49.26\n\n[2.49.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.24...v2.49.25\n\n[2.49.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.23...v2.49.24\n\n[2.49.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.22...v2.49.23\n\n[2.49.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.21...v2.49.22\n\n[2.49.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.20...v2.49.21\n\n[2.49.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.19...v2.49.20\n\n[2.49.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.18...v2.49.19\n\n[2.49.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.17...v2.49.18\n\n[2.49.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.16...v2.49.17\n\n[2.49.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.15...v2.49.16\n\n[2.49.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.14...v2.49.15\n\n[2.49.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.13...v2.49.14\n\n[2.49.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.12...v2.49.13\n\n[2.49.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.11...v2.49.12\n\n[2.49.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.10...v2.49.11\n\n[2.49.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.9...v2.49.10\n\n[2.49.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.8...v2.49.9\n\n[2.49.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.7...v2.49.8\n\n[2.49.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.6...v2.49.7\n\n[2.49.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.5...v2.49.6\n\n[2.49.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.4...v2.49.5\n\n[2.49.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.3...v2.49.4\n\n[2.49.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.2...v2.49.3\n\n[2.49.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.1...v2.49.2\n\n[2.49.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.0...v2.49.1\n\n[2.49.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.22...v2.4\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on the first day of the\nmonth\" (UTC), Automerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi42OS4xIiwidXBkYXRlZEluVmVyIjoiNDIuNjkuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-01-05T18:10:24Z",
          "tree_id": "f22444dc95bfdf00a36aa4a84b00e7b47b25894b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/813c4858716e4cdd27ef86f4c9a8fedda6f5c0bd"
        },
        "date": 1767639887558,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 1085000,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.688920259475708,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 272.00115406613116,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 272.6859194085028,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 68.00028851653279,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 68.1714798521257,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 71.04895833333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 73.875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 29412400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 28327400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 490195.6372648282,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 472112.7107973404,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00135,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13619381.998943528,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12991024.699484965,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 4620000,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.594564437866211,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 1110.6102741545071,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 1122.0264449853644,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.4131421346567,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 70.12665281158527,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 250.66744791666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 264.23046875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 100553600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 95933600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1675532.7866032028,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1598549.3521552386,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.012911,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 47274982.47519295,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 45084369.54977305,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 51800,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.284956693649292,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 170.93921027936923,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 171.54447889655174,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.46960513968462,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.77223944827587,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.235286458333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.90625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 18178200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 18126400,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 302964.07705229364,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 302100.760596797,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001173,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 8569028.682572199,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8174647.155310312,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1281800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.780950665473938,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 688.3673115260456,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 690.3667115573835,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.0459139407557,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.29583894467294,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 146.65703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 152.10546875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 71972800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 70691000,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1199502.7248835117,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1178140.174131621,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002198,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 34176102.4847086,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32640381.471844587,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 2600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.02294873632490635,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.51070016374571,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.65649105037764,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.51070016374571,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.65649105037764,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.03294270833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.984375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11329600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11327000,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 188824.12383513237,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 188780.79108534672,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000808,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5352499.188403754,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5110865.874805196,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "198982749+Copilot@users.noreply.github.com",
            "name": "Copilot",
            "username": "Copilot"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "31631de9800d7427d9bd40723a07015e80c2d7b6",
          "message": "Add validation for overlapping core ranges in CoreSet allocations (#1707)\n\n## Plan for Improving Core-ID Validation\n\n- [x] Understand the issue: Add validation to check for overlapping\ncores in CoreSet allocations\n- [x] Review existing validation logic in `select_cores_for_quota`\n- [x] Add validation logic to detect overlapping core ranges\n- [x] Add test case for overlapping ranges (e.g., [2-5, 4-7])\n- [x] Add test case for multiple overlapping ranges\n- [x] Add test case for adjacent but non-overlapping ranges (should\npass)\n- [x] Add test case for fully overlapping ranges\n- [x] Add test case for identical ranges\n- [x] Run tests to verify the changes work correctly\n- [x] Run broader test suite to ensure no regressions\n- [x] Address code review feedback (improve comment clarity and extract\noverlap calculation)\n- [x] Run cargo fmt to format code\n- [x] Final validation - all tests pass\n\n## Summary\n\nThis PR successfully implements overlapping core range detection in the\n`select_cores_for_quota` function. The implementation:\n\n1. Uses a pairwise comparison algorithm (O(n²)) to check all range pairs\n2. Correctly detects overlaps using the standard interval overlap\ncondition: `r1.start <= r2.end && r2.start <= r1.end`\n3. Provides clear error messages showing which ranges overlap and the\nspecific cores that are shared\n4. Maintains backward compatibility with all existing tests passing\n5. Includes comprehensive test coverage for various overlap scenarios\n6. Code formatted with `cargo fmt` for consistency\n\nThe changes are minimal, focused, and surgical - only adding the\nnecessary validation logic without modifying any other functionality.\n\n<!-- START COPILOT ORIGINAL PROMPT -->\n\n\n\n<details>\n\n<summary>Original prompt</summary>\n\n> \n> ----\n> \n> *This section details on the original issue you should resolve*\n> \n> <issue_title>Improve core-id validation to check for overlapping\ncores</issue_title>\n> <issue_description>Follow up to\nhttps://github.com/open-telemetry/otel-arrow/pull/1652#pullrequestreview-3595151623</issue_description>\n> \n> ## Comments on the Issue (you are @copilot in this section)\n> \n> <comments>\n> <comment_new><author>@drewrelmas</author><body>\n> Assigning Copilot for this issue as requested to see if it functions\nwell.</body></comment_new>\n> </comments>\n> \n\n\n</details>\n\n\n\n<!-- START COPILOT CODING AGENT SUFFIX -->\n\n- Fixes open-telemetry/otel-arrow#1658\n\n<!-- START COPILOT CODING AGENT TIPS -->\n---\n\n💡 You can make Copilot smarter by setting up custom instructions,\ncustomizing its development environment and configuring Model Context\nProtocol (MCP) servers. Learn more [Copilot coding agent\ntips](https://gh.io/copilot-coding-agent-tips) in the docs.\n\n---------\n\nCo-authored-by: copilot-swe-agent[bot] <198982749+Copilot@users.noreply.github.com>\nCo-authored-by: drewrelmas <13971805+drewrelmas@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-01-05T18:12:38Z",
          "tree_id": "52b8e0fb0cd075fe86c081b8025464758c695143",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/31631de9800d7427d9bd40723a07015e80c2d7b6"
        },
        "date": 1767641029930,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 536600,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.4695974588394165,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 345.5937559519979,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 346.27943949670095,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.39843898799947,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.56985987417524,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 82.90768229166666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 86.23828125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 36513400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 35976800,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 608543.5931218078,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 599600.451916958,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001289,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17195135.64134717,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16414381.412108192,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 5901400,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 6.693995952606201,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 924.4518194204328,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 933.8116864178068,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 57.77823871377705,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.36323040111292,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 256.4165364583333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 272.64453125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 88159600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 82258200,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1469174.2153555867,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1370827.7537734169,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006226,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 40401876.33524969,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 38448040.8232604,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 117800,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.6390990018844604,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 170.94037558797376,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 171.38390539795054,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.47018779398688,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.69195269897527,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.111328125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.80078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 18432200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 18314400,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 307198.28504151583,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 305234.9839717634,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000986,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 8644044.279149227,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8241380.425171656,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 2093600,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.7000644207000732,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 531.3398801465692,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 534.8345092923078,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 66.41748501832114,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 66.85431366153847,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 123.48489583333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 128.61328125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 56582800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 54489200,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 943019.4134056192,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 908127.0884604768,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001734,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26556353.16403554,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 25335536.278378494,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 164600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.45888352394104,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.56628459359442,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.6821892410368,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.56628459359442,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.6821892410368,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.356640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.8203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11282600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11118000,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 188040.8417921796,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 185297.54480753132,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000795,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5335826.7771294555,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5095423.033271569,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "7d3f2bf5a1791f6e22fdfa7853dd50c23e62b846",
          "message": "Improve performance producing stream per batch `BatchArrowRecord`s (#1718)\n\ncloses #1669 \n\nIt's conceivable that there would be a need for an exporter that will\nproduce serialized OTAP batches that may be consumed opportunistically\nby downstream consumers which do not coordinate to ensure each consumer\nis reading from a single Arrow IPC stream.\n\nExample:\n```\n    exporter ───────> queue ─┰───> consumer1 (pulls next BAR)\n(produces BARs)              └───> consumer2 (pulls next BAR)\n```\n\nWithin each `BatchArrowRecord` (BAR),\n`otap_df_pdata::Producer::serialize_batch` will produce a single Arrow\nIPC stream for each payload type. This means that if `consumer1`\nreceives the first BAR, and `consumer2` receives the 2nd BAR, the second\nBAR will only data messages in the Arrow IPC stream, but the stream\nheader (which contains the schema) will be missing. This will cause an\nerror when consuming the BAR.\n\nOne solution is to create a new Arrow IPC stream for every batch.\nCurrently, doing this would mean creating a new `Producer` for every\nbatch. There's some overhead in doing this because the `Producer`s\ncontain a `HashMap` of `arrow::ipc::StreamWriters`, which contain a ztd\ncontext, all of which needs to be reinitialized.\n\nThis PR adds a method called `reset_streams` to the `Producer`. Calling\nthis will mean the next serialized batch begins a new Arrow IPC stream\nfor each payload type. This can be called after each batch to create a\nnew Arrow IPC stream per batch, with less overhead than creating new\n`Producer`s.\n\n---\n\nThis PR also moves a few test utility functions for creating\n`OtelArrowRecord`s from proto structs from the query-engine crate into\nthe pdata crate.",
          "timestamp": "2026-01-05T18:18:46Z",
          "tree_id": "406828e90f7f80cb8edb8605a3ae20f442cc73c3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7d3f2bf5a1791f6e22fdfa7853dd50c23e62b846"
        },
        "date": 1767642170758,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 802400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.5452492237091064,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 294.5029186234578,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 294.9301770707149,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 73.62572965586445,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 73.73254426767872,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 70.20130208333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 75.18359375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 31525400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 30723000,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 525397.0722363393,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 512024.40731337445,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002999,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14668141.8575652,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13995650.18805058,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 4747800,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.818604469299316,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 1066.3092455373703,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 1075.3532893232618,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 66.64432784608564,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 67.20958058270386,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 267.4526041666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 282.35546875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 98530600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 93782800,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1642114.9778806642,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1562987.9504193293,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002254,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 46159277.35207324,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 43991318.436643206,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 337800,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.8379472494125366,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 170.3471436345112,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 171.10420550142385,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.1735718172556,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.55210275071192,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.447265625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.30078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 18379200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 18041400,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 306315.9872605669,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 300686.0610126007,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000786,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 8685349.018184658,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8277952.329127373,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1086600,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.4681243896484375,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 697.6462473555623,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 702.6181819628564,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 87.20578091944529,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.82727274535705,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 138.68072916666668,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 146.265625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 74012800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 72926200,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1233488.6104694006,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1215379.4628066174,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002824,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 35261884.334646046,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33662087.013681605,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.46818874476116,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.64553013009007,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.46818874476116,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.64553013009007,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.885807291666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.78125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11283600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11283600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 188056.90019542846,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 188056.90019542846,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000989,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5383480.874417347,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5142496.418729033,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lalit_fin@yahoo.com",
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "5c6e628a3e1eec4d4efeef1d41c350b0ec93f6a0",
          "message": "feat: Add HTTP proxy support for OTLP/OTAP gRPC exporters (#1679)\n\nfixes: #1678\n\n  ## Summary\n\n  Implements HTTP CONNECT proxy support for OTLP/OTAP gRPC exporters:\n\n- `HTTP/1.1 CONNECT` tunneling to the proxy for both `http://` targets\n(gRPC over h2c inside the tunnel) and `https://` targets (TLS inside the\ntunnel).\n- Environment variable support (`HTTP_PROXY`, `HTTPS_PROXY`, `NO_PROXY`,\n`ALL_PROXY`)\n  - NO_PROXY with CIDR notation (e.g., `192.168.0.0/16`, `10.0.0.0/8`)\n- Configures TCP socket options (`TCP_NODELAY` + `keepalive`). When\nusing a proxy, these are applied to the TCP connection to the proxy (the\nsame connection that carries the CONNECT tunnel).\n- Explicit rejection of `https://` proxy URLs with helpful error\nmessages\n\n  ## Why OTLP/OTAP exporters need custom proxy implementation\n\n**Azure Monitor and Geneva exporters** use the `reqwest` HTTP client,\nwhich provides built-in proxy support via `reqwest::Proxy::all()` - no\ncustom code needed.\n\n  **OTLP/OTAP exporters** use gRPC (`tonic`), which:\n  - Has no built-in proxy support\n  - Requires custom TCP connectors with `tower::service_fn`\n  - Needs manual HTTP CONNECT tunnel implementation for proxy traversal\n\nThis PR implements the missing proxy infrastructure for gRPC-based\nexporters.\n\n  ## Changes\n\n- `proxy.rs`: HTTP/1.1 CONNECT tunnel implementation with socket option\nhandling\n  - `client_settings.rs`: Integration with tonic via custom connector\n  - Dependencies: Added `socket2` and `ipnet` crates\n\n\n## How HTTP CONNECT Tunneling Works\n\n**Step 1: THE HANDSHAKE**\nThe Exporter creates a TCP connection to the Proxy and sends a plaintext\nrequest. The Proxy establishes a TCP leg to the Backend.\n\n```\n┌───────────┐  TCP + HTTP/1.1 CONNECT backend:PORT       ┌───────────┐\n│ Exporter  │ ─────────────────────────────────────────>.│   Proxy   │\n└───────────┘ <─────── 200 Connection Established ────── └───────────┘\n```\n\n**Step 2: THE DATA TUNNEL**\n\nOnce the 200 is received, the Exporter uses the socket for the actual\nOTLP data. The Proxy merely moves bytes back and forth.\n\n```\n┌───────────┐       ┌───────────┐       ┌───────────────┐\n│ OTLP/OTAP │  TCP  │   Proxy   │  TCP  │    Backend    │\n│ Exporter  │══════>│ (relays)  │══════>│               │\n└───────────┘       └───────────┘       └───────────────┘\n          ║                                     ║\n          ╚═════════════════════════════════════╝\n     On-the-wire protocol inside the tunnel (opaque to proxy):\n     - Case 1 (TLS): TLS + gRPC over HTTP/2 (HTTP/2 negotiated via ALPN)\n     - Case 2 (h2c): gRPC over HTTP/2 cleartext (HTTP/2 prior knowledge)\n```\n\n## Test Setup:\n\nThe implementation was verified via a manual end-to-end test setup (not\nincluded in the PR). The architecture below was used to validate proxy\ntraversal for both h2c and TLS traffic:\n\n  ```\n  ┌─────────────────┐\n  │ Fake Data Gen   │ Generates test telemetry (logs/traces/metrics)\n  └────────┬────────┘\n           │\n           ▼\n  ┌─────────────────┐\n  │ OTAP Exporter   │ gRPC client with proxy configuration\n  │ (df_engine)     │ • HTTP_PROXY=http://localhost:8080\n  │                 │ • grpc_endpoint=http://remote:4317\n│ │ • admin_port: 8081 (changed to avoid proxy conflict)\n  └────────┬────────┘\n           │\n           │ HTTP CONNECT tunneling\n           ▼\n  ┌─────────────────┐\n  │   mitmproxy     │ Intercepts and logs proxy traffic\n  │   :8080         │ (validates CONNECT requests are working)\n  └────────┬────────┘\n           │\n           ▼\n  ┌─────────────────┐\n  │ OTel Collector  │ Receives telemetry via gRPC\n  │   :4317         │\n  └────────┬────────┘\n           │\n           ▼\n  ┌─────────────────┐\n  │ Debug Exporter  │ Displays received data\n  └─────────────────┘\n  ```\n\n---------\n\nCo-authored-by: Copilot <175728472+Copilot@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-05T18:25:14Z",
          "tree_id": "0ff9ab21f7d7f058aa59e1ee6026389a50345cd3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5c6e628a3e1eec4d4efeef1d41c350b0ec93f6a0"
        },
        "date": 1767643306799,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 585600,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6010323762893677,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 343.1398824424873,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 347.32176643352335,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.78497061062183,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.83044160838084,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 84.07760416666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 86.08203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 36576400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 35990800,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 609540.0033083048,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 599781.0706102443,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006562,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17199888.323165458,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16416698.077056173,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6337600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 7.155502796173096,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 913.9922889744582,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 924.026192368869,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 57.12451806090364,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 57.751637023054315,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 243.33776041666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 254.67578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 88569600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 82232000,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1476102.3336021672,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1370479.7932560768,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002344,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 41047347.81295996,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 39115201.32715387,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 196.3726613393515,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 196.68031668291172,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.18633066967575,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.34015834145586,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 51.323567708333336,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.85546875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 21201200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 21201200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 353346.0602935923,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 353346.0602935923,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001235,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10017901.800775718,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9560192.450173145,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1417000,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.0090513229370117,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 666.6084171434886,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 670.7404125487786,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.32605214293608,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 83.84255156859733,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 138.59856770833332,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 144.4296875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 70530800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 69113800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1175450.9560692646,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1151835.542593873,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003184,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33422345.46327926,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 31910918.62806871,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 167600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.4819267988204956,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.5366689528512,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.6561409817845,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.5366689528512,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.6561409817845,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.14622395833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.8515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11309600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11142000,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 188490.84839564865,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 185697.55188727428,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000791,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5395591.664123541,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5152214.713832705,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3bfb8a228645d7465d76ff5a6f24a0738b32be55",
          "message": "chore(deps): upgrade reqwest to 0.13.1 (#1717)\n\nsupersedes https://github.com/open-telemetry/otel-arrow/pull/1713 and\ncorrects the feature name.\n\nThis `rustls-tls` feature was renamed `rustls` in the 0.13 release of\nthis crate. See\nhttps://github.com/seanmonstar/reqwest/releases/tag/v0.13.0\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-05T18:34:36Z",
          "tree_id": "ceaf0ce28a425c7e49904e737cfa99aba3d29638",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3bfb8a228645d7465d76ff5a6f24a0738b32be55"
        },
        "date": 1767644442571,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 673600,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.8554736375808716,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 345.4278014769689,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 347.05501305080816,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.35695036924223,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.76375326270204,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 80.28111979166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 83.06640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 36303400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 35629800,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 605039.6247172371,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 593813.2742594416,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00169,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17235266.861730628,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16454709.000376362,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 5142600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.794961929321289,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 959.930950725116,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 971.7724206785934,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 59.99568442031975,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 60.73577629241209,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 274.05104166666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 287.57421875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 88742600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 83600000,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1478840.5103573378,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1393142.2638718432,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008229,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 41248700.66191925,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 39249198.9776142,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 396000,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.3855135440826416,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 151.6383131047328,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 151.9922003461063,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 75.8191565523664,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 75.99610017305315,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 51.84153645833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.140625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 16600200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 16204200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 276664.93242065446,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 270065.0533084402,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001099,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7735804.33591329,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7370569.757047448,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1219800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6883559226989746,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 689.2965047673342,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 690.8319823116823,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.16206309591678,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.35399778896029,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 143.38046875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 147.8359375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 72247800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 71028000,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1204087.375306914,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1183758.0949634383,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002124,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 34598962.22208204,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33041725.808124192,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.5422478194251,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.71715356367834,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.5422478194251,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.71715356367834,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.52721354166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.62109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11377600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11377600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 189622.8647282289,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 189622.8647282289,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001203,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5423265.776895878,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5176681.949382668,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e0050e4ab0862952d06e0836125512ee4e43cb77",
          "message": "Use python 3.11 in perf run (#1723)\n\nPerf test run on GH runners were broken by\nhttps://github.com/open-telemetry/otel-arrow/pull/1709 as it bumped\npython version to 3.14. This PR reverts it back to 3.11\n(Dedicated runners used 3.11 and has no change)",
          "timestamp": "2026-01-06T00:30:00Z",
          "tree_id": "629a9b39be2641b13a8492b28485dc8dcf4127df",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e0050e4ab0862952d06e0836125512ee4e43cb77"
        },
        "date": 1767661117349,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 369600,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.9970487952232361,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 351.32786531760877,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 352.058330990099,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 87.83196632940219,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.01458274752476,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 79.83528645833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 81.58203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 37069400,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 36699800,
            "unit": "count",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 617813.9837483793,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 611654.0769683019,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000908,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17469230.29080293,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16678021.881831842,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 4579400,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.580517292022705,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 1089.3637154627147,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 1097.2104854299205,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 68.08523221641967,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 68.57565533937003,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 257.1372395833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 264.66796875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 99975600,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 95396200,
            "unit": "count",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666124.0998109253,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1589806.9914097341,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004894,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 46731570.65462611,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 44540272.79297637,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 298800,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.5975021123886108,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 170.32962156868138,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 170.78939483572418,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.16481078434069,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.39469741786209,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.975390625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.09375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 18704200,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 18405400,
            "unit": "count",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 311730.4320580255,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 306750.53165603353,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0012,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 8773760.769005727,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8368577.568061831,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1758800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.665421962738037,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 621.053554020821,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 624.3462091751063,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.63169425260263,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 78.04327614688829,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 148.99205729166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 156.36328125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 65985800,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 64227000,
            "unit": "count",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 1099730.194796797,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1070417.7447452918,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001808,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 31121909.34382625,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 29693272.686856195,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 98.55477066921401,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 98.71064944784325,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.55477066921401,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.71064944784325,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.629947916666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.88671875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 11459600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 11459600,
            "unit": "count",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 190990.05785384114,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 190990.05785384114,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001029,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5421408.390264867,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5177740.284040955,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1367edbd33a199d21ce9caeb6efc5615bd1e068c",
          "message": "[PerfTest] - remove less relevant metrics from benchmark results (#1721)\n\nRemoving few metrics that I think are not very relevant.\n1. Cpu usage - we have Cpu usage that is normalized (0-100 always) as\nalternative.\n2. Total count of logs (produced/lost/received) - we have Log/sec and\ntest duration. And lost percentage.\n\nThe metrics are still collected, just not stored and rendered in the\nbenchmarks in pages like\nhttps://open-telemetry.github.io/otel-arrow/benchmarks/continuous-saturation/\nIf needed, they can still be found in the console output.",
          "timestamp": "2026-01-06T00:30:15Z",
          "tree_id": "34ba2bb7fb8fc1be14a140908c9151223a876b95",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1367edbd33a199d21ce9caeb6efc5615bd1e068c"
        },
        "date": 1767662033570,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.1970582008361816,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.83137323141979,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.09948144459052,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 82.453125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 86.06640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 605483.0427366159,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 592180.2284347538,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002341,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17125626.035359163,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16340476.675850375,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.5055558681488037,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 71.53070941705583,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 72.11780185369591,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 267.36549479166666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 281.8125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1736299.3796557356,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1675432.4334403796,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009582,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 48980515.423835225,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 46696736.67012475,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.7567228078842163,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.37064351994438,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.58285148982445,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 51.48046875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.421875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 305298.733499082,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 299935.4809720867,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000904,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 8671932.808306063,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8266815.839381799,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.2555456161499023,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 82.82253399647671,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 83.06424272108843,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 130.91940104166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 139.97265625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1154026.6839672008,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1139537.3533742742,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002772,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33109642.331400573,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 31634650.447454985,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.05850233882665634,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.53140928687016,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.65864376055139,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.667578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.75390625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 188023.72722906433,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 187913.7289487041,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000938,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5372400.399928446,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5128955.402246484,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jmacd@users.noreply.github.com",
            "name": "Joshua MacDonald",
            "username": "jmacd"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4bf9bf85b4d0c2e7992e18e3dedda40427aa2485",
          "message": "[batch_processor] Support bytes-based batching via new `format = [otap|otlp|preserve]` (#1633)\n\nFixes #1570.\n\nAdds dual format configuration to batch processor, with separate\n`FormatConfig` structs for each payload format.\nThis supports forcing payload into one or the other format, or allowing\nboth to be preserved.\n\nThe new bytes-based batching routines operate by scanning through\ntop-level fields. Unlike the items-based batching mode, this may produce\nbatches that are less than the limit; like that mode, it can also\nproduce outputs greater than the limit.",
          "timestamp": "2026-01-06T01:10:06Z",
          "tree_id": "9ac74c28f618d39535e18ed3fa289d61d5dd60bf",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4bf9bf85b4d0c2e7992e18e3dedda40427aa2485"
        },
        "date": 1767666250134,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.848125696182251,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.3219111171065,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.7238995152923,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 84.29127604166666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 88.95703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 651224.5117103632,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 645701.3097371834,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001427,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18676282.32051835,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17875656.628040783,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.3447558879852295,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 68.00973830726387,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 69.22773845938593,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 267.8703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 281.10546875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1640595.036917829,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1585721.1348353517,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.013957,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 45987659.59674808,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 43755583.3195823,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.7166557312011719,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.58102250321257,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.89025663752417,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.930598958333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.7265625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 320463.67084250087,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 318167.04953410506,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001185,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9031377.308223955,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8634145.592266757,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.7876496315002441,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.1880927264957,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.64093915680358,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 135.92578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 141.28515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1177856.6195416022,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1156800.6687772616,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00204,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33678778.03366875,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32169994.58416254,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.255878210067749,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.30169886687533,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.859906870843,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.99427083333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.79296875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 178623.40083548805,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 176380.10851701593,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001097,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5062065.5963332355,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4831318.956013887,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lilotom@gmail.com",
            "name": "Tom Tan",
            "username": "ThomsonTan"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "eff6c47d961a2e7c2ff1c9913aff9a25e2878ad0",
          "message": "chore(docs): minor doc fixes (#1726)",
          "timestamp": "2026-01-06T16:12:58Z",
          "tree_id": "128104fc058e1405c2f28023e0dd8d3a0a6349e1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/eff6c47d961a2e7c2ff1c9913aff9a25e2878ad0"
        },
        "date": 1767717687412,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.8834363222122192,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 71.54186818733052,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 71.89247752130132,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 71.073046875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 72.34765625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 505962.07932592253,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 496432.6051940701,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003311,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14118459.568810942,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13502068.674122198,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 4.729498386383057,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 68.75184119063096,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 69.50496330859617,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 248.39453125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 262.84375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1634353.5416270727,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1557056.819740018,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005132,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 45744674.21143906,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 43517506.83497259,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6562564373016357,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 74.86857044233626,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 75.11786889301175,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 51.365234375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.5078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 272694.25261472125,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 268177.7363480463,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001998,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7682893.415123471,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7332383.972548529,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.3687974214553833,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.99436894565233,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.5469931955091,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 153.362890625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 159.95703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1186896.2723642604,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1170650.0673605858,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001705,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33888866.63739486,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32358870.559264723,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.40991558458947,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.76717807710172,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.088802083333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.3515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 176574.06808496467,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 176574.06808496467,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000883,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5046560.436128301,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4815276.1763077695,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "129437996+c1ly@users.noreply.github.com",
            "name": "c1ly",
            "username": "c1ly"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f913bbf3f08f455f7106ea2d5bb22652a3cea590",
          "message": "[otap-dataflow] Add validation process for verifying otlp requests after encoding and decoding (#1696)\n\nAdd the validation process based on\n[https://github.com/open-telemetry/otel-arrow/blob/main/docs/validation_process.md](https://github.com/open-telemetry/otel-arrow/blob/main/docs/validation_process.md)\nto verify that otlp requests are equivalent after encoding and decoding.",
          "timestamp": "2026-01-06T18:22:35Z",
          "tree_id": "f4b579284028c88e60d371d58390dec0db937f92",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f913bbf3f08f455f7106ea2d5bb22652a3cea590"
        },
        "date": 1767725678815,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.325303077697754,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.57060319077812,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.9351144537295,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 80.45403645833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 83.53125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 632029.9739341008,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 623653.6610270493,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002534,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17968917.142143153,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17178213.737410314,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.1034111976623535,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 73.33776876358233,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 73.8465881211372,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 253.087109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 269.00390625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1762519.41416103,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1707821.1909416474,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001949,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 49963874.13932799,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47634934.18105244,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.53365558385849,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.71517214660459,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.69527926855726,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.51158854166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.41015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 311681.39925101935,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 310018.09402754426,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001014,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 8798018.172491977,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8396249.055206025,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.1064651012420654,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 82.93513150782765,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 83.19928694306931,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 145.34244791666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 151.44921875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1158687.2681585576,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1134279.9236776372,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006528,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 32936536.750927523,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 31467974.786998533,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.31944209748829,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.52685260632498,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.166796875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.76171875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 177857.68192154562,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 177857.68192154562,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000782,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5036344.412081824,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4806143.42474627,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "76450334+andborja@users.noreply.github.com",
            "name": "Andres Borja",
            "username": "andborja"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4d58ac8f2141e9fcc0baaa73cc7cdbeac38993eb",
          "message": "feat: Add 'tls' option to internal telemetry OTLP configuration (#1724)\n\nAdd 'tls' option to internal telemetry OTLP configuration with ca file.",
          "timestamp": "2026-01-06T21:31:56Z",
          "tree_id": "5d9e2b4a2da229689c1ecee2bea8fe3cac395d35",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4d58ac8f2141e9fcc0baaa73cc7cdbeac38993eb"
        },
        "date": 1767736997571,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.9790064096450806,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.87947823204814,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.67855075345658,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 81.38502604166666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 85.73046875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 606038.9299273174,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 600105.770237791,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001756,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17137152.042166825,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16359132.75278381,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.397179126739502,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 58.70206912790778,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.00592823638614,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 252.75716145833334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 262.76953125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1446648.095667035,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1368569.90248142,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00395,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 40040472.611128874,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 38086759.02650312,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.40735115084551,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.56497198520346,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 51.67552083333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.765625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 340982.0122621993,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 340982.0122621993,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000819,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9655511.060521543,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9243082.02277746,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.9453976154327393,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 73.61058104690362,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 74.41713391357646,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 138.266015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 142.234375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1031096.3898177881,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1000726.5009360189,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006805,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 29083075.970769618,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 27780250.255587682,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.2705289050926,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.64013783549784,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.00963541666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.33203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 179756.85425505054,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 179756.85425505054,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00105,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5146220.258675777,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4912487.365020229,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "342cd0e29c6e837241acf56d9d44919490e37c0b",
          "message": "Upgrade Collector dependencies to v0.143.0/v1.49.0 (#1732)",
          "timestamp": "2026-01-07T15:46:15Z",
          "tree_id": "0a308e315698f0191950823512e90e4a991d095a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/342cd0e29c6e837241acf56d9d44919490e37c0b"
        },
        "date": 1767802495223,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.965201735496521,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.16079906497957,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 93.79760444754162,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 86.505078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 88.953125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 654421.174484214,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 648104.6895217648,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001726,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18604649.409854,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17802664.083194133,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 5.150842189788818,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.05526704676926,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 69.91369101525594,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 258.34270833333335,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 273.7421875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1652563.9760932657,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1567443.013378744,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001671,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 46938970.70104789,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 44636219.44943897,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.5758178234100342,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.04834813830024,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.40612862076976,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.572265625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.41796875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 312012.81850496074,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 310216.19319062005,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001381,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 8809992.789596163,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8408843.553614018,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.83086758852005,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 89.0701737099697,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 89.60935606273661,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 137.59296875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 145.625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1226728.0793420747,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1216535.5928344955,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004985,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 35435027.247969314,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33837477.41820569,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.2128472328186035,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.47517546220135,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.85469623505823,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.391276041666664,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.25390625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 178089.6290690487,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 175929.67399611423,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001248,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5089127.383485373,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4867675.610820955,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "197425009+otelbot[bot]@users.noreply.github.com",
            "name": "otelbot[bot]",
            "username": "otelbot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5d4801241809c9e08da610997e8f0685894a653a",
          "message": "chore(release) Prepare Release v0.46.0 (#1733)\n\n## Release v0.46.0\n\nThis PR prepares the repository for release v0.46.0.\n\n### Changes included:\n- Updated CHANGELOG.md with release notes\n- Updated collector/otelarrowcol-build.yaml version to v0.46.0\n- Updated collector/cmd/otelarrowcol/main.go version to v0.46.0\n\n### Release Notes:\n- Upgrade various Go dependencies.\n[#1463](https://github.com/open-telemetry/otel-arrow/pull/1463),\n[#1466](https://github.com/open-telemetry/otel-arrow/pull/1466),\n[#1682](https://github.com/open-telemetry/otel-arrow/pull/1682)\n- Upgrade Go toolchain to `v1.25.4`.\n[#1464](https://github.com/open-telemetry/otel-arrow/pull/1464)\n- Upgrade to v0.140.0/v0.140.1/v1.46.0 of collector dependencies.\n- Note: Collector-Contrib dependencies were [released at\nv0.140.1](https://github.com/open-telemetry/opentelemetry-collector-contrib/releases/tag/v0.140.1)\ndue to some release process failures.\n[#1470](https://github.com/open-telemetry/otel-arrow/pull/1470)\n- Upgrade to v0.142.0/v1.48.0 of collector dependencies.\n[#1682](https://github.com/open-telemetry/otel-arrow/pull/1682)\n- Upgrade `arrow-go` to `v18.5.0`.\n[#1702](https://github.com/open-telemetry/otel-arrow/pull/1702)\n- Note: Includes small changes to JSON serialization of timestamps due\nto upstream bugfix.\n- Upgrade to v0.143.0/v1.49.0 of collector dependencies.\n[#1732](https://github.com/open-telemetry/otel-arrow/pull/1732)\n\n### Checklist:\n- [ ] Verify CHANGELOG.md formatting and content\n- [ ] Verify collector version update in\ncollector/otelarrowcol-build.yaml\n- [ ] Verify collector main.go version update in\ncollector/cmd/otelarrowcol/main.go\n- [ ] Confirm all tests pass\n- [ ] Ready to merge and tag release\n\nAfter merging this PR, run the **Push Release** workflow to create git\ntags and publish the GitHub release.\n\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2026-01-07T16:15:45Z",
          "tree_id": "01e2be262cca043f1b19ee355dd057b35ce8bf13",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5d4801241809c9e08da610997e8f0685894a653a"
        },
        "date": 1767804246244,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.0642279386520386,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.59447429993388,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 83.90331955703554,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 85.474609375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 89.34375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 578810.7345530113,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 572650.8686327594,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001306,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16454162.677927436,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15720381.522368647,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 6.64755392074585,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.27610193854498,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.61586486018848,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 260.0377604166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 273.8984375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1317262.3184024128,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1229696.5959879528,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002931,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36239084.03234526,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 34308272.6348797,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.0806763172149658,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.38640201311333,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.66510356944015,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.30104166666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.140625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 342682.6344343348,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 338979.34467637766,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000706,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9704995.238933723,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9289478.648959469,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.5869901180267334,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 81.6317724347897,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.92558531962588,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 149.19856770833334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 153.75390625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1129773.9718878227,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1100546.8308114454,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.012712,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 32383612.454622872,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 30931872.44943122,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.42707128645124,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.69070842447715,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.9453125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.37109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 177074.48865045627,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 177074.48865045627,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000738,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5062405.334611252,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4831336.440303197,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "geukhanuslu@gmail.com",
            "name": "Gokhan Uslu",
            "username": "gouslu"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a2b3698c369bc0ea91874aee3ba5ca43cbb0ed68",
          "message": "perf(azure-monitor-exporter): Azure monitor exporter log transformer optimizations (#1731)\n\n- Reuse buffer across log records\n- Schema pre-parsing to avoid parsing for each transformation\n- Use enum matching instead of string comparisons for log record fields\n- Use Cow<str> for HashMap lookups and eq_ignore_ascii_case to avoid\ntemporary strings\n- Remove \"disable_schema_mapping\", not needed for customers",
          "timestamp": "2026-01-07T21:20:16Z",
          "tree_id": "4b6459c7d86f15b5068ed8fe7cf7f02682ac9276",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a2b3698c369bc0ea91874aee3ba5ca43cbb0ed68"
        },
        "date": 1767822706774,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.237973690032959,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.32068464323424,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.54037788919838,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 82.70169270833334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 83.6328125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 629481.7770446854,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 621688.9581672783,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003961,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17855621.797708552,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17081773.11517153,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 6.207927703857422,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 61.2391742558664,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 61.80226378344934,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 268.8639322916667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 282.078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1511749.0268552923,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1417900.742714754,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001097,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 42395206.10567855,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 40288158.02441459,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.6546909213066101,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.5666096790045,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.46323038006038,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.92161458333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.67578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 310065.25083390804,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 308035.28192626516,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000919,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 8746376.827615708,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8347906.637531159,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.3544328212738037,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 79.39274396955825,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 79.84741903431144,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 139.791796875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 144.30078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1106325.9171242667,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1069214.957036831,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003837,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 31314344.820048664,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 29912717.952391278,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.46312110509118,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.78624944229878,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.426432291666664,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.14453125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 178373.70269030696,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 178373.70269030696,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000997,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5101085.659242162,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4867228.783314197,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}