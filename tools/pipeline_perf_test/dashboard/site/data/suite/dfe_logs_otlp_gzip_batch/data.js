window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otlp_gzip_batch"] = {
  "name": "DFE OTLP Batch Processor w/ Gzip (Logs)",
  "slug": "dfe_logs_otlp_gzip_batch",
  "description": "Dataflow Engine OTLP logs through a batch processor with gzip compression",
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
      "name": "100k",
      "metrics": [
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.052631575614213943
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 9.894174495024059
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 11.1263523573201
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 18.219140625
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 29.6484375
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 99081.32318622382
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/100k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 99029.17512138897
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.001121
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 36325104.99937021
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 35979448.99758989
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.055051,
            "value": 9.808406695598263
          },
          {
            "t": 3.071673,
            "value": 9.502332506203475
          },
          {
            "t": 5.086882,
            "value": 10.03910891089109
          },
          {
            "t": 7.106413,
            "value": 9.568693498452012
          },
          {
            "t": 9.123917,
            "value": 10.41225404732254
          },
          {
            "t": 11.141365,
            "value": 9.0564166150031
          },
          {
            "t": 13.159554,
            "value": 9.730818858560793
          },
          {
            "t": 15.181628,
            "value": 9.675136476426799
          },
          {
            "t": 17.203174,
            "value": 10.0222249844624
          },
          {
            "t": 19.120151,
            "value": 11.1263523573201
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.047646,
            "value": 99245.23995017888
          },
          {
            "t": 1.055051,
            "value": 99264.94309637135
          },
          {
            "t": 2.062771,
            "value": 99233.91418251101
          },
          {
            "t": 3.071673,
            "value": 99117.65463840889
          },
          {
            "t": 4.079768,
            "value": 99197.00028271145
          },
          {
            "t": 5.086882,
            "value": 99293.62515067807
          },
          {
            "t": 6.095861,
            "value": 99110.09049742363
          },
          {
            "t": 7.106413,
            "value": 98955.81820628727
          },
          {
            "t": 8.116675,
            "value": 98984.2238943957
          },
          {
            "t": 9.123917,
            "value": 99281.00694768487
          },
          {
            "t": 10.132397,
            "value": 99159.13057274313
          },
          {
            "t": 11.141365,
            "value": 99111.17101830781
          },
          {
            "t": 12.149646,
            "value": 99178.70117556515
          },
          {
            "t": 13.159554,
            "value": 99018.92053533588
          },
          {
            "t": 14.168982,
            "value": 99066.00569827665
          },
          {
            "t": 15.181628,
            "value": 98751.19242064848
          },
          {
            "t": 16.190998,
            "value": 99071.69818797863
          },
          {
            "t": 17.203174,
            "value": 98797.04715385467
          },
          {
            "t": 18.212392,
            "value": 99086.61954107042
          },
          {
            "t": 19.223813,
            "value": 98870.79663166971
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.047646,
            "value": 104207.50194768782
          },
          {
            "t": 1.055051,
            "value": 83382.55220095195
          },
          {
            "t": 2.062771,
            "value": 105187.94903346169
          },
          {
            "t": 3.071673,
            "value": 104073.53737032933
          },
          {
            "t": 4.079768,
            "value": 105148.82029967415
          },
          {
            "t": 5.086882,
            "value": 83406.64512656958
          },
          {
            "t": 6.095861,
            "value": 105056.69592726904
          },
          {
            "t": 7.106413,
            "value": 103903.60911660163
          },
          {
            "t": 8.116675,
            "value": 104923.27732805946
          },
          {
            "t": 9.123917,
            "value": 83396.04583605529
          },
          {
            "t": 10.132397,
            "value": 105108.67840710773
          },
          {
            "t": 11.141365,
            "value": 104066.7295692232
          },
          {
            "t": 12.149646,
            "value": 84301.89599923037
          },
          {
            "t": 13.159554,
            "value": 104960.05576745604
          },
          {
            "t": 14.168982,
            "value": 104019.30598319048
          },
          {
            "t": 15.181628,
            "value": 103688.75204168091
          },
          {
            "t": 16.190998,
            "value": 84210.94345978183
          },
          {
            "t": 17.203174,
            "value": 104724.86998308596
          },
          {
            "t": 18.212392,
            "value": 104040.95051812394
          },
          {
            "t": 19.223813,
            "value": 103814.33646325319
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.055051,
            "value": 36027033.61273641
          },
          {
            "t": 3.071673,
            "value": 35811967.24026615
          },
          {
            "t": 5.086882,
            "value": 35663627.94132023
          },
          {
            "t": 7.106413,
            "value": 35943980.06269772
          },
          {
            "t": 9.123917,
            "value": 35803984.031754084
          },
          {
            "t": 11.141365,
            "value": 35800770.577482045
          },
          {
            "t": 13.159554,
            "value": 35613718.53676737
          },
          {
            "t": 15.181628,
            "value": 35881944.47878762
          },
          {
            "t": 17.203174,
            "value": 35566106.33643756
          },
          {
            "t": 19.120151,
            "value": 37681357.15764978
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.055051,
            "value": 37432833.583952434
          },
          {
            "t": 3.071673,
            "value": 33854888.02561908
          },
          {
            "t": 5.086882,
            "value": 37679746.36873893
          },
          {
            "t": 7.106413,
            "value": 34085658.997064166
          },
          {
            "t": 9.123917,
            "value": 37764199.724015415
          },
          {
            "t": 11.141365,
            "value": 33790654.33161103
          },
          {
            "t": 13.159554,
            "value": 37890592.50645009
          },
          {
            "t": 15.181628,
            "value": 33903082.67649948
          },
          {
            "t": 17.203174,
            "value": 37492571.032269366
          },
          {
            "t": 19.120151,
            "value": 39356822.747482106
          }
        ],
        "ram_mib": [
          {
            "t": 1.055051,
            "value": 16.72265625
          },
          {
            "t": 3.071673,
            "value": 16.828125
          },
          {
            "t": 5.086882,
            "value": 17.1328125
          },
          {
            "t": 7.106413,
            "value": 16.71484375
          },
          {
            "t": 9.123917,
            "value": 16.78125
          },
          {
            "t": 11.141365,
            "value": 17.25390625
          },
          {
            "t": 13.159554,
            "value": 17.11328125
          },
          {
            "t": 15.181628,
            "value": 29.6484375
          },
          {
            "t": 17.203174,
            "value": 17.2734375
          },
          {
            "t": 19.120151,
            "value": 16.72265625
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
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.10526315122842789
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 17.710168629606184
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 19.955112219451372
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 27.131640625
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 36.734375
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 197915.9347900328
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/200k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 197707.60222709592
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.001003
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 72011190.81142405
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 71857518.51027185
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.052757,
            "value": 16.691670822942644
          },
          {
            "t": 3.073306,
            "value": 15.224906600249067
          },
          {
            "t": 5.091761,
            "value": 16.044505289359055
          },
          {
            "t": 7.112742,
            "value": 16.029507788161993
          },
          {
            "t": 9.132527,
            "value": 17.754925373134327
          },
          {
            "t": 11.158049,
            "value": 18.796499999999998
          },
          {
            "t": 13.177392,
            "value": 19.459190031152648
          },
          {
            "t": 15.196905,
            "value": 18.6673631840796
          },
          {
            "t": 17.216736,
            "value": 19.955112219451372
          },
          {
            "t": 19.1376,
            "value": 18.478004987531172
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.040389,
            "value": 198302.13710213156
          },
          {
            "t": 1.052757,
            "value": 197556.61972721384
          },
          {
            "t": 2.063614,
            "value": 197851.92168625238
          },
          {
            "t": 3.073306,
            "value": 198080.20663727156
          },
          {
            "t": 4.081691,
            "value": 198336.94471853506
          },
          {
            "t": 5.091761,
            "value": 198996.10918055184
          },
          {
            "t": 6.102597,
            "value": 196866.75187666446
          },
          {
            "t": 7.112742,
            "value": 197991.37747551093
          },
          {
            "t": 8.122691,
            "value": 199019.95051235263
          },
          {
            "t": 9.132527,
            "value": 197061.70110790269
          },
          {
            "t": 10.146681,
            "value": 197208.70794770814
          },
          {
            "t": 11.158049,
            "value": 197751.95576684253
          },
          {
            "t": 12.168363,
            "value": 197958.25852160814
          },
          {
            "t": 13.177392,
            "value": 199201.4104649123
          },
          {
            "t": 14.187035,
            "value": 197099.37076768716
          },
          {
            "t": 15.196905,
            "value": 198045.2929584996
          },
          {
            "t": 16.20767,
            "value": 197869.93020138214
          },
          {
            "t": 17.216736,
            "value": 198203.0907789976
          },
          {
            "t": 18.229419,
            "value": 198482.64461830602
          },
          {
            "t": 19.24046,
            "value": 196826.83491569577
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.040389,
            "value": 175497.39133538643
          },
          {
            "t": 1.052757,
            "value": 219287.84789720736
          },
          {
            "t": 2.063614,
            "value": 175098.95069233337
          },
          {
            "t": 3.073306,
            "value": 217888.22730099873
          },
          {
            "t": 4.081691,
            "value": 174536.51135231086
          },
          {
            "t": 5.091761,
            "value": 218796.71705921373
          },
          {
            "t": 6.102597,
            "value": 174113.30819242686
          },
          {
            "t": 7.112742,
            "value": 218780.4721104396
          },
          {
            "t": 8.122691,
            "value": 175256.3743317732
          },
          {
            "t": 9.132527,
            "value": 219837.67661283616
          },
          {
            "t": 10.146681,
            "value": 172557.61945424462
          },
          {
            "t": 11.158049,
            "value": 219504.6709011952
          },
          {
            "t": 12.168363,
            "value": 215774.50178855288
          },
          {
            "t": 13.177392,
            "value": 174425.115630968
          },
          {
            "t": 14.187035,
            "value": 220870.14915172986
          },
          {
            "t": 15.196905,
            "value": 174279.85780347965
          },
          {
            "t": 16.20767,
            "value": 217656.92322152035
          },
          {
            "t": 17.216736,
            "value": 174418.7198855179
          },
          {
            "t": 18.229419,
            "value": 218232.1614957494
          },
          {
            "t": 19.24046,
            "value": 175067.08432200077
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.052757,
            "value": 71492108.58367188
          },
          {
            "t": 3.073306,
            "value": 71510792.85877255
          },
          {
            "t": 5.091761,
            "value": 71404612.93415013
          },
          {
            "t": 7.112742,
            "value": 71624312.15335523
          },
          {
            "t": 9.132527,
            "value": 71383280.3986563
          },
          {
            "t": 11.158049,
            "value": 71328091.7215414
          },
          {
            "t": 13.177392,
            "value": 71544382.99981727
          },
          {
            "t": 15.196905,
            "value": 71719805.71553637
          },
          {
            "t": 17.216736,
            "value": 71348315.77493365
          },
          {
            "t": 19.1376,
            "value": 75219481.96228364
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.052757,
            "value": 71671941.13601165
          },
          {
            "t": 3.073306,
            "value": 71038762.23739192
          },
          {
            "t": 5.091761,
            "value": 70924253.94670677
          },
          {
            "t": 7.112742,
            "value": 70604506.92015412
          },
          {
            "t": 9.132527,
            "value": 74826857.80912325
          },
          {
            "t": 11.158049,
            "value": 74367467.74411732
          },
          {
            "t": 13.177392,
            "value": 70812654.90805672
          },
          {
            "t": 15.196905,
            "value": 70641762.14760688
          },
          {
            "t": 17.216736,
            "value": 70448152.345419
          },
          {
            "t": 19.1376,
            "value": 74775548.91965282
          }
        ],
        "ram_mib": [
          {
            "t": 1.052757,
            "value": 27.97265625
          },
          {
            "t": 3.073306,
            "value": 26.4921875
          },
          {
            "t": 5.091761,
            "value": 26.41796875
          },
          {
            "t": 7.112742,
            "value": 28.578125
          },
          {
            "t": 9.132527,
            "value": 36.734375
          },
          {
            "t": 11.158049,
            "value": 26.7578125
          },
          {
            "t": 13.177392,
            "value": 24.15234375
          },
          {
            "t": 15.196905,
            "value": 27.25
          },
          {
            "t": 17.216736,
            "value": 21.13671875
          },
          {
            "t": 19.1376,
            "value": 25.82421875
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
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 1.2542372941970825
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 25.215908604008654
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 27.487738693467335
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 43.211328125
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 73.421875
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 306968.7419453223
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/300k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 303118.62552092335
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000769
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 108067589.50680724
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 107664446.0056344
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.095727,
            "value": 24.33652065081352
          },
          {
            "t": 3.116202,
            "value": 23.554708463949844
          },
          {
            "t": 5.137137,
            "value": 26.048020050125313
          },
          {
            "t": 7.057175,
            "value": 25.232570356472795
          },
          {
            "t": 9.079565,
            "value": 24.039271814187067
          },
          {
            "t": 11.10158,
            "value": 26.8724670433145
          },
          {
            "t": 13.122336,
            "value": 26.139749373433585
          },
          {
            "t": 15.143923,
            "value": 23.76552070263488
          },
          {
            "t": 17.177508,
            "value": 24.682518891687657
          },
          {
            "t": 19.20242,
            "value": 27.487738693467335
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.085344,
            "value": 297142.3817150464
          },
          {
            "t": 1.095727,
            "value": 296917.10965049884
          },
          {
            "t": 2.105984,
            "value": 296954.14137194795
          },
          {
            "t": 3.116202,
            "value": 296965.6054435775
          },
          {
            "t": 4.126972,
            "value": 296803.4270902382
          },
          {
            "t": 5.137137,
            "value": 296981.1862418516
          },
          {
            "t": 6.147988,
            "value": 296779.64408206556
          },
          {
            "t": 7.157876,
            "value": 297062.6445704871
          },
          {
            "t": 8.169637,
            "value": 297501.0896842239
          },
          {
            "t": 9.180444,
            "value": 295803.25423152
          },
          {
            "t": 10.192153,
            "value": 395370.6055792723
          },
          {
            "t": 11.202394,
            "value": 296958.8444737444
          },
          {
            "t": 12.21338,
            "value": 296740.01420395536
          },
          {
            "t": 13.223503,
            "value": 296993.534450755
          },
          {
            "t": 14.234467,
            "value": 296746.4716844517
          },
          {
            "t": 15.249798,
            "value": 295470.1471736803
          },
          {
            "t": 16.268507,
            "value": 294490.37949011935
          },
          {
            "t": 17.281463,
            "value": 394883.884393794
          },
          {
            "t": 18.293909,
            "value": 296312.0996082754
          },
          {
            "t": 19.305542,
            "value": 296550.2311609052
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.085344,
            "value": 344685.1627894538
          },
          {
            "t": 1.095727,
            "value": 273163.74087845895
          },
          {
            "t": 2.105984,
            "value": 279136.89288963104
          },
          {
            "t": 3.116202,
            "value": 341510.44626011414
          },
          {
            "t": 4.126972,
            "value": 276027.1871939215
          },
          {
            "t": 5.137137,
            "value": 273222.6913425035
          },
          {
            "t": 6.147988,
            "value": 276005.06899632094
          },
          {
            "t": 7.157876,
            "value": 342612.2500712951
          },
          {
            "t": 8.169637,
            "value": 275756.82399301813
          },
          {
            "t": 9.180444,
            "value": 275027.77483733295
          },
          {
            "t": 10.192153,
            "value": 346937.70639581146
          },
          {
            "t": 11.202394,
            "value": 270232.5484711074
          },
          {
            "t": 12.21338,
            "value": 273000.81306763896
          },
          {
            "t": 13.223503,
            "value": 481129.5258102231
          },
          {
            "t": 14.336289,
            "value": 250722.06156439782
          },
          {
            "t": 15.351653,
            "value": 336825.0203867776
          },
          {
            "t": 16.268507,
            "value": 304301.44821312884
          },
          {
            "t": 17.281463,
            "value": 274444.2996536868
          },
          {
            "t": 18.293909,
            "value": 341746.62154821097
          },
          {
            "t": 19.305542,
            "value": 272826.2126680328
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.095727,
            "value": 107288994.05940594
          },
          {
            "t": 3.116202,
            "value": 107275157.57433277
          },
          {
            "t": 5.137137,
            "value": 107263394.41891995
          },
          {
            "t": 7.057175,
            "value": 112605943.73653021
          },
          {
            "t": 9.079565,
            "value": 106904883.8255727
          },
          {
            "t": 11.10158,
            "value": 107203407.49203146
          },
          {
            "t": 13.122336,
            "value": 107251509.33610985
          },
          {
            "t": 15.143923,
            "value": 107216452.71759267
          },
          {
            "t": 17.177508,
            "value": 106591276.48954925
          },
          {
            "t": 19.20242,
            "value": 107043440.40629913
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.095727,
            "value": 98768601.98019803
          },
          {
            "t": 3.116202,
            "value": 111690155.03780054
          },
          {
            "t": 5.137137,
            "value": 111296895.74380174
          },
          {
            "t": 7.057175,
            "value": 104702176.72775227
          },
          {
            "t": 9.079565,
            "value": 111956863.41407937
          },
          {
            "t": 11.10158,
            "value": 111241351.3252869
          },
          {
            "t": 13.122336,
            "value": 103376804.0277995
          },
          {
            "t": 15.143923,
            "value": 105931495.899014
          },
          {
            "t": 17.177508,
            "value": 111216075.0595623
          },
          {
            "t": 19.20242,
            "value": 110495475.8527778
          }
        ],
        "ram_mib": [
          {
            "t": 1.095727,
            "value": 65.40234375
          },
          {
            "t": 3.116202,
            "value": 36.87109375
          },
          {
            "t": 5.137137,
            "value": 33.48828125
          },
          {
            "t": 7.057175,
            "value": 73.421875
          },
          {
            "t": 9.079565,
            "value": 38.01953125
          },
          {
            "t": 11.10158,
            "value": 30.546875
          },
          {
            "t": 13.122336,
            "value": 53.33203125
          },
          {
            "t": 15.143923,
            "value": 32.296875
          },
          {
            "t": 17.177508,
            "value": 34.19921875
          },
          {
            "t": 19.20242,
            "value": 34.53515625
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
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -0.7402597665786743
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 32.95196797094017
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 36.643391521197
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 55.36875
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 92.5
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 399533.13774543145
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/400k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 400324.4914622299
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000842
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 144226083.78096715
        },
        {
          "extra": "DFE OTLP Batch Processor w/ Gzip (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 143902011.50110322
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.117473,
            "value": 32.03104773713577
          },
          {
            "t": 3.038331,
            "value": 31.64217821782178
          },
          {
            "t": 5.060638,
            "value": 34.66290661719233
          },
          {
            "t": 7.083873,
            "value": 32.30494704049845
          },
          {
            "t": 9.109394,
            "value": 31.839601494396014
          },
          {
            "t": 11.13704,
            "value": 31.80623215394165
          },
          {
            "t": 13.169096,
            "value": 32.7870136307311
          },
          {
            "t": 15.103,
            "value": 32.69915632754342
          },
          {
            "t": 17.134405,
            "value": 33.1032049689441
          },
          {
            "t": 19.17473,
            "value": 36.643391521197
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.106636,
            "value": 395089.82367141166
          },
          {
            "t": 1.117473,
            "value": 395711.67260399053
          },
          {
            "t": 2.127893,
            "value": 494843.72835058684
          },
          {
            "t": 3.139252,
            "value": 395507.43109024595
          },
          {
            "t": 4.150536,
            "value": 395536.7631644523
          },
          {
            "t": 5.16161,
            "value": 396607.9634131626
          },
          {
            "t": 6.173372,
            "value": 394361.5198040646
          },
          {
            "t": 7.186169,
            "value": 394945.8776042978
          },
          {
            "t": 8.198188,
            "value": 395249.49630392314
          },
          {
            "t": 9.214085,
            "value": 393740.70402806584
          },
          {
            "t": 10.226203,
            "value": 395210.8351002551
          },
          {
            "t": 11.241893,
            "value": 393820.94930539833
          },
          {
            "t": 12.257757,
            "value": 393753.4945622642
          },
          {
            "t": 13.274217,
            "value": 393522.61771245307
          },
          {
            "t": 14.290612,
            "value": 393547.7840800083
          },
          {
            "t": 15.308693,
            "value": 392896.0465817553
          },
          {
            "t": 16.325053,
            "value": 393561.3365342989
          },
          {
            "t": 17.341736,
            "value": 393436.30217088317
          },
          {
            "t": 18.364181,
            "value": 391219.087579283
          },
          {
            "t": 19.37913,
            "value": 394108.47244541347
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.106636,
            "value": 474107.788405694
          },
          {
            "t": 1.117473,
            "value": 395711.67260399053
          },
          {
            "t": 2.127893,
            "value": 383009.0457433542
          },
          {
            "t": 3.139252,
            "value": 392541.1253570691
          },
          {
            "t": 4.150536,
            "value": 384659.50217742985
          },
          {
            "t": 5.16161,
            "value": 378805.1121876341
          },
          {
            "t": 6.173372,
            "value": 383489.39770420315
          },
          {
            "t": 7.186169,
            "value": 383097.5012761689
          },
          {
            "t": 8.198188,
            "value": 374498.89774796716
          },
          {
            "t": 9.214085,
            "value": 481348.01067431044
          },
          {
            "t": 10.327837,
            "value": 519864.38632657897
          },
          {
            "t": 11.343648,
            "value": 382945.25261096796
          },
          {
            "t": 12.360601,
            "value": 377598.5714187381
          },
          {
            "t": 13.376089,
            "value": 382082.309195185
          },
          {
            "t": 14.393797,
            "value": 381248.8454448624
          },
          {
            "t": 15.41006,
            "value": 382774.93129239185
          },
          {
            "t": 16.426324,
            "value": 459526.264828824
          },
          {
            "t": 17.443048,
            "value": 382601.37461100554
          },
          {
            "t": 18.465899,
            "value": 382264.8655571535
          },
          {
            "t": 19.483417,
            "value": 386233.95360082085
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.117473,
            "value": 143013780.20180255
          },
          {
            "t": 3.038331,
            "value": 150257374.56907278
          },
          {
            "t": 5.060638,
            "value": 142893589.3511717
          },
          {
            "t": 7.083873,
            "value": 142848558.86735845
          },
          {
            "t": 9.109394,
            "value": 142657323.7206625
          },
          {
            "t": 11.13704,
            "value": 141981601.3248861
          },
          {
            "t": 13.169096,
            "value": 142231961.61916798
          },
          {
            "t": 15.103,
            "value": 149418972.19303542
          },
          {
            "t": 17.134405,
            "value": 142069558.75367048
          },
          {
            "t": 19.17473,
            "value": 141647394.41020423
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.117473,
            "value": 139303945.84990102
          },
          {
            "t": 3.038331,
            "value": 146730578.21036226
          },
          {
            "t": 5.060638,
            "value": 137559620.27525988
          },
          {
            "t": 7.083873,
            "value": 155814558.86241588
          },
          {
            "t": 9.109394,
            "value": 137247430.6610497
          },
          {
            "t": 11.13704,
            "value": 137861318.49445122
          },
          {
            "t": 13.169096,
            "value": 153694946.89122742
          },
          {
            "t": 15.103,
            "value": 144683526.69005287
          },
          {
            "t": 17.134405,
            "value": 134452334.2218809
          },
          {
            "t": 19.17473,
            "value": 154912577.65306997
          }
        ],
        "ram_mib": [
          {
            "t": 1.117473,
            "value": 47.26953125
          },
          {
            "t": 3.038331,
            "value": 42.6796875
          },
          {
            "t": 5.060638,
            "value": 92.5
          },
          {
            "t": 7.083873,
            "value": 42.62109375
          },
          {
            "t": 9.109394,
            "value": 43.35546875
          },
          {
            "t": 11.13704,
            "value": 49.875
          },
          {
            "t": 13.169096,
            "value": 50.50390625
          },
          {
            "t": 15.103,
            "value": 40.41796875
          },
          {
            "t": 17.134405,
            "value": 68.44140625
          },
          {
            "t": 19.17473,
            "value": 76.0234375
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
