window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otap_none_baseline"] = {
  "name": "DFE OTAP Baseline (Logs)",
  "slug": "dfe_logs_otap_none_baseline",
  "description": "Dataflow Engine baseline for OTAP logs with no compression",
  "meta": {
    "binary": "dfe",
    "protocols": [
      "otap"
    ],
    "signals": [
      "logs"
    ],
    "compression": "none"
  },
  "tests": [
    {
      "name": "1000k",
      "metrics": [
        {
          "extra": "DFE OTAP Baseline (Logs)/1000k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -3.739759683609009
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/1000k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 37.53557278800351
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/1000k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 38.62756218905473
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/1000k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 15.11328125
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/1000k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 15.61328125
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/1000k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 1009075.723032709
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/1000k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 1019108.3468271166
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/1000k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000681
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/1000k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 110334823.00645073
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/1000k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 110235728.00324115
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.064973,
            "value": 37.06085874299937
          },
          {
            "t": 2.089119,
            "value": 38.62756218905473
          },
          {
            "t": 4.115474,
            "value": 38.2089552238806
          },
          {
            "t": 6.138399,
            "value": 37.618572315332095
          },
          {
            "t": 8.165084,
            "value": 37.788710280373834
          },
          {
            "t": 10.185145,
            "value": 37.4423895457374
          },
          {
            "t": 12.111817,
            "value": 37.35622828784119
          },
          {
            "t": 14.133669,
            "value": 37.31574362165526
          },
          {
            "t": 16.161798,
            "value": 36.96615384615385
          },
          {
            "t": 18.184616,
            "value": 36.970553827006846
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.367591,
            "value": 957044.4005444874
          },
          {
            "t": 1.381247,
            "value": 984554.9180392561
          },
          {
            "t": 2.396756,
            "value": 985712.5835418495
          },
          {
            "t": 3.508502,
            "value": 898586.5476466748
          },
          {
            "t": 4.518848,
            "value": 1089725.6979292242
          },
          {
            "t": 5.532277,
            "value": 1183111.9890983978
          },
          {
            "t": 6.547093,
            "value": 985400.309021537
          },
          {
            "t": 6.649898,
            "value": 89475.77040875217
          },
          {
            "t": 7.658728,
            "value": 1007867.722280169
          },
          {
            "t": 8.668895,
            "value": 990925.2628525777
          },
          {
            "t": 9.679981,
            "value": 1086950.0715072704
          },
          {
            "t": 10.696751,
            "value": 786805.2755293725
          },
          {
            "t": 10.798123,
            "value": 178868.15806936866
          },
          {
            "t": 11.807395,
            "value": 918465.4513422033
          },
          {
            "t": 12.81819,
            "value": 989320.2874964755
          },
          {
            "t": 13.829999,
            "value": 988328.8249066772
          },
          {
            "t": 14.844963,
            "value": 985256.6199392292
          },
          {
            "t": 15.955973,
            "value": 900981.9893610316
          },
          {
            "t": 16.967705,
            "value": 1186084.8525103484
          },
          {
            "t": 17.981304,
            "value": 985596.8681894911
          },
          {
            "t": 18.99652,
            "value": 985012.0565475723
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.165695,
            "value": 939477.5226005941
          },
          {
            "t": 1.078736,
            "value": 1095241.0680352799
          },
          {
            "t": 2.089119,
            "value": 990713.4225338312
          },
          {
            "t": 3.104329,
            "value": 985017.8780744871
          },
          {
            "t": 4.115474,
            "value": 988977.8419514511
          },
          {
            "t": 5.128464,
            "value": 987176.5762741981
          },
          {
            "t": 6.138399,
            "value": 990162.7332452089
          },
          {
            "t": 7.154426,
            "value": 983241.5870838078
          },
          {
            "t": 8.165084,
            "value": 990443.8494525348
          },
          {
            "t": 9.175795,
            "value": 989402.5097184062
          },
          {
            "t": 10.185145,
            "value": 989745.8760588497
          },
          {
            "t": 11.201893,
            "value": 983527.8751470374
          },
          {
            "t": 12.212636,
            "value": 989371.1853557235
          },
          {
            "t": 13.225222,
            "value": 988558.0088999848
          },
          {
            "t": 14.234305,
            "value": 990007.7595202774
          },
          {
            "t": 15.24992,
            "value": 1477922.244157481
          },
          {
            "t": 16.262521,
            "value": 987555.80924767
          },
          {
            "t": 17.275836,
            "value": 986859.9596374277
          },
          {
            "t": 18.285501,
            "value": 990427.5180381611
          },
          {
            "t": 19.30105,
            "value": 984689.06965592
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.064973,
            "value": 112194567.22512728
          },
          {
            "t": 2.089119,
            "value": 109402286.69275832
          },
          {
            "t": 4.115474,
            "value": 109288795.89213143
          },
          {
            "t": 6.138399,
            "value": 109412517.5179505
          },
          {
            "t": 8.165084,
            "value": 109314507.18784615
          },
          {
            "t": 10.185145,
            "value": 109569837.24748905
          },
          {
            "t": 12.111817,
            "value": 114939016.08576863
          },
          {
            "t": 14.133669,
            "value": 109574755.22441801
          },
          {
            "t": 16.161798,
            "value": 109186114.39410414
          },
          {
            "t": 18.184616,
            "value": 109474882.564818
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.064973,
            "value": 112304776.85568358
          },
          {
            "t": 2.089119,
            "value": 109504251.669593
          },
          {
            "t": 4.115474,
            "value": 109381420.82705152
          },
          {
            "t": 6.138399,
            "value": 109513787.70839256
          },
          {
            "t": 8.165084,
            "value": 109410576.8780052
          },
          {
            "t": 10.185145,
            "value": 109672583.15466711
          },
          {
            "t": 12.111817,
            "value": 115040854.90420789
          },
          {
            "t": 14.133669,
            "value": 109626764.47138564
          },
          {
            "t": 16.161798,
            "value": 109325637.57039124
          },
          {
            "t": 18.184616,
            "value": 109567576.0251293
          }
        ],
        "ram_mib": [
          {
            "t": 0.064973,
            "value": 15.1796875
          },
          {
            "t": 2.089119,
            "value": 15.29296875
          },
          {
            "t": 4.115474,
            "value": 15.21484375
          },
          {
            "t": 6.138399,
            "value": 14.6015625
          },
          {
            "t": 8.165084,
            "value": 14.97265625
          },
          {
            "t": 10.185145,
            "value": 14.921875
          },
          {
            "t": 12.111817,
            "value": 15.03515625
          },
          {
            "t": 14.133669,
            "value": 15.27734375
          },
          {
            "t": 16.161798,
            "value": 15.0234375
          },
          {
            "t": 18.184616,
            "value": 15.61328125
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
      "name": "100k",
      "metrics": [
        {
          "extra": "DFE OTAP Baseline (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 6.422899624233999
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 6.993888198757764
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 11.282421875
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 11.8125
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 99424.37473318946
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/100k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 99424.37473318946
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000628
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 10699506.778600562
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 10633772.775893327
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.077495,
            "value": 6.372204234122042
          },
          {
            "t": 2.088634,
            "value": 6.310788485607009
          },
          {
            "t": 4.10075,
            "value": 6.520873362445415
          },
          {
            "t": 6.112437,
            "value": 6.503366336633663
          },
          {
            "t": 8.123944,
            "value": 6.147096774193549
          },
          {
            "t": 10.135247,
            "value": 6.124919254658385
          },
          {
            "t": 12.147339,
            "value": 6.411748599875544
          },
          {
            "t": 14.158535,
            "value": 6.993888198757764
          },
          {
            "t": 16.169825,
            "value": 6.2955183116076965
          },
          {
            "t": 18.18178,
            "value": 6.548592684438933
          }
        ],
        "logs_produced_rate": [
          {
            "t": 1.083362,
            "value": 99416.72209148922
          },
          {
            "t": 2.088634,
            "value": 99475.56482225706
          },
          {
            "t": 3.094312,
            "value": 99435.4057660603
          },
          {
            "t": 4.10075,
            "value": 99360.31827097149
          },
          {
            "t": 5.106813,
            "value": 99397.35384364598
          },
          {
            "t": 6.112437,
            "value": 99440.74524872118
          },
          {
            "t": 7.118115,
            "value": 99435.4057660603
          },
          {
            "t": 8.123944,
            "value": 99420.47803354247
          },
          {
            "t": 9.129134,
            "value": 99483.67970234483
          },
          {
            "t": 10.135247,
            "value": 99392.41417216555
          },
          {
            "t": 11.141899,
            "value": 99339.19567040048
          },
          {
            "t": 12.147339,
            "value": 99458.94334818589
          },
          {
            "t": 13.152874,
            "value": 99449.54675869065
          },
          {
            "t": 14.158535,
            "value": 99437.08665246044
          },
          {
            "t": 15.164239,
            "value": 99432.8351085409
          },
          {
            "t": 16.169825,
            "value": 99444.50300620732
          },
          {
            "t": 17.175759,
            "value": 99410.10046384751
          },
          {
            "t": 18.18178,
            "value": 99401.50354714265
          },
          {
            "t": 19.187497,
            "value": 99431.54982962404
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.077495,
            "value": 99478.92936797056
          },
          {
            "t": 1.083362,
            "value": 99416.72209148922
          },
          {
            "t": 2.088634,
            "value": 99475.56482225706
          },
          {
            "t": 3.094312,
            "value": 99435.4057660603
          },
          {
            "t": 4.10075,
            "value": 99360.31827097149
          },
          {
            "t": 5.106813,
            "value": 99397.35384364598
          },
          {
            "t": 6.112437,
            "value": 99440.74524872118
          },
          {
            "t": 7.118115,
            "value": 99435.4057660603
          },
          {
            "t": 8.123944,
            "value": 99420.47803354247
          },
          {
            "t": 9.129134,
            "value": 99483.67970234483
          },
          {
            "t": 10.135247,
            "value": 99392.41417216555
          },
          {
            "t": 11.141899,
            "value": 99339.19567040048
          },
          {
            "t": 12.147339,
            "value": 100453.53278166775
          },
          {
            "t": 13.152874,
            "value": 98455.05129110374
          },
          {
            "t": 14.158535,
            "value": 99437.08665246044
          },
          {
            "t": 15.164239,
            "value": 99432.8351085409
          },
          {
            "t": 16.169825,
            "value": 99444.50300620732
          },
          {
            "t": 17.175759,
            "value": 99410.10046384751
          },
          {
            "t": 18.18178,
            "value": 99401.50354714265
          },
          {
            "t": 19.187497,
            "value": 99431.54982962404
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.077495,
            "value": 7298397.4680950735
          },
          {
            "t": 2.088634,
            "value": 11013479.9235657
          },
          {
            "t": 4.10075,
            "value": 11007781.857507223
          },
          {
            "t": 6.112437,
            "value": 11010684.067650685
          },
          {
            "t": 8.123944,
            "value": 11010264.443524184
          },
          {
            "t": 10.135247,
            "value": 10957041.778389433
          },
          {
            "t": 12.147339,
            "value": 11008080.147428647
          },
          {
            "t": 14.158535,
            "value": 11012135.067889953
          },
          {
            "t": 16.169825,
            "value": 11011236.072371464
          },
          {
            "t": 18.18178,
            "value": 11008626.932510916
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.077495,
            "value": 7359190.637839924
          },
          {
            "t": 2.088634,
            "value": 11081459.809590485
          },
          {
            "t": 4.10075,
            "value": 11076764.460895894
          },
          {
            "t": 6.112437,
            "value": 11076949.84358899
          },
          {
            "t": 8.123944,
            "value": 11075953.99866866
          },
          {
            "t": 10.135247,
            "value": 11022013.093004884
          },
          {
            "t": 12.147339,
            "value": 11072874.898364488
          },
          {
            "t": 14.158535,
            "value": 11078681.043518385
          },
          {
            "t": 16.169825,
            "value": 11077230.036444273
          },
          {
            "t": 18.18178,
            "value": 11073949.964089654
          }
        ],
        "ram_mib": [
          {
            "t": 0.077495,
            "value": 11.22265625
          },
          {
            "t": 2.088634,
            "value": 11.55078125
          },
          {
            "t": 4.10075,
            "value": 11.8125
          },
          {
            "t": 6.112437,
            "value": 11.671875
          },
          {
            "t": 8.123944,
            "value": 11.10546875
          },
          {
            "t": 10.135247,
            "value": 11.3203125
          },
          {
            "t": 12.147339,
            "value": 11.06640625
          },
          {
            "t": 14.158535,
            "value": 10.9609375
          },
          {
            "t": 16.169825,
            "value": 10.98828125
          },
          {
            "t": 18.18178,
            "value": 11.125
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
      "name": "200k",
      "metrics": [
        {
          "extra": "DFE OTAP Baseline (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 9.7292763007069
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 10.40438902743142
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 11.177734375
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 11.58984375
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 203735.59132270265
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/200k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 203735.59132270265
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000846
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 22007050.38419289
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 21967867.264377557
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.03439,
            "value": 9.786183635227982
          },
          {
            "t": 3.048575,
            "value": 9.56391277258567
          },
          {
            "t": 5.06255,
            "value": 9.537772246421904
          },
          {
            "t": 7.076234,
            "value": 9.57211742660837
          },
          {
            "t": 9.093127,
            "value": 10.032210394489669
          },
          {
            "t": 11.107873,
            "value": 9.665858838226109
          },
          {
            "t": 13.12271,
            "value": 9.342621722846442
          },
          {
            "t": 15.137128,
            "value": 10.40438902743142
          },
          {
            "t": 17.15229,
            "value": 9.388296943231442
          },
          {
            "t": 19.168746,
            "value": 9.9994
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.126869,
            "value": 198679.57554095483
          },
          {
            "t": 1.135188,
            "value": 198349.9269576394
          },
          {
            "t": 2.142337,
            "value": 297870.52362659347
          },
          {
            "t": 3.149318,
            "value": 198613.47930099975
          },
          {
            "t": 4.156484,
            "value": 198576.99723779396
          },
          {
            "t": 5.1632,
            "value": 198665.76075079766
          },
          {
            "t": 6.170109,
            "value": 198627.6813495559
          },
          {
            "t": 7.176828,
            "value": 198665.16873129446
          },
          {
            "t": 8.183899,
            "value": 198595.72959602647
          },
          {
            "t": 9.193874,
            "value": 198024.7035817718
          },
          {
            "t": 10.201646,
            "value": 198457.58762894783
          },
          {
            "t": 11.208499,
            "value": 198638.72879159122
          },
          {
            "t": 12.216418,
            "value": 198428.64357155684
          },
          {
            "t": 13.223315,
            "value": 198630.04855511538
          },
          {
            "t": 14.229996,
            "value": 198672.66790572187
          },
          {
            "t": 15.237752,
            "value": 198460.73851210013
          },
          {
            "t": 16.245496,
            "value": 198463.1017401245
          },
          {
            "t": 17.252857,
            "value": 198538.55767694005
          },
          {
            "t": 18.260699,
            "value": 198443.80369145164
          },
          {
            "t": 19.269327,
            "value": 198289.16111787496
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.126869,
            "value": 198679.57554095483
          },
          {
            "t": 1.135188,
            "value": 198349.9269576394
          },
          {
            "t": 2.142337,
            "value": 198580.34908439565
          },
          {
            "t": 3.149318,
            "value": 198613.47930099975
          },
          {
            "t": 4.156484,
            "value": 198576.99723779396
          },
          {
            "t": 5.1632,
            "value": 198665.76075079766
          },
          {
            "t": 6.170109,
            "value": 198627.6813495559
          },
          {
            "t": 7.176828,
            "value": 198665.16873129446
          },
          {
            "t": 8.183899,
            "value": 198595.72959602647
          },
          {
            "t": 9.193874,
            "value": 198024.7035817718
          },
          {
            "t": 10.201646,
            "value": 198457.58762894783
          },
          {
            "t": 11.208499,
            "value": 198638.72879159122
          },
          {
            "t": 12.216418,
            "value": 198428.64357155684
          },
          {
            "t": 13.223315,
            "value": 198630.04855511538
          },
          {
            "t": 14.229996,
            "value": 198672.66790572187
          },
          {
            "t": 15.237752,
            "value": 198460.73851210013
          },
          {
            "t": 16.245496,
            "value": 198463.1017401245
          },
          {
            "t": 17.252857,
            "value": 198538.55767694005
          },
          {
            "t": 18.260699,
            "value": 198443.80369145164
          },
          {
            "t": 19.269327,
            "value": 297433.7416768124
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.03439,
            "value": 21935846.604835022
          },
          {
            "t": 3.048575,
            "value": 22032710.00429454
          },
          {
            "t": 5.06255,
            "value": 21932856.16752939
          },
          {
            "t": 7.076234,
            "value": 21990635.57142034
          },
          {
            "t": 9.093127,
            "value": 21958350.790051825
          },
          {
            "t": 11.107873,
            "value": 21980406.959487695
          },
          {
            "t": 13.12271,
            "value": 21980377.072686277
          },
          {
            "t": 15.137128,
            "value": 21928661.77724782
          },
          {
            "t": 17.15229,
            "value": 21975843.133207154
          },
          {
            "t": 19.168746,
            "value": 21962984.56301551
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.03439,
            "value": 21965526.689094953
          },
          {
            "t": 3.048575,
            "value": 22082645.337940656
          },
          {
            "t": 5.06255,
            "value": 21974195.310269494
          },
          {
            "t": 7.076234,
            "value": 22032128.178999286
          },
          {
            "t": 9.093127,
            "value": 21997722.734919503
          },
          {
            "t": 11.107873,
            "value": 22020077.46882237
          },
          {
            "t": 13.12271,
            "value": 22019286.423666034
          },
          {
            "t": 15.137128,
            "value": 21967214.84815962
          },
          {
            "t": 17.15229,
            "value": 22012633.723740324
          },
          {
            "t": 19.168746,
            "value": 21999073.12631667
          }
        ],
        "ram_mib": [
          {
            "t": 1.03439,
            "value": 11.140625
          },
          {
            "t": 3.048575,
            "value": 10.94140625
          },
          {
            "t": 5.06255,
            "value": 10.98828125
          },
          {
            "t": 7.076234,
            "value": 11.06640625
          },
          {
            "t": 9.093127,
            "value": 11.58984375
          },
          {
            "t": 11.107873,
            "value": 11.07421875
          },
          {
            "t": 13.12271,
            "value": 11.36328125
          },
          {
            "t": 15.137128,
            "value": 11.0859375
          },
          {
            "t": 17.15229,
            "value": 11.08203125
          },
          {
            "t": 19.168746,
            "value": 11.4453125
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
      "name": "300k",
      "metrics": [
        {
          "extra": "DFE OTAP Baseline (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 12.426425497621338
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 12.834046740467405
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 13.81015625
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 14.16015625
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 296983.9250416741
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/300k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 298637.44308926497
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.00055
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 33089931.11928507
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 33049541.65256529
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.10576,
            "value": 12.387368421052631
          },
          {
            "t": 3.124773,
            "value": 11.904687116564418
          },
          {
            "t": 5.043878,
            "value": 12.63960712093309
          },
          {
            "t": 7.062405,
            "value": 12.291203931203931
          },
          {
            "t": 9.079393,
            "value": 12.687509202453986
          },
          {
            "t": 11.096051,
            "value": 12.096255371393493
          },
          {
            "t": 13.11598,
            "value": 12.834046740467405
          },
          {
            "t": 15.134719,
            "value": 12.074745242480049
          },
          {
            "t": 17.156961,
            "value": 12.80677300613497
          },
          {
            "t": 19.184268,
            "value": 12.542058823529413
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.097579,
            "value": 396584.61331017275
          },
          {
            "t": 1.10576,
            "value": 297565.61569797486
          },
          {
            "t": 2.115297,
            "value": 297165.9285395186
          },
          {
            "t": 3.124773,
            "value": 297183.8855009926
          },
          {
            "t": 4.133478,
            "value": 297411.0369235803
          },
          {
            "t": 5.144841,
            "value": 296629.4001263641
          },
          {
            "t": 6.15295,
            "value": 297586.8680866851
          },
          {
            "t": 7.163112,
            "value": 296982.0682227207
          },
          {
            "t": 8.171641,
            "value": 297462.9385967087
          },
          {
            "t": 9.18011,
            "value": 297480.6364895698
          },
          {
            "t": 10.188314,
            "value": 297558.827380173
          },
          {
            "t": 11.196796,
            "value": 297476.8017674088
          },
          {
            "t": 12.20498,
            "value": 297564.7302476532
          },
          {
            "t": 13.216942,
            "value": 296453.8194121914
          },
          {
            "t": 14.226228,
            "value": 297239.8309299842
          },
          {
            "t": 15.238892,
            "value": 296248.31138462515
          },
          {
            "t": 16.250183,
            "value": 296650.518990083
          },
          {
            "t": 17.263283,
            "value": 296120.81729345577
          },
          {
            "t": 18.277461,
            "value": 295806.0616578155
          },
          {
            "t": 19.290537,
            "value": 296127.83246271743
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.097579,
            "value": 297438.4599826296
          },
          {
            "t": 1.10576,
            "value": 297565.61569797486
          },
          {
            "t": 2.115297,
            "value": 297165.9285395186
          },
          {
            "t": 3.124773,
            "value": 297183.8855009926
          },
          {
            "t": 4.133478,
            "value": 297411.0369235802
          },
          {
            "t": 5.144841,
            "value": 296629.40012636414
          },
          {
            "t": 6.15295,
            "value": 297586.8680866851
          },
          {
            "t": 7.163112,
            "value": 296982.0682227207
          },
          {
            "t": 8.171641,
            "value": 297462.93859670864
          },
          {
            "t": 9.18011,
            "value": 297480.6364895698
          },
          {
            "t": 10.188314,
            "value": 297558.827380173
          },
          {
            "t": 11.196796,
            "value": 297476.8017674088
          },
          {
            "t": 12.20498,
            "value": 297564.7302476532
          },
          {
            "t": 13.11598,
            "value": 329308.4522502744
          },
          {
            "t": 14.124469,
            "value": 297474.73695796385
          },
          {
            "t": 15.134719,
            "value": 296956.1989606533
          },
          {
            "t": 16.145694,
            "value": 296743.24290907296
          },
          {
            "t": 17.156961,
            "value": 296657.5592795968
          },
          {
            "t": 18.171186,
            "value": 295792.35376765515
          },
          {
            "t": 19.184268,
            "value": 296126.07863924146
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.10576,
            "value": 32890719.57310443
          },
          {
            "t": 3.124773,
            "value": 32908788.60116304
          },
          {
            "t": 5.043878,
            "value": 34624787.07522517
          },
          {
            "t": 7.062405,
            "value": 32816784.714794498
          },
          {
            "t": 9.079393,
            "value": 32931808.716759842
          },
          {
            "t": 11.096051,
            "value": 32944271.66133276
          },
          {
            "t": 13.11598,
            "value": 32839731.495512962
          },
          {
            "t": 15.134719,
            "value": 32913595.07098243
          },
          {
            "t": 17.156961,
            "value": 32852751.550012317
          },
          {
            "t": 19.184268,
            "value": 32772178.06676542
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.10576,
            "value": 32932439.7694756
          },
          {
            "t": 3.124773,
            "value": 32952640.225694433
          },
          {
            "t": 5.043878,
            "value": 34663057.51899974
          },
          {
            "t": 7.062405,
            "value": 32847001.798836477
          },
          {
            "t": 9.079393,
            "value": 32982463.45540975
          },
          {
            "t": 11.096051,
            "value": 32986331.346217353
          },
          {
            "t": 13.11598,
            "value": 32876970.923235424
          },
          {
            "t": 15.134719,
            "value": 32953692.37925259
          },
          {
            "t": 17.156961,
            "value": 32893617.084404342
          },
          {
            "t": 19.184268,
            "value": 32811096.691324994
          }
        ],
        "ram_mib": [
          {
            "t": 1.10576,
            "value": 13.5546875
          },
          {
            "t": 3.124773,
            "value": 13.58203125
          },
          {
            "t": 5.043878,
            "value": 13.890625
          },
          {
            "t": 7.062405,
            "value": 13.8203125
          },
          {
            "t": 9.079393,
            "value": 13.99609375
          },
          {
            "t": 11.096051,
            "value": 14.16015625
          },
          {
            "t": 13.11598,
            "value": 13.96484375
          },
          {
            "t": 15.134719,
            "value": 13.9609375
          },
          {
            "t": 17.156961,
            "value": 13.625
          },
          {
            "t": 19.184268,
            "value": 13.546875
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
      "name": "400k",
      "metrics": [
        {
          "extra": "DFE OTAP Baseline (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 16.919168069848762
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 18.083476101800127
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 12.096484375
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 12.37890625
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 393479.07832910353
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/400k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 397643.0649113585
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000606
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 44269165.97211871
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 44231508.62446503
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.02018,
            "value": 16.09046913580247
          },
          {
            "t": 3.039285,
            "value": 16.38669140383426
          },
          {
            "t": 5.062964,
            "value": 16.39549226006192
          },
          {
            "t": 7.082482,
            "value": 16.91683168316832
          },
          {
            "t": 9.102048,
            "value": 17.677725587144625
          },
          {
            "t": 11.125495,
            "value": 17.02972067039106
          },
          {
            "t": 13.157188,
            "value": 17.036728624535318
          },
          {
            "t": 15.081926,
            "value": 16.265409429280396
          },
          {
            "t": 17.102345,
            "value": 18.083476101800127
          },
          {
            "t": 19.123116,
            "value": 17.309135802469136
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.111371,
            "value": 396143.15028878837
          },
          {
            "t": 1.120937,
            "value": 396209.8565126005
          },
          {
            "t": 2.130525,
            "value": 396201.2226769732
          },
          {
            "t": 3.140085,
            "value": 397202.74178850197
          },
          {
            "t": 4.150171,
            "value": 395015.8699358272
          },
          {
            "t": 5.163945,
            "value": 394565.258134456
          },
          {
            "t": 6.173844,
            "value": 396079.21188158414
          },
          {
            "t": 7.183729,
            "value": 396084.70271367533
          },
          {
            "t": 8.193389,
            "value": 396172.96911831707
          },
          {
            "t": 9.204303,
            "value": 395681.5317623457
          },
          {
            "t": 10.216447,
            "value": 396188.68461404706
          },
          {
            "t": 11.232509,
            "value": 392692.57190998184
          },
          {
            "t": 12.248665,
            "value": 393640.3465609611
          },
          {
            "t": 13.263484,
            "value": 197079.4791977683
          },
          {
            "t": 13.364714,
            "value": 179203.6012755712
          },
          {
            "t": 14.374937,
            "value": 378820.4446197741
          },
          {
            "t": 15.384753,
            "value": 395121.48747890705
          },
          {
            "t": 16.395223,
            "value": 395855.39402456285
          },
          {
            "t": 17.405438,
            "value": 395955.31644253945
          },
          {
            "t": 18.41559,
            "value": 395980.01092904835
          },
          {
            "t": 19.426248,
            "value": 395781.75802299095
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.111371,
            "value": 396143.15028878837
          },
          {
            "t": 1.120937,
            "value": 396209.8565126005
          },
          {
            "t": 2.130525,
            "value": 396201.2226769732
          },
          {
            "t": 3.140085,
            "value": 396212.21126035106
          },
          {
            "t": 4.150171,
            "value": 396005.88464744587
          },
          {
            "t": 5.062964,
            "value": 438215.4552017818
          },
          {
            "t": 6.072662,
            "value": 396158.05914243666
          },
          {
            "t": 7.082482,
            "value": 396110.1978570439
          },
          {
            "t": 8.092119,
            "value": 396181.9941226402
          },
          {
            "t": 9.102048,
            "value": 396067.4463254347
          },
          {
            "t": 10.113581,
            "value": 395439.39742944617
          },
          {
            "t": 11.125495,
            "value": 395290.5088772366
          },
          {
            "t": 12.142755,
            "value": 393213.14118317835
          },
          {
            "t": 13.157188,
            "value": 394308.93908222625
          },
          {
            "t": 14.17212,
            "value": 394115.07371922454
          },
          {
            "t": 15.182713,
            "value": 395807.21418018924
          },
          {
            "t": 16.192799,
            "value": 396005.88464744587
          },
          {
            "t": 17.203275,
            "value": 395853.0435161251
          },
          {
            "t": 18.21349,
            "value": 395955.31644253945
          },
          {
            "t": 19.223989,
            "value": 395844.03349236364
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.02018,
            "value": 46175104.57143304
          },
          {
            "t": 3.039285,
            "value": 43874308.17119466
          },
          {
            "t": 5.062964,
            "value": 43721421.72745776
          },
          {
            "t": 7.082482,
            "value": 43808583.03813088
          },
          {
            "t": 9.102048,
            "value": 43865019.514093615
          },
          {
            "t": 11.125495,
            "value": 43670902.67251873
          },
          {
            "t": 13.157188,
            "value": 43548431.28366342
          },
          {
            "t": 15.081926,
            "value": 45968904.85873921
          },
          {
            "t": 17.102345,
            "value": 43844208.55278039
          },
          {
            "t": 19.123116,
            "value": 43838201.85463865
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.02018,
            "value": 46217672.246415906
          },
          {
            "t": 3.039285,
            "value": 43914602.757162206
          },
          {
            "t": 5.062964,
            "value": 43759951.55358137
          },
          {
            "t": 7.082482,
            "value": 43846372.253181204
          },
          {
            "t": 9.102048,
            "value": 43901559.542990915
          },
          {
            "t": 11.125495,
            "value": 43706378.27430123
          },
          {
            "t": 13.157188,
            "value": 43583999.64955335
          },
          {
            "t": 15.081926,
            "value": 46005018.864905246
          },
          {
            "t": 17.102345,
            "value": 43881268.19238979
          },
          {
            "t": 19.123116,
            "value": 43874836.38670587
          }
        ],
        "ram_mib": [
          {
            "t": 1.02018,
            "value": 12.37890625
          },
          {
            "t": 3.039285,
            "value": 11.890625
          },
          {
            "t": 5.062964,
            "value": 12.078125
          },
          {
            "t": 7.082482,
            "value": 11.8828125
          },
          {
            "t": 9.102048,
            "value": 12.32421875
          },
          {
            "t": 11.125495,
            "value": 12.30078125
          },
          {
            "t": 13.157188,
            "value": 12.3125
          },
          {
            "t": 15.081926,
            "value": 11.921875
          },
          {
            "t": 17.102345,
            "value": 11.7109375
          },
          {
            "t": 19.123116,
            "value": 12.1640625
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
      "name": "600k",
      "metrics": [
        {
          "extra": "DFE OTAP Baseline (Logs)/600k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -4.698113441467285
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/600k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 20.410882989156423
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/600k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 23.11876160990712
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/600k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 12.433984375
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/600k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 13.33984375
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/600k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 608552.2255501227
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/600k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 605700.1305164257
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/600k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000629
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/600k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 58753211.81548078
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/600k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 58707272.04749948
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.120673,
            "value": 0.053865336658354114
          },
          {
            "t": 2.130513,
            "value": 22.868336448598132
          },
          {
            "t": 4.1439,
            "value": 22.65142857142857
          },
          {
            "t": 6.174814,
            "value": 22.900349127182047
          },
          {
            "t": 8.09321,
            "value": 22.74707844348363
          },
          {
            "t": 10.1587,
            "value": 22.28441140024783
          },
          {
            "t": 12.184723,
            "value": 23.11876160990712
          },
          {
            "t": 14.200872,
            "value": 22.26973977695167
          },
          {
            "t": 16.114821,
            "value": 22.322711028958718
          },
          {
            "t": 18.132355,
            "value": 22.89214814814815
          }
        ],
        "logs_produced_rate": [
          {
            "t": 3.137021,
            "value": 596120.4481236116
          },
          {
            "t": 4.1439,
            "value": 595900.798407753
          },
          {
            "t": 5.154392,
            "value": 593770.1634451336
          },
          {
            "t": 6.174814,
            "value": 587992.0268281163
          },
          {
            "t": 7.186289,
            "value": 395462.07271558867
          },
          {
            "t": 7.287322,
            "value": 179773.98814210773
          },
          {
            "t": 8.294258,
            "value": 649898.5192071291
          },
          {
            "t": 9.302393,
            "value": 694351.4509465498
          },
          {
            "t": 10.267266,
            "value": 725484.0792518808
          },
          {
            "t": 11.277964,
            "value": 593649.1414843999
          },
          {
            "t": 12.385827,
            "value": 632749.7172484323
          },
          {
            "t": 13.393264,
            "value": 594578.1225029457
          },
          {
            "t": 14.402034,
            "value": 595775.0527870576
          },
          {
            "t": 15.408871,
            "value": 594932.4468608126
          },
          {
            "t": 16.416599,
            "value": 595398.7583951225
          },
          {
            "t": 17.425622,
            "value": 594634.6118968547
          },
          {
            "t": 18.440053,
            "value": 591464.5747221841
          },
          {
            "t": 19.548903,
            "value": 541101.1408215719
          }
        ],
        "logs_received_rate": [
          {
            "t": 2.130513,
            "value": 594978.6981706893
          },
          {
            "t": 3.137021,
            "value": 596120.4481236115
          },
          {
            "t": 4.1439,
            "value": 595900.798407753
          },
          {
            "t": 5.256124,
            "value": 539459.6771873292
          },
          {
            "t": 6.276581,
            "value": 587971.8596667964
          },
          {
            "t": 7.287322,
            "value": 593623.8858421693
          },
          {
            "t": 8.294258,
            "value": 893800.5990450237
          },
          {
            "t": 9.302393,
            "value": 595158.3865256141
          },
          {
            "t": 10.267266,
            "value": 621843.4965016121
          },
          {
            "t": 11.379092,
            "value": 539652.7873965891
          },
          {
            "t": 12.385827,
            "value": 596979.3441173695
          },
          {
            "t": 13.393264,
            "value": 594578.1225029456
          },
          {
            "t": 14.402034,
            "value": 594783.7465428195
          },
          {
            "t": 15.408871,
            "value": 595925.6562879592
          },
          {
            "t": 16.416599,
            "value": 596391.0896591144
          },
          {
            "t": 17.425622,
            "value": 594634.6118968547
          },
          {
            "t": 18.440053,
            "value": 590478.8004309805
          },
          {
            "t": 19.448033,
            "value": 595249.9057520982
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.120673,
            "value": 731.5365402501855
          },
          {
            "t": 2.130513,
            "value": 55833778.3107113
          },
          {
            "t": 4.1439,
            "value": 65997914.45956491
          },
          {
            "t": 6.174814,
            "value": 65262944.664323546
          },
          {
            "t": 8.09321,
            "value": 69260206.96456832
          },
          {
            "t": 10.1587,
            "value": 64168436.54532339
          },
          {
            "t": 12.184723,
            "value": 65582057.0645052
          },
          {
            "t": 14.200872,
            "value": 65906974.63332323
          },
          {
            "t": 16.114821,
            "value": 69421189.90631412
          },
          {
            "t": 18.132355,
            "value": 65638486.38982045
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.120673,
            "value": 28956.40589628404
          },
          {
            "t": 2.130513,
            "value": 55869586.63376189
          },
          {
            "t": 4.1439,
            "value": 66052903.88782684
          },
          {
            "t": 6.174814,
            "value": 65315410.20446951
          },
          {
            "t": 8.09321,
            "value": 69321965.85063772
          },
          {
            "t": 10.1587,
            "value": 64215391.98931004
          },
          {
            "t": 12.184723,
            "value": 65628201.65417668
          },
          {
            "t": 14.200872,
            "value": 65951161.34769801
          },
          {
            "t": 16.114821,
            "value": 69413108.70874825
          },
          {
            "t": 18.132355,
            "value": 65735431.4722825
          }
        ],
        "ram_mib": [
          {
            "t": 0.120673,
            "value": 8.1328125
          },
          {
            "t": 2.130513,
            "value": 12.94140625
          },
          {
            "t": 4.1439,
            "value": 12.95703125
          },
          {
            "t": 6.174814,
            "value": 12.92578125
          },
          {
            "t": 8.09321,
            "value": 13.33984375
          },
          {
            "t": 10.1587,
            "value": 12.7421875
          },
          {
            "t": 12.184723,
            "value": 12.67578125
          },
          {
            "t": 14.200872,
            "value": 12.6953125
          },
          {
            "t": 16.114821,
            "value": 13.13671875
          },
          {
            "t": 18.132355,
            "value": 12.79296875
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
      "name": "800k",
      "metrics": [
        {
          "extra": "DFE OTAP Baseline (Logs)/800k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -0.006410256028175354
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/800k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 30.983794006596305
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/800k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 32.29950495049505
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/800k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 19.48359375
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/800k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 34.03125
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/800k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 798422.6035486303
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/800k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 815276.5507316691
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/800k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000631
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/800k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 88517461.89003384
        },
        {
          "extra": "DFE OTAP Baseline (Logs)/800k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 88417432.53877188
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.141428,
            "value": 32.29950495049505
          },
          {
            "t": 2.058653,
            "value": 31.366835913312695
          },
          {
            "t": 4.137474,
            "value": 30.895036855036857
          },
          {
            "t": 6.153792,
            "value": 31.007403726708077
          },
          {
            "t": 8.174842,
            "value": 30.206855733662145
          },
          {
            "t": 10.097582,
            "value": 30.878213399503725
          },
          {
            "t": 12.113576,
            "value": 31.205650557620817
          },
          {
            "t": 14.129295,
            "value": 30.115092936802974
          },
          {
            "t": 16.151905,
            "value": 30.608987654320984
          },
          {
            "t": 18.168453,
            "value": 31.25435833849969
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.242562,
            "value": 892650.8059149026
          },
          {
            "t": 1.251002,
            "value": 793304.509936139
          },
          {
            "t": 2.265358,
            "value": 690093.024539708
          },
          {
            "t": 2.424522,
            "value": 85213.71599972733
          },
          {
            "t": 3.432024,
            "value": 699255.7289508652
          },
          {
            "t": 4.43996,
            "value": 793701.1873769762
          },
          {
            "t": 5.448367,
            "value": 793330.4707325513
          },
          {
            "t": 6.45609,
            "value": 793868.9500983902
          },
          {
            "t": 7.46901,
            "value": 789795.8377759347
          },
          {
            "t": 8.482602,
            "value": 493295.13255826803
          },
          {
            "t": 8.583506,
            "value": 358906.6268519582
          },
          {
            "t": 9.593208,
            "value": 756320.22016843
          },
          {
            "t": 10.600953,
            "value": 793851.6192092245
          },
          {
            "t": 11.60876,
            "value": 793802.7816833977
          },
          {
            "t": 12.617123,
            "value": 793365.0877709715
          },
          {
            "t": 13.624894,
            "value": 793831.1382248548
          },
          {
            "t": 14.637576,
            "value": 592486.0913890046
          },
          {
            "t": 14.738687,
            "value": 179566.58014550278
          },
          {
            "t": 15.746512,
            "value": 739506.3284888559
          },
          {
            "t": 16.755745,
            "value": 990851.4683923335
          },
          {
            "t": 17.764169,
            "value": 793317.0967767526
          },
          {
            "t": 18.77252,
            "value": 892546.3454689883
          },
          {
            "t": 19.781087,
            "value": 793204.6160542631
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.141428,
            "value": 792554.1570407873
          },
          {
            "t": 1.149992,
            "value": 794198.4841814699
          },
          {
            "t": 2.159212,
            "value": 791700.521194586
          },
          {
            "t": 3.129577,
            "value": 825462.5836669706
          },
          {
            "t": 4.137474,
            "value": 793731.8991920801
          },
          {
            "t": 5.146229,
            "value": 793056.787822613
          },
          {
            "t": 6.153792,
            "value": 793995.015696289
          },
          {
            "t": 7.161712,
            "value": 793713.7868084769
          },
          {
            "t": 8.174842,
            "value": 789632.1301313749
          },
          {
            "t": 9.188463,
            "value": 789249.630779157
          },
          {
            "t": 10.198121,
            "value": 791357.0733852454
          },
          {
            "t": 11.205826,
            "value": 793883.1304796542
          },
          {
            "t": 12.214341,
            "value": 793245.514444505
          },
          {
            "t": 13.222276,
            "value": 794694.1022982632
          },
          {
            "t": 14.229855,
            "value": 792989.929325641
          },
          {
            "t": 15.242682,
            "value": 790855.6940128966
          },
          {
            "t": 16.252488,
            "value": 1186366.490197127
          },
          {
            "t": 17.26082,
            "value": 794381.2157106985
          },
          {
            "t": 18.269141,
            "value": 793398.134125938
          },
          {
            "t": 19.277266,
            "value": 794544.3273403597
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.141428,
            "value": 87886161.33622207
          },
          {
            "t": 2.058653,
            "value": 92347626.90868312
          },
          {
            "t": 4.137474,
            "value": 85268624.85995668
          },
          {
            "t": 6.153792,
            "value": 87853268.68083309
          },
          {
            "t": 8.174842,
            "value": 87599722.91630588
          },
          {
            "t": 10.097582,
            "value": 92077596.55491643
          },
          {
            "t": 12.113576,
            "value": 87872185.13547163
          },
          {
            "t": 14.129295,
            "value": 87886822.02231562
          },
          {
            "t": 16.151905,
            "value": 87530925.88289389
          },
          {
            "t": 18.168453,
            "value": 87851391.09012036
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.141428,
            "value": 88032619.24711084
          },
          {
            "t": 2.058653,
            "value": 92387896.56926025
          },
          {
            "t": 4.137474,
            "value": 85412779.16665263
          },
          {
            "t": 6.153792,
            "value": 87953586.19027355
          },
          {
            "t": 8.174842,
            "value": 87688538.13611737
          },
          {
            "t": 10.097582,
            "value": 92114893.3293113
          },
          {
            "t": 12.113576,
            "value": 88022888.95701078
          },
          {
            "t": 14.129295,
            "value": 87928752.96606323
          },
          {
            "t": 16.151905,
            "value": 87682941.84247087
          },
          {
            "t": 18.168453,
            "value": 87949722.49606755
          }
        ],
        "ram_mib": [
          {
            "t": 0.141428,
            "value": 14.07421875
          },
          {
            "t": 2.058653,
            "value": 34.03125
          },
          {
            "t": 4.137474,
            "value": 30.42578125
          },
          {
            "t": 6.153792,
            "value": 23.83984375
          },
          {
            "t": 8.174842,
            "value": 18.57421875
          },
          {
            "t": 10.097582,
            "value": 14.45703125
          },
          {
            "t": 12.113576,
            "value": 14.93359375
          },
          {
            "t": 14.129295,
            "value": 14.5703125
          },
          {
            "t": 16.151905,
            "value": 15.109375
          },
          {
            "t": 18.168453,
            "value": 14.8203125
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
