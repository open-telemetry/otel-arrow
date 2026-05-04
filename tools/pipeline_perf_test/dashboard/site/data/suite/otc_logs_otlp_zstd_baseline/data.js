window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otlp_zstd_baseline"] = {
  "name": "OTC OTLP Baseline w/ Zstd (Logs)",
  "slug": "otc_logs_otlp_zstd_baseline",
  "description": "OpenTelemetry Collector baseline for OTLP logs with zstd compression",
  "meta": {
    "binary": "otc",
    "protocols": [
      "otlp"
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
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 24.89338902486848
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 31.282645040548974
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 630.210546875
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 951.44140625
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 99406.5221351642
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/100k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 99406.5221351642
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000641
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 1346509.548176616
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 1731342.2217960302
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.051564,
            "value": 25.838601747815233
          },
          {
            "t": 3.06272,
            "value": 25.103720349563048
          },
          {
            "t": 5.072718,
            "value": 25.718874296435274
          },
          {
            "t": 7.082858,
            "value": 31.282645040548974
          },
          {
            "t": 9.093343,
            "value": 21.219076154806494
          },
          {
            "t": 11.106279,
            "value": 27.719375390381014
          },
          {
            "t": 13.120137,
            "value": 20.94218886804253
          },
          {
            "t": 15.132516,
            "value": 30.621806853582555
          },
          {
            "t": 17.144517,
            "value": 19.604110275689223
          },
          {
            "t": 19.158935,
            "value": 20.883491271820446
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.045501,
            "value": 99469.1332359199
          },
          {
            "t": 1.051564,
            "value": 99397.35384364598
          },
          {
            "t": 2.057142,
            "value": 99445.29414923556
          },
          {
            "t": 3.06272,
            "value": 99445.29414923556
          },
          {
            "t": 4.067602,
            "value": 99514.17181320791
          },
          {
            "t": 5.072718,
            "value": 99491.00402341622
          },
          {
            "t": 6.077776,
            "value": 99496.74546145595
          },
          {
            "t": 7.082858,
            "value": 99494.36961362357
          },
          {
            "t": 8.088302,
            "value": 99458.54766650355
          },
          {
            "t": 9.093343,
            "value": 99498.42842232306
          },
          {
            "t": 10.099315,
            "value": 99406.34530583356
          },
          {
            "t": 11.106279,
            "value": 99308.41618965524
          },
          {
            "t": 12.113356,
            "value": 99297.27319758073
          },
          {
            "t": 13.120137,
            "value": 99326.46722574225
          },
          {
            "t": 14.126955,
            "value": 99322.81703346584
          },
          {
            "t": 15.132516,
            "value": 99446.97536996762
          },
          {
            "t": 16.137677,
            "value": 99486.54991588413
          },
          {
            "t": 17.144517,
            "value": 99320.64677605181
          },
          {
            "t": 18.151614,
            "value": 99295.30124704969
          },
          {
            "t": 19.158935,
            "value": 99273.2207508828
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.045501,
            "value": 99469.1332359199
          },
          {
            "t": 1.051564,
            "value": 99397.35384364598
          },
          {
            "t": 2.057142,
            "value": 99445.29414923556
          },
          {
            "t": 3.06272,
            "value": 99445.29414923556
          },
          {
            "t": 4.067602,
            "value": 99514.17181320791
          },
          {
            "t": 5.072718,
            "value": 99491.00402341622
          },
          {
            "t": 6.077776,
            "value": 99496.74546145595
          },
          {
            "t": 7.082858,
            "value": 99494.36961362357
          },
          {
            "t": 8.088302,
            "value": 99458.54766650355
          },
          {
            "t": 9.093343,
            "value": 99498.42842232306
          },
          {
            "t": 10.099315,
            "value": 99406.34530583356
          },
          {
            "t": 11.106279,
            "value": 99308.41618965524
          },
          {
            "t": 12.113356,
            "value": 99297.27319758073
          },
          {
            "t": 13.120137,
            "value": 99326.46722574225
          },
          {
            "t": 14.126955,
            "value": 99322.81703346584
          },
          {
            "t": 15.132516,
            "value": 99446.97536996762
          },
          {
            "t": 16.137677,
            "value": 99486.54991588413
          },
          {
            "t": 17.144517,
            "value": 99320.64677605181
          },
          {
            "t": 18.151614,
            "value": 99295.30124704969
          },
          {
            "t": 19.158935,
            "value": 99273.2207508828
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.051564,
            "value": 1733639.2562394352
          },
          {
            "t": 3.06272,
            "value": 1732486.689247378
          },
          {
            "t": 5.072718,
            "value": 1726387.289937602
          },
          {
            "t": 7.082858,
            "value": 1734675.196752465
          },
          {
            "t": 9.093343,
            "value": 1734300.9273881675
          },
          {
            "t": 11.106279,
            "value": 1732062.5196230782
          },
          {
            "t": 13.120137,
            "value": 1731552.572227039
          },
          {
            "t": 15.132516,
            "value": 1724338.2086575143
          },
          {
            "t": 17.144517,
            "value": 1733046.3553447537
          },
          {
            "t": 19.158935,
            "value": 1730933.2025428684
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.051564,
            "value": 1347730.436511882
          },
          {
            "t": 3.06272,
            "value": 1340475.8258434453
          },
          {
            "t": 5.072718,
            "value": 1348812.7848883432
          },
          {
            "t": 7.082858,
            "value": 1349207.020406539
          },
          {
            "t": 9.093343,
            "value": 1348919.7880113504
          },
          {
            "t": 11.106279,
            "value": 1340756.4870418136
          },
          {
            "t": 13.120137,
            "value": 1346801.5123211269
          },
          {
            "t": 15.132516,
            "value": 1347841.037895943
          },
          {
            "t": 17.144517,
            "value": 1348038.0973965717
          },
          {
            "t": 19.158935,
            "value": 1346512.491449143
          }
        ],
        "ram_mib": [
          {
            "t": 1.051564,
            "value": 244.00390625
          },
          {
            "t": 3.06272,
            "value": 318.01953125
          },
          {
            "t": 5.072718,
            "value": 415.03515625
          },
          {
            "t": 7.082858,
            "value": 542.65234375
          },
          {
            "t": 9.093343,
            "value": 546.078125
          },
          {
            "t": 11.106279,
            "value": 712.1640625
          },
          {
            "t": 13.120137,
            "value": 711.99609375
          },
          {
            "t": 15.132516,
            "value": 922.75390625
          },
          {
            "t": 17.144517,
            "value": 937.9609375
          },
          {
            "t": 19.158935,
            "value": 951.44140625
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
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.6578948497772217
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 41.71983576538395
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 55.53215442092154
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 1347.154296875
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 2201.44140625
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 198690.02619989432
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/200k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 202901.0005916429
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000638
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 2708072.077614687
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 3476750.741062317
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.113316,
            "value": 44.53838909541511
          },
          {
            "t": 3.125012,
            "value": 41.11842267406038
          },
          {
            "t": 5.036647,
            "value": 50.41985111662532
          },
          {
            "t": 7.048346,
            "value": 41.83043962848297
          },
          {
            "t": 9.060822,
            "value": 34.935498154981545
          },
          {
            "t": 11.073381,
            "value": 43.59341176470588
          },
          {
            "t": 13.084255,
            "value": 35.179053472649045
          },
          {
            "t": 15.097013,
            "value": 34.587735965453426
          },
          {
            "t": 17.115196,
            "value": 55.53215442092154
          },
          {
            "t": 19.1283,
            "value": 35.46340136054422
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.108175,
            "value": 198741.17340763603
          },
          {
            "t": 1.113316,
            "value": 199971.94423468946
          },
          {
            "t": 2.119291,
            "value": 197818.03722756528
          },
          {
            "t": 3.125012,
            "value": 198862.30873174567
          },
          {
            "t": 4.131395,
            "value": 198731.49685557088
          },
          {
            "t": 5.137072,
            "value": 199865.36432671724
          },
          {
            "t": 6.143104,
            "value": 198800.83337309348
          },
          {
            "t": 7.148874,
            "value": 197858.357278503
          },
          {
            "t": 8.15578,
            "value": 198628.27314565607
          },
          {
            "t": 9.161584,
            "value": 198845.8984056536
          },
          {
            "t": 10.167782,
            "value": 198768.03571464066
          },
          {
            "t": 11.173859,
            "value": 198791.94137228062
          },
          {
            "t": 12.179018,
            "value": 198973.49573550056
          },
          {
            "t": 13.184731,
            "value": 198863.89059304193
          },
          {
            "t": 14.191913,
            "value": 198573.8426620015
          },
          {
            "t": 15.201835,
            "value": 198035.09577967407
          },
          {
            "t": 16.21031,
            "value": 198319.2444036788
          },
          {
            "t": 17.21575,
            "value": 198917.88669637177
          },
          {
            "t": 18.223233,
            "value": 198514.51587768728
          },
          {
            "t": 19.233443,
            "value": 197978.63810494848
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.108175,
            "value": 198741.17340763603
          },
          {
            "t": 1.113316,
            "value": 199971.94423468944
          },
          {
            "t": 2.119291,
            "value": 197818.03722756528
          },
          {
            "t": 3.125012,
            "value": 199856.6202754044
          },
          {
            "t": 4.131395,
            "value": 197737.83937129303
          },
          {
            "t": 5.137072,
            "value": 199865.36432671724
          },
          {
            "t": 6.143104,
            "value": 197806.82920622802
          },
          {
            "t": 7.148874,
            "value": 198852.62038040505
          },
          {
            "t": 8.15578,
            "value": 197635.1317799278
          },
          {
            "t": 9.161584,
            "value": 199840.12789768187
          },
          {
            "t": 10.167782,
            "value": 198768.03571464066
          },
          {
            "t": 11.173859,
            "value": 198791.94137228062
          },
          {
            "t": 12.179018,
            "value": 198973.49573550056
          },
          {
            "t": 13.184731,
            "value": 198863.89059304193
          },
          {
            "t": 14.191913,
            "value": 199566.7118753115
          },
          {
            "t": 15.201835,
            "value": 198035.09577967407
          },
          {
            "t": 16.21031,
            "value": 296487.27038349985
          },
          {
            "t": 17.317129,
            "value": 179794.52828330558
          },
          {
            "t": 18.324031,
            "value": 199622.20752367162
          },
          {
            "t": 19.3343,
            "value": 198956.9114760524
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.113316,
            "value": 3454434.6810176615
          },
          {
            "t": 3.125012,
            "value": 3463295.149963016
          },
          {
            "t": 5.036647,
            "value": 3644408.0590698537
          },
          {
            "t": 7.048346,
            "value": 3462768.038359615
          },
          {
            "t": 9.060822,
            "value": 3452508.2535145762
          },
          {
            "t": 11.073381,
            "value": 3452961.13058052
          },
          {
            "t": 13.084255,
            "value": 3463917.1822799444
          },
          {
            "t": 15.097013,
            "value": 3461766.889014974
          },
          {
            "t": 17.115196,
            "value": 3451335.186155071
          },
          {
            "t": 19.1283,
            "value": 3460112.840667944
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.113316,
            "value": 2698090.7045824584
          },
          {
            "t": 3.125012,
            "value": 2697302.673962666
          },
          {
            "t": 5.036647,
            "value": 2837425.554564548
          },
          {
            "t": 7.048346,
            "value": 2690329.418069005
          },
          {
            "t": 9.060822,
            "value": 2687518.2610873375
          },
          {
            "t": 11.073381,
            "value": 2695710.287251206
          },
          {
            "t": 13.084255,
            "value": 2696716.9499431592
          },
          {
            "t": 15.097013,
            "value": 2696261.5475879367
          },
          {
            "t": 17.115196,
            "value": 2687109.6426835423
          },
          {
            "t": 19.1283,
            "value": 2694255.736415009
          }
        ],
        "ram_mib": [
          {
            "t": 1.113316,
            "value": 540.5234375
          },
          {
            "t": 3.125012,
            "value": 717.24609375
          },
          {
            "t": 5.036647,
            "value": 948.02734375
          },
          {
            "t": 7.048346,
            "value": 1038.78515625
          },
          {
            "t": 9.060822,
            "value": 1254.0
          },
          {
            "t": 11.073381,
            "value": 1382.57421875
          },
          {
            "t": 13.084255,
            "value": 1661.5546875
          },
          {
            "t": 15.097013,
            "value": 1663.37890625
          },
          {
            "t": 17.115196,
            "value": 2064.01171875
          },
          {
            "t": 19.1283,
            "value": 2201.44140625
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
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.6315789222717285
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 54.57788084112407
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 72.00178660049627
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 1734.405078125
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 2777.66796875
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 295368.0996441384
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/300k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 303140.94437161577
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000617
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 4029779.274072771
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 5190885.8501359625
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.062271,
            "value": 54.91652282676674
          },
          {
            "t": 2.079501,
            "value": 57.586139359698684
          },
          {
            "t": 4.098913,
            "value": 63.22985074626865
          },
          {
            "t": 6.121971,
            "value": 51.11185509056839
          },
          {
            "t": 8.145211,
            "value": 44.999501246882794
          },
          {
            "t": 10.065504,
            "value": 72.00178660049627
          },
          {
            "t": 12.08998,
            "value": 46.17164272329794
          },
          {
            "t": 14.115664,
            "value": 62.656810877626704
          },
          {
            "t": 16.136267,
            "value": 47.70757934038581
          },
          {
            "t": 18.151423,
            "value": 45.39711959924859
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.062271,
            "value": 297780.0497292683
          },
          {
            "t": 1.069697,
            "value": 297788.6216952908
          },
          {
            "t": 2.079501,
            "value": 297087.35556603066
          },
          {
            "t": 3.091012,
            "value": 297574.6185656903
          },
          {
            "t": 4.098913,
            "value": 296656.1199959123
          },
          {
            "t": 5.110416,
            "value": 296588.3442757955
          },
          {
            "t": 6.121971,
            "value": 296573.09785429365
          },
          {
            "t": 7.134702,
            "value": 296228.71226416493
          },
          {
            "t": 8.145211,
            "value": 296880.08716399356
          },
          {
            "t": 9.157363,
            "value": 296398.16944490554
          },
          {
            "t": 10.172346,
            "value": 295571.45292088634
          },
          {
            "t": 11.183116,
            "value": 297792.77184720564
          },
          {
            "t": 12.194583,
            "value": 295610.23740764655
          },
          {
            "t": 13.207062,
            "value": 296302.4418284232
          },
          {
            "t": 14.221533,
            "value": 295720.6268094406
          },
          {
            "t": 15.329913,
            "value": 271567.5129468233
          },
          {
            "t": 16.337591,
            "value": 296721.77024803555
          },
          {
            "t": 17.34522,
            "value": 297728.6282947394
          },
          {
            "t": 18.352757,
            "value": 297755.81442666624
          },
          {
            "t": 19.360225,
            "value": 297776.20728400303
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.062271,
            "value": 298772.6498950325
          },
          {
            "t": 1.069697,
            "value": 297788.6216952908
          },
          {
            "t": 2.079501,
            "value": 296097.06438081054
          },
          {
            "t": 3.091012,
            "value": 297574.6185656903
          },
          {
            "t": 4.200208,
            "value": 270466.1755000919
          },
          {
            "t": 5.211751,
            "value": 296576.61612012534
          },
          {
            "t": 6.223194,
            "value": 296605.938248621
          },
          {
            "t": 7.236091,
            "value": 294205.62999001873
          },
          {
            "t": 8.246513,
            "value": 298885.0203182433
          },
          {
            "t": 9.258671,
            "value": 296396.41241782415
          },
          {
            "t": 10.273603,
            "value": 294601.0176051204
          },
          {
            "t": 11.284374,
            "value": 297792.4772277796
          },
          {
            "t": 12.295738,
            "value": 296629.10682998406
          },
          {
            "t": 13.309664,
            "value": 295879.5809556121
          },
          {
            "t": 14.322768,
            "value": 296119.6481308928
          },
          {
            "t": 15.329913,
            "value": 446807.55998391495
          },
          {
            "t": 16.337591,
            "value": 297714.15075053734
          },
          {
            "t": 17.34522,
            "value": 297728.6282947394
          },
          {
            "t": 18.352757,
            "value": 297755.81442666624
          },
          {
            "t": 19.360225,
            "value": 297776.20728400303
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.062271,
            "value": 5175698.290879611
          },
          {
            "t": 2.079501,
            "value": 5161312.790311467
          },
          {
            "t": 4.098913,
            "value": 5171970.850920961
          },
          {
            "t": 6.121971,
            "value": 5162613.7263489235
          },
          {
            "t": 8.145211,
            "value": 5153546.292085961
          },
          {
            "t": 10.065504,
            "value": 5430798.841635104
          },
          {
            "t": 12.08998,
            "value": 5151995.874488016
          },
          {
            "t": 14.115664,
            "value": 5156144.788624484
          },
          {
            "t": 16.136267,
            "value": 5167700.928881131
          },
          {
            "t": 18.151423,
            "value": 5177076.11718398
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.062271,
            "value": 4017500.2767695747
          },
          {
            "t": 2.079501,
            "value": 4012662.41330934
          },
          {
            "t": 4.098913,
            "value": 3995591.7861238816
          },
          {
            "t": 6.121971,
            "value": 4021362.7093242016
          },
          {
            "t": 8.145211,
            "value": 4000220.438504577
          },
          {
            "t": 10.065504,
            "value": 4214948.968725085
          },
          {
            "t": 12.08998,
            "value": 4006439.197105819
          },
          {
            "t": 14.115664,
            "value": 4002481.13723562
          },
          {
            "t": 16.136267,
            "value": 4010912.089113992
          },
          {
            "t": 18.151423,
            "value": 4015673.72451562
          }
        ],
        "ram_mib": [
          {
            "t": 0.062271,
            "value": 497.23046875
          },
          {
            "t": 2.079501,
            "value": 854.3828125
          },
          {
            "t": 4.098913,
            "value": 1168.29296875
          },
          {
            "t": 6.121971,
            "value": 1250.21875
          },
          {
            "t": 8.145211,
            "value": 1567.09375
          },
          {
            "t": 10.065504,
            "value": 2009.3671875
          },
          {
            "t": 12.08998,
            "value": 2087.19140625
          },
          {
            "t": 14.115664,
            "value": 2355.33203125
          },
          {
            "t": 16.136267,
            "value": 2777.2734375
          },
          {
            "t": 18.151423,
            "value": 2777.66796875
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
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -1.3246753215789795
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 74.69362407295283
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 91.62163647720175
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 2534.634765625
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 4303.51171875
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 397132.8452691008
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/400k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 404501.1091636092
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000659
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 5375865.538627962
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 6923200.694897878
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.064416,
            "value": 67.76505948653725
          },
          {
            "t": 3.082962,
            "value": 86.48587570621469
          },
          {
            "t": 5.102965,
            "value": 81.13117684078036
          },
          {
            "t": 7.121143,
            "value": 59.5825172197871
          },
          {
            "t": 9.1395,
            "value": 79.13897114178168
          },
          {
            "t": 11.157719,
            "value": 62.16852996845426
          },
          {
            "t": 13.177383,
            "value": 91.62163647720175
          },
          {
            "t": 15.096801,
            "value": 67.08554307116104
          },
          {
            "t": 17.120942,
            "value": 84.11693081761005
          },
          {
            "t": 19.142174,
            "value": 67.84
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.055587,
            "value": 396319.38190029195
          },
          {
            "t": 1.165328,
            "value": 360444.464068643
          },
          {
            "t": 2.174474,
            "value": 396374.7564772589
          },
          {
            "t": 3.183745,
            "value": 396325.6647619916
          },
          {
            "t": 4.193519,
            "value": 396128.2425572455
          },
          {
            "t": 5.203807,
            "value": 395926.70604817633
          },
          {
            "t": 6.213028,
            "value": 396345.2999888033
          },
          {
            "t": 7.22195,
            "value": 495578.4490773319
          },
          {
            "t": 8.231152,
            "value": 396352.76188513305
          },
          {
            "t": 9.240337,
            "value": 396359.4385568553
          },
          {
            "t": 10.24927,
            "value": 396458.43678420666
          },
          {
            "t": 11.258492,
            "value": 397335.7695333633
          },
          {
            "t": 12.268183,
            "value": 395170.4036185328
          },
          {
            "t": 13.278184,
            "value": 396039.2118423645
          },
          {
            "t": 14.289356,
            "value": 395580.5738291804
          },
          {
            "t": 15.299272,
            "value": 396072.54464727757
          },
          {
            "t": 16.31341,
            "value": 395409.6976940022
          },
          {
            "t": 17.325115,
            "value": 295540.69615154614
          },
          {
            "t": 17.426118,
            "value": 89870.83763215506
          },
          {
            "t": 18.435168,
            "value": 369360.4828827504
          },
          {
            "t": 19.444565,
            "value": 396276.1926179689
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.156608,
            "value": 395516.2924956979
          },
          {
            "t": 1.165328,
            "value": 398524.86319295736
          },
          {
            "t": 2.174474,
            "value": 396374.7564772589
          },
          {
            "t": 3.183745,
            "value": 391371.5939524667
          },
          {
            "t": 4.193519,
            "value": 401079.8455892111
          },
          {
            "t": 5.203807,
            "value": 394936.8892830559
          },
          {
            "t": 6.213028,
            "value": 397336.1632387753
          },
          {
            "t": 7.22195,
            "value": 396462.7592618656
          },
          {
            "t": 8.231152,
            "value": 395361.8799804202
          },
          {
            "t": 9.240337,
            "value": 397350.33715324744
          },
          {
            "t": 10.24927,
            "value": 593696.5090843495
          },
          {
            "t": 11.258492,
            "value": 397335.7695333633
          },
          {
            "t": 12.268183,
            "value": 395170.4036185328
          },
          {
            "t": 13.278184,
            "value": 397029.30987197044
          },
          {
            "t": 14.289356,
            "value": 395580.5738291804
          },
          {
            "t": 15.402496,
            "value": 359343.8381515353
          },
          {
            "t": 16.415,
            "value": 395060.1676635351
          },
          {
            "t": 17.426118,
            "value": 394612.69604536763
          },
          {
            "t": 18.435168,
            "value": 396412.4671720926
          },
          {
            "t": 19.444565,
            "value": 397266.8830995138
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.064416,
            "value": 6890840.268844804
          },
          {
            "t": 3.082962,
            "value": 6892610.32446127
          },
          {
            "t": 5.102965,
            "value": 6885438.784001806
          },
          {
            "t": 7.121143,
            "value": 6891909.435143977
          },
          {
            "t": 9.1395,
            "value": 6890677.912777571
          },
          {
            "t": 11.157719,
            "value": 6894574.870219733
          },
          {
            "t": 13.177383,
            "value": 6891101.688201602
          },
          {
            "t": 15.096801,
            "value": 7257592.145118989
          },
          {
            "t": 17.120942,
            "value": 6868828.801946109
          },
          {
            "t": 19.142174,
            "value": 6868432.718262921
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.064416,
            "value": 5353307.242992969
          },
          {
            "t": 3.082962,
            "value": 5337830.795037616
          },
          {
            "t": 5.102965,
            "value": 5357158.380457851
          },
          {
            "t": 7.121143,
            "value": 5362601.316633122
          },
          {
            "t": 9.1395,
            "value": 5340628.54093701
          },
          {
            "t": 11.157719,
            "value": 5364381.665220671
          },
          {
            "t": 13.177383,
            "value": 5345650.563658113
          },
          {
            "t": 15.096801,
            "value": 5643478.387719611
          },
          {
            "t": 17.120942,
            "value": 5322923.650081689
          },
          {
            "t": 19.142174,
            "value": 5330694.843540969
          }
        ],
        "ram_mib": [
          {
            "t": 1.064416,
            "value": 932.0859375
          },
          {
            "t": 3.082962,
            "value": 1376.4765625
          },
          {
            "t": 5.102965,
            "value": 1723.87890625
          },
          {
            "t": 7.121143,
            "value": 1847.609375
          },
          {
            "t": 9.1395,
            "value": 2435.0625
          },
          {
            "t": 11.157719,
            "value": 2499.390625
          },
          {
            "t": 13.177383,
            "value": 3235.12890625
          },
          {
            "t": 15.096801,
            "value": 3235.3125
          },
          {
            "t": 17.120942,
            "value": 3757.890625
          },
          {
            "t": 19.142174,
            "value": 4303.51171875
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
      "name": "600k",
      "metrics": [
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/600k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 7.420167922973633
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/600k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 99.69140063387209
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/600k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.57697877652933
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/600k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 4381.551171875
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/600k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 6519.875
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/600k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 612831.6861378915
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/600k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 576468.5204600801
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/600k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000711
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/600k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 7444553.76405589
        },
        {
          "extra": "OTC OTLP Baseline w/ Zstd (Logs)/600k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 10257667.018890815
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.110113,
            "value": 99.80523898199876
          },
          {
            "t": 3.124289,
            "value": 99.77724907063197
          },
          {
            "t": 5.138736,
            "value": 99.30904143475571
          },
          {
            "t": 7.154926,
            "value": 99.97268777157046
          },
          {
            "t": 9.121896,
            "value": 98.9091811414392
          },
          {
            "t": 11.137767,
            "value": 99.71435643564357
          },
          {
            "t": 13.1592,
            "value": 99.12128553770087
          },
          {
            "t": 15.174273,
            "value": 100.00377405338298
          },
          {
            "t": 17.191181,
            "value": 99.72421313506815
          },
          {
            "t": 19.214258,
            "value": 100.57697877652933
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.103067,
            "value": 592333.428435756
          },
          {
            "t": 1.210875,
            "value": 541610.0985008233
          },
          {
            "t": 2.217876,
            "value": 595828.6039437895
          },
          {
            "t": 3.225193,
            "value": 595641.689756055
          },
          {
            "t": 4.232381,
            "value": 695004.3090267159
          },
          {
            "t": 5.239721,
            "value": 595628.0898207161
          },
          {
            "t": 6.247631,
            "value": 595291.2462422239
          },
          {
            "t": 7.25638,
            "value": 693928.8167819746
          },
          {
            "t": 8.316398,
            "value": 566028.1240507237
          },
          {
            "t": 9.323309,
            "value": 695195.5038727357
          },
          {
            "t": 10.330599,
            "value": 595657.6556900197
          },
          {
            "t": 11.34451,
            "value": 591767.9165133823
          },
          {
            "t": 12.453416,
            "value": 721431.7534579125
          },
          {
            "t": 13.461408,
            "value": 595242.8193874555
          },
          {
            "t": 14.468514,
            "value": 595766.4833691786
          },
          {
            "t": 15.476377,
            "value": 595319.0066507056
          },
          {
            "t": 16.485456,
            "value": 594601.61196497
          },
          {
            "t": 17.493668,
            "value": 595112.9325975091
          },
          {
            "t": 18.504333,
            "value": 593668.5251789663
          },
          {
            "t": 19.521123,
            "value": 590092.3494526893
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.103067,
            "value": 495585.63512458256
          },
          {
            "t": 1.110113,
            "value": 588850.9561628763
          },
          {
            "t": 2.116772,
            "value": 668548.137949395
          },
          {
            "t": 3.124289,
            "value": 473441.14292860567
          },
          {
            "t": 4.131365,
            "value": 458753.8577028943
          },
          {
            "t": 5.138736,
            "value": 908304.8846949138
          },
          {
            "t": 6.146611,
            "value": 339327.7936252015
          },
          {
            "t": 7.154926,
            "value": 456206.64177365205
          },
          {
            "t": 8.215393,
            "value": 825108.1834701126
          },
          {
            "t": 9.222435,
            "value": 654391.7731335933
          },
          {
            "t": 10.229678,
            "value": 397123.63352239726
          },
          {
            "t": 11.23832,
            "value": 567099.1293243787
          },
          {
            "t": 12.251826,
            "value": 608777.8464064347
          },
          {
            "t": 13.259876,
            "value": 799563.5137145976
          },
          {
            "t": 14.26717,
            "value": 409016.6326812232
          },
          {
            "t": 15.274944,
            "value": 327454.36972972116
          },
          {
            "t": 16.283974,
            "value": 286413.68442960066
          },
          {
            "t": 17.291767,
            "value": 638027.8489729539
          },
          {
            "t": 18.299876,
            "value": 648739.3724289736
          },
          {
            "t": 19.214258,
            "value": 916465.9846759888
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.110113,
            "value": 10089106.425827073
          },
          {
            "t": 3.124289,
            "value": 9350174.95988434
          },
          {
            "t": 5.138736,
            "value": 11195953.529678367
          },
          {
            "t": 7.154926,
            "value": 10403703.024020554
          },
          {
            "t": 9.121896,
            "value": 9995666.93950594
          },
          {
            "t": 11.137767,
            "value": 10033521.490214402
          },
          {
            "t": 13.1592,
            "value": 10901240.357706636
          },
          {
            "t": 15.174273,
            "value": 8545498.35167262
          },
          {
            "t": 17.191181,
            "value": 11125708.758158527
          },
          {
            "t": 19.214258,
            "value": 10936096.352239683
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.110113,
            "value": 6825934.200631786
          },
          {
            "t": 3.124289,
            "value": 7361380.534769554
          },
          {
            "t": 5.138736,
            "value": 9277046.752781283
          },
          {
            "t": 7.154926,
            "value": 6202579.12200735
          },
          {
            "t": 9.121896,
            "value": 9230005.033122009
          },
          {
            "t": 11.137767,
            "value": 5353486.4086045185
          },
          {
            "t": 13.1592,
            "value": 9310255.150677761
          },
          {
            "t": 15.174273,
            "value": 5080027.373698124
          },
          {
            "t": 17.191181,
            "value": 6008272.067937655
          },
          {
            "t": 19.214258,
            "value": 9796550.99632886
          }
        ],
        "ram_mib": [
          {
            "t": 1.110113,
            "value": 1863.2734375
          },
          {
            "t": 3.124289,
            "value": 2567.33203125
          },
          {
            "t": 5.138736,
            "value": 2902.61328125
          },
          {
            "t": 7.154926,
            "value": 3550.54296875
          },
          {
            "t": 9.121896,
            "value": 4152.88671875
          },
          {
            "t": 11.137767,
            "value": 4790.4609375
          },
          {
            "t": 13.1592,
            "value": 4846.41796875
          },
          {
            "t": 15.174273,
            "value": 6102.62109375
          },
          {
            "t": 17.191181,
            "value": 6519.48828125
          },
          {
            "t": 19.214258,
            "value": 6519.875
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
