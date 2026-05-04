window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otap_none_batch"] = {
  "name": "OTC OTAP Batch Processor (Logs)",
  "slug": "otc_logs_otap_none_batch",
  "description": "OpenTelemetry Collector OTAP logs through a batch processor with no compression",
  "meta": {
    "binary": "otc",
    "protocols": [
      "otap"
    ],
    "signals": [
      "logs"
    ],
    "compression": "none"
  },
  "tests": [
    {
      "name": "100k",
      "metrics": [
        {
          "extra": "OTC OTAP Batch Processor (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.78947377204895
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 45.59721555057629
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 48.98062266500623
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 273.278515625
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 277.29296875
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 99395.32591910465
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/100k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 102167.93237895335
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000622
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 10326742.814967038
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 11059905.828159742
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.098273,
            "value": 48.98062266500623
          },
          {
            "t": 3.109667,
            "value": 45.0773929236499
          },
          {
            "t": 5.122266,
            "value": 47.192659640421574
          },
          {
            "t": 7.13515,
            "value": 45.898686493184634
          },
          {
            "t": 9.146911,
            "value": 43.338455836936376
          },
          {
            "t": 11.158267,
            "value": 44.98365217391304
          },
          {
            "t": 13.172056,
            "value": 44.971195046439625
          },
          {
            "t": 15.184426,
            "value": 45.92168944099379
          },
          {
            "t": 17.096362,
            "value": 44.37422718808194
          },
          {
            "t": 19.10708,
            "value": 45.233574097135744
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.091915,
            "value": 98299.90805483348
          },
          {
            "t": 1.098273,
            "value": 99368.21687709542
          },
          {
            "t": 2.10412,
            "value": 100412.88585639765
          },
          {
            "t": 3.109667,
            "value": 99448.35994737194
          },
          {
            "t": 4.116313,
            "value": 98346.38989277264
          },
          {
            "t": 5.122266,
            "value": 99408.22284937765
          },
          {
            "t": 6.128703,
            "value": 99360.41699579805
          },
          {
            "t": 7.13515,
            "value": 99359.42975636074
          },
          {
            "t": 8.14099,
            "value": 99419.39075797342
          },
          {
            "t": 9.146911,
            "value": 99411.38518830008
          },
          {
            "t": 10.152273,
            "value": 100461.32636801469
          },
          {
            "t": 11.158267,
            "value": 98410.129682682
          },
          {
            "t": 12.165513,
            "value": 100273.41880732213
          },
          {
            "t": 13.172056,
            "value": 98356.45372328852
          },
          {
            "t": 14.177777,
            "value": 99431.15436587283
          },
          {
            "t": 15.184426,
            "value": 99339.49171955668
          },
          {
            "t": 16.191234,
            "value": 99323.8035454625
          },
          {
            "t": 17.196885,
            "value": 99438.07543571277
          },
          {
            "t": 18.202422,
            "value": 99449.34895483707
          },
          {
            "t": 19.207502,
            "value": 99494.56759660922
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.091915,
            "value": 98299.90805483348
          },
          {
            "t": 1.098273,
            "value": 98374.53470832447
          },
          {
            "t": 2.10412,
            "value": 101407.07284507486
          },
          {
            "t": 3.109667,
            "value": 104420.77794474053
          },
          {
            "t": 4.116313,
            "value": 98346.38989277264
          },
          {
            "t": 5.122266,
            "value": 98414.14062088387
          },
          {
            "t": 6.128703,
            "value": 98366.81282584007
          },
          {
            "t": 7.13515,
            "value": 98365.83545879713
          },
          {
            "t": 8.14099,
            "value": 98425.1968503937
          },
          {
            "t": 9.146911,
            "value": 98417.27133641706
          },
          {
            "t": 10.152273,
            "value": 98471.99317260846
          },
          {
            "t": 11.158267,
            "value": 104374.3799664809
          },
          {
            "t": 12.165513,
            "value": 101266.2249341273
          },
          {
            "t": 13.172056,
            "value": 98356.45372328852
          },
          {
            "t": 14.177777,
            "value": 98436.84282221411
          },
          {
            "t": 15.184426,
            "value": 98346.0968023611
          },
          {
            "t": 16.191234,
            "value": 151965.41942455762
          },
          {
            "t": 17.196885,
            "value": 98443.69468135566
          },
          {
            "t": 18.202422,
            "value": 98454.85546528871
          },
          {
            "t": 19.207502,
            "value": 98499.62192064314
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.098273,
            "value": 10989828.555535689
          },
          {
            "t": 3.109667,
            "value": 11040777.6895029
          },
          {
            "t": 5.122266,
            "value": 10973306.157858571
          },
          {
            "t": 7.13515,
            "value": 10977619.674059706
          },
          {
            "t": 9.146911,
            "value": 11063789.883589553
          },
          {
            "t": 11.158267,
            "value": 10955138.225157555
          },
          {
            "t": 13.172056,
            "value": 10997246.980691621
          },
          {
            "t": 15.184426,
            "value": 11004309.843617227
          },
          {
            "t": 17.096362,
            "value": 11583442.646615786
          },
          {
            "t": 19.10708,
            "value": 11013598.624968793
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.098273,
            "value": 10336409.599300714
          },
          {
            "t": 3.109667,
            "value": 10117928.163253943
          },
          {
            "t": 5.122266,
            "value": 10340418.036578575
          },
          {
            "t": 7.13515,
            "value": 10339383.19346768
          },
          {
            "t": 9.146911,
            "value": 10115449.598635226
          },
          {
            "t": 11.158267,
            "value": 10270833.209039075
          },
          {
            "t": 13.172056,
            "value": 10411389.177316988
          },
          {
            "t": 15.184426,
            "value": 10112393.34714789
          },
          {
            "t": 17.096362,
            "value": 10643762.657327441
          },
          {
            "t": 19.10708,
            "value": 10579461.167602818
          }
        ],
        "ram_mib": [
          {
            "t": 1.098273,
            "value": 276.25390625
          },
          {
            "t": 3.109667,
            "value": 277.29296875
          },
          {
            "t": 5.122266,
            "value": 263.078125
          },
          {
            "t": 7.13515,
            "value": 276.73046875
          },
          {
            "t": 9.146911,
            "value": 273.625
          },
          {
            "t": 11.158267,
            "value": 275.0
          },
          {
            "t": 13.172056,
            "value": 276.37890625
          },
          {
            "t": 15.184426,
            "value": 275.546875
          },
          {
            "t": 17.096362,
            "value": 270.125
          },
          {
            "t": 19.10708,
            "value": 268.75390625
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
          "extra": "OTC OTAP Batch Processor (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 2.653846263885498
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 81.83343794512137
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 82.92584213797389
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 108.69609375
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 113.54296875
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 203570.85197376818
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/200k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 197129.6080571152
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000626
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 20598198.365544163
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 22075517.210804254
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.074991,
            "value": 82.07301310043668
          },
          {
            "t": 3.087999,
            "value": 82.35704630788486
          },
          {
            "t": 5.102364,
            "value": 80.2155223880597
          },
          {
            "t": 7.117693,
            "value": 81.68923268870867
          },
          {
            "t": 9.132554,
            "value": 81.9445153220763
          },
          {
            "t": 11.149414,
            "value": 80.42600249066003
          },
          {
            "t": 13.165663,
            "value": 81.64854828660437
          },
          {
            "t": 15.185521,
            "value": 82.92584213797389
          },
          {
            "t": 17.102545,
            "value": 82.58400996264011
          },
          {
            "t": 19.122399,
            "value": 82.47064676616915
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.067875,
            "value": 198739.59349803545
          },
          {
            "t": 1.074991,
            "value": 198586.85593317953
          },
          {
            "t": 2.081192,
            "value": 198767.44308542728
          },
          {
            "t": 3.087999,
            "value": 198647.80439547997
          },
          {
            "t": 4.095162,
            "value": 198577.5887319133
          },
          {
            "t": 5.102364,
            "value": 198569.8995832018
          },
          {
            "t": 6.110572,
            "value": 198371.7645565201
          },
          {
            "t": 7.117693,
            "value": 198585.8700195905
          },
          {
            "t": 8.125071,
            "value": 198535.20724097604
          },
          {
            "t": 9.132554,
            "value": 198514.51587768728
          },
          {
            "t": 10.140666,
            "value": 198390.6550065866
          },
          {
            "t": 11.149414,
            "value": 198265.57276941318
          },
          {
            "t": 12.15857,
            "value": 198185.4143462458
          },
          {
            "t": 13.165663,
            "value": 198591.3912617802
          },
          {
            "t": 14.174529,
            "value": 198242.38303203793
          },
          {
            "t": 15.185521,
            "value": 197825.50208112432
          },
          {
            "t": 16.196066,
            "value": 197913.00733762473
          },
          {
            "t": 17.206333,
            "value": 197967.46800598258
          },
          {
            "t": 18.216302,
            "value": 198025.8800022575
          },
          {
            "t": 19.225824,
            "value": 297170.34398457885
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.067875,
            "value": 202714.38536799615
          },
          {
            "t": 1.074991,
            "value": 193622.18453485004
          },
          {
            "t": 2.081192,
            "value": 199761.2803008544
          },
          {
            "t": 3.087999,
            "value": 196661.32635152517
          },
          {
            "t": 4.095162,
            "value": 196591.81284459415
          },
          {
            "t": 5.102364,
            "value": 199562.7490811178
          },
          {
            "t": 6.110572,
            "value": 202339.19984765048
          },
          {
            "t": 7.117693,
            "value": 196600.0113193946
          },
          {
            "t": 8.125071,
            "value": 199527.88327718092
          },
          {
            "t": 9.132554,
            "value": 195040.51184982774
          },
          {
            "t": 10.140666,
            "value": 200870.5381941689
          },
          {
            "t": 11.149414,
            "value": 199256.90063326023
          },
          {
            "t": 12.15857,
            "value": 196203.56020278332
          },
          {
            "t": 13.266839,
            "value": 182717.37276780276
          },
          {
            "t": 14.276124,
            "value": 200637.08466884974
          },
          {
            "t": 15.28666,
            "value": 195935.62228361977
          },
          {
            "t": 16.296978,
            "value": 195977.90002751607
          },
          {
            "t": 17.307274,
            "value": 198951.59438422005
          },
          {
            "t": 18.317221,
            "value": 201990.79753690047
          },
          {
            "t": 19.326778,
            "value": 194639.82717171987
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.074991,
            "value": 22048771.814460497
          },
          {
            "t": 3.087999,
            "value": 21941180.561627176
          },
          {
            "t": 5.102364,
            "value": 22034295.671340596
          },
          {
            "t": 7.117693,
            "value": 21916156.12140747
          },
          {
            "t": 9.132554,
            "value": 21961287.155788913
          },
          {
            "t": 11.149414,
            "value": 21966292.652935755
          },
          {
            "t": 13.165663,
            "value": 21956223.66086728
          },
          {
            "t": 15.185521,
            "value": 21918626.457899515
          },
          {
            "t": 17.102545,
            "value": 23093143.85213748
          },
          {
            "t": 19.122399,
            "value": 21919194.15957787
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.074991,
            "value": 20662691.250574883
          },
          {
            "t": 3.087999,
            "value": 20579019.06003354
          },
          {
            "t": 5.102364,
            "value": 20432479.218016595
          },
          {
            "t": 7.117693,
            "value": 20486833.66338697
          },
          {
            "t": 9.132554,
            "value": 20553924.067218535
          },
          {
            "t": 11.149414,
            "value": 20329912.33898238
          },
          {
            "t": 13.165663,
            "value": 20548143.111292303
          },
          {
            "t": 15.185521,
            "value": 20498472.664910104
          },
          {
            "t": 17.102545,
            "value": 21303449.513412457
          },
          {
            "t": 19.122399,
            "value": 20587058.767613895
          }
        ],
        "ram_mib": [
          {
            "t": 1.074991,
            "value": 108.15234375
          },
          {
            "t": 3.087999,
            "value": 113.54296875
          },
          {
            "t": 5.102364,
            "value": 107.953125
          },
          {
            "t": 7.117693,
            "value": 111.3671875
          },
          {
            "t": 9.132554,
            "value": 109.234375
          },
          {
            "t": 11.149414,
            "value": 105.1640625
          },
          {
            "t": 13.165663,
            "value": 108.65234375
          },
          {
            "t": 15.185521,
            "value": 108.46484375
          },
          {
            "t": 17.102545,
            "value": 103.6640625
          },
          {
            "t": 19.122399,
            "value": 110.765625
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
          "extra": "OTC OTAP Batch Processor (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -0.2894858717918396
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 89.45711430227361
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 99.68686567164178
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 113.1
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 126.8125
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 224315.33562078176
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/300k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 223788.39937369223
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000621
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 22909236.761681642
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 24583895.82387684
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.111003,
            "value": 99.68686567164178
          },
          {
            "t": 3.126659,
            "value": 94.04037336652146
          },
          {
            "t": 5.042076,
            "value": 92.77810945273632
          },
          {
            "t": 7.057564,
            "value": 91.17094527363184
          },
          {
            "t": 9.073768,
            "value": 91.55839208410637
          },
          {
            "t": 11.090462,
            "value": 88.39915685058897
          },
          {
            "t": 13.108552,
            "value": 90.15034825870647
          },
          {
            "t": 15.126336,
            "value": 81.4871756672874
          },
          {
            "t": 17.120777,
            "value": 82.31075776397515
          },
          {
            "t": 19.146823,
            "value": 82.98901863354037
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.102553,
            "value": 296664.9501771556
          },
          {
            "t": 1.111003,
            "value": 297486.2412613416
          },
          {
            "t": 2.118779,
            "value": 295700.631886451
          },
          {
            "t": 3.126659,
            "value": 226217.40683414688
          },
          {
            "t": 4.134533,
            "value": 226218.75353466807
          },
          {
            "t": 5.142686,
            "value": 224172.32305017192
          },
          {
            "t": 6.150462,
            "value": 224256.18391388562
          },
          {
            "t": 7.158215,
            "value": 225253.60877119692
          },
          {
            "t": 8.166617,
            "value": 223125.30121915662
          },
          {
            "t": 9.174568,
            "value": 225209.3603756532
          },
          {
            "t": 10.183212,
            "value": 225054.62779731996
          },
          {
            "t": 11.191118,
            "value": 223235.10327351955
          },
          {
            "t": 12.201627,
            "value": 247400.0726366613
          },
          {
            "t": 13.209204,
            "value": 198495.99583952394
          },
          {
            "t": 14.219423,
            "value": 217774.56175344158
          },
          {
            "t": 15.23217,
            "value": 203407.1688190634
          },
          {
            "t": 16.246248,
            "value": 199195.7226169979
          },
          {
            "t": 17.328066,
            "value": 185798.35055434465
          },
          {
            "t": 18.340663,
            "value": 198499.50177612616
          },
          {
            "t": 19.35224,
            "value": 198699.65410443296
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.102553,
            "value": 241102.28392324017
          },
          {
            "t": 1.111003,
            "value": 258813.02989736723
          },
          {
            "t": 2.118779,
            "value": 247078.71590512176
          },
          {
            "t": 3.126659,
            "value": 244076.67579473744
          },
          {
            "t": 4.134533,
            "value": 223242.19098815924
          },
          {
            "t": 5.142686,
            "value": 223180.4101163216
          },
          {
            "t": 6.150462,
            "value": 226240.75191312356
          },
          {
            "t": 7.158215,
            "value": 220292.07553835117
          },
          {
            "t": 8.166617,
            "value": 223125.30121915665
          },
          {
            "t": 9.174568,
            "value": 232154.14241366892
          },
          {
            "t": 10.183212,
            "value": 220097.47740530848
          },
          {
            "t": 11.191118,
            "value": 223235.10327351955
          },
          {
            "t": 12.201627,
            "value": 332505.6976236728
          },
          {
            "t": 13.310419,
            "value": 202923.54201689767
          },
          {
            "t": 14.320462,
            "value": 204941.76980583995
          },
          {
            "t": 15.333317,
            "value": 195487.01442950865
          },
          {
            "t": 16.415282,
            "value": 188545.8402074004
          },
          {
            "t": 17.429809,
            "value": 198121.88339985037
          },
          {
            "t": 18.441842,
            "value": 195645.79415888613
          },
          {
            "t": 19.453422,
            "value": 197216.23598726746
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.111003,
            "value": 27550112.308506433
          },
          {
            "t": 3.126659,
            "value": 26594836.618946884
          },
          {
            "t": 5.042076,
            "value": 26044790.76879865
          },
          {
            "t": 7.057564,
            "value": 24912475.291343834
          },
          {
            "t": 9.073768,
            "value": 24911676.100235887
          },
          {
            "t": 11.090462,
            "value": 24736478.61301714
          },
          {
            "t": 13.108552,
            "value": 24446793.750526488
          },
          {
            "t": 15.126336,
            "value": 22432333.19324566
          },
          {
            "t": 17.120777,
            "value": 22196640.562443312
          },
          {
            "t": 19.146823,
            "value": 22012821.031704117
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.111003,
            "value": 25668390.102567993
          },
          {
            "t": 3.126659,
            "value": 24992441.666633595
          },
          {
            "t": 5.042076,
            "value": 24363964.609273072
          },
          {
            "t": 7.057564,
            "value": 23072625.091293026
          },
          {
            "t": 9.073768,
            "value": 23219310.64515297
          },
          {
            "t": 11.090462,
            "value": 22982654.780546773
          },
          {
            "t": 13.108552,
            "value": 22814757.02272941
          },
          {
            "t": 15.126336,
            "value": 21065139.281508826
          },
          {
            "t": 17.120777,
            "value": 20756779.969926413
          },
          {
            "t": 19.146823,
            "value": 20156304.447184317
          }
        ],
        "ram_mib": [
          {
            "t": 1.111003,
            "value": 126.8125
          },
          {
            "t": 3.126659,
            "value": 112.97265625
          },
          {
            "t": 5.042076,
            "value": 113.3671875
          },
          {
            "t": 7.057564,
            "value": 126.71875
          },
          {
            "t": 9.073768,
            "value": 108.875
          },
          {
            "t": 11.090462,
            "value": 104.6328125
          },
          {
            "t": 13.108552,
            "value": 100.75390625
          },
          {
            "t": 15.126336,
            "value": 106.44921875
          },
          {
            "t": 17.120777,
            "value": 110.76953125
          },
          {
            "t": 19.146823,
            "value": 119.6484375
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
          "extra": "OTC OTAP Batch Processor (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.134666919708252
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 87.29882785870127
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.14457711442786
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 154.2140625
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 296.3046875
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 212352.3260272874
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/400k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 219177.22445773857
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000698
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 22688799.890707918
        },
        {
          "extra": "OTC OTAP Batch Processor (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 24284877.206656925
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.103116,
            "value": 99.98122905027932
          },
          {
            "t": 3.121989,
            "value": 100.14457711442786
          },
          {
            "t": 5.039142,
            "value": 99.45158348736906
          },
          {
            "t": 7.057028,
            "value": 81.86260442260442
          },
          {
            "t": 9.087921,
            "value": 81.79768800497203
          },
          {
            "t": 11.117154,
            "value": 82.27810945273632
          },
          {
            "t": 13.136504,
            "value": 81.71353233830845
          },
          {
            "t": 15.154772,
            "value": 82.25325912183055
          },
          {
            "t": 17.174078,
            "value": 82.43001857585139
          },
          {
            "t": 19.194523,
            "value": 81.07567701863354
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.093542,
            "value": 396167.0834674524
          },
          {
            "t": 1.103116,
            "value": 281306.76899365475
          },
          {
            "t": 2.112083,
            "value": 252733.7365840508
          },
          {
            "t": 3.121989,
            "value": 248537.98274294834
          },
          {
            "t": 4.130822,
            "value": 232942.41960760596
          },
          {
            "t": 5.139893,
            "value": 221986.36171290223
          },
          {
            "t": 6.148933,
            "value": 212082.7717434393
          },
          {
            "t": 7.162547,
            "value": 204219.75229229272
          },
          {
            "t": 8.176696,
            "value": 200167.82543787948
          },
          {
            "t": 9.194006,
            "value": 200528.84568125746
          },
          {
            "t": 10.208846,
            "value": 198060.77805368335
          },
          {
            "t": 11.319075,
            "value": 182845.16077313782
          },
          {
            "t": 12.328269,
            "value": 202141.510948341
          },
          {
            "t": 13.338349,
            "value": 198994.13907809282
          },
          {
            "t": 14.347672,
            "value": 199143.3862103608
          },
          {
            "t": 15.356679,
            "value": 200196.82717761127
          },
          {
            "t": 16.366248,
            "value": 203056.9480639758
          },
          {
            "t": 17.375961,
            "value": 200056.8478369596
          },
          {
            "t": 18.386589,
            "value": 199875.72083892388
          },
          {
            "t": 19.396368,
            "value": 199053.4562513184
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.093542,
            "value": 239185.87664347436
          },
          {
            "t": 1.103116,
            "value": 255553.3323956441
          },
          {
            "t": 2.112083,
            "value": 233407.0390805646
          },
          {
            "t": 3.121989,
            "value": 252498.74740817462
          },
          {
            "t": 4.130822,
            "value": 240872.37431765217
          },
          {
            "t": 5.139893,
            "value": 234869.49877659747
          },
          {
            "t": 6.148933,
            "value": 288392.92793149926
          },
          {
            "t": 7.162547,
            "value": 235296.67111938074
          },
          {
            "t": 8.176696,
            "value": 202632.946440809
          },
          {
            "t": 9.087921,
            "value": 225520.59041400315
          },
          {
            "t": 10.102467,
            "value": 196639.6792259789
          },
          {
            "t": 11.117154,
            "value": 198090.64273022127
          },
          {
            "t": 12.126433,
            "value": 194693.43957419108
          },
          {
            "t": 13.136504,
            "value": 207906.17689251548
          },
          {
            "t": 14.145585,
            "value": 200677.64629400417
          },
          {
            "t": 15.154772,
            "value": 202142.91305773854
          },
          {
            "t": 16.164268,
            "value": 196137.47850412485
          },
          {
            "t": 17.174078,
            "value": 200532.77349204305
          },
          {
            "t": 18.183157,
            "value": 203651.05209800223
          },
          {
            "t": 19.194523,
            "value": 195774.8233577162
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.103116,
            "value": 28516353.852348078
          },
          {
            "t": 3.121989,
            "value": 29881321.90583558
          },
          {
            "t": 5.039142,
            "value": 29138703.588080864
          },
          {
            "t": 7.057028,
            "value": 22559396.81428981
          },
          {
            "t": 9.087921,
            "value": 22066034.005730487
          },
          {
            "t": 11.117154,
            "value": 22255997.216682363
          },
          {
            "t": 13.136504,
            "value": 22093638.54705722
          },
          {
            "t": 15.154772,
            "value": 22203559.190355293
          },
          {
            "t": 17.174078,
            "value": 22185915.359039195
          },
          {
            "t": 19.194523,
            "value": 21947851.587150354
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.103116,
            "value": 25997273.243666336
          },
          {
            "t": 3.121989,
            "value": 24784863.634314787
          },
          {
            "t": 5.039142,
            "value": 27157891.936637294
          },
          {
            "t": 7.057028,
            "value": 25170420.430093676
          },
          {
            "t": 9.087921,
            "value": 20839764.576469563
          },
          {
            "t": 11.117154,
            "value": 20578348.568153583
          },
          {
            "t": 13.136504,
            "value": 20588883.551637903
          },
          {
            "t": 15.154772,
            "value": 20602080.59583762
          },
          {
            "t": 17.174078,
            "value": 20818400.48016497
          },
          {
            "t": 19.194523,
            "value": 20350071.89010342
          }
        ],
        "ram_mib": [
          {
            "t": 1.103116,
            "value": 180.32421875
          },
          {
            "t": 3.121989,
            "value": 249.74609375
          },
          {
            "t": 5.039142,
            "value": 296.3046875
          },
          {
            "t": 7.057028,
            "value": 122.8515625
          },
          {
            "t": 9.087921,
            "value": 113.1484375
          },
          {
            "t": 11.117154,
            "value": 132.19140625
          },
          {
            "t": 13.136504,
            "value": 112.25390625
          },
          {
            "t": 15.154772,
            "value": 116.78515625
          },
          {
            "t": 17.174078,
            "value": 104.625
          },
          {
            "t": 19.194523,
            "value": 113.91015625
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
