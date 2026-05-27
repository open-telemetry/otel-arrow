window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otlp_none_transform_rename_multi_transform"] = {
  "name": "OTC OTLP Transform Rename Multi Transform (Logs)",
  "slug": "otc_logs_otlp_none_transform_rename_multi_transform",
  "description": "OpenTelemetry Collector OTLP logs, transform processor (OTTL) rename sweep over 1-4 rename actions at 400k signals/sec",
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
    "started_at": "2026-05-27T18:54:57Z",
    "ended_at": "2026-05-27T18:59:08Z",
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
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-1 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 11.973121643066406
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.11013595576783
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.34696629213482
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 726.1609375
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 844.046875
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 252249.49431963518
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 225593.71381002996
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-1 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000662
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 193230344.93351725
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 197961726.38144854
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-1 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 856.5413533474361
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.092103,
            "value": 100.28286961946351
          },
          {
            "t": 3.103933,
            "value": 100.33228464419474
          },
          {
            "t": 5.121095,
            "value": 99.91118876755071
          },
          {
            "t": 7.032493,
            "value": 100.34696629213482
          },
          {
            "t": 9.049919,
            "value": 99.9978600311042
          },
          {
            "t": 11.095285,
            "value": 100.12781065088758
          },
          {
            "t": 13.107274,
            "value": 100.21725685785536
          },
          {
            "t": 15.124674,
            "value": 100.15352226720648
          },
          {
            "t": 17.136981,
            "value": 99.77528201932067
          },
          {
            "t": 19.148706,
            "value": 99.9563184079602
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.187303,
            "value": 208830.5489260143
          },
          {
            "t": 1.193091,
            "value": 299267.83775507374
          },
          {
            "t": 2.199287,
            "value": 301134.17266616045
          },
          {
            "t": 3.209886,
            "value": 267168.28336461843
          },
          {
            "t": 4.316013,
            "value": 247711.15794117673
          },
          {
            "t": 5.322406,
            "value": 514709.4624068331
          },
          {
            "t": 6.328275,
            "value": 217722.18847583534
          },
          {
            "t": 7.334074,
            "value": 276397.17279496195
          },
          {
            "t": 8.345507,
            "value": 202682.7283665848
          },
          {
            "t": 9.451673,
            "value": 208829.41619973857
          },
          {
            "t": 10.45761,
            "value": 280335.64726220426
          },
          {
            "t": 11.497276,
            "value": 127925.69921494019
          },
          {
            "t": 12.503391,
            "value": 307121.94928015186
          },
          {
            "t": 13.509284,
            "value": 239588.1072837767
          },
          {
            "t": 14.520871,
            "value": 206606.05563337606
          },
          {
            "t": 15.626865,
            "value": 220616.02504172717
          },
          {
            "t": 16.63286,
            "value": 219683.00041252692
          },
          {
            "t": 17.63942,
            "value": 215585.75743125097
          },
          {
            "t": 18.645234,
            "value": 238612.70572889224
          },
          {
            "t": 19.656123,
            "value": 213673.31131311157
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.08628,
            "value": 220784.14173148473
          },
          {
            "t": 1.092103,
            "value": 220714.7778485877
          },
          {
            "t": 2.098393,
            "value": 235518.58808097072
          },
          {
            "t": 3.103933,
            "value": 215804.44338365452
          },
          {
            "t": 4.114662,
            "value": 234484.2188163197
          },
          {
            "t": 5.121095,
            "value": 231510.6917201642
          },
          {
            "t": 6.12685,
            "value": 224706.8122952409
          },
          {
            "t": 7.132812,
            "value": 222672.42699028392
          },
          {
            "t": 8.139232,
            "value": 231513.68216052937
          },
          {
            "t": 9.150249,
            "value": 222548.1866279202
          },
          {
            "t": 10.156068,
            "value": 230657.80224871472
          },
          {
            "t": 11.195628,
            "value": 223171.34172149753
          },
          {
            "t": 12.201756,
            "value": 221641.77917720212
          },
          {
            "t": 13.207684,
            "value": 234609.23644634607
          },
          {
            "t": 14.214203,
            "value": 225529.7714201123
          },
          {
            "t": 15.225155,
            "value": 215638.32902056674
          },
          {
            "t": 16.230946,
            "value": 228675.7387966287
          },
          {
            "t": 17.237374,
            "value": 223562.93743814758
          },
          {
            "t": 18.243194,
            "value": 219721.22248513653
          },
          {
            "t": 19.249048,
            "value": 223690.51572096944
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.092103,
            "value": 207830598.16141558
          },
          {
            "t": 3.103933,
            "value": 206731183.05224597
          },
          {
            "t": 5.121095,
            "value": 215141195.40225327
          },
          {
            "t": 7.032493,
            "value": 212073389.73881945
          },
          {
            "t": 9.049919,
            "value": 202877092.39397132
          },
          {
            "t": 11.095285,
            "value": 195145565.63470793
          },
          {
            "t": 13.107274,
            "value": 194783633.0119101
          },
          {
            "t": 15.124674,
            "value": 188144590.06642213
          },
          {
            "t": 17.136981,
            "value": 186011774.5453353
          },
          {
            "t": 19.148706,
            "value": 170878241.80740407
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.092103,
            "value": 192237209.20982632
          },
          {
            "t": 3.103933,
            "value": 196006117.31607544
          },
          {
            "t": 5.121095,
            "value": 191259701.50141636
          },
          {
            "t": 7.032493,
            "value": 204073914.4856278
          },
          {
            "t": 9.049919,
            "value": 197569370.5741871
          },
          {
            "t": 11.095285,
            "value": 193625241.15488377
          },
          {
            "t": 13.107274,
            "value": 190467185.45677936
          },
          {
            "t": 15.124674,
            "value": 194617847.22910678
          },
          {
            "t": 17.136981,
            "value": 192135264.15204042
          },
          {
            "t": 19.148706,
            "value": 180311598.2552287
          }
        ],
        "ram_mib": [
          {
            "t": 1.092103,
            "value": 540.8359375
          },
          {
            "t": 3.103933,
            "value": 579.08984375
          },
          {
            "t": 5.121095,
            "value": 676.8671875
          },
          {
            "t": 7.032493,
            "value": 705.71484375
          },
          {
            "t": 9.049919,
            "value": 735.23828125
          },
          {
            "t": 11.095285,
            "value": 744.640625
          },
          {
            "t": 13.107274,
            "value": 783.984375
          },
          {
            "t": 15.124674,
            "value": 843.9375
          },
          {
            "t": 17.136981,
            "value": 844.046875
          },
          {
            "t": 19.148706,
            "value": 807.25390625
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
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-2 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 13.267961502075195
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.14387125017517
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.42611683848797
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 754.548046875
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 836.12890625
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 255958.29104071198
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 226657.48252360465
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-2 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000684
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 194750253.51782104
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 201849145.68583107
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-2 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 859.2271093346292
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.027535,
            "value": 100.05059999999999
          },
          {
            "t": 3.042057,
            "value": 100.0692403486924
          },
          {
            "t": 5.061274,
            "value": 100.1393644859813
          },
          {
            "t": 7.080361,
            "value": 100.42611683848797
          },
          {
            "t": 9.096739,
            "value": 100.10856785490931
          },
          {
            "t": 11.116953,
            "value": 99.8594635059264
          },
          {
            "t": 13.131485,
            "value": 100.35042147986263
          },
          {
            "t": 15.145093,
            "value": 100.19059919279727
          },
          {
            "t": 17.118619,
            "value": 99.89141965678627
          },
          {
            "t": 19.141511,
            "value": 100.35291913830784
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.221621,
            "value": 213947.57652462472
          },
          {
            "t": 1.229274,
            "value": 270926.598739844
          },
          {
            "t": 2.236902,
            "value": 250092.29596636858
          },
          {
            "t": 3.243613,
            "value": 267206.77533075534
          },
          {
            "t": 4.255963,
            "value": 393144.66340692446
          },
          {
            "t": 5.36346,
            "value": 300678.0153806286
          },
          {
            "t": 6.372916,
            "value": 252611.30747650226
          },
          {
            "t": 7.382983,
            "value": 257408.6669498162
          },
          {
            "t": 8.391834,
            "value": 253754.02314117746
          },
          {
            "t": 9.399193,
            "value": 269020.2797612371
          },
          {
            "t": 10.411943,
            "value": 271537.8918785485
          },
          {
            "t": 11.51931,
            "value": 231179.00388940613
          },
          {
            "t": 12.526231,
            "value": 249274.76932152573
          },
          {
            "t": 13.533865,
            "value": 243143.8399260049
          },
          {
            "t": 14.54054,
            "value": 234435.14540442548
          },
          {
            "t": 15.552475,
            "value": 246063.2352868514
          },
          {
            "t": 16.614467,
            "value": 219399.01618844585
          },
          {
            "t": 17.624339,
            "value": 228741.86035457958
          },
          {
            "t": 18.636319,
            "value": 225300.89527461014
          },
          {
            "t": 19.744333,
            "value": 203968.54191373033
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.120555,
            "value": 220526.44231246808
          },
          {
            "t": 1.128171,
            "value": 232231.32621951218
          },
          {
            "t": 2.135765,
            "value": 220326.83799228654
          },
          {
            "t": 3.142536,
            "value": 227459.8692254743
          },
          {
            "t": 4.149925,
            "value": 220371.67370300845
          },
          {
            "t": 5.161861,
            "value": 228275.30594820227
          },
          {
            "t": 6.170915,
            "value": 225954.21057743195
          },
          {
            "t": 7.181068,
            "value": 224718.43374221527
          },
          {
            "t": 8.190106,
            "value": 236859.26595430498
          },
          {
            "t": 9.197373,
            "value": 220398.36508095672
          },
          {
            "t": 10.205354,
            "value": 225202.657589776
          },
          {
            "t": 11.217377,
            "value": 217386.36374865
          },
          {
            "t": 12.224381,
            "value": 228400.28440800632
          },
          {
            "t": 13.231977,
            "value": 234220.85835989824
          },
          {
            "t": 14.23877,
            "value": 221495.38187095063
          },
          {
            "t": 15.24551,
            "value": 220513.73740985757
          },
          {
            "t": 16.212222,
            "value": 237919.8768609472
          },
          {
            "t": 17.219089,
            "value": 231410.90134049483
          },
          {
            "t": 18.228992,
            "value": 223783.868351713
          },
          {
            "t": 19.241918,
            "value": 230026.6751964112
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.027535,
            "value": 226635661.888045
          },
          {
            "t": 3.042057,
            "value": 190345501.8113478
          },
          {
            "t": 5.061274,
            "value": 215740263.18122324
          },
          {
            "t": 7.080361,
            "value": 192261261.15417513
          },
          {
            "t": 9.096739,
            "value": 195226377.69307142
          },
          {
            "t": 11.116953,
            "value": 197000980.5891851
          },
          {
            "t": 13.131485,
            "value": 192765421.44776058
          },
          {
            "t": 15.145093,
            "value": 209170685.15818372
          },
          {
            "t": 17.118619,
            "value": 204246618.4889381
          },
          {
            "t": 19.141511,
            "value": 195098685.4463807
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.027535,
            "value": 204320719.64641613
          },
          {
            "t": 3.042057,
            "value": 193632741.16639084
          },
          {
            "t": 5.061274,
            "value": 195315543.10408443
          },
          {
            "t": 7.080361,
            "value": 190667691.8825192
          },
          {
            "t": 9.096739,
            "value": 195995964.5463301
          },
          {
            "t": 11.116953,
            "value": 190133395.273966
          },
          {
            "t": 13.131485,
            "value": 193200205.3082304
          },
          {
            "t": 15.145093,
            "value": 193304029.38407078
          },
          {
            "t": 17.118619,
            "value": 198524484.09597847
          },
          {
            "t": 19.141511,
            "value": 192407760.770224
          }
        ],
        "ram_mib": [
          {
            "t": 1.027535,
            "value": 610.92578125
          },
          {
            "t": 3.042057,
            "value": 661.05078125
          },
          {
            "t": 5.061274,
            "value": 719.26171875
          },
          {
            "t": 7.080361,
            "value": 765.74609375
          },
          {
            "t": 9.096739,
            "value": 768.1015625
          },
          {
            "t": 11.116953,
            "value": 768.19921875
          },
          {
            "t": 13.131485,
            "value": 780.76171875
          },
          {
            "t": 15.145093,
            "value": 804.84765625
          },
          {
            "t": 17.118619,
            "value": 830.45703125
          },
          {
            "t": 19.141511,
            "value": 836.12890625
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
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-3 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 20.30242919921875
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.1416421389945
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.33657927590511
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 1891.9828125
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 3030.47265625
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 206880.7172658062
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 168354.59383917492
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-3 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.00066
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 145482554.4382512
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 169619878.1180518
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-3 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 864.1436572691754
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.02076,
            "value": 99.83600873090114
          },
          {
            "t": 3.041102,
            "value": 100.21436408977556
          },
          {
            "t": 5.060077,
            "value": 100.33657927590511
          },
          {
            "t": 7.084229,
            "value": 100.33378277153557
          },
          {
            "t": 9.104644,
            "value": 100.20917705735661
          },
          {
            "t": 11.126116,
            "value": 100.17655344344033
          },
          {
            "t": 13.052784,
            "value": 100.09135740971358
          },
          {
            "t": 15.154924,
            "value": 100.24172123479889
          },
          {
            "t": 17.081053,
            "value": 99.7773761296354
          },
          {
            "t": 19.106899,
            "value": 100.19950124688279
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.212923,
            "value": 240035.16919782409
          },
          {
            "t": 1.222912,
            "value": 228715.3622465195
          },
          {
            "t": 2.232641,
            "value": 201044.04251041613
          },
          {
            "t": 3.243113,
            "value": 253346.95073193515
          },
          {
            "t": 4.252713,
            "value": 439778.1299524564
          },
          {
            "t": 5.267136,
            "value": 231658.78533905477
          },
          {
            "t": 6.377277,
            "value": 204478.53020472176
          },
          {
            "t": 7.387053,
            "value": 223812.01375354535
          },
          {
            "t": 8.397475,
            "value": 227627.66448078133
          },
          {
            "t": 9.407583,
            "value": 197008.63670023406
          },
          {
            "t": 10.419716,
            "value": 143261.8045256898
          },
          {
            "t": 11.434677,
            "value": 231535.99005281978
          },
          {
            "t": 12.547684,
            "value": 202155.0628163165
          },
          {
            "t": 13.556767,
            "value": 156577.80380801184
          },
          {
            "t": 14.548725,
            "value": 180451.18845757583
          },
          {
            "t": 15.561215,
            "value": 224199.7451826685
          },
          {
            "t": 16.575509,
            "value": 162674.72744588845
          },
          {
            "t": 17.691704,
            "value": 170221.15311392723
          },
          {
            "t": 18.702147,
            "value": 92038.83841047937
          },
          {
            "t": 19.712082,
            "value": 164367.01371870466
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.111485,
            "value": 211692.76060219665
          },
          {
            "t": 1.121507,
            "value": 199005.56621538938
          },
          {
            "t": 2.131279,
            "value": 168354.83653735695
          },
          {
            "t": 3.141851,
            "value": 178116.94762965923
          },
          {
            "t": 4.151047,
            "value": 187277.7934117852
          },
          {
            "t": 5.160815,
            "value": 179249.0948415874
          },
          {
            "t": 6.175312,
            "value": 166585.0170084288
          },
          {
            "t": 7.184985,
            "value": 151534.20959062982
          },
          {
            "t": 8.194474,
            "value": 142646.42804428775
          },
          {
            "t": 9.205515,
            "value": 213641.18764718738
          },
          {
            "t": 10.215407,
            "value": 132687.455688331
          },
          {
            "t": 11.227663,
            "value": 215360.5412069674
          },
          {
            "t": 12.144133,
            "value": 129846.03969578928
          },
          {
            "t": 13.153634,
            "value": 204061.21440196692
          },
          {
            "t": 14.16365,
            "value": 124750.49900199601
          },
          {
            "t": 15.255494,
            "value": 191419.2870043706
          },
          {
            "t": 16.26764,
            "value": 155115.96153124154
          },
          {
            "t": 17.18687,
            "value": 186025.26027218433
          },
          {
            "t": 18.19854,
            "value": 177923.63122362035
          },
          {
            "t": 19.208083,
            "value": 91130.34313545832
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.02076,
            "value": 201773134.10138872
          },
          {
            "t": 3.041102,
            "value": 177383455.87034276
          },
          {
            "t": 5.060077,
            "value": 168512124.71674982
          },
          {
            "t": 7.084229,
            "value": 185082554.5710006
          },
          {
            "t": 9.104644,
            "value": 157140402.83803082
          },
          {
            "t": 11.126116,
            "value": 161544294.45473397
          },
          {
            "t": 13.052784,
            "value": 173689854.1938725
          },
          {
            "t": 15.154924,
            "value": 165068087.28248358
          },
          {
            "t": 17.081053,
            "value": 170516985.6224583
          },
          {
            "t": 19.106899,
            "value": 135487887.52945682
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.02076,
            "value": 173802749.12586045
          },
          {
            "t": 3.041102,
            "value": 142801516.77290282
          },
          {
            "t": 5.060077,
            "value": 150433678.97076485
          },
          {
            "t": 7.084229,
            "value": 148925868.21543047
          },
          {
            "t": 9.104644,
            "value": 138149668.7561714
          },
          {
            "t": 11.126116,
            "value": 146940388.48918015
          },
          {
            "t": 13.052784,
            "value": 143460005.0449792
          },
          {
            "t": 15.154924,
            "value": 153473316.23964152
          },
          {
            "t": 17.081053,
            "value": 148969579.91910198
          },
          {
            "t": 19.106899,
            "value": 107868772.8484791
          }
        ],
        "ram_mib": [
          {
            "t": 1.02076,
            "value": 620.69140625
          },
          {
            "t": 3.041102,
            "value": 1030.92578125
          },
          {
            "t": 5.060077,
            "value": 1308.734375
          },
          {
            "t": 7.084229,
            "value": 1495.23046875
          },
          {
            "t": 9.104644,
            "value": 1855.00390625
          },
          {
            "t": 11.126116,
            "value": 2082.6875
          },
          {
            "t": 13.052784,
            "value": 2255.5234375
          },
          {
            "t": 15.154924,
            "value": 2488.234375
          },
          {
            "t": 17.081053,
            "value": 2752.32421875
          },
          {
            "t": 19.106899,
            "value": 3030.47265625
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
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-4 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 10.417563438415527
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 99.8442652966277
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.26716557530402
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 773.755078125
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 847.1171875
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 235685.82724127406
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 216724.47461368548
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-4 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000667
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 187057316.96398336
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 191083270.37682372
        },
        {
          "extra": "OTC OTLP Transform Rename Multi Transform (Logs)/transform-4 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 863.1111797475377
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.05485,
            "value": 100.11983805668017
          },
          {
            "t": 3.079668,
            "value": 99.96865671641791
          },
          {
            "t": 5.100622,
            "value": 99.83731677018633
          },
          {
            "t": 7.119954,
            "value": 100.26716557530402
          },
          {
            "t": 9.045085,
            "value": 99.71612903225807
          },
          {
            "t": 11.064971,
            "value": 99.66222222222221
          },
          {
            "t": 13.084021,
            "value": 99.56955223880597
          },
          {
            "t": 15.110214,
            "value": 99.9660696517413
          },
          {
            "t": 17.129829,
            "value": 99.71315136476426
          },
          {
            "t": 19.149309,
            "value": 99.62255133789671
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.147246,
            "value": 288382.9249560241
          },
          {
            "t": 1.161431,
            "value": 287915.91277725465
          },
          {
            "t": 2.272015,
            "value": 226907.64498678173
          },
          {
            "t": 3.281949,
            "value": 306950.7512372095
          },
          {
            "t": 4.292385,
            "value": 475042.4569195872
          },
          {
            "t": 5.302822,
            "value": 265231.77595436433
          },
          {
            "t": 6.312364,
            "value": 209996.21610591735
          },
          {
            "t": 7.327146,
            "value": 208911.8648143148
          },
          {
            "t": 8.438535,
            "value": 197950.4925818053
          },
          {
            "t": 9.448599,
            "value": 194047.10988610622
          },
          {
            "t": 10.458122,
            "value": 236745.47286193582
          },
          {
            "t": 11.468441,
            "value": 213793.86114682592
          },
          {
            "t": 12.478058,
            "value": 232761.532343453
          },
          {
            "t": 13.493364,
            "value": 209788.9700248004
          },
          {
            "t": 14.705157,
            "value": 179073.48862388215
          },
          {
            "t": 15.714793,
            "value": 214928.94468897703
          },
          {
            "t": 16.724136,
            "value": 241741.410006311
          },
          {
            "t": 17.734789,
            "value": 203828.61377742904
          },
          {
            "t": 18.744503,
            "value": 215902.72096851186
          },
          {
            "t": 19.859929,
            "value": 178407.1735821112
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.045926,
            "value": 219958.34662661178
          },
          {
            "t": 1.05485,
            "value": 215080.6205422807
          },
          {
            "t": 2.069969,
            "value": 219678.67806631536
          },
          {
            "t": 3.079668,
            "value": 219867.50506834217
          },
          {
            "t": 4.090205,
            "value": 218695.6044162658
          },
          {
            "t": 5.100622,
            "value": 214762.81574834947
          },
          {
            "t": 6.110288,
            "value": 211951.27893778737
          },
          {
            "t": 7.119954,
            "value": 210960.8524006949
          },
          {
            "t": 8.135753,
            "value": 218547.17321044815
          },
          {
            "t": 9.145937,
            "value": 221741.78169521593
          },
          {
            "t": 10.155478,
            "value": 224854.6616729781
          },
          {
            "t": 11.16578,
            "value": 212807.6555327021
          },
          {
            "t": 12.175234,
            "value": 216948.9644897142
          },
          {
            "t": 13.184841,
            "value": 225830.44689666378
          },
          {
            "t": 14.201248,
            "value": 208577.86300173058
          },
          {
            "t": 15.210948,
            "value": 212934.53501039912
          },
          {
            "t": 16.220414,
            "value": 222890.122104162
          },
          {
            "t": 17.230659,
            "value": 213809.5214527169
          },
          {
            "t": 18.240474,
            "value": 213900.56594524742
          },
          {
            "t": 19.250033,
            "value": 213954.80600935654
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.05485,
            "value": 203530294.7270992
          },
          {
            "t": 3.079668,
            "value": 185196645.82199487
          },
          {
            "t": 5.100622,
            "value": 183180998.6768625
          },
          {
            "t": 7.119954,
            "value": 197795916.1742596
          },
          {
            "t": 9.045085,
            "value": 211162668.41061726
          },
          {
            "t": 11.064971,
            "value": 193250634.93682316
          },
          {
            "t": 13.084021,
            "value": 168847678.36358684
          },
          {
            "t": 15.110214,
            "value": 184749753.3551838
          },
          {
            "t": 17.129829,
            "value": 194804068.59723264
          },
          {
            "t": 19.149309,
            "value": 188314044.7045774
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.05485,
            "value": 185257222.0080606
          },
          {
            "t": 3.079668,
            "value": 191799132.07014164
          },
          {
            "t": 5.100622,
            "value": 184992553.0219886
          },
          {
            "t": 7.119954,
            "value": 184297049.22221804
          },
          {
            "t": 9.045085,
            "value": 192870113.25463048
          },
          {
            "t": 11.064971,
            "value": 190155882.06463137
          },
          {
            "t": 13.084021,
            "value": 188961212.4514004
          },
          {
            "t": 15.110214,
            "value": 179460209.36801183
          },
          {
            "t": 17.129829,
            "value": 188497873.60462266
          },
          {
            "t": 19.149309,
            "value": 184281922.57412797
          }
        ],
        "ram_mib": [
          {
            "t": 1.05485,
            "value": 672.20703125
          },
          {
            "t": 3.079668,
            "value": 702.953125
          },
          {
            "t": 5.100622,
            "value": 732.58203125
          },
          {
            "t": 7.119954,
            "value": 733.671875
          },
          {
            "t": 9.045085,
            "value": 745.63671875
          },
          {
            "t": 11.064971,
            "value": 812.765625
          },
          {
            "t": 13.084021,
            "value": 847.1171875
          },
          {
            "t": 15.110214,
            "value": 816.69921875
          },
          {
            "t": 17.129829,
            "value": 832.015625
          },
          {
            "t": 19.149309,
            "value": 841.90234375
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
