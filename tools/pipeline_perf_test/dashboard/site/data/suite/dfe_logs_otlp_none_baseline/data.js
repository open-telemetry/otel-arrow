window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otlp_none_baseline"] = {
  "name": "DFE OTLP Baseline (Logs)",
  "slug": "dfe_logs_otlp_none_baseline",
  "description": "Dataflow Engine baseline for OTLP logs with no compression",
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
  "tests": [
    {
      "name": "1000k",
      "metrics": [
        {
          "extra": "DFE OTLP Baseline (Logs)/1000k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 1.5440415143966675
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/1000k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 34.356110586802465
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/1000k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 34.913069306930694
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/1000k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 13.99140625
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/1000k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 14.67578125
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/1000k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 984091.7740533445
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/1000k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 989683.6855941284
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/1000k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000677
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/1000k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 359299431.9050686
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/1000k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 359064835.3034392
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.098778,
            "value": 34.82423645320197
          },
          {
            "t": 2.139287,
            "value": 34.22525798525798
          },
          {
            "t": 4.086896,
            "value": 33.669672637430516
          },
          {
            "t": 6.125316,
            "value": 33.87975278121137
          },
          {
            "t": 8.166793,
            "value": 34.72495356037152
          },
          {
            "t": 10.105182,
            "value": 34.64746268656717
          },
          {
            "t": 12.154726,
            "value": 34.12650931677019
          },
          {
            "t": 14.196243,
            "value": 34.913069306930694
          },
          {
            "t": 16.134099,
            "value": 33.63945611866502
          },
          {
            "t": 18.17569,
            "value": 34.91073502161829
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.302869,
            "value": 978562.6284975051
          },
          {
            "t": 1.322556,
            "value": 980693.0950379871
          },
          {
            "t": 2.34877,
            "value": 974455.6203676817
          },
          {
            "t": 3.373246,
            "value": 488054.38097134535
          },
          {
            "t": 3.47535,
            "value": 443821.1223348542
          },
          {
            "t": 4.494087,
            "value": 936897.4119587163
          },
          {
            "t": 5.513391,
            "value": 981061.5871221932
          },
          {
            "t": 6.533174,
            "value": 980600.7748707323
          },
          {
            "t": 7.554479,
            "value": 979139.434351149
          },
          {
            "t": 8.574457,
            "value": 980413.3030320262
          },
          {
            "t": 9.593965,
            "value": 980865.2801155066
          },
          {
            "t": 10.619643,
            "value": 1072461.3377687733
          },
          {
            "t": 11.644307,
            "value": 780743.7364833739
          },
          {
            "t": 11.746618,
            "value": 177466.22595887218
          },
          {
            "t": 12.765272,
            "value": 910008.3303215258
          },
          {
            "t": 13.787056,
            "value": 978680.4256085434
          },
          {
            "t": 14.807113,
            "value": 980337.3733036488
          },
          {
            "t": 15.82705,
            "value": 980452.7142362715
          },
          {
            "t": 16.846819,
            "value": 1176737.084575036
          },
          {
            "t": 17.866973,
            "value": 980244.1592151775
          },
          {
            "t": 18.889975,
            "value": 977515.1954737138
          },
          {
            "t": 19.914861,
            "value": 975718.2750081471
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.098778,
            "value": 977328.7131153797
          },
          {
            "t": 1.118287,
            "value": 982826.0466557922
          },
          {
            "t": 2.139287,
            "value": 978452.4975514202
          },
          {
            "t": 3.163919,
            "value": 975960.1495951718
          },
          {
            "t": 4.188631,
            "value": 974908.0717313742
          },
          {
            "t": 5.106183,
            "value": 1089856.4876977
          },
          {
            "t": 6.125316,
            "value": 981226.1991320072
          },
          {
            "t": 7.145296,
            "value": 979430.9692346908
          },
          {
            "t": 8.166793,
            "value": 979934.3512511539
          },
          {
            "t": 9.186676,
            "value": 980504.6265110801
          },
          {
            "t": 10.206871,
            "value": 982165.1743049123
          },
          {
            "t": 11.231869,
            "value": 974636.0480703376
          },
          {
            "t": 12.154726,
            "value": 1081424.316009956
          },
          {
            "t": 13.176532,
            "value": 978659.3541239727
          },
          {
            "t": 14.196243,
            "value": 981650.6833798988
          },
          {
            "t": 15.215881,
            "value": 981720.9637145732
          },
          {
            "t": 16.236051,
            "value": 980228.785398512
          },
          {
            "t": 17.256332,
            "value": 978161.8985357955
          },
          {
            "t": 18.277318,
            "value": 980424.8050414012
          },
          {
            "t": 19.298852,
            "value": 980877.7779300542
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.098778,
            "value": 353247900.9240045
          },
          {
            "t": 2.139287,
            "value": 354077791.3746031
          },
          {
            "t": 4.086896,
            "value": 370962192.10324043
          },
          {
            "t": 6.125316,
            "value": 354108188.20458984
          },
          {
            "t": 8.166793,
            "value": 353573907.51891893
          },
          {
            "t": 10.105182,
            "value": 372723198.49111813
          },
          {
            "t": 12.154726,
            "value": 352519225.7399695
          },
          {
            "t": 14.196243,
            "value": 353189594.7964186
          },
          {
            "t": 16.134099,
            "value": 372668654.4304634
          },
          {
            "t": 18.17569,
            "value": 353577699.45106536
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.098778,
            "value": 353494390.39191884
          },
          {
            "t": 2.139287,
            "value": 354337412.8710042
          },
          {
            "t": 4.086896,
            "value": 371209220.63925564
          },
          {
            "t": 6.125316,
            "value": 354344006.14201194
          },
          {
            "t": 8.166793,
            "value": 353811492.3655765
          },
          {
            "t": 10.105182,
            "value": 372956148.1209396
          },
          {
            "t": 12.154726,
            "value": 352781946.1304563
          },
          {
            "t": 14.196243,
            "value": 353445135.65157676
          },
          {
            "t": 16.134099,
            "value": 372895058.7659764
          },
          {
            "t": 18.17569,
            "value": 353719507.97196895
          }
        ],
        "ram_mib": [
          {
            "t": 0.098778,
            "value": 13.80859375
          },
          {
            "t": 2.139287,
            "value": 13.94140625
          },
          {
            "t": 4.086896,
            "value": 13.62890625
          },
          {
            "t": 6.125316,
            "value": 14.67578125
          },
          {
            "t": 8.166793,
            "value": 14.609375
          },
          {
            "t": 10.105182,
            "value": 14.0703125
          },
          {
            "t": 12.154726,
            "value": 13.765625
          },
          {
            "t": 14.196243,
            "value": 13.4609375
          },
          {
            "t": 16.134099,
            "value": 13.99609375
          },
          {
            "t": 18.17569,
            "value": 13.95703125
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
          "extra": "DFE OTLP Baseline (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.6315789222717285
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 6.209697830472468
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 6.736947040498443
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 10.674609375
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 10.97265625
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 99072.05980198103
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/100k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 101679.21927045421
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000574
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 36027404.36740791
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 35970306.17443672
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.072587,
            "value": 5.800545229244114
          },
          {
            "t": 2.08382,
            "value": 6.498032378580324
          },
          {
            "t": 4.096607,
            "value": 5.81908302354399
          },
          {
            "t": 6.109045,
            "value": 5.736575682382134
          },
          {
            "t": 8.121841,
            "value": 5.9421614906832305
          },
          {
            "t": 10.139543,
            "value": 6.214292803970223
          },
          {
            "t": 12.153826,
            "value": 6.494468614045991
          },
          {
            "t": 14.169225,
            "value": 6.461439205955336
          },
          {
            "t": 16.187361,
            "value": 6.393432835820896
          },
          {
            "t": 18.144121,
            "value": 6.736947040498443
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.072587,
            "value": 99334.85381882911
          },
          {
            "t": 1.07798,
            "value": 99463.59284379342
          },
          {
            "t": 2.08382,
            "value": 99419.39075797342
          },
          {
            "t": 3.090267,
            "value": 99359.42975636074
          },
          {
            "t": 4.096607,
            "value": 99369.99423654033
          },
          {
            "t": 5.10229,
            "value": 99434.91139852219
          },
          {
            "t": 6.109045,
            "value": 99329.032386231
          },
          {
            "t": 7.115802,
            "value": 99328.83506148953
          },
          {
            "t": 8.121841,
            "value": 99399.72506036049
          },
          {
            "t": 9.129396,
            "value": 99250.16500339932
          },
          {
            "t": 10.139543,
            "value": 98995.49273521577
          },
          {
            "t": 11.14589,
            "value": 99369.30303364544
          },
          {
            "t": 12.153826,
            "value": 99212.64842212205
          },
          {
            "t": 13.16298,
            "value": 99092.90356080439
          },
          {
            "t": 14.169225,
            "value": 99379.37579814061
          },
          {
            "t": 15.177334,
            "value": 99195.6226955617
          },
          {
            "t": 16.187361,
            "value": 99007.25426151975
          },
          {
            "t": 17.238456,
            "value": 95138.87897858901
          },
          {
            "t": 18.244631,
            "value": 99386.28966134122
          },
          {
            "t": 19.250547,
            "value": 99411.87932193145
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.072587,
            "value": 99334.85381882911
          },
          {
            "t": 1.07798,
            "value": 99463.59284379342
          },
          {
            "t": 2.08382,
            "value": 99419.39075797342
          },
          {
            "t": 3.090267,
            "value": 99359.42975636074
          },
          {
            "t": 4.096607,
            "value": 99369.99423654033
          },
          {
            "t": 5.10229,
            "value": 99434.91139852219
          },
          {
            "t": 6.109045,
            "value": 99329.032386231
          },
          {
            "t": 7.115802,
            "value": 99328.83506148953
          },
          {
            "t": 8.121841,
            "value": 99399.72506036049
          },
          {
            "t": 9.129396,
            "value": 99250.16500339932
          },
          {
            "t": 10.139543,
            "value": 98995.49273521577
          },
          {
            "t": 11.14589,
            "value": 99369.30303364544
          },
          {
            "t": 12.153826,
            "value": 100204.77490634327
          },
          {
            "t": 13.16298,
            "value": 98101.97452519635
          },
          {
            "t": 14.169225,
            "value": 99379.37579814061
          },
          {
            "t": 15.177334,
            "value": 99195.6226955617
          },
          {
            "t": 16.187361,
            "value": 99007.25426151975
          },
          {
            "t": 17.238456,
            "value": 95138.87897858901
          },
          {
            "t": 18.244631,
            "value": 150073.29738862524
          },
          {
            "t": 19.250547,
            "value": 98417.76052871214
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.072587,
            "value": 35895434.07996083
          },
          {
            "t": 2.08382,
            "value": 35940811.9297963
          },
          {
            "t": 4.096607,
            "value": 35912667.858049564
          },
          {
            "t": 6.109045,
            "value": 35919087.19672358
          },
          {
            "t": 8.121841,
            "value": 35736499.37698605
          },
          {
            "t": 10.139543,
            "value": 35825078.232563585
          },
          {
            "t": 12.153826,
            "value": 35880040.19296197
          },
          {
            "t": 14.169225,
            "value": 35861100.45703109
          },
          {
            "t": 16.187361,
            "value": 35796934.398871034
          },
          {
            "t": 18.144121,
            "value": 36935408.02142317
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.072587,
            "value": 35950746.12265004
          },
          {
            "t": 2.08382,
            "value": 35998420.37198077
          },
          {
            "t": 4.096607,
            "value": 35968256.45237176
          },
          {
            "t": 6.109045,
            "value": 35974732.140816264
          },
          {
            "t": 8.121841,
            "value": 35788056.51442074
          },
          {
            "t": 10.139543,
            "value": 35878007.75337488
          },
          {
            "t": 12.153826,
            "value": 35940261.12517457
          },
          {
            "t": 14.169225,
            "value": 35918978.822555736
          },
          {
            "t": 16.187361,
            "value": 35860547.554773316
          },
          {
            "t": 18.144121,
            "value": 36996036.81596108
          }
        ],
        "ram_mib": [
          {
            "t": 0.072587,
            "value": 10.71484375
          },
          {
            "t": 2.08382,
            "value": 10.6640625
          },
          {
            "t": 4.096607,
            "value": 10.62890625
          },
          {
            "t": 6.109045,
            "value": 10.58203125
          },
          {
            "t": 8.121841,
            "value": 10.97265625
          },
          {
            "t": 10.139543,
            "value": 10.71484375
          },
          {
            "t": 12.153826,
            "value": 10.36328125
          },
          {
            "t": 14.169225,
            "value": 10.67578125
          },
          {
            "t": 16.187361,
            "value": 10.67578125
          },
          {
            "t": 18.144121,
            "value": 10.75390625
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
          "extra": "DFE OTLP Baseline (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.6315789222717285
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 9.853429781744353
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 10.485700000000001
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 12.427734375
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 13.0
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 198288.3126676906
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/200k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 202432.85886012547
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000706
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 72001802.05828768
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 71973165.48861563
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.122044,
            "value": 9.651732332707942
          },
          {
            "t": 3.13534,
            "value": 9.793437305053025
          },
          {
            "t": 5.149517,
            "value": 10.485700000000001
          },
          {
            "t": 7.164814,
            "value": 10.1373
          },
          {
            "t": 9.1839,
            "value": 10.0409
          },
          {
            "t": 11.103412,
            "value": 9.505941213258287
          },
          {
            "t": 13.118079,
            "value": 9.572431077694235
          },
          {
            "t": 15.138483,
            "value": 9.66158452900811
          },
          {
            "t": 17.157236,
            "value": 9.574452097683157
          },
          {
            "t": 19.176281,
            "value": 10.110819262038774
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.114525,
            "value": 198478.85803204245
          },
          {
            "t": 1.122044,
            "value": 198507.42268880288
          },
          {
            "t": 2.128745,
            "value": 198668.72090124077
          },
          {
            "t": 3.13534,
            "value": 198689.64181224824
          },
          {
            "t": 4.142282,
            "value": 198621.17182518952
          },
          {
            "t": 5.149517,
            "value": 198563.39384552758
          },
          {
            "t": 6.158038,
            "value": 198310.19879605877
          },
          {
            "t": 7.164814,
            "value": 198653.92103109334
          },
          {
            "t": 8.173355,
            "value": 198306.26618055193
          },
          {
            "t": 9.1839,
            "value": 197913.00733762473
          },
          {
            "t": 10.196662,
            "value": 197479.76326125982
          },
          {
            "t": 11.204568,
            "value": 198431.20290979516
          },
          {
            "t": 12.211722,
            "value": 198579.36323541382
          },
          {
            "t": 13.221401,
            "value": 198082.75699504497
          },
          {
            "t": 14.232024,
            "value": 197897.73238883342
          },
          {
            "t": 15.241192,
            "value": 198183.05772676104
          },
          {
            "t": 16.250867,
            "value": 198083.5417337262
          },
          {
            "t": 17.260072,
            "value": 198175.79183614827
          },
          {
            "t": 18.269557,
            "value": 198120.82398450695
          },
          {
            "t": 19.278539,
            "value": 198219.59162799732
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.114525,
            "value": 198478.85803204245
          },
          {
            "t": 1.122044,
            "value": 198507.42268880288
          },
          {
            "t": 2.128745,
            "value": 198668.72090124077
          },
          {
            "t": 3.13534,
            "value": 198689.64181224824
          },
          {
            "t": 4.142282,
            "value": 198621.17182518952
          },
          {
            "t": 5.149517,
            "value": 198563.39384552758
          },
          {
            "t": 6.158038,
            "value": 198310.19879605877
          },
          {
            "t": 7.265945,
            "value": 180520.5671595179
          },
          {
            "t": 8.274484,
            "value": 198306.6594350838
          },
          {
            "t": 9.285065,
            "value": 197905.95706826073
          },
          {
            "t": 10.297891,
            "value": 197467.28460762263
          },
          {
            "t": 11.305458,
            "value": 198497.9658920945
          },
          {
            "t": 12.312543,
            "value": 198592.96881593906
          },
          {
            "t": 13.322389,
            "value": 198049.999702925
          },
          {
            "t": 14.333019,
            "value": 197896.36167539062
          },
          {
            "t": 15.342116,
            "value": 198197.00187395266
          },
          {
            "t": 16.351701,
            "value": 297151.7999970285
          },
          {
            "t": 17.360889,
            "value": 198179.13015216193
          },
          {
            "t": 18.370649,
            "value": 198066.8673744256
          },
          {
            "t": 19.380172,
            "value": 198113.36641166176
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.122044,
            "value": 71735759.97812606
          },
          {
            "t": 3.13534,
            "value": 71812047.50816572
          },
          {
            "t": 5.149517,
            "value": 71768338.13512914
          },
          {
            "t": 7.164814,
            "value": 71724930.37006457
          },
          {
            "t": 9.1839,
            "value": 71607878.51532821
          },
          {
            "t": 11.103412,
            "value": 74946350.94753249
          },
          {
            "t": 13.118079,
            "value": 71755962.15156151
          },
          {
            "t": 15.138483,
            "value": 71541730.26780783
          },
          {
            "t": 17.157236,
            "value": 71603587.95751636
          },
          {
            "t": 19.176281,
            "value": 71235069.05492447
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.122044,
            "value": 71771661.92847002
          },
          {
            "t": 3.13534,
            "value": 71823546.56245281
          },
          {
            "t": 5.149517,
            "value": 71807384.35599254
          },
          {
            "t": 7.164814,
            "value": 71755421.16124819
          },
          {
            "t": 9.1839,
            "value": 71263288.43843204
          },
          {
            "t": 11.103412,
            "value": 75335464.43054275
          },
          {
            "t": 13.118079,
            "value": 71776685.675598
          },
          {
            "t": 15.138483,
            "value": 71578124.47411507
          },
          {
            "t": 17.157236,
            "value": 71634690.3261568
          },
          {
            "t": 19.176281,
            "value": 71271753.22986858
          }
        ],
        "ram_mib": [
          {
            "t": 1.122044,
            "value": 12.578125
          },
          {
            "t": 3.13534,
            "value": 12.27734375
          },
          {
            "t": 5.149517,
            "value": 12.640625
          },
          {
            "t": 7.164814,
            "value": 13.0
          },
          {
            "t": 9.1839,
            "value": 12.6953125
          },
          {
            "t": 11.103412,
            "value": 12.3203125
          },
          {
            "t": 13.118079,
            "value": 11.80859375
          },
          {
            "t": 15.138483,
            "value": 12.3515625
          },
          {
            "t": 17.157236,
            "value": 11.890625
          },
          {
            "t": 19.176281,
            "value": 12.71484375
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
          "extra": "DFE OTLP Baseline (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.8306493163108826
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 14.04652149484224
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 15.930515970515971
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 11.037890625
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 11.203125
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 307333.65624467446
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/300k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 304780.79149539676
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000577
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 107965275.73540445
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 107871353.78904602
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.065159,
            "value": 14.70741241548863
          },
          {
            "t": 2.08332,
            "value": 13.627770228536134
          },
          {
            "t": 4.102499,
            "value": 15.930515970515971
          },
          {
            "t": 6.120243,
            "value": 13.450521091811416
          },
          {
            "t": 8.136909,
            "value": 13.629538461538463
          },
          {
            "t": 10.155918,
            "value": 13.988571428571428
          },
          {
            "t": 12.173055,
            "value": 13.211061728395062
          },
          {
            "t": 14.193063,
            "value": 14.314204334365325
          },
          {
            "t": 16.116329,
            "value": 13.755334987593052
          },
          {
            "t": 18.142912,
            "value": 13.850284301606921
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.165918,
            "value": 298444.71633870795
          },
          {
            "t": 1.17501,
            "value": 296305.9859755106
          },
          {
            "t": 2.184081,
            "value": 297303.1630083512
          },
          {
            "t": 3.193147,
            "value": 297304.6361684964
          },
          {
            "t": 4.203283,
            "value": 296989.7122763668
          },
          {
            "t": 5.211881,
            "value": 297442.5886230192
          },
          {
            "t": 6.220986,
            "value": 297293.1459065211
          },
          {
            "t": 7.229213,
            "value": 297552.0393720858
          },
          {
            "t": 8.237641,
            "value": 297492.7312609328
          },
          {
            "t": 9.24597,
            "value": 297521.93976370804
          },
          {
            "t": 10.256668,
            "value": 296824.5707421999
          },
          {
            "t": 11.265757,
            "value": 297297.85975270765
          },
          {
            "t": 12.273787,
            "value": 298602.22414015455
          },
          {
            "t": 13.285095,
            "value": 295656.71387945116
          },
          {
            "t": 14.297446,
            "value": 296339.90582317795
          },
          {
            "t": 15.3094,
            "value": 395274.8840362309
          },
          {
            "t": 16.320819,
            "value": 395483.9685629793
          },
          {
            "t": 17.335193,
            "value": 295748.9052361357
          },
          {
            "t": 18.347227,
            "value": 296432.7285446931
          },
          {
            "t": 19.360041,
            "value": 296204.43635257805
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.165918,
            "value": 297453.20565319736
          },
          {
            "t": 1.17501,
            "value": 297296.97589516116
          },
          {
            "t": 2.184081,
            "value": 297303.1630083512
          },
          {
            "t": 3.193147,
            "value": 297304.6361684964
          },
          {
            "t": 4.203283,
            "value": 296989.7122763668
          },
          {
            "t": 5.211881,
            "value": 297442.5886230192
          },
          {
            "t": 6.220986,
            "value": 297293.1459065211
          },
          {
            "t": 7.229213,
            "value": 297552.0393720858
          },
          {
            "t": 8.237641,
            "value": 297492.7312609328
          },
          {
            "t": 9.24597,
            "value": 297521.93976370804
          },
          {
            "t": 10.256668,
            "value": 296824.5707421999
          },
          {
            "t": 11.265757,
            "value": 297297.85975270765
          },
          {
            "t": 12.273787,
            "value": 446415.2852593673
          },
          {
            "t": 13.285095,
            "value": 296645.53232051956
          },
          {
            "t": 14.297446,
            "value": 296339.90582317795
          },
          {
            "t": 15.3094,
            "value": 296456.16302717314
          },
          {
            "t": 16.320819,
            "value": 296612.9764222345
          },
          {
            "t": 17.335193,
            "value": 295748.90523613576
          },
          {
            "t": 18.347227,
            "value": 296432.72854469315
          },
          {
            "t": 19.360041,
            "value": 296204.43635257805
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.065159,
            "value": 107494252.07721962
          },
          {
            "t": 2.08332,
            "value": 107451207.31200334
          },
          {
            "t": 4.102499,
            "value": 107395251.7335016
          },
          {
            "t": 6.120243,
            "value": 107442674.59102839
          },
          {
            "t": 8.136909,
            "value": 107516977.03040564
          },
          {
            "t": 10.155918,
            "value": 107215266.99484748
          },
          {
            "t": 12.173055,
            "value": 107309114.84941281
          },
          {
            "t": 14.193063,
            "value": 107164838.45608534
          },
          {
            "t": 16.116329,
            "value": 112739167.64503714
          },
          {
            "t": 18.142912,
            "value": 106984787.20091899
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.065159,
            "value": 107578674.5384094
          },
          {
            "t": 2.08332,
            "value": 107560815.51471859
          },
          {
            "t": 4.102499,
            "value": 107477560.92946689
          },
          {
            "t": 6.120243,
            "value": 107542391.89907144
          },
          {
            "t": 8.136909,
            "value": 107603191.10849294
          },
          {
            "t": 10.155918,
            "value": 107304821.32570979
          },
          {
            "t": 12.173055,
            "value": 107388294.39943841
          },
          {
            "t": 14.193063,
            "value": 107278453.35266
          },
          {
            "t": 16.116329,
            "value": 112840544.15769842
          },
          {
            "t": 18.142912,
            "value": 107078010.12837866
          }
        ],
        "ram_mib": [
          {
            "t": 0.065159,
            "value": 11.203125
          },
          {
            "t": 2.08332,
            "value": 10.87890625
          },
          {
            "t": 4.102499,
            "value": 11.1640625
          },
          {
            "t": 6.120243,
            "value": 10.84765625
          },
          {
            "t": 8.136909,
            "value": 11.05078125
          },
          {
            "t": 10.155918,
            "value": 11.171875
          },
          {
            "t": 12.173055,
            "value": 11.09375
          },
          {
            "t": 14.193063,
            "value": 10.94140625
          },
          {
            "t": 16.116329,
            "value": 10.85546875
          },
          {
            "t": 18.142912,
            "value": 11.171875
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
          "extra": "DFE OTLP Baseline (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 1.2784810066223145
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 18.219617504988296
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 18.948858218318694
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 11.12109375
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 11.453125
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 409142.60748448194
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/400k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 403911.79693309806
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000764
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 143733309.0974164
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 143645451.63629675
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.077604,
            "value": 17.785858585858584
          },
          {
            "t": 2.097699,
            "value": 17.921211356466877
          },
          {
            "t": 4.117894,
            "value": 17.744242424242422
          },
          {
            "t": 6.139509,
            "value": 17.72580075662043
          },
          {
            "t": 8.1632,
            "value": 18.930643127364437
          },
          {
            "t": 10.085604,
            "value": 18.948858218318694
          },
          {
            "t": 12.107679,
            "value": 17.987894073139977
          },
          {
            "t": 14.129742,
            "value": 18.186194690265488
          },
          {
            "t": 16.151021,
            "value": 18.55898670044332
          },
          {
            "t": 18.173146,
            "value": 18.40648511716276
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.178539,
            "value": 395806.0392085462
          },
          {
            "t": 1.188456,
            "value": 396072.15246401436
          },
          {
            "t": 2.198555,
            "value": 396000.7880415682
          },
          {
            "t": 3.209054,
            "value": 395844.03349236364
          },
          {
            "t": 4.218714,
            "value": 396172.96911831707
          },
          {
            "t": 5.229855,
            "value": 395592.7017102461
          },
          {
            "t": 6.242547,
            "value": 394986.82718931325
          },
          {
            "t": 7.354884,
            "value": 359603.2497345679
          },
          {
            "t": 8.365349,
            "value": 395857.3528029175
          },
          {
            "t": 9.376987,
            "value": 395398.35395665246
          },
          {
            "t": 10.389195,
            "value": 395175.69511404773
          },
          {
            "t": 11.399679,
            "value": 395849.9095482957
          },
          {
            "t": 12.410653,
            "value": 395658.0485749386
          },
          {
            "t": 13.421275,
            "value": 395795.85641317925
          },
          {
            "t": 14.432629,
            "value": 593264.07963977
          },
          {
            "t": 15.443225,
            "value": 395806.0392085462
          },
          {
            "t": 16.454234,
            "value": 395644.3513361404
          },
          {
            "t": 17.465304,
            "value": 494525.6015903944
          },
          {
            "t": 18.476483,
            "value": 395577.83537830587
          },
          {
            "t": 19.48721,
            "value": 395754.7389156518
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.178539,
            "value": 395806.0392085462
          },
          {
            "t": 1.188456,
            "value": 396072.15246401436
          },
          {
            "t": 2.198555,
            "value": 396000.7880415682
          },
          {
            "t": 3.209054,
            "value": 592776.4401548146
          },
          {
            "t": 4.218714,
            "value": 397163.40154111286
          },
          {
            "t": 5.331708,
            "value": 359390.9760519823
          },
          {
            "t": 6.344407,
            "value": 394984.09695279645
          },
          {
            "t": 7.354884,
            "value": 394863.020138014
          },
          {
            "t": 8.365349,
            "value": 395857.3528029175
          },
          {
            "t": 9.376987,
            "value": 396386.8498415441
          },
          {
            "t": 10.389195,
            "value": 395175.69511404773
          },
          {
            "t": 11.399679,
            "value": 394860.284774425
          },
          {
            "t": 12.410653,
            "value": 395658.0485749386
          },
          {
            "t": 13.421275,
            "value": 396785.3460542122
          },
          {
            "t": 14.432629,
            "value": 395509.3864265133
          },
          {
            "t": 15.443225,
            "value": 395806.0392085462
          },
          {
            "t": 16.454234,
            "value": 395644.3513361404
          },
          {
            "t": 17.465304,
            "value": 394631.4300691347
          },
          {
            "t": 18.476483,
            "value": 396566.77996675164
          },
          {
            "t": 19.48721,
            "value": 394765.3520683627
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.077604,
            "value": 142909766.55470094
          },
          {
            "t": 2.097699,
            "value": 142945491.17739514
          },
          {
            "t": 4.117894,
            "value": 142956832.3849925
          },
          {
            "t": 6.139509,
            "value": 142849613.3042147
          },
          {
            "t": 8.1632,
            "value": 142898112.4094538
          },
          {
            "t": 10.085604,
            "value": 150396689.7696842
          },
          {
            "t": 12.107679,
            "value": 143011532.70773834
          },
          {
            "t": 14.129742,
            "value": 142989293.11302367
          },
          {
            "t": 16.151021,
            "value": 142709338.49310264
          },
          {
            "t": 18.173146,
            "value": 142787846.44866168
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.077604,
            "value": 143020329.02775916
          },
          {
            "t": 2.097699,
            "value": 143060404.08990666
          },
          {
            "t": 4.117894,
            "value": 143055088.24643165
          },
          {
            "t": 6.139509,
            "value": 142954803.9562429
          },
          {
            "t": 8.1632,
            "value": 142983940.73008183
          },
          {
            "t": 10.085604,
            "value": 150514455.33821195
          },
          {
            "t": 12.107679,
            "value": 143101487.82809737
          },
          {
            "t": 14.129742,
            "value": 142924990.96220046
          },
          {
            "t": 16.151021,
            "value": 142971970.22281438
          },
          {
            "t": 18.173146,
            "value": 142745620.57241765
          }
        ],
        "ram_mib": [
          {
            "t": 0.077604,
            "value": 10.9453125
          },
          {
            "t": 2.097699,
            "value": 10.890625
          },
          {
            "t": 4.117894,
            "value": 10.9609375
          },
          {
            "t": 6.139509,
            "value": 11.20703125
          },
          {
            "t": 8.1632,
            "value": 11.43359375
          },
          {
            "t": 10.085604,
            "value": 11.4453125
          },
          {
            "t": 12.107679,
            "value": 10.96484375
          },
          {
            "t": 14.129742,
            "value": 10.98828125
          },
          {
            "t": 16.151021,
            "value": 10.921875
          },
          {
            "t": 18.173146,
            "value": 11.453125
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
          "extra": "DFE OTLP Baseline (Logs)/600k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -0.008771929889917374
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/600k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 23.363388624345085
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/600k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 25.6213
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/600k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 13.8078125
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/600k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 15.125
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/600k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 585554.2232199473
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/600k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 591731.546848338
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/600k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000601
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/600k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 216048687.57088217
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/600k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 215955617.39515734
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.123396,
            "value": 23.346449184441656
          },
          {
            "t": 2.137647,
            "value": 22.81465495608532
          },
          {
            "t": 4.155621,
            "value": 25.6213
          },
          {
            "t": 6.076186,
            "value": 23.03060587133042
          },
          {
            "t": 8.095957,
            "value": 23.355373978629792
          },
          {
            "t": 10.111375,
            "value": 24.437313245448838
          },
          {
            "t": 12.128114,
            "value": 23.22219419924338
          },
          {
            "t": 14.147075,
            "value": 23.12677966101695
          },
          {
            "t": 16.166579,
            "value": 21.894472361809044
          },
          {
            "t": 18.181663,
            "value": 22.78474278544542
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.223929,
            "value": 595678.9449334528
          },
          {
            "t": 1.231171,
            "value": 595686.0416861093
          },
          {
            "t": 2.23873,
            "value": 595498.6258869207
          },
          {
            "t": 3.348722,
            "value": 540544.436356298
          },
          {
            "t": 4.356782,
            "value": 595202.666507946
          },
          {
            "t": 5.370488,
            "value": 591887.5887091524
          },
          {
            "t": 6.382878,
            "value": 296328.4900087911
          },
          {
            "t": 6.484313,
            "value": 269342.1318429735
          },
          {
            "t": 7.49141,
            "value": 568514.085877524
          },
          {
            "t": 8.499158,
            "value": 595386.9419735885
          },
          {
            "t": 9.506248,
            "value": 595775.948524958
          },
          {
            "t": 10.514305,
            "value": 595204.4378442885
          },
          {
            "t": 11.523235,
            "value": 594689.423448604
          },
          {
            "t": 12.530777,
            "value": 595508.6735838308
          },
          {
            "t": 13.542797,
            "value": 592873.6586233474
          },
          {
            "t": 14.554851,
            "value": 296426.8705029574
          },
          {
            "t": 14.655621,
            "value": 269584.4086755857
          },
          {
            "t": 15.662588,
            "value": 568746.7673785923
          },
          {
            "t": 16.670256,
            "value": 595434.2104740847
          },
          {
            "t": 17.677694,
            "value": 595570.149230027
          },
          {
            "t": 18.684977,
            "value": 595661.795145952
          },
          {
            "t": 19.692664,
            "value": 595422.9835256385
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.223929,
            "value": 889547.224433956
          },
          {
            "t": 1.231171,
            "value": 598664.4718945398
          },
          {
            "t": 2.340208,
            "value": 541009.9031862778
          },
          {
            "t": 3.348722,
            "value": 593943.1678687654
          },
          {
            "t": 4.356782,
            "value": 597186.6753963059
          },
          {
            "t": 5.370488,
            "value": 591887.5887091523
          },
          {
            "t": 6.382878,
            "value": 592656.9800175822
          },
          {
            "t": 7.390318,
            "value": 592591.1220519337
          },
          {
            "t": 8.397675,
            "value": 598596.128284213
          },
          {
            "t": 9.404956,
            "value": 595662.9778582143
          },
          {
            "t": 10.412949,
            "value": 593258.0881018024
          },
          {
            "t": 11.422153,
            "value": 593537.0846726727
          },
          {
            "t": 12.429784,
            "value": 594463.6479028533
          },
          {
            "t": 13.436966,
            "value": 598700.1356259346
          },
          {
            "t": 14.448789,
            "value": 593977.4051390411
          },
          {
            "t": 15.460866,
            "value": 592840.2680823692
          },
          {
            "t": 16.468418,
            "value": 593517.7539223782
          },
          {
            "t": 17.475909,
            "value": 596531.3834068989
          },
          {
            "t": 18.483317,
            "value": 594595.2384733891
          },
          {
            "t": 19.491112,
            "value": 594366.9099370408
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.123396,
            "value": 215190164.34458256
          },
          {
            "t": 2.137647,
            "value": 215226513.97467348
          },
          {
            "t": 4.155621,
            "value": 214796009.7602843
          },
          {
            "t": 6.076186,
            "value": 225720126.62940332
          },
          {
            "t": 8.095957,
            "value": 214111981.01170877
          },
          {
            "t": 10.111375,
            "value": 214934337.19456708
          },
          {
            "t": 12.128114,
            "value": 214999026.1506323
          },
          {
            "t": 14.147075,
            "value": 214731935.38656765
          },
          {
            "t": 16.166579,
            "value": 214677768.89770955
          },
          {
            "t": 18.181663,
            "value": 215168310.60144392
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.123396,
            "value": 215277492.44697994
          },
          {
            "t": 2.137647,
            "value": 215314023.92253995
          },
          {
            "t": 4.155621,
            "value": 214383219.01075038
          },
          {
            "t": 6.076186,
            "value": 225634540.8772939
          },
          {
            "t": 8.095957,
            "value": 214737518.75831467
          },
          {
            "t": 10.111375,
            "value": 215192932.68195483
          },
          {
            "t": 12.128114,
            "value": 215059348.7803826
          },
          {
            "t": 14.147075,
            "value": 214830459.3303189
          },
          {
            "t": 16.166579,
            "value": 214811829.043171
          },
          {
            "t": 18.181663,
            "value": 215245510.85711566
          }
        ],
        "ram_mib": [
          {
            "t": 0.123396,
            "value": 13.453125
          },
          {
            "t": 2.137647,
            "value": 13.83203125
          },
          {
            "t": 4.155621,
            "value": 13.58203125
          },
          {
            "t": 6.076186,
            "value": 15.125
          },
          {
            "t": 8.095957,
            "value": 13.0703125
          },
          {
            "t": 10.111375,
            "value": 13.85546875
          },
          {
            "t": 12.128114,
            "value": 13.58203125
          },
          {
            "t": 14.147075,
            "value": 13.8359375
          },
          {
            "t": 16.166579,
            "value": 14.14453125
          },
          {
            "t": 18.181663,
            "value": 13.59765625
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
          "extra": "DFE OTLP Baseline (Logs)/800k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/800k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 29.187127171858922
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/800k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 29.71414204902577
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/800k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 13.739453125
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/800k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 15.1171875
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/800k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 798799.9566804639
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/800k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 815507.3106309689
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/800k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000612
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/800k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 287044948.5633317
        },
        {
          "extra": "DFE OTLP Baseline (Logs)/800k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 286815888.0729737
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.095691,
            "value": 28.40978274363749
          },
          {
            "t": 2.139104,
            "value": 29.325456821026286
          },
          {
            "t": 4.069666,
            "value": 28.91305210918114
          },
          {
            "t": 6.101293,
            "value": 29.01804604853765
          },
          {
            "t": 8.135529,
            "value": 29.31528971962617
          },
          {
            "t": 10.16857,
            "value": 29.71414204902577
          },
          {
            "t": 12.110386,
            "value": 28.610718301061837
          },
          {
            "t": 14.14297,
            "value": 29.27770287141074
          },
          {
            "t": 16.174432,
            "value": 29.58311607697083
          },
          {
            "t": 18.209254,
            "value": 29.70396497811132
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.202828,
            "value": 784027.0175710255
          },
          {
            "t": 1.225863,
            "value": 488741.83190213435
          },
          {
            "t": 1.327458,
            "value": 266754.3992246339
          },
          {
            "t": 2.342496,
            "value": 743330.1314360984
          },
          {
            "t": 3.358863,
            "value": 787117.2519375383
          },
          {
            "t": 4.374333,
            "value": 787812.5400061056
          },
          {
            "t": 5.390344,
            "value": 787393.049878397
          },
          {
            "t": 6.405861,
            "value": 787776.0785885416
          },
          {
            "t": 7.423063,
            "value": 786471.1237296034
          },
          {
            "t": 8.440119,
            "value": 786584.0229053267
          },
          {
            "t": 9.456817,
            "value": 786860.9951037574
          },
          {
            "t": 10.4785,
            "value": 880899.4570723013
          },
          {
            "t": 11.500434,
            "value": 489268.3871952592
          },
          {
            "t": 11.602576,
            "value": 355847.82523601607
          },
          {
            "t": 12.617777,
            "value": 742998.1450769547
          },
          {
            "t": 13.63322,
            "value": 787833.4874532594
          },
          {
            "t": 14.649949,
            "value": 786837.0037640316
          },
          {
            "t": 15.665629,
            "value": 787649.6534341526
          },
          {
            "t": 16.682499,
            "value": 885068.8878617719
          },
          {
            "t": 17.700988,
            "value": 883661.9737670217
          },
          {
            "t": 18.716654,
            "value": 787660.510443394
          },
          {
            "t": 19.732123,
            "value": 787813.3158176173
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.095691,
            "value": 786749.5638457105
          },
          {
            "t": 1.117236,
            "value": 783127.5176326056
          },
          {
            "t": 2.139104,
            "value": 782879.9805845765
          },
          {
            "t": 3.154601,
            "value": 786806.8541807607
          },
          {
            "t": 4.170953,
            "value": 788112.7798243129
          },
          {
            "t": 5.186926,
            "value": 787422.5004010933
          },
          {
            "t": 6.202659,
            "value": 787608.554610316
          },
          {
            "t": 7.118545,
            "value": 873471.152523349
          },
          {
            "t": 8.135529,
            "value": 786639.711145898
          },
          {
            "t": 9.151748,
            "value": 787231.8860403122
          },
          {
            "t": 10.16857,
            "value": 786765.0385219833
          },
          {
            "t": 11.190414,
            "value": 782898.3680483518
          },
          {
            "t": 12.211715,
            "value": 783314.6153778367
          },
          {
            "t": 13.227295,
            "value": 787727.2100671538
          },
          {
            "t": 14.244259,
            "value": 786655.1815010167
          },
          {
            "t": 15.259716,
            "value": 787822.6256749424
          },
          {
            "t": 16.275742,
            "value": 1181072.1379177303
          },
          {
            "t": 17.193875,
            "value": 871333.4560461284
          },
          {
            "t": 18.209254,
            "value": 787883.1451113328
          },
          {
            "t": 19.224888,
            "value": 787685.3275884817
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.095691,
            "value": 284287133.06333166
          },
          {
            "t": 2.139104,
            "value": 282734812.7862551
          },
          {
            "t": 4.069666,
            "value": 298703850.48498833
          },
          {
            "t": 6.101293,
            "value": 284189588.4431542
          },
          {
            "t": 8.135529,
            "value": 284165428.69165623
          },
          {
            "t": 10.16857,
            "value": 284171775.67988056
          },
          {
            "t": 12.110386,
            "value": 297898715.9442501
          },
          {
            "t": 14.14297,
            "value": 284054842.0139094
          },
          {
            "t": 16.174432,
            "value": 283861409.1723104
          },
          {
            "t": 18.209254,
            "value": 284091324.450001
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.095691,
            "value": 284534788.8362783
          },
          {
            "t": 2.139104,
            "value": 282922374.96776223
          },
          {
            "t": 4.069666,
            "value": 299014421.7072542
          },
          {
            "t": 6.101293,
            "value": 284427043.9406446
          },
          {
            "t": 8.135529,
            "value": 284420746.1671114
          },
          {
            "t": 10.16857,
            "value": 284427510.31582737
          },
          {
            "t": 12.110386,
            "value": 298129668.82547057
          },
          {
            "t": 14.14297,
            "value": 284122556.80454046
          },
          {
            "t": 16.174432,
            "value": 284285435.31702787
          },
          {
            "t": 18.209254,
            "value": 284164938.75139934
          }
        ],
        "ram_mib": [
          {
            "t": 0.095691,
            "value": 13.41796875
          },
          {
            "t": 2.139104,
            "value": 13.9140625
          },
          {
            "t": 4.069666,
            "value": 13.36328125
          },
          {
            "t": 6.101293,
            "value": 13.25
          },
          {
            "t": 8.135529,
            "value": 13.4296875
          },
          {
            "t": 10.16857,
            "value": 14.33203125
          },
          {
            "t": 12.110386,
            "value": 15.1171875
          },
          {
            "t": 14.14297,
            "value": 13.5234375
          },
          {
            "t": 16.174432,
            "value": 13.41015625
          },
          {
            "t": 18.209254,
            "value": 13.63671875
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
