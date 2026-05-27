window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otap_none_transform_rename_multi_transform"] = {
  "name": "OTC OTAP Transform Rename Multi Transform (Logs)",
  "slug": "otc_logs_otap_none_transform_rename_multi_transform",
  "description": "OpenTelemetry Collector OTAP logs, transform processor (OTTL) rename sweep over 1-4 rename actions at 240k signals/sec",
  "meta": {
    "binary": "otc",
    "protocols": [
      "otap"
    ],
    "signals": [
      "logs"
    ],
    "compression": "none"
  },
  "env": {
    "started_at": "2026-05-27T23:35:45Z",
    "ended_at": "2026-05-27T23:39:57Z",
    "cpu": {
      "model": "INTEL(R) XEON(R) PLATINUM 8581C CPU @ 2.30GHz",
      "architecture": "x86_64",
      "physical_cores": 16,
      "logical_cores": 32,
      "max_freq_mhz": null
    },
    "os": {
      "system": "Linux",
      "release": "6.1.0-45-cloud-amd64",
      "version": "#1 SMP PREEMPT_DYNAMIC Debian 6.1.170-1 (2026-04-30)",
      "distro": {
        "NAME": "Debian GNU/Linux",
        "VERSION": "12 (bookworm)",
        "VERSION_ID": "12",
        "ID": "debian"
      }
    },
    "memory": {
      "total_bytes": 126619074560,
      "total_gib_rounded": 118
    }
  },
  "tests": [
    {
      "name": "transform-1",
      "metrics": [
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-1 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 1.1502516269683838
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.22259107344182
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.45432407118327
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 228.789453125
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 237.01953125
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 70685.98384345052
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 71679.2402480133
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-1 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000654
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9583171.714442577
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 10006681.162211895
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-1 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 133.6952188846364
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.042743,
            "value": 99.83990028046121
          },
          {
            "t": 3.055652,
            "value": 100.22868535825545
          },
          {
            "t": 5.073263,
            "value": 100.13898537192657
          },
          {
            "t": 7.085215,
            "value": 100.32399251403619
          },
          {
            "t": 9.130075,
            "value": 100.38588822978458
          },
          {
            "t": 11.047713,
            "value": 100.18880996884735
          },
          {
            "t": 13.060674,
            "value": 100.24059850374066
          },
          {
            "t": 15.077814,
            "value": 100.3264878353088
          },
          {
            "t": 17.090522,
            "value": 100.45432407118327
          },
          {
            "t": 19.113221,
            "value": 100.09823860087445
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.137551,
            "value": 155877.8785612669
          },
          {
            "t": 1.243969,
            "value": 62363.410573580695
          },
          {
            "t": 2.250235,
            "value": 90433.34466234571
          },
          {
            "t": 3.256782,
            "value": 72525.17766184788
          },
          {
            "t": 4.267905,
            "value": 72196.95328857123
          },
          {
            "t": 5.374818,
            "value": 65949.17577081487
          },
          {
            "t": 6.380775,
            "value": 70579.55757552262
          },
          {
            "t": 7.387387,
            "value": 72520.49449042928
          },
          {
            "t": 8.425543,
            "value": 42382.84034384043
          },
          {
            "t": 9.436648,
            "value": 89011.52699274558
          },
          {
            "t": 10.543716,
            "value": 70456.37666340279
          },
          {
            "t": 11.549988,
            "value": 79501.36742351967
          },
          {
            "t": 12.556844,
            "value": 72502.91998061292
          },
          {
            "t": 13.568057,
            "value": 73179.43895104197
          },
          {
            "t": 14.674443,
            "value": 63269.057996033924
          },
          {
            "t": 15.680463,
            "value": 71569.15369475757
          },
          {
            "t": 16.686434,
            "value": 50697.2865022948
          },
          {
            "t": 17.697866,
            "value": 93926.23527829848
          },
          {
            "t": 18.709014,
            "value": 70217.21844873352
          },
          {
            "t": 19.816134,
            "value": 63227.11178553364
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.030973,
            "value": 79379.42926190361
          },
          {
            "t": 1.042743,
            "value": 69185.68449351136
          },
          {
            "t": 2.049102,
            "value": 72538.72623984085
          },
          {
            "t": 3.055652,
            "value": 74511.94674879538
          },
          {
            "t": 4.061622,
            "value": 66602.38376889967
          },
          {
            "t": 5.073263,
            "value": 76113.95742165452
          },
          {
            "t": 6.079277,
            "value": 66599.47078271277
          },
          {
            "t": 7.085215,
            "value": 73563.18182631534
          },
          {
            "t": 8.124118,
            "value": 76041.74788214108
          },
          {
            "t": 9.130075,
            "value": 65609.16619696468
          },
          {
            "t": 10.141954,
            "value": 76096.05496309341
          },
          {
            "t": 11.148177,
            "value": 73542.34598096048
          },
          {
            "t": 12.154949,
            "value": 67542.60150262422
          },
          {
            "t": 13.161045,
            "value": 73551.62926798237
          },
          {
            "t": 14.172195,
            "value": 69228.10661128418
          },
          {
            "t": 15.178196,
            "value": 78528.74897738671
          },
          {
            "t": 16.184305,
            "value": 69574.96652947147
          },
          {
            "t": 17.190892,
            "value": 73515.7517432671
          },
          {
            "t": 18.201723,
            "value": 72217.80891167761
          },
          {
            "t": 19.213654,
            "value": 67198.25758870912
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.042743,
            "value": 10321194.735599319
          },
          {
            "t": 3.055652,
            "value": 9486474.053223468
          },
          {
            "t": 5.073263,
            "value": 10432133.845424118
          },
          {
            "t": 7.085215,
            "value": 9914582.952277191
          },
          {
            "t": 9.130075,
            "value": 9779175.102452002
          },
          {
            "t": 11.047713,
            "value": 10298023.923180496
          },
          {
            "t": 13.060674,
            "value": 10244541.747207224
          },
          {
            "t": 15.077814,
            "value": 9699626.699187959
          },
          {
            "t": 17.090522,
            "value": 10079398.005075749
          },
          {
            "t": 19.113221,
            "value": 9811660.558491403
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.042743,
            "value": 9353194.711887583
          },
          {
            "t": 3.055652,
            "value": 9461323.388190921
          },
          {
            "t": 5.073263,
            "value": 9644386.355942747
          },
          {
            "t": 7.085215,
            "value": 9927875.01888713
          },
          {
            "t": 9.130075,
            "value": 9188765.979089033
          },
          {
            "t": 11.047713,
            "value": 10066745.652724862
          },
          {
            "t": 13.060674,
            "value": 9665795.810251664
          },
          {
            "t": 15.077814,
            "value": 9573217.030052451
          },
          {
            "t": 17.090522,
            "value": 9335484.33255097
          },
          {
            "t": 19.113221,
            "value": 9614928.864848403
          }
        ],
        "ram_mib": [
          {
            "t": 1.042743,
            "value": 227.796875
          },
          {
            "t": 3.055652,
            "value": 234.5078125
          },
          {
            "t": 5.073263,
            "value": 237.01953125
          },
          {
            "t": 7.085215,
            "value": 227.42578125
          },
          {
            "t": 9.130075,
            "value": 229.1640625
          },
          {
            "t": 11.047713,
            "value": 221.98046875
          },
          {
            "t": 13.060674,
            "value": 230.58203125
          },
          {
            "t": 15.077814,
            "value": 228.046875
          },
          {
            "t": 17.090522,
            "value": 222.37109375
          },
          {
            "t": 19.113221,
            "value": 229.0
          }
        ]
      },
      "configFiles": [
        "backend-config.rendered.yaml",
        "loadgen-config.rendered.yaml",
        "otelcol-config.rendered.yaml"
      ]
    },
    {
      "name": "transform-2",
      "metrics": [
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-2 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 5.928571701049805
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.11204955442882
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.3763971276928
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 468.15
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 552.00390625
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 71333.5231484416
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 68860.88360382624
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-2 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000658
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9113590.120720385
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9898435.1490026
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-2 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 132.34785329147286
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.049132,
            "value": 100.3067248908297
          },
          {
            "t": 3.1181,
            "value": 100.18003114294612
          },
          {
            "t": 5.033254,
            "value": 100.09314624259433
          },
          {
            "t": 7.050635,
            "value": 99.79770573566084
          },
          {
            "t": 9.071465,
            "value": 100.15322118380064
          },
          {
            "t": 11.086658,
            "value": 99.9506962222916
          },
          {
            "t": 13.107513,
            "value": 100.20029925187033
          },
          {
            "t": 15.12232,
            "value": 100.3763971276928
          },
          {
            "t": 17.145344,
            "value": 100.13248598130842
          },
          {
            "t": 19.161083,
            "value": 99.92978776529338
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.141631,
            "value": 174827.0742936048
          },
          {
            "t": 1.198802,
            "value": 79457.34417610774
          },
          {
            "t": 2.210957,
            "value": 82003.25049029052
          },
          {
            "t": 3.319771,
            "value": 80265.94180809405
          },
          {
            "t": 4.327796,
            "value": 85315.34436149898
          },
          {
            "t": 5.335221,
            "value": 78417.74821947042
          },
          {
            "t": 6.345225,
            "value": 23762.282129575728
          },
          {
            "t": 7.357671,
            "value": 95807.57887334238
          },
          {
            "t": 8.466466,
            "value": 77561.67731636597
          },
          {
            "t": 9.473933,
            "value": 73451.53737045483
          },
          {
            "t": 10.481135,
            "value": 64535.21736454059
          },
          {
            "t": 11.493938,
            "value": 72077.1956639149
          },
          {
            "t": 12.602025,
            "value": 61367.02262547976
          },
          {
            "t": 13.610328,
            "value": 67440.0453038422
          },
          {
            "t": 14.617797,
            "value": 45658.973129694314
          },
          {
            "t": 15.630969,
            "value": 88829.93213393186
          },
          {
            "t": 16.74114,
            "value": 76564.78146159466
          },
          {
            "t": 17.748766,
            "value": 64508.06152282693
          },
          {
            "t": 18.756914,
            "value": 76377.6747064915
          },
          {
            "t": 19.767747,
            "value": 60346.268869338455
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.035566,
            "value": 65518.82971457615
          },
          {
            "t": 1.049132,
            "value": 74982.7835582488
          },
          {
            "t": 2.104775,
            "value": 54942.81684243631
          },
          {
            "t": 3.1181,
            "value": 69079.51545654159
          },
          {
            "t": 4.126459,
            "value": 72394.85143683945
          },
          {
            "t": 5.133847,
            "value": 63530.635663716464
          },
          {
            "t": 6.14184,
            "value": 74405.27860808557
          },
          {
            "t": 7.151171,
            "value": 73315.88943567571
          },
          {
            "t": 8.163887,
            "value": 69121.05664371848
          },
          {
            "t": 9.171975,
            "value": 63486.52101800636
          },
          {
            "t": 10.179284,
            "value": 71477.57043767106
          },
          {
            "t": 11.187194,
            "value": 69450.64539492613
          },
          {
            "t": 12.199677,
            "value": 73087.6469037011
          },
          {
            "t": 13.208059,
            "value": 67434.76182637137
          },
          {
            "t": 14.215569,
            "value": 63522.9427003206
          },
          {
            "t": 15.222979,
            "value": 79411.5603378962
          },
          {
            "t": 16.137844,
            "value": 74327.90630311576
          },
          {
            "t": 17.145344,
            "value": 68486.3523573201
          },
          {
            "t": 18.153671,
            "value": 63471.47304396292
          },
          {
            "t": 19.161083,
            "value": 63529.12214664904
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.049132,
            "value": 9687227.536109969
          },
          {
            "t": 3.1181,
            "value": 9856827.655140148
          },
          {
            "t": 5.033254,
            "value": 10636174.427748369
          },
          {
            "t": 7.050635,
            "value": 8980919.320643945
          },
          {
            "t": 9.071465,
            "value": 9962164.061301544
          },
          {
            "t": 11.086658,
            "value": 9825148.26123354
          },
          {
            "t": 13.107513,
            "value": 9087161.127344614
          },
          {
            "t": 15.12232,
            "value": 9733451.888940232
          },
          {
            "t": 17.145344,
            "value": 9952138.481797546
          },
          {
            "t": 19.161083,
            "value": 11263138.729766106
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.049132,
            "value": 9025134.716538588
          },
          {
            "t": 3.1181,
            "value": 9134032.039161554
          },
          {
            "t": 5.033254,
            "value": 9699504.060770048
          },
          {
            "t": 7.050635,
            "value": 8866096.191051666
          },
          {
            "t": 9.071465,
            "value": 9353496.830510234
          },
          {
            "t": 11.086658,
            "value": 9393568.258722614
          },
          {
            "t": 13.107513,
            "value": 9361723.132040646
          },
          {
            "t": 15.12232,
            "value": 9122475.750779107
          },
          {
            "t": 17.145344,
            "value": 8825335.240708958
          },
          {
            "t": 19.161083,
            "value": 8354534.98692043
          }
        ],
        "ram_mib": [
          {
            "t": 1.049132,
            "value": 396.48046875
          },
          {
            "t": 3.1181,
            "value": 420.953125
          },
          {
            "t": 5.033254,
            "value": 429.73828125
          },
          {
            "t": 7.050635,
            "value": 462.2421875
          },
          {
            "t": 9.071465,
            "value": 469.6484375
          },
          {
            "t": 11.086658,
            "value": 483.48828125
          },
          {
            "t": 13.107513,
            "value": 479.7265625
          },
          {
            "t": 15.12232,
            "value": 487.70703125
          },
          {
            "t": 17.145344,
            "value": 499.51171875
          },
          {
            "t": 19.161083,
            "value": 552.00390625
          }
        ]
      },
      "configFiles": [
        "backend-config.rendered.yaml",
        "loadgen-config.rendered.yaml",
        "otelcol-config.rendered.yaml"
      ]
    },
    {
      "name": "transform-3",
      "metrics": [
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-3 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 7.459207534790039
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 99.97066752435397
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.24411599625817
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 803.298046875
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 869.91796875
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 65668.56569749661
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 62047.06230603455
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-3 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000602
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 8342892.347574681
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9449923.109549481
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-3 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 134.4607147784863
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.07427,
            "value": 99.71473061351604
          },
          {
            "t": 3.09645,
            "value": 100.19391521197008
          },
          {
            "t": 5.113943,
            "value": 99.61169881767267
          },
          {
            "t": 7.036391,
            "value": 100.13527725856697
          },
          {
            "t": 9.054113,
            "value": 100.06615193026151
          },
          {
            "t": 11.078317,
            "value": 99.79501246882792
          },
          {
            "t": 13.096536,
            "value": 100.13278504672898
          },
          {
            "t": 15.114566,
            "value": 100.24411599625817
          },
          {
            "t": 17.138137,
            "value": 100.13856697819314
          },
          {
            "t": 19.156625,
            "value": 99.67442092154421
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.167186,
            "value": 239863.30752746336
          },
          {
            "t": 1.281861,
            "value": 75358.28829030883
          },
          {
            "t": 2.289484,
            "value": 60538.51490091036
          },
          {
            "t": 3.298242,
            "value": 77322.80685754164
          },
          {
            "t": 4.306711,
            "value": 60487.72941954587
          },
          {
            "t": 5.320689,
            "value": 75938.53121073633
          },
          {
            "t": 6.429557,
            "value": 62225.62108384407
          },
          {
            "t": 7.439426,
            "value": 68325.69372859252
          },
          {
            "t": 8.447938,
            "value": 69409.18898337353
          },
          {
            "t": 9.458114,
            "value": 68304.92904206792
          },
          {
            "t": 10.472042,
            "value": 62134.589438303316
          },
          {
            "t": 11.581909,
            "value": 62169.61131378805
          },
          {
            "t": 12.591392,
            "value": 57455.15278612915
          },
          {
            "t": 13.600173,
            "value": 74347.15760903507
          },
          {
            "t": 14.609503,
            "value": 56473.105921750066
          },
          {
            "t": 15.724883,
            "value": 60965.76951352903
          },
          {
            "t": 16.733222,
            "value": 65454.17761288613
          },
          {
            "t": 17.742173,
            "value": 65414.47503397093
          },
          {
            "t": 18.751152,
            "value": 62439.35701337689
          },
          {
            "t": 19.765604,
            "value": 63088.248630787864
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.062356,
            "value": 63402.93456519952
          },
          {
            "t": 1.07427,
            "value": 53364.21869842694
          },
          {
            "t": 2.08794,
            "value": 57217.83223336983
          },
          {
            "t": 3.09645,
            "value": 53544.33768628968
          },
          {
            "t": 4.105085,
            "value": 56512.018718366904
          },
          {
            "t": 5.113943,
            "value": 63438.06561478424
          },
          {
            "t": 6.127275,
            "value": 63157.97783944453
          },
          {
            "t": 7.137007,
            "value": 63383.15513423363
          },
          {
            "t": 8.145646,
            "value": 63451.83955805793
          },
          {
            "t": 9.154843,
            "value": 63416.756094201635
          },
          {
            "t": 10.164697,
            "value": 63375.49784424283
          },
          {
            "t": 11.178989,
            "value": 62112.29113509719
          },
          {
            "t": 12.188659,
            "value": 71310.42815969574
          },
          {
            "t": 13.197438,
            "value": 63443.03360795575
          },
          {
            "t": 14.206539,
            "value": 63422.78919553147
          },
          {
            "t": 15.215349,
            "value": 62449.81711124989
          },
          {
            "t": 16.229202,
            "value": 62139.18585830491
          },
          {
            "t": 17.23885,
            "value": 65369.316831212454
          },
          {
            "t": 18.24784,
            "value": 63429.766400063425
          },
          {
            "t": 19.257463,
            "value": 64380.46676828876
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.07427,
            "value": 11362257.839017954
          },
          {
            "t": 3.09645,
            "value": 9818437.033300694
          },
          {
            "t": 5.113943,
            "value": 10384503.936320968
          },
          {
            "t": 7.036391,
            "value": 9382617.891355189
          },
          {
            "t": 9.054113,
            "value": 8641387.16830168
          },
          {
            "t": 11.078317,
            "value": 9197039.428832272
          },
          {
            "t": 13.096536,
            "value": 8654284.297194704
          },
          {
            "t": 15.114566,
            "value": 9247567.677388344
          },
          {
            "t": 17.138137,
            "value": 8658877.301562436
          },
          {
            "t": 19.156625,
            "value": 9152258.522220593
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.07427,
            "value": 7808884.838752287
          },
          {
            "t": 3.09645,
            "value": 7284743.197934901
          },
          {
            "t": 5.113943,
            "value": 7553771.438116514
          },
          {
            "t": 7.036391,
            "value": 9022290.329829467
          },
          {
            "t": 9.054113,
            "value": 8341631.800614753
          },
          {
            "t": 11.078317,
            "value": 8818463.45526439
          },
          {
            "t": 13.096536,
            "value": 8330857.553119854
          },
          {
            "t": 15.114566,
            "value": 8858329.162599169
          },
          {
            "t": 17.138137,
            "value": 8564708.62648259
          },
          {
            "t": 19.156625,
            "value": 8845243.073032884
          }
        ],
        "ram_mib": [
          {
            "t": 1.07427,
            "value": 519.84375
          },
          {
            "t": 3.09645,
            "value": 649.55859375
          },
          {
            "t": 5.113943,
            "value": 821.328125
          },
          {
            "t": 7.036391,
            "value": 861.765625
          },
          {
            "t": 9.054113,
            "value": 862.06640625
          },
          {
            "t": 11.078317,
            "value": 862.8203125
          },
          {
            "t": 13.096536,
            "value": 861.515625
          },
          {
            "t": 15.114566,
            "value": 862.578125
          },
          {
            "t": 17.138137,
            "value": 861.5859375
          },
          {
            "t": 19.156625,
            "value": 869.91796875
          }
        ]
      },
      "configFiles": [
        "backend-config.rendered.yaml",
        "loadgen-config.rendered.yaml",
        "otelcol-config.rendered.yaml"
      ]
    },
    {
      "name": "transform-4",
      "metrics": [
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-4 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -1.2422360181808472
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.18407877051918
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.49750547045953
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 204.370703125
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 212.8671875
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 65615.64798712793
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 67804.77818815962
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-4 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000596
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9097576.663694132
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9483979.09309965
        },
        {
          "extra": "OTC OTAP Transform Rename Multi Transform (Logs)/transform-4 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 134.17309084690424
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.066927,
            "value": 100.18792147086319
          },
          {
            "t": 3.092958,
            "value": 100.33716781035558
          },
          {
            "t": 5.114159,
            "value": 100.14415446901278
          },
          {
            "t": 7.041951,
            "value": 100.29963817841548
          },
          {
            "t": 9.0637,
            "value": 100.21825436408977
          },
          {
            "t": 11.086635,
            "value": 100.49750547045953
          },
          {
            "t": 13.112685,
            "value": 99.76570894359614
          },
          {
            "t": 15.134391,
            "value": 100.3980768029972
          },
          {
            "t": 17.060085,
            "value": 100.12712328767124
          },
          {
            "t": 19.081659,
            "value": 99.86523690773068
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.15805,
            "value": 135849.5023773663
          },
          {
            "t": 1.173346,
            "value": 67960.47655068079
          },
          {
            "t": 2.284918,
            "value": 54877.2369221247
          },
          {
            "t": 3.294914,
            "value": 77228.02862585595
          },
          {
            "t": 4.306082,
            "value": 68237.91892148486
          },
          {
            "t": 5.31839,
            "value": 67173.23186223955
          },
          {
            "t": 6.333703,
            "value": 65989.50274447388
          },
          {
            "t": 7.445423,
            "value": 62065.98783866441
          },
          {
            "t": 8.455999,
            "value": 69267.4276848055
          },
          {
            "t": 9.467123,
            "value": 66262.89159390936
          },
          {
            "t": 10.479052,
            "value": 59292.697412565496
          },
          {
            "t": 11.495193,
            "value": 76761.00068789667
          },
          {
            "t": 12.606469,
            "value": 60291.052807763335
          },
          {
            "t": 13.616887,
            "value": 70267.94851239782
          },
          {
            "t": 14.628241,
            "value": 66247.82222644099
          },
          {
            "t": 15.638638,
            "value": 60372.30910226377
          },
          {
            "t": 16.654364,
            "value": 72854.29338227042
          },
          {
            "t": 17.765743,
            "value": 42289.8039282729
          },
          {
            "t": 18.777147,
            "value": 89973.93721994376
          },
          {
            "t": 19.787514,
            "value": 53445.92608428422
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.051734,
            "value": 66281.50844841913
          },
          {
            "t": 1.066927,
            "value": 65997.30297588735
          },
          {
            "t": 2.08292,
            "value": 69882.3712368097
          },
          {
            "t": 3.092958,
            "value": 70294.38496373403
          },
          {
            "t": 4.104123,
            "value": 66260.20481325994
          },
          {
            "t": 5.114159,
            "value": 68314.39671457255
          },
          {
            "t": 6.126628,
            "value": 69137.91928444228
          },
          {
            "t": 7.142741,
            "value": 65937.54828449198
          },
          {
            "t": 8.153226,
            "value": 67294.41802698704
          },
          {
            "t": 9.164462,
            "value": 68233.33030074087
          },
          {
            "t": 10.174811,
            "value": 69282.99033304334
          },
          {
            "t": 11.187377,
            "value": 68143.70618804108
          },
          {
            "t": 12.202983,
            "value": 66955.09872923161
          },
          {
            "t": 13.213453,
            "value": 67295.41698417568
          },
          {
            "t": 14.224689,
            "value": 66255.55261086432
          },
          {
            "t": 15.235118,
            "value": 68287.82625993513
          },
          {
            "t": 16.246046,
            "value": 70232.49924821549
          },
          {
            "t": 17.261637,
            "value": 65971.43929002916
          },
          {
            "t": 18.272985,
            "value": 68225.77391758327
          },
          {
            "t": 19.283417,
            "value": 66308.2721053965
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.066927,
            "value": 9075522.95545038
          },
          {
            "t": 3.092958,
            "value": 9643918.577751277
          },
          {
            "t": 5.114159,
            "value": 9366227.307427613
          },
          {
            "t": 7.041951,
            "value": 9749364.558002109
          },
          {
            "t": 9.0637,
            "value": 9451737.085068425
          },
          {
            "t": 11.086635,
            "value": 9534524.34210689
          },
          {
            "t": 13.112685,
            "value": 9206552.651711458
          },
          {
            "t": 15.134391,
            "value": 9503561.843314508
          },
          {
            "t": 17.060085,
            "value": 9878934.555542054
          },
          {
            "t": 19.081659,
            "value": 9429447.054621793
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.066927,
            "value": 8826677.551648961
          },
          {
            "t": 3.092958,
            "value": 9154121.037634665
          },
          {
            "t": 5.114159,
            "value": 9043919.92681579
          },
          {
            "t": 7.041951,
            "value": 9346323.151045341
          },
          {
            "t": 9.0637,
            "value": 9106153.137704039
          },
          {
            "t": 11.086635,
            "value": 9101213.83039989
          },
          {
            "t": 13.112685,
            "value": 8825450.507144444
          },
          {
            "t": 15.134391,
            "value": 9172086.34687734
          },
          {
            "t": 17.060085,
            "value": 9423185.61516004
          },
          {
            "t": 19.081659,
            "value": 8976635.532510806
          }
        ],
        "ram_mib": [
          {
            "t": 1.066927,
            "value": 198.42578125
          },
          {
            "t": 3.092958,
            "value": 199.296875
          },
          {
            "t": 5.114159,
            "value": 204.53125
          },
          {
            "t": 7.041951,
            "value": 201.6640625
          },
          {
            "t": 9.0637,
            "value": 205.08984375
          },
          {
            "t": 11.086635,
            "value": 201.1015625
          },
          {
            "t": 13.112685,
            "value": 206.64453125
          },
          {
            "t": 15.134391,
            "value": 212.8671875
          },
          {
            "t": 17.060085,
            "value": 210.21875
          },
          {
            "t": 19.081659,
            "value": 203.8671875
          }
        ]
      },
      "configFiles": [
        "backend-config.rendered.yaml",
        "loadgen-config.rendered.yaml",
        "otelcol-config.rendered.yaml"
      ]
    }
  ]
};
