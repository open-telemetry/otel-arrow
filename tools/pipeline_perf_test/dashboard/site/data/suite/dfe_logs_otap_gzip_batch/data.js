window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otap_gzip_batch"] = {
  "name": "DFE OTAP Batch Processor w/ Gzip (Logs)",
  "slug": "dfe_logs_otap_gzip_batch",
  "description": "Dataflow Engine OTAP logs through a batch processor with gzip compression",
  "meta": {
    "binary": "dfe",
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
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.052631575614213943
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 10.300261753575072
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 10.925530864197531
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 19.669140625
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 20.484375
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 99254.4683447463
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/100k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 99202.22915088064
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000989
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 861783.8861419328
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 791860.4441743094
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.019321,
            "value": 10.925530864197531
          },
          {
            "t": 3.034542,
            "value": 10.592594427244583
          },
          {
            "t": 5.048553,
            "value": 10.02461919504644
          },
          {
            "t": 7.06266,
            "value": 9.826633663366337
          },
          {
            "t": 9.077418,
            "value": 10.431284916201117
          },
          {
            "t": 11.091695,
            "value": 9.67771144278607
          },
          {
            "t": 13.106123,
            "value": 10.592990712074304
          },
          {
            "t": 15.121454,
            "value": 10.840696517412935
          },
          {
            "t": 17.137681,
            "value": 9.906501240694789
          },
          {
            "t": 19.153379,
            "value": 10.184054556726597
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.111206,
            "value": 99285.44266911042
          },
          {
            "t": 1.119881,
            "value": 99139.96083971547
          },
          {
            "t": 2.128231,
            "value": 99171.91451380968
          },
          {
            "t": 3.135054,
            "value": 99322.3237848162
          },
          {
            "t": 4.142651,
            "value": 99246.02792584733
          },
          {
            "t": 5.149143,
            "value": 99354.9874216586
          },
          {
            "t": 6.156765,
            "value": 99243.56554342799
          },
          {
            "t": 7.163245,
            "value": 99356.17200540497
          },
          {
            "t": 8.169943,
            "value": 99334.65647095752
          },
          {
            "t": 9.178044,
            "value": 99196.4098835335
          },
          {
            "t": 10.185951,
            "value": 99215.50301763951
          },
          {
            "t": 11.192169,
            "value": 99382.04245998382
          },
          {
            "t": 12.199484,
            "value": 99273.81206474638
          },
          {
            "t": 13.206767,
            "value": 99276.96585765868
          },
          {
            "t": 14.214581,
            "value": 99224.65851833772
          },
          {
            "t": 15.222196,
            "value": 99244.2549981888
          },
          {
            "t": 16.23149,
            "value": 99079.1583027344
          },
          {
            "t": 17.238192,
            "value": 99334.26177756675
          },
          {
            "t": 18.246553,
            "value": 99170.83266806233
          },
          {
            "t": 19.253921,
            "value": 99268.58903598288
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.111206,
            "value": 98292.58824241931
          },
          {
            "t": 1.119881,
            "value": 98148.56123131831
          },
          {
            "t": 2.128231,
            "value": 98180.19536867159
          },
          {
            "t": 3.135054,
            "value": 98329.10054696804
          },
          {
            "t": 4.142651,
            "value": 98253.56764658885
          },
          {
            "t": 5.149143,
            "value": 98361.43754744201
          },
          {
            "t": 6.156765,
            "value": 98251.12988799371
          },
          {
            "t": 7.163245,
            "value": 98362.61028535092
          },
          {
            "t": 8.169943,
            "value": 107281.42898863411
          },
          {
            "t": 9.178044,
            "value": 98204.44578469817
          },
          {
            "t": 10.185951,
            "value": 98223.34798746312
          },
          {
            "t": 11.192169,
            "value": 98388.22203538398
          },
          {
            "t": 12.199484,
            "value": 98281.07394409893
          },
          {
            "t": 13.206767,
            "value": 98284.19619908209
          },
          {
            "t": 14.214581,
            "value": 98232.41193315433
          },
          {
            "t": 15.222196,
            "value": 98251.81244820691
          },
          {
            "t": 16.23149,
            "value": 107005.49096695315
          },
          {
            "t": 17.238192,
            "value": 98340.91915979108
          },
          {
            "t": 18.246553,
            "value": 98179.1243413817
          },
          {
            "t": 19.253921,
            "value": 98275.90314562304
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.019321,
            "value": 829093.1190323039
          },
          {
            "t": 3.034542,
            "value": 788080.810987976
          },
          {
            "t": 5.048553,
            "value": 788507.6099385753
          },
          {
            "t": 7.06266,
            "value": 788388.6010028265
          },
          {
            "t": 9.077418,
            "value": 788305.5930290387
          },
          {
            "t": 11.091695,
            "value": 784378.2161043392
          },
          {
            "t": 13.106123,
            "value": 788342.8943600863
          },
          {
            "t": 15.121454,
            "value": 788017.9484164139
          },
          {
            "t": 17.137681,
            "value": 787647.9186123387
          },
          {
            "t": 19.153379,
            "value": 787841.7302591956
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.019321,
            "value": 890473.7191642929
          },
          {
            "t": 3.034542,
            "value": 882202.0016663185
          },
          {
            "t": 5.048553,
            "value": 847253.5651493463
          },
          {
            "t": 7.06266,
            "value": 847093.5258156592
          },
          {
            "t": 9.077418,
            "value": 882294.5485264235
          },
          {
            "t": 11.091695,
            "value": 846525.5771673906
          },
          {
            "t": 13.106123,
            "value": 847310.9984571302
          },
          {
            "t": 15.121454,
            "value": 882440.1549919094
          },
          {
            "t": 17.137681,
            "value": 846125.9570474951
          },
          {
            "t": 19.153379,
            "value": 846118.8134333616
          }
        ],
        "ram_mib": [
          {
            "t": 1.019321,
            "value": 18.08203125
          },
          {
            "t": 3.034542,
            "value": 19.13671875
          },
          {
            "t": 5.048553,
            "value": 19.5078125
          },
          {
            "t": 7.06266,
            "value": 20.11328125
          },
          {
            "t": 9.077418,
            "value": 20.484375
          },
          {
            "t": 11.091695,
            "value": 19.76953125
          },
          {
            "t": 13.106123,
            "value": 19.66796875
          },
          {
            "t": 15.121454,
            "value": 19.91796875
          },
          {
            "t": 17.137681,
            "value": 19.90234375
          },
          {
            "t": 19.153379,
            "value": 20.109375
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
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.052631575614213943
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 17.367778865543986
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 18.177700865265763
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 19.74765625
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 20.421875
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 198371.56779993037
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/200k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 198267.1617116146
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.001193
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 1684711.9364182628
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 1583855.4983552466
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.116985,
            "value": 16.007060285891857
          },
          {
            "t": 3.132734,
            "value": 16.512535647861128
          },
          {
            "t": 5.048145,
            "value": 17.422772277227722
          },
          {
            "t": 7.06332,
            "value": 17.510535491905355
          },
          {
            "t": 9.082503,
            "value": 18.177700865265763
          },
          {
            "t": 11.098699,
            "value": 17.49207687538748
          },
          {
            "t": 13.113929,
            "value": 17.542024691358023
          },
          {
            "t": 15.131371,
            "value": 17.716499068901303
          },
          {
            "t": 17.147576,
            "value": 17.487771570453134
          },
          {
            "t": 19.165471,
            "value": 17.80881188118812
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.110169,
            "value": 198503.08821179485
          },
          {
            "t": 1.116985,
            "value": 198646.02866859487
          },
          {
            "t": 2.125321,
            "value": 198346.58288507006
          },
          {
            "t": 3.132734,
            "value": 198528.309640634
          },
          {
            "t": 4.141089,
            "value": 198342.84552563337
          },
          {
            "t": 5.148672,
            "value": 198494.81382675175
          },
          {
            "t": 6.155914,
            "value": 198562.01389536975
          },
          {
            "t": 7.164038,
            "value": 198388.29350357695
          },
          {
            "t": 8.173457,
            "value": 198133.77794553104
          },
          {
            "t": 9.183305,
            "value": 198049.60746567798
          },
          {
            "t": 10.191063,
            "value": 198460.34464623453
          },
          {
            "t": 11.199537,
            "value": 198319.4410564873
          },
          {
            "t": 12.206847,
            "value": 198548.60966336084
          },
          {
            "t": 13.214561,
            "value": 198469.01005642474
          },
          {
            "t": 14.223646,
            "value": 198199.3588250742
          },
          {
            "t": 15.231956,
            "value": 198351.69739465046
          },
          {
            "t": 16.239662,
            "value": 198470.58566685126
          },
          {
            "t": 17.248379,
            "value": 198271.66588845037
          },
          {
            "t": 18.257494,
            "value": 198193.46655237512
          },
          {
            "t": 19.26614,
            "value": 198285.62250779758
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.110169,
            "value": 196518.0573296769
          },
          {
            "t": 1.116985,
            "value": 196659.56838190893
          },
          {
            "t": 2.125321,
            "value": 196363.11705621937
          },
          {
            "t": 3.132734,
            "value": 196543.02654422767
          },
          {
            "t": 4.141089,
            "value": 205284.84511903053
          },
          {
            "t": 5.148672,
            "value": 196509.86568848425
          },
          {
            "t": 6.155914,
            "value": 196576.39375641604
          },
          {
            "t": 7.164038,
            "value": 196404.41056854118
          },
          {
            "t": 8.173457,
            "value": 205068.46017362463
          },
          {
            "t": 9.183305,
            "value": 196069.1113910212
          },
          {
            "t": 10.191063,
            "value": 196475.7411997722
          },
          {
            "t": 11.199537,
            "value": 196336.24664592242
          },
          {
            "t": 12.206847,
            "value": 196563.12356672724
          },
          {
            "t": 13.214561,
            "value": 205415.4254083996
          },
          {
            "t": 14.223646,
            "value": 196217.36523682345
          },
          {
            "t": 15.231956,
            "value": 196368.18042070395
          },
          {
            "t": 16.239662,
            "value": 196485.87981018273
          },
          {
            "t": 17.248379,
            "value": 205211.17419454613
          },
          {
            "t": 18.257494,
            "value": 196211.53188685136
          },
          {
            "t": 19.26614,
            "value": 196302.76628271962
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.116985,
            "value": 1574268.612763279
          },
          {
            "t": 3.132734,
            "value": 1577251.929679737
          },
          {
            "t": 5.048145,
            "value": 1655766.8301998891
          },
          {
            "t": 7.06332,
            "value": 1577686.3051596014
          },
          {
            "t": 9.082503,
            "value": 1574579.9167286968
          },
          {
            "t": 11.098699,
            "value": 1576744.522853929
          },
          {
            "t": 13.113929,
            "value": 1577686.9141487572
          },
          {
            "t": 15.131371,
            "value": 1575965.504832357
          },
          {
            "t": 17.147576,
            "value": 1572882.2217978828
          },
          {
            "t": 19.165471,
            "value": 1575722.2253883376
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.116985,
            "value": 1656072.384388666
          },
          {
            "t": 3.132734,
            "value": 1690762.0938916502
          },
          {
            "t": 5.048145,
            "value": 1778911.1579707959
          },
          {
            "t": 7.06332,
            "value": 1655388.7379507981
          },
          {
            "t": 9.082503,
            "value": 1687901.4928315065
          },
          {
            "t": 11.098699,
            "value": 1654480.516775155
          },
          {
            "t": 13.113929,
            "value": 1691192.5685901858
          },
          {
            "t": 15.131371,
            "value": 1689055.7448491703
          },
          {
            "t": 17.147576,
            "value": 1654240.0202360377
          },
          {
            "t": 19.165471,
            "value": 1689114.6466986635
          }
        ],
        "ram_mib": [
          {
            "t": 1.116985,
            "value": 19.0859375
          },
          {
            "t": 3.132734,
            "value": 19.9140625
          },
          {
            "t": 5.048145,
            "value": 19.375
          },
          {
            "t": 7.06332,
            "value": 19.6953125
          },
          {
            "t": 9.082503,
            "value": 20.3046875
          },
          {
            "t": 11.098699,
            "value": 20.421875
          },
          {
            "t": 13.113929,
            "value": 19.546875
          },
          {
            "t": 15.131371,
            "value": 19.5703125
          },
          {
            "t": 17.147576,
            "value": 19.73828125
          },
          {
            "t": 19.165471,
            "value": 19.82421875
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
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 1.7758620977401733
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 24.82121329665862
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 25.57916924984501
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 20.089453125
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 21.02734375
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 301221.52599656023
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/300k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 297495.973725941
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000616
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 2500202.4247121015
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 2375349.0339819845
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.070718,
            "value": 24.714772868699438
          },
          {
            "t": 3.092436,
            "value": 24.806062111801243
          },
          {
            "t": 5.115543,
            "value": 24.646077210460774
          },
          {
            "t": 7.144883,
            "value": 25.57916924984501
          },
          {
            "t": 9.069517,
            "value": 24.741628340584214
          },
          {
            "t": 11.099556,
            "value": 24.562830540037243
          },
          {
            "t": 13.123699,
            "value": 24.550459627329193
          },
          {
            "t": 15.151845,
            "value": 24.570759651307597
          },
          {
            "t": 17.181136,
            "value": 24.691972619788423
          },
          {
            "t": 19.108277,
            "value": 25.34840074673304
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.059113,
            "value": 296752.63590528845
          },
          {
            "t": 1.070718,
            "value": 296558.4393117867
          },
          {
            "t": 2.080875,
            "value": 296983.53820247744
          },
          {
            "t": 3.092436,
            "value": 297559.9098818559
          },
          {
            "t": 4.103927,
            "value": 295603.2233603661
          },
          {
            "t": 5.115543,
            "value": 296555.214626894
          },
          {
            "t": 6.129762,
            "value": 295794.10364033806
          },
          {
            "t": 7.144883,
            "value": 295531.27164150873
          },
          {
            "t": 8.159528,
            "value": 295669.91410789
          },
          {
            "t": 9.174364,
            "value": 394152.35565155355
          },
          {
            "t": 10.189019,
            "value": 295667.000113339
          },
          {
            "t": 11.201682,
            "value": 296248.603928454
          },
          {
            "t": 12.215455,
            "value": 295924.2355043979
          },
          {
            "t": 13.227101,
            "value": 297534.90845612
          },
          {
            "t": 14.242552,
            "value": 294450.44615643687
          },
          {
            "t": 15.261331,
            "value": 294470.1451443345
          },
          {
            "t": 16.272764,
            "value": 296608.87078036804
          },
          {
            "t": 17.285368,
            "value": 296265.8650370727
          },
          {
            "t": 18.300045,
            "value": 295660.58952750475
          },
          {
            "t": 19.314045,
            "value": 295857.9881656805
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.059113,
            "value": 302687.68862339424
          },
          {
            "t": 1.070718,
            "value": 293592.8549186688
          },
          {
            "t": 2.080875,
            "value": 294013.7028204527
          },
          {
            "t": 3.092436,
            "value": 302502.7655277339
          },
          {
            "t": 4.103927,
            "value": 293625.94427434355
          },
          {
            "t": 5.115543,
            "value": 293589.66248062503
          },
          {
            "t": 6.129762,
            "value": 301709.9857131448
          },
          {
            "t": 7.144883,
            "value": 292575.9589250937
          },
          {
            "t": 8.159528,
            "value": 292713.21496681107
          },
          {
            "t": 9.174364,
            "value": 301526.5520734384
          },
          {
            "t": 10.189019,
            "value": 292710.3301122056
          },
          {
            "t": 11.201682,
            "value": 293286.1178891694
          },
          {
            "t": 12.114653,
            "value": 335169.4632140561
          },
          {
            "t": 13.123699,
            "value": 294337.42366552167
          },
          {
            "t": 14.13801,
            "value": 292809.6017888005
          },
          {
            "t": 15.151845,
            "value": 301824.2613443016
          },
          {
            "t": 16.169684,
            "value": 291794.6747963087
          },
          {
            "t": 17.181136,
            "value": 293637.26602943096
          },
          {
            "t": 18.193747,
            "value": 302189.09334384085
          },
          {
            "t": 19.208952,
            "value": 292551.75063164585
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.070718,
            "value": 2353024.007341235
          },
          {
            "t": 3.092436,
            "value": 2350158.6274643647
          },
          {
            "t": 5.115543,
            "value": 2356191.7387463935
          },
          {
            "t": 7.144883,
            "value": 2349182.4928301815
          },
          {
            "t": 9.069517,
            "value": 2476719.2099900553
          },
          {
            "t": 11.099556,
            "value": 2348189.86236225
          },
          {
            "t": 13.123699,
            "value": 2355067.305027362
          },
          {
            "t": 15.151845,
            "value": 2342652.8464913275
          },
          {
            "t": 17.181136,
            "value": 2356796.5363272196
          },
          {
            "t": 19.108277,
            "value": 2465507.713239457
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.070718,
            "value": 2482061.735988466
          },
          {
            "t": 3.092436,
            "value": 2482973.391936957
          },
          {
            "t": 5.115543,
            "value": 2446201.807417996
          },
          {
            "t": 7.144883,
            "value": 2509438.043896045
          },
          {
            "t": 9.069517,
            "value": 2608455.9453901364
          },
          {
            "t": 11.099556,
            "value": 2437594.5486761583
          },
          {
            "t": 13.123699,
            "value": 2480281.2844744665
          },
          {
            "t": 15.151845,
            "value": 2475143.3082233723
          },
          {
            "t": 17.181136,
            "value": 2474784.0501928995
          },
          {
            "t": 19.108277,
            "value": 2605090.1309245145
          }
        ],
        "ram_mib": [
          {
            "t": 1.070718,
            "value": 21.02734375
          },
          {
            "t": 3.092436,
            "value": 19.99609375
          },
          {
            "t": 5.115543,
            "value": 19.7890625
          },
          {
            "t": 7.144883,
            "value": 19.98046875
          },
          {
            "t": 9.069517,
            "value": 19.859375
          },
          {
            "t": 11.099556,
            "value": 20.5546875
          },
          {
            "t": 13.123699,
            "value": 20.21875
          },
          {
            "t": 15.151845,
            "value": 19.703125
          },
          {
            "t": 17.181136,
            "value": 19.80078125
          },
          {
            "t": 19.108277,
            "value": 19.96484375
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
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -0.800000011920929
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 24.53280490979004
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 26.116129032258062
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 19.954296875
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 20.50390625
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 286196.9848054899
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/400k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 290014.72096728487
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000548
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 2406961.2662220034
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Gzip (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 2283029.505676779
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.080909,
            "value": 18.079103920348473
          },
          {
            "t": 3.100645,
            "value": 25.70554037267081
          },
          {
            "t": 5.121187,
            "value": 26.116129032258062
          },
          {
            "t": 7.141522,
            "value": 24.9990099009901
          },
          {
            "t": 9.16157,
            "value": 25.860774032459428
          },
          {
            "t": 11.084576,
            "value": 24.56639801611903
          },
          {
            "t": 13.110362,
            "value": 24.829036668738347
          },
          {
            "t": 15.13719,
            "value": 25.464401735895848
          },
          {
            "t": 17.164085,
            "value": 24.554143920595532
          },
          {
            "t": 19.185847,
            "value": 25.153511497824738
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.069576,
            "value": 198063.72898543836
          },
          {
            "t": 1.080909,
            "value": 198747.59352260828
          },
          {
            "t": 2.091034,
            "value": 197005.32112362335
          },
          {
            "t": 3.100645,
            "value": 198096.09839829398
          },
          {
            "t": 4.110892,
            "value": 296957.080793113
          },
          {
            "t": 5.121187,
            "value": 296942.9721022078
          },
          {
            "t": 6.131403,
            "value": 296966.1933685469
          },
          {
            "t": 7.141522,
            "value": 296994.7105242056
          },
          {
            "t": 8.151766,
            "value": 296957.96263081
          },
          {
            "t": 9.16157,
            "value": 297087.35556603066
          },
          {
            "t": 10.175182,
            "value": 295971.2394880882
          },
          {
            "t": 11.187837,
            "value": 296250.9442998849
          },
          {
            "t": 12.200865,
            "value": 296141.863798434
          },
          {
            "t": 13.216261,
            "value": 393934.9770926811
          },
          {
            "t": 14.227966,
            "value": 296529.1265734576
          },
          {
            "t": 15.240198,
            "value": 296374.7441298042
          },
          {
            "t": 16.252027,
            "value": 296492.7868246512
          },
          {
            "t": 17.265174,
            "value": 296107.080216395
          },
          {
            "t": 18.276478,
            "value": 296646.7056394516
          },
          {
            "t": 19.287109,
            "value": 296844.2487911018
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.069576,
            "value": 204995.9594999287
          },
          {
            "t": 1.080909,
            "value": 195781.21152973353
          },
          {
            "t": 2.091034,
            "value": 249474.07499071897
          },
          {
            "t": 3.100645,
            "value": 294172.7061214666
          },
          {
            "t": 4.110892,
            "value": 293987.50998518185
          },
          {
            "t": 5.121187,
            "value": 302881.83154425194
          },
          {
            "t": 6.131403,
            "value": 293996.5314348615
          },
          {
            "t": 7.141522,
            "value": 294024.7634189635
          },
          {
            "t": 8.151766,
            "value": 302897.1218834262
          },
          {
            "t": 9.16157,
            "value": 294116.48201037035
          },
          {
            "t": 10.175182,
            "value": 293011.5270932073
          },
          {
            "t": 11.187837,
            "value": 302175.9631858826
          },
          {
            "t": 12.200865,
            "value": 293180.44516044966
          },
          {
            "t": 13.216261,
            "value": 292496.7204913157
          },
          {
            "t": 14.125942,
            "value": 336381.65466795507
          },
          {
            "t": 15.13719,
            "value": 293696.50174833473
          },
          {
            "t": 16.150769,
            "value": 293021.0669321286
          },
          {
            "t": 17.164085,
            "value": 301978.8496382175
          },
          {
            "t": 18.174525,
            "value": 293931.35663671274
          },
          {
            "t": 19.185847,
            "value": 293675.0115195754
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.080909,
            "value": 1573533.6392050106
          },
          {
            "t": 3.100645,
            "value": 2299387.6427414278
          },
          {
            "t": 5.121187,
            "value": 2355654.076975386
          },
          {
            "t": 7.141522,
            "value": 2359784.3921923838
          },
          {
            "t": 9.16157,
            "value": 2360151.8379761274
          },
          {
            "t": 11.084576,
            "value": 2475070.2805919484
          },
          {
            "t": 13.110362,
            "value": 2353287.5634445096
          },
          {
            "t": 15.13719,
            "value": 2346691.48048083
          },
          {
            "t": 17.164085,
            "value": 2350431.57144302
          },
          {
            "t": 19.185847,
            "value": 2356302.571717146
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.080909,
            "value": 1651943.5616782666
          },
          {
            "t": 3.100645,
            "value": 2417035.691793383
          },
          {
            "t": 5.121187,
            "value": 2487145.528279046
          },
          {
            "t": 7.141522,
            "value": 2487366.2041196134
          },
          {
            "t": 9.16157,
            "value": 2487574.5526838964
          },
          {
            "t": 11.084576,
            "value": 2613404.7423668983
          },
          {
            "t": 13.110362,
            "value": 2480934.80752656
          },
          {
            "t": 15.13719,
            "value": 2479533.5371328993
          },
          {
            "t": 17.164085,
            "value": 2479191.5713443467
          },
          {
            "t": 19.185847,
            "value": 2485482.465295124
          }
        ],
        "ram_mib": [
          {
            "t": 1.080909,
            "value": 19.08203125
          },
          {
            "t": 3.100645,
            "value": 19.92578125
          },
          {
            "t": 5.121187,
            "value": 19.71484375
          },
          {
            "t": 7.141522,
            "value": 20.27734375
          },
          {
            "t": 9.16157,
            "value": 20.50390625
          },
          {
            "t": 11.084576,
            "value": 20.328125
          },
          {
            "t": 13.110362,
            "value": 19.67578125
          },
          {
            "t": 15.13719,
            "value": 19.9140625
          },
          {
            "t": 17.164085,
            "value": 19.9140625
          },
          {
            "t": 19.185847,
            "value": 20.20703125
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
