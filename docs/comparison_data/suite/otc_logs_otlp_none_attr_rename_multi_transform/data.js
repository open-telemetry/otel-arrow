window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otlp_none_attr_rename_multi_transform"] = {
  "name": "OTC OTLP Attr Rename Multi Transform (Logs)",
  "slug": "otc_logs_otlp_none_attr_rename_multi_transform",
  "description": "OpenTelemetry Collector OTLP logs, attributes processor rename sweep over 1-4 rename actions at 400k signals/sec",
  "meta": {
    "binary": "otc",
    "protocols": [
      "otlp"
    ],
    "signals": [
      "logs"
    ],
    "compression": "none"
  },
  "env": {
    "started_at": "2026-05-27T18:45:23Z",
    "ended_at": "2026-05-27T18:49:27Z",
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
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-1 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 5.8801774978637695
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 99.88457664878014
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.19340604549704
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 641.800390625
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 684.2734375
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 275925.5048077009
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 265204.53000599076
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-1 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000677
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 228262055.8410264
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 230839888.84729257
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-1 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 860.7019489292666
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.061856,
            "value": 99.69572713178295
          },
          {
            "t": 3.080041,
            "value": 99.84399126909884
          },
          {
            "t": 5.098349,
            "value": 99.87777570674122
          },
          {
            "t": 7.116815,
            "value": 99.66192343604108
          },
          {
            "t": 9.134178,
            "value": 99.94265464718683
          },
          {
            "t": 11.083241,
            "value": 100.06020541549954
          },
          {
            "t": 13.094403,
            "value": 100.19340604549704
          },
          {
            "t": 15.117036,
            "value": 99.84387577639751
          },
          {
            "t": 17.129787,
            "value": 100.03146235220908
          },
          {
            "t": 19.145016,
            "value": 99.69474470734745
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.153687,
            "value": 382496.33358436724
          },
          {
            "t": 1.163925,
            "value": 300919.1893395418
          },
          {
            "t": 2.272224,
            "value": 212036.64354113827
          },
          {
            "t": 3.281397,
            "value": 315109.50055144157
          },
          {
            "t": 4.290536,
            "value": 247735.94123307097
          },
          {
            "t": 5.299732,
            "value": 555888.0534603784
          },
          {
            "t": 6.311233,
            "value": 253089.22087076533
          },
          {
            "t": 7.424063,
            "value": 243523.26950208028
          },
          {
            "t": 8.430008,
            "value": 272380.69675777503
          },
          {
            "t": 9.43586,
            "value": 267434.9705523278
          },
          {
            "t": 10.444982,
            "value": 261613.56109568515
          },
          {
            "t": 11.485102,
            "value": 258624.00492250896
          },
          {
            "t": 12.490678,
            "value": 265519.4634716819
          },
          {
            "t": 13.501335,
            "value": 285952.6031086709
          },
          {
            "t": 14.613519,
            "value": 252656.03533228312
          },
          {
            "t": 15.61958,
            "value": 276325.19300519547
          },
          {
            "t": 16.625495,
            "value": 258471.14318804274
          },
          {
            "t": 17.631996,
            "value": 257327.11641617838
          },
          {
            "t": 18.641421,
            "value": 260544.36931916684
          },
          {
            "t": 19.75318,
            "value": 215874.12379841317
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.052742,
            "value": 269536.70406158123
          },
          {
            "t": 1.061856,
            "value": 256660.79352778773
          },
          {
            "t": 2.071057,
            "value": 277447.2082370113
          },
          {
            "t": 3.080041,
            "value": 268587.0142638535
          },
          {
            "t": 4.089228,
            "value": 269523.8840769847
          },
          {
            "t": 5.098349,
            "value": 255668.0517004403
          },
          {
            "t": 6.107636,
            "value": 267515.5827826971
          },
          {
            "t": 7.116815,
            "value": 266553.3071932729
          },
          {
            "t": 8.128344,
            "value": 271865.66079667513
          },
          {
            "t": 9.134178,
            "value": 269428.15613709623
          },
          {
            "t": 10.143469,
            "value": 260578.9608745149
          },
          {
            "t": 11.183657,
            "value": 265336.6506823767
          },
          {
            "t": 12.189177,
            "value": 259567.18911607924
          },
          {
            "t": 13.194724,
            "value": 269505.0554573779
          },
          {
            "t": 14.20616,
            "value": 272879.3517335748
          },
          {
            "t": 15.217557,
            "value": 257070.1712581706
          },
          {
            "t": 16.223688,
            "value": 262391.27906803385
          },
          {
            "t": 17.230165,
            "value": 260313.94656807854
          },
          {
            "t": 18.236139,
            "value": 265414.4142890373
          },
          {
            "t": 19.245476,
            "value": 262548.5838723836
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.061856,
            "value": 249031313.71537662
          },
          {
            "t": 3.080041,
            "value": 215681837.88899434
          },
          {
            "t": 5.098349,
            "value": 219079023.61780262
          },
          {
            "t": 7.116815,
            "value": 240778777.54690936
          },
          {
            "t": 9.134178,
            "value": 234207650.28405893
          },
          {
            "t": 11.083241,
            "value": 232125012.37774253
          },
          {
            "t": 13.094403,
            "value": 229112414.11681405
          },
          {
            "t": 15.117036,
            "value": 221831316.9022754
          },
          {
            "t": 17.129787,
            "value": 232859757.61532348
          },
          {
            "t": 19.145016,
            "value": 233691784.4076281
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.061856,
            "value": 230932249.3265215
          },
          {
            "t": 3.080041,
            "value": 227956872.14006644
          },
          {
            "t": 5.098349,
            "value": 226261070.6591858
          },
          {
            "t": 7.116815,
            "value": 229219066.85572112
          },
          {
            "t": 9.134178,
            "value": 228912015.8345325
          },
          {
            "t": 11.083241,
            "value": 234296640.48827565
          },
          {
            "t": 13.094403,
            "value": 229601894.82498175
          },
          {
            "t": 15.117036,
            "value": 224930671.55534396
          },
          {
            "t": 17.129787,
            "value": 225177727.8958003
          },
          {
            "t": 19.145016,
            "value": 225332348.8298352
          }
        ],
        "ram_mib": [
          {
            "t": 1.061856,
            "value": 603.671875
          },
          {
            "t": 3.080041,
            "value": 618.765625
          },
          {
            "t": 5.098349,
            "value": 607.20703125
          },
          {
            "t": 7.116815,
            "value": 608.0
          },
          {
            "t": 9.134178,
            "value": 649.05078125
          },
          {
            "t": 11.083241,
            "value": 662.25
          },
          {
            "t": 13.094403,
            "value": 656.48046875
          },
          {
            "t": 15.117036,
            "value": 655.25390625
          },
          {
            "t": 17.129787,
            "value": 673.05078125
          },
          {
            "t": 19.145016,
            "value": 684.2734375
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
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-2 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 7.369323253631592
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 99.97891132986028
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.26626902764835
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 643.265625
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 758.6171875
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 298758.7763270612
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 276742.2769576291
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-2 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000678
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 230828503.43650436
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 238759906.36432475
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-2 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 834.091942778391
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.065578,
            "value": 100.05024587612823
          },
          {
            "t": 3.086188,
            "value": 99.93248439450687
          },
          {
            "t": 5.100349,
            "value": 99.68328767123288
          },
          {
            "t": 7.117789,
            "value": 100.08229140722291
          },
          {
            "t": 9.13323,
            "value": 100.26626902764835
          },
          {
            "t": 11.047376,
            "value": 99.71582684521955
          },
          {
            "t": 13.070588,
            "value": 100.18233717669057
          },
          {
            "t": 15.08484,
            "value": 100.11226409218312
          },
          {
            "t": 17.160803,
            "value": 99.74230220291653
          },
          {
            "t": 19.083563,
            "value": 100.02180460485377
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.159094,
            "value": 298078.28926908225
          },
          {
            "t": 1.171177,
            "value": 296418.3767536852
          },
          {
            "t": 2.180071,
            "value": 299337.69057998166
          },
          {
            "t": 3.28716,
            "value": 270980.92384623096
          },
          {
            "t": 4.294542,
            "value": 297801.62837930396
          },
          {
            "t": 5.301212,
            "value": 598011.2648633614
          },
          {
            "t": 6.311501,
            "value": 297934.5514006388
          },
          {
            "t": 7.320209,
            "value": 285513.74629724363
          },
          {
            "t": 8.4274,
            "value": 195991.47753188023
          },
          {
            "t": 9.434578,
            "value": 300840.56641427835
          },
          {
            "t": 10.442414,
            "value": 305605.2770490437
          },
          {
            "t": 11.454245,
            "value": 291550.6640931144
          },
          {
            "t": 12.464351,
            "value": 282148.60618588544
          },
          {
            "t": 13.572846,
            "value": 207488.53174800065
          },
          {
            "t": 14.579895,
            "value": 320739.1100135147
          },
          {
            "t": 15.586944,
            "value": 308823.1059263253
          },
          {
            "t": 16.599274,
            "value": 282516.5706834728
          },
          {
            "t": 17.667257,
            "value": 251876.66844884236
          },
          {
            "t": 18.678665,
            "value": 315401.89517978893
          },
          {
            "t": 19.689901,
            "value": 292711.098101729
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.159094,
            "value": 261315.30025922874
          },
          {
            "t": 1.171177,
            "value": 265788.47782247106
          },
          {
            "t": 2.281539,
            "value": 247666.9770759446
          },
          {
            "t": 3.28716,
            "value": 257552.29852996304
          },
          {
            "t": 4.294542,
            "value": 271992.1539197643
          },
          {
            "t": 5.301212,
            "value": 271191.1549961755
          },
          {
            "t": 6.311501,
            "value": 269229.8936244975
          },
          {
            "t": 7.421595,
            "value": 242321.82139530528
          },
          {
            "t": 8.4274,
            "value": 277389.75248681405
          },
          {
            "t": 9.434578,
            "value": 265097.1327808987
          },
          {
            "t": 10.442414,
            "value": 544731.4840906656
          },
          {
            "t": 11.454245,
            "value": 269807.90270311944
          },
          {
            "t": 12.566014,
            "value": 244655.1396917885
          },
          {
            "t": 13.572846,
            "value": 258235.73346894022
          },
          {
            "t": 14.579895,
            "value": 270096.09264295973
          },
          {
            "t": 15.586944,
            "value": 269103.09230236063
          },
          {
            "t": 16.599274,
            "value": 265723.6276708188
          },
          {
            "t": 17.667257,
            "value": 253749.35743359214
          },
          {
            "t": 18.678665,
            "value": 259044.81673073574
          },
          {
            "t": 19.689901,
            "value": 266011.0992883956
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.065578,
            "value": 248657122.02690166
          },
          {
            "t": 3.086188,
            "value": 249609248.19732654
          },
          {
            "t": 5.100349,
            "value": 243422788.94288987
          },
          {
            "t": 7.117789,
            "value": 248464578.37655643
          },
          {
            "t": 9.13323,
            "value": 237159180.05042073
          },
          {
            "t": 11.047376,
            "value": 260990353.92284602
          },
          {
            "t": 13.070588,
            "value": 225342723.84703135
          },
          {
            "t": 15.08484,
            "value": 235580494.8933897
          },
          {
            "t": 17.160803,
            "value": 217941566.3959329
          },
          {
            "t": 19.083563,
            "value": 220431006.98995194
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.065578,
            "value": 228108213.52870247
          },
          {
            "t": 3.086188,
            "value": 226461885.27226925
          },
          {
            "t": 5.100349,
            "value": 232692321.02101073
          },
          {
            "t": 7.117789,
            "value": 232317775.0019827
          },
          {
            "t": 9.13323,
            "value": 229997779.14610252
          },
          {
            "t": 11.047376,
            "value": 245746983.77239773
          },
          {
            "t": 13.070588,
            "value": 225733944.8362307
          },
          {
            "t": 15.08484,
            "value": 227596398.0673719
          },
          {
            "t": 17.160803,
            "value": 221227176.9776244
          },
          {
            "t": 19.083563,
            "value": 238402556.74135098
          }
        ],
        "ram_mib": [
          {
            "t": 1.065578,
            "value": 409.16796875
          },
          {
            "t": 3.086188,
            "value": 503.1484375
          },
          {
            "t": 5.100349,
            "value": 559.33203125
          },
          {
            "t": 7.117789,
            "value": 602.24609375
          },
          {
            "t": 9.13323,
            "value": 652.703125
          },
          {
            "t": 11.047376,
            "value": 690.5859375
          },
          {
            "t": 13.070588,
            "value": 743.22265625
          },
          {
            "t": 15.08484,
            "value": 755.296875
          },
          {
            "t": 17.160803,
            "value": 758.3359375
          },
          {
            "t": 19.083563,
            "value": 758.6171875
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
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-3 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 4.692526340484619
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.27311409432187
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.50140668959048
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 648.065625
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 697.6796875
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 271121.33990679885
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 262456.1824010546
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-3 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000705
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 226120489.17750683
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 229091297.12162971
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-3 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 861.5552017440235
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.101939,
            "value": 99.99746189735615
          },
          {
            "t": 3.019949,
            "value": 100.39649144634527
          },
          {
            "t": 5.044353,
            "value": 99.97074626865671
          },
          {
            "t": 7.062522,
            "value": 100.50140668959048
          },
          {
            "t": 9.08097,
            "value": 100.1561121495327
          },
          {
            "t": 11.103593,
            "value": 100.25110071718117
          },
          {
            "t": 13.121068,
            "value": 100.22084788029926
          },
          {
            "t": 15.14349,
            "value": 100.4585869903517
          },
          {
            "t": 17.066201,
            "value": 100.43721337082161
          },
          {
            "t": 19.08413,
            "value": 100.34117353308365
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.19501,
            "value": 394482.00943981577
          },
          {
            "t": 1.203132,
            "value": 299566.91749609675
          },
          {
            "t": 2.213084,
            "value": 305955.1345014417
          },
          {
            "t": 3.224334,
            "value": 225463.53522867739
          },
          {
            "t": 4.338669,
            "value": 322165.2375632103
          },
          {
            "t": 5.347619,
            "value": 367709.0044105258
          },
          {
            "t": 6.356088,
            "value": 260791.35798918956
          },
          {
            "t": 7.365422,
            "value": 281373.6582736735
          },
          {
            "t": 8.374942,
            "value": 266463.26967271575
          },
          {
            "t": 9.388389,
            "value": 259510.36413349689
          },
          {
            "t": 10.49723,
            "value": 236282.74928506432
          },
          {
            "t": 11.506494,
            "value": 263558.3950284564
          },
          {
            "t": 12.515302,
            "value": 236913.26793601955
          },
          {
            "t": 13.523733,
            "value": 274684.1380322501
          },
          {
            "t": 14.537826,
            "value": 253428.43309242843
          },
          {
            "t": 15.652617,
            "value": 206316.69972219007
          },
          {
            "t": 16.660899,
            "value": 297535.8084345451
          },
          {
            "t": 17.67098,
            "value": 241564.7853983987
          },
          {
            "t": 18.6797,
            "value": 293441.1927987945
          },
          {
            "t": 19.688127,
            "value": 263777.14995730977
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.093722,
            "value": 236923.1318264097
          },
          {
            "t": 1.101939,
            "value": 255897.29195203018
          },
          {
            "t": 2.110849,
            "value": 262659.7020546927
          },
          {
            "t": 3.12053,
            "value": 264439.95677842805
          },
          {
            "t": 4.131382,
            "value": 251273.18341359563
          },
          {
            "t": 5.145922,
            "value": 266130.46306700574
          },
          {
            "t": 6.154471,
            "value": 263745.2419267681
          },
          {
            "t": 7.163331,
            "value": 270602.4621850405
          },
          {
            "t": 8.173125,
            "value": 273323.0738150553
          },
          {
            "t": 9.181541,
            "value": 254855.13914892267
          },
          {
            "t": 10.195029,
            "value": 259499.86580995534
          },
          {
            "t": 11.204198,
            "value": 273492.34865518066
          },
          {
            "t": 12.212912,
            "value": 258745.2935123335
          },
          {
            "t": 13.22161,
            "value": 238921.85768188295
          },
          {
            "t": 14.230389,
            "value": 273598.0824343092
          },
          {
            "t": 15.24412,
            "value": 252532.47656429568
          },
          {
            "t": 16.258133,
            "value": 257393.14979196517
          },
          {
            "t": 17.26791,
            "value": 264414.8163406376
          },
          {
            "t": 18.27699,
            "value": 265588.4568121457
          },
          {
            "t": 19.285497,
            "value": 279621.26192480564
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.101939,
            "value": 223317492.0165415
          },
          {
            "t": 3.019949,
            "value": 247522203.2210468
          },
          {
            "t": 5.044353,
            "value": 227160461.05421644
          },
          {
            "t": 7.062522,
            "value": 229620665.5636867
          },
          {
            "t": 9.08097,
            "value": 218141427.47298917
          },
          {
            "t": 11.103593,
            "value": 222754265.62438974
          },
          {
            "t": 13.121068,
            "value": 215879270.87076667
          },
          {
            "t": 15.14349,
            "value": 224417084.56494242
          },
          {
            "t": 17.066201,
            "value": 251300514.22184613
          },
          {
            "t": 19.08413,
            "value": 230799586.60587165
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.101939,
            "value": 211162140.68239543
          },
          {
            "t": 3.019949,
            "value": 235800801.35140067
          },
          {
            "t": 5.044353,
            "value": 223498890.53765947
          },
          {
            "t": 7.062522,
            "value": 231333719.3267759
          },
          {
            "t": 9.08097,
            "value": 223253201.46964404
          },
          {
            "t": 11.103593,
            "value": 226594144.83074704
          },
          {
            "t": 13.121068,
            "value": 217440994.31219715
          },
          {
            "t": 15.14349,
            "value": 219032107.04788613
          },
          {
            "t": 17.066201,
            "value": 237503698.68378556
          },
          {
            "t": 19.08413,
            "value": 235585193.53257722
          }
        ],
        "ram_mib": [
          {
            "t": 1.101939,
            "value": 697.6796875
          },
          {
            "t": 3.019949,
            "value": 623.3671875
          },
          {
            "t": 5.044353,
            "value": 685.234375
          },
          {
            "t": 7.062522,
            "value": 651.171875
          },
          {
            "t": 9.08097,
            "value": 614.609375
          },
          {
            "t": 11.103593,
            "value": 622.3671875
          },
          {
            "t": 13.121068,
            "value": 676.65234375
          },
          {
            "t": 15.14349,
            "value": 656.03125
          },
          {
            "t": 17.066201,
            "value": 625.0390625
          },
          {
            "t": 19.08413,
            "value": 628.50390625
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
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-4 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 7.584064483642578
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.29949212403959
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.49430447014691
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 678.822265625
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 763.2578125
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 280352.3278695882
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 263158.64792552456
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-4 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.00066
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 227764150.584308
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 230747251.112992
        },
        {
          "extra": "OTC OTLP Attr Rename Multi Transform (Logs)/transform-4 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 865.501295054406
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.072515,
            "value": 100.49430447014691
          },
          {
            "t": 3.095938,
            "value": 100.49080337605503
          },
          {
            "t": 5.120881,
            "value": 100.33068664169788
          },
          {
            "t": 7.040289,
            "value": 100.23623323978796
          },
          {
            "t": 9.064792,
            "value": 100.39515302935665
          },
          {
            "t": 11.089778,
            "value": 99.89171918876755
          },
          {
            "t": 13.110035,
            "value": 100.24536005009392
          },
          {
            "t": 15.134202,
            "value": 100.36730565095225
          },
          {
            "t": 17.160004,
            "value": 100.21216957605985
          },
          {
            "t": 19.077607,
            "value": 100.33118601747815
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.163908,
            "value": 297308.17181241047
          },
          {
            "t": 1.173908,
            "value": 299009.900990099
          },
          {
            "t": 2.188065,
            "value": 299756.34936208103
          },
          {
            "t": 3.303646,
            "value": 274296.5324794883
          },
          {
            "t": 4.313553,
            "value": 299037.43612035556
          },
          {
            "t": 5.323086,
            "value": 563626.9443396105
          },
          {
            "t": 6.332746,
            "value": 269397.6190004556
          },
          {
            "t": 7.343421,
            "value": 252306.62675934398
          },
          {
            "t": 8.358221,
            "value": 279858.1001182499
          },
          {
            "t": 9.475007,
            "value": 216693.26083958792
          },
          {
            "t": 10.483778,
            "value": 283513.3048035679
          },
          {
            "t": 11.493419,
            "value": 247612.7653294587
          },
          {
            "t": 12.503907,
            "value": 294907.0152243272
          },
          {
            "t": 13.513471,
            "value": 253574.81051226077
          },
          {
            "t": 14.527843,
            "value": 278004.51905218203
          },
          {
            "t": 15.644974,
            "value": 230053.59264043337
          },
          {
            "t": 16.654126,
            "value": 214041.09589041094
          },
          {
            "t": 17.664332,
            "value": 297959.03013840737
          },
          {
            "t": 18.675147,
            "value": 213688.9539628913
          },
          {
            "t": 19.682205,
            "value": 272079.6617473869
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.062496,
            "value": 271514.82480852754
          },
          {
            "t": 1.072515,
            "value": 271282.0253876412
          },
          {
            "t": 2.081543,
            "value": 269566.35494753363
          },
          {
            "t": 3.095938,
            "value": 275040.78785877296
          },
          {
            "t": 4.111445,
            "value": 260953.39569298885
          },
          {
            "t": 5.120881,
            "value": 269457.3999738468
          },
          {
            "t": 6.130656,
            "value": 271347.5774306157
          },
          {
            "t": 7.141124,
            "value": 266213.27939133154
          },
          {
            "t": 8.151055,
            "value": 265364.66352651815
          },
          {
            "t": 9.165651,
            "value": 257245.24835501026
          },
          {
            "t": 10.181226,
            "value": 266843.90616153413
          },
          {
            "t": 11.190665,
            "value": 261531.40506756725
          },
          {
            "t": 12.200263,
            "value": 272385.6426023031
          },
          {
            "t": 13.21079,
            "value": 260260.24044879555
          },
          {
            "t": 14.220173,
            "value": 251638.87245971055
          },
          {
            "t": 15.234954,
            "value": 251285.7453972828
          },
          {
            "t": 16.250663,
            "value": 255978.82858180837
          },
          {
            "t": 17.260912,
            "value": 254392.72892128574
          },
          {
            "t": 18.270989,
            "value": 258396.14207629717
          },
          {
            "t": 19.279041,
            "value": 260899.23932495547
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.072515,
            "value": 251780412.96135244
          },
          {
            "t": 3.095938,
            "value": 246581900.57145736
          },
          {
            "t": 5.120881,
            "value": 233676936.09153444
          },
          {
            "t": 7.040289,
            "value": 252531146.58269632
          },
          {
            "t": 9.064792,
            "value": 221304403.10535473
          },
          {
            "t": 11.089778,
            "value": 220228887.0145275
          },
          {
            "t": 13.110035,
            "value": 214492477.4422264
          },
          {
            "t": 15.134202,
            "value": 210751198.88823405
          },
          {
            "t": 17.160004,
            "value": 214969490.60174686
          },
          {
            "t": 19.077607,
            "value": 241155657.87078974
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.072515,
            "value": 232094470.90193403
          },
          {
            "t": 3.095938,
            "value": 229057490.69769394
          },
          {
            "t": 5.120881,
            "value": 229307884.22192624
          },
          {
            "t": 7.040289,
            "value": 240541992.6352292
          },
          {
            "t": 9.064792,
            "value": 227276378.94337523
          },
          {
            "t": 11.089778,
            "value": 225911203.83054498
          },
          {
            "t": 13.110035,
            "value": 221800890.67876017
          },
          {
            "t": 15.134202,
            "value": 216304985.70523086
          },
          {
            "t": 17.160004,
            "value": 220345881.77916697
          },
          {
            "t": 19.077607,
            "value": 235000326.44921812
          }
        ],
        "ram_mib": [
          {
            "t": 1.072515,
            "value": 489.48046875
          },
          {
            "t": 3.095938,
            "value": 628.37109375
          },
          {
            "t": 5.120881,
            "value": 639.25
          },
          {
            "t": 7.040289,
            "value": 654.5546875
          },
          {
            "t": 9.064792,
            "value": 756.76953125
          },
          {
            "t": 11.089778,
            "value": 717.5859375
          },
          {
            "t": 13.110035,
            "value": 763.2578125
          },
          {
            "t": 15.134202,
            "value": 726.125
          },
          {
            "t": 17.160004,
            "value": 700.265625
          },
          {
            "t": 19.077607,
            "value": 712.5625
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
