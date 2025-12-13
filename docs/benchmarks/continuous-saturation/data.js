window.BENCHMARK_DATA = {
  "lastUpdate": 1765584558654,
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
      }
    ]
  }
}