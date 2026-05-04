window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otlp_gzip_baseline"] = {
  "name": "OTC OTLP Baseline w/ Gzip (Logs)",
  "slug": "otc_logs_otlp_gzip_baseline",
  "description": "OpenTelemetry Collector baseline for OTLP logs with gzip compression",
  "meta": {
    "binary": "otc",
    "protocols": [
      "otlp"
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
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 35.14221284838418
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 36.78343106674984
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 219.537890625
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 220.7578125
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 98808.62796939429
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/100k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 99328.59100302533
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000569
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 1884219.9013534866
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 2502224.2922629816
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.049839,
            "value": 36.78343106674984
          },
          {
            "t": 3.066395,
            "value": 36.09228091982598
          },
          {
            "t": 5.078322,
            "value": 36.12116408668731
          },
          {
            "t": 7.090587,
            "value": 34.984004990642546
          },
          {
            "t": 9.100943,
            "value": 35.05826575171553
          },
          {
            "t": 11.11345,
            "value": 33.135890068707056
          },
          {
            "t": 13.127848,
            "value": 34.925143570536825
          },
          {
            "t": 15.141885,
            "value": 34.03937926753569
          },
          {
            "t": 17.15787,
            "value": 33.93636701797892
          },
          {
            "t": 19.172234,
            "value": 36.34620174346202
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.043804,
            "value": 98410.22750655323
          },
          {
            "t": 1.049839,
            "value": 99400.12027414553
          },
          {
            "t": 2.057991,
            "value": 99191.39177425626
          },
          {
            "t": 3.066395,
            "value": 99166.60386115088
          },
          {
            "t": 4.073287,
            "value": 99315.51745370905
          },
          {
            "t": 5.078322,
            "value": 99499.02242210471
          },
          {
            "t": 6.083707,
            "value": 99464.38429059515
          },
          {
            "t": 7.090587,
            "value": 99316.70109645638
          },
          {
            "t": 8.09622,
            "value": 99439.85529512257
          },
          {
            "t": 9.201736,
            "value": 90455.49770423947
          },
          {
            "t": 10.208152,
            "value": 99362.49026247596
          },
          {
            "t": 11.214168,
            "value": 99401.99758254342
          },
          {
            "t": 12.220492,
            "value": 99371.57416498066
          },
          {
            "t": 13.22876,
            "value": 99179.97992597207
          },
          {
            "t": 14.234615,
            "value": 99417.90814779467
          },
          {
            "t": 15.242964,
            "value": 99172.01286459352
          },
          {
            "t": 16.250336,
            "value": 99268.19486743725
          },
          {
            "t": 17.25857,
            "value": 99183.32450601745
          },
          {
            "t": 18.266471,
            "value": 99216.09364411783
          },
          {
            "t": 19.272894,
            "value": 99361.79916396982
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.043804,
            "value": 99404.27020863963
          },
          {
            "t": 1.049839,
            "value": 99400.12027414553
          },
          {
            "t": 2.057991,
            "value": 99191.39177425626
          },
          {
            "t": 3.066395,
            "value": 99166.60386115088
          },
          {
            "t": 4.073287,
            "value": 99315.51745370905
          },
          {
            "t": 5.078322,
            "value": 99499.02242210471
          },
          {
            "t": 6.083707,
            "value": 99464.38429059515
          },
          {
            "t": 7.090587,
            "value": 99316.70109645638
          },
          {
            "t": 8.09622,
            "value": 99439.85529512257
          },
          {
            "t": 9.100943,
            "value": 99529.920186957
          },
          {
            "t": 10.107548,
            "value": 99343.83397658466
          },
          {
            "t": 11.11345,
            "value": 99413.26292223297
          },
          {
            "t": 12.119776,
            "value": 99371.37667117812
          },
          {
            "t": 13.127848,
            "value": 99199.26354466744
          },
          {
            "t": 14.133716,
            "value": 99416.62325474118
          },
          {
            "t": 15.141885,
            "value": 99189.719183986
          },
          {
            "t": 16.14963,
            "value": 99231.45240115307
          },
          {
            "t": 17.15787,
            "value": 99182.73426961835
          },
          {
            "t": 18.165525,
            "value": 99240.3153857223
          },
          {
            "t": 19.172234,
            "value": 99333.5710716801
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.049839,
            "value": 2505441.276165143
          },
          {
            "t": 3.066395,
            "value": 2499482.2856394765
          },
          {
            "t": 5.078322,
            "value": 2505494.4836467723
          },
          {
            "t": 7.090587,
            "value": 2492837.672970508
          },
          {
            "t": 9.100943,
            "value": 2507295.722747613
          },
          {
            "t": 11.11345,
            "value": 2504979.1131161284
          },
          {
            "t": 13.127848,
            "value": 2502240.8679913306
          },
          {
            "t": 15.141885,
            "value": 2502553.3294572043
          },
          {
            "t": 17.15787,
            "value": 2499756.6946182633
          },
          {
            "t": 19.172234,
            "value": 2502161.476277376
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.049839,
            "value": 1877276.5587755241
          },
          {
            "t": 3.066395,
            "value": 1882686.124263348
          },
          {
            "t": 5.078322,
            "value": 1886329.8718094642
          },
          {
            "t": 7.090587,
            "value": 1886266.4708674054
          },
          {
            "t": 9.100943,
            "value": 1887850.211604313
          },
          {
            "t": 11.11345,
            "value": 1886151.948788253
          },
          {
            "t": 13.127848,
            "value": 1884085.965136979
          },
          {
            "t": 15.141885,
            "value": 1884547.8012568785
          },
          {
            "t": 17.15787,
            "value": 1873307.5890941648
          },
          {
            "t": 19.172234,
            "value": 1893696.4719385374
          }
        ],
        "ram_mib": [
          {
            "t": 1.049839,
            "value": 220.06640625
          },
          {
            "t": 3.066395,
            "value": 220.53515625
          },
          {
            "t": 5.078322,
            "value": 220.7578125
          },
          {
            "t": 7.090587,
            "value": 218.98828125
          },
          {
            "t": 9.100943,
            "value": 219.1328125
          },
          {
            "t": 11.11345,
            "value": 218.90625
          },
          {
            "t": 13.127848,
            "value": 219.59765625
          },
          {
            "t": 15.141885,
            "value": 219.046875
          },
          {
            "t": 17.15787,
            "value": 219.921875
          },
          {
            "t": 19.172234,
            "value": 218.42578125
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
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 60.096054263128295
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 61.52054828660436
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 59.04609375
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 61.19921875
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 197657.4163169214
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/200k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 198701.30915926234
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000727
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 3769809.5158722876
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 5007760.515640144
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.054675,
            "value": 60.36114570361145
          },
          {
            "t": 3.067598,
            "value": 61.52054828660436
          },
          {
            "t": 5.080483,
            "value": 60.39029247044182
          },
          {
            "t": 7.095091,
            "value": 60.34582247051521
          },
          {
            "t": 9.10798,
            "value": 60.26195395146235
          },
          {
            "t": 11.1211,
            "value": 59.53709316770186
          },
          {
            "t": 13.13326,
            "value": 57.749359850839035
          },
          {
            "t": 15.146263,
            "value": 60.64306172839507
          },
          {
            "t": 17.158659,
            "value": 60.08917544947303
          },
          {
            "t": 19.172179,
            "value": 60.06208955223881
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.047997,
            "value": 198837.99078187076
          },
          {
            "t": 1.054675,
            "value": 198673.25996992088
          },
          {
            "t": 2.161826,
            "value": 180643.8326840693
          },
          {
            "t": 3.168488,
            "value": 199669.79979377388
          },
          {
            "t": 4.174966,
            "value": 197719.1751831635
          },
          {
            "t": 5.181496,
            "value": 198702.47285227466
          },
          {
            "t": 6.188767,
            "value": 198556.29716332545
          },
          {
            "t": 7.196071,
            "value": 198549.79231691724
          },
          {
            "t": 8.202985,
            "value": 198626.6950305587
          },
          {
            "t": 9.209017,
            "value": 198800.83337309348
          },
          {
            "t": 10.215525,
            "value": 198706.81604120386
          },
          {
            "t": 11.222027,
            "value": 198708.00058022735
          },
          {
            "t": 12.227988,
            "value": 198814.86459216606
          },
          {
            "t": 13.234176,
            "value": 198770.0111708746
          },
          {
            "t": 14.240427,
            "value": 199751.3542843684
          },
          {
            "t": 15.247128,
            "value": 197675.37729673457
          },
          {
            "t": 16.253294,
            "value": 198774.35731280924
          },
          {
            "t": 17.259538,
            "value": 199752.74386729265
          },
          {
            "t": 18.266238,
            "value": 197675.57365650145
          },
          {
            "t": 19.27318,
            "value": 198621.17182518952
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.047997,
            "value": 198837.99078187076
          },
          {
            "t": 1.054675,
            "value": 198673.25996992088
          },
          {
            "t": 2.060987,
            "value": 198745.51828856257
          },
          {
            "t": 3.067598,
            "value": 198686.4836565466
          },
          {
            "t": 4.074114,
            "value": 197711.71049441837
          },
          {
            "t": 5.080483,
            "value": 199727.93279602213
          },
          {
            "t": 6.087904,
            "value": 198526.73311356426
          },
          {
            "t": 7.095091,
            "value": 198572.85687762054
          },
          {
            "t": 8.101853,
            "value": 198656.68350613155
          },
          {
            "t": 9.10798,
            "value": 198782.06230426178
          },
          {
            "t": 10.114561,
            "value": 198692.40528084675
          },
          {
            "t": 11.1211,
            "value": 198700.69614788893
          },
          {
            "t": 12.127072,
            "value": 198812.69061166712
          },
          {
            "t": 13.13326,
            "value": 198770.0111708746
          },
          {
            "t": 14.139543,
            "value": 198751.24592187288
          },
          {
            "t": 15.146263,
            "value": 198664.9713922441
          },
          {
            "t": 16.15233,
            "value": 198793.91730371834
          },
          {
            "t": 17.158659,
            "value": 198742.1608638924
          },
          {
            "t": 18.165366,
            "value": 198667.53683047797
          },
          {
            "t": 19.172179,
            "value": 198646.62057402913
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.054675,
            "value": 5011548.196740209
          },
          {
            "t": 3.067598,
            "value": 5009819.550971399
          },
          {
            "t": 5.080483,
            "value": 5010653.365691532
          },
          {
            "t": 7.095091,
            "value": 5006720.910469928
          },
          {
            "t": 9.10798,
            "value": 5010958.8755266685
          },
          {
            "t": 11.1211,
            "value": 5008998.9667779375
          },
          {
            "t": 13.13326,
            "value": 4987849.8727735365
          },
          {
            "t": 15.146263,
            "value": 5033932.388575676
          },
          {
            "t": 17.158659,
            "value": 4987687.810947746
          },
          {
            "t": 19.172179,
            "value": 5009435.217926814
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.054675,
            "value": 3774026.321202948
          },
          {
            "t": 3.067598,
            "value": 3771199.8918984984
          },
          {
            "t": 5.080483,
            "value": 3763331.73529536
          },
          {
            "t": 7.095091,
            "value": 3769972.6199836396
          },
          {
            "t": 9.10798,
            "value": 3772269.1117095877
          },
          {
            "t": 11.1211,
            "value": 3760724.6463201405
          },
          {
            "t": 13.13326,
            "value": 3772355.5780852414
          },
          {
            "t": 15.146263,
            "value": 3771154.3400581125
          },
          {
            "t": 17.158659,
            "value": 3772252.5785183436
          },
          {
            "t": 19.172179,
            "value": 3770808.335650999
          }
        ],
        "ram_mib": [
          {
            "t": 1.054675,
            "value": 56.609375
          },
          {
            "t": 3.067598,
            "value": 61.19921875
          },
          {
            "t": 5.080483,
            "value": 60.203125
          },
          {
            "t": 7.095091,
            "value": 60.79296875
          },
          {
            "t": 9.10798,
            "value": 57.48828125
          },
          {
            "t": 11.1211,
            "value": 59.1640625
          },
          {
            "t": 13.13326,
            "value": 57.33203125
          },
          {
            "t": 15.146263,
            "value": 60.546875
          },
          {
            "t": 17.158659,
            "value": 58.41796875
          },
          {
            "t": 19.172179,
            "value": 58.70703125
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
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.614035129547119
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 88.0795617784792
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 90.54105393676379
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 69.14296875
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 71.19921875
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 295723.92542485934
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/300k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 303454.252598246
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000785
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 5625051.543348766
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 7539009.371006295
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.119829,
            "value": 85.21619875776398
          },
          {
            "t": 3.135243,
            "value": 89.22152889993785
          },
          {
            "t": 5.050411,
            "value": 87.27707377557347
          },
          {
            "t": 7.065779,
            "value": 90.54105393676379
          },
          {
            "t": 9.081049,
            "value": 89.53175591531756
          },
          {
            "t": 11.096741,
            "value": 88.99263288009888
          },
          {
            "t": 13.114134,
            "value": 87.62623682579044
          },
          {
            "t": 15.136695,
            "value": 89.55826517967782
          },
          {
            "t": 17.16521,
            "value": 86.38218886804253
          },
          {
            "t": 19.185999,
            "value": 86.44868274582561
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.112523,
            "value": 297801.0371417453
          },
          {
            "t": 1.119829,
            "value": 298816.844136737
          },
          {
            "t": 2.127631,
            "value": 296685.26158908196
          },
          {
            "t": 3.135243,
            "value": 297733.65144519915
          },
          {
            "t": 4.14321,
            "value": 297628.79141876666
          },
          {
            "t": 5.151016,
            "value": 298668.59296332824
          },
          {
            "t": 6.158488,
            "value": 296782.4415963918
          },
          {
            "t": 7.166527,
            "value": 298599.55815201596
          },
          {
            "t": 8.173872,
            "value": 297812.5666976061
          },
          {
            "t": 9.181743,
            "value": 296664.9501771556
          },
          {
            "t": 10.188875,
            "value": 297875.55156622967
          },
          {
            "t": 11.198978,
            "value": 296999.4149111526
          },
          {
            "t": 12.207186,
            "value": 297557.6468347801
          },
          {
            "t": 13.220209,
            "value": 296143.3254723733
          },
          {
            "t": 14.230353,
            "value": 296987.36021794914
          },
          {
            "t": 15.244153,
            "value": 295916.3543105149
          },
          {
            "t": 16.25841,
            "value": 295783.02146300196
          },
          {
            "t": 17.270518,
            "value": 296411.0549467053
          },
          {
            "t": 18.379718,
            "value": 271366.7508113956
          },
          {
            "t": 19.387257,
            "value": 296762.7059597693
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.112523,
            "value": 297801.0371417453
          },
          {
            "t": 1.119829,
            "value": 297824.0971462495
          },
          {
            "t": 2.127631,
            "value": 296685.26158908196
          },
          {
            "t": 3.135243,
            "value": 299718.5424548338
          },
          {
            "t": 4.14321,
            "value": 296636.69544737076
          },
          {
            "t": 5.151016,
            "value": 297676.3385016561
          },
          {
            "t": 6.158488,
            "value": 296782.4415963918
          },
          {
            "t": 7.166527,
            "value": 299591.58326215553
          },
          {
            "t": 8.173872,
            "value": 296819.8581419474
          },
          {
            "t": 9.181743,
            "value": 297657.14064597554
          },
          {
            "t": 10.188875,
            "value": 296882.63306100894
          },
          {
            "t": 11.198978,
            "value": 445499.12236672896
          },
          {
            "t": 12.308565,
            "value": 269469.63149351964
          },
          {
            "t": 13.32135,
            "value": 299175.04702380067
          },
          {
            "t": 14.331444,
            "value": 295022.0474530093
          },
          {
            "t": 15.345339,
            "value": 296874.9229456699
          },
          {
            "t": 16.359583,
            "value": 296772.7686828811
          },
          {
            "t": 17.371848,
            "value": 294389.31505090074
          },
          {
            "t": 18.379718,
            "value": 296665.24452558364
          },
          {
            "t": 19.387257,
            "value": 298747.7407822427
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.119829,
            "value": 7507205.078696971
          },
          {
            "t": 3.135243,
            "value": 7525546.116083346
          },
          {
            "t": 5.050411,
            "value": 7880757.719427225
          },
          {
            "t": 7.065779,
            "value": 7513840.648457254
          },
          {
            "t": 9.081049,
            "value": 7514341.502627439
          },
          {
            "t": 11.096741,
            "value": 7512712.755718632
          },
          {
            "t": 13.114134,
            "value": 7505587.16125217
          },
          {
            "t": 15.136695,
            "value": 7487088.893734231
          },
          {
            "t": 17.16521,
            "value": 7465831.901662053
          },
          {
            "t": 19.185999,
            "value": 7477181.93240363
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.119829,
            "value": 5597944.1005812315
          },
          {
            "t": 3.135243,
            "value": 5617749.008392321
          },
          {
            "t": 5.050411,
            "value": 5892926.364684456
          },
          {
            "t": 7.065779,
            "value": 5618277.158315504
          },
          {
            "t": 9.081049,
            "value": 5609578.865362953
          },
          {
            "t": 11.096741,
            "value": 5588418.766359146
          },
          {
            "t": 13.114134,
            "value": 5603368.803202945
          },
          {
            "t": 15.136695,
            "value": 5597536.489628742
          },
          {
            "t": 17.16521,
            "value": 5546534.780368891
          },
          {
            "t": 19.185999,
            "value": 5578181.096591479
          }
        ],
        "ram_mib": [
          {
            "t": 1.119829,
            "value": 68.80859375
          },
          {
            "t": 3.135243,
            "value": 69.45703125
          },
          {
            "t": 5.050411,
            "value": 67.625
          },
          {
            "t": 7.065779,
            "value": 68.359375
          },
          {
            "t": 9.081049,
            "value": 71.19921875
          },
          {
            "t": 11.096741,
            "value": 69.12109375
          },
          {
            "t": 13.114134,
            "value": 69.96875
          },
          {
            "t": 15.136695,
            "value": 68.20703125
          },
          {
            "t": 17.16521,
            "value": 69.4765625
          },
          {
            "t": 19.185999,
            "value": 69.20703125
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
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 4.44537353515625
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.6771231866523
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 101.0488
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 82.7703125
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 85.94140625
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 371858.36424811906
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/400k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 359084.02728577773
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000592
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 6719065.4029306425
        },
        {
          "extra": "OTC OTLP Baseline w/ Gzip (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9013340.58848605
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.084207,
            "value": 99.89599003735991
          },
          {
            "t": 3.104908,
            "value": 101.01386833855798
          },
          {
            "t": 5.134805,
            "value": 100.53023720349563
          },
          {
            "t": 7.052221,
            "value": 100.79419274092616
          },
          {
            "t": 9.072799,
            "value": 100.92370927318295
          },
          {
            "t": 11.091818,
            "value": 100.81081351689612
          },
          {
            "t": 13.111863,
            "value": 100.50734872114784
          },
          {
            "t": 15.130334,
            "value": 100.66390000000001
          },
          {
            "t": 17.148488,
            "value": 101.0488
          },
          {
            "t": 19.170521,
            "value": 100.58237203495631
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.074712,
            "value": 496598.6461987406
          },
          {
            "t": 1.084207,
            "value": 395247.1285147524
          },
          {
            "t": 2.094568,
            "value": 494872.6247351194
          },
          {
            "t": 3.104908,
            "value": 395906.32856266206
          },
          {
            "t": 4.119706,
            "value": 395152.53281933937
          },
          {
            "t": 5.134805,
            "value": 387154.3563731222
          },
          {
            "t": 6.245021,
            "value": 341375.0117094331
          },
          {
            "t": 7.254058,
            "value": 366686.2563018006
          },
          {
            "t": 8.264354,
            "value": 344453.50669506757
          },
          {
            "t": 9.274863,
            "value": 341412.1002385926
          },
          {
            "t": 10.284192,
            "value": 372524.7169158916
          },
          {
            "t": 11.293909,
            "value": 367429.6857436291
          },
          {
            "t": 12.304645,
            "value": 354197.33738582575
          },
          {
            "t": 13.313906,
            "value": 359669.10442393
          },
          {
            "t": 14.323028,
            "value": 364673.448800046
          },
          {
            "t": 15.332225,
            "value": 366628.1211696032
          },
          {
            "t": 16.341666,
            "value": 359604.96948310995
          },
          {
            "t": 17.350543,
            "value": 362779.60544248705
          },
          {
            "t": 18.362574,
            "value": 348803.5445554534
          },
          {
            "t": 19.372382,
            "value": 349571.40367277735
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.074712,
            "value": 362784.63973800204
          },
          {
            "t": 1.084207,
            "value": 353642.1676184627
          },
          {
            "t": 2.094568,
            "value": 357298.0350587562
          },
          {
            "t": 3.104908,
            "value": 348397.5691351426
          },
          {
            "t": 4.119706,
            "value": 348837.8968031076
          },
          {
            "t": 5.134805,
            "value": 355630.3375335805
          },
          {
            "t": 6.143666,
            "value": 364767.79258986126
          },
          {
            "t": 7.153001,
            "value": 363605.74041324237
          },
          {
            "t": 8.163157,
            "value": 354400.7064255422
          },
          {
            "t": 9.173593,
            "value": 339457.42234045506
          },
          {
            "t": 10.182923,
            "value": 361626.02914804866
          },
          {
            "t": 11.192657,
            "value": 369404.21932905103
          },
          {
            "t": 12.203488,
            "value": 358121.18939763424
          },
          {
            "t": 13.212764,
            "value": 359663.75897177774
          },
          {
            "t": 14.221874,
            "value": 363686.8131323642
          },
          {
            "t": 15.231138,
            "value": 355704.751185022
          },
          {
            "t": 16.24038,
            "value": 372556.8297791808
          },
          {
            "t": 17.249305,
            "value": 352850.8065515276
          },
          {
            "t": 18.160609,
            "value": 401622.29069553077
          },
          {
            "t": 19.170521,
            "value": 345574.66393111483
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.084207,
            "value": 9004716.705361487
          },
          {
            "t": 3.104908,
            "value": 8769786.326626256
          },
          {
            "t": 5.134805,
            "value": 8927916.539607674
          },
          {
            "t": 7.052221,
            "value": 9578938.529771317
          },
          {
            "t": 9.072799,
            "value": 8743720.361203577
          },
          {
            "t": 11.091818,
            "value": 9154332.87155792
          },
          {
            "t": 13.111863,
            "value": 9011510.63466408
          },
          {
            "t": 15.130334,
            "value": 9067051.743621781
          },
          {
            "t": 17.148488,
            "value": 9100732.649738325
          },
          {
            "t": 19.170521,
            "value": 8774699.522708086
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.084207,
            "value": 6710165.887320287
          },
          {
            "t": 3.104908,
            "value": 6557856.902134458
          },
          {
            "t": 5.134805,
            "value": 6658451.635723389
          },
          {
            "t": 7.052221,
            "value": 7136190.581490923
          },
          {
            "t": 9.072799,
            "value": 6522377.755275966
          },
          {
            "t": 11.091818,
            "value": 6806720.98677625
          },
          {
            "t": 13.111863,
            "value": 6700211.62894886
          },
          {
            "t": 15.130334,
            "value": 6780590.853175498
          },
          {
            "t": 17.148488,
            "value": 6789385.2500849785
          },
          {
            "t": 19.170521,
            "value": 6528702.548375818
          }
        ],
        "ram_mib": [
          {
            "t": 1.084207,
            "value": 83.7109375
          },
          {
            "t": 3.104908,
            "value": 81.671875
          },
          {
            "t": 5.134805,
            "value": 84.2265625
          },
          {
            "t": 7.052221,
            "value": 77.25390625
          },
          {
            "t": 9.072799,
            "value": 83.33203125
          },
          {
            "t": 11.091818,
            "value": 83.62109375
          },
          {
            "t": 13.111863,
            "value": 82.48828125
          },
          {
            "t": 15.130334,
            "value": 85.94140625
          },
          {
            "t": 17.148488,
            "value": 81.671875
          },
          {
            "t": 19.170521,
            "value": 83.78515625
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
