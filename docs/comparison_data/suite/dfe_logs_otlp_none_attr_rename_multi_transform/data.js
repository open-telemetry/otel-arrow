window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otlp_none_attr_rename_multi_transform"] = {
  "name": "DFE OTLP Attr Rename Multi Transform (Logs)",
  "slug": "dfe_logs_otlp_none_attr_rename_multi_transform",
  "description": "Dataflow Engine OTLP logs, attributes processor rename sweep over 1-4 rename actions at 400k signals/sec",
  "meta": {
    "binary": "dfe",
    "protocols": [
      "otlp"
    ],
    "signals": [
      "logs"
    ],
    "compression": "none"
  },
  "env": {
    "started_at": "2026-05-27T18:20:30Z",
    "ended_at": "2026-05-27T18:26:16Z",
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
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-1 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 83.98110921858233
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 84.50750000000001
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 65.8484375
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 77.1328125
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 390507.5025226271
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 398729.60550532263
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-1 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000588
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 371905781.1604625
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 344026443.3047768
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-1 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 932.7267803180417
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.058421,
            "value": 83.65775695095282
          },
          {
            "t": 2.069427,
            "value": 83.7351689612015
          },
          {
            "t": 4.119954,
            "value": 84.30809255784865
          },
          {
            "t": 6.13731,
            "value": 84.50750000000001
          },
          {
            "t": 8.148622,
            "value": 84.05093408309902
          },
          {
            "t": 10.059608,
            "value": 84.08052483598875
          },
          {
            "t": 12.074078,
            "value": 83.66061663033322
          },
          {
            "t": 14.08478,
            "value": 83.79545824210197
          },
          {
            "t": 16.096308,
            "value": 83.93038701622972
          },
          {
            "t": 18.113367,
            "value": 84.08465290806755
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.259481,
            "value": 795719.0316099386
          },
          {
            "t": 1.265264,
            "value": 397700.10031985026
          },
          {
            "t": 2.272237,
            "value": 397230.1144122037
          },
          {
            "t": 3.384422,
            "value": 359652.39595930534
          },
          {
            "t": 4.325026,
            "value": 425258.6635821238
          },
          {
            "t": 5.33267,
            "value": 396965.59499188204
          },
          {
            "t": 6.439292,
            "value": 361460.37219574524
          },
          {
            "t": 7.444725,
            "value": 397838.5431948225
          },
          {
            "t": 8.45003,
            "value": 397889.1978056411
          },
          {
            "t": 9.455953,
            "value": 397644.75014489185
          },
          {
            "t": 10.461472,
            "value": 398799.0281635652
          },
          {
            "t": 11.469508,
            "value": 395819.1969334429
          },
          {
            "t": 12.576144,
            "value": 361455.79937757313
          },
          {
            "t": 13.581395,
            "value": 397910.57158858835
          },
          {
            "t": 14.58748,
            "value": 397580.7213108237
          },
          {
            "t": 15.592728,
            "value": 397911.7590883046
          },
          {
            "t": 16.602042,
            "value": 396308.780022867
          },
          {
            "t": 17.609906,
            "value": 397871.14134446706
          },
          {
            "t": 18.715793,
            "value": 362604.8592668148
          },
          {
            "t": 19.721335,
            "value": 395806.4407056095
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.158766,
            "value": 397843.29151668807
          },
          {
            "t": 1.164394,
            "value": 396766.99535016925
          },
          {
            "t": 2.16977,
            "value": 397861.09873321024
          },
          {
            "t": 3.177147,
            "value": 398063.48566624016
          },
          {
            "t": 4.119954,
            "value": 423204.3249572818
          },
          {
            "t": 5.129226,
            "value": 397316.08525749255
          },
          {
            "t": 6.13731,
            "value": 395800.3499708358
          },
          {
            "t": 7.143177,
            "value": 398661.0555868718
          },
          {
            "t": 8.148622,
            "value": 396839.21049883385
          },
          {
            "t": 9.154425,
            "value": 398686.42268913495
          },
          {
            "t": 10.159978,
            "value": 396796.58854381624
          },
          {
            "t": 11.165839,
            "value": 397669.2604644181
          },
          {
            "t": 12.174416,
            "value": 396598.3757313522
          },
          {
            "t": 13.179791,
            "value": 397861.4944672386
          },
          {
            "t": 14.18522,
            "value": 397840.12595618394
          },
          {
            "t": 15.191124,
            "value": 397652.2610507564
          },
          {
            "t": 16.196657,
            "value": 398793.4756989577
          },
          {
            "t": 17.206352,
            "value": 396159.2362049926
          },
          {
            "t": 18.213712,
            "value": 396084.81575603556
          },
          {
            "t": 19.219302,
            "value": 398770.8708320489
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.058421,
            "value": 343523640.5090792
          },
          {
            "t": 2.069427,
            "value": 342976434.6799562
          },
          {
            "t": 4.119954,
            "value": 336783879.94891065
          },
          {
            "t": 6.13731,
            "value": 341507063.2055027
          },
          {
            "t": 8.148622,
            "value": 343363419.00212395
          },
          {
            "t": 10.059608,
            "value": 361211930.9089653
          },
          {
            "t": 12.074078,
            "value": 342176217.5659106
          },
          {
            "t": 14.08478,
            "value": 342629138.4799935
          },
          {
            "t": 16.096308,
            "value": 343724826.10234606
          },
          {
            "t": 18.113367,
            "value": 342367882.6449796
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.058421,
            "value": 371428696.4843136
          },
          {
            "t": 2.069427,
            "value": 370883476.2303046
          },
          {
            "t": 4.119954,
            "value": 363276012.459236
          },
          {
            "t": 6.13731,
            "value": 370633561.4537048
          },
          {
            "t": 8.148622,
            "value": 370319122.04570943
          },
          {
            "t": 10.059608,
            "value": 390824607.2969661
          },
          {
            "t": 12.074078,
            "value": 370432116.139729
          },
          {
            "t": 14.08478,
            "value": 370729979.8776745
          },
          {
            "t": 16.096308,
            "value": 371218873.9107782
          },
          {
            "t": 18.113367,
            "value": 369311365.7062089
          }
        ],
        "ram_mib": [
          {
            "t": 0.058421,
            "value": 60.12109375
          },
          {
            "t": 2.069427,
            "value": 63.6875
          },
          {
            "t": 4.119954,
            "value": 61.1953125
          },
          {
            "t": 6.13731,
            "value": 63.17578125
          },
          {
            "t": 8.148622,
            "value": 64.2109375
          },
          {
            "t": 10.059608,
            "value": 66.2734375
          },
          {
            "t": 12.074078,
            "value": 62.6328125
          },
          {
            "t": 14.08478,
            "value": 63.234375
          },
          {
            "t": 16.096308,
            "value": 77.1328125
          },
          {
            "t": 18.113367,
            "value": 76.8203125
          }
        ]
      },
      "configFiles": [
        "backend-config.rendered.yaml",
        "engine-config.rendered.yaml",
        "loadgen-config.rendered.yaml"
      ]
    },
    {
      "name": "transform-2",
      "metrics": [
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-2 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -0.013157893903553486
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 97.54895879752411
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 97.88282777950795
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 60.05234375
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 60.91015625
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 388042.57031229255
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 398384.2194355418
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-2 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000608
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 371353376.3301112
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 343452318.9541049
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-2 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 932.1488106538715
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.075431,
            "value": 97.60228926905133
          },
          {
            "t": 2.095608,
            "value": 97.2620895522388
          },
          {
            "t": 4.109949,
            "value": 97.63783447417548
          },
          {
            "t": 6.123765,
            "value": 97.61793833696669
          },
          {
            "t": 8.140837,
            "value": 97.88282777950795
          },
          {
            "t": 10.155566,
            "value": 97.24881693648817
          },
          {
            "t": 12.070749,
            "value": 97.73459526774595
          },
          {
            "t": 14.088907,
            "value": 97.5397506234414
          },
          {
            "t": 16.125703,
            "value": 97.33698630136986
          },
          {
            "t": 18.144613,
            "value": 97.62645943425552
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.277187,
            "value": 794392.384160213
          },
          {
            "t": 1.290032,
            "value": 394927.1606218128
          },
          {
            "t": 2.397466,
            "value": 361195.33985772514
          },
          {
            "t": 3.404278,
            "value": 397293.6357532489
          },
          {
            "t": 4.41181,
            "value": 397009.7227681106
          },
          {
            "t": 5.41868,
            "value": 397270.7499478582
          },
          {
            "t": 6.428235,
            "value": 396214.1735715241
          },
          {
            "t": 7.536184,
            "value": 361027.44801430387
          },
          {
            "t": 8.54311,
            "value": 397248.6558098609
          },
          {
            "t": 9.651329,
            "value": 360939.4893969513
          },
          {
            "t": 10.658107,
            "value": 397307.0527961477
          },
          {
            "t": 11.665905,
            "value": 396904.93531441817
          },
          {
            "t": 12.674254,
            "value": 396688.0514583741
          },
          {
            "t": 13.684825,
            "value": 396805.370429193
          },
          {
            "t": 14.697661,
            "value": 393943.34324609314
          },
          {
            "t": 15.721741,
            "value": 390594.4848058745
          },
          {
            "t": 16.830856,
            "value": 360647.9039594632
          },
          {
            "t": 17.841423,
            "value": 395817.39755998366
          },
          {
            "t": 18.852019,
            "value": 395806.0392085462
          },
          {
            "t": 19.862667,
            "value": 395785.6741417388
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.176016,
            "value": 397243.9216714435
          },
          {
            "t": 1.183233,
            "value": 398126.7194656166
          },
          {
            "t": 2.196091,
            "value": 393934.78651499026
          },
          {
            "t": 3.202889,
            "value": 397299.1603082247
          },
          {
            "t": 4.210431,
            "value": 397005.78238922055
          },
          {
            "t": 5.217263,
            "value": 397285.74379836954
          },
          {
            "t": 6.224285,
            "value": 397210.78586167924
          },
          {
            "t": 7.234546,
            "value": 395937.287493034
          },
          {
            "t": 8.241347,
            "value": 397297.97646208137
          },
          {
            "t": 9.248102,
            "value": 397316.129544924
          },
          {
            "t": 10.256028,
            "value": 396854.53098739387
          },
          {
            "t": 11.263218,
            "value": 397144.5308233799
          },
          {
            "t": 12.271735,
            "value": 396621.97067575454
          },
          {
            "t": 13.279322,
            "value": 396988.0516521154
          },
          {
            "t": 14.290096,
            "value": 396725.67755007546
          },
          {
            "t": 15.208295,
            "value": 434546.32383611833
          },
          {
            "t": 16.226245,
            "value": 392946.6083795865
          },
          {
            "t": 17.23455,
            "value": 396705.3619688487
          },
          {
            "t": 18.245147,
            "value": 395805.64755288215
          },
          {
            "t": 19.255587,
            "value": 396856.8148529354
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.075431,
            "value": 342272797.0463291
          },
          {
            "t": 2.095608,
            "value": 341692106.6817413
          },
          {
            "t": 4.109949,
            "value": 341824413.5426921
          },
          {
            "t": 6.123765,
            "value": 343166094.12180656
          },
          {
            "t": 8.140837,
            "value": 341669377.6920209
          },
          {
            "t": 10.155566,
            "value": 342352375.4311374
          },
          {
            "t": 12.070749,
            "value": 360408511.3537453
          },
          {
            "t": 14.088907,
            "value": 341184705.0627354
          },
          {
            "t": 16.125703,
            "value": 338904236.3594587
          },
          {
            "t": 18.144613,
            "value": 341048572.2493821
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.075431,
            "value": 370290107.2180159
          },
          {
            "t": 2.095608,
            "value": 369041027.0981206
          },
          {
            "t": 4.109949,
            "value": 369970237.4126327
          },
          {
            "t": 6.123765,
            "value": 370056601.9934294
          },
          {
            "t": 8.140837,
            "value": 369929512.18399733
          },
          {
            "t": 10.155566,
            "value": 369736819.6913828
          },
          {
            "t": 12.070749,
            "value": 389774694.6375359
          },
          {
            "t": 14.088907,
            "value": 369272987.5460692
          },
          {
            "t": 16.125703,
            "value": 365882018.62140346
          },
          {
            "t": 18.144613,
            "value": 369579756.89852446
          }
        ],
        "ram_mib": [
          {
            "t": 0.075431,
            "value": 60.91015625
          },
          {
            "t": 2.095608,
            "value": 60.10546875
          },
          {
            "t": 4.109949,
            "value": 59.80859375
          },
          {
            "t": 6.123765,
            "value": 58.9921875
          },
          {
            "t": 8.140837,
            "value": 59.44921875
          },
          {
            "t": 10.155566,
            "value": 60.11328125
          },
          {
            "t": 12.070749,
            "value": 60.29296875
          },
          {
            "t": 14.088907,
            "value": 60.46875
          },
          {
            "t": 16.125703,
            "value": 60.12890625
          },
          {
            "t": 18.144613,
            "value": 60.25390625
          }
        ]
      },
      "configFiles": [
        "backend-config.rendered.yaml",
        "engine-config.rendered.yaml",
        "loadgen-config.rendered.yaml"
      ]
    },
    {
      "name": "transform-3",
      "metrics": [
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-3 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 97.21896778574181
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 97.75795319812792
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 58.999609375
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 60.40625
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 391499.35392303986
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 397789.3589148181
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-3 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000575
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 370731608.36232454
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 342943543.6402109
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-3 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 931.9797024578336
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.039272,
            "value": 97.08573204849237
          },
          {
            "t": 2.057482,
            "value": 97.32309797822705
          },
          {
            "t": 4.075818,
            "value": 96.89368650669991
          },
          {
            "t": 6.108614,
            "value": 97.1305269722482
          },
          {
            "t": 8.132735,
            "value": 97.62250855897915
          },
          {
            "t": 10.050045,
            "value": 97.1157671957672
          },
          {
            "t": 12.074088,
            "value": 97.09490976975731
          },
          {
            "t": 14.093151,
            "value": 97.08349127182045
          },
          {
            "t": 16.111964,
            "value": 97.08200435729847
          },
          {
            "t": 18.129802,
            "value": 97.75795319812792
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.34195,
            "value": 630729.5017417144
          },
          {
            "t": 1.350971,
            "value": 396423.860355731
          },
          {
            "t": 2.359929,
            "value": 396448.61332186277
          },
          {
            "t": 3.369467,
            "value": 396220.8455749065
          },
          {
            "t": 4.383261,
            "value": 395543.86788637534
          },
          {
            "t": 5.401348,
            "value": 391911.49675813556
          },
          {
            "t": 6.416268,
            "value": 395105.03290899773
          },
          {
            "t": 7.526054,
            "value": 359528.77401589137
          },
          {
            "t": 8.53579,
            "value": 396143.15028878837
          },
          {
            "t": 9.544739,
            "value": 396452.14971222525
          },
          {
            "t": 10.55373,
            "value": 396435.6470969513
          },
          {
            "t": 11.568644,
            "value": 394122.06354430027
          },
          {
            "t": 12.578334,
            "value": 396161.1979914627
          },
          {
            "t": 13.688319,
            "value": 360365.2301607679
          },
          {
            "t": 14.697418,
            "value": 396393.2181084314
          },
          {
            "t": 15.706413,
            "value": 396434.0754909588
          },
          {
            "t": 16.716146,
            "value": 396144.32726275164
          },
          {
            "t": 17.725095,
            "value": 396452.14971222525
          },
          {
            "t": 18.739345,
            "value": 394380.08380576776
          },
          {
            "t": 19.754498,
            "value": 394029.27440494194
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.140222,
            "value": 394330.70741943084
          },
          {
            "t": 1.149043,
            "value": 396502.4518720368
          },
          {
            "t": 2.15813,
            "value": 396397.9319919888
          },
          {
            "t": 3.167695,
            "value": 396210.2489686152
          },
          {
            "t": 4.176464,
            "value": 395531.5835439035
          },
          {
            "t": 5.190649,
            "value": 395391.3733687641
          },
          {
            "t": 6.2093,
            "value": 391694.5057728309
          },
          {
            "t": 7.223641,
            "value": 394344.7026197305
          },
          {
            "t": 8.233383,
            "value": 397131.14835274755
          },
          {
            "t": 9.242412,
            "value": 396420.7173431091
          },
          {
            "t": 10.251309,
            "value": 396472.58342526545
          },
          {
            "t": 11.165646,
            "value": 437475.4603608954
          },
          {
            "t": 12.174735,
            "value": 396397.14633694355
          },
          {
            "t": 13.184541,
            "value": 396115.68954828946
          },
          {
            "t": 14.193792,
            "value": 396333.5186192533
          },
          {
            "t": 15.202952,
            "value": 396369.2576003805
          },
          {
            "t": 16.212613,
            "value": 395182.1452943117
          },
          {
            "t": 17.221652,
            "value": 396416.788647416
          },
          {
            "t": 18.230738,
            "value": 397389.320632731
          },
          {
            "t": 19.245811,
            "value": 394060.32866601716
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.039272,
            "value": 340704077.6188556
          },
          {
            "t": 2.057482,
            "value": 342471409.8136468
          },
          {
            "t": 4.075818,
            "value": 341172299.85493
          },
          {
            "t": 6.108614,
            "value": 339590719.8754819
          },
          {
            "t": 8.132735,
            "value": 340196905.718581
          },
          {
            "t": 10.050045,
            "value": 360035595.18283427
          },
          {
            "t": 12.074088,
            "value": 340619488.815208
          },
          {
            "t": 14.093151,
            "value": 341464556.5789676
          },
          {
            "t": 16.111964,
            "value": 341949171.1218424
          },
          {
            "t": 18.129802,
            "value": 341231211.8217617
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.039272,
            "value": 368439064.019545
          },
          {
            "t": 2.057482,
            "value": 369282537.00060946
          },
          {
            "t": 4.075818,
            "value": 369770506.00098294
          },
          {
            "t": 6.108614,
            "value": 366705529.2316593
          },
          {
            "t": 8.132735,
            "value": 368266529.0266738
          },
          {
            "t": 10.050045,
            "value": 388766780.01992375
          },
          {
            "t": 12.074088,
            "value": 368730229.5455186
          },
          {
            "t": 14.093151,
            "value": 368725408.7663436
          },
          {
            "t": 16.111964,
            "value": 369683030.57291585
          },
          {
            "t": 18.129802,
            "value": 368946469.43907297
          }
        ],
        "ram_mib": [
          {
            "t": 0.039272,
            "value": 58.578125
          },
          {
            "t": 2.057482,
            "value": 60.40625
          },
          {
            "t": 4.075818,
            "value": 58.875
          },
          {
            "t": 6.108614,
            "value": 59.81640625
          },
          {
            "t": 8.132735,
            "value": 58.21875
          },
          {
            "t": 10.050045,
            "value": 59.38671875
          },
          {
            "t": 12.074088,
            "value": 58.81640625
          },
          {
            "t": 14.093151,
            "value": 58.37890625
          },
          {
            "t": 16.111964,
            "value": 58.84765625
          },
          {
            "t": 18.129802,
            "value": 58.671875
          }
        ]
      },
      "configFiles": [
        "backend-config.rendered.yaml",
        "engine-config.rendered.yaml",
        "loadgen-config.rendered.yaml"
      ]
    },
    {
      "name": "transform-4",
      "metrics": [
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-4 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -5.263157844543457
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 97.55815268847337
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 98.07651741293533
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 60.433203125
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 62.98046875
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 390781.6862630828
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 409207.6425460757
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-4 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000665
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 372044351.7812897
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 344235017.6351231
        },
        {
          "extra": "DFE OTLP Attr Rename Multi Transform (Logs)/transform-4 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 909.1823150380152
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.120889,
            "value": 98.07651741293533
          },
          {
            "t": 2.041949,
            "value": 97.42490189971971
          },
          {
            "t": 4.072917,
            "value": 97.90744316412334
          },
          {
            "t": 6.102364,
            "value": 97.37673723536737
          },
          {
            "t": 8.124014,
            "value": 97.80379943942698
          },
          {
            "t": 10.14539,
            "value": 97.12388059701492
          },
          {
            "t": 12.071239,
            "value": 97.7194401244168
          },
          {
            "t": 14.098525,
            "value": 97.37165628891657
          },
          {
            "t": 16.118999,
            "value": 97.59174129353234
          },
          {
            "t": 18.14298,
            "value": 97.1854094292804
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.322418,
            "value": 791665.3472244212
          },
          {
            "t": 1.333657,
            "value": 395554.36449741357
          },
          {
            "t": 2.344213,
            "value": 395821.7060707175
          },
          {
            "t": 3.36381,
            "value": 392311.86439348094
          },
          {
            "t": 4.378139,
            "value": 394349.3679072569
          },
          {
            "t": 5.393941,
            "value": 393777.52751028247
          },
          {
            "t": 6.505361,
            "value": 359899.9478145075
          },
          {
            "t": 7.516649,
            "value": 395535.19867733034
          },
          {
            "t": 8.527295,
            "value": 395786.457374788
          },
          {
            "t": 9.537614,
            "value": 395914.55767930724
          },
          {
            "t": 10.552442,
            "value": 394155.46279763663
          },
          {
            "t": 11.564501,
            "value": 395233.8747049332
          },
          {
            "t": 12.580132,
            "value": 393843.8271380059
          },
          {
            "t": 13.692272,
            "value": 359666.9484057763
          },
          {
            "t": 14.702945,
            "value": 395775.88399017294
          },
          {
            "t": 15.713332,
            "value": 395887.9122554032
          },
          {
            "t": 16.723463,
            "value": 395988.24310906214
          },
          {
            "t": 17.7361,
            "value": 395008.28036107705
          },
          {
            "t": 18.754842,
            "value": 392641.12012658746
          },
          {
            "t": 19.770617,
            "value": 393787.99438852107
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.322418,
            "value": 394843.0919281801
          },
          {
            "t": 1.333657,
            "value": 396543.2504086571
          },
          {
            "t": 2.344213,
            "value": 394832.1518055407
          },
          {
            "t": 3.36381,
            "value": 393292.64405446465
          },
          {
            "t": 4.480673,
            "value": 358145.98567595135
          },
          {
            "t": 5.495787,
            "value": 394044.4127457605
          },
          {
            "t": 6.505361,
            "value": 792413.433784943
          },
          {
            "t": 7.516649,
            "value": 395535.19867733034
          },
          {
            "t": 8.527295,
            "value": 394796.99123135104
          },
          {
            "t": 9.537614,
            "value": 396904.3440735055
          },
          {
            "t": 10.552442,
            "value": 393170.07414064254
          },
          {
            "t": 11.667096,
            "value": 359752.89192879584
          },
          {
            "t": 12.682415,
            "value": 392979.9402946266
          },
          {
            "t": 13.692272,
            "value": 396095.68483458547
          },
          {
            "t": 14.702945,
            "value": 396765.3237001483
          },
          {
            "t": 15.713332,
            "value": 394898.19247476466
          },
          {
            "t": 16.723463,
            "value": 396978.2137168348
          },
          {
            "t": 17.7361,
            "value": 395008.28036107705
          },
          {
            "t": 18.857642,
            "value": 355760.19444657443
          },
          {
            "t": 19.872395,
            "value": 394184.5946747632
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.120889,
            "value": 339722455.6506536
          },
          {
            "t": 2.041949,
            "value": 359393600.92865396
          },
          {
            "t": 4.072917,
            "value": 339482969.6972084
          },
          {
            "t": 6.102364,
            "value": 340183158.26922315
          },
          {
            "t": 8.124014,
            "value": 341482971.3352954
          },
          {
            "t": 10.14539,
            "value": 341114417.60464156
          },
          {
            "t": 12.071239,
            "value": 358036987.84276444
          },
          {
            "t": 14.098525,
            "value": 340978221.1291352
          },
          {
            "t": 16.118999,
            "value": 340418901.7032637
          },
          {
            "t": 18.14298,
            "value": 341536492.1903911
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.120889,
            "value": 367639622.1863405
          },
          {
            "t": 2.041949,
            "value": 387951688.1305113
          },
          {
            "t": 4.072917,
            "value": 367375468.74199885
          },
          {
            "t": 6.102364,
            "value": 367658491.697492
          },
          {
            "t": 8.124014,
            "value": 368643471.4218584
          },
          {
            "t": 10.14539,
            "value": 368695734.9844858
          },
          {
            "t": 12.071239,
            "value": 386934762.2788703
          },
          {
            "t": 14.098525,
            "value": 368063189.4069213
          },
          {
            "t": 16.118999,
            "value": 369294531.87717336
          },
          {
            "t": 18.14298,
            "value": 368186557.0872454
          }
        ],
        "ram_mib": [
          {
            "t": 0.120889,
            "value": 57.7265625
          },
          {
            "t": 2.041949,
            "value": 59.78125
          },
          {
            "t": 4.072917,
            "value": 59.13671875
          },
          {
            "t": 6.102364,
            "value": 59.3203125
          },
          {
            "t": 8.124014,
            "value": 60.1328125
          },
          {
            "t": 10.14539,
            "value": 60.5078125
          },
          {
            "t": 12.071239,
            "value": 60.81640625
          },
          {
            "t": 14.098525,
            "value": 62.98046875
          },
          {
            "t": 16.118999,
            "value": 61.17578125
          },
          {
            "t": 18.14298,
            "value": 62.75390625
          }
        ]
      },
      "configFiles": [
        "backend-config.rendered.yaml",
        "engine-config.rendered.yaml",
        "loadgen-config.rendered.yaml"
      ]
    }
  ]
};
