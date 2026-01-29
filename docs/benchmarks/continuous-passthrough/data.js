window.BENCHMARK_DATA = {
  "lastUpdate": 1769722574518,
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
      }
    ]
  }
}