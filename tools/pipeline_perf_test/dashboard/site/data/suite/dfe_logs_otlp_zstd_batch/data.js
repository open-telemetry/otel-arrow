window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otlp_zstd_batch"] = {
  "name": "DFE OTLP Batch Processor w/ Zstd (Logs)",
  "slug": "dfe_logs_otlp_zstd_batch",
  "description": "Dataflow Engine OTLP logs through a batch processor with zstd compression",
  "meta": {
    "binary": "dfe",
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
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 4.949999809265137
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 10.768050679753111
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 11.503980099502488
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 17.548046875
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 25.4375
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 104301.85164469677
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/100k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 99138.90998828429
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000969
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 36360669.11727246
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 35980030.36960243
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.04903,
            "value": 11.412104607721046
          },
          {
            "t": 3.066576,
            "value": 11.37216957605985
          },
          {
            "t": 5.082225,
            "value": 11.211172069825437
          },
          {
            "t": 7.101119,
            "value": 10.025840049597026
          },
          {
            "t": 9.119964,
            "value": 11.202583850931678
          },
          {
            "t": 11.139044,
            "value": 9.827872738615097
          },
          {
            "t": 13.157502,
            "value": 10.773499377334995
          },
          {
            "t": 15.178923,
            "value": 10.242791277258567
          },
          {
            "t": 17.197895,
            "value": 10.108493150684932
          },
          {
            "t": 19.115769,
            "value": 11.503980099502488
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.042548,
            "value": 99238.14873218304
          },
          {
            "t": 1.04903,
            "value": 99355.97457281897
          },
          {
            "t": 2.056726,
            "value": 99236.27760753244
          },
          {
            "t": 3.066576,
            "value": 99024.60761499233
          },
          {
            "t": 4.074866,
            "value": 99177.81590613813
          },
          {
            "t": 5.082225,
            "value": 99269.47592665575
          },
          {
            "t": 6.090733,
            "value": 99156.37753989061
          },
          {
            "t": 7.101119,
            "value": 98972.07601847214
          },
          {
            "t": 8.109387,
            "value": 99179.97992597207
          },
          {
            "t": 9.119964,
            "value": 197906.74040671813
          },
          {
            "t": 10.128645,
            "value": 99139.37111931325
          },
          {
            "t": 11.139044,
            "value": 98970.80262351803
          },
          {
            "t": 12.148572,
            "value": 99056.1925969364
          },
          {
            "t": 13.157502,
            "value": 99114.90390810065
          },
          {
            "t": 14.168134,
            "value": 98947.98502323299
          },
          {
            "t": 15.178923,
            "value": 98932.61600591222
          },
          {
            "t": 16.187837,
            "value": 99116.47573529556
          },
          {
            "t": 17.197895,
            "value": 99004.21559950024
          },
          {
            "t": 18.208776,
            "value": 98923.61217591389
          },
          {
            "t": 19.217663,
            "value": 99119.12830673801
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.042548,
            "value": 84352.42642235558
          },
          {
            "t": 1.04903,
            "value": 104323.77330145992
          },
          {
            "t": 2.056726,
            "value": 105190.45426398439
          },
          {
            "t": 3.066576,
            "value": 104966.08407189188
          },
          {
            "t": 4.074866,
            "value": 83309.36536115603
          },
          {
            "t": 5.082225,
            "value": 105225.6444822551
          },
          {
            "t": 6.090733,
            "value": 105105.76019228405
          },
          {
            "t": 7.101119,
            "value": 103920.67981939575
          },
          {
            "t": 8.109387,
            "value": 84302.98293707626
          },
          {
            "t": 9.119964,
            "value": 104890.57241556061
          },
          {
            "t": 10.128645,
            "value": 104096.33967527891
          },
          {
            "t": 11.139044,
            "value": 84125.18222999033
          },
          {
            "t": 12.148572,
            "value": 104999.56415275257
          },
          {
            "t": 13.157502,
            "value": 104070.64910350568
          },
          {
            "t": 14.168134,
            "value": 104884.86412462697
          },
          {
            "t": 15.178923,
            "value": 83103.39744496626
          },
          {
            "t": 16.187837,
            "value": 105063.4642794133
          },
          {
            "t": 17.197895,
            "value": 103954.42637947525
          },
          {
            "t": 18.208776,
            "value": 103869.79278470959
          },
          {
            "t": 19.217663,
            "value": 84251.25906072732
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.04903,
            "value": 35860030.91116442
          },
          {
            "t": 3.066576,
            "value": 35798731.23091122
          },
          {
            "t": 5.082225,
            "value": 35842489.93748416
          },
          {
            "t": 7.101119,
            "value": 35774460.17472933
          },
          {
            "t": 9.119964,
            "value": 35783596.06606748
          },
          {
            "t": 11.139044,
            "value": 35773257.622283414
          },
          {
            "t": 13.157502,
            "value": 35791390.25929695
          },
          {
            "t": 15.178923,
            "value": 35724431.47666913
          },
          {
            "t": 17.197895,
            "value": 35786613.18730522
          },
          {
            "t": 19.115769,
            "value": 37665302.83011293
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.04903,
            "value": 37627675.37220249
          },
          {
            "t": 3.066576,
            "value": 34013655.698556565
          },
          {
            "t": 5.082225,
            "value": 37600866.023796804
          },
          {
            "t": 7.101119,
            "value": 33821163.96403179
          },
          {
            "t": 9.119964,
            "value": 38285906.050241604
          },
          {
            "t": 11.139044,
            "value": 33811353.68583711
          },
          {
            "t": 13.157502,
            "value": 37564066.72816576
          },
          {
            "t": 15.178923,
            "value": 33786883.58338021
          },
          {
            "t": 17.197895,
            "value": 37545833.721319556
          },
          {
            "t": 19.115769,
            "value": 39549286.34519264
          }
        ],
        "ram_mib": [
          {
            "t": 1.04903,
            "value": 16.7890625
          },
          {
            "t": 3.066576,
            "value": 16.5390625
          },
          {
            "t": 5.082225,
            "value": 16.421875
          },
          {
            "t": 7.101119,
            "value": 16.82421875
          },
          {
            "t": 9.119964,
            "value": 16.71875
          },
          {
            "t": 11.139044,
            "value": 17.01953125
          },
          {
            "t": 13.157502,
            "value": 16.7578125
          },
          {
            "t": 15.178923,
            "value": 25.4375
          },
          {
            "t": 17.197895,
            "value": 16.51953125
          },
          {
            "t": 19.115769,
            "value": 16.453125
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
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.1315789222717285
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 18.324152345291353
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 20.556594694632942
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 25.373828125
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 32.15625
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 198052.81766355946
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/200k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 202274.46982954585
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000649
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 71928234.88372776
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 71910976.67249511
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.072571,
            "value": 18.425560371517026
          },
          {
            "t": 3.092485,
            "value": 18.819987600743957
          },
          {
            "t": 5.110834,
            "value": 16.50785185185185
          },
          {
            "t": 7.130222,
            "value": 16.924356435643563
          },
          {
            "t": 9.14802,
            "value": 17.10195786864932
          },
          {
            "t": 11.169998,
            "value": 16.752643962848296
          },
          {
            "t": 13.086604,
            "value": 19.98030921459493
          },
          {
            "t": 15.111301,
            "value": 19.407425742574258
          },
          {
            "t": 17.132124,
            "value": 20.556594694632942
          },
          {
            "t": 19.149644,
            "value": 18.764835709857408
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.064175,
            "value": 198101.59243965085
          },
          {
            "t": 1.072571,
            "value": 198334.7811772359
          },
          {
            "t": 2.080828,
            "value": 198362.12394260592
          },
          {
            "t": 3.092485,
            "value": 197695.46397642678
          },
          {
            "t": 4.101569,
            "value": 198199.555240198
          },
          {
            "t": 5.110834,
            "value": 198164.01044324334
          },
          {
            "t": 6.119424,
            "value": 198296.63193170662
          },
          {
            "t": 7.130222,
            "value": 197863.47024825928
          },
          {
            "t": 8.138705,
            "value": 198317.67119525067
          },
          {
            "t": 9.14802,
            "value": 198154.19368581663
          },
          {
            "t": 10.160467,
            "value": 197541.2046260199
          },
          {
            "t": 11.169998,
            "value": 198111.7964678648
          },
          {
            "t": 12.178,
            "value": 198412.3047374906
          },
          {
            "t": 13.191683,
            "value": 197300.33945523406
          },
          {
            "t": 14.204044,
            "value": 197557.985738289
          },
          {
            "t": 15.215362,
            "value": 197761.73270919733
          },
          {
            "t": 16.224554,
            "value": 198178.34465592273
          },
          {
            "t": 17.232954,
            "value": 198333.99444664817
          },
          {
            "t": 18.241521,
            "value": 198301.15401356577
          },
          {
            "t": 19.250976,
            "value": 198126.71193862034
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.064175,
            "value": 218902.2596458142
          },
          {
            "t": 1.072571,
            "value": 175526.28134185378
          },
          {
            "t": 2.080828,
            "value": 217206.52571715348
          },
          {
            "t": 3.092485,
            "value": 174960.4856191377
          },
          {
            "t": 4.101569,
            "value": 218019.51076421782
          },
          {
            "t": 5.110834,
            "value": 218971.23153978388
          },
          {
            "t": 6.119424,
            "value": 173509.5529402433
          },
          {
            "t": 7.130222,
            "value": 219628.4519755678
          },
          {
            "t": 8.138705,
            "value": 175511.13900779685
          },
          {
            "t": 9.14802,
            "value": 218960.38402282738
          },
          {
            "t": 10.160467,
            "value": 174823.96609402762
          },
          {
            "t": 11.169998,
            "value": 216932.41713231194
          },
          {
            "t": 12.178,
            "value": 176586.95121636664
          },
          {
            "t": 13.191683,
            "value": 216043.8717034813
          },
          {
            "t": 14.204044,
            "value": 174838.81737838575
          },
          {
            "t": 15.215362,
            "value": 217537.90598011704
          },
          {
            "t": 16.224554,
            "value": 305194.650770121
          },
          {
            "t": 17.232954,
            "value": 174533.91511305037
          },
          {
            "t": 18.241521,
            "value": 220114.28095505803
          },
          {
            "t": 19.250976,
            "value": 174351.50650598592
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.072571,
            "value": 71607481.04910904
          },
          {
            "t": 3.092485,
            "value": 71543610.27251655
          },
          {
            "t": 5.110834,
            "value": 71596364.15704122
          },
          {
            "t": 7.130222,
            "value": 71378072.46551925
          },
          {
            "t": 9.14802,
            "value": 71610576.47990532
          },
          {
            "t": 11.169998,
            "value": 71470541.22250588
          },
          {
            "t": 13.086604,
            "value": 75405381.17902167
          },
          {
            "t": 15.111301,
            "value": 71501622.21804053
          },
          {
            "t": 17.132124,
            "value": 71375614.78664881
          },
          {
            "t": 19.149644,
            "value": 71620502.89464292
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.072571,
            "value": 70969181.04697819
          },
          {
            "t": 3.092485,
            "value": 70798590.43503833
          },
          {
            "t": 5.110834,
            "value": 70672007.66567129
          },
          {
            "t": 7.130222,
            "value": 70716793.40473449
          },
          {
            "t": 9.14802,
            "value": 70869209.9010902
          },
          {
            "t": 11.169998,
            "value": 70659784.62673679
          },
          {
            "t": 13.086604,
            "value": 80691187.44280253
          },
          {
            "t": 15.111301,
            "value": 72487604.81197926
          },
          {
            "t": 17.132124,
            "value": 70392315.90297616
          },
          {
            "t": 19.149644,
            "value": 71025673.59927039
          }
        ],
        "ram_mib": [
          {
            "t": 1.072571,
            "value": 25.984375
          },
          {
            "t": 3.092485,
            "value": 25.78515625
          },
          {
            "t": 5.110834,
            "value": 26.26953125
          },
          {
            "t": 7.130222,
            "value": 26.0703125
          },
          {
            "t": 9.14802,
            "value": 26.2265625
          },
          {
            "t": 11.169998,
            "value": 25.6015625
          },
          {
            "t": 13.086604,
            "value": 32.15625
          },
          {
            "t": 15.111301,
            "value": 21.890625
          },
          {
            "t": 17.132124,
            "value": 16.93359375
          },
          {
            "t": 19.149644,
            "value": 26.8203125
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
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -1.758923888206482
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 25.54499664884757
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 28.47283582089552
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 39.062890625
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 50.890625
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 301116.5926666193
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/300k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 306413.00453970005
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000905
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 108288958.45567715
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 107979830.0699412
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.076059,
            "value": 24.99491271820449
          },
          {
            "t": 3.100005,
            "value": 28.47283582089552
          },
          {
            "t": 5.127541,
            "value": 24.49311801242236
          },
          {
            "t": 7.054203,
            "value": 24.770981366459626
          },
          {
            "t": 9.082846,
            "value": 28.05131137352393
          },
          {
            "t": 11.107248,
            "value": 23.712917705735663
          },
          {
            "t": 13.134254,
            "value": 25.026650062266505
          },
          {
            "t": 15.162916,
            "value": 27.531065420560747
          },
          {
            "t": 17.094854,
            "value": 24.55250155183116
          },
          {
            "t": 19.120716,
            "value": 23.843672456575682
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.066406,
            "value": 297135.61269363336
          },
          {
            "t": 1.076059,
            "value": 296141.3475718886
          },
          {
            "t": 2.088627,
            "value": 296276.3982270821
          },
          {
            "t": 3.100005,
            "value": 296625.00074156246
          },
          {
            "t": 4.112591,
            "value": 297258.7019769185
          },
          {
            "t": 5.127541,
            "value": 295581.063106557
          },
          {
            "t": 6.145924,
            "value": 293602.70153763366
          },
          {
            "t": 7.160031,
            "value": 295826.77173118806
          },
          {
            "t": 8.172654,
            "value": 296260.30615540035
          },
          {
            "t": 9.186405,
            "value": 295930.6575283279
          },
          {
            "t": 10.198602,
            "value": 296384.9922495324
          },
          {
            "t": 11.213209,
            "value": 296666.59110374755
          },
          {
            "t": 12.225609,
            "value": 295337.8111418412
          },
          {
            "t": 13.23921,
            "value": 295974.4514853478
          },
          {
            "t": 14.253064,
            "value": 295900.59318205575
          },
          {
            "t": 15.272957,
            "value": 294148.5038136354
          },
          {
            "t": 16.285643,
            "value": 296241.8755665626
          },
          {
            "t": 17.296712,
            "value": 296715.6544212116
          },
          {
            "t": 18.310871,
            "value": 394415.4713412788
          },
          {
            "t": 19.324727,
            "value": 295900.0094688003
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.066406,
            "value": 272374.31163583056
          },
          {
            "t": 1.076059,
            "value": 345663.312048793
          },
          {
            "t": 2.088627,
            "value": 271586.69837482524
          },
          {
            "t": 3.100005,
            "value": 275861.2506896531
          },
          {
            "t": 4.112591,
            "value": 340711.80126922554
          },
          {
            "t": 5.127541,
            "value": 274890.388689098
          },
          {
            "t": 6.145924,
            "value": 271017.8783424311
          },
          {
            "t": 7.160031,
            "value": 343159.0552081782
          },
          {
            "t": 8.172654,
            "value": 272559.48166296835
          },
          {
            "t": 9.186405,
            "value": 279161.2536017227
          },
          {
            "t": 10.198602,
            "value": 339854.7911127972
          },
          {
            "t": 11.213209,
            "value": 274983.318664271
          },
          {
            "t": 12.225609,
            "value": 275582.77360726986
          },
          {
            "t": 13.23921,
            "value": 340370.61920814996
          },
          {
            "t": 14.355032,
            "value": 250039.88091290544
          },
          {
            "t": 15.272957,
            "value": 300678.15998039057
          },
          {
            "t": 16.285643,
            "value": 479911.8384178314
          },
          {
            "t": 17.296712,
            "value": 274956.50643032277
          },
          {
            "t": 18.310871,
            "value": 273132.71390383557
          },
          {
            "t": 19.324727,
            "value": 343244.0109838083
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.076059,
            "value": 107152896.0878882
          },
          {
            "t": 3.100005,
            "value": 106726055.43823798
          },
          {
            "t": 5.127541,
            "value": 106871764.54573433
          },
          {
            "t": 7.054203,
            "value": 112482488.36588877
          },
          {
            "t": 9.082846,
            "value": 106822177.6823226
          },
          {
            "t": 11.107248,
            "value": 106866598.63011399
          },
          {
            "t": 13.134254,
            "value": 107091736.28494439
          },
          {
            "t": 15.162916,
            "value": 106648264.71832173
          },
          {
            "t": 17.094854,
            "value": 112167543.67893794
          },
          {
            "t": 19.120716,
            "value": 106968775.26702213
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.076059,
            "value": 110741321.83888124
          },
          {
            "t": 3.100005,
            "value": 111040869.66747136
          },
          {
            "t": 5.127541,
            "value": 98259548.53575966
          },
          {
            "t": 7.054203,
            "value": 116458387.61547172
          },
          {
            "t": 9.082846,
            "value": 111109386.91529262
          },
          {
            "t": 11.107248,
            "value": 98597213.39931497
          },
          {
            "t": 13.134254,
            "value": 110694541.11137313
          },
          {
            "t": 15.162916,
            "value": 110952860.55538082
          },
          {
            "t": 17.094854,
            "value": 104548983.97360578
          },
          {
            "t": 19.120716,
            "value": 110486470.94422029
          }
        ],
        "ram_mib": [
          {
            "t": 1.076059,
            "value": 39.4453125
          },
          {
            "t": 3.100005,
            "value": 37.44140625
          },
          {
            "t": 5.127541,
            "value": 38.59765625
          },
          {
            "t": 7.054203,
            "value": 39.640625
          },
          {
            "t": 9.082846,
            "value": 37.5859375
          },
          {
            "t": 11.107248,
            "value": 37.7109375
          },
          {
            "t": 13.134254,
            "value": 32.63671875
          },
          {
            "t": 15.162916,
            "value": 40.9765625
          },
          {
            "t": 17.094854,
            "value": 50.890625
          },
          {
            "t": 19.120716,
            "value": 35.703125
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
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.5924463272094727
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 31.433396460256276
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 33.853899999999996
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 58.904296875
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 110.33203125
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 392314.67984329094
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/400k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 402485.2275376097
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000752
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 143149670.25110218
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Zstd (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 143912755.91816574
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.077984,
            "value": 29.66825870646766
          },
          {
            "t": 3.107314,
            "value": 33.853899999999996
          },
          {
            "t": 5.14027,
            "value": 29.71514392991239
          },
          {
            "t": 7.070103,
            "value": 30.30726368159204
          },
          {
            "t": 9.10096,
            "value": 30.1121
          },
          {
            "t": 11.128897,
            "value": 33.2460907395898
          },
          {
            "t": 13.155918,
            "value": 30.915522388059703
          },
          {
            "t": 15.183164,
            "value": 31.55516811955168
          },
          {
            "t": 17.107677,
            "value": 31.272926525529265
          },
          {
            "t": 19.133304,
            "value": 33.68759051186018
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.066845,
            "value": 395190.5312348716
          },
          {
            "t": 1.077984,
            "value": 395593.4841797221
          },
          {
            "t": 2.091015,
            "value": 393867.512445325
          },
          {
            "t": 3.107314,
            "value": 393584.9587572161
          },
          {
            "t": 4.12341,
            "value": 393663.59084181016
          },
          {
            "t": 5.14027,
            "value": 393367.81857876206
          },
          {
            "t": 6.159801,
            "value": 392337.2609562632
          },
          {
            "t": 7.173749,
            "value": 394497.5481977379
          },
          {
            "t": 8.188761,
            "value": 394084.0108294286
          },
          {
            "t": 9.204014,
            "value": 393990.46346083196
          },
          {
            "t": 10.217793,
            "value": 394563.31212226726
          },
          {
            "t": 11.230743,
            "value": 394886.22340688086
          },
          {
            "t": 12.244473,
            "value": 295936.7879021041
          },
          {
            "t": 12.345962,
            "value": 89668.48663805047
          },
          {
            "t": 13.358174,
            "value": 368165.6538947497
          },
          {
            "t": 14.372733,
            "value": 394259.96910973144
          },
          {
            "t": 15.385123,
            "value": 395104.65334505483
          },
          {
            "t": 16.398296,
            "value": 394799.3087064104
          },
          {
            "t": 17.410477,
            "value": 396174.2020448912
          },
          {
            "t": 18.423413,
            "value": 393904.45200881397
          },
          {
            "t": 19.4365,
            "value": 394832.8228473961
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.066845,
            "value": 383334.81529782544
          },
          {
            "t": 1.077984,
            "value": 380758.7285229825
          },
          {
            "t": 2.091015,
            "value": 383009.0095959551
          },
          {
            "t": 3.107314,
            "value": 387681.1843758579
          },
          {
            "t": 4.12341,
            "value": 379885.3651623468
          },
          {
            "t": 5.14027,
            "value": 387467.3013000806
          },
          {
            "t": 6.159801,
            "value": 377624.61367040337
          },
          {
            "t": 7.276098,
            "value": 436263.82584563066
          },
          {
            "t": 8.188761,
            "value": 420746.7597568873
          },
          {
            "t": 9.204014,
            "value": 388080.60650891945
          },
          {
            "t": 10.217793,
            "value": 380753.5961979879
          },
          {
            "t": 11.332804,
            "value": 526452.2054042516
          },
          {
            "t": 12.345962,
            "value": 384935.0249418156
          },
          {
            "t": 13.358174,
            "value": 382330.9741437565
          },
          {
            "t": 14.372733,
            "value": 386374.76972753677
          },
          {
            "t": 15.385123,
            "value": 483015.4387143295
          },
          {
            "t": 16.398296,
            "value": 387890.3208040482
          },
          {
            "t": 17.410477,
            "value": 387282.51172468165
          },
          {
            "t": 18.423413,
            "value": 387981.0767906363
          },
          {
            "t": 19.4365,
            "value": 384962.0022762112
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.077984,
            "value": 142467623.58097553
          },
          {
            "t": 3.107314,
            "value": 142405896.03465185
          },
          {
            "t": 5.14027,
            "value": 141964892.009468
          },
          {
            "t": 7.070103,
            "value": 149738019.30011562
          },
          {
            "t": 9.10096,
            "value": 142284663.56813896
          },
          {
            "t": 11.128897,
            "value": 142517432.74076068
          },
          {
            "t": 13.155918,
            "value": 142556949.82933083
          },
          {
            "t": 15.183164,
            "value": 142548794.27558374
          },
          {
            "t": 17.107677,
            "value": 150152305.54431173
          },
          {
            "t": 19.133304,
            "value": 142490982.29832047
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.077984,
            "value": 137646579.93415737
          },
          {
            "t": 3.107314,
            "value": 148675103.60562354
          },
          {
            "t": 5.14027,
            "value": 144777490.5113539
          },
          {
            "t": 7.070103,
            "value": 144665949.33343974
          },
          {
            "t": 9.10096,
            "value": 137298674.4019889
          },
          {
            "t": 11.128897,
            "value": 155580548.6067861
          },
          {
            "t": 13.155918,
            "value": 138294234.74152464
          },
          {
            "t": 15.183164,
            "value": 139184258.8417982
          },
          {
            "t": 17.107677,
            "value": 146312568.4264019
          },
          {
            "t": 19.133304,
            "value": 139061294.1079478
          }
        ],
        "ram_mib": [
          {
            "t": 1.077984,
            "value": 49.28125
          },
          {
            "t": 3.107314,
            "value": 80.2421875
          },
          {
            "t": 5.14027,
            "value": 48.1015625
          },
          {
            "t": 7.070103,
            "value": 49.3359375
          },
          {
            "t": 9.10096,
            "value": 52.609375
          },
          {
            "t": 11.128897,
            "value": 55.47265625
          },
          {
            "t": 13.155918,
            "value": 46.83203125
          },
          {
            "t": 15.183164,
            "value": 48.27734375
          },
          {
            "t": 17.107677,
            "value": 48.55859375
          },
          {
            "t": 19.133304,
            "value": 110.33203125
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
