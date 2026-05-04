window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otap_gzip_baseline"] = {
  "name": "OTC OTAP Baseline w/ Gzip (Logs)",
  "slug": "otc_logs_otap_gzip_baseline",
  "description": "OpenTelemetry Collector baseline for OTAP logs with gzip compression",
  "meta": {
    "binary": "otc",
    "protocols": [
      "otap"
    ],
    "signals": [
      "logs"
    ],
    "compression": "gzip"
  },
  "tests": [
    {
      "name": "100k",
      "metrics": [
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 100.0
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 37.99519272182094
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 39.236116264687695
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 226.09140625
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 226.9765625
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 99459.06309457871
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000708
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 1878050.586070375
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 805229.4174713637
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.090955,
            "value": 38.119750312109865
          },
          {
            "t": 3.101946,
            "value": 38.74915422885572
          },
          {
            "t": 5.112714,
            "value": 37.52134579439252
          },
          {
            "t": 7.122859,
            "value": 37.726169154228856
          },
          {
            "t": 9.133453,
            "value": 37.85821584529008
          },
          {
            "t": 11.144631,
            "value": 39.236116264687695
          },
          {
            "t": 13.155928,
            "value": 39.10082972136223
          },
          {
            "t": 15.16716,
            "value": 37.047706650093225
          },
          {
            "t": 17.178085,
            "value": 37.49091361093847
          },
          {
            "t": 19.189283,
            "value": 37.10172563625078
          }
        ],
        "logs_produced_rate": [
          {
            "t": 1.090955,
            "value": 99501.59650311588
          },
          {
            "t": 2.096417,
            "value": 99456.76713789282
          },
          {
            "t": 3.101946,
            "value": 99450.14017497259
          },
          {
            "t": 4.107013,
            "value": 99495.85450522204
          },
          {
            "t": 5.112714,
            "value": 100427.46303324746
          },
          {
            "t": 6.1177,
            "value": 99503.8736858026
          },
          {
            "t": 7.122859,
            "value": 98491.88038907279
          },
          {
            "t": 8.128034,
            "value": 99485.16427487752
          },
          {
            "t": 9.133453,
            "value": 99461.02072867133
          },
          {
            "t": 10.139629,
            "value": 99386.19088509366
          },
          {
            "t": 11.144631,
            "value": 99502.28954768249
          },
          {
            "t": 12.150355,
            "value": 100425.16634782504
          },
          {
            "t": 13.155928,
            "value": 98451.33073382042
          },
          {
            "t": 14.161,
            "value": 99495.35953643122
          },
          {
            "t": 15.16716,
            "value": 99387.77132861574
          },
          {
            "t": 16.172913,
            "value": 99427.99076910535
          },
          {
            "t": 17.178085,
            "value": 99485.46119470101
          },
          {
            "t": 18.18403,
            "value": 99409.01341524636
          },
          {
            "t": 19.189283,
            "value": 99477.44498151212
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.090955,
            "value": 805458.9985567444
          },
          {
            "t": 3.101946,
            "value": 805175.6571759892
          },
          {
            "t": 5.112714,
            "value": 805239.0927247698
          },
          {
            "t": 7.122859,
            "value": 805562.7827843265
          },
          {
            "t": 9.133453,
            "value": 805290.3768737
          },
          {
            "t": 11.144631,
            "value": 805142.0610209538
          },
          {
            "t": 13.155928,
            "value": 805027.3032774375
          },
          {
            "t": 15.16716,
            "value": 805124.9184579401
          },
          {
            "t": 17.178085,
            "value": 805144.8960055696
          },
          {
            "t": 19.189283,
            "value": 805128.0878362051
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.090955,
            "value": 1878640.6631911122
          },
          {
            "t": 3.101946,
            "value": 1877920.3885049708
          },
          {
            "t": 5.112714,
            "value": 1878075.9391436505
          },
          {
            "t": 7.122859,
            "value": 1878432.6503809425
          },
          {
            "t": 9.133453,
            "value": 1878649.792051503
          },
          {
            "t": 11.144631,
            "value": 1877721.4150115007
          },
          {
            "t": 13.155928,
            "value": 1877477.5679573927
          },
          {
            "t": 15.16716,
            "value": 1877871.8715692668
          },
          {
            "t": 17.178085,
            "value": 1877834.3299725251
          },
          {
            "t": 19.189283,
            "value": 1877881.242920886
          }
        ],
        "ram_mib": [
          {
            "t": 1.090955,
            "value": 225.62109375
          },
          {
            "t": 3.101946,
            "value": 225.01953125
          },
          {
            "t": 5.112714,
            "value": 226.10546875
          },
          {
            "t": 7.122859,
            "value": 225.83203125
          },
          {
            "t": 9.133453,
            "value": 225.58984375
          },
          {
            "t": 11.144631,
            "value": 226.9765625
          },
          {
            "t": 13.155928,
            "value": 225.75390625
          },
          {
            "t": 15.16716,
            "value": 226.64453125
          },
          {
            "t": 17.178085,
            "value": 226.68359375
          },
          {
            "t": 19.189283,
            "value": 226.6875
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
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 100.0
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 68.94575484398825
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 69.8055292652553
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 68.396484375
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 70.18359375
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 203905.07756050496
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000682
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 3758619.709125015
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 1612566.899263531
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.101214,
            "value": 68.82189054726369
          },
          {
            "t": 3.113854,
            "value": 69.55703426791277
          },
          {
            "t": 5.125883,
            "value": 69.37675810473816
          },
          {
            "t": 7.138245,
            "value": 68.13600991325899
          },
          {
            "t": 9.151261,
            "value": 69.22105787181083
          },
          {
            "t": 11.163082,
            "value": 69.8055292652553
          },
          {
            "t": 13.175433,
            "value": 68.54205096333126
          },
          {
            "t": 15.189403,
            "value": 69.2960398505604
          },
          {
            "t": 17.103631,
            "value": 68.62306351183064
          },
          {
            "t": 19.120472,
            "value": 68.0781141439206
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.09363,
            "value": 198759.9367545881
          },
          {
            "t": 1.101214,
            "value": 198494.61682599166
          },
          {
            "t": 2.107265,
            "value": 198797.078875723
          },
          {
            "t": 3.113854,
            "value": 198690.8261465206
          },
          {
            "t": 4.119616,
            "value": 198854.2020875714
          },
          {
            "t": 5.125883,
            "value": 198754.40613674105
          },
          {
            "t": 6.132408,
            "value": 198703.45992399595
          },
          {
            "t": 7.138245,
            "value": 198839.37457063122
          },
          {
            "t": 8.144582,
            "value": 198740.58093859215
          },
          {
            "t": 9.151261,
            "value": 198673.06261479575
          },
          {
            "t": 10.157164,
            "value": 198826.32818472557
          },
          {
            "t": 11.163082,
            "value": 198823.36333577885
          },
          {
            "t": 12.169388,
            "value": 198746.7032890592
          },
          {
            "t": 13.175433,
            "value": 198798.26449115097
          },
          {
            "t": 14.181901,
            "value": 198714.71323479735
          },
          {
            "t": 15.189403,
            "value": 198510.77218705276
          },
          {
            "t": 16.197441,
            "value": 297607.82827631495
          },
          {
            "t": 17.20413,
            "value": 198671.0890851097
          },
          {
            "t": 18.215231,
            "value": 197804.17584395624
          },
          {
            "t": 19.225081,
            "value": 199039.4613061346
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.101214,
            "value": 1604827.2365545533
          },
          {
            "t": 3.113854,
            "value": 1606084.5456713568
          },
          {
            "t": 5.125883,
            "value": 1602601.6523618696
          },
          {
            "t": 7.138245,
            "value": 1606068.3912735383
          },
          {
            "t": 9.151261,
            "value": 1601658.90385372
          },
          {
            "t": 11.163082,
            "value": 1606327.3024786997
          },
          {
            "t": 13.175433,
            "value": 1606246.1270424495
          },
          {
            "t": 15.189403,
            "value": 1604990.143845241
          },
          {
            "t": 17.103631,
            "value": 1684459.7404279949
          },
          {
            "t": 19.120472,
            "value": 1602404.9491258855
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.101214,
            "value": 3740761.725335345
          },
          {
            "t": 3.113854,
            "value": 3743331.6440098574
          },
          {
            "t": 5.125883,
            "value": 3744711.930096435
          },
          {
            "t": 7.138245,
            "value": 3734449.3684535883
          },
          {
            "t": 9.151261,
            "value": 3742401.9481216245
          },
          {
            "t": 11.163082,
            "value": 3744469.3141189003
          },
          {
            "t": 13.175433,
            "value": 3734751.5418532854
          },
          {
            "t": 15.189403,
            "value": 3740313.410825385
          },
          {
            "t": 17.103631,
            "value": 3926085.6073571173
          },
          {
            "t": 19.120472,
            "value": 3734920.601078618
          }
        ],
        "ram_mib": [
          {
            "t": 1.101214,
            "value": 67.6171875
          },
          {
            "t": 3.113854,
            "value": 69.70703125
          },
          {
            "t": 5.125883,
            "value": 69.26171875
          },
          {
            "t": 7.138245,
            "value": 68.0703125
          },
          {
            "t": 9.151261,
            "value": 69.140625
          },
          {
            "t": 11.163082,
            "value": 68.25390625
          },
          {
            "t": 13.175433,
            "value": 67.98828125
          },
          {
            "t": 15.189403,
            "value": 70.18359375
          },
          {
            "t": 17.103631,
            "value": 67.4375
          },
          {
            "t": 19.120472,
            "value": 66.3046875
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
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 100.0
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 97.37232772122555
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 98.1309717097171
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 74.651171875
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 77.109375
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 295847.6382586844
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000739
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 5621821.826438785
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 2408147.422314248
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.098389,
            "value": 98.09632452366318
          },
          {
            "t": 3.11421,
            "value": 96.92915899324738
          },
          {
            "t": 5.12927,
            "value": 97.28757687576875
          },
          {
            "t": 7.147829,
            "value": 96.63921133703019
          },
          {
            "t": 9.069637,
            "value": 97.71123997532388
          },
          {
            "t": 11.091726,
            "value": 96.92071209330878
          },
          {
            "t": 13.10747,
            "value": 96.70396063960641
          },
          {
            "t": 15.122745,
            "value": 98.1309717097171
          },
          {
            "t": 17.139365,
            "value": 98.01425061425061
          },
          {
            "t": 19.155464,
            "value": 97.2898704503393
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.09027,
            "value": 297833.8543771152
          },
          {
            "t": 1.098389,
            "value": 298575.86257177975
          },
          {
            "t": 2.106214,
            "value": 296678.49080941634
          },
          {
            "t": 3.11421,
            "value": 298612.29608054005
          },
          {
            "t": 4.12188,
            "value": 296724.1259539333
          },
          {
            "t": 5.12927,
            "value": 297799.2634431551
          },
          {
            "t": 6.139126,
            "value": 298062.29799100064
          },
          {
            "t": 7.147829,
            "value": 297411.626613582
          },
          {
            "t": 8.161414,
            "value": 294992.526527129
          },
          {
            "t": 9.172142,
            "value": 297805.14638953307
          },
          {
            "t": 10.18496,
            "value": 196481.50013131677
          },
          {
            "t": 10.285777,
            "value": 89796.02832166734
          },
          {
            "t": 11.293139,
            "value": 279745.4390940162
          },
          {
            "t": 12.301078,
            "value": 298629.18291682337
          },
          {
            "t": 13.308999,
            "value": 296650.23350044293
          },
          {
            "t": 14.31651,
            "value": 297763.4983637895
          },
          {
            "t": 15.324022,
            "value": 297763.20282041305
          },
          {
            "t": 16.332582,
            "value": 297453.79551043076
          },
          {
            "t": 17.340956,
            "value": 297508.6624605553
          },
          {
            "t": 18.349185,
            "value": 298543.28728889965
          },
          {
            "t": 19.356944,
            "value": 296697.9208322625
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.098389,
            "value": 2394246.379860593
          },
          {
            "t": 3.11421,
            "value": 2406151.141395987
          },
          {
            "t": 5.12927,
            "value": 2391160.064712713
          },
          {
            "t": 7.147829,
            "value": 2395515.8110315325
          },
          {
            "t": 9.069637,
            "value": 2513819.7988560773
          },
          {
            "t": 11.091726,
            "value": 2391147.4717482766
          },
          {
            "t": 13.10747,
            "value": 2398071.8781750062
          },
          {
            "t": 15.122745,
            "value": 2401980.8710969966
          },
          {
            "t": 17.139365,
            "value": 2395673.453600579
          },
          {
            "t": 19.155464,
            "value": 2393707.3526647254
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.098389,
            "value": 5597951.663993903
          },
          {
            "t": 3.11421,
            "value": 5607069.774548435
          },
          {
            "t": 5.12927,
            "value": 5598863.061149544
          },
          {
            "t": 7.147829,
            "value": 5582162.820110781
          },
          {
            "t": 9.069637,
            "value": 5870256.5500820065
          },
          {
            "t": 11.091726,
            "value": 5579844.408431084
          },
          {
            "t": 13.10747,
            "value": 5606486.24031623
          },
          {
            "t": 15.122745,
            "value": 5569042.934587091
          },
          {
            "t": 17.139365,
            "value": 5601299.699497178
          },
          {
            "t": 19.155464,
            "value": 5605241.1116716
          }
        ],
        "ram_mib": [
          {
            "t": 1.098389,
            "value": 73.4375
          },
          {
            "t": 3.11421,
            "value": 73.18359375
          },
          {
            "t": 5.12927,
            "value": 71.5859375
          },
          {
            "t": 7.147829,
            "value": 77.109375
          },
          {
            "t": 9.069637,
            "value": 74.8984375
          },
          {
            "t": 11.091726,
            "value": 74.16015625
          },
          {
            "t": 13.10747,
            "value": 74.6484375
          },
          {
            "t": 15.122745,
            "value": 75.6484375
          },
          {
            "t": 17.139365,
            "value": 76.8359375
          },
          {
            "t": 19.155464,
            "value": 75.00390625
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
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 100.0
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.08774033370933
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.56194878201123
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 110.9390625
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 126.94140625
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 321735.0349312324
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000619
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 5756682.380426457
        },
        {
          "extra": "OTC OTAP Baseline w/ Gzip (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 2464833.1819047076
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.080821,
            "value": 100.56194878201123
          },
          {
            "t": 3.114785,
            "value": 99.9731343283582
          },
          {
            "t": 5.140041,
            "value": 100.46373050530255
          },
          {
            "t": 7.067241,
            "value": 99.97336645962733
          },
          {
            "t": 9.096749,
            "value": 99.75895716945996
          },
          {
            "t": 11.13081,
            "value": 100.07009334163037
          },
          {
            "t": 13.16272,
            "value": 100.2504607721046
          },
          {
            "t": 15.091272,
            "value": 99.76
          },
          {
            "t": 17.111374,
            "value": 99.99411471321696
          },
          {
            "t": 19.129845,
            "value": 100.07159726538222
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.069235,
            "value": 394960.73654733045
          },
          {
            "t": 1.080821,
            "value": 395418.6791829859
          },
          {
            "t": 2.104346,
            "value": 353679.6854009428
          },
          {
            "t": 3.114785,
            "value": 318673.36870409787
          },
          {
            "t": 4.12705,
            "value": 314146.98720196786
          },
          {
            "t": 5.140041,
            "value": 316883.36816417915
          },
          {
            "t": 6.158405,
            "value": 313247.5224968675
          },
          {
            "t": 7.169816,
            "value": 313423.52416574466
          },
          {
            "t": 8.188825,
            "value": 315993.2836707036
          },
          {
            "t": 9.199721,
            "value": 316550.8618097213
          },
          {
            "t": 10.220099,
            "value": 318509.4151383115
          },
          {
            "t": 11.237224,
            "value": 324443.8982425955
          },
          {
            "t": 12.254128,
            "value": 314680.6384870155
          },
          {
            "t": 13.271653,
            "value": 318419.69484779245
          },
          {
            "t": 14.283741,
            "value": 302345.2506106188
          },
          {
            "t": 15.393337,
            "value": 267664.98797760624
          },
          {
            "t": 16.40332,
            "value": 308916.09066687257
          },
          {
            "t": 17.413874,
            "value": 315668.4353334904
          },
          {
            "t": 18.422984,
            "value": 312156.25650325534
          },
          {
            "t": 19.432995,
            "value": 377223.61439627886
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.080821,
            "value": 2360527.902432124
          },
          {
            "t": 3.114785,
            "value": 2530049.2044106973
          },
          {
            "t": 5.140041,
            "value": 2427875.290827431
          },
          {
            "t": 7.067241,
            "value": 2531791.199667912
          },
          {
            "t": 9.096749,
            "value": 2465729.132380853
          },
          {
            "t": 11.13081,
            "value": 2400606.9631146756
          },
          {
            "t": 13.16272,
            "value": 2390718.092828915
          },
          {
            "t": 15.091272,
            "value": 2594488.5074397787
          },
          {
            "t": 17.111374,
            "value": 2418335.806805795
          },
          {
            "t": 19.129845,
            "value": 2528209.719138893
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.080821,
            "value": 5548184.228709579
          },
          {
            "t": 3.114785,
            "value": 5829632.677864505
          },
          {
            "t": 5.140041,
            "value": 5643312.746635486
          },
          {
            "t": 7.067241,
            "value": 5924595.786633458
          },
          {
            "t": 9.096749,
            "value": 5713783.833323151
          },
          {
            "t": 11.13081,
            "value": 5626155.2627969375
          },
          {
            "t": 13.16272,
            "value": 5660557.800296273
          },
          {
            "t": 15.091272,
            "value": 6018675.151097818
          },
          {
            "t": 17.111374,
            "value": 5706428.6852842085
          },
          {
            "t": 19.129845,
            "value": 5895497.631623145
          }
        ],
        "ram_mib": [
          {
            "t": 1.080821,
            "value": 125.02734375
          },
          {
            "t": 3.114785,
            "value": 119.0859375
          },
          {
            "t": 5.140041,
            "value": 102.6796875
          },
          {
            "t": 7.067241,
            "value": 107.84765625
          },
          {
            "t": 9.096749,
            "value": 96.83984375
          },
          {
            "t": 11.13081,
            "value": 126.94140625
          },
          {
            "t": 13.16272,
            "value": 102.45703125
          },
          {
            "t": 15.091272,
            "value": 107.0078125
          },
          {
            "t": 17.111374,
            "value": 111.63671875
          },
          {
            "t": 19.129845,
            "value": 109.8671875
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
