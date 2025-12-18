window.BENCHMARK_DATA = {
  "lastUpdate": 1766084606644,
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
      }
    ]
  }
}