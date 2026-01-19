window.BENCHMARK_DATA = {
  "lastUpdate": 1768845690324,
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
      }
    ]
  }
}