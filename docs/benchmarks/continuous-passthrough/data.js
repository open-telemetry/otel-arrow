window.BENCHMARK_DATA = {
  "lastUpdate": 1771253720882,
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
          "id": "b793a1e733d3f1c1a626430c0d93e00e9e6d98e2",
          "message": "PerfTest - add passthrough scenario (#1810)\n\nAll existing tests add a dummy processor in the middle to force\nconversion into internal format. But there are real scenarios where\nengine can act as \"pass-through\" without having to do this conversion.\nThis adds a perf-test to continuously measure the throughput when\noperating as pass-through.\nModelled after saturation tests - where we put as much load as required\nto hit 100% CPU in engine. Local run shows it can do twice (minimum) the\nthroughput!",
          "timestamp": "2026-01-16T22:14:05Z",
          "tree_id": "1cf5cc0d17331750aa5a89bae24befe3b9d85c4a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b793a1e733d3f1c1a626430c0d93e00e9e6d98e2"
        },
        "date": 1768603839941,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.88627355647486,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.35741167310366,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.70872395833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.7265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 565936.8956417819,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 565936.8956417819,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001149,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10151891.240092983,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10137779.0385659,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "c68e70eda406b6341cbd0ae73cf4521a56639d47",
          "message": "Update batch size variation perf tests (#1809)\n\nModified to use 10, 100, 512, 1024, 4096, 8192 as sizes.\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-16T23:41:49Z",
          "tree_id": "2ebd0b963e9f0a0c3a4e59c7f3429710cd874ea8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c68e70eda406b6341cbd0ae73cf4521a56639d47"
        },
        "date": 1768609059170,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.16226525157725,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.8441080331295,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.4703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.8359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 570846.5889158417,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 570846.5889158417,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000821,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10353526.431908403,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10311706.708369318,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "8470d2d442782c9e6dadf2b9379160f88ccc2c39",
          "message": "Split opentelemetry_client into otel_sdk, tracing_init, and ITS parts (#1808)\n\nPart of https://github.com/open-telemetry/otel-arrow/pull/1771.\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1736.\n\nThis is a non-functional refactoring of `opentelemetry_client.rs` into\nother places. This will make it clearer what changes in #1771 and what\nis just moving around.\n\nMoves runtime elements into the InternalTelemetrySystem, simplifies\nsetup for the controller where logs/metrics were separated.\n\nMoves OTel-SDK specific pieces into `otel_sdk` module, separates the\nTokio `tracing` setup.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-01-17T02:49:23Z",
          "tree_id": "0d830a7035fae4fc9093f5ad8a0572cb4a6bc8c0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8470d2d442782c9e6dadf2b9379160f88ccc2c39"
        },
        "date": 1768621823511,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.70227227680343,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.67590817445002,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.09010416666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.9921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 560083.2358186145,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 560083.2358186145,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002053,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10126874.629998853,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10083547.927030776,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "d8a0d6d381f1a9f2968c182c88920cb4ded93cc0",
          "message": "Create entity and expose entity keys via thread locals and task locals (#1785)\n\nThe engine now creates the following entities:\n\n* **Pipeline** -> Stored in a thread local associated with the pipeline\nthread.\n* **Node** -> Stored in the task local of the node.\n* **Channel**\n  * **Sender entity** stored in the task local of the sender node.\n  * **Receiver entity** stored in the task local of the receiver node.\n\nAn entity cleanup mechanism is in place. A unit test has been added to\nvalidate this cleanup process.\n\nThe final goal is to be able to use these entities directly when\nreporting metric sets and events. This allows us to report the\nattributes of all our entities using a simple numerical ID.\n\nCloses https://github.com/open-telemetry/otel-arrow/issues/1791",
          "timestamp": "2026-01-18T07:23:23Z",
          "tree_id": "0c4a094815fe796e1d1add0c2bcef4a588b7a0f7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d8a0d6d381f1a9f2968c182c88920cb4ded93cc0"
        },
        "date": 1768724562085,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.3163589537143707,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.53002363824076,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.07931411692307,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.49752604166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.76171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 558336.1490813495,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 556569.8026016478,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001818,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10152645.282562107,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10100967.296759335,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "c28577df824da63d5759a149df623c30aa108c09",
          "message": "chore(deps): update dependency kubernetes to v35 (#1820)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [kubernetes](https://redirect.github.com/kubernetes-client/python) |\n`==34.1.0` → `==35.0.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/kubernetes/35.0.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/kubernetes/34.1.0/35.0.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>kubernetes-client/python (kubernetes)</summary>\n\n###\n[`v35.0.0`](https://redirect.github.com/kubernetes-client/python/blob/HEAD/CHANGELOG.md#v3500snapshot)\n\n[Compare\nSource](https://redirect.github.com/kubernetes-client/python/compare/v34.1.0...v35.0.0)\n\nKubernetes API Version: v1.35.0\n\n##### API Change\n\n- Added `ObservedGeneration` to CustomResourceDefinition conditions.\n([kubernetes/kubernetes#134984](https://redirect.github.com/kubernetes/kubernetes/pull/134984),\n[@&#8203;michaelasp](https://redirect.github.com/michaelasp))\n- Added `WithOrigin` within `apis/core/validation` with adjusted tests.\n([kubernetes/kubernetes#132825](https://redirect.github.com/kubernetes/kubernetes/pull/132825),\n[@&#8203;PatrickLaabs](https://redirect.github.com/PatrickLaabs))\n- Added scoring for the prioritized list feature so nodes that best\nsatisfy the highest-ranked subrequests were chosen.\n([kubernetes/kubernetes#134711](https://redirect.github.com/kubernetes/kubernetes/pull/134711),\n[@&#8203;mortent](https://redirect.github.com/mortent)) \\[SIG Node,\nScheduling and Testing]\n- Added the `--min-compatibility-version` flag to `kube-apiserver`,\n`kube-controller-manager`, and `kube-scheduler`.\n([kubernetes/kubernetes#133980](https://redirect.github.com/kubernetes/kubernetes/pull/133980),\n[@&#8203;siyuanfoundation](https://redirect.github.com/siyuanfoundation))\n\\[SIG API Machinery, Architecture, Cluster Lifecycle, Etcd, Scheduling\nand Testing]\n- Added the `StorageVersionMigration` `v1beta1` API and removed the\n`v1alpha1` API.\n\nACTION REQUIRED: The `v1alpha1` API is no longer supported. Users must\nremove any `v1alpha1` resources before upgrading.\n([kubernetes/kubernetes#134784](https://redirect.github.com/kubernetes/kubernetes/pull/134784),\n[@&#8203;michaelasp](https://redirect.github.com/michaelasp)) \\[SIG API\nMachinery, Apps, Auth, Etcd and Testing]\n- Added validation to ensure `log-flush-frequency` is a positive value,\nreturning an error instead of causing a panic.\n([kubernetes/kubernetes#133540](https://redirect.github.com/kubernetes/kubernetes/pull/133540),\n[@&#8203;BenTheElder](https://redirect.github.com/BenTheElder)) \\[SIG\nArchitecture, Instrumentation, Network and Node]\n- All containers are restarted when a source container in a restart\npolicy rule exits. This alpha feature is gated behind\n`RestartAllContainersOnContainerExit`.\n([kubernetes/kubernetes#134345](https://redirect.github.com/kubernetes/kubernetes/pull/134345),\n[@&#8203;yuanwang04](https://redirect.github.com/yuanwang04)) \\[SIG\nApps, Node and Testing]\n- CSI drivers can now opt in to receive service account tokens via the\nsecrets field instead of volume context by setting\n`spec.serviceAccountTokenInSecrets: true` in the CSIDriver object. This\nprevents tokens from being exposed in logs and other outputs. The\nfeature is gated by the `CSIServiceAccountTokenSecrets` feature gate\n(beta in `v1.35`).\n([kubernetes/kubernetes#134826](https://redirect.github.com/kubernetes/kubernetes/pull/134826),\n[@&#8203;aramase](https://redirect.github.com/aramase)) \\[SIG API\nMachinery, Auth, Storage and Testing]\n- Changed kuberc configuration schema. Two new optional fields added to\nkuberc configuration, `credPluginPolicy` and `credPluginAllowlist`. This\nis documented in\n[KEP-3104](https://redirect.github.com/kubernetes/enhancements/blob/master/keps/sig-cli/3104-introduce-kuberc/README.md#allowlist-design-details)\nand documentation is added to the website by\n[kubernetes/website#52877](https://redirect.github.com/kubernetes/website/pull/52877)\n([kubernetes/kubernetes#134870](https://redirect.github.com/kubernetes/kubernetes/pull/134870),\n[@&#8203;pmengelbert](https://redirect.github.com/pmengelbert)) \\[SIG\nAPI Machinery, Architecture, Auth, CLI, Instrumentation and Testing]\n- DRA device taints: `DeviceTaintRule` status provides information about\nthe rule, including whether Pods still need to be evicted\n(`EvictionInProgress` condition). The newly added `None` effect can be\nused to preview what a `DeviceTaintRule` would do if it used the\n`NoExecute` effect and to taint devices (`device health`) without\nimmediately affecting scheduling or running Pods.\n([kubernetes/kubernetes#134152](https://redirect.github.com/kubernetes/kubernetes/pull/134152),\n[@&#8203;pohly](https://redirect.github.com/pohly)) \\[SIG API Machinery,\nApps, Auth, Node, Release, Scheduling and Testing]\n- DRA: The `DynamicResourceAllocation` feature gate for the core\nfunctionality (GA in `v1.34`) has now been locked to enabled-by-default\nand cannot be disabled anymore.\n([kubernetes/kubernetes#134452](https://redirect.github.com/kubernetes/kubernetes/pull/134452),\n[@&#8203;pohly](https://redirect.github.com/pohly)) \\[SIG Auth, Node,\nScheduling and Testing]\n- Enabled `kubectl get -o kyaml` by default. To disable it, set\n`KUBECTL_KYAML=false`.\n([kubernetes/kubernetes#133327](https://redirect.github.com/kubernetes/kubernetes/pull/133327),\n[@&#8203;thockin](https://redirect.github.com/thockin))\n- Enabled in-place resizing of pod-level resources.\n- Added `Resources` in `PodStatus` to capture resources set in the\npod-level cgroup.\n- Added `AllocatedResources` in `PodStatus` to capture resources\nrequested in the `PodSpec`.\n([kubernetes/kubernetes#132919](https://redirect.github.com/kubernetes/kubernetes/pull/132919),\n[@&#8203;ndixita](https://redirect.github.com/ndixita)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Instrumentation, Node,\nScheduling and Testing]\n- Enabled the `NominatedNodeNameForExpectation` feature in\nkube-scheduler by default.\n- Enabled the `ClearingNominatedNodeNameAfterBinding` feature in\nkube-apiserver by default.\n([kubernetes/kubernetes#135103](https://redirect.github.com/kubernetes/kubernetes/pull/135103),\n[@&#8203;ania-borowiec](https://redirect.github.com/ania-borowiec))\n\\[SIG API Machinery, Apps, Architecture, Auth, Autoscaling, CLI, Cloud\nProvider, Cluster Lifecycle, Etcd, Instrumentation, Network, Node,\nScheduling, Storage and Testing]\n- Enhanced discovery responses to merge API groups and resources from\nall peer apiservers when the `UnknownVersionInteroperabilityProxy`\nfeature is enabled.\n([kubernetes/kubernetes#133648](https://redirect.github.com/kubernetes/kubernetes/pull/133648),\n[@&#8203;richabanker](https://redirect.github.com/richabanker)) \\[SIG\nAPI Machinery, Auth, Cloud Provider, Node, Scheduling and Testing]\n- Extended `core/v1` `Toleration` to support numeric comparison\noperators (`Gt`,`Lt`).\n([kubernetes/kubernetes#134665](https://redirect.github.com/kubernetes/kubernetes/pull/134665),\n[@&#8203;helayoty](https://redirect.github.com/helayoty)) \\[SIG API\nMachinery, Apps, Node, Scheduling, Testing and Windows]\n- Feature gate dependencies are now explicit, and validated at startup.\nA feature can no longer be enabled if it depends on a disabled feature.\nIn particular, this means that `AllAlpha=true` will no longer work\nwithout enabling disabled-by-default beta features that are depended on\n(either with `AllBeta=true` or explicitly enumerating the disabled\ndependencies).\n([kubernetes/kubernetes#133697](https://redirect.github.com/kubernetes/kubernetes/pull/133697),\n[@&#8203;tallclair](https://redirect.github.com/tallclair)) \\[SIG API\nMachinery, Architecture, Cluster Lifecycle and Node]\n- Generated OpenAPI model packages for API types into\n`zz_generated.model_name.go` files, accessible via the\n`OpenAPIModelName()` function. This allows API authors to declare\ndesired OpenAPI model packages instead of relying on the Go package path\nof API types.\n([kubernetes/kubernetes#131755](https://redirect.github.com/kubernetes/kubernetes/pull/131755),\n[@&#8203;jpbetz](https://redirect.github.com/jpbetz)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Cloud Provider, Cluster\nLifecycle, Instrumentation, Network, Node, Scheduling, Storage and\nTesting]\n- Implemented constrained impersonation as described in\n[KEP-5284](https://kep.k8s.io/5284).\n([kubernetes/kubernetes#134803](https://redirect.github.com/kubernetes/kubernetes/pull/134803),\n[@&#8203;enj](https://redirect.github.com/enj)) \\[SIG API Machinery,\nAuth and Testing]\n- Introduced a new declarative validation tag `+k8s:customUnique` to\ncontrol listmap uniqueness.\n([kubernetes/kubernetes#134279](https://redirect.github.com/kubernetes/kubernetes/pull/134279),\n[@&#8203;yongruilin](https://redirect.github.com/yongruilin)) \\[SIG API\nMachinery and Auth]\n- Introduced a structured and versioned `v1alpha1` response for the\n`statusz` endpoint.\n([kubernetes/kubernetes#134313](https://redirect.github.com/kubernetes/kubernetes/pull/134313),\n[@&#8203;richabanker](https://redirect.github.com/richabanker)) \\[SIG\nAPI Machinery, Architecture, Instrumentation, Network, Node, Scheduling\nand Testing]\n- Introduced a structured and versioned `v1alpha1` response format for\nthe `flagz` endpoint.\n([kubernetes/kubernetes#134995](https://redirect.github.com/kubernetes/kubernetes/pull/134995),\n[@&#8203;yongruilin](https://redirect.github.com/yongruilin)) \\[SIG API\nMachinery, Architecture, Instrumentation, Network, Node, Scheduling and\nTesting]\n- Introduced the GangScheduling kube-scheduler plugin to support\n\"all-or-nothing\" scheduling using the `scheduling.k8s.io/v1alpha1`\nWorkload API.\n([kubernetes/kubernetes#134722](https://redirect.github.com/kubernetes/kubernetes/pull/134722),\n[@&#8203;macsko](https://redirect.github.com/macsko)) \\[SIG API\nMachinery, Apps, Auth, CLI, Etcd, Scheduling and Testing]\n- Introduced the Node Declared Features capability (alpha), which\nincludes:\n- A new `Node.Status.DeclaredFeatures` field for publishing\nnode-specific features.\n- A `component-helpers` library for feature registration and inference.\n- A `NodeDeclaredFeatures` scheduler plugin to match pods with nodes\nthat provide required features.\n- A `NodeDeclaredFeatureValidator` admission plugin to validate pod\nupdates against a node's declared features.\n([kubernetes/kubernetes#133389](https://redirect.github.com/kubernetes/kubernetes/pull/133389),\n[@&#8203;pravk03](https://redirect.github.com/pravk03)) \\[SIG API\nMachinery, Apps, Node, Release, Scheduling and Testing]\n- Introduced the `scheduling.k8s.io/v1alpha1` Workload API to express\nworkload-level scheduling requirements and allow the kube-scheduler to\nact on them.\n([kubernetes/kubernetes#134564](https://redirect.github.com/kubernetes/kubernetes/pull/134564),\n[@&#8203;macsko](https://redirect.github.com/macsko)) \\[SIG API\nMachinery, Apps, CLI, Etcd, Scheduling and Testing]\n- Introduced the alpha `MutableSchedulingDirectivesForSuspendedJobs`\nfeature gate (disabled by default), which allows mutating a Job's\nscheduling directives while the Job is suspended.\nIt also updates the Job controller to clears the `status.startTime`\nfield for suspended Jobs.\n([kubernetes/kubernetes#135104](https://redirect.github.com/kubernetes/kubernetes/pull/135104),\n[@&#8203;mimowo](https://redirect.github.com/mimowo)) \\[SIG Apps and\nTesting]\n- Kube-apiserver: Fixed a `v1.34` regression in\n`CustomResourceDefinition` handling that incorrectly warned about\nunrecognized formats on number and integer properties.\n([kubernetes/kubernetes#133896](https://redirect.github.com/kubernetes/kubernetes/pull/133896),\n[@&#8203;yongruilin](https://redirect.github.com/yongruilin)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Cloud Provider, Contributor\nExperience, Network, Node and Scheduling]\n- Kube-apiserver: Fixed a possible panic validating a custom resource\nwhose `CustomResourceDefinition` indicates a status subresource exists,\nbut which does not define a `status` property in the `openAPIV3Schema`.\n([kubernetes/kubernetes#133721](https://redirect.github.com/kubernetes/kubernetes/pull/133721),\n[@&#8203;fusida](https://redirect.github.com/fusida)) \\[SIG API\nMachinery, Apps, Architecture, Auth, Autoscaling, CLI, Cloud Provider,\nCluster Lifecycle, Etcd, Instrumentation, Network, Node, Release,\nScheduling, Storage and Testing]\n- Kubernetes API Go types removed runtime use of the\n`github.com/gogo/protobuf` library, and are no longer registered into\nthe global gogo type registry. Kubernetes API Go types were not suitable\nfor use with the `google.golang.org/protobuf` library, and no longer\nimplement `ProtoMessage()` by default to avoid accidental incompatible\nuse. If removal of these marker methods impacts your use, it can be\nre-enabled for one more release with a\n`kubernetes_protomessage_one_more_release` build tag, but will be\nremoved in `v1.36`.\n([kubernetes/kubernetes#134256](https://redirect.github.com/kubernetes/kubernetes/pull/134256),\n[@&#8203;liggitt](https://redirect.github.com/liggitt)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Cluster Lifecycle,\nInstrumentation, Network, Node, Scheduling and Storage]\n- Made node affinity in Persistent Volume mutable.\n([kubernetes/kubernetes#134339](https://redirect.github.com/kubernetes/kubernetes/pull/134339),\n[@&#8203;huww98](https://redirect.github.com/huww98)) \\[SIG API\nMachinery, Apps and Node]\n- Moved the `ImagePullIntent` and `ImagePulledRecord` objects used by\nthe kubelet to track image pulls to the `v1beta1` API version.\n([kubernetes/kubernetes#132579](https://redirect.github.com/kubernetes/kubernetes/pull/132579),\n[@&#8203;stlaz](https://redirect.github.com/stlaz)) \\[SIG Auth and Node]\n- Pod resize now only allows CPU and memory resources; other resource\ntypes are forbidden.\n([kubernetes/kubernetes#135084](https://redirect.github.com/kubernetes/kubernetes/pull/135084),\n[@&#8203;tallclair](https://redirect.github.com/tallclair)) \\[SIG Apps,\nNode and Testing]\n- Prevented Pods from being scheduled onto nodes that lack the required\nCSI driver.\n([kubernetes/kubernetes#135012](https://redirect.github.com/kubernetes/kubernetes/pull/135012),\n[@&#8203;gnufied](https://redirect.github.com/gnufied)) \\[SIG API\nMachinery, Scheduling, Storage and Testing]\n- Promoted HPA configurable tolerance to beta. The\n`HPAConfigurableTolerance` feature gate has now been enabled by default.\n([kubernetes/kubernetes#133128](https://redirect.github.com/kubernetes/kubernetes/pull/133128),\n[@&#8203;jm-franc](https://redirect.github.com/jm-franc)) \\[SIG API\nMachinery and Autoscaling]\n- Promoted ReplicaSet and Deployment `.status.terminatingReplicas`\ntracking to beta. The `DeploymentReplicaSetTerminatingReplicas` feature\ngate is now enabled by default.\n([kubernetes/kubernetes#133087](https://redirect.github.com/kubernetes/kubernetes/pull/133087),\n[@&#8203;atiratree](https://redirect.github.com/atiratree)) \\[SIG API\nMachinery, Apps and Testing]\n- Promoted `PodObservedGenerationTracking` to GA.\n([kubernetes/kubernetes#134948](https://redirect.github.com/kubernetes/kubernetes/pull/134948),\n[@&#8203;natasha41575](https://redirect.github.com/natasha41575)) \\[SIG\nAPI Machinery, Apps, Node, Scheduling and Testing]\n- Promoted the `JobManagedBy` feature to general availability. The\n`JobManagedBy` feature gate was locked to `true` and will be removed in\na future Kubernetes release.\n([kubernetes/kubernetes#135080](https://redirect.github.com/kubernetes/kubernetes/pull/135080),\n[@&#8203;dejanzele](https://redirect.github.com/dejanzele)) \\[SIG API\nMachinery, Apps and Testing]\n- Promoted the `MaxUnavailableStatefulSet` feature to beta and enabling\nit by default.\n([kubernetes/kubernetes#133153](https://redirect.github.com/kubernetes/kubernetes/pull/133153),\n[@&#8203;helayoty](https://redirect.github.com/helayoty)) \\[SIG API\nMachinery and Apps]\n- Removed the `StrictCostEnforcementForVAP` and\n`StrictCostEnforcementForWebhooks` feature gates, which were locked\nsince `v1.32`.\n([kubernetes/kubernetes#134994](https://redirect.github.com/kubernetes/kubernetes/pull/134994),\n[@&#8203;liggitt](https://redirect.github.com/liggitt)) \\[SIG API\nMachinery, Auth, Node and Testing]\n- Scheduler: Added the `bindingTimeout` argument to the DynamicResources\nplugin configuration, allowing customization of the wait duration in\n`PreBind` for device binding conditions.\nDefaults to 10 minutes when `DRADeviceBindingConditions` and\n`DRAResourceClaimDeviceStatus` are both enabled.\n([kubernetes/kubernetes#134905](https://redirect.github.com/kubernetes/kubernetes/pull/134905),\n[@&#8203;fj-naji](https://redirect.github.com/fj-naji)) \\[SIG Node and\nScheduling]\n- The DRA device taints and toleration feature received a separate\nfeature gate, `DRADeviceTaintRules`, which controlled support for\n`DeviceTaintRules`. This allowed disabling it while keeping\n`DRADeviceTaints` enabled so that tainting via `ResourceSlices`\ncontinued to work.\n([kubernetes/kubernetes#135068](https://redirect.github.com/kubernetes/kubernetes/pull/135068),\n[@&#8203;pohly](https://redirect.github.com/pohly)) \\[SIG API Machinery,\nApps, Auth, Node, Scheduling and Testing]\n- The Pod Certificates feature moved to beta. The\n`PodCertificateRequest` feature gate is set disabled by default. To use\nthe feature, users must enable the certificates API groups in `v1beta1`\nand enable the `PodCertificateRequest` feature gate. The\n`UserAnnotations` field was added to the `PodCertificateProjection` API\nand the corresponding `UnverifiedUserAnnotations` field was added to the\n`PodCertificateRequest` API.\n([kubernetes/kubernetes#134624](https://redirect.github.com/kubernetes/kubernetes/pull/134624),\n[@&#8203;yt2985](https://redirect.github.com/yt2985)) \\[SIG API\nMachinery, Apps, Auth, Etcd, Instrumentation, Node and Testing]\n- The `KubeletEnsureSecretPulledImages` feature was promoted to Beta and\nenabled by default.\n([kubernetes/kubernetes#135228](https://redirect.github.com/kubernetes/kubernetes/pull/135228),\n[@&#8203;aramase](https://redirect.github.com/aramase)) \\[SIG Auth, Node\nand Testing]\n- The `PreferSameZone` and `PreferSameNode` values for the Service\n  `trafficDistribution` field graduated to general availability. The\n  `PreferClose` value is now deprecated in favor of the more explicit\n`PreferSameZone`.\n([kubernetes/kubernetes#134457](https://redirect.github.com/kubernetes/kubernetes/pull/134457),\n[@&#8203;danwinship](https://redirect.github.com/danwinship)) \\[SIG API\nMachinery, Apps, Network and Testing]\n- Updated `ResourceQuota` to count device class requests within a\n`ResourceClaim` as two additional quotas when the `DRAExtendedResource`\nfeature is enabled:\n- `requests.deviceclass.resource.k8s.io/<deviceclass>` is charged based\non the worst-case number of devices requested.\n- Device classes mapping to an extended resource now consume\n`requests.<extended resource name>`.\n([kubernetes/kubernetes#134210](https://redirect.github.com/kubernetes/kubernetes/pull/134210),\n[@&#8203;yliaog](https://redirect.github.com/yliaog)) \\[SIG API\nMachinery, Apps, Node, Scheduling and Testing]\n- Updated storage version for `MutatingAdmissionPolicy` to `v1beta1`.\n([kubernetes/kubernetes#133715](https://redirect.github.com/kubernetes/kubernetes/pull/133715),\n[@&#8203;cici37](https://redirect.github.com/cici37)) \\[SIG API\nMachinery, Etcd and Testing]\n- Updated the Partitionable Devices feature to support referencing\ncounter sets across ResourceSlices within the same resource pool.\nDevices from incomplete pools were no longer considered for allocation.\nThis change introduced backwards-incompatible updates to the alpha\nfeature, requiring any ResourceSlices using it to be removed before\nupgrading or downgrading between v1.34 and v1.35.\n([kubernetes/kubernetes#134189](https://redirect.github.com/kubernetes/kubernetes/pull/134189),\n[@&#8203;mortent](https://redirect.github.com/mortent)) \\[SIG API\nMachinery, Node, Scheduling and Testing]\n- Upgraded the `PodObservedGenerationTracking` feature to beta in\n`v1.34` and removed the alpha version description from the OpenAPI\nspecification.\n([kubernetes/kubernetes#133883](https://redirect.github.com/kubernetes/kubernetes/pull/133883),\n[@&#8203;yangjunmyfm192085](https://redirect.github.com/yangjunmyfm192085))\n- Add scoring for the prioritized list feature so that the node that can\nsatisfy the best ranked subrequests are chosen.\n([kubernetes/kubernetes#134711](https://redirect.github.com/kubernetes/kubernetes/pull/134711),\n[@&#8203;mortent](https://redirect.github.com/mortent)) \\[SIG Node,\nScheduling and Testing]\n- Allows restart all containers when the source container exits with a\nmatching restart policy rule. This is an alpha feature behind feature\ngate RestartAllContainersOnContainerExit.\n([kubernetes/kubernetes#134345](https://redirect.github.com/kubernetes/kubernetes/pull/134345),\n[@&#8203;yuanwang04](https://redirect.github.com/yuanwang04)) \\[SIG\nApps, Node and Testing]\n- Changed kuberc configuration schema. Two new optional fields added to\nkuberc configuration, `credPluginPolicy` and `credPluginAllowlist`. This\nis documented in\n[KEP-3104](https://redirect.github.com/kubernetes/enhancements/blob/master/keps/sig-cli/3104-introduce-kuberc/README.md#allowlist-design-details)\nand documentation is added to the website by\n[kubernetes/website#52877](https://redirect.github.com/kubernetes/website/pull/52877)\n([kubernetes/kubernetes#134870](https://redirect.github.com/kubernetes/kubernetes/pull/134870),\n[@&#8203;pmengelbert](https://redirect.github.com/pmengelbert)) \\[SIG\nAPI Machinery, Architecture, Auth, CLI, Instrumentation and Testing]\n- Enhanced discovery response to support merged API groups/resources\nfrom all peer apiservers when UnknownVersionInteroperabilityProxy\nfeature is enabled\n([kubernetes/kubernetes#133648](https://redirect.github.com/kubernetes/kubernetes/pull/133648),\n[@&#8203;richabanker](https://redirect.github.com/richabanker)) \\[SIG\nAPI Machinery, Auth, Cloud Provider, Node, Scheduling and Testing]\n- Extend `core/v1 Toleration` to support numeric comparison operators\n(`Gt`, `Lt`).\n([kubernetes/kubernetes#134665](https://redirect.github.com/kubernetes/kubernetes/pull/134665),\n[@&#8203;helayoty](https://redirect.github.com/helayoty)) \\[SIG API\nMachinery, Apps, Node, Scheduling, Testing and Windows]\n- Features: NominatedNodeNameForExpectation in kube-scheduler and\nCleaeringNominatedNodeNameAfterBinding in kube-apiserver are now enabled\nby default.\n([kubernetes/kubernetes#135103](https://redirect.github.com/kubernetes/kubernetes/pull/135103),\n[@&#8203;ania-borowiec](https://redirect.github.com/ania-borowiec))\n\\[SIG API Machinery, Apps, Architecture, Auth, Autoscaling, CLI, Cloud\nProvider, Cluster Lifecycle, Etcd, Instrumentation, Network, Node,\nScheduling, Storage and Testing]\n- Implement changes to prevent pod scheduling to a node without CSI\ndriver\n([kubernetes/kubernetes#135012](https://redirect.github.com/kubernetes/kubernetes/pull/135012),\n[@&#8203;gnufied](https://redirect.github.com/gnufied)) \\[SIG API\nMachinery, Scheduling, Storage and Testing]\n- Introduce scheduling.k8s.io/v1alpha1 Workload API to allow for\nexpressing workload-level scheduling requirements and let kube-scheduler\nact on those.\n([kubernetes/kubernetes#134564](https://redirect.github.com/kubernetes/kubernetes/pull/134564),\n[@&#8203;macsko](https://redirect.github.com/macsko)) \\[SIG API\nMachinery, Apps, CLI, Etcd, Scheduling and Testing]\n- Introduce the alpha MutableSchedulingDirectivesForSuspendedJobs\nfeature gate (disabled by default) which:\n  1. allows to mutate Job's scheduling directives for suspended Jobs\n2. makes the Job controller to clear the status.startTime field for\nsuspended Jobs\n([kubernetes/kubernetes#135104](https://redirect.github.com/kubernetes/kubernetes/pull/135104),\n[@&#8203;mimowo](https://redirect.github.com/mimowo)) \\[SIG Apps and\nTesting]\n- Introduced GangScheduling kube-scheduler plugin to enable\n\"all-or-nothing\" scheduling. Workload API in scheduling.k8s.io/v1alpha1\nis used to express the desired policy.\n([kubernetes/kubernetes#134722](https://redirect.github.com/kubernetes/kubernetes/pull/134722),\n[@&#8203;macsko](https://redirect.github.com/macsko)) \\[SIG API\nMachinery, Apps, Auth, CLI, Etcd, Scheduling and Testing]\n- PV node affinity is now mutable.\n([kubernetes/kubernetes#134339](https://redirect.github.com/kubernetes/kubernetes/pull/134339),\n[@&#8203;huww98](https://redirect.github.com/huww98)) \\[SIG API\nMachinery, Apps and Node]\n- ResourceQuota now counts device class requests within a ResourceClaim\nobject as consuming two additional quotas when the DRAExtendedResource\nfeature is enabled:\n- `requests.deviceclass.resource.k8s.io/<deviceclass>` with a quantity\nequal to the worst case count of devices requested\n- requests for device classes that map to an extended resource consume\n`requests.<extended resource name>`\n([kubernetes/kubernetes#134210](https://redirect.github.com/kubernetes/kubernetes/pull/134210),\n[@&#8203;yliaog](https://redirect.github.com/yliaog)) \\[SIG API\nMachinery, Apps, Node, Scheduling and Testing]\n- The DRA device taints and toleration feature now has a separate\nfeature gate, DRADeviceTaintRules, which controls whether support for\nDeviceTaintRules is enabled. It is possible to disable that and keep\nDRADeviceTaints enabled, in which case tainting by DRA drivers through\nResourceSlices continues to work.\n([kubernetes/kubernetes#135068](https://redirect.github.com/kubernetes/kubernetes/pull/135068),\n[@&#8203;pohly](https://redirect.github.com/pohly)) \\[SIG API Machinery,\nApps, Auth, Node, Scheduling and Testing]\n- The ImagePullIntent and ImagePulledRecord objects used by kubelet to\nstore information about image pulls have been moved to the v1beta1 API\nversion.\n([kubernetes/kubernetes#132579](https://redirect.github.com/kubernetes/kubernetes/pull/132579),\n[@&#8203;stlaz](https://redirect.github.com/stlaz)) \\[SIG Auth and Node]\n- The KubeletEnsureSecretPulledImages feature is now beta and enabled by\ndefault.\n([kubernetes/kubernetes#135228](https://redirect.github.com/kubernetes/kubernetes/pull/135228),\n[@&#8203;aramase](https://redirect.github.com/aramase)) \\[SIG Auth, Node\nand Testing]\n- This change adds a new alpha feature Node Declared Features, which\nincludes:\n- A new `Node.Status.DeclaredFeatures` field for Kubelet to publish\nnode-specific features.\n- A library in `component-helpers` for feature registration and\ninference.\n- A scheduler plugin (`NodeDeclaredFeatures`) scheduler plugin to match\npods with nodes that provide their required features.\n- An admission plugin (`NodeDeclaredFeatureValidator`) to validate pod\nupdates against a node's declared features.\n([kubernetes/kubernetes#133389](https://redirect.github.com/kubernetes/kubernetes/pull/133389),\n[@&#8203;pravk03](https://redirect.github.com/pravk03)) \\[SIG API\nMachinery, Apps, Node, Release, Scheduling and Testing]\n- This change allows In Place Resize of Pod Level Resources\n- Add Resources in PodStatus to capture resources set at pod-level\ncgroup\n- Add AllocatedResources in PodStatus to capture resources requested in\nthe PodSpec\n([kubernetes/kubernetes#132919](https://redirect.github.com/kubernetes/kubernetes/pull/132919),\n[@&#8203;ndixita](https://redirect.github.com/ndixita)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Instrumentation, Node,\nScheduling and Testing]\n- Updates to the Partitionable Devices feature which allows for\nreferencing counter sets across different ResourceSlices within the same\nresource pool.\n\n  Devices from incomplete pools are no longer considered for allocation.\n\nThis contains backwards incompatible changes to the Partitionable\nDevices alpha feature, so any ResourceSlices that uses the feature\nshould be removed prior to upgrading or downgrading between 1.34 and\n1.35.\n([kubernetes/kubernetes#134189](https://redirect.github.com/kubernetes/kubernetes/pull/134189),\n[@&#8203;mortent](https://redirect.github.com/mortent)) \\[SIG API\nMachinery, Node, Scheduling and Testing]\n- Add ObservedGeneration to CustomResourceDefinition Conditions.\n([kubernetes/kubernetes#134984](https://redirect.github.com/kubernetes/kubernetes/pull/134984),\n[@&#8203;michaelasp](https://redirect.github.com/michaelasp)) \\[SIG API\nMachinery]\n- Add StorageVersionMigration v1beta1 api and remove the v1alpha API.\n\n  Any use of the v1alpha1 api is no longer supported and\nusers must remove any v1alpha1 resources prior to upgrade.\n([kubernetes/kubernetes#134784](https://redirect.github.com/kubernetes/kubernetes/pull/134784),\n[@&#8203;michaelasp](https://redirect.github.com/michaelasp)) \\[SIG API\nMachinery, Apps, Auth, Etcd and Testing]\n- CSI drivers can now opt-in to receive service account tokens via the\nsecrets field instead of volume context by setting\n`spec.serviceAccountTokenInSecrets: true` in the CSIDriver object. This\nprevents tokens from being exposed in logs and other outputs. The\nfeature is gated by the `CSIServiceAccountTokenSecrets` feature gate\n(Beta in v1.35).\n([kubernetes/kubernetes#134826](https://redirect.github.com/kubernetes/kubernetes/pull/134826),\n[@&#8203;aramase](https://redirect.github.com/aramase)) \\[SIG API\nMachinery, Auth, Storage and Testing]\n- DRA device taints: DeviceTaintRule status provided information about\nthe rule, in particular whether pods still need to be evicted\n(\"EvictionInProgress\" condition). The new \"None\" effect can be used to\npreview what a DeviceTaintRule would do if it used the \"NoExecute\"\neffect and to taint devices (\"device health\") without immediately\naffecting scheduling or running pods.\n([kubernetes/kubernetes#134152](https://redirect.github.com/kubernetes/kubernetes/pull/134152),\n[@&#8203;pohly](https://redirect.github.com/pohly)) \\[SIG API Machinery,\nApps, Auth, Node, Release, Scheduling and Testing]\n- DRA: the DynamicResourceAllocation feature gate for the core\nfunctionality (GA in 1.34) is now locked to enabled-by-default and thus\ncannot be disabled anymore.\n([kubernetes/kubernetes#134452](https://redirect.github.com/kubernetes/kubernetes/pull/134452),\n[@&#8203;pohly](https://redirect.github.com/pohly)) \\[SIG Auth, Node,\nScheduling and Testing]\n- Forbid adding resources other than CPU & memory on pod resize.\n([kubernetes/kubernetes#135084](https://redirect.github.com/kubernetes/kubernetes/pull/135084),\n[@&#8203;tallclair](https://redirect.github.com/tallclair)) \\[SIG Apps,\nNode and Testing]\n- Implement constrained impersonation as described in\n<https://kep.k8s.io/5284>\n([kubernetes/kubernetes#134803](https://redirect.github.com/kubernetes/kubernetes/pull/134803),\n[@&#8203;enj](https://redirect.github.com/enj)) \\[SIG API Machinery,\nAuth and Testing]\n- Introduces a structured and versioned v1alpha1 response for flagz\n([kubernetes/kubernetes#134995](https://redirect.github.com/kubernetes/kubernetes/pull/134995),\n[@&#8203;yongruilin](https://redirect.github.com/yongruilin)) \\[SIG API\nMachinery, Architecture, Instrumentation, Network, Node, Scheduling and\nTesting]\n- Introduces a structured and versioned v1alpha1 response for statusz\n([kubernetes/kubernetes#134313](https://redirect.github.com/kubernetes/kubernetes/pull/134313),\n[@&#8203;richabanker](https://redirect.github.com/richabanker)) \\[SIG\nAPI Machinery, Architecture, Instrumentation, Network, Node, Scheduling\nand Testing]\n- New `--min-compatibility-version` flag for apiserver, kcm and kube\nscheduler\n([kubernetes/kubernetes#133980](https://redirect.github.com/kubernetes/kubernetes/pull/133980),\n[@&#8203;siyuanfoundation](https://redirect.github.com/siyuanfoundation))\n\\[SIG API Machinery, Architecture, Cluster Lifecycle, Etcd, Scheduling\nand Testing]\n- Promote PodObservedGenerationTracking to GA.\n([kubernetes/kubernetes#134948](https://redirect.github.com/kubernetes/kubernetes/pull/134948),\n[@&#8203;natasha41575](https://redirect.github.com/natasha41575)) \\[SIG\nAPI Machinery, Apps, Node, Scheduling and Testing]\n- Promoted Job Managed By to general availability. The `JobManagedBy`\nfeature gate is now locked to true, and will be removed in a future\nrelease of Kubernetes.\n([kubernetes/kubernetes#135080](https://redirect.github.com/kubernetes/kubernetes/pull/135080),\n[@&#8203;dejanzele](https://redirect.github.com/dejanzele)) \\[SIG API\nMachinery, Apps and Testing]\n- Promoted ReplicaSet and Deployment `.status.terminatingReplicas`\ntracking to beta. The `DeploymentReplicaSetTerminatingReplicas` feature\ngate is now enabled by default.\n([kubernetes/kubernetes#133087](https://redirect.github.com/kubernetes/kubernetes/pull/133087),\n[@&#8203;atiratree](https://redirect.github.com/atiratree)) \\[SIG API\nMachinery, Apps and Testing]\n- Scheduler: added a new `bindingTimeout` argument to the\nDynamicResources plugin configuration.\nThis allows customizing the wait duration in PreBind for device binding\nconditions.\nDefaults to 10 minutes when DRADeviceBindingConditions and\nDRAResourceClaimDeviceStatus are both enabled.\n([kubernetes/kubernetes#134905](https://redirect.github.com/kubernetes/kubernetes/pull/134905),\n[@&#8203;fj-naji](https://redirect.github.com/fj-naji)) \\[SIG Node and\nScheduling]\n- The Pod Certificates feature is moving to beta. The\nPodCertificateRequest feature gate is still set false by default. To use\nthe feature, users will need to enable the certificates API groups in\nv1beta1 and enable the feature gate PodCertificateRequest. A new field\nUserAnnotations is added to the PodCertificateProjection API and the\ncorresponding UnverifiedUserAnnotations is added to the\nPodCertificateRequest API.\n([kubernetes/kubernetes#134624](https://redirect.github.com/kubernetes/kubernetes/pull/134624),\n[@&#8203;yt2985](https://redirect.github.com/yt2985)) \\[SIG API\nMachinery, Apps, Auth, Etcd, Instrumentation, Node and Testing]\n- The StrictCostEnforcementForVAP and StrictCostEnforcementForWebhooks\nfeature gates, locked on since 1.32, have been removed\n([kubernetes/kubernetes#134994](https://redirect.github.com/kubernetes/kubernetes/pull/134994),\n[@&#8203;liggitt](https://redirect.github.com/liggitt)) \\[SIG API\nMachinery, Auth, Node and Testing]\n- The `PreferSameZone` and `PreferSameNode` values for Service's\n`trafficDistribution` field are now GA. The old value `PreferClose` is\nnow\ndeprecated in favor of the more-explicit `PreferSameZone`.\n([kubernetes/kubernetes#134457](https://redirect.github.com/kubernetes/kubernetes/pull/134457),\n[@&#8203;danwinship](https://redirect.github.com/danwinship)) \\[SIG API\nMachinery, Apps, Network and Testing]\n- Kube-apiserver: fix a possible panic validating a custom resource\nwhose CustomResourceDefinition indicates a status subresource exists,\nbut which does not define a `status` property in the `openAPIV3Schema`\n([kubernetes/kubernetes#133721](https://redirect.github.com/kubernetes/kubernetes/pull/133721),\n[@&#8203;fusida](https://redirect.github.com/fusida)) \\[SIG API\nMachinery, Apps, Architecture, Auth, Autoscaling, CLI, Cloud Provider,\nCluster Lifecycle, Etcd, Instrumentation, Network, Node, Release,\nScheduling, Storage and Testing]\n- Kubernetes API Go types removed runtime use of the\ngithub.com/gogo/protobuf library, and are no longer registered into the\nglobal gogo type registry. Kubernetes API Go types were not suitable for\nuse with the google.golang.org/protobuf library, and no longer implement\n`ProtoMessage()` by default to avoid accidental incompatible use. If\nremoval of these marker methods impacts your use, it can be re-enabled\nfor one more release with a `kubernetes_protomessage_one_more_release`\nbuild tag, but will be removed in 1.36.\n([kubernetes/kubernetes#134256](https://redirect.github.com/kubernetes/kubernetes/pull/134256),\n[@&#8203;liggitt](https://redirect.github.com/liggitt)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Cluster Lifecycle,\nInstrumentation, Network, Node, Scheduling and Storage]\n- Promoted HPA configurable tolerance to beta. The\n`HPAConfigurableTolerance` feature gate is now enabled by default.\n([kubernetes/kubernetes#133128](https://redirect.github.com/kubernetes/kubernetes/pull/133128),\n[@&#8203;jm-franc](https://redirect.github.com/jm-franc)) \\[SIG API\nMachinery and Autoscaling]\n- The MaxUnavailableStatefulSet feature is now beta and enabled by\ndefault.\n([kubernetes/kubernetes#133153](https://redirect.github.com/kubernetes/kubernetes/pull/133153),\n[@&#8203;helayoty](https://redirect.github.com/helayoty)) \\[SIG API\nMachinery and Apps]\n- Added WithOrigin within apis/core/validation with adjusted tests\n([kubernetes/kubernetes#132825](https://redirect.github.com/kubernetes/kubernetes/pull/132825),\n[@&#8203;PatrickLaabs](https://redirect.github.com/PatrickLaabs)) \\[SIG\nApps]\n- Component-base: validate that log-flush-frequency is positive and\nreturn an error instead of panic-ing\n([kubernetes/kubernetes#133540](https://redirect.github.com/kubernetes/kubernetes/pull/133540),\n[@&#8203;BenTheElder](https://redirect.github.com/BenTheElder)) \\[SIG\nArchitecture, Instrumentation, Network and Node]\n- Feature gate dependencies are now explicit, and validated at startup.\nA feature can no longer be enabled if it depends on a disabled feature.\nIn particular, this means that `AllAlpha=true` will no longer work\nwithout enabling disabled-by-default beta features that are depended on\n(either with `AllBeta=true` or explicitly enumerating the disabled\ndependencies).\n([kubernetes/kubernetes#133697](https://redirect.github.com/kubernetes/kubernetes/pull/133697),\n[@&#8203;tallclair](https://redirect.github.com/tallclair)) \\[SIG API\nMachinery, Architecture, Cluster Lifecycle and Node]\n- In version 1.34, the PodObservedGenerationTracking feature has been\nupgraded to beta, and the description of the alpha version in the\nopenapi has been removed.\n([kubernetes/kubernetes#133883](https://redirect.github.com/kubernetes/kubernetes/pull/133883),\n[@&#8203;yangjunmyfm192085](https://redirect.github.com/yangjunmyfm192085))\n\\[SIG Apps]\n- Introduce a new declarative validation tag +k8s:customUnique to\ncontrol listmap uniqueness\n([kubernetes/kubernetes#134279](https://redirect.github.com/kubernetes/kubernetes/pull/134279),\n[@&#8203;yongruilin](https://redirect.github.com/yongruilin)) \\[SIG API\nMachinery and Auth]\n- Kube-apiserver: Fixed a 1.34 regression in CustomResourceDefinition\nhandling that incorrectly warned about unrecognized formats on number\nand integer properties\n([kubernetes/kubernetes#133896](https://redirect.github.com/kubernetes/kubernetes/pull/133896),\n[@&#8203;yongruilin](https://redirect.github.com/yongruilin)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Cloud Provider, Contributor\nExperience, Network, Node and Scheduling]\n- OpenAPI model packages of API types are generated into\n`zz_generated.model_name.go` files and are accessible using the\n`OpenAPIModelName()` function. This allows API authors to declare the\ndesired OpenAPI model packages instead of using the go package path of\nAPI types.\n([kubernetes/kubernetes#131755](https://redirect.github.com/kubernetes/kubernetes/pull/131755),\n[@&#8203;jpbetz](https://redirect.github.com/jpbetz)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Cloud Provider, Cluster\nLifecycle, Instrumentation, Network, Node, Scheduling, Storage and\nTesting]\n- Support for `kubectl get -o kyaml` is now on by default. To disable\nit, set `KUBECTL_KYAML=false`.\n([kubernetes/kubernetes#133327](https://redirect.github.com/kubernetes/kubernetes/pull/133327),\n[@&#8203;thockin](https://redirect.github.com/thockin)) \\[SIG CLI]\n- The storage version for MutatingAdmissionPolicy is updated to v1beta1.\n([kubernetes/kubernetes#133715](https://redirect.github.com/kubernetes/kubernetes/pull/133715),\n[@&#8203;cici37](https://redirect.github.com/cici37)) \\[SIG API\nMachinery, Etcd and Testing]\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi43NC41IiwidXBkYXRlZEluVmVyIjoiNDIuNzQuNSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-01-19T11:46:28Z",
          "tree_id": "5b1dea8df4cafdb30d91aa76e6283dbb9e3f1228",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c28577df824da63d5759a149df623c30aa108c09"
        },
        "date": 1768825368027,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.5079549551010132,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.38207369938897,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.83207647923273,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.742578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.9140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 559409.8970426436,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 562251.4470787222,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001055,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10164344.031972397,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10108284.727559982,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "2c3976c9672536835e94dae07a4cc7f26333276e",
          "message": "user lowercase for event names (#1816)\n\nhttps://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/docs/telemetry/events-guide.md#event-naming\n\nMoving to lowercase. We are not fully following the guided name yet.\nWill tackle that one module at a time in follow ups.\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-19T12:14:46Z",
          "tree_id": "ed21e6fbb8d8f52aecdf6a40f56b90cb4c53b8e7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2c3976c9672536835e94dae07a4cc7f26333276e"
        },
        "date": 1768827298122,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.09704699367284775,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.3910786047434,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.15395520221863,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.635807291666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.06640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 553925.8106981373,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 554463.379071198,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00353,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10097460.637969457,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10045464.307191519,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "30e7b3e15561011f3f17cb88d4f057849249b58c",
          "message": "chore(deps): update dependency pyarrow to v23 (#1821)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pyarrow](https://redirect.github.com/apache/arrow) | `==22.0.0` →\n`==23.0.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pyarrow/23.0.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pyarrow/22.0.0/23.0.0?slim=true)\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi43NC41IiwidXBkYXRlZEluVmVyIjoiNDIuNzQuNSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-19T17:24:08Z",
          "tree_id": "ddffb8972a81dc0b3ad16c3d0719449e75ff01cf",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/30e7b3e15561011f3f17cb88d4f057849249b58c"
        },
        "date": 1768845689443,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.4229148328304291,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.40198000694161,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.24667153455441,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.960286458333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.58203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 550835.3147336065,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 553164.8791300973,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000917,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9978019.15102052,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9924752.333601108,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "32945cca5f96e7b2d691909fdd615247eb017e5a",
          "message": "chore(deps): update dependency prometheus_client to v0.24.1 (#1819)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[prometheus_client](https://redirect.github.com/prometheus/client_python)\n| `==0.23.1` → `==0.24.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/prometheus-client/0.24.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/prometheus-client/0.23.1/0.24.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>prometheus/client_python (prometheus_client)</summary>\n\n###\n[`v0.24.1`](https://redirect.github.com/prometheus/client_python/releases/tag/v0.24.1)\n\n[Compare\nSource](https://redirect.github.com/prometheus/client_python/compare/v0.24.0...v0.24.1)\n\n- \\[Django] Pass correct registry to MultiProcessCollector by\n[@&#8203;jelly](https://redirect.github.com/jelly) in\n[#&#8203;1152](https://redirect.github.com/prometheus/client_python/pull/1152)\n\n###\n[`v0.24.0`](https://redirect.github.com/prometheus/client_python/releases/tag/v0.24.0)\n\n[Compare\nSource](https://redirect.github.com/prometheus/client_python/compare/v0.23.1...v0.24.0)\n\n##### What's Changed\n\n- Add an AIOHTTP exporter by\n[@&#8203;Lexicality](https://redirect.github.com/Lexicality) in\n[#&#8203;1139](https://redirect.github.com/prometheus/client_python/pull/1139)\n- Add remove\\_matching() method for metric label deletion by\n[@&#8203;hazel-shen](https://redirect.github.com/hazel-shen) in\n[#&#8203;1121](https://redirect.github.com/prometheus/client_python/pull/1121)\n- fix(multiprocess): avoid double-building child metric names\n([#&#8203;1035](https://redirect.github.com/prometheus/client_python/issues/1035))\nby [@&#8203;hazel-shen](https://redirect.github.com/hazel-shen) in\n[#&#8203;1146](https://redirect.github.com/prometheus/client_python/pull/1146)\n- Don't interleave histogram metrics in multi-process collector by\n[@&#8203;cjwatson](https://redirect.github.com/cjwatson) in\n[#&#8203;1148](https://redirect.github.com/prometheus/client_python/pull/1148)\n- Relax registry type annotations for exposition by\n[@&#8203;cjwatson](https://redirect.github.com/cjwatson) in\n[#&#8203;1149](https://redirect.github.com/prometheus/client_python/pull/1149)\n- Added compression support in pushgateway by\n[@&#8203;ritesh-avesha](https://redirect.github.com/ritesh-avesha) in\n[#&#8203;1144](https://redirect.github.com/prometheus/client_python/pull/1144)\n- Add Django exporter\n([#&#8203;1088](https://redirect.github.com/prometheus/client_python/issues/1088))\nby [@&#8203;Chadys](https://redirect.github.com/Chadys) in\n[#&#8203;1143](https://redirect.github.com/prometheus/client_python/pull/1143)\n\n**Full Changelog**:\n<https://github.com/prometheus/client_python/compare/v0.23.1...v0.24.0>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi43NC41IiwidXBkYXRlZEluVmVyIjoiNDIuODUuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-19T18:08:44Z",
          "tree_id": "38e6c40fa1caa51c00879c236988a95415ad332f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32945cca5f96e7b2d691909fdd615247eb017e5a"
        },
        "date": 1768848306447,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.373633474111557,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.07113539685012,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.56654775705996,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.303776041666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.01953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 554964.4746474689,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 557038.0076723521,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001938,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10061600.222022463,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10009320.677595342,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "66651184+utpilla@users.noreply.github.com",
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "2bfa0c9d7a502bcb0b103b66c3749e9a95202907",
          "message": "[syslog-cef-receiver] Add support for parsing tags for RFC 3164 (#1807)\n\nFixes #1729 \n\n## Changes\n- Parse `syslog.tag` field further into `syslog.app_name` and\n`syslog.process_id` when applicable for RFC 3164",
          "timestamp": "2026-01-19T18:16:37Z",
          "tree_id": "f98bbf92402dc99819fd7077543dc1c2e31da057",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2bfa0c9d7a502bcb0b103b66c3749e9a95202907"
        },
        "date": 1768849840041,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.3796670436859131,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.36371525863369,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.98153889104304,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.717838541666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.44140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 555142.1211145121,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 553034.4293378349,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001183,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10044269.67205705,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9992478.296467518,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "1f5104a94f78d7fa1328606fadfd55f63383dd58",
          "message": "fix(deps): update all patch versions (#1802)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) | Type |\nUpdate |\n|---|---|---|---|---|---|\n|\n[github.com/klauspost/compress](https://redirect.github.com/klauspost/compress)\n| `v1.18.2` → `v1.18.3` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fklauspost%2fcompress/v1.18.3?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fklauspost%2fcompress/v1.18.2/v1.18.3?slim=true)\n| require | patch |\n| [go](https://go.dev/)\n([source](https://redirect.github.com/golang/go)) | `1.25.5` → `1.25.6`\n|\n![age](https://developer.mend.io/api/mc/badges/age/golang-version/go/1.25.6?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/golang-version/go/1.25.5/1.25.6?slim=true)\n| toolchain | patch |\n\n---\n\n### Release Notes\n\n<details>\n<summary>klauspost/compress (github.com/klauspost/compress)</summary>\n\n###\n[`v1.18.3`](https://redirect.github.com/klauspost/compress/releases/tag/v1.18.3)\n\n[Compare\nSource](https://redirect.github.com/klauspost/compress/compare/v1.18.2...v1.18.3)\n\nDownstream CVE-2025-61728\n\nSee\n[golang/go#77102](https://redirect.github.com/golang/go/issues/77102)\n\n**Full Changelog**:\n<https://github.com/klauspost/compress/compare/v1.18.2...v1.18.3>\n\n</details>\n\n<details>\n<summary>golang/go (go)</summary>\n\n###\n[`v1.25.6`](https://redirect.github.com/golang/go/compare/go1.25.5...go1.25.6)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi43NC41IiwidXBkYXRlZEluVmVyIjoiNDIuNzQuNSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-19T18:22:57Z",
          "tree_id": "54bea0c49e2d98f01274f5cee1fd84b9b76d79cb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1f5104a94f78d7fa1328606fadfd55f63383dd58"
        },
        "date": 1768851201914,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7399064302444458,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.43238383718906,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.91407850931677,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.01927083333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.12890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 545496.780763707,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 549532.9466607651,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001498,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9939359.258322097,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9895537.623970468,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sachinnb999@gmail.com",
            "name": "Sachin Bansal",
            "username": "Apostlex0"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ede3e1715444e0d20dccf3e7be1f99ecc3f64944",
          "message": "fix: Always materialize the parent IDs when we transform attributes (#1824)\n\nfixes #966\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-19T19:01:58Z",
          "tree_id": "cb212c155c6aba852458914ce0b950b05daf5ce7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ede3e1715444e0d20dccf3e7be1f99ecc3f64944"
        },
        "date": 1768852764513,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7676288485527039,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.95898662144198,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.47497525766777,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.727734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 561365.3999844705,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 565674.6028207779,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001817,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10183417.609587105,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10127210.999977686,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "distinct": false,
          "id": "1b503f3f269f3a1316cd414b1ee72443bc57cc02",
          "message": "Move ObservedEvent into crates/telemetry, consolidated with self_tracing::LogRecord (#1818)\n\nThe ObservedEvent has associated flume channels and a connection with\nthe existing metrics and admin component which make it an appealing way\nto transport log events in the engine.\n\nMove PipelineKey, DeployedPipelineKey, CoreId types into crates/config.\n\nTherefore, moving ObservedEvent into crates/telemetry lets us\n(optionally) use the same channel already use for lifecycle events for\ntokio log records. The existing event structure is extended with a\n`EventMessage` enum which supports None, String, or LogRecord messages.\nThis way we can use a log record as the event message for all existing\nevent types. The `event.rs` file moves, only ObservedEventRingBuffer\nfrom that file remains in crates/state.\n\nThe LogRecord has been storing a timestamp. Now, we leave that to the\nObservedEvent struct. LogRecord passes through SystemTime everywhere it\nhas been used. Callers generally compute this and pass it in. Minor\ncleanup in self_tracing/formatter.rs, do not pass SavedCallsite it can\nbe calculated from record metadata as needed.\n\nIn internal_events, the raw_error! macro has been replaced with a helper\nto generate LogRecord values first, by level. This lets us pass\n`info_event!(\"string\", key=value)` to any of the event constructors and\nconstruct an OTLP bytes message instead of a String message.",
          "timestamp": "2026-01-19T20:34:34Z",
          "tree_id": "6b4eaf69b8e790706f385d4b96e952de7368cc6d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1b503f3f269f3a1316cd414b1ee72443bc57cc02"
        },
        "date": 1768857285011,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0789324045181274,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.94328553005153,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.38665441486069,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.012109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.95703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 555987.20917483,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 561985.935452128,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002071,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10292582.903948208,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10242157.064321138,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "e6f696bcd8427d326ae7546a1aed3d37abd02084",
          "message": "Improve fake-signal-generator to better suit it for use as load generator. (#1857)\n\n# Change Summary\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1817\nFake-generator was generating new batches continuously, taking up CPU\nitself. For load-tests, we want to minimize the CPU taken by the\nfake-generator, so this PR adds additional options to it to re-use\nbatches of telemetry.\nIt also adds option to generate data using hardcodes values, while\nmaintaining existing ability to generate based on OTel semantic\nconventions.\nWhen using hardcoded values, each log is designed to be approximately\n300KB in size, similar to the ones from semantic convention. (We can\nmove to 1 KB size in a future version)\n\n## How are these changes tested?\n\nLocally ran perf tests.\n\n## Are there any user-facing changes?\n\nFake-signal now supports more options.\n\n---------\n\nCo-authored-by: Sachin Bansal <sachinnb999@gmail.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCo-authored-by: Aaron Marten <AaronRM@users.noreply.github.com>",
          "timestamp": "2026-01-22T21:57:45Z",
          "tree_id": "a14d5dd8b3afd698873aafbcce96f2a49397fe9f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e6f696bcd8427d326ae7546a1aed3d37abd02084"
        },
        "date": 1769124727052,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.8944810628890991,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.20443371349192,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 93.57932479777278,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.37447916666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.84765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1145417.4209392504,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1167117.13860306,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001518,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7701254.720600042,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7647227.0623321505,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "bdf8cb02517b41d3cb2bb690bb81b884991d89df",
          "message": "Add shared settings for rust-analyzer in VSCode (#1864)\n\n# Change Summary\n\n- Add a shared `.vscode/settings.json` file with\n`rust-analyzer.linkedProjects` section to allow rust-analyzer to work\nwell with the multi-workspace otel-arrow project by default\n- Add extensions.json so a user is recommended to install rust-analyzer\nwhen opening the project in VSCode\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nValidated that rust-analyzer is able to correctly provide rust-analyzer\nfeature support in VSCode when these settings are applied.\n\n## Are there any user-facing changes?\n\nChanges are only applicable to otel-arrow developers.\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-22T22:12:34Z",
          "tree_id": "a8d39163d45ee7fc764c7874cc61414fbad7f3a0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bdf8cb02517b41d3cb2bb690bb81b884991d89df"
        },
        "date": 1769126044890,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.097011685371399,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.46199631265927,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 93.76936806862972,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.84596354166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.34375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1161228.957518933,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1173967.7749328052,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006826,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7730688.771302045,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7697386.923097801,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "totan@microsoft.com",
            "name": "Tom Tan",
            "username": "ThomsonTan"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a0e5def802f9740cd20dc1370f1dcd32ebd37293",
          "message": "ci: add workflow to mark stale issues and pull requests (#1850)\n\nFixes #1844",
          "timestamp": "2026-01-22T22:38:52Z",
          "tree_id": "604317a2fa0ef3e809984347998cb5bcfc31c2cf",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a0e5def802f9740cd20dc1370f1dcd32ebd37293"
        },
        "date": 1769127346595,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.9304933547973633,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.06676563741209,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 93.67815834917691,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.55416666666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.96875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1148360.0641549719,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1170529.078523509,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001585,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7726933.602533352,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7678681.700927404,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "c9f4e5d1a3249bebfe87f9d1d74c7d91f2ef171b",
          "message": "Add few logs to various components to expose shutdown issue (#1869)\n\n# Change Summary\n\nAdds/improves few internal logs to make the engine more observable. \n\n## How are these changes tested?\n\nLocal, manual runs\n\n## Are there any user-facing changes?\n\nBetter logs!",
          "timestamp": "2026-01-23T00:01:10Z",
          "tree_id": "4bf8a18e1b7205a96c09906a5d55e427142434e8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c9f4e5d1a3249bebfe87f9d1d74c7d91f2ef171b"
        },
        "date": 1769128856283,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.43551063537597656,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.17183398177632,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.4751642245245,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.244140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.0234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1161852.005750818,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1166911.9948770811,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003291,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7717361.552432701,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7667548.255687754,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "716de95f90eedbe19e76db7b3ca7aef58e1274e6",
          "message": "perf: some optimizations for `decode_transport_optimized_ids` (#1873)\n\n# Change Summary\n\nSome optimizations for the `decode_transport_optimized_ids` function,\nwhich is used to remove various forms of delta encoding from ID/Parent\nID columns (converting them to plain encoded arrays).\n\nAlso added a new benchmark for this function.\n\n#### Perf improvement summary/benchmark results:\n`materialize_parent_ids_for_attributes` (synthetic bench)\n\n| Size | Before | After | Improvement | Speedup |\n|------|--------|-------|-------------|---------|\n| 128 | 2.27 µs | 1.77 µs | 22% faster | 1.28× |\n| 1536 | 16.45 µs | 7.97 µs | 51% faster | 2.06× |\n| 8092 | 83.18 µs | 38.21 µs | 54% faster | 2.18× |\n\n`decode_transport_optimized_ids` (generated data using weaver)\n\n| Size | Before | After | Improvement | Speedup |\n|------|--------|-------|-------------|---------|\n| 127 | 15.61 µs | 4.36 µs | 72% faster | 3.58× |\n| 1536 | 54.85 µs | 10.19 µs | 82% faster | 5.38× |\n| 8096 | 229.98 µs | 36.73 µs | 84% faster | 6.26× |\n\nNote that I only tested this on Logs batches, which use a u16 parent_id\nfor attributes. Spans/Metrics have some batches which use u32 IDs (which\nI think we may dictionary encode), and the current code casts these to\nprimitive array.\n\n#### Discussion of optimizations\n\nThe majority of the time is spent in\n`materialize_parent_id_for_attributes` so this is where most of the\neffort was dedicated.\n\nThis function makes heavy use of `create_next_element_equality_array`,\nwhich is used to calculate a bitmask (BooleanArray) indicating which\nrows in a column are equal to the value in the previous row. It does\nthis using arrow's `eq` kernel, which is SIMD optimized. Much of the\nperformance gain came from optimizing how this method is invoked. For\nexample, when invoking it for the \"keys\" column, we were calling `eq` on\nthe DictionaryArray when it should have only been called on the\ndictionary keys.\n\nWe also call `create_next_element_equality_array` for the various values\ncolumns, and we were calling it for every individual range where\ntype/key were equal to one another. This meant an invocation for every\nunique key. This is less efficient than invoking it once per value\ncolumn. Also, since the batches _should_ be sorted by the type column,\nwhen we find that this is indeed the case, we only need to invoke this\non slices of the values columns where the type column indicates the\nattribute value is of a specific type. These ranges can be computed\nefficiently when the batch is sorted, and this PR makes this\noptimization.\n\nAdditionally, we now AND the null validity buffer into the equality bits\nduring the equality array computation. This treats null values as \"not\nequal\" for delta encoding purposes and eliminates the need to check\nnulls separately in the hot decoding loop.\n\nMuch of the rest of the optimization comes from accessing data more\nefficiently. For example, before this change we were using\n`PrimitiveBuilder` to build up the new parent ID column. This is slower\nthan simply copying the values buffer from the existing column into a\nVec and replacing values at indices only where they are delta encoded.\nSimilarly, we were accessing the existing values using\n`MaybeDictArrayAccessor::value_at`, so these method invocations are also\neliminated. A similar optimization is made for removing delta encoding\nfrom the ID column of the logs record batch.\n\nAlso, after we compute the equality bitmasks for various columns, the\nold code was calling BooleanArray::value_at for every index. Arrow has\nsome custom iterators for finding sequences or instances of set bits in\nbit buffers (`BitSliceIterator` and **`BitIndexIterator`**) and this PR\nuses these for yet another performance increase.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* relates to #1853\n\n## How are these changes tested?\n\nThe existing unit tests cover this code\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-01-23T19:12:50Z",
          "tree_id": "c5fac8d51e6536740d46a17d03e94a11f5ddaa0e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/716de95f90eedbe19e76db7b3ca7aef58e1274e6"
        },
        "date": 1769197949136,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.7841233015060425,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 92.35500135662504,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 92.8285277967152,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.571875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.6953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1139743.2587475856,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1160077.6840591931,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001499,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7704534.141314868,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7641460.099143578,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "c432e64e8abf4a85cb540263a81841003611f512",
          "message": "Fix minor syntax warning in beaubourg engine (flagged by rust-analyzer in vscode) (#1874)\n\n# Change Summary\n\nFix a minor syntax warning about unused parens in beaubourg\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nn/a\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-01-23T20:31:46Z",
          "tree_id": "77b94f740adbe782f47149bd9e535761e5c69524",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c432e64e8abf4a85cb540263a81841003611f512"
        },
        "date": 1769202491642,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.7318953275680542,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 91.98650889626101,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 92.18032423751062,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.24205729166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.6796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1132229.88613177,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1151838.9230255939,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001723,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7619385.122986863,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7558299.919251448,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "58180138e1dfd8118f9b7d39cad784272aa50b74",
          "message": "perftest- temporarily remove saturation test with 24 core (#1884)\n\nI am observing similar issue to\nhttps://github.com/open-telemetry/otel-arrow/issues/1870 in the OTLP to\nOTLP scenario in loadtest - for the 24 core SUT, we use 72 core\nLoad-generator, and the load-generator is not shutting down properly. It\nis entirely possible that 72 pipelines instances would need more time to\nshutdown; until this can be investigated, its best to temporarily remove\nthis scenario.\n\nTo unblock perf tests, disabling the 24 core test temporarily. I'll\ninvestigate a proper fix next week.",
          "timestamp": "2026-01-24T18:42:13Z",
          "tree_id": "21e9a816fd1f30ebb4cbb5cde6711f1da69dfe6e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/58180138e1dfd8118f9b7d39cad784272aa50b74"
        },
        "date": 1769282125951,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2808349132537842,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.39624686020507,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.84621341180114,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.97083333333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.01953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 521644.44045715395,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 528325.8445806564,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001759,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11310985.102128187,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11262941.36246068,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "992828ebfbaacf6c41daad362efd95dd7d1b7fcc",
          "message": "chore(deps): update docker.io/rust docker tag to v1.93 (#1888)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| docker.io/rust | stage | minor | `1.92` → `1.93` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-01-26T13:51:23Z",
          "tree_id": "09e19e543b85fcccc074b952676ddcc9ba4115fa",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/992828ebfbaacf6c41daad362efd95dd7d1b7fcc"
        },
        "date": 1769438144061,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.10859670490026474,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.33880804048704,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.84827094566474,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.898046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.9296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 534317.5650125678,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 534897.8162929047,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001591,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11298333.770534985,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11255279.906878412,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "76c2cd4254a4ff1c141e0cbc1035d7dda6085641",
          "message": "chore(deps): update rust crate nix to 0.31.0 (#1889)\n\n> ℹ️ **Note**\n> \n> This PR body was truncated due to platform limits.\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [nix](https://redirect.github.com/nix-rust/nix) |\nworkspace.dependencies | minor | `0.30.1` → `0.31.0` |\n\n---\n\n> [!WARNING]\n> Some dependencies could not be looked up. Check the Dependency\nDashboard for more information.\n\n---\n\n### Release Notes\n\n<details>\n<summary>nix-rust/nix (nix)</summary>\n\n###\n[`v0.31.1`](https://redirect.github.com/nix-rust/nix/blob/HEAD/CHANGELOG.md#0311---2026-01-23)\n\n[Compare\nSource](https://redirect.github.com/nix-rust/nix/compare/v0.31.0...v0.31.1)\n\n##### Added\n\n- termios: Add definition for IUCLC to supported platforms\n  ([#&#8203;2702](https://redirect.github.com/nix-rust/nix/pull/2702))\n- termios: Add definition for XCASE for supported platforms\n  ([#&#8203;2703](https://redirect.github.com/nix-rust/nix/pull/2703))\n\n###\n[`v0.31.0`](https://redirect.github.com/nix-rust/nix/blob/HEAD/CHANGELOG.md#0310---2026-01-22)\n\n[Compare\nSource](https://redirect.github.com/nix-rust/nix/compare/v0.30.1...v0.31.0)\n\n##### Added\n\n- Added the UDP GSO/GRO socket options and CMsgs on Android. This\nincludes the\n  following types:\n\n  - UdpGsoSegment\n  - UdpGroSegment\n  - ControlMessage::UdpGsoSegments\n  - ControlMessageOwned::UdpGroSegments\n\n  ([#&#8203;2666](https://redirect.github.com/nix-rust/nix/pull/2666))\n- Define errno EWOULDBLOCK as an alias of EAGAIN to match the AIX libc\ndefinition.\n([#&#8203;2692](https://redirect.github.com/nix-rust/nix/pull/2692))\n- Enable module `ifaddrs` on GNU Hurd\n  ([#&#8203;2697](https://redirect.github.com/nix-rust/nix/pull/2697))\n- Add termios `OutputFlags::OFILL` for Linux, Android, Aix, Cygwin,\nFuchsia,\n  Haiku,\n  GNU/Hurd, Nto, Redox, Illumos, Solaris and Apple OSes.\n  ([#&#8203;2701](https://redirect.github.com/nix-rust/nix/pull/2701))\n- add sync() for cygwin\n([#&#8203;2708](https://redirect.github.com/nix-rust/nix/pull/2708))\n\n##### Changed\n\n- changed `EpollEvent` methods to be `const`\n  ([#&#8203;2656](https://redirect.github.com/nix-rust/nix/pull/2656))\n- Bumped libc to\n\n[0.2.180](https://redirect.github.com/rust-lang/libc/releases/tag/0.2.180)\n  ([#&#8203;2724](https://redirect.github.com/nix-rust/nix/pull/2724))\n\n##### Fixed\n\n- Fixed `nix::sys::ptrace::syscall_info`, which was not setting the\n`data`\n  argument properly, causing garbage values to be returned.\n  ([#&#8203;2653](https://redirect.github.com/nix-rust/nix/pull/2653))\n- Cast the 'addr' argument of 'madvise()' to '\\*mut u8' on AIX to match\nthe\n  signature in the AIX libc.\n  ([#&#8203;2655](https://redirect.github.com/nix-rust/nix/pull/2655))\n- Fixed the Dir module on NTO, Solaris, Hurd, and possibly other\nplatforms.\n  The\nd\\_name field was not copied correctly on those platforms. For some\nother\nplatforms, it could be copied incorrectly for files with very long\npathnames.\n  ([#&#8203;2674](https://redirect.github.com/nix-rust/nix/pull/2674))\n- Fix the build on Illumos\n([#&#8203;2694](https://redirect.github.com/nix-rust/nix/pull/2694))\n\n##### Removed\n\n- Removed `Eq` and `PartialEq` implementations from `SigHandler`,\nbecause they\n  never worked reliably.  The suggested alternative is `matches!`.  For\n  example:\n  ````\n  let h: SigHandler = ...\n  if matches!(h, SigHandler::SigIgn) {\n      ...\n  }\n``` ([#&#8203;2642](https://redirect.github.com/nix-rust/nix/pull/2642))\n  ````\n- Removed `IFF_NOTRAILERS` by NetBSD, as it has been removed upstream\nand from\nlibc\n([#&#8203;2724](https://redirect.github.com/nix-rust/nix/pull/2724))\n\n#### \\[0.30.1] - 2025-05-04\n\n##### Fixed\n\n- doc.rs build\n  ([#&#8203;2634](https://redirect.github.com/nix-rust/nix/pull/2634))\n\n#### \\[0.30.0] - 2025-04-29\n\n##### Added\n\n- Add socket option `IPV6_PKTINFO` for BSDs/Linux/Android, also\n  `IPV6_RECVPKTINFO` for DragonFlyBSD\n  ([#&#8203;2113](https://redirect.github.com/nix-rust/nix/pull/2113))\n- Add `fcntl`'s `F_PREALLOCATE` constant for Apple targets.\n  ([#&#8203;2393](https://redirect.github.com/nix-rust/nix/pull/2393))\n- Improve support for extracting the TTL / Hop Limit from incoming\npackets\n  and support for DSCP (ToS / Traffic Class).\n  ([#&#8203;2425](https://redirect.github.com/nix-rust/nix/pull/2425))\n- Add socket option IP\\_TOS (nix::sys::socket::sockopt::IpTos)\nIPV6\\_TCLASS\n  (nix::sys::socket::sockopt::Ipv6TClass) on Android/FreeBSD\n  ([#&#8203;2464](https://redirect.github.com/nix-rust/nix/pull/2464))\n- Add `SeekData` and `SeekHole` to `Whence` for hurd and apple targets\n  ([#&#8203;2473](https://redirect.github.com/nix-rust/nix/pull/2473))\n- Add `From` trait implementation between `SocketAddr` and `Sockaddr`,\n`Sockaddr6`\n([#&#8203;2474](https://redirect.github.com/nix-rust/nix/pull/2474))\n- Added wrappers for `posix_spawn` API\n  ([#&#8203;2475](https://redirect.github.com/nix-rust/nix/pull/2475))\n- Add the support for Emscripten.\n  ([#&#8203;2477](https://redirect.github.com/nix-rust/nix/pull/2477))\n- Add fcntl constant `F_RDADVISE` for Apple target\n  ([#&#8203;2480](https://redirect.github.com/nix-rust/nix/pull/2480))\n- Add fcntl constant `F_RDAHEAD` for Apple target\n  ([#&#8203;2482](https://redirect.github.com/nix-rust/nix/pull/2482))\n- Add `F_LOG2PHYS` and `F_LOG2PHYS_EXT` for Apple target\n  ([#&#8203;2483](https://redirect.github.com/nix-rust/nix/pull/2483))\n- `MAP_SHARED_VALIDATE` was added for all linux targets. & `MAP_SYNC`\nwas added\n  for linux with the exclusion of mips architecures, and uclibc\n  ([#&#8203;2499](https://redirect.github.com/nix-rust/nix/pull/2499))\n- Add `getregs()`/`getregset()`/`setregset()` for Linux/musl/aarch64\n  ([#&#8203;2502](https://redirect.github.com/nix-rust/nix/pull/2502))\n- Add FcntlArgs `F_TRANSFEREXTENTS` constant for Apple targets\n  ([#&#8203;2504](https://redirect.github.com/nix-rust/nix/pull/2504))\n- Add `MapFlags::MAP_STACK` in `sys::man` for netbsd\n  ([#&#8203;2526](https://redirect.github.com/nix-rust/nix/pull/2526))\n- Add support for `libc::LOCAL_PEERTOKEN` in `getsockopt`.\n  ([#&#8203;2529](https://redirect.github.com/nix-rust/nix/pull/2529))\n- Add support for `syslog`, `openlog`, `closelog` on all `unix`.\n  ([#&#8203;2537](https://redirect.github.com/nix-rust/nix/pull/2537))\n- Add the `TCP_FUNCTION_BLK` sockopt, on FreeBSD.\n  ([#&#8203;2539](https://redirect.github.com/nix-rust/nix/pull/2539))\n- Implements `Into<OwnedFd>` for\n`PtyMaster/Fanotify/Inotify/SignalFd/TimerFd`\n  ([#&#8203;2548](https://redirect.github.com/nix-rust/nix/pull/2548))\n- Add `MremapFlags::MREMAP_DONTUNMAP` to `sys::mman::mremap` for linux\ntarget.\n  ([#&#8203;2555](https://redirect.github.com/nix-rust/nix/pull/2555))\n- Added `sockopt_impl!` to the public API. It's now possible for users\nto\n  define\n  their own sockopts without needing to make a PR to Nix.\n  ([#&#8203;2556](https://redirect.github.com/nix-rust/nix/pull/2556))\n- Add the `TCP_FUNCTION_ALIAS` sockopt, on FreeBSD.\n  ([#&#8203;2558](https://redirect.github.com/nix-rust/nix/pull/2558))\n- Add `sys::mman::MmapAdvise` `MADV_PAGEOUT`, `MADV_COLD`,\n`MADV_WIPEONFORK`,\n  `MADV_KEEPONFORK` for Linux and Android targets\n  ([#&#8203;2559](https://redirect.github.com/nix-rust/nix/pull/2559))\n- Add socket protocol `Sctp`, as well as `MSG_NOTIFICATION` for\nnon-Android\nLinux targets.\n([#&#8203;2562](https://redirect.github.com/nix-rust/nix/pull/2562))\n- Added `from_owned_fd` constructor to `EventFd`\n  ([#&#8203;2563](https://redirect.github.com/nix-rust/nix/pull/2563))\n- Add `sys::mman::MmapAdvise` `MADV_POPULATE_READ`,\n`MADV_POPULATE_WRITE` for\n  Linux and Android targets\n  ([#&#8203;2565](https://redirect.github.com/nix-rust/nix/pull/2565))\n- Added `from_owned_fd` constructor to\n  `PtyMaster/Fanotify/Inotify/SignalFd/TimerFd`\n  ([#&#8203;2566](https://redirect.github.com/nix-rust/nix/pull/2566))\n- Added `FcntlArg::F_READAHEAD` for FreeBSD target\n  ([#&#8203;2569](https://redirect.github.com/nix-rust/nix/pull/2569))\n- Added `sockopt::LingerSec` for Apple targets\n  ([#&#8203;2572](https://redirect.github.com/nix-rust/nix/pull/2572))\n- Added `sockopt::EsclBind` for solarish targets\n  ([#&#8203;2573](https://redirect.github.com/nix-rust/nix/pull/2573))\n- Exposed the `std::os::fd::AsRawFd` trait method for\n  `nix::sys::fanotify::Fanotify` struct\n  ([#&#8203;2575](https://redirect.github.com/nix-rust/nix/pull/2575))\n- Add support for syslog's `setlogmask` on all `unix`.\n  ([#&#8203;2579](https://redirect.github.com/nix-rust/nix/pull/2579))\n- Added Fuchsia support for `ioctl`.\n  ([#&#8203;2580](https://redirect.github.com/nix-rust/nix/pull/2580))\n- Add `sys::socket::SockProtocol::EthIp`,\n  `sys::socket::SockProtocol::EthIpv6`,\n  `sys::socket::SockProtocol::EthLoop`\n  ([#&#8203;2581](https://redirect.github.com/nix-rust/nix/pull/2581))\n- Add OpenHarmony target into CI and Update documents.\n  ([#&#8203;2599](https://redirect.github.com/nix-rust/nix/pull/2599))\n- Added the TcpMaxSeg `setsockopt` option for apple targets\n  ([#&#8203;2603](https://redirect.github.com/nix-rust/nix/pull/2603))\n- Add `FilAttach` and `FilDetach` to socket::sockopt for Illumos\n  ([#&#8203;2611](https://redirect.github.com/nix-rust/nix/pull/2611))\n- Add `PeerPidfd` (`SO_PEERPIDFD`) to `socket::sockopt` for Linux\n  ([#&#8203;2620](https://redirect.github.com/nix-rust/nix/pull/2620))\n- Added `socket::sockopt::AttachReusePortCbpf` for Linux\n  ([#&#8203;2621](https://redirect.github.com/nix-rust/nix/pull/2621))\n- Add `ptrace::syscall_info` for linux/glibc\n  ([#&#8203;2627](https://redirect.github.com/nix-rust/nix/pull/2627))\n\n##### Changed\n\n- Module sys/signal now adopts I/O safety\n  ([#&#8203;1936](https://redirect.github.com/nix-rust/nix/pull/1936))\n- Change the type of the `name` argument of `memfd_create()` from\n`&CStr` to\n`<P: NixPath>(name: &P)`\n([#&#8203;2431](https://redirect.github.com/nix-rust/nix/pull/2431))\n- Public interfaces in `fcntl.rs` and `dir.rs` now use I/O-safe types.\n  ([#&#8203;2434](https://redirect.github.com/nix-rust/nix/pull/2434))\n- Module `sys/stat` now adopts I/O safety.\n  ([#&#8203;2439](https://redirect.github.com/nix-rust/nix/pull/2439))\n- Module unistd now adopts I/O safety.\n  ([#&#8203;2440](https://redirect.github.com/nix-rust/nix/pull/2440))\n- Module sys/fanotify now adopts I/O safety\n  ([#&#8203;2443](https://redirect.github.com/nix-rust/nix/pull/2443))\n- Socket option `IpTos` has been renamed to `Ipv4Tos`, the old symbol is\ndeprecated since 0.30.0\n([#&#8203;2465](https://redirect.github.com/nix-rust/nix/pull/2465))\n- Rename Flags `EventFlag` to `EvFlags`, and `MemFdCreateFlag` to\n`MFdFlags`\n  ([#&#8203;2476](https://redirect.github.com/nix-rust/nix/pull/2476))\n- Made `nix::sys::socket::UnknownCmsg` public and more readable\n  ([#&#8203;2520](https://redirect.github.com/nix-rust/nix/pull/2520))\n- recvmsg: take slice for cmsg\\_buffer instead of Vec\n  ([#&#8203;2524](https://redirect.github.com/nix-rust/nix/pull/2524))\n- linkat: allow distinct types for path arguments\n  ([#&#8203;2582](https://redirect.github.com/nix-rust/nix/pull/2582))\n\n##### Fixed\n\n- Disable unsupported signals on sparc-linux\n  ([#&#8203;2454](https://redirect.github.com/nix-rust/nix/pull/2454))\n- Fix cmsg\\_len() return type on OpenHarmony\n  ([#&#8203;2456](https://redirect.github.com/nix-rust/nix/pull/2456))\n- The `ns` argument of `sys::prctl::set_timerslack()` should be of type\n`c_ulong`\n([#&#8203;2505](https://redirect.github.com/nix-rust/nix/pull/2505))\n- Properly exclude NUL characters from `OSString`s returned by\n`getsockopt`.\n  ([#&#8203;2557](https://redirect.github.com/nix-rust/nix/pull/2557))\n- Fixes the build on OpenHarmony\n  ([#&#8203;2587](https://redirect.github.com/nix-rust/nix/pull/2587))\n\n##### Removed\n\n- Type `SigevNotify` is no longer `PartialEq`, `Eq` and `Hash` due to\nthe use\nof `BorrowedFd`\n([#&#8203;1936](https://redirect.github.com/nix-rust/nix/pull/1936))\n- `EventFd::defuse()` is removed because it does nothing,\n`EventFd::arm()` is\n  also removed for symmetry reasons.\n  ([#&#8203;2452](https://redirect.github.com/nix-rust/nix/pull/2452))\n- Removed the `Copy` trait from `PollFd`\n  ([#&#8203;2631](https://redirect.github.com/nix-rust/nix/pull/2631))\n\n#### \\[0.29.0] - 2024-05-24\n\n##### Added\n\n- Add `getregset()/setregset()` for\nLinux/glibc/x86/x86\\_64/aarch64/riscv64 and\n  `getregs()/setregs()` for Linux/glibc/aarch64/riscv64\n  ([#&#8203;2044](https://redirect.github.com/nix-rust/nix/pull/2044))\n- Add socket option Ipv6Ttl for apple targets.\n  ([#&#8203;2287](https://redirect.github.com/nix-rust/nix/pull/2287))\n- Add socket option UtunIfname.\n  ([#&#8203;2325](https://redirect.github.com/nix-rust/nix/pull/2325))\n- make SigAction repr(transparent) & can be converted to the libc raw\ntype\n  ([#&#8203;2326](https://redirect.github.com/nix-rust/nix/pull/2326))\n- Add `From` trait implementation for conversions between `sockaddr_in`\nand\n  `SockaddrIn`, `sockaddr_in6` and `SockaddrIn6`\n  ([#&#8203;2328](https://redirect.github.com/nix-rust/nix/pull/2328))\n- Add socket option ReusePortLb for FreeBSD.\n  ([#&#8203;2332](https://redirect.github.com/nix-rust/nix/pull/2332))\n- Added support for openat2 on linux.\n  ([#&#8203;2339](https://redirect.github.com/nix-rust/nix/pull/2339))\n- Add if\\_indextoname function.\n  ([#&#8203;2340](https://redirect.github.com/nix-rust/nix/pull/2340))\n- Add `mount` and `unmount` API for apple targets.\n  ([#&#8203;2347](https://redirect.github.com/nix-rust/nix/pull/2347))\n- Added `_PC_MIN_HOLE_SIZE` for `pathconf` and `fpathconf`.\n  ([#&#8203;2349](https://redirect.github.com/nix-rust/nix/pull/2349))\n- Added `impl AsFd for pty::PtyMaster`\n  ([#&#8203;2355](https://redirect.github.com/nix-rust/nix/pull/2355))\n- Add `open` flag `O_SEARCH` to AIX, Empscripten, FreeBSD, Fuchsia,\nsolarish,\nWASI\n([#&#8203;2374](https://redirect.github.com/nix-rust/nix/pull/2374))\n- Add prctl function `prctl_set_vma_anon_name` for Linux/Android.\n  ([#&#8203;2378](https://redirect.github.com/nix-rust/nix/pull/2378))\n- Add `sync(2)` for `apple_targets/solarish/haiku/aix/hurd`, `syncfs(2)`\nfor\n  `hurd` and `fdatasync(2)` for `aix/hurd`\n  ([#&#8203;2379](https://redirect.github.com/nix-rust/nix/pull/2379))\n- Add fdatasync support for Apple targets.\n  ([#&#8203;2380](https://redirect.github.com/nix-rust/nix/pull/2380))\n- Add `fcntl::OFlag::O_PATH` for FreeBSD and Fuchsia\n  ([#&#8203;2382](https://redirect.github.com/nix-rust/nix/pull/2382))\n- Added `PathconfVar::MIN_HOLE_SIZE` for apple\\_targets.\n  ([#&#8203;2388](https://redirect.github.com/nix-rust/nix/pull/2388))\n- Add `open` flag `O_SEARCH` to apple\\_targets\n  ([#&#8203;2391](https://redirect.github.com/nix-rust/nix/pull/2391))\n- `O_DSYNC` may now be used with `aio_fsync` and `fcntl` on FreeBSD.\n  ([#&#8203;2404](https://redirect.github.com/nix-rust/nix/pull/2404))\n- Added `Flock::relock` for upgrading and downgrading locks.\n  ([#&#8203;2407](https://redirect.github.com/nix-rust/nix/pull/2407))\n\n##### Changed\n\n- Change the `ForkptyResult` type to the following repr so that the\n  uninitialized\n  `master` field won't be accessed in the child process:\n\n  ````rs\n  pub enum ForkptyResult {\n      Parent {\n          child: Pid,\n          master: OwnedFd,\n      },\n      Child,\n  }\n``` ([#&#8203;2315](https://redirect.github.com/nix-rust/nix/pull/2315))\n  ````\n- Updated `cfg_aliases` dependency from version 0.1 to 0.2\n  ([#&#8203;2322](https://redirect.github.com/nix-rust/nix/pull/2322))\n- Change the signature of `ptrace::write` and `ptrace::write_user` to\nmake them\nsafe\n([#&#8203;2324](https://redirect.github.com/nix-rust/nix/pull/2324))\n- Allow use of `SignalFd` through shared reference\n\nLike with many other file descriptors, concurrent use of signalfds is\nsafe.\nChanging the signal mask of and reading signals from a signalfd can now\nbe\n  done\n  with the `SignalFd` API even if other references to it exist.\n  ([#&#8203;2367](https://redirect.github.com/nix-rust/nix/pull/2367))\n- Changed tee, splice and vmsplice RawFd arguments to AsFd.\n  ([#&#8203;2387](https://redirect.github.com/nix-rust/nix/pull/2387))\n- Added I/O safety to the sys/aio module. Most functions that previously\n  accepted a `AsRawFd` argument now accept an `AsFd` instead.\n  ([#&#8203;2401](https://redirect.github.com/nix-rust/nix/pull/2401))\n- `RecvMsg::cmsgs()` now returns a `Result`, and checks that cmsgs were\nnot\ntruncated.\n([#&#8203;2413](https://redirect.github.com/nix-rust/nix/pull/2413))\n\n##### Fixed\n\n- No longer panics when the `fanotify` queue overflows.\n  ([#&#8203;2399](https://redirect.github.com/nix-rust/nix/pull/2399))\n- Fixed ControlMessageOwned::UdpGroSegments wrapped type from u16 to i32\nto\n  reflect the used kernel's one.\n  ([#&#8203;2406](https://redirect.github.com/nix-rust/nix/pull/2406))\n\n#### \\[0.28.0] - 2024-02-24\n\n##### Added\n\n- Added `mkdtemp` wrapper\n([#&#8203;1297](https://redirect.github.com/nix-rust/nix/pull/1297))\n- Add associated constants `UTIME_OMIT` `UTIME_NOW` for `TimeSpec`\n  ([#&#8203;1879](https://redirect.github.com/nix-rust/nix/pull/1879))\n- Added `EventFd` type.\n([#&#8203;1945](https://redirect.github.com/nix-rust/nix/pull/1945))\n- - Added `impl From<Signal> for SigSet`.\n  - Added `impl std::ops::BitOr for SigSet`.\n  - Added `impl std::ops::BitOr for Signal`.\n  - Added `impl std::ops::BitOr<Signal> for SigSet`\n\n  ([#&#8203;1959](https://redirect.github.com/nix-rust/nix/pull/1959))\n- Added `TlsGetRecordType` control message type and corresponding enum\nfor\nlinux\n([#&#8203;2065](https://redirect.github.com/nix-rust/nix/pull/2065))\n- Added `Ipv6HopLimit` to `::nix::sys::socket::ControlMessage` for\nLinux,\n  MacOS, FreeBSD, DragonflyBSD, Android, iOS and Haiku.\n  ([#&#8203;2074](https://redirect.github.com/nix-rust/nix/pull/2074))\n- Added `Icmp` and `IcmpV6` to `SockProtocol`\n  ([#&#8203;2103](https://redirect.github.com/nix-rust/nix/pull/2103))\n- Added rfork support for FreeBSD in `unistd`\n  ([#&#8203;2121](https://redirect.github.com/nix-rust/nix/pull/2121))\n- Added `MapFlags::map_hugetlb_with_size_log2` method for Linux targets\n  ([#&#8203;2125](https://redirect.github.com/nix-rust/nix/pull/2125))\n- Added `mmap_anonymous` function\n  ([#&#8203;2127](https://redirect.github.com/nix-rust/nix/pull/2127))\n- Added `mips32r6` and `mips64r6` support for signal, ioctl and ptrace\n  ([#&#8203;2138](https://redirect.github.com/nix-rust/nix/pull/2138))\n- Added `F_GETPATH` FcntlFlags entry on Apple/NetBSD/DragonflyBSD for\n`::nix::fcntl`.\n([#&#8203;2142](https://redirect.github.com/nix-rust/nix/pull/2142))\n- Added `F_KINFO` FcntlFlags entry on FreeBSD for `::nix::fcntl`.\n  ([#&#8203;2152](https://redirect.github.com/nix-rust/nix/pull/2152))\n- Added `F_GETPATH_NOFIRMLINK` and `F_BARRIERFSYNC` FcntlFlags entry\n  on Apple for `::nix::fcntl`.\n  ([#&#8203;2155](https://redirect.github.com/nix-rust/nix/pull/2155))\n- Added newtype `Flock` to automatically unlock a held flock upon drop.\n  Added `Flockable` trait to represent valid types for `Flock`.\n  ([#&#8203;2170](https://redirect.github.com/nix-rust/nix/pull/2170))\n- Added `SetSockOpt` impls to enable Linux Kernel TLS on a TCP socket\nand to\nimport TLS parameters.\n([#&#8203;2175](https://redirect.github.com/nix-rust/nix/pull/2175))\n- - Added the `::nix::sys::socket::SocketTimestamp` enum for configuring\nthe\n    `TsClock` (a.k.a `SO_TS_CLOCK`) sockopt\n  - Added FreeBSD's `ScmRealtime` and `ScmMonotonic` as new options in\n    `::nix::sys::socket::ControlMessageOwned`\n\n  ([#&#8203;2187](https://redirect.github.com/nix-rust/nix/pull/2187))\n- Added new fanotify API: wrappers for `fanotify_init` and\n`fanotify_mark`\n  ([#&#8203;2194](https://redirect.github.com/nix-rust/nix/pull/2194))\n- Added `SpecialCharacterindices` support for haiku.\n  ([#&#8203;2195](https://redirect.github.com/nix-rust/nix/pull/2195))\n- Added `sys::sendfile` support for solaris/illumos.\n  ([#&#8203;2198](https://redirect.github.com/nix-rust/nix/pull/2198))\n- impl Display for InterfaceFlags\n  ([#&#8203;2206](https://redirect.github.com/nix-rust/nix/pull/2206))\n- Added `sendfilev` in sys::sendfile for solarish\n  ([#&#8203;2207](https://redirect.github.com/nix-rust/nix/pull/2207))\n- Added `fctrl::SealFlag::F_SEAL_FUTURE_WRITE`\n  ([#&#8203;2213](https://redirect.github.com/nix-rust/nix/pull/2213))\n- Added `Ipv6MulticastHops` as socket option to set and read.\n  ([#&#8203;2234](https://redirect.github.com/nix-rust/nix/pull/2234))\n- Enable `ControlMessageOwned::Ipv4RecvIf` and\n  `ControlMessageOwned::Ipv4RecvDstAddr` for DragonFlyBSD\n  ([#&#8203;2240](https://redirect.github.com/nix-rust/nix/pull/2240))\n- `ClockId::set_time()` and `time::clock_settime()` are now enabled on\nmacOS\n  ([#&#8203;2241](https://redirect.github.com/nix-rust/nix/pull/2241))\n- Added `IpBindAddressNoPort` sockopt to support\n`IP_BIND_ADDRESS_NO_PORT`\navailable on linux.\n([#&#8203;2244](https://redirect.github.com/nix-rust/nix/pull/2244))\n- Enable `MapFlags::map_hugetlb_with_size_log2` method for\nAndroid/Fuchsia\n  ([#&#8203;2245](https://redirect.github.com/nix-rust/nix/pull/2245))\n- Added `TcpFastOpenConnect` sockopt to support `TCP_FASTOPEN_CONNECT`\navailable on linux.\n([#&#8203;2247](https://redirect.github.com/nix-rust/nix/pull/2247))\n- Add `reboot(2)` for OpenBSD/NetBSD\n  ([#&#8203;2251](https://redirect.github.com/nix-rust/nix/pull/2251))\n- Added new `MemFdCreateFlag` constants to `sys::memfd` on Linux and\nAndroid\n  related to hugetlbfs support.\n  ([#&#8203;2252](https://redirect.github.com/nix-rust/nix/pull/2252))\n- Expose the inner fd of `Kqueue` through:\n\n  - impl AsFd for Kqueue\n  - impl From\\<Kqueue> for OwnedFd\n\n  ([#&#8203;2258](https://redirect.github.com/nix-rust/nix/pull/2258))\n- Added `sys::eventfd` support on FreeBSD\n  ([#&#8203;2259](https://redirect.github.com/nix-rust/nix/pull/2259))\n- Added `MmapFlags::MAP_FIXED` constant in `sys::mman` for netbsd and\nopenbsd\n  ([#&#8203;2260](https://redirect.github.com/nix-rust/nix/pull/2260))\n- Added the `SO_LISTENQLIMIT` sockopt.\n  ([#&#8203;2263](https://redirect.github.com/nix-rust/nix/pull/2263))\n- Enable the `AT_EMPTY_PATH` flag for the `fchownat()` function\n  ([#&#8203;2267](https://redirect.github.com/nix-rust/nix/pull/2267))\n- Add `AtFlags::AT_EMPTY_PATH` for FreeBSD and Hurd\n  ([#&#8203;2270](https://redirect.github.com/nix-rust/nix/pull/2270))\n- Enable \\`OFlag::O\\_DIRECTORY for Solarish\n  ([#&#8203;2275](https://redirect.github.com/nix-rust/nix/pull/2275))\n- Added the `Backlog` wrapper type for the `listen` call.\n  ([#&#8203;2276](https://redirect.github.com/nix-rust/nix/pull/2276))\n- Add `clock_nanosleep()`\n([#&#8203;2277](https://redirect.github.com/nix-rust/nix/pull/2277))\n- Enabled `O_DIRECT` in `fcntl::OFlags` for solarish\n  ([#&#8203;2278](https://redirect.github.com/nix-rust/nix/pull/2278))\n- Added a new API sigsuspend.\n  ([#&#8203;2279](https://redirect.github.com/nix-rust/nix/pull/2279))\n- - Added `errno::Errno::set` function\n  - Added `errno::Errno::set_raw` function\n  - Added `errno::Errno::last_raw` function\n  - Added `errno::Errno::from_raw` function\n\n  ([#&#8203;2283](https://redirect.github.com/nix-rust/nix/pull/2283))\n- Enable the `AT_EMPTY_PATH` flag for the `linkat()` function\n  ([#&#8203;2284](https://redirect.github.com/nix-rust/nix/pull/2284))\n- Enable unistd::{sync, syncfs} for Android\n  ([#&#8203;2296](https://redirect.github.com/nix-rust/nix/pull/2296))\n\n##### Changed\n\n- `poll` now takes `PollTimeout` replacing `libc::c_int`.\n  ([#&#8203;1876](https://redirect.github.com/nix-rust/nix/pull/1876))\n- Deprecated `sys::eventfd::eventfd`.\n  ([#&#8203;1945](https://redirect.github.com/nix-rust/nix/pull/1945))\n- `mmap`, `mmap_anonymous`, `munmap`, `mremap`, `madvise`, `msync`,\n`mprotect`,\n  `munlock` and `mlock` updated to use `NonNull`.\n  ([#&#8203;2000](https://redirect.github.com/nix-rust/nix/pull/2000))\n- `mmap` function now accepts `F` instead of `Option<F>`\n  ([#&#8203;2127](https://redirect.github.com/nix-rust/nix/pull/2127))\n- `PollFd::new` now takes a `BorrowedFd` argument, with relaxed lifetime\n  requirements relative to the previous version.\n  ([#&#8203;2134](https://redirect.github.com/nix-rust/nix/pull/2134))\n- `FdSet::{insert, remove, contains}` now take `BorrowedFd` arguments,\nand have\n  relaxed lifetime requirements relative to 0.27.1.\n  ([#&#8203;2136](https://redirect.github.com/nix-rust/nix/pull/2136))\n- The following APIs now take an implementation of `AsFd` rather than a\n  `RawFd`:\n\n  - `unistd::tcgetpgrp`\n  - `unistd::tcsetpgrp`\n  - `unistd::fpathconf`\n  - `unistd::ttyname`\n- `unistd::getpeereid`\n([#&#8203;2137](https://redirect.github.com/nix-rust/nix/pull/2137))\n- Changed `openat()` and `Dir::openat()`, now take optional `dirfd`s\n  ([#&#8203;2139](https://redirect.github.com/nix-rust/nix/pull/2139))\n- The MSRV is now 1.69\n([#&#8203;2144](https://redirect.github.com/nix-rust/nix/pull/2144))\n- Changed function `SockaddrIn::ip()` to return `net::Ipv4Addr` and\nrefactored\n  `SocketAddrV6::ip()` to be `const`\n  ([#&#8203;2151](https://redirect.github.com/nix-rust/nix/pull/2151))\n- The following APIs now take optional `dirfd`s:\n\n  - `readlinkat()`\n  - `fstatat()`\n  - `mknodat()`\n  - `mkdirat()`\n  - `execveat()`\n\n  ([#&#8203;2157](https://redirect.github.com/nix-rust/nix/pull/2157))\n- `Epoll::wait` now takes `EpollTimeout` replacing `isize`.\n  ([#&#8203;2202](https://redirect.github.com/nix-rust/nix/pull/2202))\n- - Deprecated `errno::errno()` function (use `Errno::last_raw()`)\n  - Deprecated `errno::from_i32()` function (use `Errno::from_raw()`)\n- Deprecated `errno::Errno::from_i32()` function (use\n`Errno::from_raw()`)\n\n  ([#&#8203;2283](https://redirect.github.com/nix-rust/nix/pull/2283))\n\n##### Fixed\n\n- Fix `SigSet` incorrect implementation of `Eq`, `PartialEq` and `Hash`\n  ([#&#8203;1946](https://redirect.github.com/nix-rust/nix/pull/1946))\n- Fixed `::sys::socket::sockopt::IpMulticastTtl` by fixing the value of\noptlen\n  passed to `libc::setsockopt` and added tests.\n  ([#&#8203;2072](https://redirect.github.com/nix-rust/nix/pull/2072))\n- Fixed the function signature of `recvmmsg`, potentially causing UB\n  ([#&#8203;2119](https://redirect.github.com/nix-rust/nix/pull/2119))\n- Fix `SignalFd::set_mask`.  In 0.27.0 it would actually close the file\ndescriptor.\n([#&#8203;2141](https://redirect.github.com/nix-rust/nix/pull/2141))\n- Fixed UnixAddr::new for haiku, it did not record the `sun_len` value\nas\n  needed.\n  Fixed `sys::socket::addr::from_raw_parts` and\n  `sys::socket::Sockaddrlike::len` build for solaris.\n  ([#&#8203;2242](https://redirect.github.com/nix-rust/nix/pull/2242))\n- Fixed solaris build globally.\n  ([#&#8203;2248](https://redirect.github.com/nix-rust/nix/pull/2248))\n- Changed the `dup3` wrapper to perform a real call to `dup3` instead of\n  emulating it via `dup2` and `fcntl` to get rid of race condition\n  ([#&#8203;2268](https://redirect.github.com/nix-rust/nix/pull/2268))\n- Fixed `::unistd::Group::members` using read\\_unaligned to avoid crash\non\nmisaligned pointers\n([#&#8203;2311](https://redirect.github.com/nix-rust/nix/pull/2311))\n\n##### Removed\n\n- The `FchownatFlags` type has been deprecated, please use `AtFlags`\ninstead.\n  ([#&#8203;2267](https://redirect.github.com/nix-rust/nix/pull/2267))\n- Removed the `dup3` wrapper on macOS, which was emulated via `dup2` and\n`fcntl` and could cause a race condition. The `dup3` system call is not\nsupported on macOS.\n([#&#8203;2268](https://redirect.github.com/nix-rust/nix/pull/2268))\n- The `LinkatFlags` type has been deprecated, please use `AtFlags`\ninstead.\n  ([#&#8203;2284](https://redirect.github.com/nix-rust/nix/pull/2284))\n\n#### \\[0.27.1] - 2023-08-28\n\n##### Fixed\n\n- Fixed generating the documentation on docs.rs.\n  ([#&#8203;2111](https://redirect.github.com/nix-rust/nix/pull/2111))\n\n#### \\[0.27.0] - 2023-08-28\n\n##### Added\n\n- Added `AT_EACCESS` to `AtFlags` on all platforms but android\n  ([#&#8203;1995](https://redirect.github.com/nix-rust/nix/pull/1995))\n- Add `PF_ROUTE` to `SockType` on macOS, iOS, all of the BSDs, Fuchsia,\nHaiku, Illumos.\n  ([#&#8203;1867](https://redirect.github.com/nix-rust/nix/pull/1867))\n- Added `nix::ucontext` module on `aarch64-unknown-linux-gnu`.\n  (#[1662](https://redirect.github.com/nix-rust/nix/pull/1662))\n- Added `CanRaw` to `SockProtocol` and `CanBcm` as a separate\n`SocProtocol` constant.\n  ([#&#8203;1912](https://redirect.github.com/nix-rust/nix/pull/1912))\n- Added `Generic` and `NFLOG` to `SockProtocol`.\n  ([#&#8203;2092](https://redirect.github.com/nix-rust/nix/pull/2092))\n- Added `mq_timedreceive` to `::nix::mqueue`.\n\n(\\[[#&#8203;1966](https://redirect.github.com/nix-rust/nix/issues/1966)])([#&#8203;1966](https://redirect.github.com/nix-rust/nix/pull/1966))\n- Added `LocalPeerPid` to `nix::sys::socket::sockopt` for macOS.\n([#&#8203;1967](https://redirect.github.com/nix-rust/nix/pull/1967))\n- Added `TFD_TIMER_CANCEL_ON_SET` to\n`::nix::sys::time::TimerSetTimeFlags` on Linux and Android.\n  ([#&#8203;2040](https://redirect.github.com/nix-rust/nix/pull/2040))\n- Added `SOF_TIMESTAMPING_OPT_ID` and `SOF_TIMESTAMPING_OPT_TSONLY` to\n`nix::sys::socket::TimestampingFlag`.\n  ([#&#8203;2048](https://redirect.github.com/nix-rust/nix/pull/2048))\n- Enabled socket timestamping options on Android.\n([#&#8203;2077](https://redirect.github.com/nix-rust/nix/pull/2077))\n- Added vsock support for macOS\n([#&#8203;2056](https://redirect.github.com/nix-rust/nix/pull/2056))\n- Added `SO_SETFIB` and `SO_USER_COOKIE` to `nix::sys::socket::sockopt`\nfor FreeBSD.\n  ([#&#8203;2085](https://redirect.github.com/nix-rust/nix/pull/2085))\n- Added `SO_RTABLE` for OpenBSD and `SO_ACCEPTFILTER` for FreeBSD/NetBSD\nto `nix::sys::socket::sockopt`.\n  ([#&#8203;2085](https://redirect.github.com/nix-rust/nix/pull/2085))\n- Added `MSG_WAITFORONE` to `MsgFlags` on Android, Fuchsia, Linux,\nNetBSD,\n  FreeBSD, OpenBSD, and Solaris.\n  ([#&#8203;2014](https://redirect.github.com/nix-rust/nix/pull/2014))\n- Added `SO_TS_CLOCK` for FreeBSD to `nix::sys::socket::sockopt`.\n  ([#&#8203;2093](https://redirect.github.com/nix-rust/nix/pull/2093))\n- Added support for prctl in Linux.\n  (#[1550](https://redirect.github.com/nix-rust/nix/pull/1550))\n- `nix::socket` and `nix::select` are now available on Redox.\n  ([#&#8203;2012](https://redirect.github.com/nix-rust/nix/pull/2012))\n- Implemented AsFd, AsRawFd, FromRawFd, and IntoRawFd for\n`mqueue::MqdT`.\n  ([#&#8203;2097](https://redirect.github.com/nix-rust/nix/pull/2097))\n- Add the ability to set `kevent_flags` on `SigEvent`.\n  ([#&#8203;1731](https://redirect.github.com/nix-rust/nix/pull/1731))\n\n##### Changed\n\n- All Cargo features have been removed from the default set. Users will\nneed to\n  specify which features they depend on in their Cargo.toml.\n  ([#&#8203;2091](https://redirect.github.com/nix-rust/nix/pull/2091))\n- Implemented I/O safety for many, but not all, of Nix's APIs. Many\npublic\n  functions argument and return types have changed:\n\n  | Original Type | New Type              |\n  | ------------- | --------------------- |\n  | AsRawFd       | AsFd                  |\n  | RawFd         | BorrowedFd or OwnedFd |\n\n  (#[1906](https://redirect.github.com/nix-rust/nix/pull/1906))\n- Use I/O safety with `copy_file_range`, and expose it on FreeBSD.\n  (#[1906](https://redirect.github.com/nix-rust/nix/pull/1906))\n- The MSRV is now 1.65\n  ([#&#8203;1862](https://redirect.github.com/nix-rust/nix/pull/1862))\n  ([#&#8203;2104](https://redirect.github.com/nix-rust/nix/pull/2104))\n- The epoll interface now uses a type.\n  ([#&#8203;1882](https://redirect.github.com/nix-rust/nix/pull/1882))\n- With I/O-safe type applied in `pty::OpenptyResult` and\n`pty::ForkptyResult`,\nusers no longer need to manually close the file descriptors in these\ntypes.\n  ([#&#8203;1921](https://redirect.github.com/nix-rust/nix/pull/1921))\n- Refactored `name` parameter of `mq_open` and `mq_unlink` to be generic\nover\n  `NixPath`.\n  ([#&#8203;2102](https://redirect.github.com/nix-rust/nix/pull/2102)).\n- Made `clone` unsafe, like `fork`.\n  ([#&#8203;1993](https://redirect.github.com/nix-rust/nix/pull/1993))\n\n##### Removed\n\n- `sys::event::{kevent, kevent_ts}` are deprecated in favor of\n`sys::kevent::Kqueue::kevent`, and `sys::event::kqueue` is deprecated in\n  favor of `sys::kevent::Kqueue::new`.\n  ([#&#8203;1943](https://redirect.github.com/nix-rust/nix/pull/1943))\n- Removed deprecated IoVec API.\n  ([#&#8203;1855](https://redirect.github.com/nix-rust/nix/pull/1855))\n- Removed deprecated net APIs.\n  ([#&#8203;1861](https://redirect.github.com/nix-rust/nix/pull/1861))\n- `nix::sys::signalfd::signalfd` is deprecated.  Use\n  `nix::sys::signalfd::SignalFd` instead.\n  ([#&#8203;1938](https://redirect.github.com/nix-rust/nix/pull/1938))\n- Removed `SigEvent` support on Fuchsia, where it was unsound.\n  ([#&#8203;2079](https://redirect.github.com/nix-rust/nix/pull/2079))\n- Removed `flock` from `::nix::fcntl` on Solaris.\n  ([#&#8203;2082](https://redirect.github.com/nix-rust/nix/pull/2082))\n\n#### \\[0.26.3] - 2023-08-27\n\n##### Fixed\n\n- Fix: send `ETH_P_ALL` in htons format\n  ([#&#8203;1925](https://redirect.github.com/nix-rust/nix/pull/1925))\n- Fix: `recvmsg` now sets the length of the received `sockaddr_un` field\ncorrectly on Linux platforms.\n([#&#8203;2041](https://redirect.github.com/nix-rust/nix/pull/2041))\n- Fix potentially invalid conversions in\n  `SockaddrIn::from<std::net::SocketAddrV4>`,\n`SockaddrIn6::from<std::net::SockaddrV6>`, `IpMembershipRequest::new`,\nand\n  `Ipv6MembershipRequest::new` with future Rust versions.\n  ([#&#8203;2061](https://redirect.github.com/nix-rust/nix/pull/2061))\n- Fixed an incorrect lifetime returned from `recvmsg`.\n  ([#&#8203;2095](https://redirect.github.com/nix-rust/nix/pull/2095))\n\n#### \\[0.26.2] - 2023-01-18\n\n##### Fixed\n\n- Fix `SockaddrIn6` bug that was swapping `flowinfo` and `scope_id` byte\n  ordering.\n  ([#&#8203;1964](https://redirect.github.com/nix-rust/nix/pull/1964))\n\n#### \\[0.26.1] - 2022-11-29\n\n##### Fixed\n\n- Fix UB with `sys::socket::sockopt::SockType` using `SOCK_PACKET`.\n  ([#&#8203;1821](https://redirect.github.com/nix-rust/nix/pull/1821))\n\n#### \\[0.26.0] - 2022-11-29\n\n##### Added\n\n- Added `SockaddrStorage::{as_unix_addr, as_unix_addr_mut}`\n  ([#&#8203;1871](https://redirect.github.com/nix-rust/nix/pull/1871))\n- Added `MntFlags` and `unmount` on all of the BSDs.\n- Added `any()` and `all()` to `poll::PollFd`.\n  ([#&#8203;1877](https://redirect.github.com/nix-rust/nix/pull/1877))\n- Add `MntFlags` and `unmount` on all of the BSDs.\n  ([#&#8203;1849](https://redirect.github.com/nix-rust/nix/pull/1849))\n- Added a `Statfs::flags` method.\n  ([#&#8203;1849](https://redirect.github.com/nix-rust/nix/pull/1849))\n- Added `NSFS_MAGIC` FsType on Linux and Android.\n  ([#&#8203;1829](https://redirect.github.com/nix-rust/nix/pull/1829))\n- Added `sched_getcpu` on platforms that support it.\n  ([#&#8203;1825](https://redirect.github.com/nix-rust/nix/pull/1825))\n- Added `sched_getaffinity` and `sched_setaffinity` on FreeBSD.\n  ([#&#8203;1804](https://redirect.github.com/nix-rust/nix/pull/1804))\n- Added `line_discipline` field to `Termios` on Linux, Android and Haiku\n  ([#&#8203;1805](https://redirect.github.com/nix-rust/nix/pull/1805))\n- Expose the memfd module on FreeBSD (memfd was added in FreeBSD 13)\n  ([#&#8203;1808](https://redirect.github.com/nix-rust/nix/pull/1808))\n- Added `domainname` field of `UtsName` on Android and Linux\n  ([#&#8203;1817](https://redirect.github.com/nix-rust/nix/pull/1817))\n- Re-export `RLIM_INFINITY` from `libc`\n  ([#&#8203;1831](https://redirect.github.com/nix-rust/nix/pull/1831))\n- Added `syncfs(2)` on Linux\n  ([#&#8203;1833](https://redirect.github.com/nix-rust/nix/pull/1833))\n- Added `faccessat(2)` on illumos\n  ([#&#8203;1841](https://redirect.github.com/nix-rust/nix/pull/1841))\n- Added `eaccess()` on FreeBSD, DragonFly and Linux (glibc and musl).\n  ([#&#8203;1842](https://redirect.github.com/nix-rust/nix/pull/1842))\n- Added `IP_TOS` `SO_PRIORITY` and `IPV6_TCLASS` sockopts for Linux\n  ([#&#8203;1853](https://redirect.github.com/nix-rust/nix/pull/1853))\n- Added `new_unnamed` and `is_unnamed` for `UnixAddr` on Linux and\nAndroid.\n  ([#&#8203;1857](https://redirect.github.com/nix-rust/nix/pull/1857))\n- Added `SockProtocol::Raw` for raw sockets\n  ([#&#8203;1848](https://redirect.github.com/nix-rust/nix/pull/1848))\n- added `IP_MTU` (`IpMtu`) `IPPROTO_IP` sockopt on Linux and Android.\n  ([#&#8203;1865](https://redirect.github.com/nix-rust/nix/pull/1865))\n\n##### Changed\n\n- The MSRV is now 1.56.1\n  ([#&#8203;1792](https://redirect.github.com/nix-rust/nix/pull/1792))\n- The `addr` argument of `sys::mman::mmap` is now of type\n`Option<NonZeroUsize>`.\n  ([#&#8203;1870](https://redirect.github.com/nix-rust/nix/pull/1870))\n- The `length` argument of `sys::mman::mmap` is now of type\n`NonZeroUsize`.\n  ([#&#8203;1873](https://redirect.github.com/nix-rust/nix/pull/1873))\n\n##### Fixed\n\n- Fixed using `SockaddrStorage` to store a Unix-domain socket address on\nLinux.\n  ([#&#8203;1871](https://redirect.github.com/nix-rust/nix/pull/1871))\n- Fix microsecond calculation for `TimeSpec`.\n  ([#&#8203;1801](https://redirect.github.com/nix-rust/nix/pull/1801))\n- Fix `User::from_name` and `Group::from_name` panicking\n  when given a name containing a nul.\n  ([#&#8203;1815](https://redirect.github.com/nix-rust/nix/pull/1815))\n- Fix `User::from_uid` and `User::from_name` crash on Android platform.\n  ([#&#8203;1824](https://redirect.github.com/nix-rust/nix/pull/1824))\n- Workaround XNU bug causing netmasks returned by `getifaddrs` to\nmisbehave.\n  ([#&#8203;1788](https://redirect.github.com/nix-rust/nix/pull/1788))\n\n##### Removed\n\n- Removed deprecated error constants and conversions.\n  ([#&#8203;1860](https://redirect.github.com/nix-rust/nix/pull/1860))\n\n#### \\[0.25.0] - 2022-08-13\n\n##### Added\n\n- Added `faccessat`\n  ([#&#8203;1780](https://redirect.github.com/nix-rust/nix/pull/1780))\n- Added `memfd` on Android.\n  (#[1773](https://redirect.github.com/nix-rust/nix/pull/1773))\n- Added `ETH_P_ALL` to `SockProtocol` enum\n  (#[1768](https://redirect.github.com/nix-rust/nix/pull/1768))\n- Added four non-standard Linux `SysconfVar` variants\n  (#[1761](https://redirect.github.com/nix-rust/nix/pull/1761))\n- Added const constructors for `TimeSpec` and `TimeVal`\n  (#[1760](https://redirect.github.com/nix-rust/nix/pull/1760))\n- Added `chflags`.\n  (#[1758](https://redirect.github.com/nix-rust/nix/pull/1758))\n- Added `aio_writev` and `aio_readv`.\n  (#[1713](https://redirect.github.com/nix-rust/nix/pull/1713))\n- impl `From<uid_t>` for `Uid` and `From<gid_t>` for `Gid`\n  (#[1727](https://redirect.github.com/nix-rust/nix/pull/1727))\n- impl `From<SockaddrIn>` for `std::net::SocketAddrV4` and\n  impl `From<SockaddrIn6>` for `std::net::SocketAddrV6`.\n  (#[1711](https://redirect.github.com/nix-rust/nix/pull/1711))\n- Added support for the `x86_64-unknown-haiku` target.\n  (#[1703](https://redirect.github.com/nix-rust/nix/pull/1703))\n- Added `ptrace::read_user` and `ptrace::write_user` for Linux.\n  (#[1697](https://redirect.github.com/nix-rust/nix/pull/1697))\n- Added `getrusage` and helper types `UsageWho` and `Usage`\n  (#[1747](https://redirect.github.com/nix-rust/nix/pull/1747))\n- Added the `DontRoute` SockOpt\n  (#[1752](https://redirect.github.com/nix-rust/nix/pull/1752))\n- Added `signal::SigSet::from_sigset_t_unchecked()`.\n  (#[1741](https://redirect.github.com/nix-rust/nix/pull/1741))\n- Added the `Ipv4OrigDstAddr` sockopt and control message.\n  (#[1772](https://redirect.github.com/nix-rust/nix/pull/1772))\n- Added the `Ipv6OrigDstAddr` sockopt and control message.\n  (#[1772](https://redirect.github.com/nix-rust/nix/pull/1772))\n- Added the `Ipv4SendSrcAddr` control message.\n  (#[1776](https://redirect.github.com/nix-rust/nix/pull/1776))\n\n##### Changed\n\n- Reimplemented sendmmsg/recvmmsg to avoid allocations and with better\nAPI\n  (#[1744](https://redirect.github.com/nix-rust/nix/pull/1744))\n\n- Rewrote the aio module.  The new module:\n  - Does more type checking at compile time rather than runtime.\n- Gives the caller control over whether and when to `Box` an aio\noperation.\n  - Changes the type of the `priority` arguments to `i32`.\n  - Changes the return type of `aio_return` to `usize`.\n    (#[1713](https://redirect.github.com/nix-rust/nix/pull/1713))\n\n- `nix::poll::ppoll`: `sigmask` parameter is now optional.\n  (#[1739](https://redirect.github.com/nix-rust/nix/pull/1739))\n\n- Changed `gethostname` to return an owned `OsString`.\n  (#[1745](https://redirect.github.com/nix-rust/nix/pull/1745))\n\n- `signal:SigSet` is now marked as `repr(transparent)`.\n  (#[1741](https://redirect.github.com/nix-rust/nix/pull/1741))\n\n##### Removed\n\n- Removed support for resubmitting partially complete `lio_listio`\noperations.\nIt was too complicated, and didn't fit Nix's theme of zero-cost\nabstractions.\n  Instead, it can be reimplemented downstream.\n  (#[1713](https://redirect.github.com/nix-rust/nix/pull/1713))\n\n#### \\[0.24.2] - 2022-07-17\n\n##### Fixed\n\n- Fixed buffer overflow in `nix::sys::socket::recvfrom`.\n  (#[1763](https://redirect.github.com/nix-rust/nix/pull/1763))\n- Enabled `SockaddrStorage::{as_link_addr, as_link_addr_mut}` for\nLinux-like\n  operating systems.\n  (#[1729](https://redirect.github.com/nix-rust/nix/pull/1729))\n- Fixed `SockaddrLike::from_raw` implementations for `VsockAddr` and\n  `SysControlAddr`.\n  (#[1736](https://redirect.github.com/nix-rust/nix/pull/1736))\n\n#### \\[0.24.1] - 2022-04-22\n\n##### Fixed\n\n- Fixed `UnixAddr::size` on Linux-based OSes.\n  (#[1702](https://redirect.github.com/nix-rust/nix/pull/1702))\n\n#### \\[0.24.0] - 2022-04-21\n\n##### Added\n\n- Added fine-grained features flags.  Most Nix functionality can now be\n  conditionally enabled.  By default, all features are enabled.\n  (#[1611](https://redirect.github.com/nix-rust/nix/pull/1611))\n- Added statfs FS type magic constants for `target_os = \"android\"`\n  and synced constants with libc v0.2.121.\n  (#[1690](https://redirect.github.com/nix-rust/nix/pull/1690))\n- Added `fexecve` on DragonFly.\n  (#[1577](https://redirect.github.com/nix-rust/nix/pull/1577))\n- `sys::uio::IoVec` is now `Send` and `Sync`\n  (#[1582](https://redirect.github.com/nix-rust/nix/pull/1582))\n- Added `EPOLLEXCLUSIVE` on Android.\n  (#[1567](https://redirect.github.com/nix-rust/nix/pull/1567))\n- Added `fdatasync` for FreeBSD, Fuchsia, NetBSD, and OpenBSD.\n  (#[1581](https://redirect.github.com/nix-rust/nix/pull/1581))\n- Added `sched_setaffinity` and `sched_getaffinity` on DragonFly.\n  (#[1537](https://redirect.github.com/nix-rust/nix/pull/1537))\n- Added `posix_fallocate` on DragonFly.\n  (#[1621](https://redirect.github.com/nix-rust/nix/pull/1621))\n- Added `SO_TIMESTAMPING` support\n  (#[1547](https://redirect.github.com/nix-rust/nix/pull/1547))\n- Added getter methods to `MqAttr` struct\n  (#[1619](https://redirect.github.com/nix-rust/nix/pull/1619))\n- Added the `TxTime` sockopt and control message.\n  (#[1564](https://redirect.github.com/nix-rust/nix/pull/1564))\n- Added POSIX per-process timer support\n  (#[1622](https://redirect.github.com/nix-rust/nix/pull/1622))\n- Added `sendfile` on DragonFly.\n  (#[1615](https://redirect.github.com/nix-rust/nix/pull/1615))\n- Added `UMOUNT_NOFOLLOW`, `FUSE_SUPER_MAGIC` on Linux.\n  (#[1634](https://redirect.github.com/nix-rust/nix/pull/1634))\n- Added `getresuid`, `setresuid`, `getresgid`, and `setresgid` on\nDragonFly, FreeBSD, and OpenBSD.\n  (#[1628](https://redirect.github.com/nix-rust/nix/pull/1628))\n- Added `MAP_FIXED_NOREPLACE` on Linux.\n  (#[1636](https://redirect.github.com/nix-rust/nix/pull/1636))\n- Added `fspacectl` on FreeBSD\n  (#[1640](https://redirect.github.com/nix-rust/nix/pull/1640))\n- Added `accept4` on DragonFly, Emscripten, Fuchsia, Illumos, and\nNetBSD.\n  (#[1654](https://redirect.github.com/nix-rust/nix/pull/1654))\n- Added `AsRawFd` implementation on `OwningIter`.\n  (#[1563](https://redirect.github.com/nix-rust/nix/pull/1563))\n- Added `process_vm_readv` and `process_vm_writev` on Android.\n  (#[1557](https://redirect.github.com/nix-rust/nix/pull/1557))\n- Added `nix::ucontext` module on s390x.\n  (#[1662](https://redirect.github.com/nix-rust/nix/pull/1662))\n- Implemented `Extend`, `FromIterator`, and `IntoIterator` for `SigSet`\nand\n  added `SigSet::iter` and `SigSetIter`.\n  (#[1553](https://redirect.github.com/nix-rust/nix/pull/1553))\n- Added `ENOTRECOVERABLE` and `EOWNERDEAD` error codes on DragonFly.\n  (#[1665](https://redirect.github.com/nix-rust/nix/pull/1665))\n- Implemented `Read` and `Write` for `&PtyMaster`\n  (#[1664](https://redirect.github.com/nix-rust/nix/pull/1664))\n- Added `MSG_NOSIGNAL` for Android, Dragonfly, FreeBSD, Fuchsia, Haiku,\nIllumos, Linux, NetBSD, OpenBSD and Solaris.\n  (#[1670](https://redirect.github.com/nix-rust/nix/pull/1670))\n- Added `waitid`.\n  (#[1584](https://redirect.github.com/nix-rust/nix/pull/1584))\n- Added `Ipv6DontFrag` for android, iOS, linux and macOS.\n- Added `IpDontFrag` for iOS, macOS.\n  (#[1692](https://redirect.github.com/nix-rust/nix/pull/1692))\n\n##### Changed\n\n- `mqueue` functions now operate on a distinct type,\n`nix::mqueue::MqdT`.\n  Accessors take this type by reference, not by value.\n  (#[1639](https://redirect.github.com/nix-rust/nix/pull/1639))\n- Removed `SigSet::extend` in favor of `<SigSet as\nExtend<Signal>>::extend`.\nBecause of this change, you now need `use std::iter::Extend` to call\n`extend`\n  on a `SigSet`.\n  (#[1553](https://redirect.github.com/nix-rust/nix/pull/1553))\n- Removed the the `PATH_MAX` restriction from APIs accepting paths.\nPaths\nwill now be allocated on the heap if they are too long. In addition,\nlarge\n  instruction count improvements (\\~30x) were made to path handling.\n  (#[1656](https://redirect.github.com/nix-rust/nix/pull/1656))\n- Changed `getrlimit` and `setrlimit` to use `rlim_t` directly\n  instead of `Option<rlim_t>`.\n  (#[1668](https://redirect.github.com/nix-rust/nix/pull/1668))\n- Deprecated `InetAddr` and `SockAddr` in favor of `SockaddrIn`,\n`SockaddrIn6`,\n  and `SockaddrStorage`.\n  (#[1684](https://redirect.github.com/nix-rust/nix/pull/1684))\n- Deprecated `IpAddr`, `Ipv4Addr`, and `Ipv6Addr` in favor of their\nequivalents\n  from the standard library.\n  (#[1685](https://redirect.github.com/nix-rust/nix/pull/1685))\n- `uname` now returns a `Result<UtsName>` instead of just a `UtsName`\nand\nignoring failures from libc. And getters on the `UtsName` struct now\nreturn\n  an `&OsStr` instead of `&str`.\n  (#[1672](https://redirect.github.com/nix-rust/nix/pull/1672))\n- Replaced `IoVec` with `IoSlice` and `IoSliceMut`, and replaced\n`IoVec::from_slice` with\n`IoSlice::new`.\n(#[1643](https://redirect.github.com/nix-rust/nix/pull/1643))\n\n##### Fixed\n\n- `InetAddr::from_std` now sets the `sin_len`/`sin6_len` fields on the\nBSDs.\n  (#[1642](https://redirect.github.com/nix-rust/nix/pull/1642))\n- Fixed a panic in `LinkAddr::addr`. That function now returns an\n`Option`.\n  (#[1675](https://redirect.github.com/nix-rust/nix/pull/1675))\n  (#[1677](https://redirect.github.com/nix-rust/nix/pull/1677))\n\n##### Removed\n\n- Removed public access to the inner fields of `NetlinkAddr`, `AlgAddr`,\n  `SysControlAddr`, `LinkAddr`, and `VsockAddr`.\n  (#[1614](https://redirect.github.com/nix-rust/nix/pull/1614))\n- Removed `EventFlag::EV_SYSFLAG`.\n  (#[1635](https://redirect.github.com/nix-rust/nix/pull/1635))\n\n#### \\[0.23.1] - 2021-12-16\n\n##### Changed\n\n- Relaxed the bitflags requirement from 1.3.1 to 1.1. This partially\nreverts\n[#&#8203;1492](https://redirect.github.com/nix-rust/nix/issues/1492).\nFrom now on, the MSRV is not guaranteed to work with all versions of\n  all dependencies, just with some version of all dependencies.\n  (#[1607](https://redirect.github.com/nix-rust/nix/pull/1607))\n\n##### Fixed\n\n- Fixed soundness issues in `FdSet::insert`, `FdSet::remove`, and\n  `FdSet::contains` involving file descriptors outside of the range\n  `0..FD_SETSIZE`.\n  (#[1575](https://redirect.github.com/nix-rust/nix/pull/1575))\n\n#### \\[0.23.0] - 2021-09-28\n\n##### Added\n\n- Added the `LocalPeerCred` sockopt.\n  (#[1482](https://redirect.github.com/nix-rust/nix/pull/1482))\n- Added `TimeSpec::from_duration` and `TimeSpec::from_timespec`\n  (#[1465](https://redirect.github.com/nix-rust/nix/pull/1465))\n- Added `IPV6_V6ONLY` sockopt.\n  (#[1470](https://redirect.github.com/nix-rust/nix/pull/1470))\n- Added `impl From<User> for libc::passwd` trait implementation to\nconvert a `User`\ninto a `libc::passwd`. Consumes the `User` struct to give ownership over\n  the member pointers.\n  (#[1471](https://redirect.github.com/nix-rust/nix/pull/1471))\n- Added `pthread_kill`.\n  (#[1472](https://redirect.github.com/nix-rust/nix/pull/1472))\n- Added `mknodat`.\n  (#[1473](https://redirect.github.com/nix-rust/nix/pull/1473))\n- Added `setrlimit` and `getrlimit`.\n  (#[1302](https://redirect.github.com/nix-rust/nix/pull/1302))\n- Added `ptrace::interrupt` method for platforms that support\n`PTRACE_INTERRUPT`\n  (#[1422](https://redirect.github.com/nix-rust/nix/pull/1422))\n- Added `IP6T_SO_ORIGINAL_DST` sockopt.\n  (#[1490](https://redirect.github.com/nix-rust/nix/pull/1490))\n- Added the `PTRACE_EVENT_STOP` variant to the `sys::ptrace::Event` enum\n  (#[1335](https://redirect.github.com/nix-rust/nix/pull/1335))\n- Exposed `SockAddr::from_raw_sockaddr`\n  (#[1447](https://redirect.github.com/nix-rust/nix/pull/1447))\n- Added `TcpRepair`\n  (#[1503](https://redirect.github.com/nix-rust/nix/pull/1503))\n- Enabled `pwritev` and `preadv` for more operating systems.\n  (#[1511](https://redirect.github.com/nix-rust/nix/pull/1511))\n- Added support for `TCP_MAXSEG` TCP Maximum Segment Size socket options\n  (#[1292](https://redirect.github.com/nix-rust/nix/pull/1292))\n- Added `Ipv4RecvErr` and `Ipv6RecvErr` sockopts and associated control\nmessages.\n  (#[1514](https://redirect.github.com/nix-rust/nix/pull/1514))\n- Added `AsRawFd` implementation on `PollFd`.\n  (#[1516](https://redirect.github.com/nix-rust/nix/pull/1516))\n- Added `Ipv4Ttl` and `Ipv6Ttl` sockopts.\n  (#[1515](https://redirect.github.com/nix-rust/nix/pull/1515))\n- Added `MAP_EXCL`, `MAP_ALIGNED_SUPER`, and `MAP_CONCEAL` mmap flags,\nand\n  exposed `MAP_ANONYMOUS` for all operating systems.\n  (#[1522](https://redirect.github.com/nix-rust/nix/pull/1522))\n  (#[1525](https://redirect.github.com/nix-rust/nix/pull/1525))\n  (#[1531](https://redirect.github.com/nix-rust/nix/pull/1531))\n  (#[1534](https://redirect.github.com/nix-rust/nix/pull/1534))\n- Added read/write accessors for 'events' on `PollFd`.\n  (#[1517](https://redirect.github.com/nix-rust/nix/pull/1517))\n\n##### Changed\n\n- `FdSet::{contains, highest, fds}` no longer require a mutable\nreference.\n  (#[1464](https://redirect.github.com/nix-rust/nix/pull/1464))\n- `User::gecos` and corresponding `libc::passwd::pw_gecos` are supported\non\n  64-bit Android, change conditional compilation to include the field in\n  64-bit Android builds\n  (#[1471](https://redirect.github.com/nix-rust/nix/pull/1471))\n- `eventfd`s are supported on Android, change conditional compilation to\ninclude `sys::eventfd::eventfd` and `sys::eventfd::EfdFlags`for Android\n  builds.\n  (#[1481](https://redirect.github.com/nix-rust/nix/pull/1481))\n- Most enums that come from C, for example `Errno`, are now marked as\n  `#[non_exhaustive]`.\n  (#[1474](https://redirect.github.com/nix-rust/nix/pull/1474))\n- Many more functions, mostly contructors, are now `const`.\n  (#[1476](https://redirect.github.com/nix-rust/nix/pull/1476))\n  (#[1492](https://redirect.github.com/nix-rust/nix/pull/1492))\n- `sys::event::KEvent::filter` now returns a `Result` instead of being\ninfalliable. The only cases where it will now return an error are cases\n  where it previously would've had undefined behavior.\n  (#[1484](https://redirect.github.com/nix-rust/nix/pull/1484))\n- Minimum supported Rust version is now 1.46.0.\n  ([#&#8203;1492](https://redirect.github.com/nix-rust/nix/pull/1492))\n- Rework `UnixAddr` to encapsulate internals better in order to fix\nsoundness\nissues. No longer allows creating a `UnixAddr` from a raw `sockaddr_un`.\n  ([#&#8203;1496](https://redirect.github.com/nix-rust/nix/pull/1496))\n- Raised bitflags to 1.3.0 and the MSRV to 1.46.0.\n  ([#&#8203;1492](https://redirect.github.com/nix-rust/nix/pull/1492))\n\n##### Fixed\n\n- `posix_fadvise` now returns errors in the conventional way, rather\nthan as a\n  non-zero value in `Ok()`.\n  (#[1538](https://redirect.github.com/nix-rust/nix/pull/1538))\n- Added more errno definitions for better backwards compatibility with\n  Nix 0.21.0.\n  (#[1467](https://redirect.github.com/nix-rust/nix/pull/1467))\n- Fixed potential undefined behavior in `Signal::try_from` on some\nplatforms.\n  (#[1484](https://redirect.github.com/nix-rust/nix/pull/1484))\n- Fixed buffer overflow in `unistd::getgrouplist`.\n  (#[1545](https://redirect.github.com/nix-rust/nix/pull/1545))\n\n##### Removed\n\n- Removed a couple of termios constants on redox that were never\nactually\n  supported.\n  (#[1483](https://redirect.github.com/nix-rust/nix/pull/1483))\n- Removed `nix::sys::signal::NSIG`. It was of dubious utility, and not\ncorrect\n  for all platforms.\n  (#[1484](https://redirect.github.com/nix-rust/nix/pull/1484))\n- Removed support for 32-bit Apple targets, since they've been dropped\nby both\n  Rustc and Xcode.\n  (#[1492](https://redirect.github.com/nix-rust/nix/pull/1492))\n- Deprecated `SockAddr/InetAddr::to_str` in favor of\n`ToString::to_string`\n  (#[1495](https://redirect.github.com/nix-rust/nix/pull/1495))\n- Removed `SigevNotify` on OpenBSD and Redox.\n  (#[1511](https://redirect.github.com/nix-rust/nix/pull/1511))\n\n#### \\[0.22.3] - 22 January 2022\n\n##### Changed\n\n- Relaxed the bitflags requirement from 1.3.1 to 1.1. This partially\nreverts\n[#&#8203;1492](https://redirect.github.com/nix-rust/nix/issues/1492).\nFrom now on, the MSRV is not guaranteed to work with all versions of\n  all dependencies, just with some version of all dependencies.\n  (#[1607](https://redirect.github.com/nix-rust/nix/pull/1607))\n\n#### \\[0.22.2] - 28 September 2021\n\n##### Fixed\n\n- Fixed buffer overflow in `unistd::getgrouplist`.\n  (#[1545](https://redirect.github.com/nix-rust/nix/pull/1545))\n- Added more errno definitions for better backwards compatibility with\n  Nix 0.21.0.\n  (#[1467](https://redirect.github.com/nix-rust/nix/pull/1467))\n\n#### \\[0.22.1] - 13 August 2021\n\n##### Fixed\n\n- Locked bitflags to < 1.3.0 to fix the build with rust < 1.46.0.\n\n##### Removed\n\n- Removed a couple of termios constants on redox that were never\nactually\n  supported.\n  (#[1483](https://redirect.github.com/nix-rust/nix/pull/1483))\n\n#### \\[0.22.0] - 9 July 2021\n\n##### Added\n\n- Added `if_nameindex`\n(#[1445](https://redirect.github.com/nix-rust/nix/pull/1445))\n- Added `nmount` for FreeBSD.\n  (#[1453](https://redirect.github.com/nix-rust/nix/pull/1453))\n- Added `IpFreebind` socket option (sockopt) on Linux, Fuchsia and\nAndroid.\n  (#[1456](https://redirect.github.com/nix-rust/nix/pull/1456))\n- Added `TcpUserTimeout` socket option (sockopt) on Linux and Fuchsia.\n  (#[1457](https://redirect.github.com/nix-rust/nix/pull/1457))\n- Added `renameat2` for Linux\n  (#[1458](https://redirect.github.com/nix-rust/nix/pull/1458))\n- Added `RxqOvfl` support on Linux, Fuchsia and Android.\n  (#[1455](https://redirect.github.com/nix-rust/nix/pull/1455))\n\n##### Changed\n\n- `ptsname_r` now returns a lossily-converted string in the event of bad\nUTF,\n  just like `ptsname`.\n  ([#&#8203;1446](https://redirect.github.com/nix-rust/nix/pull/1446))\n- Nix's error type is now a simple wrapper around the platform's Errno.\nThis\nmeans it is now `Into<std::io::Error>`. It's also `Clone`, `Copy`, `Eq`,\nand\nhas a small fixed size. It also requires less typing. For example, the\nold\nenum variant `nix::Error::Sys(nix::errno::Errno::EINVAL)` is now simply\n  `nix::Error::EINVAL`.\n  ([#&#8203;1446](https://redirect.github.com/nix-rust/nix/pull/1446))\n\n#### \\[0.21.2] - 29 September 2021\n\n##### Fixed\n\n- Fixed buffer overflow in `unistd::getgrouplist`.\n  (#[1545](https://redirect.github.com/nix-rust/nix/pull/1545))\n\n#### \\[0.21.1] - 13 August 2021\n\n##### Fixed\n\n- Locked bitflags to < 1.3.0 to fix the build with rust < 1.46.0.\n\n##### Removed\n\n- Removed a couple of termios constants on redox that were never\nactually\n  supported.\n  (#[1483](https://redirect.github.com/nix-rust/nix/pull/1483))\n\n#### \\[0.21.0] - 31 May 2021\n\n##### Added\n\n- Added `getresuid` and `getresgid`\n  (#[1430](https://redirect.github.com/nix-rust/nix/pull/1430))\n- Added TIMESTAMPNS support for linux\n  (#[1402](https://redirect.github.com/nix-rust/nix/pull/1402))\n- Added `sendfile64`\n(#[1439](https://redirect.github.com/nix-rust/nix/pull/1439))\n- Added `MS_LAZYTIME` to `MsFlags`\n  (#[1437](https://redirect.github.com/nix-rust/nix/pull/1437))\n\n##### Changed\n\n- Made `forkpty` unsafe, like `fork`\n  (#[1390](https://redirect.github.com/nix-rust/nix/pull/1390))\n- Made `Uid`, `Gid` and `Pid` methods `from_raw` and `as_raw` a `const\nfn`\n  (#[1429](https://redirect.github.com/nix-rust/nix/pull/1429))\n- Made `Uid::is_root` a `const fn`\n  (#[1429](https://redirect.github.com/nix-rust/nix/pull/1429))\n- `AioCb` is now always pinned. Once a `libc::aiocb` gets sent to the\nkernel,\n  its address in memory must not change.  Nix now enforces that by using\n`std::pin`. Most users won't need to change anything, except when using\n  `aio_suspend`.  See that method's documentation fo\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-26T14:21:39Z",
          "tree_id": "a9242ed3107581dd6865df85112a0cb645902126",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/76c2cd4254a4ff1c141e0cbc1035d7dda6085641"
        },
        "date": 1769439491458,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.2343301624059677,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.63318663309535,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.00224990865459,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.54283854166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.0703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 535297.5519592857,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 534043.1883149253,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001739,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11315110.373882614,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11265638.664061336,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "4bd8858b6c8047dc1d641268f501d1457b60c07f",
          "message": "chore(deps): upgrade azure crates to 0.31.0 (#1892)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nUpgrades azure crates to 0.31.0\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* ~Closes #NNN~\n\nSupersedes renovate PR\nhttps://github.com/open-telemetry/otel-arrow/pull/1887\n\n## How are these changes tested?\n\nexisting tests\n\n## Are there any user-facing changes?\n\nno",
          "timestamp": "2026-01-26T15:56:24Z",
          "tree_id": "d4d8bf76a6fa9df62da504f5b4d6e3390a3f54ac",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4bd8858b6c8047dc1d641268f501d1457b60c07f"
        },
        "date": 1769447793193,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.0689190924167633,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.47896018982092,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.00081739130435,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.780338541666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.1171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 532396.7729840125,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 532763.6960068089,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001686,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11506601.625083733,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11450110.172457373,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "3f0c85c4d65a91562de3165088edececc378f0eb",
          "message": "fix(deps): update module go.opentelemetry.io/collector/pdata to v1.50.0 (#1890)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.49.0` → `v1.50.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.50.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.49.0/v1.50.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.50.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1500v01440)\n\n##### 🛑 Breaking changes 🛑\n\n- `pkg/exporterhelper`: Change verbosity level for\notelcol\\_exporter\\_queue\\_batch\\_send\\_size metric to detailed.\n([#&#8203;14278](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14278))\n- `pkg/service`: Remove deprecated\n`telemetry.disableHighCardinalityMetrics` feature gate.\n([#&#8203;14373](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14373))\n- `pkg/service`: Remove deprecated `service.noopTracerProvider` feature\ngate.\n([#&#8203;14374](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14374))\n\n##### 🚩 Deprecations 🚩\n\n- `exporter/otlp_grpc`: Rename `otlp` exporter to `otlp_grpc` exporter\nand add deprecated alias `otlp`.\n([#&#8203;14403](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14403))\n- `exporter/otlp_http`: Rename `otlphttp` exporter to `otlp_http`\nexporter and add deprecated alias `otlphttp`.\n([#&#8203;14396](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14396))\n\n##### 💡 Enhancements 💡\n\n- `cmd/builder`: Avoid duplicate CLI error logging in generated\ncollector binaries by relying on cobra's error handling.\n([#&#8203;14317](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14317))\n\n- `cmd/mdatagen`: Add the ability to disable attributes at the metric\nlevel and re-aggregate data points based off of these new dimensions\n([#&#8203;10726](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/10726))\n\n- `cmd/mdatagen`: Add optional `display_name` and `description` fields\nto metadata.yaml for human-readable component names\n([#&#8203;14114](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14114))\nThe `display_name` field allows components to specify a human-readable\nname in metadata.yaml.\nWhen provided, this name is used as the title in generated README files.\nThe `description` field allows components to include a brief description\nin generated README files.\n\n- `cmd/mdatagen`: Validate stability level for entities\n([#&#8203;14425](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14425))\n\n- `pkg/xexporterhelper`: Reenable batching for profiles\n([#&#8203;14313](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14313))\n\n- `receiver/nop`: add profiles signal support\n([#&#8203;14253](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14253))\n\n##### 🧰 Bug fixes 🧰\n\n- `pkg/exporterhelper`: Fix reference count bug in partition batcher\n([#&#8203;14444](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14444))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-26T16:09:46Z",
          "tree_id": "6a3491a6ee07525b4a94648a89771dfa8016ffd5",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3f0c85c4d65a91562de3165088edececc378f0eb"
        },
        "date": 1769448935314,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.32272785902023315,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.70747394345375,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.34644299930358,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.227864583333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.31640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 523521.2999501832,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 525210.8490665957,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001807,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11350255.431555135,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11286210.911037732,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "4431eac5b920adcde80feb83ec32289fcb7eb31f",
          "message": "Fix cargo warning about 'profiles for the non root package will be ignored' (#1897)\n\n# Change Summary\n\nFixes a cargo warning about 'profiles for the non root package will be\nignored' from the `query-engine` crate.\n\n## What issue does this PR close?\nn/a\n\n## How are these changes tested?\nValidated that build warning is resolved\n\n## Are there any user-facing changes?\nNo.",
          "timestamp": "2026-01-27T16:29:34Z",
          "tree_id": "bbb6476c4a7413943919972e391048abdce9b468",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4431eac5b920adcde80feb83ec32289fcb7eb31f"
        },
        "date": 1769533662634,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.3727128505706787,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.09547841178238,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.48463485398334,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.42747395833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.44921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 520903.98788884043,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 528054.5037512226,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003503,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11447318.343064303,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11401969.178601988,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "6996b41185f183d8b66f9f287cb6361d1791840f",
          "message": "Implement no-op for update_rusage_metrics on unsupported platforms (#1896)\n\n# Change Summary\n\nImplement no-op for update_rusage_metrics on unsupported platforms to\nfix the `error: field rusage_thread_supported is never read` build\nwarning on macos, Windows, etc...\n\n## What issue does this PR close?\n* Closes #1858\n\n## How are these changes tested?\nVerified that build warning is fixed on macos\n\n## Are there any user-facing changes?\nNo",
          "timestamp": "2026-01-27T16:36:09Z",
          "tree_id": "8a672c022e6f6197373e2f10718e75a0fb3cdea4",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6996b41185f183d8b66f9f287cb6361d1791840f"
        },
        "date": 1769534994629,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0623586177825928,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.09381207859272,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.55879148671741,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.415234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.36328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 524502.6124713646,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 530074.711417853,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001806,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11452567.383806719,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11398465.231301717,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "e1c7a802b626d7c8a6061e9f1a3ced60ac9417eb",
          "message": "fix(deps): update all patch versions (#1894)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [duckdb](https://redirect.github.com/duckdb/duckdb-python)\n([changelog](https://redirect.github.com/duckdb/duckdb-python/releases))\n| `==1.4.3` → `==1.4.4` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/duckdb/1.4.4?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/duckdb/1.4.3/1.4.4?slim=true)\n|\n|\n[github.com/apache/arrow-go/v18](https://redirect.github.com/apache/arrow-go)\n| `v18.5.0` → `v18.5.1` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fapache%2farrow-go%2fv18/v18.5.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fapache%2farrow-go%2fv18/v18.5.0/v18.5.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>duckdb/duckdb-python (duckdb)</summary>\n\n###\n[`v1.4.4`](https://redirect.github.com/duckdb/duckdb-python/releases/tag/v1.4.4):\nBugfix Release\n\n[Compare\nSource](https://redirect.github.com/duckdb/duckdb-python/compare/v1.4.3...v1.4.4)\n\n**DuckDB core v1.4.4 Changelog**:\n<https://github.com/duckdb/duckdb/compare/v1.4.3...v1.4.4>\n\n#### What's Changed in the Python Extension\n\n- fix polars tests by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;218](https://redirect.github.com/duckdb/duckdb-python/pull/218)\n- tests for string and binary views by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;221](https://redirect.github.com/duckdb/duckdb-python/pull/221)\n- Quote view names in unregister by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;222](https://redirect.github.com/duckdb/duckdb-python/pull/222)\n- Limit string nodes in Polars expressions to constant expressions by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;225](https://redirect.github.com/duckdb/duckdb-python/pull/225)\n- Escape identifiers in relation aggregations by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;272](https://redirect.github.com/duckdb/duckdb-python/pull/272)\n- Fix DECREF bug during interpreter shutdown by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;275](https://redirect.github.com/duckdb/duckdb-python/pull/275)\n- Support for Pandas 3.0.0 by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;277](https://redirect.github.com/duckdb/duckdb-python/pull/277)\n- Prepare for v1.4.4 by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;280](https://redirect.github.com/duckdb/duckdb-python/pull/280)\n\n**Full Changelog**:\n<https://github.com/duckdb/duckdb-python/compare/v1.4.3...v1.4.4>\n\n</details>\n\n<details>\n<summary>apache/arrow-go (github.com/apache/arrow-go/v18)</summary>\n\n###\n[`v18.5.1`](https://redirect.github.com/apache/arrow-go/releases/tag/v18.5.1)\n\n[Compare\nSource](https://redirect.github.com/apache/arrow-go/compare/v18.5.0...v18.5.1)\n\n#### What's Changed\n\n- fix(internal): fix assertion on undefined behavior by\n[@&#8203;amoeba](https://redirect.github.com/amoeba) in\n[#&#8203;602](https://redirect.github.com/apache/arrow-go/pull/602)\n- chore: Bump actions/upload-artifact from 5.0.0 to 6.0.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;611](https://redirect.github.com/apache/arrow-go/pull/611)\n- chore: Bump google.golang.org/protobuf from 1.36.10 to 1.36.11 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;607](https://redirect.github.com/apache/arrow-go/pull/607)\n- chore: Bump github.com/pierrec/lz4/v4 from 4.1.22 to 4.1.23 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;616](https://redirect.github.com/apache/arrow-go/pull/616)\n- chore: Bump golang.org/x/tools from 0.39.0 to 0.40.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;609](https://redirect.github.com/apache/arrow-go/pull/609)\n- chore: Bump actions/cache from 4 to 5 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;608](https://redirect.github.com/apache/arrow-go/pull/608)\n- chore: Bump actions/download-artifact from 6.0.0 to 7.0.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;610](https://redirect.github.com/apache/arrow-go/pull/610)\n- ci(benchmark): switch to new conbench instance by\n[@&#8203;rok](https://redirect.github.com/rok) in\n[#&#8203;593](https://redirect.github.com/apache/arrow-go/pull/593)\n- fix(flight): make StreamChunksFromReader ctx aware and\ncancellation-safe by\n[@&#8203;arnoldwakim](https://redirect.github.com/arnoldwakim) in\n[#&#8203;615](https://redirect.github.com/apache/arrow-go/pull/615)\n- fix(parquet/variant): fix basic stringify by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;624](https://redirect.github.com/apache/arrow-go/pull/624)\n- chore: Bump github.com/google/flatbuffers from 25.9.23+incompatible to\n25.12.19+incompatible by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;617](https://redirect.github.com/apache/arrow-go/pull/617)\n- chore: Bump google.golang.org/grpc from 1.77.0 to 1.78.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;621](https://redirect.github.com/apache/arrow-go/pull/621)\n- chore: Bump golang.org/x/tools from 0.40.0 to 0.41.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;626](https://redirect.github.com/apache/arrow-go/pull/626)\n- fix(parquet/pqarrow): fix partial struct panic by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;630](https://redirect.github.com/apache/arrow-go/pull/630)\n- Flaky test fixes by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;629](https://redirect.github.com/apache/arrow-go/pull/629)\n- ipc: clear variadicCounts in recordEncoder.reset() by\n[@&#8203;asubiotto](https://redirect.github.com/asubiotto) in\n[#&#8203;631](https://redirect.github.com/apache/arrow-go/pull/631)\n- fix(arrow/cdata): Handle errors to prevent panic by\n[@&#8203;xiaocai2333](https://redirect.github.com/xiaocai2333) in\n[#&#8203;614](https://redirect.github.com/apache/arrow-go/pull/614)\n- chore: Bump github.com/substrait-io/substrait-go/v7 from 7.2.0 to\n7.2.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;612](https://redirect.github.com/apache/arrow-go/pull/612)\n- chore: bump version to 18.5.1 by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;632](https://redirect.github.com/apache/arrow-go/pull/632)\n\n#### New Contributors\n\n- [@&#8203;rok](https://redirect.github.com/rok) made their first\ncontribution in\n[#&#8203;593](https://redirect.github.com/apache/arrow-go/pull/593)\n- [@&#8203;asubiotto](https://redirect.github.com/asubiotto) made their\nfirst contribution in\n[#&#8203;631](https://redirect.github.com/apache/arrow-go/pull/631)\n- [@&#8203;xiaocai2333](https://redirect.github.com/xiaocai2333) made\ntheir first contribution in\n[#&#8203;614](https://redirect.github.com/apache/arrow-go/pull/614)\n\n**Full Changelog**:\n<https://github.com/apache/arrow-go/compare/v18.5.0...v18.5.1>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-27T17:02:49Z",
          "tree_id": "81935babe8db34da4b24add20ff29879c02b1ddd",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e1c7a802b626d7c8a6061e9f1a3ced60ac9417eb"
        },
        "date": 1769536938387,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.9706127643585205,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.20178002638512,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.8032898765432,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.5,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.62890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 520923.57347173133,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 531188.9601227788,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001247,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11460915.5513163,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11405008.736256724,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "92fcfc3adeabafb0240b40613f18d6a87f8df833",
          "message": "Formatting and encoding for scope attributes (#1898)\n\n# Change Summary\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1576, part\nof #1903.\n\nHalf of #1895, for a reasonable sized PR.\n\nThis PR:\n\n- Refactors the self_tracing formatter to fix poor structure. A new type\nStyledBufWriter separates the behavior of formatting log messages (w/\ncolor option) from the behavior of ConsoleWriter.\n- Adds ScopeFormatter argument to the basic log format, which formats a\nsuffix. Different callers use this differently, e.g., raw_error! ignores\nit, console_direct/async will append a suffix, and console_exporter\nbypasses b/c scopes print on a separate line\n- Adds ScopeToBytesMap for caching pre-calculated OTLP scope attributes\nas Bytes (with benchmark) and add a use in ITR\n- Extends LogRecord with LogContext, defines LogContextFn to be\nconfigured later in #1895\n- Adds TODOs for console_direct, console_async, and ITS provider mode,\ncurrently using empty context\n\n## How are these changes tested?\n\nNew test for encoding and formatting a scope/entity key.\n\n## Are there any user-facing changes?\n\nNo. See #1895.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-01-28T15:18:59Z",
          "tree_id": "fdf71f5f0a3dcfa969c8a609fae050f165158b25",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/92fcfc3adeabafb0240b40613f18d6a87f8df833"
        },
        "date": 1769616005757,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.8027398586273193,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.03663416083214,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.3632651856426,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.944661458333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.73046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 524450.0789836363,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 533904.5498357705,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002935,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11515511.486552259,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11458653.79963451,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "distinct": false,
          "id": "adfd2e91a374dd818f125c94b5d8881e34185fa1",
          "message": "Self-instrumentation scope attributes  (#1895)\n\n# Change Summary\n\nPart of #1576. \n\nFixes #1903.\n\nPortions of this PR were merged in #1898. Lines that are crossed out\nbelow have been merged already.\n\n- ~Telemetry crate defines LogContext = SmallVec<[EntityKey; 1]>,\nLogContextFn = fn() -> LogContext~\n- InternalTelemetrySystem: accepts context function; registry now passed\nin\n- Controller passes node_context() first or pipeline_context() second\n- Entity attribute set definition structs are re-ordered; first field\nbecomes identifying for console logs\n- Console exporter: now prints entity definition in scope attributes\n- ~Internal telemetry receiver: now encodes scope information on receipt\nof each record (as singletons, currently)~\n- Observed state store: prints scope information in symbolic form (for\nconsole_async, examples below)\n- Entity registry: logs definition of each entity set for correlation in\nconsole logs\n- Console direct logging: prints unsymbolized information (examples\nbelow)\n- ~Self tracing encoder.rs: now encodes scope attributes from cached\ninformation~\n- ~Self tracing formatter.rs: refactored for clarity, now supports\noptional suffix for use in console_direct, console_async modes~\n\n## How are these changes tested?\n\nInternal logging example configurations revised.\n\n## Are there any user-facing changes?\n\nYes. Example logs, e.g., console exporter:\n\n```\n2026-01-27T01:29:54.567Z  RESOURCE   v1.Resource [service.id=1234, service.name=test]\n2026-01-27T01:29:54.567Z  │ SCOPE    v1.InstrumentationScope [node.id=generator, node.urn=urn:otel:otap:fake_data_generator:receiver, node.type=receiver, pipeline.id=default_pipeline, pipeline.group.id=default_pipeline_group, core.id=0, numa.node.id=0, process.instance.id=AGN72ERHGR5OFI24GZVBC7YCNU, host.id=JoshCorpSurfaceLaptop, container.id=]\n2026-01-27T01:29:54.567Z  │ └─ DEBUG otap-df-otap::rate_limit.sleep (crates/otap/src/fake_data_generator.rs:35\n```\n\nE.g., defining a new pipeline entity:\n\n```\n2026-01-27T01:30:27.395Z  INFO  otap-df-telemetry::registry.define_entity (crates/telemetry/src/registry.rs:82):  [schema=pipeline.attrs, entity_name=default_pipeline, definition=pipeline.id=default_pipeline, pipeline.group.id=default_pipeline_group, core.id=0, numa.node.id=0, process.instance.id=AGN72EWQWJZIFEBDCGPW4NHUCU, host.id=JoshCorpSurfaceLaptop, container.id=]\n```\n\ne.g., referring to that pipeline to define a channel with \"named\" entity\nin suffix:\n\n```\n2026-01-27T01:30:27.400Z  INFO  otap-df-telemetry::registry.define_entity (crates/telemetry/src/registry.rs:82):  [schema=channel.attrs, entity_name=batch:control, definition=channel.id=batch:control, node.port=input, channel.kind=control, channel.mode=local, channel.type=mpsc, channel.impl=internal, node.id=batch, node.urn=urn:otel:batch:processor, node.type=processor, pipeline.id=default_pipeline, pipeline.group.id=default_pipeline_group, core.id=0, numa.node.id=0, process.instance.id=AGN72EWQWJZIFEBDCGPW4NHUCU, host.id=JoshCorpSurfaceLaptop, container.id=] entity/pipeline.attrs=default_pipeline\n```\n\nIn the raw logging mode, these print unsymbolized instead of by name,\nsince that is done synchronously and we use a mutex to lookup entity\nnames from keys.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-01-29T00:49:00Z",
          "tree_id": "26dfe87b4e50ad48664a6ef1ddc8e5900aaea24c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/adfd2e91a374dd818f125c94b5d8881e34185fa1"
        },
        "date": 1769652021247,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.053600139915943146,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.29015173198077,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.71051640874553,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.746484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.19140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 525358.9671104326,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 525640.5602677189,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001458,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11536124.489832537,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11481662.929262413,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "e18aa77064e45cdcfe526303105e59a469dc63ee",
          "message": "chore(deps): update dependency psutil to v7.2.2 (#1910)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [psutil](https://redirect.github.com/giampaolo/psutil) | `==7.2.1` →\n`==7.2.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/psutil/7.2.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/psutil/7.2.1/7.2.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>giampaolo/psutil (psutil)</summary>\n\n###\n[`v7.2.2`](https://redirect.github.com/giampaolo/psutil/blob/HEAD/HISTORY.rst#722)\n\n[Compare\nSource](https://redirect.github.com/giampaolo/psutil/compare/release-7.2.1...release-7.2.2)\n\n\\=====\n\n2026-01-28\n\n**Enhancements**\n\n- 2705\\_: \\[Linux]: `Process.wait()`\\_ now uses `pidfd_open()` +\n`poll()` for\n  waiting, resulting in no busy loop and faster response times. Requires\n  Linux >= 5.3 and Python >= 3.9. Falls back to traditional polling if\n  unavailable.\n- 2705\\_: \\[macOS], \\[BSD]: `Process.wait()`\\_ now uses `kqueue()` for\nwaiting,\n  resulting in no busy loop and faster response times.\n\n**Bug fixes**\n\n- 2701\\_, \\[macOS]: fix compilation error on macOS < 10.7. (patch by\nSergey\n  Fedorov)\n- 2707\\_, \\[macOS]: fix potential memory leaks in error paths of\n  `Process.memory_full_info()` and `Process.threads()`.\n- 2708\\_, \\[macOS]: Process.cmdline()`_ and `Process.environ()`_ may\nfail with ``OSError: [Errno 0] Undefined error`` (from\n``sysctl(KERN_PROCARGS2)``).\n  They now raise `AccessDenied\\`\\_ instead.\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-29T01:16:36Z",
          "tree_id": "ffbceeedcd0ce32acc7cb360ecf94c27b27323c9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e18aa77064e45cdcfe526303105e59a469dc63ee"
        },
        "date": 1769653576349,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.39620161056518555,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.12467565124523,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.70589803526448,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.99752604166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.39453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 531959.8251729622,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 529852.1918468302,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002847,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11453671.135603143,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11407688.310668109,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "66b8e1d7c730b96be1b289b0d9bb3fd7c464d3d7",
          "message": "fix: Field to batch index mapping in otap batch unify (#1911)\n\n# Change Summary\n\nPart of the otap batch `unify` logic tracks which otap batches have\nwhich fields. The implementation extracts the schemas for some payload\ntype from each batch and assumes that the index in the schemas list is\nequivalent to the index in the `batches` slice.\n\nHowever, `select` filters out missing payload types from each batch, so\nif some batches are missing a payload then the index is not the same.\n\nThe fix is to maintain the 1:1 mapping of schema index to batch index by\nnot filtering out missing batches.\n\n## What issue does this PR close?\n\nRelated to #1334, but there are still more issues listed there.\n\n## How are these changes tested?\n\nUncommenting the complex metrics tests. The tests now make it farther\nand some scenarios see more success, but there are still at least two\nmore known issues.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-01-29T20:31:23Z",
          "tree_id": "517bf9901f2ea77047ad5654a8604bb53fc85612",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/66b8e1d7c730b96be1b289b0d9bb3fd7c464d3d7"
        },
        "date": 1769720885361,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.11098960041999817,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.29044922476065,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.70322105915145,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.186588541666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.11328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 522797.4871018098,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 523377.73795664066,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001635,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11514654.572061183,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11457101.157201732,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "227ca92e721977e14fde6c92c6f6dd189fc872c4",
          "message": "feat(azure-monitor-exporter): Add heartbeat support and refactor auth handling (#1854)\n\n# Change Summary\n\nAdds heartbeat functionality to the Azure Monitor Exporter and refactors\nauthentication to use a dedicated `AuthHeader` module for reusability\n\n- **Heartbeat support**: Sends periodic health heartbeats to Azure\nMonitor every 60 seconds via the `HEALTH_ASSESSMENT_BLOB` stream\n- Heartbeat metadata sourced from environment variables (`IMAGE`,\n`POD_NAME`, `EXPORTER_ID`, `ARM_RESOURCE_ID`, `HOSTNAME`) with sensible\nfallbacks\n- Move auth out of clients and update auth header of clients\npro-actively using periodic tasks.\n\n## What issue does this PR close?\n\n* Closes heartbeat item on issue #1396\n\n## How are these changes tested?\n\nLocal manual tests and unit tests\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-01-29T20:54:56Z",
          "tree_id": "3e8e2da52c8582588443f50706426c08c609a35f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/227ca92e721977e14fde6c92c6f6dd189fc872c4"
        },
        "date": 1769722573955,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0731674432754517,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.58328401020316,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.93968306930692,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.37317708333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.65625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 515243.6491440859,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 520773.07653379417,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001873,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11386482.048562689,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11333995.055722412,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "distinct": false,
          "id": "bba637d52deba66f13387d0c846e1b0217cc149f",
          "message": "Support NackMsg permanent status; update retry processor (#1917)\n\n# Change Summary\n\nAdds NackMsg::permanent and new constructors.\n\n## What issue does this PR close?\n\nFixes #1900.\n\n## How are these changes tested?\n\nRetry processor.\n\n## Are there any user-facing changes?\n\nI decided not to format! any new \"reason\" strings for NackMsgs at the\nretry_processor. Permanent NackMsgs pass through the retry processor\nwithout modification.",
          "timestamp": "2026-01-29T23:54:49Z",
          "tree_id": "e61c616967508dca22d48fd2103c48a4c97f9adc",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bba637d52deba66f13387d0c846e1b0217cc149f"
        },
        "date": 1769733094201,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.27164632081985474,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.32320037930953,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.94551636760805,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.39596354166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.65234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 524511.9787045253,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 523087.161285919,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.010496,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11393950.032966258,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11343304.559586143,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "d895debb34a90b66be13d3b978682550ea43bad7",
          "message": "[otap-dataflow] Save source node in Pdata msg (#1899)\n\nDefined two new effect handler extension traits one for local, one for\nshared that allows us to update otap pdata with the source node\n\n```rust\n/// Effect handler extension for adding message source\n#[async_trait(?Send)]\npub trait MessageSourceLocalEffectHandlerExtension<PData> {\n    /// Send data after tagging with the source node.\n    async fn send_message_with_source_node(&self, data: PData) -> Result<(), TypedError<PData>>;\n    /// Try to send data after tagging with the source node.\n    fn try_send_message_with_source_node(&self, data: PData) -> Result<(), TypedError<PData>>;\n    /// Send data to a specific port after tagging with the source node.\n    async fn send_message_with_source_node_to<P>(\n        &self,\n        port: P,\n        data: PData,\n    ) -> Result<(), TypedError<PData>>\n    where\n        P: Into<PortName> + Send + 'static;\n    /// Try to send data to a specific port after tagging with the source node.\n    fn try_send_message_with_source_node_to<P>(\n        &self,\n        port: P,\n        data: PData,\n    ) -> Result<(), TypedError<PData>>\n    where\n        P: Into<PortName> + Send + 'static;\n}\n\n/// Send-friendly variant for use in `Send` contexts (e.g., `tokio::spawn`).\n#[async_trait]\npub trait MessageSourceSharedEffectHandlerExtension<PData: Send + 'static> {\n    /// Send data after tagging with the source node.\n    async fn send_message_with_source_node(&self, data: PData) -> Result<(), TypedError<PData>>;\n    /// Try to send data after tagging with the source node.\n    fn try_send_message_with_source_node(&self, data: PData) -> Result<(), TypedError<PData>>;\n    /// Send data to a specific port after tagging with the source node.\n    async fn send_message_with_source_node_to<P>(\n        &self,\n        port: P,\n        data: PData,\n    ) -> Result<(), TypedError<PData>>\n    where\n        P: Into<PortName> + Send + 'static;\n    /// Try to send data to a specific port after tagging with the source node.\n    fn try_send_message_with_source_node_to<P>(\n        &self,\n        port: P,\n        data: PData,\n    ) -> Result<(), TypedError<PData>>\n    where\n        P: Into<PortName> + Send + 'static;\n}\n```\n\nAdded a field to the Context struct that will store the node information\nand added new functions for OtapPdata and Context getting and setting\nthe source node\n\n```rust\npub struct Context {\n    source_node: Option<NodeId>,\n    stack: Vec<Frame>,\n}\n\n...\n\n\n  /// update the source node\n  pub fn add_source_node(mut self, node_id: Option<NodeId>) -> Self {\n      self.source_node = node_id;\n      self\n  }\n\n  /// return the source node field\n  pub fn get_source_node(&self) -> Option<NodeId> {\n      self.source_node.clone()\n  }\n```\n\nUpdated pipeline nodes to use send_message functions that will tag\notappdata with source node name\n\nCloses #1880",
          "timestamp": "2026-01-30T00:14:46Z",
          "tree_id": "3ea46d7b476afc9fc8384dff94c55b0c4d0bd170",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d895debb34a90b66be13d3b978682550ea43bad7"
        },
        "date": 1769734425057,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.9528863430023193,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.9944914780962,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.37290284832588,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.71627604166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.0546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 516896.5445750887,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 526990.9469428575,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003156,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11468013.534200806,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11408065.408024665,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2d1f9b0bd4eefcc144e4a89c69729921df7c0be3",
          "message": "fix: Batches may differ by field order after unification (#1922)\n\n# Change Summary\n\nNote this is a band-aid to avoid larger changes, but it does solve a\nbunch of panics.\n\n- Project batches to the merged schema before coalescing (reorder the\nfields to be the same)\n\n## What issue does this PR close?\n\nRelated to: https://github.com/open-telemetry/otel-arrow/issues/1334.\n\n## How are these changes tested?\n\nNew unit tests for the coalescing.\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-01-30T00:26:59Z",
          "tree_id": "37f6dfdc465e3c1d3b9932bf39d5e186c0505304",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2d1f9b0bd4eefcc144e4a89c69729921df7c0be3"
        },
        "date": 1769738884715,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.4723619818687439,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.06338047052014,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.4356542766468,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.579296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.1875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527446.0718235332,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 524954.617159404,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006711,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11371878.599867726,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11322005.546260633,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d901f72f37936e97c6bfa82c2cd2c3f2cd563ac4",
          "message": "refactor: Use `.fields.find()` instead of `.index_of()` to look up field indices when batching (#1924)\n\n# Change Summary\n\nSwap out the `index_of` API which creates and expensive string on the\nfailure/missing case for `.fields.find()` API which just returns an\noption.\n\n\n## What issue does this PR close?\n\nAlbert pointed this out to me here:\nhttps://github.com/open-telemetry/otel-arrow/pull/1922#discussion_r2744264230\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-01-30T03:02:21Z",
          "tree_id": "8c48b5eb137d32f1c3055451e3acc629e2f332a1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d901f72f37936e97c6bfa82c2cd2c3f2cd563ac4"
        },
        "date": 1769744430665,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0269492864608765,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.38953014137725,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.71489838767134,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.055729166666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.2890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 521820.9463438554,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 527179.7825929387,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001087,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11473666.421209909,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11418868.705610031,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "6ad291b19e1b329ce9441810ea2b4a41cd1085eb",
          "message": "Allow mixed local/shared pdata senders (#1919)\n\n# Change Summary\n\n- Allow local receivers/processors to use the generic message::Sender so\nmixed local/shared edges can share channels safely.\n- Introduce ChannelMode to centralize control-channel wiring and\nmetrics, reducing duplication across wrappers making the overall design\nless error-prone.\n- Add pipeline test for mixed local/shared receivers targeting the same\nexporter.\n  \n  ## What issue does this PR close?\n\n  NA\n  \n  ## How are these changes tested?\n\n See pipeline_tests.rs\n\n  ## Are there any user-facing changes?\n\n  No",
          "timestamp": "2026-01-30T03:15:37Z",
          "tree_id": "89a8b63aa93fa4ecc95c92f5ae06f108e20cff0b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ad291b19e1b329ce9441810ea2b4a41cd1085eb"
        },
        "date": 1769745944528,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8196586966514587,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.35315326153882,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.76953643682619,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.28658854166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.25390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 518436.36049408105,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 522685.76891658467,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002705,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11484129.427520676,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11427586.339749234,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "c6b0c2bdd3dfca85de3aa72635682fdc38d3de3e",
          "message": "chore(deps): update docker digest updates (#1929)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| alpine | final | digest | `865b95f` → `2510918` |\n| docker.io/alpine | final | digest | `865b95f` → `2510918` |\n| golang | stage | digest | `6cc2338` → `ce63a16` |\n| python | final | digest | `3955a7d` → `9b81fe9` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on the first day of the\nmonth\" (UTC), Automerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-02-02T13:50:39Z",
          "tree_id": "7f564f73633b87749c8551ea7a0e8baa5b0c895a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c6b0c2bdd3dfca85de3aa72635682fdc38d3de3e"
        },
        "date": 1770042453388,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.404669165611267,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.2400805829892,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.58189376906992,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.45703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.71875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 521207.2234751694,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 528528.4604638354,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002975,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11406519.306741068,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11356264.371875256,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "af5b129e030ce83f745f5b1a56725ea29ffb915c",
          "message": "chore(deps): update github workflow dependencies (#1930)\n\n> ℹ️ **Note**\n> \n> This PR body was truncated due to platform limits.\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[EmbarkStudios/cargo-deny-action](https://redirect.github.com/EmbarkStudios/cargo-deny-action)\n| action | patch | `v2.0.14` → `v2.0.15` |\n| [actions/setup-go](https://redirect.github.com/actions/setup-go) |\naction | minor | `v6.1.0` → `v6.2.0` |\n| [actions/setup-node](https://redirect.github.com/actions/setup-node) |\naction | minor | `v6.1.0` → `v6.2.0` |\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | minor | `v4.31.9` → `v4.32.0` |\n| [go](https://redirect.github.com/actions/go-versions) | uses-with |\npatch | `1.25.5` → `1.25.6` |\n| [korandoru/hawkeye](https://redirect.github.com/korandoru/hawkeye) |\naction | minor | `v6.3.0` → `v6.4.1` |\n| [python](https://redirect.github.com/actions/python-versions) |\nuses-with | minor | `3.11` → `3.14` |\n|\n[step-security/harden-runner](https://redirect.github.com/step-security/harden-runner)\n| action | patch | `v2.14.0` → `v2.14.1` |\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | minor | `v2.65.13` → `v2.67.18` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>EmbarkStudios/cargo-deny-action\n(EmbarkStudios/cargo-deny-action)</summary>\n\n###\n[`v2.0.15`](https://redirect.github.com/EmbarkStudios/cargo-deny-action/releases/tag/v2.0.15):\nRelease 2.0.15 - cargo-deny 0.19.0\n\n[Compare\nSource](https://redirect.github.com/EmbarkStudios/cargo-deny-action/compare/v2.0.14...v2.0.15)\n\n##### Changed\n\n-\n[PR#802](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/802)\nmade relative paths passed to `--config` be resolved relative to the\ncurrent working directory (rather than the resolved manifest path's\ndirectory).\n-\n[PR#825](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/825)\nupdated `gix`, `reqwest`, and `tame-index` to newer versions. The\n`reqwest` 0.13 changes means it is no longer possible to choose the\nsource of root certificates for `gix`, so that decision is now left to\n`rustls-platform-verifier`. The `native-certs` feature has thus been\nremoved, and `cargo-deny` no longer defaults to using `webpki-roots`.\n\n##### Fixed\n\n-\n[PR#802](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/802)\nfixed path handling of paths passed to `--config`, resolving\n[#&#8203;748](https://redirect.github.com/EmbarkStudios/krates/issues/748).\n-\n[PR#819](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/819)\nadded locations to all SARIF results since that's mandatory for valid\nSARIF.\n-\n[PR#821](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/821)\nfixed compilation on an Alpine host.\n\n##### Added\n\n-\n[PR#795](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/795)\nadded `[bans.allow-workspace]` to allow workspace crates while denying\nall external crates.\n-\n[PR#800](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/800)\nadded `[licenses.include-build]` to toggle whether build dependencies\nare included in the license check.\n-\n[PR#823](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/823)\nadded `[advisories.unused-ignored-advisory]` to disable the warning when\nan advisory is ignored but not encountered in the crate graph.\n-\n[PR#826](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/826)\nadded `[advisories.unsound]` to determine which crates can show\n`unsound` advisories, similarly to the `unmaintained` field. Defaults to\n`workspace` crates, ignoring `unsound` advisories for transitive\ndependencies, resolving\n[#&#8203;824](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/824).\n\n</details>\n\n<details>\n<summary>actions/setup-go (actions/setup-go)</summary>\n\n###\n[`v6.2.0`](https://redirect.github.com/actions/setup-go/releases/tag/v6.2.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-go/compare/v6.1.0...v6.2.0)\n\n##### What's Changed\n\n##### Enhancements\n\n- Example for restore-only cache in documentation by\n[@&#8203;aparnajyothi-y](https://redirect.github.com/aparnajyothi-y) in\n[#&#8203;696](https://redirect.github.com/actions/setup-go/pull/696)\n- Update Node.js version in action.yml by\n[@&#8203;ccoVeille](https://redirect.github.com/ccoVeille) in\n[#&#8203;691](https://redirect.github.com/actions/setup-go/pull/691)\n- Documentation update of actions/checkout by\n[@&#8203;deining](https://redirect.github.com/deining) in\n[#&#8203;683](https://redirect.github.com/actions/setup-go/pull/683)\n\n##### Dependency updates\n\n- Upgrade js-yaml from 3.14.1 to 3.14.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot) in\n[#&#8203;682](https://redirect.github.com/actions/setup-go/pull/682)\n- Upgrade\n[@&#8203;actions/cache](https://redirect.github.com/actions/cache) to v5\nby [@&#8203;salmanmkc](https://redirect.github.com/salmanmkc) in\n[#&#8203;695](https://redirect.github.com/actions/setup-go/pull/695)\n- Upgrade actions/checkout from 5 to 6 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot) in\n[#&#8203;686](https://redirect.github.com/actions/setup-go/pull/686)\n- Upgrade qs from 6.14.0 to 6.14.1 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot) in\n[#&#8203;703](https://redirect.github.com/actions/setup-go/pull/703)\n\n##### New Contributors\n\n- [@&#8203;ccoVeille](https://redirect.github.com/ccoVeille) made their\nfirst contribution in\n[#&#8203;691](https://redirect.github.com/actions/setup-go/pull/691)\n- [@&#8203;deining](https://redirect.github.com/deining) made their\nfirst contribution in\n[#&#8203;683](https://redirect.github.com/actions/setup-go/pull/683)\n\n**Full Changelog**:\n<https://github.com/actions/setup-go/compare/v6...v6.2.0>\n\n</details>\n\n<details>\n<summary>actions/setup-node (actions/setup-node)</summary>\n\n###\n[`v6.2.0`](https://redirect.github.com/actions/setup-node/compare/v6.1.0...v6.2.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-node/compare/v6.1.0...v6.2.0)\n\n</details>\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v4.32.0`](https://redirect.github.com/github/codeql-action/releases/tag/v4.32.0)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.31.11...v4.32.0)\n\n- Update default CodeQL bundle version to\n[2.24.0](https://redirect.github.com/github/codeql-action/releases/tag/codeql-bundle-v2.24.0).\n[#&#8203;3425](https://redirect.github.com/github/codeql-action/pull/3425)\n\n###\n[`v4.31.11`](https://redirect.github.com/github/codeql-action/releases/tag/v4.31.11)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.31.10...v4.31.11)\n\n- When running a Default Setup workflow with [Actions debugging\nenabled](https://docs.github.com/en/actions/how-tos/monitor-workflows/enable-debug-logging),\nthe CodeQL Action will now use more unique names when uploading logs\nfrom the Dependabot authentication proxy as workflow artifacts. This\nensures that the artifact names do not clash between multiple jobs in a\nbuild matrix.\n[#&#8203;3409](https://redirect.github.com/github/codeql-action/pull/3409)\n- Improved error handling throughout the CodeQL Action.\n[#&#8203;3415](https://redirect.github.com/github/codeql-action/pull/3415)\n- Added experimental support for automatically excluding [generated\nfiles](https://docs.github.com/en/repositories/working-with-files/managing-files/customizing-how-changed-files-appear-on-github)\nfrom the analysis. This feature is not currently enabled for any\nanalysis. In the future, it may be enabled by default for some\nGitHub-managed analyses.\n[#&#8203;3318](https://redirect.github.com/github/codeql-action/pull/3318)\n- The changelog extracts that are included with releases of the CodeQL\nAction are now shorter to avoid duplicated information from appearing in\nDependabot PRs.\n[#&#8203;3403](https://redirect.github.com/github/codeql-action/pull/3403)\n\n###\n[`v4.31.10`](https://redirect.github.com/github/codeql-action/releases/tag/v4.31.10)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.31.9...v4.31.10)\n\n##### CodeQL Action Changelog\n\nSee the [releases\npage](https://redirect.github.com/github/codeql-action/releases) for the\nrelevant changes to the CodeQL CLI and language packs.\n\n##### 4.31.10 - 12 Jan 2026\n\n- Update default CodeQL bundle version to 2.23.9.\n[#&#8203;3393](https://redirect.github.com/github/codeql-action/pull/3393)\n\nSee the full\n[CHANGELOG.md](https://redirect.github.com/github/codeql-action/blob/v4.31.10/CHANGELOG.md)\nfor more information.\n\n</details>\n\n<details>\n<summary>actions/go-versions (go)</summary>\n\n###\n[`v1.25.6`](https://redirect.github.com/actions/go-versions/releases/tag/1.25.6-21053840953):\n1.25.6\n\n[Compare\nSource](https://redirect.github.com/actions/go-versions/compare/1.25.5-19880500865...1.25.6-21053840953)\n\nGo 1.25.6\n\n</details>\n\n<details>\n<summary>korandoru/hawkeye (korandoru/hawkeye)</summary>\n\n###\n[`v6.4.1`](https://redirect.github.com/korandoru/hawkeye/releases/tag/v6.4.1):\n6.4.1 2026-01-13\n\n[Compare\nSource](https://redirect.github.com/korandoru/hawkeye/compare/v6.4.0...v6.4.1)\n\n#### Release Notes\n\n#### Install hawkeye 6.4.1\n\n##### Install prebuilt binaries via shell script\n\n```sh\ncurl --proto '=https' --tlsv1.2 -LsSf https://github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-installer.sh | sh\n```\n\n#### Download hawkeye 6.4.1\n\n| File | Platform | Checksum |\n|\n----------------------------------------------------------------------------------------------------------------------------------------------------\n| ------------------- |\n--------------------------------------------------------------------------------------------------------------------------\n|\n|\n[hawkeye-aarch64-apple-darwin.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-aarch64-apple-darwin.tar.xz)\n| Apple Silicon macOS |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-aarch64-apple-darwin.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-apple-darwin.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-x86_64-apple-darwin.tar.xz)\n| Intel macOS |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-x86_64-apple-darwin.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-pc-windows-msvc.zip](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-x86_64-pc-windows-msvc.zip)\n| x64 Windows |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-x86_64-pc-windows-msvc.zip.sha256)\n|\n|\n[hawkeye-aarch64-unknown-linux-gnu.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-aarch64-unknown-linux-gnu.tar.xz)\n| ARM64 Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-aarch64-unknown-linux-gnu.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-unknown-linux-gnu.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-x86_64-unknown-linux-gnu.tar.xz)\n| x64 Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-x86_64-unknown-linux-gnu.tar.xz.sha256)\n|\n|\n[hawkeye-aarch64-unknown-linux-musl.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-aarch64-unknown-linux-musl.tar.xz)\n| ARM64 MUSL Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-aarch64-unknown-linux-musl.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-unknown-linux-musl.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-x86_64-unknown-linux-musl.tar.xz)\n| x64 MUSL Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-x86_64-unknown-linux-musl.tar.xz.sha256)\n|\n\n###\n[`v6.4.0`](https://redirect.github.com/korandoru/hawkeye/releases/tag/v6.4.0)\n\n[Compare\nSource](https://redirect.github.com/korandoru/hawkeye/compare/v6.3.0...v6.4.0)\n\n#### Install hawkeye 6.4.0\n\n##### Install prebuilt binaries via shell script\n\n```sh\ncurl --proto '=https' --tlsv1.2 -LsSf https://github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-installer.sh | sh\n```\n\n#### Download hawkeye 6.4.0\n\n| File | Platform | Checksum |\n|\n----------------------------------------------------------------------------------------------------------------------------------------------------\n| ------------------- |\n--------------------------------------------------------------------------------------------------------------------------\n|\n|\n[hawkeye-aarch64-apple-darwin.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-aarch64-apple-darwin.tar.xz)\n| Apple Silicon macOS |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-aarch64-apple-darwin.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-apple-darwin.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-x86_64-apple-darwin.tar.xz)\n| Intel macOS |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-x86_64-apple-darwin.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-pc-windows-msvc.zip](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-x86_64-pc-windows-msvc.zip)\n| x64 Windows |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-x86_64-pc-windows-msvc.zip.sha256)\n|\n|\n[hawkeye-aarch64-unknown-linux-gnu.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-aarch64-unknown-linux-gnu.tar.xz)\n| ARM64 Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-aarch64-unknown-linux-gnu.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-unknown-linux-gnu.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-x86_64-unknown-linux-gnu.tar.xz)\n| x64 Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-x86_64-unknown-linux-gnu.tar.xz.sha256)\n|\n|\n[hawkeye-aarch64-unknown-linux-musl.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-aarch64-unknown-linux-musl.tar.xz)\n| ARM64 MUSL Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-aarch64-unknown-linux-musl.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-unknown-linux-musl.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-x86_64-unknown-linux-musl.tar.xz)\n| x64 MUSL Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-x86_64-unknown-linux-musl.tar.xz.sha256)\n|\n\n</details>\n\n<details>\n<summary>actions/python-versions (python)</summary>\n\n###\n[`v3.14.2`](https://redirect.github.com/actions/python-versions/releases/tag/3.14.2-20014991423):\n3.14.2\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.14.1-19879739908...3.14.2-20014991423)\n\nPython 3.14.2\n\n###\n[`v3.14.1`](https://redirect.github.com/actions/python-versions/releases/tag/3.14.1-19879739908):\n3.14.1\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.14.0-18313368925...3.14.1-19879739908)\n\nPython 3.14.1\n\n###\n[`v3.14.0`](https://redirect.github.com/actions/python-versions/releases/tag/3.14.0-18313368925):\n3.14.0\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.11-20014977833...3.14.0-18313368925)\n\nPython 3.14.0\n\n###\n[`v3.13.11`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.11-20014977833):\n3.13.11\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.10-19879712315...3.13.11-20014977833)\n\nPython 3.13.11\n\n###\n[`v3.13.10`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.10-19879712315):\n3.13.10\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.9-18515951191...3.13.10-19879712315)\n\nPython 3.13.10\n\n###\n[`v3.13.9`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.9-18515951191):\n3.13.9\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.8-18331000654...3.13.9-18515951191)\n\nPython 3.13.9\n\n###\n[`v3.13.8`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.8-18331000654):\n3.13.8\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.7-16980743123...3.13.8-18331000654)\n\nPython 3.13.8\n\n###\n[`v3.13.7`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.7-16980743123):\n3.13.7\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.6-16792117939...3.13.7-16980743123)\n\nPython 3.13.7\n\n###\n[`v3.13.6`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.6-16792117939):\n3.13.6\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.5-15601068749...3.13.6-16792117939)\n\nPython 3.13.6\n\n###\n[`v3.13.5`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.5-15601068749):\n3.13.5\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.4-15433317575...3.13.5-15601068749)\n\nPython 3.13.5\n\n###\n[`v3.13.4`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.4-15433317575):\n3.13.4\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.3-14344076652...3.13.4-15433317575)\n\nPython 3.13.4\n\n###\n[`v3.13.3`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.3-14344076652):\n3.13.3\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.2-13708744326...3.13.3-14344076652)\n\nPython 3.13.3\n\n###\n[`v3.13.2`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.2-13708744326):\n3.13.2\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.1-13437882550...3.13.2-13708744326)\n\nPython 3.13.2\n\n###\n[`v3.13.1`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.1-13437882550):\n3.13.1\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.0-13707372259...3.13.1-13437882550)\n\nPython 3.13.1\n\n###\n[`v3.13.0`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.0-13707372259):\n3.13.0\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.12-18393146713...3.13.0-13707372259)\n\nPython 3.13.0\n\n###\n[`v3.12.12`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.12-18393146713):\n3.12.12\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.11-15433310049...3.12.12-18393146713)\n\nPython 3.12.12\n\n###\n[`v3.12.11`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.11-15433310049):\n3.12.11\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.10-14343898437...3.12.11-15433310049)\n\nPython 3.12.11\n\n###\n[`v3.12.10`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.10-14343898437):\n3.12.10\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.9-13149478207...3.12.10-14343898437)\n\nPython 3.12.10\n\n###\n[`v3.12.9`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.9-13149478207):\n3.12.9\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.8-12154062663...3.12.9-13149478207)\n\nPython 3.12.9\n\n###\n[`v3.12.8`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.8-12154062663):\n3.12.8\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.7-11128208086...3.12.8-12154062663)\n\nPython 3.12.8\n\n###\n[`v3.12.7`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.7-11128208086):\n3.12.7\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.6-10765725458...3.12.7-11128208086)\n\nPython 3.12.7\n\n###\n[`v3.12.6`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.6-10765725458):\n3.12.6\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.5-10375840348...3.12.6-10765725458)\n\nPython 3.12.6\n\n###\n[`v3.12.5`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.5-10375840348):\n3.12.5\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.4-9947065640...3.12.5-10375840348)\n\nPython 3.12.5\n\n###\n[`v3.12.4`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.4-9947065640):\n3.12.4\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.3-11057844995...3.12.4-9947065640)\n\nPython 3.12.4\n\n###\n[`v3.12.3`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.3-11057844995):\n3.12.3\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.2-11057786931...3.12.3-11057844995)\n\nPython 3.12.3\n\n###\n[`v3.12.2`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.2-11057786931):\n3.12.2\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.1-11057762749...3.12.2-11057786931)\n\nPython 3.12.2\n\n###\n[`v3.12.1`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.1-11057762749):\n3.12.1\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.0-11057302691...3.12.1-11057762749)\n\nPython 3.12.1\n\n###\n[`v3.12.0`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.0-11057302691):\n3.12.0\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.11.14-18393181605...3.12.0-11057302691)\n\nPython 3.12.0\n\n</details>\n\n<details>\n<summary>step-security/harden-runner\n(step-security/harden-runner)</summary>\n\n###\n[`v2.14.1`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.14.1)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.14.0...v2.14.1)\n\n#### What's Changed\n\n1. In some self-hosted environments, the agent could briefly fall back\nto public DNS resolvers during startup if the system DNS was not yet\navailable. This behavior was unintended for GitHub-hosted runners and\nhas now been fixed to prevent any use of public DNS resolvers.\n\n2. Fixed npm audit vulnerabilities\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.14.0...v2.14.1>\n\n</details>\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.67.18`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.67.17...v2.67.18)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.18...HEAD\n\n[2.67.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.17...v2.67.18\n\n[2.67.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.16...v2.67.17\n\n[2.67.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.15...v2.67.16\n\n[2.67.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.14...v2.67.15\n\n[2.67.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.13...v2.67.14\n\n[2.67.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.12...v2.67.13\n\n[2.67.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.11...v2.67.12\n\n[2.67.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.10...v2.67.11\n\n[2.67.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.9...v2.67.10\n\n[2.67.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.8...v2.67.9\n\n[2.67.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.7...v2.67.8\n\n[2.67.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.6...v2.67.7\n\n[2.67.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.5...v2.67.6\n\n[2.67.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.4...v2.67.5\n\n[2.67.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.3...v2.67.4\n\n[2.67.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.2...v2.67.3\n\n[2.67.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.1...v2.67.2\n\n[2.67.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.0...v2.67.1\n\n[2.67.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.7...v2.67.0\n\n[2.66.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.6...v2.66.7\n\n[2.66.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.5...v2.66.6\n\n[2.66.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.4...v2.66.5\n\n[2.66.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.3...v2.66.4\n\n[2.66.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.2...v2.66.3\n\n[2.66.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.1...v2.66.2\n\n[2.66.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.0...v2.66.1\n\n[2.66.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.16...v2.66.0\n\n[2.65.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.15...v2.65.16\n\n[2.65.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.14...v2.65.15\n\n[2.65.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.13...v2.65.14\n\n[2.65.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.12...v2.65.13\n\n[2.65.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.11...v2.65.12\n\n[2.65.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.10...v2.65.11\n\n[2.65.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.9...v2.65.10\n\n[2.65.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.8...v2.65.9\n\n[2.65.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.7...v2.65.8\n\n[2.65.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.6...v2.65.7\n\n[2.65.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.5...v2.65.6\n\n[2.65.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.4...v2.65.5\n\n[2.65.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.3...v2.65.4\n\n[2.65.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.2...v2.65.3\n\n[2.65.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.1...v2.65.2\n\n[2.65.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.0...v2.65.1\n\n[2.65.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.2...v2.65.0\n\n[2.64.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.1...v2.64.2\n\n[2.64.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.0...v2.64.1\n\n[2.64.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.3...v2.64.0\n\n[2.63.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.2...v2.63.3\n\n[2.63.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.1...v2.63.2\n\n[2.63.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.0...v2.63.1\n\n[2.63.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.67...v2.63.0\n\n[2.62.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.66...v2.62.67\n\n[2.62.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.65...v2.62.66\n\n[2.62.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.64...v2.62.65\n\n[2.62.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.63...v2.62.64\n\n[2.62.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.62...v2.62.63\n\n[2.62.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.61...v2.62.62\n\n[2.62.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.60...v2.62.61\n\n[2.62.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.59...v2.62.60\n\n[2.62.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.58...v2.62.59\n\n[2.62.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.57...v2.62.58\n\n[2.62.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.56...v2.62.57\n\n[2.62.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.55...v2.62.56\n\n[2.62.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.54...v2.62.55\n\n[2.62.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.53...v2.62.54\n\n[2.62.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.52...v2.62.53\n\n[2.62.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.51...v2.62.52\n\n[2.62.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.50...v2.62.51\n\n[2.62.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.49...v2.62.50\n\n[2.62.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.48...v2.62.49\n\n[2.62.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.47...v2.62.48\n\n[2.62.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.46...v2.62.47\n\n[2.62.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.45...v2.62.46\n\n[2.62.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.44...v2.62.45\n\n[2.62.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.43...v2.62.44\n\n[2.62.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.42...v2.62.43\n\n[2.62.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.41...v2.62.42\n\n[2.62.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.40...v2.62.41\n\n[2.62.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.39...v2.62.40\n\n[2.62.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.38...v2.62.39\n\n[2.62.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.37...v2.62.38\n\n[2.62.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.36...v2.62.37\n\n[2.62.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.35...v2.62.36\n\n[2.62.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.34...v2.62.35\n\n[2.62.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.33...v2.62.34\n\n[2.62.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.32...v2.62.33\n\n[2.62.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.31...v2.62.32\n\n[2.62.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.30...v2.62.31\n\n[2.62.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.29...v2.62.30\n\n[2.62.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.28...v2.62.29\n\n[2.62.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.27...v2.62.28\n\n[2.62.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.26...v2.62.27\n\n[2.62.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.25...v2.62.26\n\n[2.62.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.24...v2.62.25\n\n[2.62.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.23...v2.62.24\n\n[2.62.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.22...v2.62.23\n\n[2.62.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.21...v2.62.22\n\n[2.62.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.20...v2.62.21\n\n[2.62.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.19...v2.62.20\n\n[2.62.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.18...v2.62.19\n\n[2.62.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.17...v2.62.18\n\n[2.62.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.16...v2.62.17\n\n[2.62.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.15...v2.62.16\n\n[2.62.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.14...v2.62.15\n\n[2.62.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.13...v2.62.14\n\n[2.62.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.12...v2.62.13\n\n[2.62.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.11...v2.62.12\n\n[2.62.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.10...v2.62.11\n\n[2.62.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.9...v2.62.10\n\n[2.62.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.8...v2.62.9\n\n[2.62.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.7...v2.62.8\n\n[2.62.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.6...v2.62.7\n\n[2.62.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.5...v2.62.6\n\n[2.62.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.4...v2.62.5\n\n[2.62.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.3...v2.62.4\n\n[2.62.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.2...v2.62.3\n\n[2.62.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.1...v2.62.2\n\n[2.62.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.0...v2.62.1\n\n[2.62.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.13...v2.62.0\n\n[2.61.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.12...v2.61.13\n\n[2.61.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.11...v2.61.12\n\n[2.61.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.10...v2.61.11\n\n[2.61.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.9...v2.61.10\n\n[2.61.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.8...v2.61.9\n\n[2.61.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.7...v2.61.8\n\n[2.61.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.6...v2.61.7\n\n[2.61.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.5...v2.61.6\n\n[2.61.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.4...v2.61.5\n\n[2.61.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.3...v2.61.4\n\n[2.61.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.2...v2.61.3\n\n[2.61.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.1...v2.61.2\n\n[2.61.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.0...v2.61.1\n\n[2.61.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.60.0...v2.61.0\n\n[2.60.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.1...v2.60.0\n\n[2.59.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.0...v2.59.1\n\n[2.59.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.33...v2.59.0\n\n[2.58.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.32...v2.58.33\n\n[2.58.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.31...v2.58.32\n\n[2.58.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.30...v2.58.31\n\n[2.58.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.29...v2.58.30\n\n[2.58.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.28...v2.58.29\n\n[2.58.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.27...v2.58.28\n\n[2.58.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.26...v2.58.27\n\n[2.58.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.25...v2.58.26\n\n[2.58.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.24...v2.58.25\n\n[2.58.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.23...v2.58.24\n\n[2.58.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.22...v2.58.23\n\n[2.58.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.21...v2.58.22\n\n[2.58.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.20...v2.58.21\n\n[2.58.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.19...v2.58.20\n\n[2.58.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.18...v2.58.19\n\n[2.58.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.17...v2.58.18\n\n[2.58.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.16...v2.58.17\n\n[2.58.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.15...v2.58.16\n\n[2.58.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.14...v2.58.15\n\n[2.58.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.13...v2.58.14\n\n[2.58.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.12...v2.58.13\n\n[2.58.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.11...v2.58.12\n\n[2.58.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.10...v2.58.11\n\n[2.58.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.9...v2.58.10\n\n[2.58.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.8...v2.58.9\n\n[2.58.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.7...v2.58.8\n\n[2.58.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.6...v2.58.7\n\n[2.58.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.5...v2.58.6\n\n[2.58.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.4...v2.58.5\n\n[2.58.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.3...v2.58.4\n\n[2.58.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.2...v2.58.3\n\n[2.58.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.1...v2.58.2\n\n[2.58.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.0...v2.58.1\n\n[2.58.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.8...v2.58.0\n\n[2.57.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.7...v2.57.8\n\n[2.57.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.6...v2.57.7\n\n[2.57.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.5...v2.57.6\n\n[2.57.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.4...v2.57.5\n\n[2.57.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.3...v2.57.4\n\n[2.57.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.2...v2.57.3\n\n[2.57.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.1...v2.57.2\n\n[2.57.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.0...v2.57.1\n\n[2.57.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.24...v2.57.0\n\n[2.56.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.23...v2.56.24\n\n[2.56.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.22...v2.56.23\n\n[2.56.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.21...v2.56.22\n\n[2.56.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.20...v2.56.21\n\n[2.56.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.19...v2.56.20\n\n[2.56.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.18...v2.56.19\n\n[2.56.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.17...v2.56.18\n\n[2.56.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.16...v2.56.17\n\n[2.56.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.15...v2.56.16\n\n[2.56.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.14...v2.56.15\n\n[2.56.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.13...v2.56.14\n\n[2.56.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.12...v2.56.13\n\n[2.56.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.11...v2.56.12\n\n[2.56.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.10...v2.56.11\n\n[2.56.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.9...v2.56.10\n\n[2.56.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.8...v2.56.9\n\n[2.56.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.7...v2.56.8\n\n[2.56.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.6...v2.56.7\n\n[2.56.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.5...v2.56.6\n\n[2.56.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.4...v2.56.5\n\n[2.56.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.3...v2.56.4\n\n[2.56.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.2...v2.56.3\n\n[2.56.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.1...v2.56.2\n\n[2.56.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.0...v2.56.1\n\n[2.56.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.4...v2.56.0\n\n[2.55.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.3...v2.55.4\n\n[2.55.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.2...v2.55.3\n\n[2.55.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.1...v2.55.2\n\n[2.55.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.0...v2.55.1\n\n[2.55.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.3...v2.55.0\n\n[2.54.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.2...v2.54.3\n\n[2.54.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.1...v2.54.2\n\n[2.54.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.0...v2.54.1\n\n[2.54.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.2...v2.54.0\n\n[2.53.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.1...v2.53.2\n\n[2.53.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.0...v2.53.1\n\n[2.53.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.8...v2.53.0\n\n[2.52.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.7...v2.52.8\n\n[2.52.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.6...v2.52.7\n\n[2.52.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.5...v2.52.6\n\n[2.52.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.4...v2.52.5\n\n[2.52.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.3...v2.52.4\n\n[2.52.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.2...v2.52.3\n\n[2.52.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.1...v2.52.2\n\n[2.52.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.0...v2.52.1\n\n[2.52.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.3...v2.52.0\n\n[2.51.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.2...v2.51.3\n\n[2.51.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.1...v2.51.2\n\n[2.51.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.0...v2.51.1\n\n[2.51.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.10...v2.51.0\n\n[2.50.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.9...v2.50.10\n\n[2.50.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.8...v2.50.9\n\n[2.50.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.7...v2.50.8\n\n[2.50.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.6...v2.50.7\n\n[2.50.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.5...v2.50.6\n\n[2.50.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.4...v2.50.5\n\n[2.50.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.3...v2.50.4\n\n[2.50.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.2...v2.50.3\n\n[2.50.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.1...v2.50.2\n\n[2.50.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.0...v2.50.1\n\n[2.50.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.50...v2.50.0\n\n[2.49.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.49...v2.49.50\n\n[2.49.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.48...v2.49.49\n\n[2.49.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.47...v2.49.48\n\n[2.49.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.46...v2.49.47\n\n[2.49.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.45...v2.49.46\n\n[2.49.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.44...v2.49.45\n\n[2.49.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.43...v2.49.44\n\n[2.49.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.42...v2.49.43\n\n[2.49.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.41...v2.49.42\n\n[2.49.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.40...v2.49.41\n\n[2.49.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.39...v2.49.40\n\n[2.49.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.38...v2.49.39\n\n[2.49.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.37...v2.49.38\n\n[2.49.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.36...v2.49.37\n\n[2.49.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.35...v2.49.36\n\n[2.49.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.34...v2.49.35\n\n[2.49.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.33...v2.49.34\n\n[2.49.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.32...v2.49.33\n\n[2.49.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.31...v2.49.32\n\n[2.49.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.30...v2.49.31\n\n[2.49.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.29...v2.49.30\n\n[2.49.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.28...v2.49.29\n\n[2.49.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.27...v2.49.28\n\n[2.49.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.26...v2.49.27\n\n[2.49.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.25...v2.49.26\n\n[2.49.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.24...v2.49.25\n\n[2.49.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.23...v2.49.24\n\n[2.49.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.22...v2.49.23\n\n[2.49.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.21...v2.49.22\n\n[2.49.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.20...v2.49.21\n\n[2.49.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.19...v2.49.20\n\n[2.49.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.18...v2.49.19\n\n[2.49.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.17...v2.49.18\n\n[2.49.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.16...v2.49.17\n\n[2.49.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.15...v2.49.16\n\n[2.49.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.14...v2.49.15\n\n[2.49.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.13...v2.49.14\n\n[2.49.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.12...v2.49.13\n\n[2.49.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.11...v2.49.12\n\n[2.49.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.10...v2.49.11\n\n[2.49.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.9...v2.49.10\n\n[2.49.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.8...v2.49.9\n\n[2.49.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.7...v2.49.8\n\n[2.49.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.6...v2.49.7\n\n[2.49.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.5...v2.49.6\n\n[2.49.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.4...v2.49.5\n\n[2.49.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.3...v2.49.4\n\n[2.49.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.2...v2.49.3\n\n[2.49.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.1...v2.49.2\n\n[2.49.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.0...v2.49.1\n\n[2.49.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.22...v2.49.0\n\n[2.48.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.21...v2.48.22\n\n[2.48.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.20...v2.48.21\n\n[2.48.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.19...v2.48.20\n\n[2.48.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.18...v2.48.19\n\n[2.48.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.17...v2.48.18\n\n[2.48.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.16...v2.48.17\n\n[2.48.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.15...v2.48.16\n\n[2.48.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.14...v2.48.15\n\n[2.48.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.13...v2.48.14\n\n[2.48.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.12...v2.48.13\n\n[2.48.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.11...v2.48.12\n\n[2.48.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.10...v2.48.11\n\n[2.48.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.9...v2.48.10\n\n[2.48.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.8...v2.48.9\n\n[2.48.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.7...v2.48.8\n\n[2.48.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.6...v2.48.7\n\n[2.48.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.5...v2.48.6\n\n[2.48.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.4...v2.48.5\n\n[2.48.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.3...v2.48.4\n\n[2.48.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.2...v2.48.3\n\n[2.48.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.1...v2.48.2\n\n[2.48.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.0...v2.48.1\n\n[2.48.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.32...v2.48.0\n\n[2.47.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.31...v2.47.32\n\n[2.47.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.30...v2.47.31\n\n[2.47.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.29...v2.47.30\n\n[2.47.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.28...v2.47.29\n\n[2.47.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.27...v2.47.28\n\n[2.47.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.26...v2.47.27\n\n[2.47.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.25...v2.47.26\n\n[2.47.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.24...v2.47.25\n\n[2.47.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.23...v2.47.24\n\n[2.47.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.22...v2.47.23\n\n[2.47.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.21...v2.47.22\n\n[2.47.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.20...v2.47.21\n\n[2.47.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.19...v2.47.20\n\n[2.47.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.18...v2.47.19\n\n[2.47.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.17...v2.47.18\n\n[2.47.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.16...v2.47.17\n\n[2.47.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.15...v2.47.16\n\n[2.47.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.14...v2.47.15\n\n[2.47.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.13...v2.47.14\n\n[2.47.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.12...v2.47.13\n\n[2.47.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.11...v2.47.12\n\n[2.47.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.10...v2.47.11\n\n[2.47.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.9...v2.47.10\n\n[2.47.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.8...v2.47.9\n\n[2.47.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.7...v2.47.8\n\n[2.47.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.6...v2.47.7\n\n[2.47.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.5...v2.47.6\n\n[2.47.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.4...v2.47.5\n\n[2.47.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.3...v2.47.4\n\n[2.47.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.2...v2.47.3\n\n[2.47.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.1...v2.47.2\n\n[2.47.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.0...v2.47.1\n\n[2.47.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.20...v2.47.0\n\n[2.46.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.19...v2.46.20\n\n[2.46.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.18...v2.46.19\n\n[2.46.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.17...v2.46.18\n\n[2.46.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.16...v2.46.17\n\n[2.46.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.15...v2.46.16\n\n[2.46.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.14...v2.46.15\n\n[2.46.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.13...v2.46.14\n\n[2.46.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.12...v2.46.13\n\n[2.46.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.11...v2.46.12\n\n[2.46.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.10...v2.46.11\n\n[2.46.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.9...v2.46.10\n\n[2.46.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.8...v2.46.9\n\n[2.46.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.7...v2.46.8\n\n[2.46.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.6...v2.46.7\n\n[2.46.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.5...v2.46.6\n\n[2.46.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.4...v2.46.5\n\n[2.46.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.3...v2.46.4\n\n[2.46.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.2...v2.46.3\n\n[2.46.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.1...v2.46.2\n\n[2.46.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.0...v2.46.1\n\n[2.46.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.15...v2.46.0\n\n[2.45.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.14...v2.45.15\n\n[2.45.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.13...v2.45.14\n\n[2.45.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.12...v2.45.13\n\n[2.45.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.11...v2.45.12\n\n[2.45.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.10...v2.45.11\n\n[2.45.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.9...v2.45.10\n\n[2.45.9]: https://redirect.github.com/taiki-e/instal\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on the first day of the\nmonth\" (UTC), Automerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-02T15:20:28Z",
          "tree_id": "6288929369d7af6c90b5dfd277404d4deff1466e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/af5b129e030ce83f745f5b1a56725ea29ffb915c"
        },
        "date": 1770047383764,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.16744095087051392,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.95259057063497,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.39165072250985,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.91653645833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.31640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 519810.33870707,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 518939.9633677046,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0017,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11367870.242750902,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11313521.268386744,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "66651184+utpilla@users.noreply.github.com",
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "22dfe0b04aef2b541dd8b231181815a5853c7bf5",
          "message": "[otap-df-otap] Add TLS support for Syslog CEF Receiver (#1928)\n\n# Change Summary\n\n- Add TLS support for Syslog/CEF over TCP\n\n## What issue does this PR close?\n\n* Closes #1260 \n\n## How are these changes tested?\n\n- Only through some unit tests targeting TLS functionality for now\n- Need to add integration tests\n\n## Are there any user-facing changes?\n- Receiver config now allows for TLS settings\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-02-02T16:18:16Z",
          "tree_id": "2d905f61e4231d7ab4bf99504a85f06f422fb2e5",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/22dfe0b04aef2b541dd8b231181815a5853c7bf5"
        },
        "date": 1770052697660,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0107566118240356,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.01496512138688,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.46373248257704,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.29075520833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.0703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 525946.6613388937,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 531262.7015422477,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002556,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11582480.136599524,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11531576.42754349,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "a3bd796eb5d8b37008f40f52e16bbf25a0a10d28",
          "message": "fix(deps): update rust crate sysinfo to 0.38 (#1932)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [sysinfo](https://redirect.github.com/GuillaumeGomez/sysinfo) |\ndependencies | minor | `0.37` → `0.38` |\n| [sysinfo](https://redirect.github.com/GuillaumeGomez/sysinfo) |\nworkspace.dependencies | minor | `0.37` → `0.38` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>GuillaumeGomez/sysinfo (sysinfo)</summary>\n\n###\n[`v0.38.0`](https://redirect.github.com/GuillaumeGomez/sysinfo/blob/HEAD/CHANGELOG.md#0380)\n\n[Compare\nSource](https://redirect.github.com/GuillaumeGomez/sysinfo/compare/v0.37.2...v0.38.0)\n\n- Add NetBSD support.\n- Windows: Fix unsoundness for a function used in `Motherboard` and\n`Product`.\n- Linux: Improve CPU info parsing.\n- Fix `serde` serialization of `MacAddr` and of `Disk::file_system`.\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about these\nupdates again.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-02T16:51:25Z",
          "tree_id": "dae5ac6053f22f928a591e019cc8f0cc491ffd36",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a3bd796eb5d8b37008f40f52e16bbf25a0a10d28"
        },
        "date": 1770054031636,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.9309934377670288,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.75974220871487,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.94778760306288,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.5109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.75,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 516138.2620531628,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 526104.8580172227,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002031,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11428779.499007551,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11369620.975628467,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "843e8d6887f93cbca0d74586f368cacd81eade1e",
          "message": "Performance improvement for adding transport optimized encoding (#1927)\n\n# Change Summary\n\n- Optimizes the implementation of applying transport optimized encoding.\n- Renames `materialize_parent_id` bench to `transport_optimize` as this\nnow contains benchmarks that do both encoding & decoding\n\n**Benchmark summary:**\n\n| Benchmark | Size | Nulls | Before (µs) | After (µs) | Speedup |\nImprovement |\n\n|-----------|------|-------|-------------|------------|---------|-------------|\n| encode_transport_optimized_ids | 127 | No | 48.037 | 16.298 | 2.95x |\n66.1% faster |\n| encode_transport_optimized_ids | 127 | Yes | 47.768 | 18.446 | 2.59x |\n61.4% faster |\n| encode_transport_optimized_ids | 1536 | No | 518.36 | 98.955 | 5.24x |\n80.9% faster |\n| encode_transport_optimized_ids | 1536 | Yes | 520.94 | 107.01 | 4.87x\n| 79.5% faster |\n| encode_transport_optimized_ids | 8096 | No | 3418.3 | 508.92 | 6.72x |\n85.1% faster |\n| encode_transport_optimized_ids | 8096 | Yes | 3359.5 | 545.16 | 6.16x\n| 83.8% faster |\n\nNulls* column above signifies there were null rows in the attribute\nvalues column. Ordinarily we wouldn't encode attributes like this in\nOTAP because it we'd use the AttributeValuesType::Empty value in the\ntype column, but we handle it because it is valid arrow data since the\ncolumns are nullable.\n\n**Context:** \nwhen fixing #966 we added code to eagerly remove the transport optimized\nencoding from when transforming attributes, and noticed a significant\nregression in the performance benchmarks, especially on OTAP-ATTR-OTAP\nscenario because we do a round trip decode/encode of the transport\noptimized encoding.\n\n**Changes**\n\nThis PR specifically focuses on optimizing adding the transport\noptimized encoding for attributes, as this is where all the time was\nbeing spent. Adding this encoding involves sorting the attribute record\nbatch by type, key, value, then parent_id, and adding delta encoding to\nthe parent_id column for sequences where type, key and value are all\nequal to the previous row (unless value is null, or the type is Map or\nSlice).\n\nBefore this change, we were doing this sorting using arrow's\n`RowConverter`. We'd then do a second pass over the dataset to find\nsequences where type/key/value were equal, and apply the delta encoding\nto the parent_id column.\n\nAlthough using the `RowConverter` is sometimes [an efficient way to sort\nmultiple\ncolumns](https://arrow.apache.org/blog/2022/11/07/multi-column-sorts-in-arrow-rust-part-2/),\nit's notable that the `RowConverter` actually expands the dictionaries\nfor all the columns before it sorts (see\nhttps://github.com/apache/arrow-rs/issues/4811). This is extremely\nexpensive for us since most of our attribute columns are dictionary\nencoded.\n\nThis PR changes the implementation to sort the attributes record batch\ndirectly, starting by combining type & key together (using the sorted\ndictionary values from the keys column), then sorting this hybrid\ncolumn. It then partitions the type column to identify the attributes\nvalue column for this segment of the sorted result, and partitions the\nkey column to find segments of the value column to sort together. For\neach segment, it sorts it, appends it to a builder for the new values\ncolumn. It then partitions the sorted segment of values and for each\nsegment takes the parent_ids for the value segment, sorts them, adds\ndelta encoding, and appends these to a buffer containing the encoded\nparent IDs. Then it combines everything together and produces the\nresult.\n\nThe advantages of this approach are a) it's a lot faster and b) we build\nup enough state during the sorting that we don't need to do a second\npass over the `RecordBatch` to add delta encoding.\n\nThere are quite a few transformations that happen, and I tried to do\nthese as efficiently as possible. This means working with arrow's\nbuffers directly in many places, instead of always using immutable\n`Array`s and compute kernels, which reduces quite a lot the amount of\nallocations.\n\n**Future Work/Followups**\nThere are some code paths I didn't spent a lot of time optimizing:\n- If the parent_id is a u32 which may be dictionary encoded, we simply\ncast it to a primitive array and then cast it back into a dict when\nwe're done. I did some quick testing and figure this adds ~10% overhead.\n- If the value type is something that could be in a dictionary (string,\nint, bytes, & ser columns), but isn't dictionary encoded, or if the type\nis boolean, the way we build up the result column allocates many small\narrays. This could be improved\n- If the key column is not dictionary encoded. I didn't spend very much\ntime optimizing this.\n\nThere's also probably some methods that we were using before to encode\nthe ID column that I need to go back and delete\n\n## What issue does this PR close?\n\nRelated to #1853 \n\n## How are these changes tested?\n\nExisting unit tests plus new ones\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-02-02T23:55:12Z",
          "tree_id": "543096c9995627492ec66d70fac814fd2bb0ba5f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/843e8d6887f93cbca0d74586f368cacd81eade1e"
        },
        "date": 1770080483595,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0815362930297852,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.41742786345088,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.75356586699621,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.80494791666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.3203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 516788.05041765043,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 522377.30065746355,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000892,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11380428.17906754,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11326712.730273033,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "dab43aec0e346bfc2d7bd3f8e4c08747ad8ddf48",
          "message": "feat: add durable_buffer processor to otap-dataflow (#1882)\n\n# Change Summary\n\nAdds the `durable_buffer` processor to `otap-dataflow`, providing\ndurable buffering via Quiver's WAL and segment storage.\n\n## What issue does this PR close?\n\nCloses #1416\n\n## How are these changes tested?\n\nAdded unit tests, basic e2e tests & have performed manual validation\n\n## Are there any user-facing changes?\n\nYes. This PR adds the ability to configure a `durable_buffer` processor\nin the pipeline. For example:\n\n``` yaml\n  persistence:\n    kind: processor\n    plugin_urn: \"urn:otel:durable_buffer:processor\"\n    out_ports:\n      out_port:\n        destinations:\n          - noop\n        dispatch_strategy: round_robin\n    config:\n      path: /var/lib/otap/buffer\n      poll_interval: 10ms\n      retention_size_cap: 10 GiB\n      size_cap_policy: backpressure\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-02-03T15:37:31Z",
          "tree_id": "7aabe7edc36bab4d21261271549fc7f6300744ba",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dab43aec0e346bfc2d7bd3f8e4c08747ad8ddf48"
        },
        "date": 1770137254034,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0761040449142456,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.12685287413447,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.60876527217975,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.28684895833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.1875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527265.8202923073,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 532939.748969984,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007804,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11703177.464778202,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11648711.659969697,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "66651184+utpilla@users.noreply.github.com",
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "873c41457c4190c8b2c72f9f7c42cfde272d3665",
          "message": "[otap-df-otap] Update Syslog CEF Receiver to skip body for successfully parsed messages (#1940)\n\n# Change Summary\n\n- This PR optimizes storage by not duplicating data in the log body when\nmessages are fully parsed. For successfully parsed messages, body is now\nnull instead of containing the original input.\n- Fix process id handling for [RFC\n5424](https://www.rfc-editor.org/rfc/rfc5424) to comply with the\nspecification. As per RFC 5424, `PROCID = 1*128PRINTUSASCII` - It can be\nany printable ASCII string, not just numeric. Previously, non-numeric\nvalues were silently converted to 0 and lost. Now we store:\n\n- `syslog.process_id_str` (string) - always present when `proc_id`\nexists, contains the original value\n- `syslog.process_id` (integer) - only present if the value is parseable\nas an integer\n\nRFC 3164 behavior is unchanged (`proc_id` is conventionally numeric in\nthat format).\n\n## What issue does this PR close?\n\nRelated to #1149 \n\n## How are these changes tested?\n\nAdded tests for mixed fully-parsed and partially-parsed messages to\nverify:\n\n- Body is null for fully parsed messages\n- Body contains original input for partially parsed messages\n\nAdded a test for RFC 5424 proc_id parsing as well to ensure that\n`process_id_str` is always logged and `process_id` is only logged when\nit can be parsed into an integer.\n\n## Are there any user-facing changes?\n\nYes, users would now see `syslog.process_id_str` attribute always being\nlogged for valid RFC5424 messages.",
          "timestamp": "2026-02-03T19:34:23Z",
          "tree_id": "ff06bfdd339ba8624aa9257e5f54bf8d35ee21e2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/873c41457c4190c8b2c72f9f7c42cfde272d3665"
        },
        "date": 1770151061935,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.1209945678710938,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.07535066618274,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.54049863115516,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.64140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.5546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 523008.1124870202,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 534101.0862231216,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001945,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11525671.646281697,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11475734.627651278,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "af1e8e04e20c0b020bf6cd3d33eb8ccebb781314",
          "message": "feat: add Windows support for CI workflows and conditional compilation in metrics and exporter modules (#1939)\n\n# Change Summary\n\nEnable `cargo clippy` and `cargo fmt` on Windows for CI\n\n## What issue does this PR close?\n\n* Closes #1938\n\n## How are these changes tested?\n\n* Validated that clippy and fmt are clean on Windows\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-02-04T00:37:13Z",
          "tree_id": "fd7e8c719fbb0cbf02d2ed726a608d3cd631bc5a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/af1e8e04e20c0b020bf6cd3d33eb8ccebb781314"
        },
        "date": 1770175329033,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.1803622245788574,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.3902181420131,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.942462880582,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.804036458333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.0859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526381.8280800416,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 537858.8582284951,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001585,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11648378.639040656,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11594960.687117537,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "totan@microsoft.com",
            "name": "Tom Tan",
            "username": "ThomsonTan"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a4cb065c991d01b042e3cb0b7ed2bad73ccae929",
          "message": "[docs] add link to contribute page (#1945)",
          "timestamp": "2026-02-04T15:57:34Z",
          "tree_id": "d34f319ebb8cff0332da2eaaf6abf176572ac65d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a4cb065c991d01b042e3cb0b7ed2bad73ccae929"
        },
        "date": 1770223682286,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.149399757385254,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.91477355527867,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.58183913232105,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.37291666666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.44140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 516092.5347853245,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 527185.4268064369,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002387,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11530413.996387336,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11478392.08538756,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "66651184+utpilla@users.noreply.github.com",
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "26f00148a53e133941df54673ef06115f1a3454e",
          "message": "[otap-df-otap] Syslog CEF Receiver minor refactoring (#1946)\n\n# Change Summary\n- Avoid unnecessary conversion of bytes to `&str` for `input()` method\n- Minor edits",
          "timestamp": "2026-02-04T17:29:11Z",
          "tree_id": "5edab205058e25fe4c5f5326529af4af802d3685",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/26f00148a53e133941df54673ef06115f1a3454e"
        },
        "date": 1770233252359,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.7343299388885498,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.11349895104702,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.41272554209381,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.72018229166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.95703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 523972.06236417976,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 533059.4668362733,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003932,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11624773.910435833,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11568464.680494828,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "67cb11e83f5778f99638a5f1807fc75dfada5fc2",
          "message": "fix(tests): Switch from assert!(result.is_ok()) => result.unwrap() for CI diagnosability (#1937)\n\n# Change Summary\n\nSwitch pattern from `assert!(result.is_ok())` to `result.unwrap()` in\nexporter tests. This is to improve diagnostics for flakey tests in CI.\nCurrently, failures output the following which is not actionable:\n\n```\n    thread 'parquet_exporter::test::test_traces' (2500) panicked at crates\\otap\\src\\parquet_exporter.rs:1299:21:\n    assertion failed: exporter_result.is_ok()\n    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\n```\n\nWith the change above, the error string from the result will be properly\nlogged.\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nn/a\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-02-04T17:53:31Z",
          "tree_id": "8ef850e4db3fe0cbc477c736c2681b3a11ae7ebe",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/67cb11e83f5778f99638a5f1807fc75dfada5fc2"
        },
        "date": 1770237979635,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.2255918979644775,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.50091759158374,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.88943894761279,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.07643229166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.90234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 523343.68688398926,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 534991.181382728,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002604,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11690599.469129471,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11638185.741429718,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "65b8becc4dfeeacacbf77b74c0329703fd8d2ef6",
          "message": "PerfTest - tweak idle state test to confirm memory growth pattern (#1942)\n\n# Change Summary\n\nModified the Idle State Test to run on 1/2/4/8/16/32 cores and confirm\nif the memory growth (idle state) is predictable.\n\n## What issue does this PR close?\n\nPart of the comment\nhttps://github.com/open-telemetry/otel-arrow/pull/1528/changes#r2710193083\n\n## How are these changes tested?\n\nRan locally.\n\n## Are there any user-facing changes?\nNo",
          "timestamp": "2026-02-04T19:15:31Z",
          "tree_id": "d455abab756cfd7f4190ab48868cefd5ccc08e47",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/65b8becc4dfeeacacbf77b74c0329703fd8d2ef6"
        },
        "date": 1770241456036,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.3076750040054321,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.12370101272201,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.54858708788231,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.381640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 528563.3996503989,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 535475.290672917,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000946,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11533476.820639638,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11485141.769861262,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "00600327d39aee678e2f63bc5cd7cf99be343977",
          "message": "Remove OTel logging SDK in favor of internal logging setup (#1936)\n\n# Change Summary\n\nRemoves the OTel logging SDK since we have otap-dataflow-internal\nlogging configurable in numerous ways. Updates OTel feature settings to\ndisable the OTel logging SDK from the build.\n\n## What issue does this PR close?\n\nRemoves `ProviderMode::OpenTelemetry`, the OTel logging SDK and its\nassociated configuration (service::telemetry::logs::processors::*).\n\nFixes #1576.\n\n## Are there any user-facing changes?\n\nYes.\n\n**Note: this removes the potential to use the OpenTelemetry tracing\nsupport via the opentelemetry tracing appender. However, we view tracing\ninstrumentation as having limited value until otap-dataflow is properly\ninstrumented for tracing. When this happens, we are likely to use an\ninternal tracing pipeline.**\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-02-04T23:21:04Z",
          "tree_id": "c90db5a0c6669f33adf3d3fbd35290be6424113b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/00600327d39aee678e2f63bc5cd7cf99be343977"
        },
        "date": 1770251186926,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.212749481201172,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.03057774048962,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.39984212383949,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.10546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 524459.7198298879,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 536064.6988245292,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001832,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11649933.077538436,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11593468.637266029,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "66651184+utpilla@users.noreply.github.com",
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e986683cebfc0d75fce626d10f1e5a5a5d678f5f",
          "message": "[otap-df-otap] Update Syslog CEF Receiver README (#1943)\n\n# Change Summary\n- Update Syslog CEF Receiver README\n\n---------\n\nCo-authored-by: Cijo Thomas <cijo.thomas@gmail.com>",
          "timestamp": "2026-02-05T02:26:48Z",
          "tree_id": "1afcdb08fca2bb345ce37ff81bfad4bfaada8c15",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e986683cebfc0d75fce626d10f1e5a5a5d678f5f"
        },
        "date": 1770262596186,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7971684336662292,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.91703938174732,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.36182366543665,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.32643229166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.13671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 535174.1891557083,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 539440.4289850801,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006003,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11643948.484311352,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11596124.431486292,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "8c726ba2cb1ff2463db6c67ed0f03b102d322a54",
          "message": "OTLP receiver: enable HTTP-only mode (#1925)\n\n# Change Summary\n\nThis PR restructures the OTLP receiver configuration to support flexible\nprotocol deployment modes, aligning with the Go collector's otlpreceiver\nmodel:\n- gRPC only - Configure only protocols.grpc\n- HTTP only - Configure only protocols.http (new!)\n  - Both protocols - Configure both with a global concurrency cap   \n\n## Key Changes\n### Configuration restructure:\n- Moved from flat config to protocols.grpc / protocols.http structure\n- TLS configuration is now per-protocol (under each protocol's config)\n- At least one protocol must be configured (validated at startup)\n### Concurrency model for dual-protocol mode:\n- Each protocol enforces its own max_concurrent_requests limit\n- When both protocols are enabled, an additional global semaphore caps\ncombined load to prevent exceeding downstream capacity\n- Permits acquired in consistent order (global -> local) to prevent\ndeadlocks\n\n## What issue does this PR close?\n\n* Closes #1893\n\n## How are these changes tested?\n\n Manual tested, along with unit tests.\n\n## Are there any user-facing changes?\n\n⚠️ Breaking change: The OTLP receiver configuration format has changed.\n  **_Before_**:\n\n```yaml                                                                                              \n  config:                                                                                                                                                                             \n    listening_addr: \"127.0.0.1:4317\"                                                                                                                                                  \n    tls:                                                                                                                                                                              \n      cert_file: \"/path/to/cert\"                                                                                                                                                      \n    http:                                                                                                                                                                             \n      listening_addr: \"127.0.0.1:4318\"\n```\n  **_After_**:\n\n```yaml                                                                                                                                                                   \n  config:                                                                                                                                                                             \n    protocols:                                                                                                                                                                        \n      grpc:                                                                                                                                                                           \n        listening_addr: \"127.0.0.1:4317\"                                                                                                                                              \n        tls:                                                                                                                                                                          \n          cert_file: \"/path/to/cert\"                                                                                                                                                  \n      http:                                                                                                                                                                           \n        listening_addr: \"127.0.0.1:4318\"                                                                                                                                              \n        tls:                                                                                                                                                                          \n          cert_file: \"/path/to/cert\"\n```\nRefer to `otlp_receiver.md` (updated in this PR) for more details.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-02-05T07:37:32Z",
          "tree_id": "eb856afc70e086d0007c667769f06b8d6a12ebf1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8c726ba2cb1ff2463db6c67ed0f03b102d322a54"
        },
        "date": 1770280995460,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2983384132385254,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.40433413630542,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.85078608938115,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.544010416666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.40625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 519207.1554230734,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 525948.2215854845,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002378,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11417712.903308954,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11364124.388817286,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "56887295a09a2ba52bcd736ffba8852d9293227c",
          "message": "Fanout Processor (#1878)\n\n## Fan-out Processor Implementation\n\nImplements all four discussed scenarios:\n\n| Scenario | Config | Description |\n|----------|--------|-------------|\n| 1 | `mode: parallel, await_ack: primary` | Duplicate to all, wait for\nprimary only |\n| 2 | `mode: parallel, await_ack: all` | Duplicate to all, wait for all\n(with per-destination timeout) |\n| 3 | `mode: sequential` | Send one-by-one, advance after ack |\n| 4 | `fallback_for: <port>` | Failover to backup on nack/timeout |\n\n### Why Stateful (not Stateless like Go collector)\n\nThe Go Collector's fanout is stateless because it uses **synchronous,\nblocking calls**:\n```go\nerr := consumer.ConsumeLogs(ctx, ld)  // blocks until complete, error returns directly\n```\n\nOur OTAP engine uses async message passing with explicit ack/nack\nrouting:\n\n```rust\neffect_handler.send_message_to(port, pdata).await?;  // returns immediately\n// ack arrives later as separate NodeControlMsg::Ack\n```\nI explored making scenarios 1 and 3 stateless but hit three blockers:\n\n1. **`subscribe_to()` mutates context** - Fanout must subscribe to\nreceive acks, which pushes a frame onto the context stack. For correct\nupstream routing, we need the *original* pdata (pre-subscription). We\ncannot use `ack.accepted` from downstream.\n\n2. **Downstream may mutate/drop payloads** - `into_parts()`,\ntransformers, and filters mean we can't rely on getting intact pdata\nback in ack/nack messages.\n\n3. **Sequential/fallback/timeout require coordination** - Need to know\nwhich destination is active, when to advance to the next, and when to\ntrigger fallbacks or finish.\n\nEven if downstream guaranteed returning intact payloads, we'd still need\nstate for `await_all` completion tracking, fallback chains, and\nsequential advancement. The only gain would be a minor memory\noptimization (not storing `original_pdata`), not true statelessness.\n\nAdopting Go's synchronous model would require fundamental engine\narchitecture changes, not just fanout changes.\n\n### Memory Optimizations\n\nWhile full statelessness isn't possible, I have implemented fast paths\nto minimize allocations for common configurations:\n\n| Configuration | Fast Path | State Per Request |\n\n|-----------------------------------------------------------|------------------|------------------------------------------------|\n| `await_ack: none` | Fire-and-forget | None (zero inflight tracking) |\n| `parallel + primary + no fallback + no timeout` | Slim primary |\nMinimal (`request_id → original_pdata`) |\n| All other configs | Full | Complete endpoint tracking |\n\n#### Fast Path Details\n\n- **Fire-and-forget (`await_ack: none`)**  \nBypasses all inflight state. Clone, send, and ACK upstream immediately.\n  Zero allocations per request.\n\n- **Slim primary path**  \nUses a tiny `HashMap<u64, OtapPdata>` instead of the full `Inflight`\nstruct with `EndpointVec`.\n  Ignores non-primary ACKs and NACKs.\n\n- **Full path**  \n  Required for:\n  - Sequential mode  \n  - `await_all`  \n  - Any fallback  \n  - Any timeout  \n\n  Tracks all endpoints and request state.\n\n### Code Structure\n\n`Inflight` holds per-request state:\n- `original_pdata` - pre-subscription pdata, used for all upstream\nacks/nacks\n- `endpoints[]` - per-destination status\n(`Acked`/`Nacked`/`InFlight`/`PendingSend`)\n- `next_send_queue` - drives sequential mode advancement\n- `completed_origins` - tracks completion for `await_ack: all`\n- `timeout_at` - per-destination deadlines for timeout/fallback\ntriggering\n\nNot all fields are used for every scenario, but the overhead is minimal\n- empty HashSets don't allocate, SmallVec is inline for ≤4 items, and\nclone cost is O(1) for `bytes::Bytes`.\n\n### Documentation\n\nSee\n[`crates/otap/src/fanout_processor/README.md`](crates/otap/src/fanout_processor/README.md)\nfor configuration examples and behavior details.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-02-05T17:31:50Z",
          "tree_id": "8ed430a68b4bdcfaa58b83efa9911da2c181a023",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/56887295a09a2ba52bcd736ffba8852d9293227c"
        },
        "date": 1770319690018,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.199604034423828,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.18525098398337,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.58986465626448,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.60546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 525654.9545052535,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 537217.281432214,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001763,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11495069.051917732,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11443743.469279978,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "f182711855e702a2042f15246919ebe30f844bda",
          "message": "Add additional Rust-CI clippy/fmt for more OS values (#1965)\n\n# Change Summary\n\nFollow-up from 2026-02-05 SIG meeting\n\nRequested to add `clippy` and `fmt` for the 4 OS targets already\ntargeted in `test_and_coverage`\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nCI runs\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-05T17:36:09Z",
          "tree_id": "dd6888619fe687813c10c7cc326f29554ba28c70",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f182711855e702a2042f15246919ebe30f844bda"
        },
        "date": 1770322788242,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.527256965637207,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.19742017469905,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.65899562610772,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.98658854166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.1171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526722.8539283249,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 540034.4947358838,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001619,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11653337.31561002,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11602409.140686888,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "d.dahl@f5.com",
            "name": "David Dahl",
            "username": "daviddahl"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0824c0193edf0ff675fc33b5b0af470e254928b0",
          "message": "Standardization of urns, validation and usage (#1948)\n\n# Change Summary\n\nStandardized otap-df-otap node URNs to the canonical\nurn:<namespace>:<id>:<kind> format, added strict parsing/normalization\n(including OTel shortcut support), updated component\nconstants/configs/templates/docs to match, and documented otelcol config\n  compatibility design and URN rules.\n\n  ## What issue does this PR close?\n\n  - Closes #1831\n\n  ## How are these changes tested?\n\n  - cargo test (per local confirmation)\n- Added unit/config tests for URN normalization and legacy URN rejection\nin otap_df_config\n\n  ## Are there any user-facing changes?\n\nYes. Configuration now enforces canonical URN format and accepts the\nOTel shortcut form; legacy URNs are rejected with a doc-linked\n  error message.",
          "timestamp": "2026-02-05T18:53:12Z",
          "tree_id": "012cb22877f7a9665f03d772d2537cfc4933b66d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0824c0193edf0ff675fc33b5b0af470e254928b0"
        },
        "date": 1770326078830,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.134752035140991,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.12021018010863,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.97175865582656,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.446614583333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 519633.1470042875,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 530726.0266388438,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002454,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11560124.522144636,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11499354.316789607,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "fcc18902578ae018e6a652347113a2f603fc615c",
          "message": "chore(deps): update dependency go to v1.25.7 (#1959)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [go](https://go.dev/)\n([source](https://redirect.github.com/golang/go)) | toolchain | patch |\n`1.25.6` → `1.25.7` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>golang/go (go)</summary>\n\n###\n[`v1.25.7`](https://redirect.github.com/golang/go/compare/go1.25.6...go1.25.7)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45NS4yIiwidXBkYXRlZEluVmVyIjoiNDIuOTUuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-05T21:35:38Z",
          "tree_id": "9a57ae35d3da4c3e43f9bf208490a83f10487842",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fcc18902578ae018e6a652347113a2f603fc615c"
        },
        "date": 1770331589904,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9017422795295715,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.8323820904726,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.11985844198124,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.883463541666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.86328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 529794.5872423376,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 534571.9691117735,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.016136,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11487138.047541022,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11437351.331856543,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "1363f1071a18e405664a07c25e6adaf410b8dd3a",
          "message": "fix: improve reliability of test_durable_buffer_recovery_after_outage (#1976)\n\n# Change Summary\n\nImprove the reliability of `test_durable_buffer_recovery_after_outage`\nso that it is not subject to minor timing differences across runs that\nmay lead to test failure. Make test more precise by validating the exact\nnumber of signals persisted and received by the exporter.\n\n## What issue does this PR close?\n\n* Closes #1975\n\n## How are these changes tested?\n\n* Code inspection, manually running the test to attempt failure repro.\n\n## Are there any user-facing changes?\n\nNo, this is change only affects test code.",
          "timestamp": "2026-02-06T00:05:14Z",
          "tree_id": "17359dbaa05b6905edf0108133cc6bdf7fbcc0c5",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1363f1071a18e405664a07c25e6adaf410b8dd3a"
        },
        "date": 1770340825359,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.157910108566284,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.72299688856893,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.4453542702284,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.7203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.68359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 523945.9977613091,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 535252.281710264,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002031,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11472116.097592456,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11415637.81303057,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "Tom.Tan@microsoft.com",
            "name": "Tom Tan",
            "username": "ThomsonTan"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "436d0bac02078e5bbe70240807ac852287387a94",
          "message": "[ci] add triage:deciding label to new issues (#1968)\n\n# Change Summary\n\nBased on the discussion in today's SIG, add CI task to apply label\ntriage:diciding to new issues for later triage.",
          "timestamp": "2026-02-06T11:09:29Z",
          "tree_id": "7a1982a52ddf7849b128c61150f07a3c4c10b9be",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/436d0bac02078e5bbe70240807ac852287387a94"
        },
        "date": 1770380393545,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.179950475692749,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.22037665708564,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.69396939130435,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.081119791666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.8203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527916.7492962658,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 534145.905825373,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001703,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11552817.839959968,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11505658.084909212,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aloc@techie.com",
            "name": "Arthur Câmara",
            "username": "alochaus"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "124487caac570c285d5cb272b3a54efb0fac5d4e",
          "message": "fix: implement num_items for OTLP metrics to count data points. (#1963)\n\n# Change Summary\n\nWhen processing OTLP metrics, calling `OtlpProtoBytes::num_items()`\npanics with the message `ToDo`. This happens because metrics_data_view\nwas previously unimplemented, but has since been added without the\ncorresponding counter logic for num_items(). This PR implements this\nlogic.\n\nImportant to mention that the implementation counts data points since\n`otap.rs` does the same thing in its definition of `num_items`.\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/8c726ba2cb1ff2463db6c67ed0f03b102d322a54/rust/otap-dataflow/crates/pdata/src/otap.rs#L423-L430\n\n## What issue does this PR close?\n\n* https://github.com/open-telemetry/otel-arrow/issues/1923\n\n## How are these changes tested?\n\nTODO\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-06T12:59:09Z",
          "tree_id": "13b8a97de31ba83ae90a202ef7a394bd283e4955",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/124487caac570c285d5cb272b3a54efb0fac5d4e"
        },
        "date": 1770387677629,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8501491546630859,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.69792025110317,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.30070570408321,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.73020833333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.0078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 531925.6151137508,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 536447.7763535295,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006706,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11590747.911740575,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11531139.140726548,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "82d68ca1d58f39c2619f6d9709f6d0a394c3e671",
          "message": "Add AuthError source in azure_monitor_exporter (#1979)\n\n# Change Summary\n\nTroubleshooting some transient Auth errors using\n`azure_monitor_exporter` component. We should expose the error coming\nfrom `azure_core` crate.\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Cijo Thomas <cijo.thomas@gmail.com>",
          "timestamp": "2026-02-06T18:09:51Z",
          "tree_id": "3f152bb60eccc6abaada8d03711d11cc9621dbf1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/82d68ca1d58f39c2619f6d9709f6d0a394c3e671"
        },
        "date": 1770405943343,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.957058310508728,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.75921295535035,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.07578117191733,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.241015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.70703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 525393.454314288,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 535675.7110616523,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002392,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11688114.389404248,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11631666.843621189,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "e95eee989bfc60af479e1e780b82f76b0702a897",
          "message": "Add WAL Replay support for crash recovery (#1954)\n\n# Change Summary\n\nAdd WAL replay support for crash recovery in Quiver. On engine startup,\n`QuiverEngine::open()` now replays any WAL entries that were written but\nnot yet finalized to segments, ensuring recover of data which had been\nwritten to the WAL, but not yet finalized in a segment file. The\nimplementation includes a new `MultiFileWalReader` that reads entries\nacross rotated WAL files in global position order, and a `ReplayBundle`\ntype that decodes WAL entries back into `RecordBundle` implementations\nfor replay through the normal ingest path. The replay logic respects the\npersisted cursor to skip already-finalized entries and handles edge\ncases like truncated entries (crash mid-write) and corrupted entries\n(CRC mismatch) by stopping replay at the first invalid entry rather than\nfailing startup.\n\n## What issue does this PR close?\n\n* Closes #1951 \n\n## How are these changes tested?\n\n- Added unit tests for MultiFileWalReader covering single-file reads,\nmulti-file iteration, mid-stream starts, and WAL position preservation\n- Added unit tests for ReplayBundle verifying IPC payload decoding,\nmulti-slot reconstruction, timestamp handling, and error cases\n- Added tests for end-to-end WAL replay scenarios including recovery of\nunfinalized bundles, cursor-based deduplication, empty/missing WAL\nhandling, segment finalization during replay, multi-file replay after\nrotation, and graceful recovery from truncated and corrupted WAL\nentries.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-02-06T18:27:22Z",
          "tree_id": "c1501e431944b21f5add24f80adca0c7d41ed4df",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e95eee989bfc60af479e1e780b82f76b0702a897"
        },
        "date": 1770409254372,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.8276596069335938,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.46367575201647,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.84699871365775,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.08216145833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.68359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 536923.0619746747,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 546736.1880874949,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001267,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11739836.588009609,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11685413.648014259,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "a9fbb0ed89ecc74f8a67e731a3186d45c589333a",
          "message": "feat: Add message parameter to otel_debug macro (#1973)\n\n# Change Summary\n\nAdding \"message\" attribute to the otel_debug macro.\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1972\n\n## What issue does this PR close?\n\n\n* Closes NA\n\n## How are these changes tested?\n\nBuilding the package\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-06T20:47:49Z",
          "tree_id": "778c0652dd59aaa67bce252e7ac30380fb815d86",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a9fbb0ed89ecc74f8a67e731a3186d45c589333a"
        },
        "date": 1770415345134,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.4265170097351074,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.54367242024608,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.03171241384652,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.46875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.03125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527483.0354585524,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 540282.5008675471,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002506,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11662959.452482764,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11608099.5117847,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "66651184+utpilla@users.noreply.github.com",
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "a2c71012e8bdb1a5f6bef4ff435306df97756260",
          "message": "[otap-df-otap] Implement graceful shutdown for Syslog CEF Receiver (#1962)\n\n# Change Summary\n\n1. Proper Shutdown Deadline Handling: Both TCP and UDP now capture the\ndeadline from `NodeControlMsg::Shutdown` and return\n`TerminalState::new(deadline, [snapshot])` instead of\n`TerminalState::default()`\n2. UDP Graceful Flush: On shutdown, flushes any pending records in\n`arrow_records_builder` using `try_send_message_with_source_node()`\nbefore returning. Uses `try_send` (non-blocking) since we're shutting\ndown and can't wait indefinitely\n3. TCP Task Shutdown Signaling:\n- Added `Rc<Cell<bool>>` shutdown flag to signal spawned connection\ntasks to flush and exit\n- Tasks check `shutdown_flag.get()` at the top of each loop iteration\n(cheap bool read, no locks)\n- When flag is set, tasks flush pending records via `try_send` and exit\ncleanly\n5. TCP Task Tracking & Graceful Drain:\n   - Added `Rc<Cell<usize>>` to track active spawned tasks\n- Tasks increment counter when starting, decrement at all exit points\n(shutdown, EOF, read error, TLS handshake failure)\n   - On shutdown, waits for tasks to finish with timeout:\n- Uses 90% of time until deadline, capped at 1 second\n(`MAX_TASK_DRAIN_WAIT`)\n- Busy-spins with `yield_now()` should be rare (acceptable during\nshutdown)\n   - Takes final metrics snapshot only after drain wait completes\n   \n# Key Design Decisions\n- Used `Rc<Cell<T>>` instead of `CancellationToken` - simpler, no\nexternal dependency, cheaper (just pointer deref + bool read)\n- Used `try_send` during shutdown flush - non-blocking, won't hang if\ndownstream is full\n- Rare case: All the tasks handling the active connections during\nshutdown are awaitnng I/O, then we could have a busy-spin during drain\nwait which would keep checking if the active task count is zero. I think\nthis is acceptable behavior during shutdown.\n\n## What issue does this PR close?\n\nRelated to #1149 \n\n## How are these changes tested?\n\n## Are there any user-facing changes?",
          "timestamp": "2026-02-06T20:50:02Z",
          "tree_id": "9ef7853afdf2282c47e3dc4bd6b4f6624c67f2a2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a2c71012e8bdb1a5f6bef4ff435306df97756260"
        },
        "date": 1770418645994,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9859403371810913,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.50603728977123,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.97549851407476,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.91588541666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.38671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 532213.4368085549,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 537460.7437267661,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007925,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11591441.273471197,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11544127.588000484,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "4f64cf46c72dd774e9db42df5984c953f0d1bb22",
          "message": "[otap-df-quiver] Implement time-based segment retention (max_age) for quiver & durable_buffer processor (#1961)\n\n# Change Summary\n\nThis PR implements time-based segment retention (`max_age`) for the\nquiver storage engine, allowing segments to be automatically deleted\nafter a configurable duration regardless of subscriber consumption\nstatus. The feature is *opt-in* (`max_age: None` by default) to avoid\nunexpected data loss. Segments are timestamped using file modification\ntime when finalized, and expired segments are cleaned up both during\nstartup (without loading them) and during periodic maintenance. The\nimplementation coordinates with the subscriber registry to\nforce-complete expired segments before deletion, ensuring subscribers\ndon't attempt to read from deleted files.\n\nAlso updates the `durable_buffer` processor to pass its existing\n`max_age` config option through to quiver, replacing the previous\nplaceholder implementation.\n\n## What issue does this PR close?\n\n* Closes #1960 \n\n## How are these changes tested?\n\nComprehensive unit tests cover the new functionality.\n\n## Are there any user-facing changes?\n\nAfter this change, the user-facing `max_age` setting on the\n`durable_buffer` processor will work as expected. (A `max_age` setting\nis being added to the Quiver configuration.)\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-02-06T20:54:27Z",
          "tree_id": "c7bb1f7a35db4bcc99f0e94b8eadc099537e50c0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4f64cf46c72dd774e9db42df5984c953f0d1bb22"
        },
        "date": 1770422165817,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0832583904266357,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.38802464005384,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.75510517783805,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.408854166666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.77734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 531721.4187402604,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 537481.335893452,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000863,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11634670.779383808,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11584579.441520654,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "3d8dc2c6eaf0ee1d288655d3736deb3b9e32ec4d",
          "message": "Fix internal logging macros (#1985)\n\nReverting https://github.com/open-telemetry/otel-arrow/pull/1973\nFixing the empty \"\" from our internal macros, that caused the\n`message=\"user friendly message here\"` from being omitted in stdout!\n\nTaking\nhttps://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/crates/controller/src/lib.rs#L668-L671\nas example\n```rust\notel_warn!(\n                \"core_affinity.set_failed\",\n                message = \"Failed to set core affinity for pipeline thread. Performance may be less predictable.\"\n            );\n```\n\nBefore\n```txt\n2026-02-06T22:15:09.891Z  WARN  otap-df-controller::core_affinity.set_failed (crates/controller/src/lib.rs:668): \n```\n(Missing message!)\n\nAfter (i.e with this PR)\n```txt\n2026-02-06T22:11:19.095Z  WARN  otap-df-controller::core_affinity.set_failed (crates/controller/src/lib.rs:668): Failed to set core affinity for pipeline thread. Performance may be less predictable.\n```\n(Message is back)\n\n\"message\" is already special cased in this repo, OTel Rust repo, and\n`tracing` itself. Passing user friendly string as an attribute named\n\"message\" is\n*[faster](https://github.com/open-telemetry/opentelemetry-rust/pull/2001/changes)*\ntoo!\n\nAlso, we avoid the less friendly syntax -\nhttps://github.com/open-telemetry/otel-arrow/pull/1981#discussion_r2776145173",
          "timestamp": "2026-02-06T22:38:34Z",
          "tree_id": "6f81ba35d91815c876bae0ba2c7845703f8d0e82",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3d8dc2c6eaf0ee1d288655d3736deb3b9e32ec4d"
        },
        "date": 1770425483444,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.405754327774048,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.6459419457019,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.01947331171839,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.83841145833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.72265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526719.8604845576,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 539391.4465460795,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00196,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11568213.032507593,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11521601.2065015,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "5ab5ccb795234e636b7bd06a61605713cb5104ee",
          "message": "feat: Remove line from event name on logs. (#1982)\n\n# Change Summary\n\nRemoves line number from event name to make it fixed.\n\n## What issue does this PR close?\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1972\n\n* Closes #N/A\n\n## How are these changes tested?\n\nMaking local calls.\n\n## Are there any user-facing changes?\n\nThe event name produced for internal telemetry does not include the line\nnumber now.",
          "timestamp": "2026-02-06T23:47:27Z",
          "tree_id": "af230a236e8658de4ec483e5c779df1abbbd2f74",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5ab5ccb795234e636b7bd06a61605713cb5104ee"
        },
        "date": 1770428781615,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.963584542274475,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.8678471087271,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.50765553232675,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.94908854166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.73046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 525824.8099162935,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 536149.8240346069,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001855,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11618580.18752669,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11572011.658397336,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "a623996f7f69b3575e2b83687a985ca323dc5f88",
          "message": "PerfTest - move saturation test to nightly only (#1969)\n\n# Change Summary\n\nSaturation tests were initially run continuously as we were figuring out\nthe right inputs. We are still not finalized, but I think it's now\nstable enough, and can be moved to nightly. These tests take 20+ minutes\nof scarce resource (perf machine!), so moving to nightly !\n\n## How are these changes tested?\nN/A\n\n## Are there any user-facing changes?\nNone.",
          "timestamp": "2026-02-06T23:48:29Z",
          "tree_id": "4c8f73e7ce516c9c7c19d59b3e070d615f39ef37",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a623996f7f69b3575e2b83687a985ca323dc5f88"
        },
        "date": 1770430569895,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2957693338394165,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.46254633990957,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.88526350310559,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.90546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.9921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 536700.6988751444,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 543655.1016222268,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002277,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11645952.358892716,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11596251.666552205,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "f71dbe1d88dfa33c53694a64234695dae693d2ec",
          "message": "test: allow for \"Channel is closed\" error during shutdown in durable buffer tests (#1986)\n\n# Change Summary\n\nMinor test reliability improvement. In the durable_buffer_tests, allow\nfor expected \"Channel is closed\" errors during shutdown. (We are seeing\nthese errors occasionally during PR checks.)\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nn/a\n\n## Are there any user-facing changes?\n\nNo. This is minor test reliability improvement.",
          "timestamp": "2026-02-06T23:51:30Z",
          "tree_id": "f973853c27ae12c23fc79369d0611806ef658ea8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f71dbe1d88dfa33c53694a64234695dae693d2ec"
        },
        "date": 1770434654670,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9981968402862549,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.39094005091991,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.73689150286333,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.3140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.94140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 530005.351494527,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 535295.8479835566,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00193,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11615487.16250684,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11563957.915925944,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "82f71508f8e598e78853335cce82195e894894cd",
          "message": "feat(quiver): skip expired WAL entries during replay (#1984)\n\n# Change Summary\n\nDuring WAL replay, entries older than the configured `max_age` retention\nare now skipped rather than replayed into new segments. Without this\nfiltering, replaying expired WAL entries would effectively reset their\nage to zero, causing data to be retained longer than intended by the\nconfigured policy. The cutoff is computed once before the replay loop\nand compared against each entry's ingestion_time (with no assumption\nabout WAL ordering). Skipped entries advance the cursor so they won't be\nretried, and the expired_bundles counter is incremented so operators\nhave visibility into filtered data. When *all* replayed entries are\nexpired (nothing is replayed), the cursor is explicitly persisted to the\nsidecar to avoid redundant re-scanning on subsequent restarts.\n\n## What issue does this PR close?\n\n* Closes #1980\n\n## How are these changes tested?\n\nTwo new tests cover the mixed old/fresh filtering case and the\nall-expired edge case, the latter including a third engine reopen to\nverify cursor persistence.\n\n## Are there any user-facing changes?\n\nNo, this is an optimization to the WAL recovery behavior. No config or\nuser-facing changes.",
          "timestamp": "2026-02-07T00:29:34Z",
          "tree_id": "f9d931ae4bfd39df396026e552cb13dd7e3f3608",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/82f71508f8e598e78853335cce82195e894894cd"
        },
        "date": 1770444885314,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0757559537887573,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.15222285644461,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.62250709767659,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.9609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527496.4948486254,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 533171.0693034572,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000975,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11538993.513163716,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11487789.241592813,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "66e251ff7f54c7a1cc9be20a3a86b6d2897a6341",
          "message": "chore(deps): update dependency grpcio to v1.78.0 (#1996)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [grpcio](https://redirect.github.com/grpc/grpc) | `==1.76.0` →\n`==1.78.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/grpcio/1.78.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/grpcio/1.76.0/1.78.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>grpc/grpc (grpcio)</summary>\n\n###\n[`v1.78.0`](https://redirect.github.com/grpc/grpc/releases/tag/v1.78.0)\n\n[Compare\nSource](https://redirect.github.com/grpc/grpc/compare/v1.76.0...v1.78.0)\n\nThis is release 1.78.0\n([gutsy](https://redirect.github.com/grpc/grpc/blob/master/doc/g_stands_for.md))\nof gRPC Core.\n\nFor gRPC documentation, see [grpc.io](https://grpc.io/). For previous\nreleases, see\n[Releases](https://redirect.github.com/grpc/grpc/releases).\n\nThis release contains refinements, improvements, and bug fixes, with\nhighlights listed below.\n\n## C++\n\n- adding address\\_sorting dep in naming test build.\n([#&#8203;41045](https://redirect.github.com/grpc/grpc/pull/41045))\n\n## Objective-C\n\n- \\[Backport]\\[v1.78.x]\\[Fix]\\[Compiler] Plugins fall back to the\nedition 2023 for older protobuf.\n([#&#8203;41358](https://redirect.github.com/grpc/grpc/pull/41358))\n\n## Python\n\n- \\[python] aio: fix race condition causing `asyncio.run()` to hang\nforever during the shutdown process.\n([#&#8203;40989](https://redirect.github.com/grpc/grpc/pull/40989))\n- \\[Python] Migrate to pyproject.toml build system from setup.py builds.\n([#&#8203;40833](https://redirect.github.com/grpc/grpc/pull/40833))\n- \\[Python] Log error details when ExecuteBatchError occurs (at DEBUG\nlevel).\n([#&#8203;40921](https://redirect.github.com/grpc/grpc/pull/40921))\n- \\[Python] Update setuptools min version to 77.0.1 .\n([#&#8203;40931](https://redirect.github.com/grpc/grpc/pull/40931))\n\n## Ruby\n\n- \\[ruby] Fix version comparison for the ruby\\_abi\\_version symbol for\nruby 4 compatibility.\n([#&#8203;41061](https://redirect.github.com/grpc/grpc/pull/41061))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45NS4yIiwidXBkYXRlZEluVmVyIjoiNDIuOTUuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-02-09T12:46:52Z",
          "tree_id": "0cac7acc8e5abf312935a8b81966a61cdebd48eb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/66e251ff7f54c7a1cc9be20a3a86b6d2897a6341"
        },
        "date": 1770645956511,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0584660768508911,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.0699969148428,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.33395204932562,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.559765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 528051.9709693601,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 533641.2221407013,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000882,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11663157.90243817,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11607548.570189454,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "ac156091ee91d828561e21f9c4380e438c5e403f",
          "message": "chore: bump rand from 0.9.2 to v0.10.0 (#2000)\n\n### Description:\n\n  Updates rand dependency from 0.9.2 to 0.10.0.\n\nThe main breaking change affecting this codebase is the trait rename\n`Rng` -> `RngExt` as indicated in [migration\nguide](https://rust-random.github.io/book/update-0.10.html):\n\n``\nUsers of rand will often need to import rand::RngExt may need to migrate\nfrom R: RngCore to R: Rng (noting that where R: Rng was previously used\nit may be preferable to keep R: Rng even though the direct replacement\nwould be R: RngExt; the two bounds are equivalent for R: Sized).\n``\n\nNote - this supersede #1997 which is failing for these breaking changes\nin newer version.",
          "timestamp": "2026-02-09T12:48:08Z",
          "tree_id": "4e183d0019d0d5d1ab6d4f4c0c41302c510b11ab",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac156091ee91d828561e21f9c4380e438c5e403f"
        },
        "date": 1770647705077,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.0594661235809326,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.53870782490264,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.95722888234383,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.869140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 511700.7633803555,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 522239.06682683004,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002068,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11398700.44180786,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11335633.104541667,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "466fa0268c689767413b158469cb93852522eeed",
          "message": "fix: address review feedback for Geneva exporter (#1995)\n\n### Description:\n\n  Follow-up from #1653 review comments from @utpilla:\n\n- Remove redundant `exports_failed` metric (already tracked per-signal\nin `pdata_metrics`)\n- Use `upload_batches_concurrent` return value for log count instead of\n`batches.len()`\n- Rename \"OTLP fallback\" → \"OTLP path\" (it's the direct path, not a\nfallback)\n   - Use array instead of `vec!` for fixed-size `TerminalState` metrics",
          "timestamp": "2026-02-09T12:50:20Z",
          "tree_id": "cc3c38fbcb5cfb0668ab314a9b03a4ec456e2235",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/466fa0268c689767413b158469cb93852522eeed"
        },
        "date": 1770649469106,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.1501171588897705,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.93882639706372,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.33371760291496,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.80286458333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.23828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 523855.83927518857,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 535119.3541931461,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002584,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11628142.575590603,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11568891.005478537,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "totan@microsoft.com",
            "name": "Tom Tan",
            "username": "ThomsonTan"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "c83cd5d367c602a7a151fb859f9b1b14c9438992",
          "message": "Improve command line help and error message (#1993)\n\n# Change Summary\n\nImprove the command line parsing error message and make it more verbose\nand clear for users.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #1992\n\n## How are these changes tested?\n\nLocal run tested.\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-02-09T15:38:04Z",
          "tree_id": "5245d65a7e342c672c8b46096a1e7af45db8468e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c83cd5d367c602a7a151fb859f9b1b14c9438992"
        },
        "date": 1770654514501,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.4073891639709473,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.29110749831929,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.6566738330362,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.84596354166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.62109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 521047.26357699314,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 533590.8991843787,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001743,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11594232.192033444,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11551356.239353405,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "f0dd51b8b00600fd485240729bbaa42aac1e43e1",
          "message": "chore: Replace duplicated OpenSSL/rcgen test cert helpers with shared dev crate (#2003)\n\n### Summary\n\nPulls the TLS/mTLS test certificate generation code into a small,\nunpublished workspace crate (`otap-test-tls-certs`) so all the exporter,\nreceiver, and tls_utils unit tests share one `rcgen`-based\nimplementation. Removes all `OpenSSL` CLI calls from test paths.\n\n### Motivation\n\nWe had the same cert-generation logic duplicated across multiple test\nfiles, some using `OpenSSL` CLI calls and some using inline `rcgen`.\nThis made tests flaky on systems without `OpenSSL` installed and meant\nbug fixes had to be applied in several places. This consolidates\neverything into one shared rcgen-only helper.\n\n### What changed\n\n- New internal crate `crates/otap-test-tls-certs` (publish = false) with\nCA, leaf, and self-signed cert helpers using rcgen.\n- All TLS integration tests and `tls_utils` unit tests now import from\n`otap_test_tls_certs` instead of per-file OpenSSL/rcgen copies.\n- Removed `skip_if_no_openssl()` guards - tests no longer depend on the\n`OpenSSL` CLI being installed.\n   - Added `#[must_use]` where clippy asked for it.\n\n### Alternatives considered\n\n1. **_Shared module in tests/common/_** - works for integration tests,\nbut unit tests in src/ can't import from tests/common/ without\ninclude!(), which is fragile and poorly supported by tooling.\n2. **_Feature-gated module in src/_** - avoids a new crate, but\n#[cfg(test)] doesn't apply when the crate is built as a dependency for\nintegration tests, so you end up\nneeding an extra Cargo feature and awkward wiring. Mixing test-only code\ninto the main crate felt wrong.\n3. **_Dedicated dev helper crate (this PR)_** - standard pattern in Rust\nworkspaces (publish = false, listed in [dev-dependencies]). Clean\nimports everywhere, no special tricks, no impact on production builds.\n\n**_Went with option 3 because it's the most straightforward to maintain\nand extend._**",
          "timestamp": "2026-02-09T17:25:03Z",
          "tree_id": "330969f2bbd431140197b4a504af0f7100a3ad67",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f0dd51b8b00600fd485240729bbaa42aac1e43e1"
        },
        "date": 1770661045822,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.834346890449524,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.79479923928335,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.46942679491157,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.91627604166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.6484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 516349.60028673586,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 525821.2432057977,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002262,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11506422.768883148,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11446049.626600873,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "66651184+utpilla@users.noreply.github.com",
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e1710212d7d3d96538834e01eb5b52b858cbcf46",
          "message": "Mark methods const (#2004)\n\n# Change Summary\n- Mark methods `const` when applicable\n\n## What issue does this PR close?\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-09T21:01:27Z",
          "tree_id": "33985437d97834b67b18c3f8ee5929ee499c8ed9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e1710212d7d3d96538834e01eb5b52b858cbcf46"
        },
        "date": 1770674004585,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.2014665603637695,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.2001413747564,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.77294070500928,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.19166666666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.9296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527148.0155366701,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 538753.0028479651,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001789,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11546083.169562846,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11497676.991758214,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "65bc572bdd387a7bef9cbcfc0e5ff0436b12fba3",
          "message": "feat: Add message parameter to otel_warn macro (#1977)\n\n# Change Summary\n\nAdd event name to missing otel_warn! calls.\n\n\n## What issue does this PR close?\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1972\n\n* Closes #NA\n\n## How are these changes tested?\n\nlocal builds and tests\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-10T01:04:00Z",
          "tree_id": "14d074a8e5184157d0ee7fa0996be431382aab2a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/65bc572bdd387a7bef9cbcfc0e5ff0436b12fba3"
        },
        "date": 1770694400037,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8696771860122681,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.94398115462664,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.30808340951205,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.238671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.3125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 529791.4342382598,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 534398.9098307461,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006829,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11662792.202375142,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11603612.963221895,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "5cdec9ce4d886ed0e53c07457fe3f5e29b6e66c6",
          "message": "InternalLogs - catch more scenarios of direct use of tracing (#2006)\n\nFollow up to\nhttps://github.com/open-telemetry/otel-arrow/pull/1987/changes#diff-01748cfa22e108f927f1500697086488ddb8d06bcd3e66db97f7b4cbc6927678",
          "timestamp": "2026-02-10T01:22:00Z",
          "tree_id": "33032773adb0341ba3a03c31a58dbbc7401f4aad",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5cdec9ce4d886ed0e53c07457fe3f5e29b6e66c6"
        },
        "date": 1770696531280,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.7830513715744019,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.9991344399275,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.60603925834363,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.24231770833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.15625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526414.14985483,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 535800.3841289711,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002764,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11571948.912135184,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11514653.203723392,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "1462125e597dfc76318a44ed765e9c71d195e27a",
          "message": "Minor improvement to OTLP exporter internal logs/events (#2005)\n\n# Change Summary\nApplied suggestion from\nhttps://github.com/open-telemetry/otel-arrow/pull/1987\n\n\n## Are there any user-facing changes?\n\nYes, less expensive logs, without losing information!",
          "timestamp": "2026-02-10T12:30:57Z",
          "tree_id": "7b1d14208a5d57af68d3313fd86e1388699be4fb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1462125e597dfc76318a44ed765e9c71d195e27a"
        },
        "date": 1770730685422,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.927687406539917,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.2253355808713,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.57137316831682,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.47994791666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.78515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526758.1362497471,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 536912.3865921497,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00246,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11593177.304184837,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11543103.521253437,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "2d5d736ad106c66e0f57dc2ebcbe3007a1f8a042",
          "message": "fix(deps): update golang.org/x/exp digest to 2842357 (#2007)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [golang.org/x/exp](https://pkg.go.dev/golang.org/x/exp) | require |\ndigest | `716be56` → `2842357` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - At any time (no schedule defined),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Ny4wIiwidXBkYXRlZEluVmVyIjoiNDIuOTcuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-10T13:23:43Z",
          "tree_id": "48b50944b0426910a9c0884999dfbc8306525ef5",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2d5d736ad106c66e0f57dc2ebcbe3007a1f8a042"
        },
        "date": 1770733157413,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.023432493209839,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.93508965625104,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.41121046335039,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.81953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.6171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 522916.51584003,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 533497.3793830221,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002664,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11630453.168693641,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11572998.67074697,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "84f5069142ac6ec6a74580df161bbd2113a93d44",
          "message": "fix(deps): update module github.com/klauspost/compress to v1.18.4 (#2009)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[github.com/klauspost/compress](https://redirect.github.com/klauspost/compress)\n| `v1.18.3` → `v1.18.4` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fklauspost%2fcompress/v1.18.4?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fklauspost%2fcompress/v1.18.3/v1.18.4?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>klauspost/compress (github.com/klauspost/compress)</summary>\n\n###\n[`v1.18.4`](https://redirect.github.com/klauspost/compress/releases/tag/v1.18.4)\n\n[Compare\nSource](https://redirect.github.com/klauspost/compress/compare/v1.18.3...v1.18.4)\n\n#### What's Changed\n\n- gzhttp: Add zstandard to server handler wrapper by\n[@&#8203;klauspost](https://redirect.github.com/klauspost) in\n[#&#8203;1121](https://redirect.github.com/klauspost/compress/pull/1121)\n- zstd: Add ResetWithOptions to encoder/decoder by\n[@&#8203;klauspost](https://redirect.github.com/klauspost) in\n[#&#8203;1122](https://redirect.github.com/klauspost/compress/pull/1122)\n- gzhttp: preserve qvalue when extra parameters follow in\nAccept-Encoding by\n[@&#8203;analytically](https://redirect.github.com/analytically) in\n[#&#8203;1116](https://redirect.github.com/klauspost/compress/pull/1116)\n\n#### New Contributors\n\n- [@&#8203;analytically](https://redirect.github.com/analytically) made\ntheir first contribution in\n[#&#8203;1116](https://redirect.github.com/klauspost/compress/pull/1116)\n- [@&#8203;ethaizone](https://redirect.github.com/ethaizone) made their\nfirst contribution in\n[#&#8203;1124](https://redirect.github.com/klauspost/compress/pull/1124)\n- [@&#8203;zwass](https://redirect.github.com/zwass) made their first\ncontribution in\n[#&#8203;1125](https://redirect.github.com/klauspost/compress/pull/1125)\n\n**Full Changelog**:\n<https://github.com/klauspost/compress/compare/v1.18.2...v1.18.4>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Ny4wIiwidXBkYXRlZEluVmVyIjoiNDIuOTcuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-10T14:07:39Z",
          "tree_id": "654e39be563757b78a49ce4e4ca35681dcc8111b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/84f5069142ac6ec6a74580df161bbd2113a93d44"
        },
        "date": 1770735436865,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.09291934967041,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.37382512787767,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.8373266625387,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.58216145833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 534059.2529377372,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 545236.681704168,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006645,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11636227.092990782,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11584608.91923787,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "8ae3f080c7df5bf627db50c83925e0a756adeadb",
          "message": "feat: Add event name to missing otel_error logs (#1978)\n\n# Change Summary\n\nAdd eventName to missing logs.\n\n## What issue does this PR close?\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1972\n\n* Closes #N/A\n\n## How are these changes tested?\n\nlocal builds and tests\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-10T14:46:08Z",
          "tree_id": "b131440f62ae2355e91a0282ee9ed5450493f20f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8ae3f080c7df5bf627db50c83925e0a756adeadb"
        },
        "date": 1770737887829,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8570920825004578,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.1624574979017,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.67388096537898,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.879947916666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.05078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527615.7785936925,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 532137.9313930627,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006818,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11576082.668165257,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11518856.84118842,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "1f22e633cf950a3e0bae41c7360d2c895d2eb143",
          "message": "fix: error applying transport optimized encoding to plain encoded String/Binary types (#2013)\n\n# Change Summary\n\nProperly track the current offsets when appending segments of the sorted\nvalues column when the type is a plain encoded String/Binary array.\n\nPrior to this fix, we'd have an error when we built the final value\ncolumn if more than one non-null segment of the array was appended. This\nwould happen because we appended offsets generated from slices of the\nvalues array, which were offset from the start of the slice, not from\nthe start of the array. This would cause a non-monotonically increasing\noffsets array for the resulting `StringArray`/`BinaryArray`, which is\ninvalid.\n\n## What issue does this PR close?\n\n* Closes #1974 \n\n## How are these changes tested?\n\nNew unit tests\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-02-10T20:26:32Z",
          "tree_id": "7a779cded6a4f033c46a08cb2ed4fc29eb2586a2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1f22e633cf950a3e0bae41c7360d2c895d2eb143"
        },
        "date": 1770758375738,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9116285443305969,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.91055875941214,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.53195116207003,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.12473958333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.7890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 528806.220551699,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 533626.9688068859,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007282,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11586100.600037752,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11529569.245845294,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "f3407598cbb0df2975be6688db50815da581c185",
          "message": "perf: avoid eagerly removing transport optimized encoding when transforming attributes (#1952)\n\n# Change Summary\n\nTo fix #966 we added a change to eargerly remove transport optimized\nencoding when transforming attributes, which led to the performance\nregression documented in #1853.\n\nRemoving this encoding is actually only necessary under somewhat rare\nconditions, at least for delete and renaming attributes. Basically, if\nthe operation would join sequences where type/key/value or two adjacent\nrows are equal after the transformation, but were not equal before. For\nexample if we had attribute columns like:\n```\nkey | str val\n--- | ------\n A  |  1    \n B  |  1        <-- if key \"B\" were renamed to \"A\"\n ...\n A  |  1    \n B  |  1        <--- if key \"B\" were deleted\n A  |  1    \n```\n\nThis PR removes the eager decoding behaviour, and instead adds code to\ncheck if we've made a transform that produces such a sequence. As part\nof the transformation process, we already compute a sequence of ranges\nof the attribute keys column that will be renamed or deleted. We use\nthese ranges to compute the neighbouring rows of each transformed range,\nand check that the post-transform type/key/value sequences are not\nequal. If we find any neighbours w/ equal values for these columns, we\nremove the transport optimized encoding.\n\nPerforming this check isn't free: For dictionary encoded keys, we\ncompute the transformed ranges only for the dictionary values, so the\nranges need to be mapped back to the equivalent ranges for the\ndictionary keys. Despite this it is faster to check than removing the\ntransport optimized encoding. Moreover, if we remove the encoding and\nthen have to re-apply it to use the OTAP exporter, performing this check\nbecomes even more worthwhile. Also, once we've computed the input to\nperform this check, we're able to use the resulting ranges both to\ncalculate statistics (for attribute transform metrics) and to speed up\nthe computation of which rows were deleted.\n\nBench results:\n\nnum_rows | dict_keys | operation | mode | before_us | after_us | percent\nchange\n-- | -- | -- | -- | -- | -- | --\n128 | FALSE | rename | decode=true | 1.8455 | 1.9344 | 4.60%\n128 | FALSE | rename | decode=false | 1.8486 | 0.6966 | -62.32%\n128 | FALSE | delete | decode=true | 2.8462 | 2.9733 | 4.27%\n128 | FALSE | delete | decode=false | 2.8477 | 1.6827 | -40.91%\n128 | FALSE | rename | no_encode | 0.84347 | 0.84786 | 0.52%\n128 | FALSE | rename | no_encode+stat | 0.84413 | 0.84534 | 0.14%\n128 | TRUE | rename | decode=true | 1.3191 | 1.3912 | 5.18%\n128 | TRUE | rename | decode=false | 1.3286 | 0.4037 | -69.61%\n128 | TRUE | delete | decode=true | 2.5701 | 2.5734 | 0.13%\n128 | TRUE | delete | decode=false | 2.5649 | 1.5479 | -39.65%\n128 | TRUE | rename | no_encode | 0.37642 | 0.29236 | -22.33%\n128 | TRUE | rename | no_encode+stat | 0.37804 | 0.32997 | -12.72%\n1536 | FALSE | rename | decode=true | 9.7934 | 9.7847 | -0.09%\n1536 | FALSE | rename | decode=false | 10.626 | 4.8693 | -54.18%\n1536 | FALSE | delete | decode=true | 10.438 | 10.608 | 1.60%\n1536 | FALSE | delete | decode=false | 10.443 | 5.438 | -47.93%\n1536 | FALSE | rename | no_encode | 6.8959 | 6.9046 | 0.13%\n1536 | FALSE | rename | no_encode+stat | 6.8789 | 6.8796 | 0.01%\n1536 | TRUE | rename | decode=true | 2.8206 | 2.7863 | -1.22%\n1536 | TRUE | rename | decode=false | 2.8938 | 0.7897 | -72.71%\n1536 | TRUE | delete | decode=true | 6.1078 | 4.9459 | -19.02%\n1536 | TRUE | delete | decode=false | 6.0088 | 2.8422 | -52.70%\n1536 | TRUE | rename | no_encode | 0.88044 | 0.2937 | -66.64%\n1536 | TRUE | rename | no_encode+stat | 0.8823 | 0.70662 | -19.91%\n8092 | FALSE | rename | decode=true | 47.095 | 46.716 | -0.80%\n8092 | FALSE | rename | decode=false | 48.285 | 23.947 | -50.40%\n8092 | FALSE | delete | decode=true | 45.416 | 45.583 | 0.37%\n8092 | FALSE | delete | decode=false | 45.38 | 22.297 | -50.87%\n8092 | FALSE | rename | no_encode | 34.7 | 34.575 | -0.36%\n8092 | FALSE | rename | no_encode+stat | 34.605 | 34.619 | 0.04%\n8092 | TRUE | rename | decode=true | 9.8332 | 9.0626 | -7.84%\n8092 | TRUE | rename | decode=false | 9.9166 | 2.3137 | -76.67%\n8092 | TRUE | delete | decode=true | 21.988 | 15.892 | -27.72%\n8092 | TRUE | delete | decode=false | 21.914 | 8.5716 | -60.89%\n8092 | TRUE | rename | no_encode | 3.202 | 0.2927 | -90.86%\n8092 | TRUE | rename | no_encode+stat | 3.1789 | 2.4248 | -23.72%\n\nExplanation of `mode` column from table:\n- `decode=true` = the transformation produced a result that required\nremoval of transport optimized encoding\n- `decode=false` = the transformation produced a result that **_did\nnot_** require removal of transport optmized encoding\n- `no_enocde` = the input record batch did not have transport optimized\nencoding\n- `no_encode+sta`t = the input record batch did not have transport\noptimized encoding, but the caller specified to track statistics of\ntransformations\n\nObservations:\n- performance is significantly improved for the case where\n`decode=false` e.g. we detected that we did not need to remove the\ndecoding. This is expected because in the old code, we'd always eagerly\nremove the decoding\n- performance is improved for most cases where keys are dictionary\nencoded. This is expected because the state we use to track when to\ntrack when remove the dictionary encoding helps us compute the deleted\nranges & statistics more efficiently\n- there are some cases where performance has slightly increased, notably\nfor small batch sizes (128 rows) where we did actually need to do the\ndecode operation. This is the effect of the overhead of having to check\nwhen to perform the decode. This can actually be further optimized --\nfrom profiling, I found that much of the extra time is spent in these\nconstructors, which in the future be optimized to make fewer passes over\nthe arrow schema:\n\nhttps://github.com/open-telemetry/otel-arrow/blob/f67c4d06672b324f8ce3aeb7f3eb0fb360891ca4/rust/otap-dataflow/crates/pdata/src/otap/transform.rs#L2203-L2205\n\n**_TL;DR_** - performance has been improved in the common case, where\nthe key column is dictionary encoded and the transformation produces a\nresult that does not need to have the transport optimized encoding\nremoved.\n\n## What issue does this PR close?\n\n* Closes #1853\n\n## How are these changes tested?\n\nExisting unit tests + many new ones\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-10T22:15:05Z",
          "tree_id": "1508d0757f0448529ac7d34e0410427491a4256d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f3407598cbb0df2975be6688db50815da581c185"
        },
        "date": 1770765981117,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.0313360691070557,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.17017643285719,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.54559697879586,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.553645833333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.41015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527181.9211861263,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 537890.7572725039,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002786,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11618393.910753975,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11569705.535099564,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "0d0af9a8664649f5c330cdcb2becf5bd611ca404",
          "message": "Add support for schema key aliases in query engine Parsers (#1725)\n\nDraft PR to open discussion - The current `otlp-bridge` for the\n`recordset` engine uses the OpenTelemetry [log data model\nspec](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/data-model.md)\nfor its initial schema keys (`Attributes`, `Timestamp`,\n`ObservedTimestamp`, `SeverityText`, etc).\n\nHowever, many well-versed in the OpenTelemetry space may be more used to\nthe snake case representation (`attributes`, `time_unix_nano`,\n`observed_time_unix_nano`, `severity_text`, etc) from the\n[proto](https://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/crates/pdata/src/views/otlp/proto/logs.rs)\nrepresentation.\n\nDo we have any significant risks if we plan to support both? Inspired by\n`severity_text` reference in #1722, been on the back of my mind for a\nwhile.\n\nThis is still somewhat incomplete, could need more wiring for\nuser-provided aliases in bridge, but for the moment just doing it for\nknown OpenTelemetry fields.",
          "timestamp": "2026-02-10T23:42:30Z",
          "tree_id": "23733c36cd3932f419a3794afd5e3a1e00b7ad7e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0d0af9a8664649f5c330cdcb2becf5bd611ca404"
        },
        "date": 1770772465621,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1855703592300415,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.0890406438148,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.69682696406443,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.408203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.70703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 521806.7447609042,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 527993.1309780624,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00272,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11536994.171041125,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11483133.082881093,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "70c62ad23a1d932f7e95bf93f57d4c86c82927c3",
          "message": "Emit warning and skip unconnected nodes during engine build (#2023)\n\n# Change Summary\n\nAdd a pre-processing step at the start of pipeline build that gracefully\nremoves unconnected nodes from the incoming `PipelineConfig`.\n\nInput with unconnected nodes:\n```yaml\nnodes:\n  unconnected_receiver:\n    kind: receiver\n    plugin_urn: \"urn:otel:syslog_cef:receiver\"\n    out_ports:\n    config:\n      listening_addr: \"127.0.0.1:5514\"\n      protocol: tcp\n\n  connected_receiver:\n    kind: receiver\n    plugin_urn: \"urn:otel:syslog_cef:receiver\"\n    out_ports:\n      out_port:\n        destinations:\n          - connected_proc\n        dispatch_strategy: round_robin\n    config:\n      listening_addr: \"127.0.0.1:5514\"\n      protocol: tcp\n\n  unconnected_proc:\n    kind: processor\n    plugin_urn: \"urn:otel:batch:processor\"\n    out_ports:\n      out_port:\n        destinations:\n          - connected_proc\n        dispatch_strategy: round_robin\n    config:\n      otap:\n        min_size: 1\n        sizer: items\n      flush_timeout: 5s\n\n  connected_proc:\n    kind: processor\n    plugin_urn: \"urn:otel:debug:processor\"\n    out_ports:\n      out_port:\n        destinations:\n          - noop_exporter\n        dispatch_strategy: round_robin\n    config:\n      verbosity: detailed\n      mode: signal\n\n  noop_exporter:\n    kind: exporter\n    plugin_urn: \"urn:otel:noop:exporter\"  \n```\n\nOutput (confirmed that log was able to pass through remaining connected\nnodes with debug processor):\n```log\n2026-02-11T19:01:57.699Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=unconnected_receiver, node_kind=receiver] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.706Z  INFO  otap-df-admin::endpoint.start: Admin HTTP server listening [bind_address=127.0.0.1:8080]\n2026-02-11T19:01:57.701Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=unconnected_proc, node_kind=processor] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.702Z  WARN  otap-df-engine::pipeline.build.unconnected_nodes: Some pipeline nodes were removed because they had no active incoming or outgoing edges. These nodes will not participate in data processing. Check pipeline configuration if this is unintentional. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, removed_count=2] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.725Z  INFO  otap-df-otap::receiver.start: Starting Syslog/CEF Receiver [protocol=Tcp, listening_addr=127.0.0.1:5514] entity/node.attrs: node.id=connected_receiver node.urn=urn:otel:syslog_cef:receiver node.type=receiver pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n\nReceived 1 resource logs\nReceived 1 log records\nReceived 0 events\nLogRecord #0:\n   -> ObservedTimestamp: 1770836524675426978\n   -> Timestamp: 1770836524000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Attributes:\n      -> syslog.facility: 16\n      -> syslog.severity: 6\n      -> syslog.host_name: securityhost\n      -> syslog.tag: myapp[1234]\n      -> syslog.app_name: myapp\n      -> syslog.process_id: 1234\n      -> syslog.content: User admin logged in from 10.0.0.1 successfully [test_id=234tg index=1]\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0\n```\n\nInput with no connected nodes:\n```yaml\nnodes:\n  recv:\n    kind: receiver\n    plugin_urn: \"urn:test:a:receiver\"\n    config: {}\n  proc:\n    kind: processor\n    plugin_urn: \"urn:test:b:processor\"\n    out_ports:\n      out:\n        destinations: [exp]\n        dispatch_strategy: round_robin\n    config: {}\n  exp:\n    kind: exporter\n    plugin_urn: \"urn:test:c:exporter\"\n    config: {}\n```\n\nOutput:\n```log\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=proc, node_kind=processor]\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=recv, node_kind=receiver]\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=exp, node_kind=exporter]\n2026-02-11T19:00:02.759Z  WARN  otap-df-engine::pipeline.build.unconnected_nodes: Some pipeline nodes were removed because they had no active incoming or outgoing edges. These nodes will not participate in data processing. Check pipeline configuration if this is unintentional. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, removed_count=3]\n2026-02-11T19:00:02.759Z  ERROR otap-df-state::state.observed_error: [observed_event=EngineEvent { key: DeployedPipelineKey { pipeline_group_id: \"default_pipeline_group\", pipeline_id: \"default_pipeline\", core_id: 0 }, node_id: None, node_kind: None, time: SystemTime { tv_sec: 1770836402, tv_nsec: 759880158 }, type: Error(RuntimeError(Pipeline { error_kind: \"EmptyPipeline\", message: \"Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\", source: None })), message: Some(\"Pipeline encountered a runtime error.\") }]\n2026-02-11T19:00:02.760Z  ERROR otap-df-state::state.report_failed: [error=InvalidTransition { phase: Starting, event: Error(RuntimeError(Pipeline { error_kind: \"EmptyPipeline\", message: \"Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\", source: None })), message: \"event not valid for current phase\" }]\n2026-02-11T19:00:02.760Z  INFO  otap-df-admin::endpoint.start: Admin HTTP server listening [bind_address=127.0.0.1:8080]\nPipeline failed to run: Pipeline runtime error: Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\n```\n\n\n## What issue does this PR close?\n\n* Closes #2012\n\n## How are these changes tested?\n\nUnit tests and local engine runs.\n\n## Are there any user-facing changes?\n\n1. Engine is now more flexible and does not crash with unconnected nodes\npresent in the config.\n2. Engine provides visible error if there are no nodes provided instead\nof starting up successfully.",
          "timestamp": "2026-02-11T23:33:54Z",
          "tree_id": "064919dc61bfd3f53c0804592d3587a88f34a226",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/70c62ad23a1d932f7e95bf93f57d4c86c82927c3"
        },
        "date": 1770855859039,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.303250551223755,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.78611772032757,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.12244023811365,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.219140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526081.9593464008,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 538198.9447203041,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001723,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11604729.668982176,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11553195.731325746,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "af32dd1fcec30131a150839000dea64d1b507b04",
          "message": "[query-engine] RecordSet engine diagnostic level adjustments (#2032)\n\n# Changes\n\n* Lowers some spammy diagnostics from \"Warn\" to \"Info\" in RecordSet\nengine\n\n# Details\n\n@drewrelmas has been doing some integration testing and noticed these\n\"warnings\" showing up for cases that didn't really need any attention:\n\n* When using `coalesce(thing1, thing2)` we don't really need to \"warn\"\nif \"thing1\" couldn't be found.\n* When using `project-away thing1, thing2` we don't really need to\n\"warn\" if \"thing1\" wasn't found (just a no-op in that case).",
          "timestamp": "2026-02-12T20:46:08Z",
          "tree_id": "4826f1e5783ee3ade28b614a7a7b3031d0d44be9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/af32dd1fcec30131a150839000dea64d1b507b04"
        },
        "date": 1770932466442,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.472113847732544,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.2625433960563,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.73679056782089,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.337239583333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.2578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 498774.6553467129,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 511104.9327876217,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001894,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11270311.707500298,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11209365.286914378,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "c44f94aa7ac0bd300118e10038d53283e2134112",
          "message": "feat: durable event names to quiver logging (#1988)\n\n# Change Summary\n\nAll log/trace calls in the quiver crate now use crate-private\n`otel_info!`, `otel_warn!`, `otel_error!`, and `otel_debug!` macros that\nenforce a required event name as the first argument. This ensures every\nlog event has a stable, machine-readable OpenTelemetry Event name\nfollowing the `quiver.<component>.<action>` convention.\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nMinimal unit test for the macros, ensured existing tests pass.\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-12T23:39:45Z",
          "tree_id": "394fe8d5dcfb36c7e5a58dbf8385d4d75f863f57",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c44f94aa7ac0bd300118e10038d53283e2134112"
        },
        "date": 1770942718538,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.113914728164673,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.16931024029942,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.54283776397516,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.808984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.91796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 504583.95051662734,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 515250.4244756645,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001084,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11347913.68287763,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11286979.473578962,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "75a2f71ba0765bd22358a4a2c772bf4eabc66c35",
          "message": "Reindex implementation (#2021)\n\n# Change Summary\n\nThis is a reimplementation of reindex as a part of the ongoing work of\n#1926. This now reindexes all the required columns, has support for\nmetrics, and has support for dictionary encoded columns.\n\nI also was able to uncomment most of the batching tests after this\nchange 🥳. One more to go which requires split.\n\nOther minor changes:\n\n- Made it so that `allowed_payload_types` returns payload types in the\nexact same order that they are stored. This is occasionally handy to\nhave.\n\nThings deferred:\n\n- Benchmarks. I had nothing to compare it to since the original didn't\nreindex a bunch of the necessary columns anyway like scope or resource\nid nor did it support dictionaries. I'll add these in when I get to the\nnext point..\n- Some optimization opportunities like using naive offsets instead of\nsorting and reindexing everything starting at 0. We need this path\nbecause it's possible to get into situations where we absolutely need to\ncompact things down to fit into u16, but we can likely skip it a decent\nportion of the time.\n\n## What issue does this PR close?\n\nPart of #1926.\n\n## How are these changes tested?\n\nI added a big unit test suite.\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-13T02:10:05Z",
          "tree_id": "dd903447a6807b332f0a66a10ec3e5667916cbeb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/75a2f71ba0765bd22358a4a2c772bf4eabc66c35"
        },
        "date": 1770953135637,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8530117273330688,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.30236107912282,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.79407559860519,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.309244791666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 505135.33182990836,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 509444.1954252954,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006541,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11245737.134913495,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11177396.95567696,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "cbc03d838832e2dedba932c899b95cdf95b07594",
          "message": "Dataflow Engine Pipeline configuration stabilization (#2031)\n\n# Change Summary\n\nSorry in advance, this is a fairly large PR, but it's for a good reason\nas it aims to stabilize our configuration model, which we discussed\nduring our SIG meetings.\n\n- Reworked node identity to use type: NodeUrn and removed the old\nkind/plugin_urn split.\n- Evolved NodeUrn from a type alias to a concrete parsed type\n(namespace, id, kind) with zero-cost part access and canonical URN\nreconstruction.\n- Moved URN normalization/parsing logic into the node_urn module and\ncleaned up obsolete URN plumbing.\n- Fully removed node-level out_ports wiring from NodeUserConfig.\n- Externalized graph wiring into top-level connections in\nPipelineConfig.\n- Simplified connection syntax:\n    - removed out_port field from connections\n    - default source output is implicit (default)\n    - multi-output selection stays explicit via from: node[\"output\"]\n- Standardized naming around output ports:\n    - config fields use outputs and default_output\n    - default output name is `default`\n    - outputs/default_output are optional for single-output nodes\n- Replaced connection fanout schema with policy-oriented schema:\n- policies.dispatch with one_of (default) and broadcast. I believe\n`one_of` better reflect the underlying implementation (was never really\na round robin strategy as the channel receivers were competing\ntogether).\n- broadcast is currently parsed but rejected for multi-destination edges\n(reserved for future support)\n    - single-destination edges treat dispatch as no-op\n- Refactored PipelineConfigBuilder API for readability in tests:\n- one_of(src, targets) and broadcast(src, targets) for default output\n- one_of_output(src, output, targets) and broadcast_output(...) for\nexplicit output\n    - added to(src, dst) and to_output(src, output, dst) aliases\n- Updated engine wiring internals and channel identity labeling to use\ndispatch policy terminology (one_of/broadcast) consistently.\n- Updated docs and examples to the new model:\n\n**To do: update the configuration of our continuous benchmarks.**\n  \n## What issue does this PR close?\n\n* Closes #1970 \n* Closes #1828\n* Closes #1829 \n\n## How are these changes tested?\n\nAll unit tests passed\n\n## Are there any user-facing changes?\n\nThe structure of the configuration files have changed.",
          "timestamp": "2026-02-13T15:01:21Z",
          "tree_id": "9142fd04755df35038dedb7e2e987134e92818dd",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbc03d838832e2dedba932c899b95cdf95b07594"
        },
        "date": 1770997851803,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0240871906280518,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.13436018912961,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.45814652963449,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.713671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.0546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 504107.78672368,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 509270.2900831571,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001898,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11363378.348464128,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11304379.3173831,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "cb40f580d296d43b12f319e1f8e43c0be4fd7199",
          "message": "otap_df_otap::pdata::Context remove source_node, use last frame's node_id (#2011)\n\n# Change Summary\n\nRemoves the `Option<Cow<str>>` field in `OtapPdata`.\n\nUses the `.stack.last().map(|frame| frame.node_id)` i.e., the last\nnode's frame.\n\nThis makes a simpler/smaller `OtapPdata` at the expense of always adding\na Frame.\n\nI think this makes sense because for us to implement the Collector's\nspecification for component-level telemetry (with producer/consumer\ncounts, with outcome attributes) requires maintaining a small amount of\nstate for every node that opts in. The current PR does not support an\n\"opt-in\" concept, so all nodes produce a frame and the current user of\nsource_node information (validation logic) continues to work. As a TODO\nfor the future, we can add this opt-in mechanism, for example to let\nusers disable this frame behavior when it is not useful (thus disabling\ncertain metrics).\n\nSee\nhttps://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md\n\nRenames `current_calldata()` which was confusingly named and used in two\nreal cases (retry behavior w/ delayed data) and some test cases. Now\nthat we have `source_node()`, the semantics are more clear if we name\nthis `source_calldata()`. Added comments to indicate that this changes\nafter a call to subscribe_to() otherwise is automatically maintained.\n\n## What issue does this PR close?\n\nPart of #2018 \nPart of #1950 \nHistorical connection with #487 \nFollows #1899 \n\n## How are these changes tested?\n\nNew tests.\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-02-16T04:30:36Z",
          "tree_id": "35b9136c4f147a429b4ccab257b6b4c712802530",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cb40f580d296d43b12f319e1f8e43c0be4fd7199"
        },
        "date": 1771219412447,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.552156448364258,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.25482698750483,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.65246920571208,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.824869791666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.87890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 498177.2827497582,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 510891.54658616567,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001901,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11268328.430527734,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11203366.79630846,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
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
          "id": "db05091d702a2f8b9482826be78354a36db47cb4",
          "message": "chore(deps): update azure-sdk-for-rust monorepo to 0.32.0 (#2043)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [azure_core](https://redirect.github.com/azure/azure-sdk-for-rust) |\nworkspace.dependencies | minor | `0.31.0` → `0.32.0` |\n| [azure_identity](https://redirect.github.com/azure/azure-sdk-for-rust)\n| workspace.dependencies | minor | `0.31.0` → `0.32.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>azure/azure-sdk-for-rust (azure_core)</summary>\n\n###\n[`v0.32.0`](https://redirect.github.com/Azure/azure-sdk-for-rust/releases/tag/azure_core%400.32.0)\n\n[Compare\nSource](https://redirect.github.com/azure/azure-sdk-for-rust/compare/azure_core@0.31.0...azure_core@0.32.0)\n\n#### 0.32.0 (2026-02-10)\n\n##### Features Added\n\n- Added `PagerContinuation` for `Pager` continuation.\n- Added `PollerContinuation` for `Poller` continuation.\n\n##### Breaking Changes\n\n- Changed our minimum supported Rust version (MSRV) from 1.85 to 1.88.\n- Changed paging APIs to use `PagerContinuation` and non-generic\n`PagerState`/`PagerResult` types.\n- Changed polling APIs to use `PollerContinuation` and non-generic\n`PollerState`/`PollerResult` types.\n- Renamed `PagerOptions::continuation_token` to `continuation`.\n- Renamed `Pager::continuation_token` to `continuation`.\n- Renamed `Pager::into_continuation_token` to `into_continuation`.\n- Renamed `PageIterator::continuation_token` to `continuation`.\n- Renamed `PageIterator::into_continuation_token` to\n`into_continuation`.\n- `Pager` callbacks must now return `Result`.\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about these\nupdates again.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My44LjUiLCJ1cGRhdGVkSW5WZXIiOiI0My44LjUiLCJ0YXJnZXRCcmFuY2giOiJtYWluIiwibGFiZWxzIjpbImRlcGVuZGVuY2llcyJdfQ==-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-16T13:05:42Z",
          "tree_id": "067b1b1fd1959f252cef28f28afaa1882e898eb3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/db05091d702a2f8b9482826be78354a36db47cb4"
        },
        "date": 1771253720452,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.065657138824463,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.86530208341732,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.2655299139735,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.4984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.7890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 499842.6295453833,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 510167.6646572858,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001733,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11317438.362074682,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11254963.788142048,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}