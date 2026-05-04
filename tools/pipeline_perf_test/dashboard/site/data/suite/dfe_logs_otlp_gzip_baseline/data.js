window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otlp_gzip_baseline"] = {
  "name": "DFE OTLP Baseline w/ Gzip (Logs)",
  "slug": "dfe_logs_otlp_gzip_baseline",
  "description": "Dataflow Engine baseline for OTLP logs with gzip compression",
  "meta": {
    "binary": "dfe",
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
      "name": "1000k",
      "metrics": [
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/1000k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -7.845304012298584
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/1000k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 96.00066928683655
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/1000k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 98.4261386138614
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/1000k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 74.409765625
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/1000k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 79.2109375
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/1000k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 971144.610890523
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/1000k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 1019243.2286116612
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/1000k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000622
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/1000k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 24940931.61359114
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/1000k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 24940245.593418885
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.045362,
            "value": 98.37457479826195
          },
          {
            "t": 2.081189,
            "value": 98.4261386138614
          },
          {
            "t": 4.111086,
            "value": 96.02247852760736
          },
          {
            "t": 6.145697,
            "value": 94.38071471835251
          },
          {
            "t": 8.080188,
            "value": 94.732397094431
          },
          {
            "t": 10.110247,
            "value": 94.642143727162
          },
          {
            "t": 12.144442,
            "value": 95.20749391727495
          },
          {
            "t": 14.114937,
            "value": 94.31195151515152
          },
          {
            "t": 16.143569,
            "value": 95.810989010989
          },
          {
            "t": 18.176381,
            "value": 98.09781094527364
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.253354,
            "value": 882256.066000596
          },
          {
            "t": 0.355332,
            "value": 267358.23329679435
          },
          {
            "t": 1.369288,
            "value": 923148.0671522641
          },
          {
            "t": 2.385733,
            "value": 983821.0626251297
          },
          {
            "t": 3.401258,
            "value": 984712.3409074125
          },
          {
            "t": 4.415749,
            "value": 985715.989594782
          },
          {
            "t": 5.435622,
            "value": 980514.2404985719
          },
          {
            "t": 6.455455,
            "value": 686386.8888337602
          },
          {
            "t": 6.557344,
            "value": 267445.9447171402
          },
          {
            "t": 7.572617,
            "value": 922074.7551899694
          },
          {
            "t": 8.587461,
            "value": 985373.1213861438
          },
          {
            "t": 9.601915,
            "value": 985751.9414384487
          },
          {
            "t": 10.616633,
            "value": 985495.4775612535
          },
          {
            "t": 11.636931,
            "value": 882095.2310011388
          },
          {
            "t": 11.738974,
            "value": 89099.48046092944
          },
          {
            "t": 12.753528,
            "value": 904585.9133374819
          },
          {
            "t": 13.708183,
            "value": 837999.0677260372
          },
          {
            "t": 13.809916,
            "value": 283986.56554220605
          },
          {
            "t": 14.824088,
            "value": 923101.3537056764
          },
          {
            "t": 15.8384,
            "value": 985889.9431338678
          },
          {
            "t": 16.85294,
            "value": 985668.3817296509
          },
          {
            "t": 17.869002,
            "value": 984191.9095488266
          },
          {
            "t": 18.891155,
            "value": 978327.1193255801
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.146627,
            "value": 985296.0496000974
          },
          {
            "t": 1.16618,
            "value": 1010246.6473052407
          },
          {
            "t": 2.182378,
            "value": 988980.4939588546
          },
          {
            "t": 3.096899,
            "value": 1088001.2596758304
          },
          {
            "t": 4.111086,
            "value": 1010661.7418681171
          },
          {
            "t": 5.125972,
            "value": 985332.3427458848
          },
          {
            "t": 6.145697,
            "value": 980656.5495599303
          },
          {
            "t": 7.166791,
            "value": 969548.3471649035
          },
          {
            "t": 8.181272,
            "value": 995582.963111187
          },
          {
            "t": 9.196398,
            "value": 1487500.073882454
          },
          {
            "t": 10.211498,
            "value": 975273.3720815685
          },
          {
            "t": 11.226528,
            "value": 999970.4442233234
          },
          {
            "t": 12.245548,
            "value": 986241.6831858059
          },
          {
            "t": 13.196559,
            "value": 1056770.1109661192
          },
          {
            "t": 14.216155,
            "value": 975876.7197988223
          },
          {
            "t": 15.230776,
            "value": 965877.8992352809
          },
          {
            "t": 16.244704,
            "value": 991194.6410396005
          },
          {
            "t": 17.26059,
            "value": 940066.1097800343
          },
          {
            "t": 18.278239,
            "value": 987570.3705305071
          },
          {
            "t": 19.298091,
            "value": 980534.4304859921
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.045362,
            "value": 25476790.460871406
          },
          {
            "t": 2.081189,
            "value": 25069508.361958068
          },
          {
            "t": 4.111086,
            "value": 24654132.697373316
          },
          {
            "t": 6.145697,
            "value": 24598526.696257908
          },
          {
            "t": 8.080188,
            "value": 25917691.527125224
          },
          {
            "t": 10.110247,
            "value": 24616905.71554817
          },
          {
            "t": 12.144442,
            "value": 24689641.356900394
          },
          {
            "t": 14.114937,
            "value": 25424240.10210632
          },
          {
            "t": 16.143569,
            "value": 24657444.031248644
          },
          {
            "t": 18.176381,
            "value": 24297574.984799385
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.045362,
            "value": 25515587.37746866
          },
          {
            "t": 2.081189,
            "value": 25029789.859354455
          },
          {
            "t": 4.111086,
            "value": 24801865.316319004
          },
          {
            "t": 6.145697,
            "value": 24570117.334468357
          },
          {
            "t": 8.080188,
            "value": 25965071.432226874
          },
          {
            "t": 10.110247,
            "value": 24560122.144233245
          },
          {
            "t": 12.144442,
            "value": 24931016.937904183
          },
          {
            "t": 14.114937,
            "value": 25428602.457758076
          },
          {
            "t": 16.143569,
            "value": 24512233.86005939
          },
          {
            "t": 18.176381,
            "value": 24094909.41611915
          }
        ],
        "ram_mib": [
          {
            "t": 0.045362,
            "value": 77.6015625
          },
          {
            "t": 2.081189,
            "value": 78.28125
          },
          {
            "t": 4.111086,
            "value": 73.40625
          },
          {
            "t": 6.145697,
            "value": 77.1015625
          },
          {
            "t": 8.080188,
            "value": 77.88671875
          },
          {
            "t": 10.110247,
            "value": 76.56640625
          },
          {
            "t": 12.144442,
            "value": 67.89453125
          },
          {
            "t": 14.114937,
            "value": 66.05078125
          },
          {
            "t": 16.143569,
            "value": 70.09765625
          },
          {
            "t": 18.176381,
            "value": 79.2109375
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
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.6315789222717285
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 13.275362177521984
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 14.53539987600744
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 10.13828125
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 10.34765625
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 99265.50838098688
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/100k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 101877.14118322305
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.001032
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 2529520.3609475074
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 2499160.591004531
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.042662,
            "value": 12.41286157666046
          },
          {
            "t": 3.055803,
            "value": 13.971481711097333
          },
          {
            "t": 5.068805,
            "value": 12.997037958929683
          },
          {
            "t": 7.083585,
            "value": 12.783462017434621
          },
          {
            "t": 9.099,
            "value": 13.365068322981365
          },
          {
            "t": 11.116071,
            "value": 12.75064039408867
          },
          {
            "t": 13.129636,
            "value": 13.626584004959701
          },
          {
            "t": 15.144867,
            "value": 12.542967101179393
          },
          {
            "t": 17.160574,
            "value": 13.76811881188119
          },
          {
            "t": 19.176779,
            "value": 14.53539987600744
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.137436,
            "value": 99241.10328319343
          },
          {
            "t": 1.143346,
            "value": 99412.47228877334
          },
          {
            "t": 2.149732,
            "value": 99365.45222210961
          },
          {
            "t": 3.156545,
            "value": 99323.31028701457
          },
          {
            "t": 4.163168,
            "value": 99342.05755282762
          },
          {
            "t": 5.170086,
            "value": 99312.95299120684
          },
          {
            "t": 6.176891,
            "value": 99324.09950288289
          },
          {
            "t": 7.184296,
            "value": 99264.94309637135
          },
          {
            "t": 8.192655,
            "value": 99171.02936553351
          },
          {
            "t": 9.199721,
            "value": 99298.35780375864
          },
          {
            "t": 10.205835,
            "value": 99392.3153837438
          },
          {
            "t": 11.217494,
            "value": 98847.53657111734
          },
          {
            "t": 12.224548,
            "value": 99299.54103752134
          },
          {
            "t": 13.230561,
            "value": 99402.29400614106
          },
          {
            "t": 14.238467,
            "value": 99215.60145489758
          },
          {
            "t": 15.245567,
            "value": 99295.00546122529
          },
          {
            "t": 16.254295,
            "value": 99134.75188554298
          },
          {
            "t": 17.261239,
            "value": 99310.38866113705
          },
          {
            "t": 18.26867,
            "value": 99262.38124496864
          },
          {
            "t": 19.278022,
            "value": 99073.46495573397
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.036077,
            "value": 99289.28728163804
          },
          {
            "t": 1.042662,
            "value": 99345.80785527301
          },
          {
            "t": 2.049053,
            "value": 99364.95854990753
          },
          {
            "t": 3.055803,
            "value": 99329.52570151477
          },
          {
            "t": 4.062324,
            "value": 99352.12479421691
          },
          {
            "t": 5.068805,
            "value": 99356.0732890139
          },
          {
            "t": 6.07621,
            "value": 99264.94309637135
          },
          {
            "t": 7.083585,
            "value": 99267.89924308228
          },
          {
            "t": 8.091892,
            "value": 99176.1437736721
          },
          {
            "t": 9.099,
            "value": 99294.21670764209
          },
          {
            "t": 10.105132,
            "value": 99390.53722573181
          },
          {
            "t": 11.116071,
            "value": 98917.93669054216
          },
          {
            "t": 12.123775,
            "value": 99235.48978668339
          },
          {
            "t": 13.129636,
            "value": 99417.31511610452
          },
          {
            "t": 14.137523,
            "value": 99217.47179991408
          },
          {
            "t": 15.144867,
            "value": 99270.95411299416
          },
          {
            "t": 16.153629,
            "value": 99131.41058049371
          },
          {
            "t": 17.160574,
            "value": 99310.29003570206
          },
          {
            "t": 18.167969,
            "value": 99265.92845904536
          },
          {
            "t": 19.176779,
            "value": 148690.04074107116
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.042662,
            "value": 2490619.706685511
          },
          {
            "t": 3.055803,
            "value": 2504121.6685766173
          },
          {
            "t": 5.068805,
            "value": 2503763.036499715
          },
          {
            "t": 7.083585,
            "value": 2501267.632198056
          },
          {
            "t": 9.099,
            "value": 2500550.506967548
          },
          {
            "t": 11.116071,
            "value": 2499107.369051461
          },
          {
            "t": 13.129636,
            "value": 2490813.557049313
          },
          {
            "t": 15.144867,
            "value": 2501336.0751199243
          },
          {
            "t": 17.160574,
            "value": 2500286.9960763147
          },
          {
            "t": 19.176779,
            "value": 2499739.361820847
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.042662,
            "value": 2520964.691124935
          },
          {
            "t": 3.055803,
            "value": 2534386.3147191377
          },
          {
            "t": 5.068805,
            "value": 2534256.796565527
          },
          {
            "t": 7.083585,
            "value": 2531679.8856450827
          },
          {
            "t": 9.099,
            "value": 2530985.9259755434
          },
          {
            "t": 11.116071,
            "value": 2529360.146469807
          },
          {
            "t": 13.129636,
            "value": 2521210.8871578523
          },
          {
            "t": 15.144867,
            "value": 2531675.525039065
          },
          {
            "t": 17.160574,
            "value": 2530619.777576801
          },
          {
            "t": 19.176779,
            "value": 2530063.6592013217
          }
        ],
        "ram_mib": [
          {
            "t": 1.042662,
            "value": 10.1796875
          },
          {
            "t": 3.055803,
            "value": 10.19921875
          },
          {
            "t": 5.068805,
            "value": 9.9765625
          },
          {
            "t": 7.083585,
            "value": 10.12109375
          },
          {
            "t": 9.099,
            "value": 10.015625
          },
          {
            "t": 11.116071,
            "value": 10.34765625
          },
          {
            "t": 13.129636,
            "value": 10.00390625
          },
          {
            "t": 15.144867,
            "value": 10.015625
          },
          {
            "t": 17.160574,
            "value": 10.2578125
          },
          {
            "t": 19.176779,
            "value": 10.265625
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
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 23.521157028735125
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 24.90417445482866
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 10.46640625
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 10.94921875
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 198590.3741677822
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/200k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 198590.3741677822
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000773
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 5047574.122230332
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 5018169.183959795
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.1003,
            "value": 22.158554517133958
          },
          {
            "t": 3.114019,
            "value": 23.136539662710806
          },
          {
            "t": 5.128199,
            "value": 23.38153271028037
          },
          {
            "t": 7.141921,
            "value": 22.164993757802748
          },
          {
            "t": 9.05573,
            "value": 24.90417445482866
          },
          {
            "t": 11.069625,
            "value": 23.54734872114785
          },
          {
            "t": 13.085425,
            "value": 23.807064676616914
          },
          {
            "t": 15.099117,
            "value": 24.455208462974486
          },
          {
            "t": 17.113877,
            "value": 24.490068535825547
          },
          {
            "t": 19.127456,
            "value": 23.166084788029924
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.093457,
            "value": 198630.4430948609
          },
          {
            "t": 1.1003,
            "value": 198640.70167841463
          },
          {
            "t": 2.107187,
            "value": 198632.02126951682
          },
          {
            "t": 3.114019,
            "value": 198642.87189918477
          },
          {
            "t": 4.120855,
            "value": 198642.08272250893
          },
          {
            "t": 5.128199,
            "value": 199534.61776711827
          },
          {
            "t": 6.135241,
            "value": 197608.44135597127
          },
          {
            "t": 7.141921,
            "value": 198672.86526006277
          },
          {
            "t": 8.14889,
            "value": 199608.9253988951
          },
          {
            "t": 9.156324,
            "value": 198524.17131047792
          },
          {
            "t": 10.163533,
            "value": 198568.51954261726
          },
          {
            "t": 11.170148,
            "value": 197692.2656626416
          },
          {
            "t": 12.179027,
            "value": 198239.82856219626
          },
          {
            "t": 13.185947,
            "value": 198625.511460692
          },
          {
            "t": 14.192834,
            "value": 198632.02126951682
          },
          {
            "t": 15.199809,
            "value": 199607.73604111324
          },
          {
            "t": 16.206526,
            "value": 197672.2355935183
          },
          {
            "t": 17.214493,
            "value": 199411.29025057366
          },
          {
            "t": 18.221323,
            "value": 198643.2664898742
          },
          {
            "t": 19.228322,
            "value": 197616.8794606549
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.093457,
            "value": 198630.4430948609
          },
          {
            "t": 1.1003,
            "value": 198640.70167841463
          },
          {
            "t": 2.107187,
            "value": 198632.02126951682
          },
          {
            "t": 3.114019,
            "value": 198642.87189918477
          },
          {
            "t": 4.120855,
            "value": 198642.08272250893
          },
          {
            "t": 5.128199,
            "value": 198541.90822598833
          },
          {
            "t": 6.135241,
            "value": 198601.4485989661
          },
          {
            "t": 7.141921,
            "value": 198672.86526006277
          },
          {
            "t": 8.14889,
            "value": 198615.84616805482
          },
          {
            "t": 9.156324,
            "value": 198524.17131047792
          },
          {
            "t": 10.163533,
            "value": 198568.51954261726
          },
          {
            "t": 11.170148,
            "value": 198685.69413330816
          },
          {
            "t": 12.179027,
            "value": 198239.82856219626
          },
          {
            "t": 13.185947,
            "value": 198625.511460692
          },
          {
            "t": 14.192834,
            "value": 198632.02126951682
          },
          {
            "t": 15.199809,
            "value": 198614.66272747586
          },
          {
            "t": 16.206526,
            "value": 198665.56341057117
          },
          {
            "t": 17.214493,
            "value": 198419.19427917778
          },
          {
            "t": 18.221323,
            "value": 198643.2664898742
          },
          {
            "t": 19.228322,
            "value": 198609.92910618582
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.1003,
            "value": 4995549.073414714
          },
          {
            "t": 3.114019,
            "value": 4996038.672724446
          },
          {
            "t": 5.128199,
            "value": 4970972.306347992
          },
          {
            "t": 7.141921,
            "value": 4997233.481086267
          },
          {
            "t": 9.05573,
            "value": 5257627.589796056
          },
          {
            "t": 11.069625,
            "value": 4996790.299394953
          },
          {
            "t": 13.085425,
            "value": 4991720.408770711
          },
          {
            "t": 15.099117,
            "value": 4996705.057178556
          },
          {
            "t": 17.113877,
            "value": 4994505.549048026
          },
          {
            "t": 19.127456,
            "value": 4984549.401836233
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.1003,
            "value": 5014503.376308139
          },
          {
            "t": 3.114019,
            "value": 5026075.634187292
          },
          {
            "t": 5.128199,
            "value": 5013343.39532713
          },
          {
            "t": 7.141921,
            "value": 5027720.807539471
          },
          {
            "t": 9.05573,
            "value": 5289566.51369076
          },
          {
            "t": 11.069625,
            "value": 5027346.510120934
          },
          {
            "t": 13.085425,
            "value": 5022188.21311638
          },
          {
            "t": 15.099117,
            "value": 5027126.78999569
          },
          {
            "t": 17.113877,
            "value": 5025054.597073597
          },
          {
            "t": 19.127456,
            "value": 5002815.384943923
          }
        ],
        "ram_mib": [
          {
            "t": 1.1003,
            "value": 9.96484375
          },
          {
            "t": 3.114019,
            "value": 10.40625
          },
          {
            "t": 5.128199,
            "value": 10.5
          },
          {
            "t": 7.141921,
            "value": 10.484375
          },
          {
            "t": 9.05573,
            "value": 10.94921875
          },
          {
            "t": 11.069625,
            "value": 10.77734375
          },
          {
            "t": 13.085425,
            "value": 10.5390625
          },
          {
            "t": 15.099117,
            "value": 10.0546875
          },
          {
            "t": 17.113877,
            "value": 10.2578125
          },
          {
            "t": 19.127456,
            "value": 10.73046875
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
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 1.7241379022598267
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 33.95101702540562
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 34.681544209215446
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 10.45
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 11.265625
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 300467.7869003712
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/300k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 298412.2270692977
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.001036
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 7536098.32115146
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 7504596.860353266
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.10629,
            "value": 34.456880049720326
          },
          {
            "t": 3.124602,
            "value": 34.334006230529596
          },
          {
            "t": 5.141998,
            "value": 34.0859738643435
          },
          {
            "t": 7.058276,
            "value": 33.50071028037383
          },
          {
            "t": 9.075742,
            "value": 34.362816199376944
          },
          {
            "t": 11.093616,
            "value": 33.40287841191067
          },
          {
            "t": 13.12413,
            "value": 33.35770287141074
          },
          {
            "t": 15.147664,
            "value": 33.480647571606475
          },
          {
            "t": 17.171665,
            "value": 34.681544209215446
          },
          {
            "t": 19.198733,
            "value": 33.847010565568674
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.097639,
            "value": 297575.9463411054
          },
          {
            "t": 1.10629,
            "value": 298418.38257236645
          },
          {
            "t": 2.114887,
            "value": 296451.4072518558
          },
          {
            "t": 3.124602,
            "value": 298103.92041318596
          },
          {
            "t": 4.133139,
            "value": 296469.04377330723
          },
          {
            "t": 5.141998,
            "value": 297365.6378146005
          },
          {
            "t": 6.150527,
            "value": 297462.9385967087
          },
          {
            "t": 7.158965,
            "value": 297489.7812260149
          },
          {
            "t": 8.167462,
            "value": 297472.3772108395
          },
          {
            "t": 9.176389,
            "value": 297345.5958657069
          },
          {
            "t": 10.185984,
            "value": 297148.8567197737
          },
          {
            "t": 11.198196,
            "value": 296380.6001114391
          },
          {
            "t": 12.212046,
            "value": 295901.7606154757
          },
          {
            "t": 13.227297,
            "value": 295493.4297035906
          },
          {
            "t": 14.240807,
            "value": 296987.69622401363
          },
          {
            "t": 15.252588,
            "value": 296506.8527675455
          },
          {
            "t": 16.264498,
            "value": 295480.82339338475
          },
          {
            "t": 17.278184,
            "value": 295949.63331840426
          },
          {
            "t": 18.291698,
            "value": 394666.47722675756
          },
          {
            "t": 19.400873,
            "value": 270471.2962336872
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.097639,
            "value": 297575.9463411054
          },
          {
            "t": 1.10629,
            "value": 297426.9593744516
          },
          {
            "t": 2.114887,
            "value": 297442.8835302901
          },
          {
            "t": 3.124602,
            "value": 297113.5419400524
          },
          {
            "t": 4.133139,
            "value": 297460.57903676317
          },
          {
            "t": 5.141998,
            "value": 297365.6378146005
          },
          {
            "t": 6.150527,
            "value": 297462.93859670864
          },
          {
            "t": 7.158965,
            "value": 297489.7812260149
          },
          {
            "t": 8.167462,
            "value": 297472.3772108395
          },
          {
            "t": 9.176389,
            "value": 297345.59586570685
          },
          {
            "t": 10.185984,
            "value": 297148.8567197738
          },
          {
            "t": 11.198196,
            "value": 296380.6001114391
          },
          {
            "t": 12.212046,
            "value": 295901.7606154757
          },
          {
            "t": 13.12413,
            "value": 328917.07342744747
          },
          {
            "t": 14.137163,
            "value": 296140.4021389234
          },
          {
            "t": 15.147664,
            "value": 296882.437523565
          },
          {
            "t": 16.159398,
            "value": 296520.62696321367
          },
          {
            "t": 17.171665,
            "value": 296364.4967187511
          },
          {
            "t": 18.185104,
            "value": 296021.763520054
          },
          {
            "t": 19.198733,
            "value": 295966.2756294463
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.10629,
            "value": 7485928.430079973
          },
          {
            "t": 3.124602,
            "value": 7468055.483988601
          },
          {
            "t": 5.141998,
            "value": 7482767.884936819
          },
          {
            "t": 7.058276,
            "value": 7865474.1117938
          },
          {
            "t": 9.075742,
            "value": 7471094.432322526
          },
          {
            "t": 11.093616,
            "value": 7481871.018705826
          },
          {
            "t": 13.12413,
            "value": 7435843.830675385
          },
          {
            "t": 15.147664,
            "value": 7460242.822705227
          },
          {
            "t": 17.171665,
            "value": 7459296.215762739
          },
          {
            "t": 19.198733,
            "value": 7435394.37256175
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.10629,
            "value": 7522782.411913544
          },
          {
            "t": 3.124602,
            "value": 7502665.098359422
          },
          {
            "t": 5.141998,
            "value": 7501570.341172481
          },
          {
            "t": 7.058276,
            "value": 7896615.731120433
          },
          {
            "t": 9.075742,
            "value": 7513737.0344779035
          },
          {
            "t": 11.093616,
            "value": 7500212.599993855
          },
          {
            "t": 13.12413,
            "value": 7477812.0219806405
          },
          {
            "t": 15.147664,
            "value": 7478285.020167686
          },
          {
            "t": 17.171665,
            "value": 7489684.04659879
          },
          {
            "t": 19.198733,
            "value": 7477618.905729853
          }
        ],
        "ram_mib": [
          {
            "t": 1.10629,
            "value": 10.51953125
          },
          {
            "t": 3.124602,
            "value": 10.2265625
          },
          {
            "t": 5.141998,
            "value": 10.15625
          },
          {
            "t": 7.058276,
            "value": 10.41796875
          },
          {
            "t": 9.075742,
            "value": 10.48046875
          },
          {
            "t": 11.093616,
            "value": 10.40234375
          },
          {
            "t": 13.12413,
            "value": 11.265625
          },
          {
            "t": 15.147664,
            "value": 10.65234375
          },
          {
            "t": 17.171665,
            "value": 10.3203125
          },
          {
            "t": 19.198733,
            "value": 10.05859375
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
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.6315789222717285
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 42.654016286460966
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 43.939435582822085
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 11.99921875
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 12.859375
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 394154.6042392158
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/400k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 404527.0938244583
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000638
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 10063732.655452592
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9997336.462517522
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.118343,
            "value": 42.15687041564792
          },
          {
            "t": 2.131479,
            "value": 42.387872860635696
          },
          {
            "t": 4.144808,
            "value": 42.52180708180708
          },
          {
            "t": 6.158534,
            "value": 43.10053789731052
          },
          {
            "t": 8.113907,
            "value": 42.08137003058104
          },
          {
            "t": 10.12805,
            "value": 42.48365185636032
          },
          {
            "t": 12.141334,
            "value": 42.35275820170109
          },
          {
            "t": 14.156306,
            "value": 42.78423500611995
          },
          {
            "t": 16.175473,
            "value": 43.939435582822085
          },
          {
            "t": 18.191674,
            "value": 42.73162393162393
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.218855,
            "value": 397474.44736146525
          },
          {
            "t": 1.225457,
            "value": 397376.52021355013
          },
          {
            "t": 2.23208,
            "value": 397368.2302113105
          },
          {
            "t": 3.238505,
            "value": 397446.4068360782
          },
          {
            "t": 4.245541,
            "value": 397205.2637641554
          },
          {
            "t": 5.252056,
            "value": 397410.8681937179
          },
          {
            "t": 6.259088,
            "value": 397206.84149063786
          },
          {
            "t": 7.26699,
            "value": 396863.98082353245
          },
          {
            "t": 8.315447,
            "value": 381513.0234239459
          },
          {
            "t": 9.32236,
            "value": 397253.78458714904
          },
          {
            "t": 10.329048,
            "value": 397342.5728726279
          },
          {
            "t": 11.335858,
            "value": 397294.4249659817
          },
          {
            "t": 12.344234,
            "value": 396677.42984759656
          },
          {
            "t": 13.350728,
            "value": 397419.15997512155
          },
          {
            "t": 14.362607,
            "value": 395304.1816264593
          },
          {
            "t": 15.469866,
            "value": 361252.4260358236
          },
          {
            "t": 16.477493,
            "value": 396972.2923264263
          },
          {
            "t": 17.484205,
            "value": 397333.10023124784
          },
          {
            "t": 18.493446,
            "value": 396337.4456646133
          },
          {
            "t": 19.500629,
            "value": 397147.29100868467
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.218855,
            "value": 397474.44736146525
          },
          {
            "t": 1.225457,
            "value": 397376.52021355013
          },
          {
            "t": 2.23208,
            "value": 397368.2302113105
          },
          {
            "t": 3.238505,
            "value": 397446.4068360782
          },
          {
            "t": 4.245541,
            "value": 397205.2637641554
          },
          {
            "t": 5.252056,
            "value": 596116.3022905769
          },
          {
            "t": 6.259088,
            "value": 397206.84149063786
          },
          {
            "t": 7.26699,
            "value": 396863.98082353245
          },
          {
            "t": 8.315447,
            "value": 381513.0234239459
          },
          {
            "t": 9.32236,
            "value": 397253.78458714904
          },
          {
            "t": 10.329048,
            "value": 397342.5728726279
          },
          {
            "t": 11.335858,
            "value": 397294.4249659817
          },
          {
            "t": 12.344234,
            "value": 396677.42984759656
          },
          {
            "t": 13.451971,
            "value": 361096.5418686927
          },
          {
            "t": 14.463367,
            "value": 395492.96220273763
          },
          {
            "t": 15.469866,
            "value": 397417.18571007025
          },
          {
            "t": 16.477493,
            "value": 396972.2923264263
          },
          {
            "t": 17.484205,
            "value": 397333.10023124784
          },
          {
            "t": 18.493446,
            "value": 396337.4456646133
          },
          {
            "t": 19.500629,
            "value": 397147.29100868467
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.118343,
            "value": 9957453.381443964
          },
          {
            "t": 2.131479,
            "value": 9969491.38061214
          },
          {
            "t": 4.144808,
            "value": 9982935.228171848
          },
          {
            "t": 6.158534,
            "value": 9967747.84652927
          },
          {
            "t": 8.113907,
            "value": 10278473.723427704
          },
          {
            "t": 10.12805,
            "value": 9964848.076824734
          },
          {
            "t": 12.141334,
            "value": 9994641.093854617
          },
          {
            "t": 14.156306,
            "value": 9936239.312506575
          },
          {
            "t": 16.175473,
            "value": 9964954.359891975
          },
          {
            "t": 18.191674,
            "value": 9956580.221912397
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.118343,
            "value": 10038438.175934916
          },
          {
            "t": 2.131479,
            "value": 10049854.058543487
          },
          {
            "t": 4.144808,
            "value": 10045573.276896125
          },
          {
            "t": 6.158534,
            "value": 9993702.718244687
          },
          {
            "t": 8.113907,
            "value": 10343191.81046276
          },
          {
            "t": 10.12805,
            "value": 10040271.222053252
          },
          {
            "t": 12.141334,
            "value": 10058060.363068499
          },
          {
            "t": 14.156306,
            "value": 10023655.415559124
          },
          {
            "t": 16.175473,
            "value": 10014409.902697498
          },
          {
            "t": 18.191674,
            "value": 10030169.611065563
          }
        ],
        "ram_mib": [
          {
            "t": 0.118343,
            "value": 12.17578125
          },
          {
            "t": 2.131479,
            "value": 12.015625
          },
          {
            "t": 4.144808,
            "value": 12.11328125
          },
          {
            "t": 6.158534,
            "value": 12.33203125
          },
          {
            "t": 8.113907,
            "value": 12.859375
          },
          {
            "t": 10.12805,
            "value": 11.94140625
          },
          {
            "t": 12.141334,
            "value": 12.02734375
          },
          {
            "t": 14.156306,
            "value": 12.06640625
          },
          {
            "t": 16.175473,
            "value": 12.03515625
          },
          {
            "t": 18.191674,
            "value": 10.42578125
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
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/600k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -0.8620689511299133
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/600k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 63.3520848657789
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/600k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 65.46822372464659
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/600k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 12.661328125
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/600k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 15.40625
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/600k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 594525.1714526028
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/600k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 612287.7677149767
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/600k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.00063
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/600k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 15032979.981401032
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/600k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 14966608.69510597
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.11181,
            "value": 65.46822372464659
          },
          {
            "t": 2.13222,
            "value": 63.92176904176904
          },
          {
            "t": 4.094092,
            "value": 62.40801484230055
          },
          {
            "t": 6.112621,
            "value": 64.23684340320592
          },
          {
            "t": 8.127141,
            "value": 64.15643564356436
          },
          {
            "t": 10.143177,
            "value": 63.91058533579791
          },
          {
            "t": 12.164665,
            "value": 61.179175384615384
          },
          {
            "t": 14.179349,
            "value": 61.38548802946593
          },
          {
            "t": 16.193942,
            "value": 63.14995681677976
          },
          {
            "t": 18.211672,
            "value": 63.70435643564356
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.217569,
            "value": 591422.6924641493
          },
          {
            "t": 1.326474,
            "value": 541074.3030286634
          },
          {
            "t": 2.333736,
            "value": 595674.2138589562
          },
          {
            "t": 3.389136,
            "value": 568504.8322910746
          },
          {
            "t": 4.398782,
            "value": 594267.6938253605
          },
          {
            "t": 5.407655,
            "value": 594723.0226202902
          },
          {
            "t": 6.414544,
            "value": 695210.6935322563
          },
          {
            "t": 7.422004,
            "value": 595557.1437079387
          },
          {
            "t": 8.429397,
            "value": 595596.7532035661
          },
          {
            "t": 9.437564,
            "value": 596131.3948978691
          },
          {
            "t": 10.450757,
            "value": 394791.5155355396
          },
          {
            "t": 10.551708,
            "value": 179510.0094781285
          },
          {
            "t": 11.560348,
            "value": 557788.5587545481
          },
          {
            "t": 12.567539,
            "value": 596709.0651127741
          },
          {
            "t": 13.574848,
            "value": 594653.6762800689
          },
          {
            "t": 14.581959,
            "value": 595763.525569674
          },
          {
            "t": 15.589635,
            "value": 595429.4832862944
          },
          {
            "t": 16.596693,
            "value": 695094.0263619374
          },
          {
            "t": 17.607403,
            "value": 593642.0931820206
          },
          {
            "t": 18.619573,
            "value": 592785.7968523074
          },
          {
            "t": 19.728938,
            "value": 540849.9456896513
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.217569,
            "value": 592410.042535041
          },
          {
            "t": 1.22527,
            "value": 595414.7113082155
          },
          {
            "t": 2.23276,
            "value": 595539.4098204449
          },
          {
            "t": 3.241168,
            "value": 594997.2630125901
          },
          {
            "t": 4.197247,
            "value": 626517.2647866965
          },
          {
            "t": 5.206035,
            "value": 595764.4222572037
          },
          {
            "t": 6.213163,
            "value": 893630.203906554
          },
          {
            "t": 7.22018,
            "value": 595819.137114865
          },
          {
            "t": 8.227674,
            "value": 595537.0453819081
          },
          {
            "t": 9.235609,
            "value": 595276.4811222946
          },
          {
            "t": 10.243753,
            "value": 595153.0733704709
          },
          {
            "t": 11.258156,
            "value": 591480.9005888193
          },
          {
            "t": 12.265223,
            "value": 595789.5552133076
          },
          {
            "t": 13.272462,
            "value": 595687.8159006949
          },
          {
            "t": 14.279901,
            "value": 595569.5580576095
          },
          {
            "t": 15.287241,
            "value": 595628.089820716
          },
          {
            "t": 16.2945,
            "value": 595675.9880030856
          },
          {
            "t": 17.302,
            "value": 595533.4987593052
          },
          {
            "t": 18.312232,
            "value": 593922.9800679447
          },
          {
            "t": 19.326231,
            "value": 591716.5598782641
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.11181,
            "value": 14905421.57659941
          },
          {
            "t": 2.13222,
            "value": 14912228.211105667
          },
          {
            "t": 4.094092,
            "value": 15345547.008163631
          },
          {
            "t": 6.112621,
            "value": 14926073.393050088
          },
          {
            "t": 8.127141,
            "value": 14956406.488890653
          },
          {
            "t": 10.143177,
            "value": 14908851.82605866
          },
          {
            "t": 12.164665,
            "value": 14903609.618261399
          },
          {
            "t": 14.179349,
            "value": 14930991.16288212
          },
          {
            "t": 16.193942,
            "value": 14955942.465798302
          },
          {
            "t": 18.211672,
            "value": 14921015.200249787
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.11181,
            "value": 14998508.004794601
          },
          {
            "t": 2.13222,
            "value": 14940687.28624388
          },
          {
            "t": 4.094092,
            "value": 15436455.589355472
          },
          {
            "t": 6.112621,
            "value": 14978151.911614845
          },
          {
            "t": 8.127141,
            "value": 15033837.837301193
          },
          {
            "t": 10.143177,
            "value": 14984955.625792395
          },
          {
            "t": 12.164665,
            "value": 14944239.589846686
          },
          {
            "t": 14.179349,
            "value": 15019867.135491224
          },
          {
            "t": 16.193942,
            "value": 15020489.001996929
          },
          {
            "t": 18.211672,
            "value": 14972607.831573106
          }
        ],
        "ram_mib": [
          {
            "t": 0.11181,
            "value": 15.40625
          },
          {
            "t": 2.13222,
            "value": 15.38671875
          },
          {
            "t": 4.094092,
            "value": 12.3359375
          },
          {
            "t": 6.112621,
            "value": 12.296875
          },
          {
            "t": 8.127141,
            "value": 11.7890625
          },
          {
            "t": 10.143177,
            "value": 12.0703125
          },
          {
            "t": 12.164665,
            "value": 11.77734375
          },
          {
            "t": 14.179349,
            "value": 11.91796875
          },
          {
            "t": 16.193942,
            "value": 11.87109375
          },
          {
            "t": 18.211672,
            "value": 11.76171875
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
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/800k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 1.2658227682113647
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/800k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 78.49758723811306
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/800k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 82.84196078431373
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/800k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 13.614453125
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/800k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 18.14453125
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/800k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 808683.9967988398
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/800k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 815240.7374542289
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/800k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000751
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/800k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 20057456.59719386
        },
        {
          "extra": "DFE OTLP Baseline w/ Gzip (Logs)/800k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 20016243.225099668
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.036279,
            "value": 75.80866789441374
          },
          {
            "t": 2.053073,
            "value": 75.68967623701893
          },
          {
            "t": 4.078969,
            "value": 78.13120784794604
          },
          {
            "t": 6.096093,
            "value": 80.28452088452089
          },
          {
            "t": 8.114479,
            "value": 81.27262446220037
          },
          {
            "t": 10.13933,
            "value": 81.57256299938537
          },
          {
            "t": 12.101,
            "value": 82.84196078431373
          },
          {
            "t": 14.11959,
            "value": 76.60835272504592
          },
          {
            "t": 16.142879,
            "value": 76.13613259668509
          },
          {
            "t": 18.161424,
            "value": 76.6301659496005
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.237966,
            "value": 793530.3470802547
          },
          {
            "t": 1.246427,
            "value": 892448.9891031978
          },
          {
            "t": 2.258921,
            "value": 888894.1564098158
          },
          {
            "t": 3.27251,
            "value": 690615.2296443627
          },
          {
            "t": 3.373672,
            "value": 89706.13168321893
          },
          {
            "t": 4.381561,
            "value": 730387.5276838234
          },
          {
            "t": 5.389761,
            "value": 793493.3544931561
          },
          {
            "t": 6.398804,
            "value": 792830.4343818846
          },
          {
            "t": 7.408235,
            "value": 891591.401492524
          },
          {
            "t": 8.423683,
            "value": 689350.907185794
          },
          {
            "t": 8.52506,
            "value": 179079.08580126698
          },
          {
            "t": 9.533782,
            "value": 729709.7317630365
          },
          {
            "t": 10.542529,
            "value": 794054.406109758
          },
          {
            "t": 11.595493,
            "value": 853780.3761572097
          },
          {
            "t": 12.605647,
            "value": 791958.4538595106
          },
          {
            "t": 13.614572,
            "value": 792923.1607899497
          },
          {
            "t": 14.623529,
            "value": 792898.0125020193
          },
          {
            "t": 15.637437,
            "value": 789026.2232865309
          },
          {
            "t": 15.738674,
            "value": 89674.43695662895
          },
          {
            "t": 16.747826,
            "value": 730403.3885057971
          },
          {
            "t": 17.756162,
            "value": 792394.5986258548
          },
          {
            "t": 18.767681,
            "value": 790889.7410725849
          },
          {
            "t": 19.775882,
            "value": 793492.5674543073
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.136824,
            "value": 793493.3544931561
          },
          {
            "t": 1.145253,
            "value": 793313.1633461552
          },
          {
            "t": 2.15365,
            "value": 793338.3379760154
          },
          {
            "t": 3.166201,
            "value": 790083.6599835465
          },
          {
            "t": 4.179714,
            "value": 788347.0660958467
          },
          {
            "t": 5.187953,
            "value": 792470.8328084907
          },
          {
            "t": 6.196969,
            "value": 1191259.603415605
          },
          {
            "t": 7.206312,
            "value": 792594.7869059377
          },
          {
            "t": 8.215112,
            "value": 793021.4115781126
          },
          {
            "t": 9.231041,
            "value": 786472.2830040287
          },
          {
            "t": 10.240083,
            "value": 793822.2591329202
          },
          {
            "t": 11.248843,
            "value": 793052.8569729172
          },
          {
            "t": 12.302957,
            "value": 756085.2052055091
          },
          {
            "t": 13.311535,
            "value": 796170.4498809215
          },
          {
            "t": 14.321039,
            "value": 792468.3805116176
          },
          {
            "t": 15.329817,
            "value": 793038.7062366547
          },
          {
            "t": 16.244093,
            "value": 875009.2969737804
          },
          {
            "t": 17.25228,
            "value": 793503.5861402696
          },
          {
            "t": 18.263911,
            "value": 790802.1798462088
          },
          {
            "t": 19.272276,
            "value": 793363.5142036862
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.036279,
            "value": 20911375.75458624
          },
          {
            "t": 2.053073,
            "value": 19889349.631147258
          },
          {
            "t": 4.078969,
            "value": 19812192.728550725
          },
          {
            "t": 6.096093,
            "value": 19862104.164146576
          },
          {
            "t": 8.114479,
            "value": 19884561.72407062
          },
          {
            "t": 10.13933,
            "value": 19820637.66667276
          },
          {
            "t": 12.101,
            "value": 20455694.382847268
          },
          {
            "t": 14.11959,
            "value": 19853435.318712566
          },
          {
            "t": 16.142879,
            "value": 19800580.63875205
          },
          {
            "t": 18.161424,
            "value": 19872500.241510592
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.036279,
            "value": 20954513.163011506
          },
          {
            "t": 2.053073,
            "value": 19956994.616207704
          },
          {
            "t": 4.078969,
            "value": 19843563.04568448
          },
          {
            "t": 6.096093,
            "value": 19926862.20579399
          },
          {
            "t": 8.114479,
            "value": 19923572.597114723
          },
          {
            "t": 10.13933,
            "value": 19859557.073582206
          },
          {
            "t": 12.101,
            "value": 20445276.728501737
          },
          {
            "t": 14.11959,
            "value": 19932041.67265269
          },
          {
            "t": 16.142879,
            "value": 19825441.15052274
          },
          {
            "t": 18.161424,
            "value": 19906743.718866806
          }
        ],
        "ram_mib": [
          {
            "t": 0.036279,
            "value": 12.2890625
          },
          {
            "t": 2.053073,
            "value": 12.5390625
          },
          {
            "t": 4.078969,
            "value": 12.46875
          },
          {
            "t": 6.096093,
            "value": 12.8984375
          },
          {
            "t": 8.114479,
            "value": 12.31640625
          },
          {
            "t": 10.13933,
            "value": 12.5546875
          },
          {
            "t": 12.101,
            "value": 15.03125
          },
          {
            "t": 14.11959,
            "value": 18.14453125
          },
          {
            "t": 16.142879,
            "value": 15.30859375
          },
          {
            "t": 18.161424,
            "value": 12.59375
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
