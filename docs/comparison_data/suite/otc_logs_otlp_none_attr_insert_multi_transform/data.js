window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otlp_none_attr_insert_multi_transform"] = {
  "name": "OTC OTLP Attr Insert Multi Transform (Logs)",
  "slug": "otc_logs_otlp_none_attr_insert_multi_transform",
  "description": "OpenTelemetry Collector OTLP logs, attributes processor insert sweep over 1-4 insert actions at 400k signals/sec",
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
    "started_at": "2026-05-27T18:50:17Z",
    "ended_at": "2026-05-27T18:54:19Z",
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
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-1 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 10.808481216430664
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.0563981634074
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.21785536159602
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 701.15078125
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 799.43359375
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 297557.41829725244
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 269626.16910094523
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-1 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000688
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 271050738.4125473
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 239502495.73562807
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-1 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 1005.2834979495952
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.063227,
            "value": 100.01020217729393
          },
          {
            "t": 3.080487,
            "value": 100.10191780821917
          },
          {
            "t": 5.099011,
            "value": 99.92167238689548
          },
          {
            "t": 7.117333,
            "value": 100.09932752179327
          },
          {
            "t": 9.134828,
            "value": 100.13947056991593
          },
          {
            "t": 11.084084,
            "value": 100.04440572495332
          },
          {
            "t": 13.095454,
            "value": 100.12641544690129
          },
          {
            "t": 15.117195,
            "value": 99.77669781931465
          },
          {
            "t": 17.12959,
            "value": 100.21785536159602
          },
          {
            "t": 19.143268,
            "value": 100.12601681719092
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.155194,
            "value": 302249.52928351995
          },
          {
            "t": 1.169243,
            "value": 298802.1288912074
          },
          {
            "t": 2.275968,
            "value": 272877.1826786239
          },
          {
            "t": 3.281813,
            "value": 302233.44551098824
          },
          {
            "t": 4.290975,
            "value": 300249.11758468905
          },
          {
            "t": 5.300294,
            "value": 609321.7307907608
          },
          {
            "t": 6.310097,
            "value": 281242.9751149481
          },
          {
            "t": 7.323722,
            "value": 281169.0714021458
          },
          {
            "t": 8.430456,
            "value": 268357.16622060945
          },
          {
            "t": 9.436327,
            "value": 271406.57201569586
          },
          {
            "t": 10.445525,
            "value": 262584.7455107918
          },
          {
            "t": 11.485905,
            "value": 291239.73932601546
          },
          {
            "t": 12.491649,
            "value": 242606.46844525047
          },
          {
            "t": 13.501696,
            "value": 327707.5225212292
          },
          {
            "t": 14.513291,
            "value": 304469.67412848026
          },
          {
            "t": 15.619466,
            "value": 207923.70104187852
          },
          {
            "t": 16.625352,
            "value": 330057.28283324355
          },
          {
            "t": 17.631791,
            "value": 290131.841075316
          },
          {
            "t": 18.639889,
            "value": 219224.71823175924
          },
          {
            "t": 19.650591,
            "value": 305728.09789631364
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.054195,
            "value": 258664.77441656723
          },
          {
            "t": 1.063227,
            "value": 276502.6282615418
          },
          {
            "t": 2.074688,
            "value": 262985.9183893398
          },
          {
            "t": 3.080487,
            "value": 272420.23505690496
          },
          {
            "t": 4.089733,
            "value": 269508.1278498998
          },
          {
            "t": 5.099011,
            "value": 277426.0411898407
          },
          {
            "t": 6.108205,
            "value": 268531.12483823724
          },
          {
            "t": 7.117333,
            "value": 280440.14238035213
          },
          {
            "t": 8.128793,
            "value": 279793.5657366579
          },
          {
            "t": 9.134828,
            "value": 265398.3211319686
          },
          {
            "t": 10.143932,
            "value": 269546.0527358924
          },
          {
            "t": 11.184394,
            "value": 266227.8872270203
          },
          {
            "t": 12.190118,
            "value": 264486.0816685293
          },
          {
            "t": 13.195904,
            "value": 264469.77786527155
          },
          {
            "t": 14.206671,
            "value": 271081.26798757777
          },
          {
            "t": 15.21767,
            "value": 267062.5786969127
          },
          {
            "t": 16.223506,
            "value": 265450.82896217675
          },
          {
            "t": 17.230053,
            "value": 276191.7724656673
          },
          {
            "t": 18.235801,
            "value": 263485.48542974977
          },
          {
            "t": 19.243727,
            "value": 261923.99045167997
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.063227,
            "value": 252585177.84406805
          },
          {
            "t": 3.080487,
            "value": 247825785.96710393
          },
          {
            "t": 5.099011,
            "value": 247141991.8712881
          },
          {
            "t": 7.117333,
            "value": 245261696.59747058
          },
          {
            "t": 9.134828,
            "value": 239912980.20565107
          },
          {
            "t": 11.084084,
            "value": 231495127.3716741
          },
          {
            "t": 13.095454,
            "value": 225330317.64419279
          },
          {
            "t": 15.117195,
            "value": 221577302.92851558
          },
          {
            "t": 17.12959,
            "value": 241237627.3047786
          },
          {
            "t": 19.143268,
            "value": 242656949.62153828
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.063227,
            "value": 269705927.97042704
          },
          {
            "t": 3.080487,
            "value": 269300049.572192
          },
          {
            "t": 5.099011,
            "value": 274064308.3758231
          },
          {
            "t": 7.117333,
            "value": 277543024.353894
          },
          {
            "t": 9.134828,
            "value": 273701305.33161175
          },
          {
            "t": 11.084084,
            "value": 275612368.0009193
          },
          {
            "t": 13.095454,
            "value": 269078864.15726596
          },
          {
            "t": 15.117195,
            "value": 261789210.88309532
          },
          {
            "t": 17.12959,
            "value": 269444386.9121122
          },
          {
            "t": 19.143268,
            "value": 270267938.5681325
          }
        ],
        "ram_mib": [
          {
            "t": 1.063227,
            "value": 501.796875
          },
          {
            "t": 3.080487,
            "value": 605.84765625
          },
          {
            "t": 5.099011,
            "value": 659.3671875
          },
          {
            "t": 7.117333,
            "value": 668.3359375
          },
          {
            "t": 9.134828,
            "value": 678.56640625
          },
          {
            "t": 11.084084,
            "value": 779.0859375
          },
          {
            "t": 13.095454,
            "value": 792.72265625
          },
          {
            "t": 15.117195,
            "value": 749.32421875
          },
          {
            "t": 17.12959,
            "value": 777.02734375
          },
          {
            "t": 19.143268,
            "value": 799.43359375
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
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-2 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 9.292274475097656
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 99.94798693960178
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.20635514018691
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 886.091015625
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 1234.4296875
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 284484.1921975817
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 262092.89161098254
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-2 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.0007
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 265240420.09332237
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 232137650.77901405
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-2 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 1012.0092096469813
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.073048,
            "value": 100.12095860566448
          },
          {
            "t": 3.091014,
            "value": 99.56285891858298
          },
          {
            "t": 5.105234,
            "value": 99.71431061313413
          },
          {
            "t": 7.119259,
            "value": 100.20635514018691
          },
          {
            "t": 9.138167,
            "value": 99.82996896337679
          },
          {
            "t": 11.053099,
            "value": 99.99099782405968
          },
          {
            "t": 13.073152,
            "value": 100.1224525365702
          },
          {
            "t": 15.093316,
            "value": 99.75113325031133
          },
          {
            "t": 17.170363,
            "value": 100.05091135303266
          },
          {
            "t": 19.08382,
            "value": 100.12992219109866
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.167611,
            "value": 300046.69600898155
          },
          {
            "t": 1.174094,
            "value": 299061.1863290289
          },
          {
            "t": 2.185541,
            "value": 297593.44780299906
          },
          {
            "t": 3.29335,
            "value": 271707.4874820479
          },
          {
            "t": 4.300131,
            "value": 299965.9310217416
          },
          {
            "t": 5.306717,
            "value": 598061.1691400437
          },
          {
            "t": 6.314058,
            "value": 260090.67435952672
          },
          {
            "t": 7.325873,
            "value": 297485.2122176485
          },
          {
            "t": 8.433122,
            "value": 238428.75450779364
          },
          {
            "t": 9.440944,
            "value": 305609.52231644077
          },
          {
            "t": 10.448294,
            "value": 255124.8324812627
          },
          {
            "t": 11.455411,
            "value": 248233.32343709818
          },
          {
            "t": 12.468417,
            "value": 239880.119170074
          },
          {
            "t": 13.480486,
            "value": 239114.1315463669
          },
          {
            "t": 14.588267,
            "value": 240119.66264090105
          },
          {
            "t": 15.553525,
            "value": 271430.021818001
          },
          {
            "t": 16.565757,
            "value": 266737.2697168238
          },
          {
            "t": 17.673306,
            "value": 235655.4879287508
          },
          {
            "t": 18.680163,
            "value": 290011.3918858388
          },
          {
            "t": 19.687149,
            "value": 266140.74078487686
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.066515,
            "value": 274204.7812968495
          },
          {
            "t": 1.073048,
            "value": 274208.59524724976
          },
          {
            "t": 2.079517,
            "value": 278200.3221162301
          },
          {
            "t": 3.091014,
            "value": 271874.26161422126
          },
          {
            "t": 4.09859,
            "value": 271939.78419493913
          },
          {
            "t": 5.105234,
            "value": 275171.75883430487
          },
          {
            "t": 6.11175,
            "value": 278187.33134893037
          },
          {
            "t": 7.119259,
            "value": 289823.7137335746
          },
          {
            "t": 8.131218,
            "value": 251986.4935239471
          },
          {
            "t": 9.138167,
            "value": 203585.2858486378
          },
          {
            "t": 10.146137,
            "value": 248023.25466035694
          },
          {
            "t": 11.15349,
            "value": 253138.67134956666
          },
          {
            "t": 12.161497,
            "value": 273807.6223677018
          },
          {
            "t": 13.173546,
            "value": 257892.65144276613
          },
          {
            "t": 14.185983,
            "value": 271621.8391860432
          },
          {
            "t": 15.193831,
            "value": 272858.60566275864
          },
          {
            "t": 16.258717,
            "value": 242279.45526563408
          },
          {
            "t": 17.270973,
            "value": 245985.20532355452
          },
          {
            "t": 18.277945,
            "value": 246282.91551304306
          },
          {
            "t": 19.284892,
            "value": 274095.8560877583
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.073048,
            "value": 254940559.2425537
          },
          {
            "t": 3.091014,
            "value": 245390925.31786957
          },
          {
            "t": 5.105234,
            "value": 257207161.0846879
          },
          {
            "t": 7.119259,
            "value": 243498705.3288812
          },
          {
            "t": 9.138167,
            "value": 180961738.72212106
          },
          {
            "t": 11.053099,
            "value": 230738098.7941086
          },
          {
            "t": 13.073152,
            "value": 238731753.5728023
          },
          {
            "t": 15.093316,
            "value": 231401299.10244912
          },
          {
            "t": 17.170363,
            "value": 200706055.2794424
          },
          {
            "t": 19.08382,
            "value": 237800211.34522492
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.073048,
            "value": 272375174.3596876
          },
          {
            "t": 3.091014,
            "value": 272687583.93352515
          },
          {
            "t": 5.105234,
            "value": 278152881.5124465
          },
          {
            "t": 7.119259,
            "value": 275669052.27094996
          },
          {
            "t": 9.138167,
            "value": 209330535.61628363
          },
          {
            "t": 11.053099,
            "value": 291990950.0702897
          },
          {
            "t": 13.073152,
            "value": 265471894.05426493
          },
          {
            "t": 15.093316,
            "value": 266441971.54290447
          },
          {
            "t": 17.170363,
            "value": 238990712.77635992
          },
          {
            "t": 19.08382,
            "value": 281293444.7965123
          }
        ],
        "ram_mib": [
          {
            "t": 1.073048,
            "value": 456.18359375
          },
          {
            "t": 3.091014,
            "value": 562.9765625
          },
          {
            "t": 5.105234,
            "value": 572.28125
          },
          {
            "t": 7.119259,
            "value": 679.97265625
          },
          {
            "t": 9.138167,
            "value": 1036.37890625
          },
          {
            "t": 11.053099,
            "value": 1234.4296875
          },
          {
            "t": 13.073152,
            "value": 1109.3203125
          },
          {
            "t": 15.093316,
            "value": 1078.69140625
          },
          {
            "t": 17.170363,
            "value": 1211.89453125
          },
          {
            "t": 19.08382,
            "value": 918.78125
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
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-3 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 6.210951805114746
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.12405114167488
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.24351730589336
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 665.701953125
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 876.6328125
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 278022.0503875884
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 266218.254425057
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-3 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000687
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 267709949.72337562
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 231474861.10088366
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-3 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 1005.6032795404666
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.057997,
            "value": 100.14434890965731
          },
          {
            "t": 3.083511,
            "value": 100.20648379052368
          },
          {
            "t": 5.09994,
            "value": 99.97067748985327
          },
          {
            "t": 7.116276,
            "value": 100.11515415758329
          },
          {
            "t": 9.137383,
            "value": 100.2125685785536
          },
          {
            "t": 11.053288,
            "value": 100.24351730589336
          },
          {
            "t": 13.070727,
            "value": 100.0553252412076
          },
          {
            "t": 15.092571,
            "value": 100.05472766884532
          },
          {
            "t": 17.092734,
            "value": 100.11834319526626
          },
          {
            "t": 19.119824,
            "value": 100.11936507936508
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.149768,
            "value": 397152.7810103527
          },
          {
            "t": 1.164095,
            "value": 306607.23809974495
          },
          {
            "t": 2.176886,
            "value": 249804.74747504666
          },
          {
            "t": 3.285266,
            "value": 276078.60120175395
          },
          {
            "t": 4.293933,
            "value": 364837.949491755
          },
          {
            "t": 5.301807,
            "value": 423664.0691197511
          },
          {
            "t": 6.309745,
            "value": 271842.1172730862
          },
          {
            "t": 7.323038,
            "value": 277313.669392762
          },
          {
            "t": 8.431573,
            "value": 239956.3387714416
          },
          {
            "t": 9.439463,
            "value": 272847.23531337746
          },
          {
            "t": 10.448058,
            "value": 277613.9084568137
          },
          {
            "t": 11.456233,
            "value": 243013.36573511543
          },
          {
            "t": 12.46486,
            "value": 303382.71729787125
          },
          {
            "t": 13.4786,
            "value": 271272.7129244185
          },
          {
            "t": 14.587523,
            "value": 250693.69108585536
          },
          {
            "t": 15.595712,
            "value": 271774.4391180622
          },
          {
            "t": 16.587718,
            "value": 257054.89684538197
          },
          {
            "t": 17.600694,
            "value": 253707.88646522717
          },
          {
            "t": 18.613964,
            "value": 251660.46562120662
          },
          {
            "t": 19.723755,
            "value": 230674.0638552665
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.04841,
            "value": 272366.7581818974
          },
          {
            "t": 1.057997,
            "value": 278331.6346189085
          },
          {
            "t": 2.070519,
            "value": 271599.0368604337
          },
          {
            "t": 3.083511,
            "value": 280357.5941369725
          },
          {
            "t": 4.091945,
            "value": 264766.95549733547
          },
          {
            "t": 5.09994,
            "value": 252977.4453246296
          },
          {
            "t": 6.107819,
            "value": 267889.3002036951
          },
          {
            "t": 7.116276,
            "value": 277651.89789946424
          },
          {
            "t": 8.129135,
            "value": 259661.018957229
          },
          {
            "t": 9.137383,
            "value": 275725.81349033176
          },
          {
            "t": 10.145866,
            "value": 277644.73967335094
          },
          {
            "t": 11.153781,
            "value": 273832.6148534351
          },
          {
            "t": 12.162632,
            "value": 270604.8762403963
          },
          {
            "t": 13.171239,
            "value": 273644.7397251853
          },
          {
            "t": 14.184834,
            "value": 271311.5198871344
          },
          {
            "t": 15.193064,
            "value": 273747.06168235425
          },
          {
            "t": 16.185025,
            "value": 261098.9746572698
          },
          {
            "t": 17.195332,
            "value": 255367.9228194994
          },
          {
            "t": 18.206205,
            "value": 252257.20738411255
          },
          {
            "t": 19.220649,
            "value": 219824.8498684994
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.057997,
            "value": 255280234.04671526
          },
          {
            "t": 3.083511,
            "value": 218670741.8462672
          },
          {
            "t": 5.09994,
            "value": 215705553.2329678
          },
          {
            "t": 7.116276,
            "value": 249578520.14743575
          },
          {
            "t": 9.137383,
            "value": 232161383.34091166
          },
          {
            "t": 11.053288,
            "value": 243492774.43297032
          },
          {
            "t": 13.070727,
            "value": 234155071.35531732
          },
          {
            "t": 15.092571,
            "value": 239938767.7783251
          },
          {
            "t": 17.092734,
            "value": 211442336.9495386
          },
          {
            "t": 19.119824,
            "value": 214323227.87838727
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.057997,
            "value": 276250819.2377882
          },
          {
            "t": 3.083511,
            "value": 272811048.45486134
          },
          {
            "t": 5.09994,
            "value": 259512815.97318825
          },
          {
            "t": 7.116276,
            "value": 273385641.57957804
          },
          {
            "t": 9.137383,
            "value": 271749558.0392329
          },
          {
            "t": 11.053288,
            "value": 285098013.2104671
          },
          {
            "t": 13.070727,
            "value": 274215356.6972781
          },
          {
            "t": 15.092571,
            "value": 273134545.9887113
          },
          {
            "t": 17.092734,
            "value": 238205656.2390165
          },
          {
            "t": 19.119824,
            "value": 252736041.81363434
          }
        ],
        "ram_mib": [
          {
            "t": 1.057997,
            "value": 605.6796875
          },
          {
            "t": 3.083511,
            "value": 605.0625
          },
          {
            "t": 5.09994,
            "value": 688.5390625
          },
          {
            "t": 7.116276,
            "value": 618.98046875
          },
          {
            "t": 9.137383,
            "value": 648.0625
          },
          {
            "t": 11.053288,
            "value": 634.5390625
          },
          {
            "t": 13.070727,
            "value": 618.1015625
          },
          {
            "t": 15.092571,
            "value": 622.5078125
          },
          {
            "t": 17.092734,
            "value": 738.9140625
          },
          {
            "t": 19.119824,
            "value": 876.6328125
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
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-4 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 7.147988796234131
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.02239011337817
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.338177278402
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 628.053515625
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 678.4765625
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 285461.401599076
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 269226.3408760625
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-4 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000814
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 271999633.7757037
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 235602933.5791989
        },
        {
          "extra": "OTC OTLP Attr Insert Multi Transform (Logs)/transform-4 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 1010.3009716308476
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.093419,
            "value": 100.27837804117281
          },
          {
            "t": 3.112388,
            "value": 100.338177278402
          },
          {
            "t": 5.036037,
            "value": 99.78296042380805
          },
          {
            "t": 7.055836,
            "value": 100.03006845052893
          },
          {
            "t": 9.075256,
            "value": 100.18124026176378
          },
          {
            "t": 11.099784,
            "value": 99.7842567778124
          },
          {
            "t": 13.119918,
            "value": 99.99158942457231
          },
          {
            "t": 15.140602,
            "value": 99.88222083593263
          },
          {
            "t": 17.166264,
            "value": 99.93300590612371
          },
          {
            "t": 19.086139,
            "value": 100.02200373366522
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.185937,
            "value": 303060.61509420135
          },
          {
            "t": 1.194871,
            "value": 306263.83886359265
          },
          {
            "t": 2.203981,
            "value": 308192.36753178545
          },
          {
            "t": 3.218942,
            "value": 307400.9740275735
          },
          {
            "t": 4.329106,
            "value": 477406.94167708553
          },
          {
            "t": 5.339551,
            "value": 314712.82454760035
          },
          {
            "t": 6.348985,
            "value": 274411.2046949083
          },
          {
            "t": 7.358688,
            "value": 275328.48768400215
          },
          {
            "t": 8.36878,
            "value": 278192.4814769348
          },
          {
            "t": 9.383071,
            "value": 257322.60268502825
          },
          {
            "t": 10.493526,
            "value": 244044.10804580105
          },
          {
            "t": 11.504104,
            "value": 266184.3024486977
          },
          {
            "t": 12.513551,
            "value": 264501.25662862934
          },
          {
            "t": 13.523411,
            "value": 261422.37537876537
          },
          {
            "t": 14.53398,
            "value": 258270.34076841857
          },
          {
            "t": 15.549399,
            "value": 259991.19575268927
          },
          {
            "t": 16.660169,
            "value": 222368.26705798678
          },
          {
            "t": 17.6704,
            "value": 299931.40182789875
          },
          {
            "t": 18.680932,
            "value": 270154.73037964164
          },
          {
            "t": 19.691201,
            "value": 269235.2234899814
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.084429,
            "value": 265435.363517692
          },
          {
            "t": 1.093419,
            "value": 276514.1379002765
          },
          {
            "t": 2.102437,
            "value": 280470.7150913066
          },
          {
            "t": 3.112388,
            "value": 262388.9673855464
          },
          {
            "t": 4.127013,
            "value": 273992.85450289515
          },
          {
            "t": 5.136755,
            "value": 270366.09351695783
          },
          {
            "t": 6.146953,
            "value": 272223.8610648606
          },
          {
            "t": 7.156605,
            "value": 274351.95493100595
          },
          {
            "t": 8.165901,
            "value": 282375.041613164
          },
          {
            "t": 9.175978,
            "value": 266316.3303391722
          },
          {
            "t": 10.190749,
            "value": 264098.99376312486
          },
          {
            "t": 11.200638,
            "value": 256463.82919310933
          },
          {
            "t": 12.210937,
            "value": 266257.8108065038
          },
          {
            "t": 13.220746,
            "value": 257474.43328391807
          },
          {
            "t": 14.231282,
            "value": 256299.62712857334
          },
          {
            "t": 15.241433,
            "value": 276196.33104357665
          },
          {
            "t": 16.256675,
            "value": 263976.47063458763
          },
          {
            "t": 17.266984,
            "value": 275163.34111643076
          },
          {
            "t": 18.276953,
            "value": 273275.7144031154
          },
          {
            "t": 19.287602,
            "value": 267155.06570530427
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.093419,
            "value": 249735873.24610683
          },
          {
            "t": 3.112388,
            "value": 250616659.7902197
          },
          {
            "t": 5.036037,
            "value": 246232498.75627
          },
          {
            "t": 7.055836,
            "value": 228793077.42998192
          },
          {
            "t": 9.075256,
            "value": 224466519.5947351
          },
          {
            "t": 11.099784,
            "value": 217882449.63764393
          },
          {
            "t": 13.119918,
            "value": 220754892.00221372
          },
          {
            "t": 15.140602,
            "value": 227590932.08042425
          },
          {
            "t": 17.166264,
            "value": 244254414.1125222
          },
          {
            "t": 19.086139,
            "value": 245702019.1418712
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.093419,
            "value": 276048031.55769295
          },
          {
            "t": 3.112388,
            "value": 272049837.81326014
          },
          {
            "t": 5.036037,
            "value": 287078583.9828368
          },
          {
            "t": 7.055836,
            "value": 275374207.03743297
          },
          {
            "t": 9.075256,
            "value": 269003290.0535797
          },
          {
            "t": 11.099784,
            "value": 256827906.55402148
          },
          {
            "t": 13.119918,
            "value": 261198047.75326785
          },
          {
            "t": 15.140602,
            "value": 268350294.75167814
          },
          {
            "t": 17.166264,
            "value": 271626672.66306025
          },
          {
            "t": 19.086139,
            "value": 282439465.5902077
          }
        ],
        "ram_mib": [
          {
            "t": 1.093419,
            "value": 517.63671875
          },
          {
            "t": 3.112388,
            "value": 605.01171875
          },
          {
            "t": 5.036037,
            "value": 629.95703125
          },
          {
            "t": 7.055836,
            "value": 631.94140625
          },
          {
            "t": 9.075256,
            "value": 641.828125
          },
          {
            "t": 11.099784,
            "value": 663.328125
          },
          {
            "t": 13.119918,
            "value": 640.375
          },
          {
            "t": 15.140602,
            "value": 648.1953125
          },
          {
            "t": 17.166264,
            "value": 623.78515625
          },
          {
            "t": 19.086139,
            "value": 678.4765625
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
