window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otap_zstd_baseline"] = {
  "name": "DFE OTAP Baseline w/ Zstd (Logs)",
  "slug": "dfe_logs_otap_zstd_baseline",
  "description": "Dataflow Engine baseline for OTAP logs with zstd compression",
  "meta": {
    "binary": "dfe",
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
      "name": "1000k",
      "metrics": [
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/1000k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -7.137362480163574
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/1000k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 44.74458069737388
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/1000k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 47.04531328320802
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/1000k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 16.68515625
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/1000k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 16.96484375
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/1000k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 971695.3681630954
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/1000k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 1018632.0490928512
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/1000k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000623
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/1000k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 7417921.117422042
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/1000k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 7362071.892531784
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.070152,
            "value": 44.29440401505646
          },
          {
            "t": 2.108221,
            "value": 43.66061519146265
          },
          {
            "t": 4.13807,
            "value": 43.85376558603492
          },
          {
            "t": 6.073351,
            "value": 43.83980037429819
          },
          {
            "t": 8.107464,
            "value": 43.64207629768605
          },
          {
            "t": 10.168094,
            "value": 47.04531328320802
          },
          {
            "t": 12.196856,
            "value": 45.87612742036227
          },
          {
            "t": 14.130697,
            "value": 45.675979899497484
          },
          {
            "t": 16.161401,
            "value": 45.4578
          },
          {
            "t": 18.198428,
            "value": 44.099924906132664
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.278063,
            "value": 977685.3104737472
          },
          {
            "t": 0.379967,
            "value": 88910.38544430298
          },
          {
            "t": 1.398375,
            "value": 990800.96059338
          },
          {
            "t": 2.412461,
            "value": 986109.6593385573
          },
          {
            "t": 3.427726,
            "value": 984964.5166532872
          },
          {
            "t": 4.443284,
            "value": 984680.3432201805
          },
          {
            "t": 5.463608,
            "value": 980080.8370674413
          },
          {
            "t": 6.483753,
            "value": 490126.40359948826
          },
          {
            "t": 6.58564,
            "value": 445620.0892666163
          },
          {
            "t": 7.598986,
            "value": 1031419.050891109
          },
          {
            "t": 8.616294,
            "value": 982986.4701742246
          },
          {
            "t": 9.660224,
            "value": 957918.6343911947
          },
          {
            "t": 10.674469,
            "value": 985955.0700274588
          },
          {
            "t": 11.688882,
            "value": 985791.7830311717
          },
          {
            "t": 12.708757,
            "value": 980512.3176859911
          },
          {
            "t": 13.824905,
            "value": 895938.5314492345
          },
          {
            "t": 14.839489,
            "value": 985625.6357285352
          },
          {
            "t": 15.855703,
            "value": 984044.6992464188
          },
          {
            "t": 16.873441,
            "value": 982571.1528900363
          },
          {
            "t": 17.892342,
            "value": 883304.6586469146
          },
          {
            "t": 17.994567,
            "value": 89196.04040937414
          },
          {
            "t": 19.008213,
            "value": 905198.613194772
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.070152,
            "value": 1085519.6450860444
          },
          {
            "t": 1.093681,
            "value": 977011.8873036329
          },
          {
            "t": 2.108221,
            "value": 985668.3817296509
          },
          {
            "t": 3.123742,
            "value": 984716.2195562673
          },
          {
            "t": 4.13807,
            "value": 985874.3917155004
          },
          {
            "t": 5.154325,
            "value": 984004.9987453937
          },
          {
            "t": 6.174434,
            "value": 980287.4006601257
          },
          {
            "t": 7.19382,
            "value": 980982.6699601526
          },
          {
            "t": 8.208633,
            "value": 985403.2220714556
          },
          {
            "t": 9.153303,
            "value": 1056453.5763811702
          },
          {
            "t": 10.168094,
            "value": 987395.434133728
          },
          {
            "t": 11.182503,
            "value": 984809.8745180692
          },
          {
            "t": 12.196856,
            "value": 986835.9437000728
          },
          {
            "t": 13.217639,
            "value": 978660.4988523516
          },
          {
            "t": 14.231894,
            "value": 984959.403700253
          },
          {
            "t": 15.248508,
            "value": 985624.829089507
          },
          {
            "t": 16.262494,
            "value": 986206.910154578
          },
          {
            "t": 17.279701,
            "value": 982100.988294418
          },
          {
            "t": 18.198428,
            "value": 1633782.3967293876
          },
          {
            "t": 19.212491,
            "value": 985145.8933024872
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.070152,
            "value": 7267103.259273952
          },
          {
            "t": 2.108221,
            "value": 7285609.5647399565
          },
          {
            "t": 4.13807,
            "value": 7312527.6806304315
          },
          {
            "t": 6.073351,
            "value": 7672730.7300593555
          },
          {
            "t": 8.107464,
            "value": 7287365.057890098
          },
          {
            "t": 10.168094,
            "value": 7208332.40319708
          },
          {
            "t": 12.196856,
            "value": 7314419.828447103
          },
          {
            "t": 14.130697,
            "value": 7673359.909113521
          },
          {
            "t": 16.161401,
            "value": 7311113.781230548
          },
          {
            "t": 18.198428,
            "value": 7288156.710735792
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.070152,
            "value": 7330498.21438566
          },
          {
            "t": 2.108221,
            "value": 7344754.765417657
          },
          {
            "t": 4.13807,
            "value": 7366569.6315341685
          },
          {
            "t": 6.073351,
            "value": 7735440.4864203185
          },
          {
            "t": 8.107464,
            "value": 7343819.148690362
          },
          {
            "t": 10.168094,
            "value": 7264104.667019309
          },
          {
            "t": 12.196856,
            "value": 7370588.072923291
          },
          {
            "t": 14.130697,
            "value": 7732302.190304167
          },
          {
            "t": 16.161401,
            "value": 7367327.783862148
          },
          {
            "t": 18.198428,
            "value": 7323806.213663343
          }
        ],
        "ram_mib": [
          {
            "t": 0.070152,
            "value": 16.5234375
          },
          {
            "t": 2.108221,
            "value": 16.54296875
          },
          {
            "t": 4.13807,
            "value": 16.421875
          },
          {
            "t": 6.073351,
            "value": 16.71484375
          },
          {
            "t": 8.107464,
            "value": 16.96484375
          },
          {
            "t": 10.168094,
            "value": 16.671875
          },
          {
            "t": 12.196856,
            "value": 16.6640625
          },
          {
            "t": 14.130697,
            "value": 16.53515625
          },
          {
            "t": 16.161401,
            "value": 16.85546875
          },
          {
            "t": 18.198428,
            "value": 16.95703125
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
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -0.052631575614213943
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 7.482810379553498
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 8.125217391304348
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 13.593359375
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 13.66015625
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 99322.82222558293
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/100k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 99375.09739517535
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000712
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 769844.4535533814
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 736855.5295729067
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.048006,
            "value": 7.440844196151458
          },
          {
            "t": 3.062644,
            "value": 7.922189054726368
          },
          {
            "t": 5.076427,
            "value": 8.001592039800995
          },
          {
            "t": 7.087517,
            "value": 7.36923076923077
          },
          {
            "t": 9.104061,
            "value": 6.863622828784119
          },
          {
            "t": 11.115353,
            "value": 6.994470734744708
          },
          {
            "t": 13.131015,
            "value": 8.125217391304348
          },
          {
            "t": 15.143447,
            "value": 7.043885429638855
          },
          {
            "t": 17.158347,
            "value": 7.487324643078833
          },
          {
            "t": 19.170942,
            "value": 7.579726708074534
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.041401,
            "value": 99231.1569955981
          },
          {
            "t": 1.048006,
            "value": 99343.83397658466
          },
          {
            "t": 2.054623,
            "value": 99342.64968702097
          },
          {
            "t": 3.062644,
            "value": 99204.28245046482
          },
          {
            "t": 4.068821,
            "value": 99386.09210904244
          },
          {
            "t": 5.076427,
            "value": 99245.14145410011
          },
          {
            "t": 6.081853,
            "value": 99460.32825886739
          },
          {
            "t": 7.087517,
            "value": 99436.79002131926
          },
          {
            "t": 8.095683,
            "value": 99190.01434287608
          },
          {
            "t": 9.104061,
            "value": 99169.16077106007
          },
          {
            "t": 10.109673,
            "value": 99441.93187829899
          },
          {
            "t": 11.115353,
            "value": 99435.20801845519
          },
          {
            "t": 12.12345,
            "value": 99196.80348220459
          },
          {
            "t": 13.131015,
            "value": 99249.17995365063
          },
          {
            "t": 14.137592,
            "value": 99346.59742871136
          },
          {
            "t": 15.143447,
            "value": 99417.90814779467
          },
          {
            "t": 16.149372,
            "value": 99410.98988493178
          },
          {
            "t": 17.158347,
            "value": 99110.48341138284
          },
          {
            "t": 18.164652,
            "value": 99373.4503952579
          },
          {
            "t": 19.170942,
            "value": 99374.93167973448
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.041401,
            "value": 98238.84542564211
          },
          {
            "t": 1.048006,
            "value": 100337.27231635051
          },
          {
            "t": 2.054623,
            "value": 99342.64968702097
          },
          {
            "t": 3.062644,
            "value": 99204.28245046482
          },
          {
            "t": 4.068821,
            "value": 98392.23118795201
          },
          {
            "t": 5.076427,
            "value": 100237.59286864112
          },
          {
            "t": 6.081853,
            "value": 98465.72497627873
          },
          {
            "t": 7.087517,
            "value": 100431.15792153245
          },
          {
            "t": 8.095683,
            "value": 99190.01434287608
          },
          {
            "t": 9.104061,
            "value": 99169.16077106007
          },
          {
            "t": 10.109673,
            "value": 99441.93187829899
          },
          {
            "t": 11.115353,
            "value": 99435.20801845519
          },
          {
            "t": 12.12345,
            "value": 99196.80348220459
          },
          {
            "t": 13.131015,
            "value": 99249.17995365063
          },
          {
            "t": 14.137592,
            "value": 99346.59742871136
          },
          {
            "t": 15.143447,
            "value": 99417.90814779467
          },
          {
            "t": 16.149372,
            "value": 98416.87998608247
          },
          {
            "t": 17.158347,
            "value": 100101.58824549668
          },
          {
            "t": 18.164652,
            "value": 99373.4503952579
          },
          {
            "t": 19.170942,
            "value": 99374.93167973448
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.048006,
            "value": 737000.9129482271
          },
          {
            "t": 3.062644,
            "value": 736877.2950773289
          },
          {
            "t": 5.076427,
            "value": 737214.4863672004
          },
          {
            "t": 7.087517,
            "value": 738187.7489321711
          },
          {
            "t": 9.104061,
            "value": 736186.7630956726
          },
          {
            "t": 11.115353,
            "value": 738090.7396837455
          },
          {
            "t": 13.131015,
            "value": 736527.2550655815
          },
          {
            "t": 15.143447,
            "value": 737657.719614874
          },
          {
            "t": 17.158347,
            "value": 736819.1969824806
          },
          {
            "t": 19.170942,
            "value": 733993.1779617856
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.048006,
            "value": 769969.3152093998
          },
          {
            "t": 3.062644,
            "value": 769845.5007797927
          },
          {
            "t": 5.076427,
            "value": 770198.1792477143
          },
          {
            "t": 7.087517,
            "value": 771221.0791163002
          },
          {
            "t": 9.104061,
            "value": 769131.7422282875
          },
          {
            "t": 11.115353,
            "value": 771121.7466185914
          },
          {
            "t": 13.131015,
            "value": 769487.1461584334
          },
          {
            "t": 15.143447,
            "value": 770670.0151856063
          },
          {
            "t": 17.158347,
            "value": 769798.004863765
          },
          {
            "t": 19.170942,
            "value": 767001.806125922
          }
        ],
        "ram_mib": [
          {
            "t": 1.048006,
            "value": 13.66015625
          },
          {
            "t": 3.062644,
            "value": 13.58203125
          },
          {
            "t": 5.076427,
            "value": 13.5859375
          },
          {
            "t": 7.087517,
            "value": 13.51953125
          },
          {
            "t": 9.104061,
            "value": 13.55078125
          },
          {
            "t": 11.115353,
            "value": 13.61328125
          },
          {
            "t": 13.131015,
            "value": 13.63671875
          },
          {
            "t": 15.143447,
            "value": 13.5390625
          },
          {
            "t": 17.158347,
            "value": 13.625
          },
          {
            "t": 19.170942,
            "value": 13.62109375
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
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.6578948497772217
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 12.57605129871261
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 13.934357713583282
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 12.511328125
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 12.78125
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 198338.44576774465
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/200k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 203610.07287893997
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000602
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 1510052.4535731315
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 1476291.744531689
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.084333,
            "value": 11.832211953173136
          },
          {
            "t": 3.097726,
            "value": 13.934357713583282
          },
          {
            "t": 5.111882,
            "value": 13.872836923076923
          },
          {
            "t": 7.127542,
            "value": 13.083434218653489
          },
          {
            "t": 9.141229,
            "value": 12.414558914250462
          },
          {
            "t": 11.154714,
            "value": 11.716224552745219
          },
          {
            "t": 13.171929,
            "value": 11.912235872235872
          },
          {
            "t": 15.088501,
            "value": 11.961820418204182
          },
          {
            "t": 17.10692,
            "value": 12.132906403940886
          },
          {
            "t": 19.131016,
            "value": 12.899926017262638
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.077591,
            "value": 198650.7640108384
          },
          {
            "t": 1.084333,
            "value": 198660.63003232208
          },
          {
            "t": 2.090898,
            "value": 198695.56362480318
          },
          {
            "t": 3.097726,
            "value": 198643.6610821312
          },
          {
            "t": 4.104918,
            "value": 198571.87110302702
          },
          {
            "t": 5.111882,
            "value": 198616.83237931048
          },
          {
            "t": 6.118992,
            "value": 198588.0390424085
          },
          {
            "t": 7.127542,
            "value": 198304.49655445936
          },
          {
            "t": 8.134573,
            "value": 198603.6179621084
          },
          {
            "t": 9.141229,
            "value": 198677.60188187426
          },
          {
            "t": 10.148086,
            "value": 198637.9396478348
          },
          {
            "t": 11.154714,
            "value": 198683.1282261173
          },
          {
            "t": 12.163282,
            "value": 198300.95739702234
          },
          {
            "t": 13.171929,
            "value": 198285.425922052
          },
          {
            "t": 14.182128,
            "value": 197980.79388318537
          },
          {
            "t": 15.19239,
            "value": 197968.4477887914
          },
          {
            "t": 16.201038,
            "value": 198285.22933669624
          },
          {
            "t": 17.212717,
            "value": 197691.16488530452
          },
          {
            "t": 18.224809,
            "value": 197610.49390766848
          },
          {
            "t": 19.236761,
            "value": 197637.83262447233
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.077591,
            "value": 197657.5101907842
          },
          {
            "t": 1.084333,
            "value": 199653.93318248368
          },
          {
            "t": 2.090898,
            "value": 198695.56362480318
          },
          {
            "t": 3.097726,
            "value": 197650.44277672053
          },
          {
            "t": 4.104918,
            "value": 198571.87110302702
          },
          {
            "t": 5.111882,
            "value": 198616.83237931048
          },
          {
            "t": 6.118992,
            "value": 199580.97923762052
          },
          {
            "t": 7.127542,
            "value": 198304.49655445936
          },
          {
            "t": 8.134573,
            "value": 198603.6179621084
          },
          {
            "t": 9.141229,
            "value": 197684.21387246487
          },
          {
            "t": 10.148086,
            "value": 199631.12934607398
          },
          {
            "t": 11.154714,
            "value": 298024.6923391759
          },
          {
            "t": 12.163282,
            "value": 198300.95739702234
          },
          {
            "t": 13.171929,
            "value": 198285.425922052
          },
          {
            "t": 14.182128,
            "value": 197980.79388318537
          },
          {
            "t": 15.19239,
            "value": 197968.4477887914
          },
          {
            "t": 16.201038,
            "value": 198285.22933669624
          },
          {
            "t": 17.212717,
            "value": 197691.16488530452
          },
          {
            "t": 18.224809,
            "value": 197610.49390766848
          },
          {
            "t": 19.236761,
            "value": 197637.83262447233
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.084333,
            "value": 1471887.7356925684
          },
          {
            "t": 3.097726,
            "value": 1472001.7403457745
          },
          {
            "t": 5.111882,
            "value": 1467734.8725719359
          },
          {
            "t": 7.127542,
            "value": 1470383.3979937092
          },
          {
            "t": 9.141229,
            "value": 1468128.363544086
          },
          {
            "t": 11.154714,
            "value": 1471892.7630451678
          },
          {
            "t": 13.171929,
            "value": 1469180.5286000748
          },
          {
            "t": 15.088501,
            "value": 1546381.7691169442
          },
          {
            "t": 17.10692,
            "value": 1464719.1688148
          },
          {
            "t": 19.131016,
            "value": 1460607.1055918296
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.084333,
            "value": 1505465.0182216938
          },
          {
            "t": 3.097726,
            "value": 1505611.1747681648
          },
          {
            "t": 5.111882,
            "value": 1501345.4767158057
          },
          {
            "t": 7.127542,
            "value": 1503957.0165603326
          },
          {
            "t": 9.141229,
            "value": 1501776.0952918702
          },
          {
            "t": 11.154714,
            "value": 1505531.4541702569
          },
          {
            "t": 13.171929,
            "value": 1502808.0794560816
          },
          {
            "t": 15.088501,
            "value": 1581775.6911819645
          },
          {
            "t": 17.10692,
            "value": 1498225.095978585
          },
          {
            "t": 19.131016,
            "value": 1494029.4333865587
          }
        ],
        "ram_mib": [
          {
            "t": 1.084333,
            "value": 12.60546875
          },
          {
            "t": 3.097726,
            "value": 12.51953125
          },
          {
            "t": 5.111882,
            "value": 12.50390625
          },
          {
            "t": 7.127542,
            "value": 12.55078125
          },
          {
            "t": 9.141229,
            "value": 12.78125
          },
          {
            "t": 11.154714,
            "value": 12.52734375
          },
          {
            "t": 13.171929,
            "value": 12.33984375
          },
          {
            "t": 15.088501,
            "value": 12.36328125
          },
          {
            "t": 17.10692,
            "value": 12.453125
          },
          {
            "t": 19.131016,
            "value": 12.46875
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
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 1.7241379022598267
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 15.91556311676158
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 16.958802992518702
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 12.633984375
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 12.8359375
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 302005.18949055264
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/300k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 298454.38328957045
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000795
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 2243519.5730398027
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 2210629.536115697
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.106576,
            "value": 16.492607879924954
          },
          {
            "t": 3.123452,
            "value": 16.831661481020536
          },
          {
            "t": 5.039962,
            "value": 16.958802992518702
          },
          {
            "t": 7.056291,
            "value": 15.68479101684342
          },
          {
            "t": 9.073158,
            "value": 15.336059850374065
          },
          {
            "t": 11.091192,
            "value": 16.05303426791277
          },
          {
            "t": 13.111232,
            "value": 14.215369211514393
          },
          {
            "t": 15.140259,
            "value": 16.42272218769422
          },
          {
            "t": 17.169945,
            "value": 15.455759849906192
          },
          {
            "t": 19.19612,
            "value": 15.704822429906542
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.097724,
            "value": 296578.7312381716
          },
          {
            "t": 1.106576,
            "value": 297367.7011097763
          },
          {
            "t": 2.115046,
            "value": 297480.34150743205
          },
          {
            "t": 3.123452,
            "value": 297499.22154370364
          },
          {
            "t": 4.131452,
            "value": 297619.0476190476
          },
          {
            "t": 5.140778,
            "value": 297228.05119455955
          },
          {
            "t": 6.148842,
            "value": 297600.15237127803
          },
          {
            "t": 7.157134,
            "value": 297532.8575452349
          },
          {
            "t": 8.165489,
            "value": 297514.2682884501
          },
          {
            "t": 9.173855,
            "value": 297511.0227833941
          },
          {
            "t": 10.183862,
            "value": 298017.73651073704
          },
          {
            "t": 11.192244,
            "value": 296514.61450125056
          },
          {
            "t": 12.203797,
            "value": 297562.2631735559
          },
          {
            "t": 13.222043,
            "value": 293642.2043396193
          },
          {
            "t": 14.232776,
            "value": 296814.29220179806
          },
          {
            "t": 15.246111,
            "value": 394736.1928681038
          },
          {
            "t": 16.262717,
            "value": 295099.5764337413
          },
          {
            "t": 17.274176,
            "value": 296601.246318437
          },
          {
            "t": 18.288678,
            "value": 295711.5905143607
          },
          {
            "t": 19.302692,
            "value": 295853.90339778346
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.097724,
            "value": 297570.633349336
          },
          {
            "t": 1.106576,
            "value": 297367.7011097762
          },
          {
            "t": 2.115046,
            "value": 297480.34150743205
          },
          {
            "t": 3.123452,
            "value": 297499.22154370364
          },
          {
            "t": 4.131452,
            "value": 297619.04761904763
          },
          {
            "t": 5.140778,
            "value": 297228.05119455955
          },
          {
            "t": 6.148842,
            "value": 297600.152371278
          },
          {
            "t": 7.157134,
            "value": 297532.8575452349
          },
          {
            "t": 8.165489,
            "value": 297514.26828845
          },
          {
            "t": 9.173855,
            "value": 297511.0227833941
          },
          {
            "t": 10.183862,
            "value": 297027.6443628608
          },
          {
            "t": 11.192244,
            "value": 297506.3021751678
          },
          {
            "t": 12.203797,
            "value": 296573.6842261355
          },
          {
            "t": 13.222043,
            "value": 294624.2852905879
          },
          {
            "t": 14.13067,
            "value": 330168.4849778842
          },
          {
            "t": 15.140259,
            "value": 297150.6226791298
          },
          {
            "t": 16.154989,
            "value": 295645.1469849123
          },
          {
            "t": 17.169945,
            "value": 295579.3157535893
          },
          {
            "t": 18.182049,
            "value": 296412.22641151503
          },
          {
            "t": 19.19612,
            "value": 295837.27372146526
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.106576,
            "value": 2197973.144486707
          },
          {
            "t": 3.123452,
            "value": 2205418.1813854696
          },
          {
            "t": 5.039962,
            "value": 2317088.8750906596
          },
          {
            "t": 7.056291,
            "value": 2206118.644328381
          },
          {
            "t": 9.073158,
            "value": 2205365.549637135
          },
          {
            "t": 11.091192,
            "value": 2196867.84266271
          },
          {
            "t": 13.111232,
            "value": 2201999.960396824
          },
          {
            "t": 15.140259,
            "value": 2192170.4344003308
          },
          {
            "t": 17.169945,
            "value": 2187939.4152593063
          },
          {
            "t": 19.19612,
            "value": 2195353.313509445
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.106576,
            "value": 2233961.9517148104
          },
          {
            "t": 3.123452,
            "value": 2237811.3478468684
          },
          {
            "t": 5.039962,
            "value": 2351186.2708777934
          },
          {
            "t": 7.056291,
            "value": 2238544.9001626223
          },
          {
            "t": 9.073158,
            "value": 2237860.503444203
          },
          {
            "t": 11.091192,
            "value": 2229267.6932103224
          },
          {
            "t": 13.111232,
            "value": 2234380.0122769848
          },
          {
            "t": 15.140259,
            "value": 2220903.9110864466
          },
          {
            "t": 17.169945,
            "value": 2223722.2900488055
          },
          {
            "t": 19.19612,
            "value": 2227556.8497291696
          }
        ],
        "ram_mib": [
          {
            "t": 1.106576,
            "value": 12.66015625
          },
          {
            "t": 3.123452,
            "value": 12.56640625
          },
          {
            "t": 5.039962,
            "value": 12.55078125
          },
          {
            "t": 7.056291,
            "value": 12.609375
          },
          {
            "t": 9.073158,
            "value": 12.8046875
          },
          {
            "t": 11.091192,
            "value": 12.5546875
          },
          {
            "t": 13.111232,
            "value": 12.60546875
          },
          {
            "t": 15.140259,
            "value": 12.5859375
          },
          {
            "t": 17.169945,
            "value": 12.56640625
          },
          {
            "t": 19.19612,
            "value": 12.8359375
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
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.6180765628814697
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 21.64895757762922
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 22.839550561797754
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 13.526171875
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 13.65234375
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 392832.21015329397
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/400k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 403116.85820230143
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000709
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 3023508.8326820657
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 2957551.719835339
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.089577,
            "value": 21.170143302180687
          },
          {
            "t": 2.106829,
            "value": 20.86071028037383
          },
          {
            "t": 4.125261,
            "value": 21.930759651307596
          },
          {
            "t": 6.138322,
            "value": 20.283146067415732
          },
          {
            "t": 8.156388,
            "value": 20.907862928348912
          },
          {
            "t": 10.170031,
            "value": 21.694903426791274
          },
          {
            "t": 12.18428,
            "value": 22.839550561797754
          },
          {
            "t": 14.197345,
            "value": 21.985059337913803
          },
          {
            "t": 16.216536,
            "value": 22.628271990018714
          },
          {
            "t": 18.130254,
            "value": 22.189168230143842
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.19014,
            "value": 397393.49605934677
          },
          {
            "t": 1.201307,
            "value": 395582.52988873253
          },
          {
            "t": 2.207299,
            "value": 397617.4760833088
          },
          {
            "t": 3.218917,
            "value": 395406.17110411247
          },
          {
            "t": 4.326338,
            "value": 361199.57992488856
          },
          {
            "t": 5.33321,
            "value": 397269.9608291819
          },
          {
            "t": 6.339556,
            "value": 397477.6071053097
          },
          {
            "t": 7.350831,
            "value": 396529.1340139922
          },
          {
            "t": 8.35765,
            "value": 396297.64634954254
          },
          {
            "t": 9.364392,
            "value": 397321.26006464416
          },
          {
            "t": 10.371146,
            "value": 397316.5241955831
          },
          {
            "t": 11.377902,
            "value": 397315.73489504907
          },
          {
            "t": 12.385375,
            "value": 398025.5550272811
          },
          {
            "t": 13.391991,
            "value": 397370.993506958
          },
          {
            "t": 14.403944,
            "value": 394287.08645559626
          },
          {
            "t": 15.511027,
            "value": 361309.8566232161
          },
          {
            "t": 16.518034,
            "value": 398209.744321539
          },
          {
            "t": 17.526084,
            "value": 395813.699717276
          },
          {
            "t": 18.532735,
            "value": 397357.177413026
          },
          {
            "t": 19.539368,
            "value": 398357.6934195482
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.19014,
            "value": 397393.49605934677
          },
          {
            "t": 1.201307,
            "value": 593373.7948330988
          },
          {
            "t": 2.308656,
            "value": 361223.06517638074
          },
          {
            "t": 3.320076,
            "value": 395483.5775444425
          },
          {
            "t": 4.326338,
            "value": 397510.7874489944
          },
          {
            "t": 5.33321,
            "value": 397269.9608291819
          },
          {
            "t": 6.339556,
            "value": 397477.6071053097
          },
          {
            "t": 7.452315,
            "value": 359466.8746781648
          },
          {
            "t": 8.459369,
            "value": 397198.16415008536
          },
          {
            "t": 9.465756,
            "value": 397461.4139491071
          },
          {
            "t": 10.472764,
            "value": 397216.30811274593
          },
          {
            "t": 11.479896,
            "value": 397167.40208830626
          },
          {
            "t": 12.486661,
            "value": 397312.183081454
          },
          {
            "t": 13.493437,
            "value": 397307.8420621867
          },
          {
            "t": 14.505231,
            "value": 395337.39081275434
          },
          {
            "t": 15.511027,
            "value": 397694.9600117718
          },
          {
            "t": 16.518034,
            "value": 397216.7025651262
          },
          {
            "t": 17.526084,
            "value": 396805.71400228166
          },
          {
            "t": 18.532735,
            "value": 397357.177413026
          },
          {
            "t": 19.539368,
            "value": 397364.28271276626
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.089577,
            "value": 2944641.6017687023
          },
          {
            "t": 2.106829,
            "value": 2937653.302611672
          },
          {
            "t": 4.125261,
            "value": 2940571.195858964
          },
          {
            "t": 6.138322,
            "value": 2943748.3513912396
          },
          {
            "t": 8.156388,
            "value": 2936567.485899866
          },
          {
            "t": 10.170031,
            "value": 2947413.220714893
          },
          {
            "t": 12.18428,
            "value": 2945761.4227436627
          },
          {
            "t": 14.197345,
            "value": 2944191.071823314
          },
          {
            "t": 16.216536,
            "value": 2942128.80307014
          },
          {
            "t": 18.130254,
            "value": 3092840.7424709387
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.089577,
            "value": 3010682.763845934
          },
          {
            "t": 2.106829,
            "value": 3003231.6240112786
          },
          {
            "t": 4.125261,
            "value": 3001887.6038429835
          },
          {
            "t": 6.138322,
            "value": 3009494.9929485493
          },
          {
            "t": 8.156388,
            "value": 3005679.199788312
          },
          {
            "t": 10.170031,
            "value": 3013265.012715759
          },
          {
            "t": 12.18428,
            "value": 3011410.2079732944
          },
          {
            "t": 14.197345,
            "value": 3009643.007056404
          },
          {
            "t": 16.216536,
            "value": 3004061.032363951
          },
          {
            "t": 18.130254,
            "value": 3165732.882274191
          }
        ],
        "ram_mib": [
          {
            "t": 0.089577,
            "value": 13.65234375
          },
          {
            "t": 2.106829,
            "value": 13.55078125
          },
          {
            "t": 4.125261,
            "value": 13.5078125
          },
          {
            "t": 6.138322,
            "value": 13.390625
          },
          {
            "t": 8.156388,
            "value": 13.484375
          },
          {
            "t": 10.170031,
            "value": 13.47265625
          },
          {
            "t": 12.18428,
            "value": 13.6171875
          },
          {
            "t": 14.197345,
            "value": 13.50390625
          },
          {
            "t": 16.216536,
            "value": 13.5390625
          },
          {
            "t": 18.130254,
            "value": 13.54296875
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
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/600k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.6315789222717285
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/600k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 28.947609241411488
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/600k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 30.04706757594544
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/600k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 14.53046875
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/600k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 14.7421875
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/600k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 585448.071911511
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/600k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 610333.5415422743
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/600k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.002174
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/600k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 4514742.574448302
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/600k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 4452035.8539465135
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.14274,
            "value": 29.10613466334165
          },
          {
            "t": 2.060945,
            "value": 29.300199004975124
          },
          {
            "t": 4.085183,
            "value": 29.693490976975735
          },
          {
            "t": 6.104888,
            "value": 30.04706757594544
          },
          {
            "t": 8.125564,
            "value": 28.280199501246884
          },
          {
            "t": 10.120539,
            "value": 28.995289719626165
          },
          {
            "t": 12.139744,
            "value": 28.49227698066126
          },
          {
            "t": 14.158346,
            "value": 28.329850746268658
          },
          {
            "t": 16.177055,
            "value": 28.802691588785045
          },
          {
            "t": 18.197263,
            "value": 28.42889165628892
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.243939,
            "value": 593322.4970259586
          },
          {
            "t": 1.253155,
            "value": 594520.895427738
          },
          {
            "t": 2.268026,
            "value": 592193.4906012685
          },
          {
            "t": 3.378101,
            "value": 539603.1799653177
          },
          {
            "t": 4.387919,
            "value": 594166.4735625627
          },
          {
            "t": 5.398257,
            "value": 593860.6684099777
          },
          {
            "t": 6.407635,
            "value": 594425.4778685487
          },
          {
            "t": 7.41789,
            "value": 593909.4585030512
          },
          {
            "t": 8.428389,
            "value": 593766.0502385454
          },
          {
            "t": 9.414251,
            "value": 608604.44971
          },
          {
            "t": 10.524706,
            "value": 541219.5901679943
          },
          {
            "t": 11.533856,
            "value": 593568.8450676311
          },
          {
            "t": 12.543352,
            "value": 595346.5887928235
          },
          {
            "t": 13.552509,
            "value": 593564.7277876484
          },
          {
            "t": 14.562115,
            "value": 594291.2383642728
          },
          {
            "t": 15.571365,
            "value": 594500.866980431
          },
          {
            "t": 16.582226,
            "value": 594542.6720389846
          },
          {
            "t": 17.591788,
            "value": 594317.1395119863
          },
          {
            "t": 18.605974,
            "value": 590621.4441926826
          },
          {
            "t": 19.716204,
            "value": 540428.5598479594
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.14274,
            "value": 594357.7617675407
          },
          {
            "t": 1.152122,
            "value": 594423.122266892
          },
          {
            "t": 2.161741,
            "value": 594283.5861844914
          },
          {
            "t": 3.176043,
            "value": 590553.8981486775
          },
          {
            "t": 4.185901,
            "value": 595133.1771397564
          },
          {
            "t": 5.195871,
            "value": 594077.0517936177
          },
          {
            "t": 6.205747,
            "value": 594132.3489220459
          },
          {
            "t": 7.215636,
            "value": 594124.7008334579
          },
          {
            "t": 8.226425,
            "value": 890393.54405321
          },
          {
            "t": 9.207005,
            "value": 611882.7632625588
          },
          {
            "t": 10.221247,
            "value": 591574.7918149711
          },
          {
            "t": 11.231069,
            "value": 594164.12001323
          },
          {
            "t": 12.240506,
            "value": 594390.7346372285
          },
          {
            "t": 13.249863,
            "value": 594437.8450835531
          },
          {
            "t": 14.259102,
            "value": 594507.3466245359
          },
          {
            "t": 15.26856,
            "value": 594378.3693823814
          },
          {
            "t": 16.277922,
            "value": 594434.9004618758
          },
          {
            "t": 17.288895,
            "value": 593487.6599078315
          },
          {
            "t": 18.298033,
            "value": 594566.8481416813
          },
          {
            "t": 19.312586,
            "value": 591393.4511060535
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.14274,
            "value": 4427011.938376183
          },
          {
            "t": 2.060945,
            "value": 4657310.350040793
          },
          {
            "t": 4.085183,
            "value": 4414884.0205548955
          },
          {
            "t": 6.104888,
            "value": 4423944.585966762
          },
          {
            "t": 8.125564,
            "value": 4421323.359113485
          },
          {
            "t": 10.120539,
            "value": 4475386.909610396
          },
          {
            "t": 12.139744,
            "value": 4425761.624005487
          },
          {
            "t": 14.158346,
            "value": 4426641.804575642
          },
          {
            "t": 16.177055,
            "value": 4426402.220428998
          },
          {
            "t": 18.197263,
            "value": 4421691.726792489
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.14274,
            "value": 4489468.965175608
          },
          {
            "t": 2.060945,
            "value": 4723183.914128052
          },
          {
            "t": 4.085183,
            "value": 4476631.206409523
          },
          {
            "t": 6.104888,
            "value": 4485733.807660029
          },
          {
            "t": 8.125564,
            "value": 4479822.0991391
          },
          {
            "t": 10.120539,
            "value": 4542996.779408263
          },
          {
            "t": 12.139744,
            "value": 4487881.616774919
          },
          {
            "t": 14.158346,
            "value": 4488606.471211264
          },
          {
            "t": 16.177055,
            "value": 4485117.964005709
          },
          {
            "t": 18.197263,
            "value": 4487982.920570555
          }
        ],
        "ram_mib": [
          {
            "t": 0.14274,
            "value": 14.48046875
          },
          {
            "t": 2.060945,
            "value": 14.375
          },
          {
            "t": 4.085183,
            "value": 14.4296875
          },
          {
            "t": 6.104888,
            "value": 14.7421875
          },
          {
            "t": 8.125564,
            "value": 14.51953125
          },
          {
            "t": 10.120539,
            "value": 14.6328125
          },
          {
            "t": 12.139744,
            "value": 14.59375
          },
          {
            "t": 14.158346,
            "value": 14.5234375
          },
          {
            "t": 16.177055,
            "value": 14.390625
          },
          {
            "t": 18.197263,
            "value": 14.6171875
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
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/800k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 1.9291566610336304
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/800k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 36.672266554791
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/800k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 37.920797011207966
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/800k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 15.503515625
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/800k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 15.6484375
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/800k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 793659.1249676434
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/800k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 794765.3408272733
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/800k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000635
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/800k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 5952279.886869678
        },
        {
          "extra": "DFE OTAP Baseline w/ Zstd (Logs)/800k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 5893957.083819194
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.1027,
            "value": 36.839726197884254
          },
          {
            "t": 2.125162,
            "value": 36.27313432835821
          },
          {
            "t": 4.14735,
            "value": 36.98996254681648
          },
          {
            "t": 6.07341,
            "value": 36.52625782227785
          },
          {
            "t": 8.09398,
            "value": 36.84571428571429
          },
          {
            "t": 10.116457,
            "value": 36.051343283582085
          },
          {
            "t": 12.142265,
            "value": 36.673951462352214
          },
          {
            "t": 14.168462,
            "value": 35.74588819875776
          },
          {
            "t": 16.190299,
            "value": 36.8558904109589
          },
          {
            "t": 18.21169,
            "value": 37.920797011207966
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.304872,
            "value": 792320.1329436029
          },
          {
            "t": 1.315934,
            "value": 889164.0670898521
          },
          {
            "t": 2.327555,
            "value": 890649.7591489301
          },
          {
            "t": 3.339402,
            "value": 789645.074798858
          },
          {
            "t": 4.35023,
            "value": 791430.3917184724
          },
          {
            "t": 5.365785,
            "value": 689278.2764104356
          },
          {
            "t": 5.467291,
            "value": 89520.62599983349
          },
          {
            "t": 6.477319,
            "value": 729667.1270523604
          },
          {
            "t": 7.48737,
            "value": 890054.0665768364
          },
          {
            "t": 8.498025,
            "value": 791565.8657009563
          },
          {
            "t": 9.509995,
            "value": 790537.2688913702
          },
          {
            "t": 10.521559,
            "value": 790854.5578925309
          },
          {
            "t": 11.535605,
            "value": 788918.8458906205
          },
          {
            "t": 12.551567,
            "value": 787431.0259635695
          },
          {
            "t": 13.662839,
            "value": 719895.759094083
          },
          {
            "t": 14.673133,
            "value": 791848.7093855849
          },
          {
            "t": 15.684933,
            "value": 790670.0929037359
          },
          {
            "t": 16.695532,
            "value": 791609.7284877581
          },
          {
            "t": 17.705897,
            "value": 791793.0648824929
          },
          {
            "t": 18.721591,
            "value": 787638.7967242103
          },
          {
            "t": 19.833407,
            "value": 719543.5215899033
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.1027,
            "value": 790259.7274147923
          },
          {
            "t": 1.113516,
            "value": 792429.0869950615
          },
          {
            "t": 2.125162,
            "value": 790790.4543684253
          },
          {
            "t": 3.137189,
            "value": 790492.7437706701
          },
          {
            "t": 4.14735,
            "value": 790963.0247059626
          },
          {
            "t": 5.158237,
            "value": 792373.4304625541
          },
          {
            "t": 6.174276,
            "value": 787371.3509028689
          },
          {
            "t": 7.184309,
            "value": 792053.3289506383
          },
          {
            "t": 8.194769,
            "value": 790728.9749223127
          },
          {
            "t": 9.206928,
            "value": 790389.6522186731
          },
          {
            "t": 10.217515,
            "value": 791619.1282888065
          },
          {
            "t": 11.228984,
            "value": 791917.4982129952
          },
          {
            "t": 12.243072,
            "value": 786913.956185262
          },
          {
            "t": 13.157848,
            "value": 875624.1965246137
          },
          {
            "t": 14.168462,
            "value": 792587.4765241725
          },
          {
            "t": 15.178754,
            "value": 791850.2769496344
          },
          {
            "t": 16.190299,
            "value": 790869.4126311732
          },
          {
            "t": 17.201243,
            "value": 791339.5796404153
          },
          {
            "t": 18.21169,
            "value": 790739.1481195946
          },
          {
            "t": 19.227842,
            "value": 787283.7921885703
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.1027,
            "value": 5857801.841207681
          },
          {
            "t": 2.125162,
            "value": 5862060.696319634
          },
          {
            "t": 4.14735,
            "value": 5855537.170629042
          },
          {
            "t": 6.07341,
            "value": 6160269.669688379
          },
          {
            "t": 8.09398,
            "value": 5882006.067594787
          },
          {
            "t": 10.116457,
            "value": 5861771.9756516395
          },
          {
            "t": 12.142265,
            "value": 5859278.371889142
          },
          {
            "t": 14.168462,
            "value": 5856951.224387363
          },
          {
            "t": 16.190299,
            "value": 5871582.1305080475
          },
          {
            "t": 18.21169,
            "value": 5872311.690316223
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.1027,
            "value": 5913685.494567964
          },
          {
            "t": 2.125162,
            "value": 5919964.380047685
          },
          {
            "t": 4.14735,
            "value": 5913696.946080186
          },
          {
            "t": 6.07341,
            "value": 6222457.244322606
          },
          {
            "t": 8.09398,
            "value": 5933584.582568285
          },
          {
            "t": 10.116457,
            "value": 5926664.679005003
          },
          {
            "t": 12.142265,
            "value": 5917981.368421884
          },
          {
            "t": 14.168462,
            "value": 5915558.5562509475
          },
          {
            "t": 16.190299,
            "value": 5929367.204181148
          },
          {
            "t": 18.21169,
            "value": 5929838.413251073
          }
        ],
        "ram_mib": [
          {
            "t": 0.1027,
            "value": 15.62109375
          },
          {
            "t": 2.125162,
            "value": 15.48828125
          },
          {
            "t": 4.14735,
            "value": 15.515625
          },
          {
            "t": 6.07341,
            "value": 15.4453125
          },
          {
            "t": 8.09398,
            "value": 15.41015625
          },
          {
            "t": 10.116457,
            "value": 15.41796875
          },
          {
            "t": 12.142265,
            "value": 15.6484375
          },
          {
            "t": 14.168462,
            "value": 15.55859375
          },
          {
            "t": 16.190299,
            "value": 15.5234375
          },
          {
            "t": 18.21169,
            "value": 15.40625
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
