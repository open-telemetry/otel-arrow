window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otap_zstd_baseline"] = {
  "name": "OTC OTAP Baseline w/ Zstd (Logs)",
  "slug": "otc_logs_otap_zstd_baseline",
  "description": "OpenTelemetry Collector baseline for OTAP logs with zstd compression",
  "meta": {
    "binary": "otc",
    "protocols": [
      "otap"
    ],
    "signals": [
      "logs"
    ],
    "compression": "zstd"
  },
  "tests": [
    {
      "name": "100k",
      "metrics": [
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 43.85604147639396
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 55.57066332916145
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 1037.277734375
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 1761.5546875
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 99437.92451489209
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/100k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 99437.92451489209
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000768
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 550636.2084643606
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 750834.5352453783
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.067463,
            "value": 37.23108343711083
          },
          {
            "t": 3.077944,
            "value": 39.8985536159601
          },
          {
            "t": 5.089364,
            "value": 40.21373358348968
          },
          {
            "t": 7.100033,
            "value": 53.05826141338337
          },
          {
            "t": 9.111185,
            "value": 38.6510569105691
          },
          {
            "t": 11.124379,
            "value": 42.605273631840795
          },
          {
            "t": 13.135158,
            "value": 55.57066332916145
          },
          {
            "t": 15.146309,
            "value": 38.0261
          },
          {
            "t": 17.15695,
            "value": 39.13491864831039
          },
          {
            "t": 19.168199,
            "value": 54.170770194113956
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.060801,
            "value": 99488.33151103873
          },
          {
            "t": 1.067463,
            "value": 99338.20885262382
          },
          {
            "t": 2.072433,
            "value": 99505.45787436442
          },
          {
            "t": 3.077944,
            "value": 99451.92046631016
          },
          {
            "t": 4.084019,
            "value": 99396.16827771289
          },
          {
            "t": 5.089364,
            "value": 99468.34171354113
          },
          {
            "t": 6.094267,
            "value": 99512.09221188512
          },
          {
            "t": 7.100033,
            "value": 99426.70561542148
          },
          {
            "t": 8.105821,
            "value": 99424.5308156391
          },
          {
            "t": 9.111185,
            "value": 99466.4618983771
          },
          {
            "t": 10.117221,
            "value": 99400.02147040464
          },
          {
            "t": 11.124379,
            "value": 99289.28728163804
          },
          {
            "t": 12.129638,
            "value": 99476.85123933236
          },
          {
            "t": 13.135158,
            "value": 99451.03031267405
          },
          {
            "t": 14.140458,
            "value": 99472.79419078882
          },
          {
            "t": 15.146309,
            "value": 99418.3035061853
          },
          {
            "t": 16.151515,
            "value": 99482.0962071456
          },
          {
            "t": 17.15695,
            "value": 99459.43795471611
          },
          {
            "t": 18.162358,
            "value": 99462.10891498774
          },
          {
            "t": 19.168199,
            "value": 99419.29191591912
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.060801,
            "value": 99488.33151103873
          },
          {
            "t": 1.067463,
            "value": 99338.20885262382
          },
          {
            "t": 2.072433,
            "value": 99505.45787436442
          },
          {
            "t": 3.077944,
            "value": 99451.92046631016
          },
          {
            "t": 4.084019,
            "value": 99396.16827771289
          },
          {
            "t": 5.089364,
            "value": 99468.34171354113
          },
          {
            "t": 6.094267,
            "value": 99512.09221188512
          },
          {
            "t": 7.100033,
            "value": 99426.70561542148
          },
          {
            "t": 8.105821,
            "value": 99424.5308156391
          },
          {
            "t": 9.111185,
            "value": 99466.4618983771
          },
          {
            "t": 10.117221,
            "value": 99400.02147040464
          },
          {
            "t": 11.124379,
            "value": 99289.28728163804
          },
          {
            "t": 12.129638,
            "value": 99476.85123933236
          },
          {
            "t": 13.135158,
            "value": 99451.03031267405
          },
          {
            "t": 14.140458,
            "value": 99472.79419078882
          },
          {
            "t": 15.146309,
            "value": 99418.3035061853
          },
          {
            "t": 16.151515,
            "value": 99482.0962071456
          },
          {
            "t": 17.15695,
            "value": 99459.43795471611
          },
          {
            "t": 18.162358,
            "value": 99462.10891498774
          },
          {
            "t": 19.168199,
            "value": 99419.29191591912
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.067463,
            "value": 751113.0551917308
          },
          {
            "t": 3.077944,
            "value": 751216.7486288107
          },
          {
            "t": 5.089364,
            "value": 750820.3159956647
          },
          {
            "t": 7.100033,
            "value": 751326.0511799804
          },
          {
            "t": 9.111185,
            "value": 750990.9743271518
          },
          {
            "t": 11.124379,
            "value": 751134.7639621418
          },
          {
            "t": 13.135158,
            "value": 750588.7021895495
          },
          {
            "t": 15.146309,
            "value": 751930.6108790439
          },
          {
            "t": 17.15695,
            "value": 751698.0903105029
          },
          {
            "t": 19.168199,
            "value": 747526.0397892056
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.067463,
            "value": 551016.1273085612
          },
          {
            "t": 3.077944,
            "value": 551040.273447001
          },
          {
            "t": 5.089364,
            "value": 550465.8400532956
          },
          {
            "t": 7.100033,
            "value": 550915.1431687662
          },
          {
            "t": 9.111185,
            "value": 550755.487402245
          },
          {
            "t": 11.124379,
            "value": 551021.9084698246
          },
          {
            "t": 13.135158,
            "value": 547618.6095040778
          },
          {
            "t": 15.146309,
            "value": 554162.7655009495
          },
          {
            "t": 17.15695,
            "value": 548728.49006859
          },
          {
            "t": 19.168199,
            "value": 550637.4397202933
          }
        ],
        "ram_mib": [
          {
            "t": 1.067463,
            "value": 340.33203125
          },
          {
            "t": 3.077944,
            "value": 463.8125
          },
          {
            "t": 5.089364,
            "value": 665.63671875
          },
          {
            "t": 7.100033,
            "value": 885.97265625
          },
          {
            "t": 9.111185,
            "value": 970.43359375
          },
          {
            "t": 11.124379,
            "value": 1042.17578125
          },
          {
            "t": 13.135158,
            "value": 1407.625
          },
          {
            "t": 15.146309,
            "value": 1413.84765625
          },
          {
            "t": 17.15695,
            "value": 1421.38671875
          },
          {
            "t": 19.168199,
            "value": 1761.5546875
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
      "name": "200k",
      "metrics": [
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.6052632331848145
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 77.98787136515536
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 94.2575670617592
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 2195.225390625
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 3391.39453125
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 198089.65464788963
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/200k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 203238.3336166368
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000627
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 1093899.3882301094
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 1498874.8536349745
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.056787,
            "value": 92.5695919648462
          },
          {
            "t": 3.07446,
            "value": 80.85143570536829
          },
          {
            "t": 5.093075,
            "value": 69.7641973766396
          },
          {
            "t": 7.112951,
            "value": 90.77333333333333
          },
          {
            "t": 9.129817,
            "value": 65.49579538365565
          },
          {
            "t": 11.14928,
            "value": 69.87049282595133
          },
          {
            "t": 13.170604,
            "value": 94.2575670617592
          },
          {
            "t": 15.08561,
            "value": 66.9633
          },
          {
            "t": 17.104841,
            "value": 64.7315
          },
          {
            "t": 19.122969,
            "value": 84.6015
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.043938,
            "value": 198854.9929505905
          },
          {
            "t": 1.056787,
            "value": 197462.80047667518
          },
          {
            "t": 2.06451,
            "value": 198467.23752459756
          },
          {
            "t": 3.07446,
            "value": 198029.6054260112
          },
          {
            "t": 4.084872,
            "value": 197939.05852266203
          },
          {
            "t": 5.093075,
            "value": 198372.7483453233
          },
          {
            "t": 6.104397,
            "value": 197760.95051823257
          },
          {
            "t": 7.112951,
            "value": 198303.7100641116
          },
          {
            "t": 8.121334,
            "value": 198337.3380947517
          },
          {
            "t": 9.129817,
            "value": 198317.67119525067
          },
          {
            "t": 10.13962,
            "value": 198058.43317954097
          },
          {
            "t": 11.14928,
            "value": 198086.48455915853
          },
          {
            "t": 12.15972,
            "value": 197933.57349273583
          },
          {
            "t": 13.170604,
            "value": 197846.6372007075
          },
          {
            "t": 14.179668,
            "value": 198203.4836244282
          },
          {
            "t": 15.1911,
            "value": 197739.4426911547
          },
          {
            "t": 16.199762,
            "value": 198282.47718264395
          },
          {
            "t": 17.20908,
            "value": 198153.6047113001
          },
          {
            "t": 18.217872,
            "value": 198256.92511439425
          },
          {
            "t": 19.227171,
            "value": 198157.33494237092
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.144791,
            "value": 180731.98260273936
          },
          {
            "t": 1.157927,
            "value": 195432.79480740987
          },
          {
            "t": 2.16565,
            "value": 199459.57371222053
          },
          {
            "t": 3.175303,
            "value": 199078.29719715586
          },
          {
            "t": 4.185882,
            "value": 195927.2852493472
          },
          {
            "t": 5.193907,
            "value": 200391.85536073014
          },
          {
            "t": 6.206418,
            "value": 197528.71820651827
          },
          {
            "t": 7.213959,
            "value": 198503.08821179485
          },
          {
            "t": 8.22226,
            "value": 198353.46786326705
          },
          {
            "t": 9.230797,
            "value": 198307.05269117543
          },
          {
            "t": 10.240435,
            "value": 198090.8008612988
          },
          {
            "t": 11.25009,
            "value": 198087.46552040052
          },
          {
            "t": 12.260554,
            "value": 196939.227919055
          },
          {
            "t": 13.271557,
            "value": 198812.46643185033
          },
          {
            "t": 14.280525,
            "value": 297333.51305492345
          },
          {
            "t": 15.291825,
            "value": 197765.25264511025
          },
          {
            "t": 16.300546,
            "value": 198270.87965849825
          },
          {
            "t": 17.310589,
            "value": 198011.37179308207
          },
          {
            "t": 18.318694,
            "value": 198392.03257597174
          },
          {
            "t": 19.329164,
            "value": 196938.05852722
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.056787,
            "value": 1485505.1032717119
          },
          {
            "t": 3.07446,
            "value": 1491620.7928638586
          },
          {
            "t": 5.093075,
            "value": 1491129.3139107754
          },
          {
            "t": 7.112951,
            "value": 1490860.8251199578
          },
          {
            "t": 9.129817,
            "value": 1491521.99501603
          },
          {
            "t": 11.14928,
            "value": 1493222.7032631943
          },
          {
            "t": 13.170604,
            "value": 1485661.873108913
          },
          {
            "t": 15.08561,
            "value": 1573840.49971645
          },
          {
            "t": 17.104841,
            "value": 1492359.7151588898
          },
          {
            "t": 19.122969,
            "value": 1493025.7149199655
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.056787,
            "value": 1079733.6975448912
          },
          {
            "t": 3.07446,
            "value": 1091843.921190401
          },
          {
            "t": 5.093075,
            "value": 1090761.239760925
          },
          {
            "t": 7.112951,
            "value": 1086004.784452115
          },
          {
            "t": 9.129817,
            "value": 1087947.3400810962
          },
          {
            "t": 11.14928,
            "value": 1088420.0403770704
          },
          {
            "t": 13.170604,
            "value": 1083412.1595548266
          },
          {
            "t": 15.08561,
            "value": 1152652.7854220823
          },
          {
            "t": 17.104841,
            "value": 1087432.2947696424
          },
          {
            "t": 19.122969,
            "value": 1090785.6191480423
          }
        ],
        "ram_mib": [
          {
            "t": 1.056787,
            "value": 955.9453125
          },
          {
            "t": 3.07446,
            "value": 1183.12890625
          },
          {
            "t": 5.093075,
            "value": 1395.91796875
          },
          {
            "t": 7.112951,
            "value": 1933.66796875
          },
          {
            "t": 9.129817,
            "value": 2038.7890625
          },
          {
            "t": 11.14928,
            "value": 2112.8203125
          },
          {
            "t": 13.170604,
            "value": 2965.19140625
          },
          {
            "t": 15.08561,
            "value": 2983.796875
          },
          {
            "t": 17.104841,
            "value": 2991.6015625
          },
          {
            "t": 19.122969,
            "value": 3391.39453125
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
      "name": "300k",
      "metrics": [
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.8623663187026978
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 99.02260992331378
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 101.33553884711779
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 3107.2046875
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 5392.42578125
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 301859.3190838866
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/300k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 297668.2909474505
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000675
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 1563606.1666459881
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 2162222.7261102633
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.101675,
            "value": 100.20661654135338
          },
          {
            "t": 3.116024,
            "value": 100.72515967438949
          },
          {
            "t": 5.130381,
            "value": 98.5713354231975
          },
          {
            "t": 7.144919,
            "value": 101.0228356336261
          },
          {
            "t": 9.060488,
            "value": 91.60266164469554
          },
          {
            "t": 11.116002,
            "value": 101.2459296482412
          },
          {
            "t": 13.131447,
            "value": 100.59291770573566
          },
          {
            "t": 15.145456,
            "value": 94.01483343808925
          },
          {
            "t": 17.166871,
            "value": 101.33553884711779
          },
          {
            "t": 19.196845,
            "value": 100.90827067669173
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.094311,
            "value": 297792.16886007146
          },
          {
            "t": 1.101675,
            "value": 297806.9496229764
          },
          {
            "t": 2.108845,
            "value": 297864.3128766743
          },
          {
            "t": 3.116024,
            "value": 297861.6512059922
          },
          {
            "t": 4.123095,
            "value": 297893.5943940397
          },
          {
            "t": 5.130381,
            "value": 297830.01054318243
          },
          {
            "t": 6.137523,
            "value": 297872.5939341225
          },
          {
            "t": 7.144919,
            "value": 297797.48976569297
          },
          {
            "t": 8.152851,
            "value": 297639.1264490065
          },
          {
            "t": 9.163482,
            "value": 296844.2487911018
          },
          {
            "t": 10.176331,
            "value": 296194.2007150128
          },
          {
            "t": 11.216744,
            "value": 288347.0314192537
          },
          {
            "t": 12.224556,
            "value": 297674.5662881569
          },
          {
            "t": 13.232057,
            "value": 397021.9384397633
          },
          {
            "t": 14.239105,
            "value": 298893.3993215815
          },
          {
            "t": 15.24649,
            "value": 296808.0723854336
          },
          {
            "t": 16.260245,
            "value": 295929.48986688105
          },
          {
            "t": 17.275517,
            "value": 295487.31768432504
          },
          {
            "t": 18.289702,
            "value": 295804.0199766315
          },
          {
            "t": 19.301934,
            "value": 294398.9125022722
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.094311,
            "value": 280917.27929133404
          },
          {
            "t": 1.101675,
            "value": 292843.5004625935
          },
          {
            "t": 2.108845,
            "value": 323679.2199926527
          },
          {
            "t": 3.116024,
            "value": 276018.4634508861
          },
          {
            "t": 4.123095,
            "value": 302858.4876339404
          },
          {
            "t": 5.130381,
            "value": 346475.5789319022
          },
          {
            "t": 6.137523,
            "value": 247234.25296532168
          },
          {
            "t": 7.144919,
            "value": 292834.19826959807
          },
          {
            "t": 8.254641,
            "value": 319899.93890361727
          },
          {
            "t": 9.264511,
            "value": 299048.3923673344
          },
          {
            "t": 10.277216,
            "value": 250813.41555536905
          },
          {
            "t": 11.216744,
            "value": 250125.59497960677
          },
          {
            "t": 12.224556,
            "value": 308589.30038538936
          },
          {
            "t": 13.232057,
            "value": 352356.97036528995
          },
          {
            "t": 14.239105,
            "value": 336627.44973427284
          },
          {
            "t": 15.347565,
            "value": 410479.40385760425
          },
          {
            "t": 16.36134,
            "value": 232793.27266898472
          },
          {
            "t": 17.3768,
            "value": 232406.98796604495
          },
          {
            "t": 18.390997,
            "value": 211004.37094568412
          },
          {
            "t": 19.404396,
            "value": 354253.3592395493
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.101675,
            "value": 2038269.7250019603
          },
          {
            "t": 3.116024,
            "value": 2217465.2952393056
          },
          {
            "t": 5.130381,
            "value": 2414617.1706405566
          },
          {
            "t": 7.144919,
            "value": 2107646.0210728217
          },
          {
            "t": 9.060488,
            "value": 2434964.7545977198
          },
          {
            "t": 11.116002,
            "value": 1792602.7261307877
          },
          {
            "t": 13.131447,
            "value": 2509082.113379427
          },
          {
            "t": 15.145456,
            "value": 2295867.0989057147
          },
          {
            "t": 17.166871,
            "value": 1880006.8269009579
          },
          {
            "t": 19.196845,
            "value": 1931705.5292333793
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.101675,
            "value": 1456774.8903353123
          },
          {
            "t": 3.116024,
            "value": 1584868.8583755842
          },
          {
            "t": 5.130381,
            "value": 1778732.8661205536
          },
          {
            "t": 7.144919,
            "value": 1501575.5473463396
          },
          {
            "t": 9.060488,
            "value": 1818701.9105028322
          },
          {
            "t": 11.116002,
            "value": 1193618.2385524982
          },
          {
            "t": 13.131447,
            "value": 1872100.206157945
          },
          {
            "t": 15.145456,
            "value": 1701642.84270825
          },
          {
            "t": 17.166871,
            "value": 1281583.4452598798
          },
          {
            "t": 19.196845,
            "value": 1446462.861100684
          }
        ],
        "ram_mib": [
          {
            "t": 1.101675,
            "value": 1167.78125
          },
          {
            "t": 3.116024,
            "value": 1703.71875
          },
          {
            "t": 5.130381,
            "value": 1828.56640625
          },
          {
            "t": 7.144919,
            "value": 2506.546875
          },
          {
            "t": 9.060488,
            "value": 2545.734375
          },
          {
            "t": 11.116002,
            "value": 3679.71875
          },
          {
            "t": 13.131447,
            "value": 3698.73828125
          },
          {
            "t": 15.145456,
            "value": 3743.4765625
          },
          {
            "t": 17.166871,
            "value": 4805.33984375
          },
          {
            "t": 19.196845,
            "value": 5392.42578125
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
      "name": "400k",
      "metrics": [
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 5.241055011749268
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 99.28934881978722
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 99.66895191122072
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 3312.98203125
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 5152.921875
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 307217.2997039195
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/400k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 295734.22903250344
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000698
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 1581083.8250349783
        },
        {
          "extra": "OTC OTAP Baseline w/ Zstd (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 2195033.4987403997
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.033725,
            "value": 99.6117254174397
          },
          {
            "t": 3.050251,
            "value": 99.41441975308642
          },
          {
            "t": 5.066843,
            "value": 99.10281846153846
          },
          {
            "t": 7.084748,
            "value": 99.30998766954377
          },
          {
            "t": 9.103335,
            "value": 99.66895191122072
          },
          {
            "t": 11.12057,
            "value": 98.92307314074985
          },
          {
            "t": 13.137939,
            "value": 99.19674876847292
          },
          {
            "t": 15.155371,
            "value": 99.49361334156886
          },
          {
            "t": 17.181798,
            "value": 99.44958024691358
          },
          {
            "t": 19.099486,
            "value": 98.72256948733786
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.125575,
            "value": 300043.62476386374
          },
          {
            "t": 1.235685,
            "value": 280152.4173280126
          },
          {
            "t": 2.243677,
            "value": 290676.9101342074
          },
          {
            "t": 3.251961,
            "value": 390762.91997096053
          },
          {
            "t": 4.260119,
            "value": 315426.7485850432
          },
          {
            "t": 5.268476,
            "value": 286604.84332433855
          },
          {
            "t": 6.278081,
            "value": 284269.5905824555
          },
          {
            "t": 7.286567,
            "value": 322265.2570288532
          },
          {
            "t": 8.295289,
            "value": 328137.9805337844
          },
          {
            "t": 9.30501,
            "value": 294140.65865719336
          },
          {
            "t": 10.31377,
            "value": 264681.3910147111
          },
          {
            "t": 11.322621,
            "value": 293403.0892569864
          },
          {
            "t": 12.330998,
            "value": 328250.2476752246
          },
          {
            "t": 13.339945,
            "value": 427178.0380931803
          },
          {
            "t": 14.348435,
            "value": 290533.37167448364
          },
          {
            "t": 15.359968,
            "value": 273841.78271989146
          },
          {
            "t": 16.374713,
            "value": 243410.9061882542
          },
          {
            "t": 17.484314,
            "value": 324440.94769200817
          },
          {
            "t": 18.492518,
            "value": 282680.88601116434
          },
          {
            "t": 19.502739,
            "value": 317752.2542097224
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.125575,
            "value": 299056.6391560878
          },
          {
            "t": 1.033725,
            "value": 352364.69746187306
          },
          {
            "t": 2.041882,
            "value": 272774.97453273647
          },
          {
            "t": 3.050251,
            "value": 293543.33582250145
          },
          {
            "t": 4.058495,
            "value": 358048.2502251439
          },
          {
            "t": 5.066843,
            "value": 241979.95136599665
          },
          {
            "t": 6.075845,
            "value": 274528.6927082404
          },
          {
            "t": 7.084748,
            "value": 359796.72971534426
          },
          {
            "t": 8.093671,
            "value": 357807.2855906744
          },
          {
            "t": 9.103335,
            "value": 232750.6972616633
          },
          {
            "t": 10.112027,
            "value": 220087.0037632895
          },
          {
            "t": 11.12057,
            "value": 360916.68872819503
          },
          {
            "t": 12.128961,
            "value": 360971.09157063084
          },
          {
            "t": 13.137939,
            "value": 342921.25299064995
          },
          {
            "t": 14.146688,
            "value": 310285.31378965435
          },
          {
            "t": 15.155371,
            "value": 245865.1528775641
          },
          {
            "t": 16.167377,
            "value": 215413.74260626914
          },
          {
            "t": 17.181798,
            "value": 181384.25762084973
          },
          {
            "t": 18.190228,
            "value": 315341.6697242248
          },
          {
            "t": 19.200134,
            "value": 328743.4672137803
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.033725,
            "value": 2331716.0559110953
          },
          {
            "t": 3.050251,
            "value": 2069717.9208202623
          },
          {
            "t": 5.066843,
            "value": 2145411.1689424533
          },
          {
            "t": 7.084748,
            "value": 2456223.6577044018
          },
          {
            "t": 9.103335,
            "value": 2099803.9717881866
          },
          {
            "t": 11.12057,
            "value": 2164015.595604875
          },
          {
            "t": 13.137939,
            "value": 2617309.9715520563
          },
          {
            "t": 15.155371,
            "value": 2029144.476740728
          },
          {
            "t": 17.181798,
            "value": 1516728.7052531377
          },
          {
            "t": 19.099486,
            "value": 2520263.4630868
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.033725,
            "value": 1678854.7135951235
          },
          {
            "t": 3.050251,
            "value": 1559117.016095999
          },
          {
            "t": 5.066843,
            "value": 1469961.6977554208
          },
          {
            "t": 7.084748,
            "value": 1813804.415966064
          },
          {
            "t": 9.103335,
            "value": 1435965.8513603823
          },
          {
            "t": 11.12057,
            "value": 1656789.6154885278
          },
          {
            "t": 13.137939,
            "value": 1845337.6650478917
          },
          {
            "t": 15.155371,
            "value": 1364303.7287006453
          },
          {
            "t": 17.181798,
            "value": 1070392.8638929506
          },
          {
            "t": 19.099486,
            "value": 1916310.6824467797
          }
        ],
        "ram_mib": [
          {
            "t": 1.033725,
            "value": 1209.78515625
          },
          {
            "t": 3.050251,
            "value": 1726.1171875
          },
          {
            "t": 5.066843,
            "value": 2428.41796875
          },
          {
            "t": 7.084748,
            "value": 2477.10546875
          },
          {
            "t": 9.103335,
            "value": 3254.95703125
          },
          {
            "t": 11.12057,
            "value": 3567.34765625
          },
          {
            "t": 13.137939,
            "value": 3594.8828125
          },
          {
            "t": 15.155371,
            "value": 4571.09375
          },
          {
            "t": 17.181798,
            "value": 5147.19140625
          },
          {
            "t": 19.099486,
            "value": 5152.921875
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
