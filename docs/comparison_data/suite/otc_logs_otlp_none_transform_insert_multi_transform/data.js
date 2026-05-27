window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otlp_none_transform_insert_multi_transform"] = {
  "name": "OTC OTLP Transform Insert Multi Transform (Logs)",
  "slug": "otc_logs_otlp_none_transform_insert_multi_transform",
  "description": "OpenTelemetry Collector OTLP logs, transform processor (OTTL) insert sweep over 1-4 insert actions at 400k signals/sec",
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
    "started_at": "2026-05-27T18:59:40Z",
    "ended_at": "2026-05-27T19:03:46Z",
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
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-1 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 7.553452491760254
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.3070069739515
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.52192620387743
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 685.8125
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 783.19140625
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 291617.09704123635
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 269589.9381164602
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-1 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000657
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 263349346.12949735
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 232598733.8447877
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-1 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 976.851539673313
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.072813,
            "value": 100.43011558887846
          },
          {
            "t": 3.088867,
            "value": 100.11644970414201
          },
          {
            "t": 5.09965,
            "value": 100.09718036886528
          },
          {
            "t": 7.111403,
            "value": 100.39545284197378
          },
          {
            "t": 9.12811,
            "value": 100.30407488299531
          },
          {
            "t": 11.139024,
            "value": 100.39335415365396
          },
          {
            "t": 13.096692,
            "value": 100.33298377028714
          },
          {
            "t": 15.1084,
            "value": 100.17605484574634
          },
          {
            "t": 17.121508,
            "value": 100.30247737909517
          },
          {
            "t": 19.132988,
            "value": 100.52192620387743
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.168073,
            "value": 300449.67965299054
          },
          {
            "t": 1.175494,
            "value": 300768.0006670499
          },
          {
            "t": 2.183207,
            "value": 299688.5025796035
          },
          {
            "t": 3.289643,
            "value": 275659.86645409226
          },
          {
            "t": 4.294952,
            "value": 269568.8589279515
          },
          {
            "t": 5.300389,
            "value": 560950.1142289373
          },
          {
            "t": 6.306547,
            "value": 284249.591018508
          },
          {
            "t": 7.317203,
            "value": 270121.58439666906
          },
          {
            "t": 8.422988,
            "value": 256831.11997359345
          },
          {
            "t": 9.429285,
            "value": 262347.9946775157
          },
          {
            "t": 10.43472,
            "value": 324237.7677323745
          },
          {
            "t": 11.444808,
            "value": 287103.69789562887
          },
          {
            "t": 12.492449,
            "value": 239585.88867751454
          },
          {
            "t": 13.598291,
            "value": 295702.2793491295
          },
          {
            "t": 14.603667,
            "value": 269550.8943917499
          },
          {
            "t": 15.609846,
            "value": 283249.79948895774
          },
          {
            "t": 16.617448,
            "value": 273917.67781326355
          },
          {
            "t": 17.723217,
            "value": 246887.0080459843
          },
          {
            "t": 18.729345,
            "value": 243507.78429782295
          },
          {
            "t": 19.734827,
            "value": 308309.84542736714
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.168073,
            "value": 254685.82116279993
          },
          {
            "t": 1.175494,
            "value": 260070.02037876917
          },
          {
            "t": 2.285202,
            "value": 244208.38635028317
          },
          {
            "t": 3.289643,
            "value": 257854.86653770608
          },
          {
            "t": 4.294952,
            "value": 258626.94952497192
          },
          {
            "t": 5.300389,
            "value": 262572.3938943962
          },
          {
            "t": 6.408719,
            "value": 242707.4968646522
          },
          {
            "t": 7.418118,
            "value": 257579.01483952333
          },
          {
            "t": 8.422988,
            "value": 266701.16532486794
          },
          {
            "t": 9.429285,
            "value": 526683.4741631944
          },
          {
            "t": 10.43472,
            "value": 267545.88809818635
          },
          {
            "t": 11.583955,
            "value": 221016.589296358
          },
          {
            "t": 12.593245,
            "value": 258597.62803554974
          },
          {
            "t": 13.598291,
            "value": 268644.4202553913
          },
          {
            "t": 14.603667,
            "value": 265572.28340441786
          },
          {
            "t": 15.609846,
            "value": 262378.76163187664
          },
          {
            "t": 16.719035,
            "value": 233503.9384631474
          },
          {
            "t": 17.723217,
            "value": 263896.38531660597
          },
          {
            "t": 18.729345,
            "value": 262392.0614474501
          },
          {
            "t": 19.734827,
            "value": 257587.9031151229
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.072813,
            "value": 248256931.72214517
          },
          {
            "t": 3.088867,
            "value": 234582019.13242403
          },
          {
            "t": 5.09965,
            "value": 234407895.82963452
          },
          {
            "t": 7.111403,
            "value": 235758766.11094898
          },
          {
            "t": 9.12811,
            "value": 217273933.199022
          },
          {
            "t": 11.139024,
            "value": 222463771.20055854
          },
          {
            "t": 13.096692,
            "value": 240491625.23982617
          },
          {
            "t": 15.1084,
            "value": 221867100.4937098
          },
          {
            "t": 17.121508,
            "value": 244262756.8913342
          },
          {
            "t": 19.132988,
            "value": 226622538.6282737
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.072813,
            "value": 263299499.24366745
          },
          {
            "t": 3.088867,
            "value": 257598789.0205322
          },
          {
            "t": 5.09965,
            "value": 267207525.62558964
          },
          {
            "t": 7.111403,
            "value": 260638020.67152378
          },
          {
            "t": 9.12811,
            "value": 265911578.6279316
          },
          {
            "t": 11.139024,
            "value": 259244039.7749481
          },
          {
            "t": 13.096692,
            "value": 271895378.5830897
          },
          {
            "t": 15.1084,
            "value": 258652143.84990266
          },
          {
            "t": 17.121508,
            "value": 264924168.49965328
          },
          {
            "t": 19.132988,
            "value": 264122317.39813468
          }
        ],
        "ram_mib": [
          {
            "t": 1.072813,
            "value": 541.7578125
          },
          {
            "t": 3.088867,
            "value": 634.7734375
          },
          {
            "t": 5.09965,
            "value": 641.88671875
          },
          {
            "t": 7.111403,
            "value": 663.421875
          },
          {
            "t": 9.12811,
            "value": 720.02734375
          },
          {
            "t": 11.139024,
            "value": 711.11328125
          },
          {
            "t": 13.096692,
            "value": 715.53125
          },
          {
            "t": 15.1084,
            "value": 715.54296875
          },
          {
            "t": 17.121508,
            "value": 730.87890625
          },
          {
            "t": 19.132988,
            "value": 783.19140625
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
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-2 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 6.617504119873047
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.15114992024088
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.4549
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 661.337890625
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 728.75390625
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 263522.99198471784
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 251251.54545546154
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-2 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000869
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 253074522.7223107
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 221956900.76175585
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-2 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 1007.2555862832386
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.047461,
            "value": 100.05390000000001
          },
          {
            "t": 3.06752,
            "value": 99.9287286291576
          },
          {
            "t": 5.092826,
            "value": 100.14584423676011
          },
          {
            "t": 7.118215,
            "value": 100.13966355140187
          },
          {
            "t": 9.044218,
            "value": 99.95741293532339
          },
          {
            "t": 11.118555,
            "value": 100.4532
          },
          {
            "t": 13.135413,
            "value": 99.80608478802992
          },
          {
            "t": 15.150939,
            "value": 100.14444859813084
          },
          {
            "t": 17.07548,
            "value": 100.42731646360512
          },
          {
            "t": 19.089515,
            "value": 100.4549
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.138262,
            "value": 297757.2568471845
          },
          {
            "t": 1.24956,
            "value": 270854.442282808
          },
          {
            "t": 2.260112,
            "value": 298846.5709829875
          },
          {
            "t": 3.27023,
            "value": 270265.4541350615
          },
          {
            "t": 4.280221,
            "value": 465350.68134270504
          },
          {
            "t": 5.295031,
            "value": 256205.5951360353
          },
          {
            "t": 6.310187,
            "value": 203909.54690707638
          },
          {
            "t": 7.325593,
            "value": 289539.35667112464
          },
          {
            "t": 8.437778,
            "value": 187918.37688873702
          },
          {
            "t": 9.447649,
            "value": 270331.5571988898
          },
          {
            "t": 10.457531,
            "value": 228739.5953190571
          },
          {
            "t": 11.521919,
            "value": 254606.4029282555
          },
          {
            "t": 12.530656,
            "value": 256756.71656735108
          },
          {
            "t": 13.537989,
            "value": 263070.901082363
          },
          {
            "t": 14.546217,
            "value": 240025.07369364868
          },
          {
            "t": 15.55819,
            "value": 259888.35670516902
          },
          {
            "t": 16.570397,
            "value": 254888.57516298542
          },
          {
            "t": 17.67858,
            "value": 231911.1554680048
          },
          {
            "t": 18.68559,
            "value": 263155.2814768473
          },
          {
            "t": 19.692533,
            "value": 251255.532835523
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.138262,
            "value": 255361.35603781717
          },
          {
            "t": 1.148226,
            "value": 249513.844057808
          },
          {
            "t": 2.158722,
            "value": 247403.25543099624
          },
          {
            "t": 3.168381,
            "value": 253550.95136080595
          },
          {
            "t": 4.178732,
            "value": 242489.9861533269
          },
          {
            "t": 5.092826,
            "value": 281152.70420766354
          },
          {
            "t": 6.103541,
            "value": 234487.4667933097
          },
          {
            "t": 7.118215,
            "value": 250326.70591736853
          },
          {
            "t": 8.134922,
            "value": 250809.72197496425
          },
          {
            "t": 9.144992,
            "value": 243547.47690754104
          },
          {
            "t": 10.154906,
            "value": 244575.2806674628
          },
          {
            "t": 11.219225,
            "value": 247106.36566668452
          },
          {
            "t": 12.228592,
            "value": 255605.74102382982
          },
          {
            "t": 13.236105,
            "value": 263023.90142856713
          },
          {
            "t": 14.244165,
            "value": 251969.12882169714
          },
          {
            "t": 15.25145,
            "value": 252162.9925989169
          },
          {
            "t": 16.263375,
            "value": 254959.60669021914
          },
          {
            "t": 17.276431,
            "value": 256649.1881988755
          },
          {
            "t": 18.283483,
            "value": 250235.34037964276
          },
          {
            "t": 19.290383,
            "value": 247293.6736518026
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.047461,
            "value": 230574438.1175964
          },
          {
            "t": 3.06752,
            "value": 220864542.57029128
          },
          {
            "t": 5.092826,
            "value": 211177949.4061638
          },
          {
            "t": 7.118215,
            "value": 214397226.40934655
          },
          {
            "t": 9.044218,
            "value": 231772964.00888264
          },
          {
            "t": 11.118555,
            "value": 216248969.6707912
          },
          {
            "t": 13.135413,
            "value": 231816272.63793483
          },
          {
            "t": 15.150939,
            "value": 219254246.28608117
          },
          {
            "t": 17.07548,
            "value": 230611055.31137034
          },
          {
            "t": 19.089515,
            "value": 212851343.19910035
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.047461,
            "value": 248371327.02756378
          },
          {
            "t": 3.06752,
            "value": 252652499.75372007
          },
          {
            "t": 5.092826,
            "value": 243620265.2833695
          },
          {
            "t": 7.118215,
            "value": 247055954.68327317
          },
          {
            "t": 9.044218,
            "value": 258777030.98074096
          },
          {
            "t": 11.118555,
            "value": 248532132.43556857
          },
          {
            "t": 13.135413,
            "value": 259380323.25528124
          },
          {
            "t": 15.150939,
            "value": 253699001.1540412
          },
          {
            "t": 17.07548,
            "value": 266746239.75275144
          },
          {
            "t": 19.089515,
            "value": 251910452.89679676
          }
        ],
        "ram_mib": [
          {
            "t": 1.047461,
            "value": 522.5234375
          },
          {
            "t": 3.06752,
            "value": 584.19140625
          },
          {
            "t": 5.092826,
            "value": 667.8984375
          },
          {
            "t": 7.118215,
            "value": 631.99609375
          },
          {
            "t": 9.044218,
            "value": 715.23046875
          },
          {
            "t": 11.118555,
            "value": 665.26171875
          },
          {
            "t": 13.135413,
            "value": 666.2421875
          },
          {
            "t": 15.150939,
            "value": 716.796875
          },
          {
            "t": 17.07548,
            "value": 714.484375
          },
          {
            "t": 19.089515,
            "value": 728.75390625
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
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-3 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 11.786733627319336
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 99.79972805312426
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.39735165521549
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 769.70078125
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 822.3203125
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 286815.25415268395
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 258321.25231970864
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-3 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000766
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 259110625.61106881
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 227861655.1517201
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-3 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 1003.0557814514743
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.086911,
            "value": 99.93488139825219
          },
          {
            "t": 3.104087,
            "value": 100.04915032679737
          },
          {
            "t": 5.024816,
            "value": 100.07651307596512
          },
          {
            "t": 7.040658,
            "value": 99.93148564294631
          },
          {
            "t": 9.061221,
            "value": 98.03181563375895
          },
          {
            "t": 11.047116,
            "value": 99.96547263681592
          },
          {
            "t": 13.073149,
            "value": 99.81017456359102
          },
          {
            "t": 15.090521,
            "value": 100.39735165521549
          },
          {
            "t": 17.107487,
            "value": 100.11914045468701
          },
          {
            "t": 19.128974,
            "value": 99.68129514321295
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.180339,
            "value": 297657.7313123035
          },
          {
            "t": 1.188288,
            "value": 297634.10648753063
          },
          {
            "t": 2.197541,
            "value": 300222.04541378625
          },
          {
            "t": 3.210584,
            "value": 302060.2284404512
          },
          {
            "t": 4.319095,
            "value": 543972.9511028759
          },
          {
            "t": 5.327329,
            "value": 316394.8051741957
          },
          {
            "t": 6.335005,
            "value": 304661.41894815397
          },
          {
            "t": 7.343161,
            "value": 298564.90463777434
          },
          {
            "t": 8.355438,
            "value": 262773.9245285628
          },
          {
            "t": 9.464303,
            "value": 184873.7222294869
          },
          {
            "t": 10.472525,
            "value": 278708.4590496934
          },
          {
            "t": 11.454685,
            "value": 294249.409464853
          },
          {
            "t": 12.467628,
            "value": 281358.3785069841
          },
          {
            "t": 13.576592,
            "value": 160510.16985222243
          },
          {
            "t": 14.585508,
            "value": 267613.95398625854
          },
          {
            "t": 15.59403,
            "value": 296473.45323156065
          },
          {
            "t": 16.601945,
            "value": 289706.9693376922
          },
          {
            "t": 17.616005,
            "value": 277103.91890026233
          },
          {
            "t": 18.72443,
            "value": 237273.60894963576
          },
          {
            "t": 19.732995,
            "value": 257792.01142216916
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.079133,
            "value": 238922.33140609262
          },
          {
            "t": 1.086911,
            "value": 261962.456017099
          },
          {
            "t": 2.094736,
            "value": 248058.9388038598
          },
          {
            "t": 3.104087,
            "value": 254619.05719615868
          },
          {
            "t": 4.117318,
            "value": 252657.09398942592
          },
          {
            "t": 5.125531,
            "value": 245979.7681640685
          },
          {
            "t": 6.13342,
            "value": 264910.1240315154
          },
          {
            "t": 7.141355,
            "value": 248031.8671342894
          },
          {
            "t": 8.150162,
            "value": 252773.82095881572
          },
          {
            "t": 9.161934,
            "value": 245114.51196514632
          },
          {
            "t": 10.170179,
            "value": 263824.7648141077
          },
          {
            "t": 11.147892,
            "value": 280245.8390141074
          },
          {
            "t": 12.160393,
            "value": 268641.7099834963
          },
          {
            "t": 13.173916,
            "value": 257517.58963536102
          },
          {
            "t": 14.182858,
            "value": 260669.0969352054
          },
          {
            "t": 15.191285,
            "value": 259810.5762737412
          },
          {
            "t": 16.199366,
            "value": 264859.66901469225
          },
          {
            "t": 17.208208,
            "value": 261686.17087710463
          },
          {
            "t": 18.22146,
            "value": 254625.7002206756
          },
          {
            "t": 19.229706,
            "value": 262832.681706647
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.086911,
            "value": 234964403.70666817
          },
          {
            "t": 3.104087,
            "value": 227492754.22670108
          },
          {
            "t": 5.024816,
            "value": 257036142.52713424
          },
          {
            "t": 7.040658,
            "value": 229267750.1510535
          },
          {
            "t": 9.061221,
            "value": 214705572.15983862
          },
          {
            "t": 11.047116,
            "value": 230466401.29513395
          },
          {
            "t": 13.073149,
            "value": 215151436.32902327
          },
          {
            "t": 15.090521,
            "value": 205160499.89788696
          },
          {
            "t": 17.107487,
            "value": 226689411.22458187
          },
          {
            "t": 19.128974,
            "value": 237682179.99917883
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.086911,
            "value": 247953654.2499432
          },
          {
            "t": 3.104087,
            "value": 253758721.103166
          },
          {
            "t": 5.024816,
            "value": 263670672.95802793
          },
          {
            "t": 7.040658,
            "value": 259125458.74131006
          },
          {
            "t": 9.061221,
            "value": 249797262.44615978
          },
          {
            "t": 11.047116,
            "value": 268887874.7365797
          },
          {
            "t": 13.073149,
            "value": 265687598.86931753
          },
          {
            "t": 15.090521,
            "value": 264349553.7759025
          },
          {
            "t": 17.107487,
            "value": 256018237.78883728
          },
          {
            "t": 19.128974,
            "value": 261857221.44144383
          }
        ],
        "ram_mib": [
          {
            "t": 1.086911,
            "value": 591.14453125
          },
          {
            "t": 3.104087,
            "value": 665.171875
          },
          {
            "t": 5.024816,
            "value": 753.72265625
          },
          {
            "t": 7.040658,
            "value": 805.62109375
          },
          {
            "t": 9.061221,
            "value": 818.37890625
          },
          {
            "t": 11.047116,
            "value": 821.671875
          },
          {
            "t": 13.073149,
            "value": 822.3203125
          },
          {
            "t": 15.090521,
            "value": 811.63671875
          },
          {
            "t": 17.107487,
            "value": 797.13671875
          },
          {
            "t": 19.128974,
            "value": 810.203125
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
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-4 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 12.26321029663086
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.11733590065694
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.35140093603745
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 1325.44296875
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 1847.9609375
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 256927.14519565197
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 230179.91385202843
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-4 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000678
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 232499933.7775936
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 210835582.5405153
        },
        {
          "extra": "OTC OTLP Transform Insert Multi Transform (Logs)/transform-4 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 1010.0791588924506
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.028183,
            "value": 100.35140093603745
          },
          {
            "t": 3.048095,
            "value": 99.80191900311527
          },
          {
            "t": 5.068115,
            "value": 99.82673730133999
          },
          {
            "t": 7.097862,
            "value": 100.09924680983504
          },
          {
            "t": 9.124557,
            "value": 100.01104477611939
          },
          {
            "t": 11.137136,
            "value": 100.19359501557632
          },
          {
            "t": 13.067559,
            "value": 100.09673723536739
          },
          {
            "t": 15.087311,
            "value": 100.15242367601248
          },
          {
            "t": 17.108618,
            "value": 100.30537285491418
          },
          {
            "t": 19.132555,
            "value": 100.33488139825218
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.219277,
            "value": 270024.32919206016
          },
          {
            "t": 1.230297,
            "value": 298708.2352475718
          },
          {
            "t": 2.240636,
            "value": 262288.2022766616
          },
          {
            "t": 3.250044,
            "value": 261539.43697692113
          },
          {
            "t": 4.259941,
            "value": 235667.59778472455
          },
          {
            "t": 5.275186,
            "value": 494461.92790902685
          },
          {
            "t": 6.391214,
            "value": 258058.04155451298
          },
          {
            "t": 7.400783,
            "value": 249611.46786400932
          },
          {
            "t": 8.416395,
            "value": 245172.36897555366
          },
          {
            "t": 9.427442,
            "value": 252213.79421530355
          },
          {
            "t": 10.437908,
            "value": 245431.31584833135
          },
          {
            "t": 11.44518,
            "value": 257130.14955245456
          },
          {
            "t": 12.461467,
            "value": 260753.11403176468
          },
          {
            "t": 13.571844,
            "value": 229651.73089860473
          },
          {
            "t": 14.581975,
            "value": 234623.03404211934
          },
          {
            "t": 15.591377,
            "value": 235783.1666669969
          },
          {
            "t": 16.602408,
            "value": 210676.03268346866
          },
          {
            "t": 17.613555,
            "value": 200762.10481759824
          },
          {
            "t": 18.728419,
            "value": 241285.0356635428
          },
          {
            "t": 19.738429,
            "value": 210889.001098999
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.117951,
            "value": 230765.0405819637
          },
          {
            "t": 1.128891,
            "value": 254218.84582665638
          },
          {
            "t": 2.13941,
            "value": 249376.805384164
          },
          {
            "t": 3.148695,
            "value": 247700.10452944413
          },
          {
            "t": 4.158399,
            "value": 259481.98680009192
          },
          {
            "t": 5.168766,
            "value": 218732.40119679284
          },
          {
            "t": 6.183636,
            "value": 236483.49049631975
          },
          {
            "t": 7.198661,
            "value": 204921.06105760942
          },
          {
            "t": 8.113543,
            "value": 252491.57814887603
          },
          {
            "t": 9.124557,
            "value": 238374.54278575766
          },
          {
            "t": 10.134276,
            "value": 199065.28449994503
          },
          {
            "t": 11.238055,
            "value": 211092.9814754584
          },
          {
            "t": 12.153641,
            "value": 265403.79603882105
          },
          {
            "t": 13.168377,
            "value": 203008.46722694373
          },
          {
            "t": 14.178732,
            "value": 204878.48330537288
          },
          {
            "t": 15.188067,
            "value": 199141.0185914488
          },
          {
            "t": 16.198138,
            "value": 255427.5887536619
          },
          {
            "t": 17.209551,
            "value": 245201.51510807162
          },
          {
            "t": 18.220919,
            "value": 198740.71554567674
          },
          {
            "t": 19.233433,
            "value": 237033.75953320155
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.028183,
            "value": 231124593.40690246
          },
          {
            "t": 3.048095,
            "value": 225565382.05624798
          },
          {
            "t": 5.068115,
            "value": 218096798.54654902
          },
          {
            "t": 7.097862,
            "value": 206229355.67831853
          },
          {
            "t": 9.124557,
            "value": 218447478.77702367
          },
          {
            "t": 11.137136,
            "value": 201476632.22164196
          },
          {
            "t": 13.067559,
            "value": 205901815.30162042
          },
          {
            "t": 15.087311,
            "value": 212094034.81219476
          },
          {
            "t": 17.108618,
            "value": 184662888.41823632
          },
          {
            "t": 19.132555,
            "value": 204756846.18641785
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.028183,
            "value": 260618426.73407236
          },
          {
            "t": 3.048095,
            "value": 254128365.49315017
          },
          {
            "t": 5.068115,
            "value": 229938529.32149184
          },
          {
            "t": 7.097862,
            "value": 211160792.94611594
          },
          {
            "t": 9.124557,
            "value": 230659167.75834545
          },
          {
            "t": 11.137136,
            "value": 212098581.4718329
          },
          {
            "t": 13.067559,
            "value": 232727120.42904586
          },
          {
            "t": 15.087311,
            "value": 238352029.3580598
          },
          {
            "t": 17.108618,
            "value": 212034745.8352442
          },
          {
            "t": 19.132555,
            "value": 243281578.42857757
          }
        ],
        "ram_mib": [
          {
            "t": 1.028183,
            "value": 751.27734375
          },
          {
            "t": 3.048095,
            "value": 694.4765625
          },
          {
            "t": 5.068115,
            "value": 816.86328125
          },
          {
            "t": 7.097862,
            "value": 1115.875
          },
          {
            "t": 9.124557,
            "value": 1314.2109375
          },
          {
            "t": 11.137136,
            "value": 1527.28125
          },
          {
            "t": 13.067559,
            "value": 1619.6171875
          },
          {
            "t": 15.087311,
            "value": 1726.62890625
          },
          {
            "t": 17.108618,
            "value": 1840.23828125
          },
          {
            "t": 19.132555,
            "value": 1847.9609375
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
