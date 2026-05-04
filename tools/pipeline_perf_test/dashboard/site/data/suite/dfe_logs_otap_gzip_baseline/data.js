window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otap_gzip_baseline"] = {
  "name": "DFE OTAP Baseline w/ Gzip (Logs)",
  "slug": "dfe_logs_otap_gzip_baseline",
  "description": "Dataflow Engine baseline for OTAP logs with gzip compression",
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
      "name": "1000k",
      "metrics": [
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/1000k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -1.015544056892395
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/1000k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 61.702420455215034
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/1000k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 62.451881188118804
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/1000k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 16.31953125
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/1000k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 16.7265625
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/1000k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 984069.4453218639
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/1000k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 1020549.018310469
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/1000k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000641
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/1000k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 8068001.088783364
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/1000k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 8005813.843041515
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.119317,
            "value": 61.05613776137761
          },
          {
            "t": 3.05539,
            "value": 61.61439309919901
          },
          {
            "t": 5.08263,
            "value": 61.422113022113024
          },
          {
            "t": 7.111649,
            "value": 61.89256299938537
          },
          {
            "t": 9.145572,
            "value": 61.875582973473165
          },
          {
            "t": 11.174293,
            "value": 61.322352941176476
          },
          {
            "t": 13.11209,
            "value": 62.451881188118804
          },
          {
            "t": 15.148942,
            "value": 61.92969696969697
          },
          {
            "t": 17.177345,
            "value": 61.557544947303164
          },
          {
            "t": 19.2076,
            "value": 61.901938650306754
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.308766,
            "value": 985009.1458099185
          },
          {
            "t": 1.327476,
            "value": 982615.2683295539
          },
          {
            "t": 2.345887,
            "value": 490960.9185289633
          },
          {
            "t": 2.447227,
            "value": 445634.78844850324
          },
          {
            "t": 3.460165,
            "value": 1032078.8931932271
          },
          {
            "t": 4.473887,
            "value": 1086096.5826922962
          },
          {
            "t": 5.487851,
            "value": 985242.079600459
          },
          {
            "t": 6.503236,
            "value": 984848.1117999577
          },
          {
            "t": 7.521974,
            "value": 981606.6545078323
          },
          {
            "t": 8.636636,
            "value": 897132.9425422233
          },
          {
            "t": 9.652077,
            "value": 984793.7989504066
          },
          {
            "t": 10.665674,
            "value": 986585.3983387875
          },
          {
            "t": 11.680588,
            "value": 985305.1588607508
          },
          {
            "t": 12.705598,
            "value": 975600.238046458
          },
          {
            "t": 13.723148,
            "value": 982752.6902854897
          },
          {
            "t": 14.742487,
            "value": 981027.9014145442
          },
          {
            "t": 15.857156,
            "value": 897127.3086449878
          },
          {
            "t": 16.87199,
            "value": 985382.8310837043
          },
          {
            "t": 17.885468,
            "value": 986701.2406781398
          },
          {
            "t": 18.902147,
            "value": 1082937.6823953283
          },
          {
            "t": 19.921203,
            "value": 980319.0403667707
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.205626,
            "value": 987595.3680691967
          },
          {
            "t": 1.220992,
            "value": 983881.6741943299
          },
          {
            "t": 2.138012,
            "value": 1090488.7570609148
          },
          {
            "t": 3.15638,
            "value": 981963.2981397688
          },
          {
            "t": 4.170223,
            "value": 987332.3581659093
          },
          {
            "t": 5.183841,
            "value": 1479847.4375948338
          },
          {
            "t": 6.199417,
            "value": 983678.2279218886
          },
          {
            "t": 7.213001,
            "value": 986598.052060806
          },
          {
            "t": 8.231643,
            "value": 980717.4650171503
          },
          {
            "t": 9.247135,
            "value": 984744.3406742741
          },
          {
            "t": 10.261104,
            "value": 987209.6681456732
          },
          {
            "t": 11.275957,
            "value": 984379.0184391236
          },
          {
            "t": 12.30091,
            "value": 975654.4934255522
          },
          {
            "t": 13.213155,
            "value": 1098389.1388826468
          },
          {
            "t": 14.230733,
            "value": 980760.1972526921
          },
          {
            "t": 15.250044,
            "value": 982035.9046453928
          },
          {
            "t": 16.264573,
            "value": 985679.06880927
          },
          {
            "t": 17.278574,
            "value": 986192.3213093479
          },
          {
            "t": 18.294567,
            "value": 984258.7498142213
          },
          {
            "t": 19.309069,
            "value": 982748.1858093921
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.119317,
            "value": 7929782.175407053
          },
          {
            "t": 3.05539,
            "value": 8308562.745309707
          },
          {
            "t": 5.08263,
            "value": 7947175.470097275
          },
          {
            "t": 7.111649,
            "value": 7944153.307583616
          },
          {
            "t": 9.145572,
            "value": 7905706.361548593
          },
          {
            "t": 11.174293,
            "value": 7930751.443890017
          },
          {
            "t": 13.11209,
            "value": 8307431.067340903
          },
          {
            "t": 15.148942,
            "value": 7899276.923409261
          },
          {
            "t": 17.177345,
            "value": 7943623.629032298
          },
          {
            "t": 19.2076,
            "value": 7941675.306796437
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.119317,
            "value": 7980967.209010277
          },
          {
            "t": 3.05539,
            "value": 8381475.285281083
          },
          {
            "t": 5.08263,
            "value": 8004654.604289576
          },
          {
            "t": 7.111649,
            "value": 8007867.3487039795
          },
          {
            "t": 9.145572,
            "value": 7966423.01601388
          },
          {
            "t": 11.174293,
            "value": 7992704.763247386
          },
          {
            "t": 13.11209,
            "value": 8366976.52024438
          },
          {
            "t": 15.148942,
            "value": 7959843.425050028
          },
          {
            "t": 17.177345,
            "value": 8000682.310172091
          },
          {
            "t": 19.2076,
            "value": 8018416.405820944
          }
        ],
        "ram_mib": [
          {
            "t": 1.119317,
            "value": 16.00390625
          },
          {
            "t": 3.05539,
            "value": 16.1015625
          },
          {
            "t": 5.08263,
            "value": 16.578125
          },
          {
            "t": 7.111649,
            "value": 16.2890625
          },
          {
            "t": 9.145572,
            "value": 16.50390625
          },
          {
            "t": 11.174293,
            "value": 16.46875
          },
          {
            "t": 13.11209,
            "value": 16.7265625
          },
          {
            "t": 15.148942,
            "value": 16.15234375
          },
          {
            "t": 17.177345,
            "value": 16.1953125
          },
          {
            "t": 19.2076,
            "value": 16.17578125
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
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 9.075634231696293
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 9.723568309419838
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 10.98515625
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 11.23828125
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 99410.6101886039
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/100k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 99410.6101886039
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000588
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 833028.0100596955
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 799620.0322772685
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.027899,
            "value": 8.906033728919425
          },
          {
            "t": 3.038852,
            "value": 9.178490330630067
          },
          {
            "t": 5.050036,
            "value": 9.143640897755612
          },
          {
            "t": 7.061546,
            "value": 9.242212554381604
          },
          {
            "t": 9.072751,
            "value": 9.723568309419838
          },
          {
            "t": 11.086192,
            "value": 9.197165941578621
          },
          {
            "t": 13.098231,
            "value": 9.217378277153559
          },
          {
            "t": 15.110166,
            "value": 9.401166977032899
          },
          {
            "t": 17.122504,
            "value": 8.29832709113608
          },
          {
            "t": 19.134981,
            "value": 8.448358208955224
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.122972,
            "value": 99484.86735682636
          },
          {
            "t": 1.128321,
            "value": 99467.9459570756
          },
          {
            "t": 2.133857,
            "value": 99449.4478566655
          },
          {
            "t": 3.139295,
            "value": 99459.14119020765
          },
          {
            "t": 4.14528,
            "value": 99405.06071164084
          },
          {
            "t": 5.150494,
            "value": 99481.30447844935
          },
          {
            "t": 6.156064,
            "value": 99446.08530485196
          },
          {
            "t": 7.162041,
            "value": 99405.85122721495
          },
          {
            "t": 8.167498,
            "value": 99457.26172277879
          },
          {
            "t": 9.17324,
            "value": 99429.07823278735
          },
          {
            "t": 10.179046,
            "value": 99422.75150476335
          },
          {
            "t": 11.186712,
            "value": 99239.23204712673
          },
          {
            "t": 12.192616,
            "value": 99413.0652626891
          },
          {
            "t": 13.198752,
            "value": 99390.14208814714
          },
          {
            "t": 14.205059,
            "value": 99373.25289399755
          },
          {
            "t": 15.210677,
            "value": 99441.3385599701
          },
          {
            "t": 16.216964,
            "value": 99375.2279419291
          },
          {
            "t": 17.222977,
            "value": 99402.29400614106
          },
          {
            "t": 18.229214,
            "value": 99380.16590524896
          },
          {
            "t": 19.23562,
            "value": 99363.47756273314
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.122972,
            "value": 99484.86735682636
          },
          {
            "t": 1.128321,
            "value": 99467.9459570756
          },
          {
            "t": 2.133857,
            "value": 99449.4478566655
          },
          {
            "t": 3.139295,
            "value": 99459.14119020765
          },
          {
            "t": 4.14528,
            "value": 99405.06071164084
          },
          {
            "t": 5.150494,
            "value": 99481.30447844935
          },
          {
            "t": 6.156064,
            "value": 99446.08530485196
          },
          {
            "t": 7.162041,
            "value": 99405.85122721495
          },
          {
            "t": 8.167498,
            "value": 99457.26172277879
          },
          {
            "t": 9.17324,
            "value": 99429.07823278735
          },
          {
            "t": 10.179046,
            "value": 99422.75150476335
          },
          {
            "t": 11.186712,
            "value": 99239.23204712673
          },
          {
            "t": 12.192616,
            "value": 99413.0652626891
          },
          {
            "t": 13.198752,
            "value": 99390.14208814714
          },
          {
            "t": 14.205059,
            "value": 99373.25289399755
          },
          {
            "t": 15.210677,
            "value": 99441.3385599701
          },
          {
            "t": 16.216964,
            "value": 99375.2279419291
          },
          {
            "t": 17.222977,
            "value": 99402.29400614106
          },
          {
            "t": 18.229214,
            "value": 99380.16590524896
          },
          {
            "t": 19.23562,
            "value": 99363.47756273314
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.027899,
            "value": 796980.3531459837
          },
          {
            "t": 3.038852,
            "value": 800736.7651058974
          },
          {
            "t": 5.050036,
            "value": 800663.1914335038
          },
          {
            "t": 7.061546,
            "value": 800488.6876028456
          },
          {
            "t": 9.072751,
            "value": 800639.4176625456
          },
          {
            "t": 11.086192,
            "value": 799648.9591698988
          },
          {
            "t": 13.098231,
            "value": 796380.6864578668
          },
          {
            "t": 15.110166,
            "value": 804254.6106111779
          },
          {
            "t": 17.122504,
            "value": 796269.3145982433
          },
          {
            "t": 19.134981,
            "value": 800138.3369847208
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.027899,
            "value": 833873.1658791346
          },
          {
            "t": 3.038852,
            "value": 833761.4056618926
          },
          {
            "t": 5.050036,
            "value": 833687.0221720141
          },
          {
            "t": 7.061546,
            "value": 833510.1490919758
          },
          {
            "t": 9.072751,
            "value": 833667.8757262437
          },
          {
            "t": 11.086192,
            "value": 828810.9758368883
          },
          {
            "t": 13.098231,
            "value": 833244.7830285595
          },
          {
            "t": 15.110166,
            "value": 833394.2199921967
          },
          {
            "t": 17.122504,
            "value": 833180.6088241637
          },
          {
            "t": 19.134981,
            "value": 833149.8943838861
          }
        ],
        "ram_mib": [
          {
            "t": 1.027899,
            "value": 11.15234375
          },
          {
            "t": 3.038852,
            "value": 10.90234375
          },
          {
            "t": 5.050036,
            "value": 10.7265625
          },
          {
            "t": 7.061546,
            "value": 10.875
          },
          {
            "t": 9.072751,
            "value": 11.06640625
          },
          {
            "t": 11.086192,
            "value": 11.0
          },
          {
            "t": 13.098231,
            "value": 10.921875
          },
          {
            "t": 15.110166,
            "value": 11.23828125
          },
          {
            "t": 17.122504,
            "value": 11.06640625
          },
          {
            "t": 19.134981,
            "value": 10.90234375
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
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.6315789222717285
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 15.414689295599773
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 16.161294337274423
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 11.5109375
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 11.66015625
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 198285.00171203446
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/200k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 202425.78759202588
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000576
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 1636882.793706017
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 1604452.4561780847
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.067452,
            "value": 16.161294337274423
          },
          {
            "t": 3.081174,
            "value": 15.151761786600495
          },
          {
            "t": 5.097072,
            "value": 14.829181141439207
          },
          {
            "t": 7.11492,
            "value": 15.902504672897196
          },
          {
            "t": 9.133635,
            "value": 14.683604723430701
          },
          {
            "t": 11.152259,
            "value": 15.231860174781522
          },
          {
            "t": 13.171387,
            "value": 15.258344741754822
          },
          {
            "t": 15.189707,
            "value": 15.906724673710379
          },
          {
            "t": 17.104904,
            "value": 15.381021170610213
          },
          {
            "t": 19.124323,
            "value": 15.640595533498761
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.060541,
            "value": 198665.16873129446
          },
          {
            "t": 1.067452,
            "value": 198627.2868207816
          },
          {
            "t": 2.074215,
            "value": 198656.48618393802
          },
          {
            "t": 3.081174,
            "value": 198617.8186003601
          },
          {
            "t": 4.088575,
            "value": 198530.67447818693
          },
          {
            "t": 5.097072,
            "value": 198314.91814055966
          },
          {
            "t": 6.106334,
            "value": 198164.59947961973
          },
          {
            "t": 7.11492,
            "value": 198297.4183659103
          },
          {
            "t": 8.124815,
            "value": 198040.39033760937
          },
          {
            "t": 9.133635,
            "value": 198251.4224539561
          },
          {
            "t": 10.14315,
            "value": 198114.93638034107
          },
          {
            "t": 11.152259,
            "value": 198194.6449788873
          },
          {
            "t": 12.161287,
            "value": 198210.55510848062
          },
          {
            "t": 13.171387,
            "value": 198000.198000198
          },
          {
            "t": 14.178408,
            "value": 198605.590151546
          },
          {
            "t": 15.189707,
            "value": 197765.4482007794
          },
          {
            "t": 16.197735,
            "value": 198407.1871019456
          },
          {
            "t": 17.20692,
            "value": 198179.71927842766
          },
          {
            "t": 18.217621,
            "value": 197882.45979770474
          },
          {
            "t": 19.224875,
            "value": 198559.6483111509
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.060541,
            "value": 198665.16873129446
          },
          {
            "t": 1.067452,
            "value": 198627.2868207816
          },
          {
            "t": 2.074215,
            "value": 198656.48618393802
          },
          {
            "t": 3.081174,
            "value": 198617.8186003601
          },
          {
            "t": 4.088575,
            "value": 198530.67447818693
          },
          {
            "t": 5.097072,
            "value": 198314.91814055966
          },
          {
            "t": 6.106334,
            "value": 198164.59947961973
          },
          {
            "t": 7.11492,
            "value": 198297.4183659103
          },
          {
            "t": 8.124815,
            "value": 297060.58550641406
          },
          {
            "t": 9.133635,
            "value": 198251.4224539561
          },
          {
            "t": 10.14315,
            "value": 198114.93638034107
          },
          {
            "t": 11.152259,
            "value": 198194.6449788873
          },
          {
            "t": 12.161287,
            "value": 198210.55510848062
          },
          {
            "t": 13.171387,
            "value": 198000.198000198
          },
          {
            "t": 14.178408,
            "value": 198605.590151546
          },
          {
            "t": 15.290989,
            "value": 179762.19259541552
          },
          {
            "t": 16.298976,
            "value": 198415.25733962838
          },
          {
            "t": 17.308065,
            "value": 198198.57316847178
          },
          {
            "t": 18.31879,
            "value": 197877.76101313412
          },
          {
            "t": 19.326861,
            "value": 198398.7238994079
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.067452,
            "value": 1599611.1500126636
          },
          {
            "t": 3.081174,
            "value": 1599383.6289219663
          },
          {
            "t": 5.097072,
            "value": 1597854.1573035938
          },
          {
            "t": 7.11492,
            "value": 1596254.524622271
          },
          {
            "t": 9.133635,
            "value": 1595675.466819239
          },
          {
            "t": 11.152259,
            "value": 1595626.03040487
          },
          {
            "t": 13.171387,
            "value": 1595330.2613801602
          },
          {
            "t": 15.189707,
            "value": 1591921.4990685321
          },
          {
            "t": 17.104904,
            "value": 1677760.0424394985
          },
          {
            "t": 19.124323,
            "value": 1595107.8008080542
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.067452,
            "value": 1631920.4620511215
          },
          {
            "t": 3.081174,
            "value": 1631735.661625587
          },
          {
            "t": 5.097072,
            "value": 1630142.4972890494
          },
          {
            "t": 7.11492,
            "value": 1628465.0776470776
          },
          {
            "t": 9.133635,
            "value": 1627905.375449234
          },
          {
            "t": 11.152259,
            "value": 1624047.3708823437
          },
          {
            "t": 13.171387,
            "value": 1631416.6313378846
          },
          {
            "t": 15.189707,
            "value": 1624202.802330651
          },
          {
            "t": 17.104904,
            "value": 1711740.3588247057
          },
          {
            "t": 19.124323,
            "value": 1627251.6996225151
          }
        ],
        "ram_mib": [
          {
            "t": 1.067452,
            "value": 11.46875
          },
          {
            "t": 3.081174,
            "value": 11.5234375
          },
          {
            "t": 5.097072,
            "value": 11.4609375
          },
          {
            "t": 7.11492,
            "value": 11.3515625
          },
          {
            "t": 9.133635,
            "value": 11.3828125
          },
          {
            "t": 11.152259,
            "value": 11.49609375
          },
          {
            "t": 13.171387,
            "value": 11.4609375
          },
          {
            "t": 15.189707,
            "value": 11.6484375
          },
          {
            "t": 17.104904,
            "value": 11.66015625
          },
          {
            "t": 19.124323,
            "value": 11.65625
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
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -1.803571343421936
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 20.90255826388491
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 21.5996
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 12.115234375
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 12.36328125
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 292205.7863424685
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/300k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 299063.9262388019
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000496
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 2398765.6700839726
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 2366689.8967522
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.035778,
            "value": 21.25677258566978
          },
          {
            "t": 3.051497,
            "value": 21.5996
          },
          {
            "t": 5.070056,
            "value": 20.85925187032419
          },
          {
            "t": 7.087383,
            "value": 20.477795867251096
          },
          {
            "t": 9.104128,
            "value": 20.424603867747972
          },
          {
            "t": 11.121365,
            "value": 20.81860696517413
          },
          {
            "t": 13.137601,
            "value": 20.79420362273579
          },
          {
            "t": 15.156265,
            "value": 21.43163138231631
          },
          {
            "t": 17.174014,
            "value": 20.787056627255758
          },
          {
            "t": 19.191642,
            "value": 20.576059850374065
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.128828,
            "value": 198382.58677006364
          },
          {
            "t": 1.13639,
            "value": 198498.9509330443
          },
          {
            "t": 2.144284,
            "value": 297650.34815169056
          },
          {
            "t": 3.152215,
            "value": 297639.4217461315
          },
          {
            "t": 4.160612,
            "value": 297501.8767410058
          },
          {
            "t": 5.171461,
            "value": 296780.2312709415
          },
          {
            "t": 6.18019,
            "value": 297403.9608259503
          },
          {
            "t": 7.188567,
            "value": 297507.7773491462
          },
          {
            "t": 8.196727,
            "value": 297571.8139977781
          },
          {
            "t": 9.205053,
            "value": 297522.8249593881
          },
          {
            "t": 10.214243,
            "value": 297268.106104896
          },
          {
            "t": 11.222348,
            "value": 297588.04886395764
          },
          {
            "t": 12.23042,
            "value": 297597.7906340023
          },
          {
            "t": 13.238641,
            "value": 297553.81012694637
          },
          {
            "t": 14.24719,
            "value": 297457.0397670317
          },
          {
            "t": 15.257317,
            "value": 296992.3583866187
          },
          {
            "t": 16.26664,
            "value": 297228.9346423296
          },
          {
            "t": 17.275136,
            "value": 297472.6721771826
          },
          {
            "t": 18.283199,
            "value": 297600.44759107317
          },
          {
            "t": 19.293404,
            "value": 296969.4269974906
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.128828,
            "value": 201358.3255716146
          },
          {
            "t": 1.13639,
            "value": 298740.9211542317
          },
          {
            "t": 2.144284,
            "value": 297650.34815169056
          },
          {
            "t": 3.152215,
            "value": 297639.42174613144
          },
          {
            "t": 4.160612,
            "value": 297501.8767410058
          },
          {
            "t": 5.070056,
            "value": 329871.877762677
          },
          {
            "t": 6.079233,
            "value": 297271.93544839014
          },
          {
            "t": 7.087383,
            "value": 297574.76565987204
          },
          {
            "t": 8.095811,
            "value": 297492.7312609328
          },
          {
            "t": 9.104128,
            "value": 297525.48057803256
          },
          {
            "t": 10.113235,
            "value": 297292.55668625823
          },
          {
            "t": 11.121365,
            "value": 297580.6691597314
          },
          {
            "t": 12.129429,
            "value": 297600.152371278
          },
          {
            "t": 13.137601,
            "value": 297568.27208055765
          },
          {
            "t": 14.146196,
            "value": 297443.4733465861
          },
          {
            "t": 15.156265,
            "value": 297009.41222827346
          },
          {
            "t": 16.1654,
            "value": 297284.30784781027
          },
          {
            "t": 17.174014,
            "value": 297437.87018621597
          },
          {
            "t": 18.182242,
            "value": 297551.74424832483
          },
          {
            "t": 19.191642,
            "value": 297206.26114523475
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.035778,
            "value": 2149680.2568858494
          },
          {
            "t": 3.051497,
            "value": 2390993.4866913497
          },
          {
            "t": 5.070056,
            "value": 2391472.827893561
          },
          {
            "t": 7.087383,
            "value": 2389054.9226773847
          },
          {
            "t": 9.104128,
            "value": 2389775.60375754
          },
          {
            "t": 11.121365,
            "value": 2393076.7678760597
          },
          {
            "t": 13.137601,
            "value": 2394301.06396275
          },
          {
            "t": 15.156265,
            "value": 2387401.7667130344
          },
          {
            "t": 17.174014,
            "value": 2392412.534958511
          },
          {
            "t": 19.191642,
            "value": 2388729.7361059617
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.035778,
            "value": 2178012.9048132366
          },
          {
            "t": 3.051497,
            "value": 2426825.366035643
          },
          {
            "t": 5.070056,
            "value": 2423546.203009176
          },
          {
            "t": 7.087383,
            "value": 2421122.6043174956
          },
          {
            "t": 9.104128,
            "value": 2421838.655853864
          },
          {
            "t": 11.121365,
            "value": 2425137.4528625044
          },
          {
            "t": 13.137601,
            "value": 2426398.9929750287
          },
          {
            "t": 15.156265,
            "value": 2419511.6175846998
          },
          {
            "t": 17.174014,
            "value": 2420650.933292496
          },
          {
            "t": 19.191642,
            "value": 2424611.970095577
          }
        ],
        "ram_mib": [
          {
            "t": 1.035778,
            "value": 11.94921875
          },
          {
            "t": 3.051497,
            "value": 12.04296875
          },
          {
            "t": 5.070056,
            "value": 11.953125
          },
          {
            "t": 7.087383,
            "value": 12.16796875
          },
          {
            "t": 9.104128,
            "value": 12.1796875
          },
          {
            "t": 11.121365,
            "value": 12.1796875
          },
          {
            "t": 13.137601,
            "value": 12.36328125
          },
          {
            "t": 15.156265,
            "value": 12.18359375
          },
          {
            "t": 17.174014,
            "value": 12.04296875
          },
          {
            "t": 19.191642,
            "value": 12.08984375
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
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -1.298701286315918
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 28.324554884282126
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 30.805643564356433
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 13.16328125
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 13.59375
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 399257.3398665662
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/400k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 408709.66585784074
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000631
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 3269748.9379063295
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 3203498.0957959043
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.126005,
            "value": 28.639210363972857
          },
          {
            "t": 2.140141,
            "value": 30.805643564356433
          },
          {
            "t": 4.153871,
            "value": 28.109140383426094
          },
          {
            "t": 6.166806,
            "value": 28.46314232902033
          },
          {
            "t": 8.129759,
            "value": 27.382024691358026
          },
          {
            "t": 10.142951,
            "value": 28.207157894736838
          },
          {
            "t": 12.155879,
            "value": 27.829458128078816
          },
          {
            "t": 14.170962,
            "value": 27.852118226600986
          },
          {
            "t": 16.18882,
            "value": 27.798540507111934
          },
          {
            "t": 18.203895,
            "value": 28.159112754158965
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.226967,
            "value": 397461.4139491071
          },
          {
            "t": 1.233699,
            "value": 397325.2067084388
          },
          {
            "t": 2.241111,
            "value": 397057.01341655647
          },
          {
            "t": 3.247938,
            "value": 397287.71675769525
          },
          {
            "t": 4.254902,
            "value": 397233.66475862096
          },
          {
            "t": 5.261213,
            "value": 496864.28946916014
          },
          {
            "t": 6.267898,
            "value": 397343.7569845582
          },
          {
            "t": 7.32463,
            "value": 378525.49179924524
          },
          {
            "t": 8.33124,
            "value": 397373.3620766732
          },
          {
            "t": 9.337778,
            "value": 397401.7871158367
          },
          {
            "t": 10.344566,
            "value": 397303.1065129898
          },
          {
            "t": 11.350761,
            "value": 397537.2566947759
          },
          {
            "t": 12.358326,
            "value": 396996.7198146025
          },
          {
            "t": 13.365287,
            "value": 397234.8482215299
          },
          {
            "t": 14.372491,
            "value": 397139.01056786906
          },
          {
            "t": 15.383843,
            "value": 395510.16856643383
          },
          {
            "t": 16.491375,
            "value": 361163.3794779745
          },
          {
            "t": 17.499379,
            "value": 396823.82212769
          },
          {
            "t": 18.506106,
            "value": 397327.18005973817
          },
          {
            "t": 19.512774,
            "value": 397350.4670854741
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.126005,
            "value": 397408.89401104796
          },
          {
            "t": 1.132692,
            "value": 397342.96757582045
          },
          {
            "t": 2.140141,
            "value": 397042.4309319876
          },
          {
            "t": 3.146596,
            "value": 397434.5599157438
          },
          {
            "t": 4.153871,
            "value": 397111.0173487876
          },
          {
            "t": 5.160251,
            "value": 397464.17854090896
          },
          {
            "t": 6.166806,
            "value": 397395.0752815295
          },
          {
            "t": 7.173047,
            "value": 398512.8811089988
          },
          {
            "t": 8.129759,
            "value": 417053.4079221333
          },
          {
            "t": 9.136233,
            "value": 398420.624874562
          },
          {
            "t": 10.142951,
            "value": 396337.4053111199
          },
          {
            "t": 11.149188,
            "value": 397520.6636209958
          },
          {
            "t": 12.155879,
            "value": 397341.38876775495
          },
          {
            "t": 13.163435,
            "value": 397000.2659901783
          },
          {
            "t": 14.170962,
            "value": 397011.69298688765
          },
          {
            "t": 15.177129,
            "value": 398542.19031234377
          },
          {
            "t": 16.18882,
            "value": 394389.1959106091
          },
          {
            "t": 17.19695,
            "value": 595161.3383194628
          },
          {
            "t": 18.203895,
            "value": 397241.1601428082
          },
          {
            "t": 19.210457,
            "value": 397392.3116509465
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.126005,
            "value": 3198900.245962329
          },
          {
            "t": 2.140141,
            "value": 3198177.7794548133
          },
          {
            "t": 4.153871,
            "value": 3187557.914914115
          },
          {
            "t": 6.166806,
            "value": 3204223.1865410456
          },
          {
            "t": 8.129759,
            "value": 3273855.2578691393
          },
          {
            "t": 10.142951,
            "value": 3200402.644159126
          },
          {
            "t": 12.155879,
            "value": 3200168.1133155283
          },
          {
            "t": 14.170962,
            "value": 3196915.958300477
          },
          {
            "t": 16.18882,
            "value": 3185224.1337101026
          },
          {
            "t": 18.203895,
            "value": 3189555.723732367
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.126005,
            "value": 3261237.022172364
          },
          {
            "t": 2.140141,
            "value": 3260369.210420746
          },
          {
            "t": 4.153871,
            "value": 3261139.775441594
          },
          {
            "t": 6.166806,
            "value": 3266539.1579956627
          },
          {
            "t": 8.129759,
            "value": 3346160.6059849625
          },
          {
            "t": 10.142951,
            "value": 3266270.181880317
          },
          {
            "t": 12.155879,
            "value": 3266345.343698334
          },
          {
            "t": 14.170962,
            "value": 3262927.6312687863
          },
          {
            "t": 16.18882,
            "value": 3246714.08989136
          },
          {
            "t": 18.203895,
            "value": 3259786.3603091696
          }
        ],
        "ram_mib": [
          {
            "t": 0.126005,
            "value": 12.97265625
          },
          {
            "t": 2.140141,
            "value": 12.9921875
          },
          {
            "t": 4.153871,
            "value": 12.96875
          },
          {
            "t": 6.166806,
            "value": 13.01171875
          },
          {
            "t": 8.129759,
            "value": 13.59375
          },
          {
            "t": 10.142951,
            "value": 13.57421875
          },
          {
            "t": 12.155879,
            "value": 13.359375
          },
          {
            "t": 14.170962,
            "value": 12.93359375
          },
          {
            "t": 16.18882,
            "value": 13.203125
          },
          {
            "t": 18.203895,
            "value": 13.0234375
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
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/600k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.622807025909424
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/600k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 39.17048698593834
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/600k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 40.39450834879406
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/600k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 14.056640625
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/600k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 14.265625
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/600k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 587227.9164442669
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/600k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 612081.29929703
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/600k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.00065
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/600k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 4862555.710759905
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/600k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 4797929.316726017
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.08462,
            "value": 38.99825230769231
          },
          {
            "t": 2.105069,
            "value": 38.39181257706535
          },
          {
            "t": 4.070777,
            "value": 38.591704615384614
          },
          {
            "t": 6.090283,
            "value": 40.39450834879406
          },
          {
            "t": 8.104406,
            "value": 39.219926017262644
          },
          {
            "t": 10.118649,
            "value": 39.77110285006196
          },
          {
            "t": 12.14018,
            "value": 39.40661728395062
          },
          {
            "t": 14.160328,
            "value": 38.845096333126165
          },
          {
            "t": 16.174914,
            "value": 38.86983970406905
          },
          {
            "t": 18.190641,
            "value": 39.21600982197667
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.190695,
            "value": 592743.6324515283
          },
          {
            "t": 1.2992,
            "value": 541269.547724187
          },
          {
            "t": 2.308373,
            "value": 594546.2274555502
          },
          {
            "t": 3.264191,
            "value": 627734.5687149647
          },
          {
            "t": 4.27752,
            "value": 593094.6415231382
          },
          {
            "t": 5.38534,
            "value": 540701.5580148399
          },
          {
            "t": 6.392531,
            "value": 595716.2047714883
          },
          {
            "t": 7.399446,
            "value": 595879.4933038042
          },
          {
            "t": 8.406631,
            "value": 595719.7535705952
          },
          {
            "t": 9.413777,
            "value": 595742.8217954497
          },
          {
            "t": 10.42255,
            "value": 594781.9777095541
          },
          {
            "t": 11.434684,
            "value": 592806.8813022781
          },
          {
            "t": 12.447757,
            "value": 592257.4187644919
          },
          {
            "t": 13.555787,
            "value": 541501.5838921329
          },
          {
            "t": 14.562982,
            "value": 595713.8389289065
          },
          {
            "t": 15.570268,
            "value": 595660.0210863649
          },
          {
            "t": 16.577655,
            "value": 595600.3005796183
          },
          {
            "t": 17.585043,
            "value": 595599.7093473419
          },
          {
            "t": 18.593517,
            "value": 594958.3231694619
          },
          {
            "t": 19.603941,
            "value": 593810.1232749816
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.08462,
            "value": 594869.8188477402
          },
          {
            "t": 1.097562,
            "value": 593321.236556486
          },
          {
            "t": 2.105069,
            "value": 594536.8121511812
          },
          {
            "t": 3.162619,
            "value": 568294.6432792776
          },
          {
            "t": 4.17149,
            "value": 594724.2016075394
          },
          {
            "t": 5.183562,
            "value": 590867.0529369451
          },
          {
            "t": 6.190857,
            "value": 597640.2146342432
          },
          {
            "t": 7.197727,
            "value": 595906.1249217874
          },
          {
            "t": 8.204939,
            "value": 595703.7843075738
          },
          {
            "t": 9.212213,
            "value": 594674.3388591387
          },
          {
            "t": 10.219498,
            "value": 596653.3801257836
          },
          {
            "t": 11.228116,
            "value": 593881.9255654768
          },
          {
            "t": 12.14018,
            "value": 657848.572030033
          },
          {
            "t": 13.152674,
            "value": 593583.7644469992
          },
          {
            "t": 14.160328,
            "value": 593457.6749558876
          },
          {
            "t": 15.167639,
            "value": 595645.2376674135
          },
          {
            "t": 16.174914,
            "value": 895485.3441215161
          },
          {
            "t": 17.181978,
            "value": 595791.3300445653
          },
          {
            "t": 18.190641,
            "value": 592864.0190033737
          },
          {
            "t": 19.198094,
            "value": 595561.2817669907
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.08462,
            "value": 4794048.811445717
          },
          {
            "t": 2.105069,
            "value": 4774844.106433768
          },
          {
            "t": 4.070777,
            "value": 4906463.7270642435
          },
          {
            "t": 6.090283,
            "value": 4779245.518458475
          },
          {
            "t": 8.104406,
            "value": 4796643.501911254
          },
          {
            "t": 10.118649,
            "value": 4792744.470255079
          },
          {
            "t": 12.14018,
            "value": 4778610.864735688
          },
          {
            "t": 14.160328,
            "value": 4774707.5956811085
          },
          {
            "t": 16.174914,
            "value": 4803693.165742242
          },
          {
            "t": 18.190641,
            "value": 4778291.405532595
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.08462,
            "value": 4857885.466361498
          },
          {
            "t": 2.105069,
            "value": 4838729.411135841
          },
          {
            "t": 4.070777,
            "value": 4972692.790587412
          },
          {
            "t": 6.090283,
            "value": 4843657.805423703
          },
          {
            "t": 8.104406,
            "value": 4861444.41029669
          },
          {
            "t": 10.118649,
            "value": 4857541.5180790005
          },
          {
            "t": 12.14018,
            "value": 4839091.757682667
          },
          {
            "t": 14.160328,
            "value": 4835242.76439152
          },
          {
            "t": 16.174914,
            "value": 4860951.083746239
          },
          {
            "t": 18.190641,
            "value": 4858320.09989448
          }
        ],
        "ram_mib": [
          {
            "t": 0.08462,
            "value": 13.96875
          },
          {
            "t": 2.105069,
            "value": 14.1875
          },
          {
            "t": 4.070777,
            "value": 13.984375
          },
          {
            "t": 6.090283,
            "value": 14.09765625
          },
          {
            "t": 8.104406,
            "value": 14.0234375
          },
          {
            "t": 10.118649,
            "value": 13.953125
          },
          {
            "t": 12.14018,
            "value": 13.90234375
          },
          {
            "t": 14.160328,
            "value": 14.1640625
          },
          {
            "t": 16.174914,
            "value": 14.265625
          },
          {
            "t": 18.190641,
            "value": 14.01953125
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
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/800k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 1.2658227682113647
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/800k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 49.922530333066526
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/800k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 50.536998738965956
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/800k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 15.073828125
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/800k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 15.25390625
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/800k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 809379.4579903374
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/800k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 815993.4293698318
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/800k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000597
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/800k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 6451113.897562741
        },
        {
          "extra": "DFE OTAP Baseline w/ Gzip (Logs)/800k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 6389955.31285055
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.064847,
            "value": 49.68465290806755
          },
          {
            "t": 2.088763,
            "value": 50.536998738965956
          },
          {
            "t": 4.119328,
            "value": 49.70008810572687
          },
          {
            "t": 6.140001,
            "value": 49.68424734540912
          },
          {
            "t": 8.161439,
            "value": 50.53440401505647
          },
          {
            "t": 10.186398,
            "value": 50.26262727844123
          },
          {
            "t": 12.105011,
            "value": 50.37433962264151
          },
          {
            "t": 14.124752,
            "value": 49.37531701192718
          },
          {
            "t": 16.146537,
            "value": 49.57393483709273
          },
          {
            "t": 18.172572,
            "value": 49.498693467336686
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.266896,
            "value": 792112.9315406496
          },
          {
            "t": 1.281088,
            "value": 788805.2755296828
          },
          {
            "t": 2.296228,
            "value": 788068.6407786119
          },
          {
            "t": 3.311576,
            "value": 393953.60014497495
          },
          {
            "t": 3.412556,
            "value": 358317.62707734643
          },
          {
            "t": 4.422345,
            "value": 756233.2363746143
          },
          {
            "t": 5.43268,
            "value": 791816.5756902414
          },
          {
            "t": 6.442953,
            "value": 791865.1691176544
          },
          {
            "t": 7.453121,
            "value": 791947.4780432562
          },
          {
            "t": 8.464429,
            "value": 791054.7528547187
          },
          {
            "t": 9.479514,
            "value": 985139.1755370239
          },
          {
            "t": 10.589866,
            "value": 720492.240298572
          },
          {
            "t": 11.599594,
            "value": 891329.1500285226
          },
          {
            "t": 12.60942,
            "value": 891242.649723814
          },
          {
            "t": 13.619322,
            "value": 792156.0705890274
          },
          {
            "t": 14.63063,
            "value": 889936.5969615586
          },
          {
            "t": 15.641252,
            "value": 791591.7128263584
          },
          {
            "t": 16.656638,
            "value": 590908.2851250658
          },
          {
            "t": 16.758067,
            "value": 179080.68928157305
          },
          {
            "t": 17.767614,
            "value": 828185.2739715814
          },
          {
            "t": 18.77777,
            "value": 791956.8858671335
          },
          {
            "t": 19.788024,
            "value": 791880.0618458327
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.165585,
            "value": 792164.6989625612
          },
          {
            "t": 1.077531,
            "value": 1316963.943040487
          },
          {
            "t": 2.088763,
            "value": 790125.3124901111
          },
          {
            "t": 3.104355,
            "value": 787717.9024647693
          },
          {
            "t": 4.119328,
            "value": 788198.3067529876
          },
          {
            "t": 5.129853,
            "value": 791667.6974839811
          },
          {
            "t": 6.140001,
            "value": 791963.1578738957
          },
          {
            "t": 7.15017,
            "value": 791946.6940680222
          },
          {
            "t": 8.161439,
            "value": 791085.2602027749
          },
          {
            "t": 9.171481,
            "value": 792046.2713431718
          },
          {
            "t": 10.186398,
            "value": 788241.7971124732
          },
          {
            "t": 11.195984,
            "value": 792404.0151111445
          },
          {
            "t": 12.205774,
            "value": 792243.9319066341
          },
          {
            "t": 13.215814,
            "value": 792047.8396895172
          },
          {
            "t": 14.225551,
            "value": 792285.5159313761
          },
          {
            "t": 15.237424,
            "value": 790613.0512425966
          },
          {
            "t": 16.24735,
            "value": 792137.2456991897
          },
          {
            "t": 17.263212,
            "value": 787508.5395457257
          },
          {
            "t": 18.273428,
            "value": 791909.8489827918
          },
          {
            "t": 19.283386,
            "value": 792112.147237806
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.064847,
            "value": 6364639.948846219
          },
          {
            "t": 2.088763,
            "value": 6347081.598248149
          },
          {
            "t": 4.119328,
            "value": 6333193.470782761
          },
          {
            "t": 6.140001,
            "value": 6367122.240956355
          },
          {
            "t": 8.161439,
            "value": 6353695.735412118
          },
          {
            "t": 10.186398,
            "value": 6355558.310069488
          },
          {
            "t": 12.105011,
            "value": 6705884.92833104
          },
          {
            "t": 14.124752,
            "value": 6364962.14118543
          },
          {
            "t": 16.146537,
            "value": 6355541.761364339
          },
          {
            "t": 18.172572,
            "value": 6351872.993309593
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.064847,
            "value": 6417757.033397441
          },
          {
            "t": 2.088763,
            "value": 6418459.560574649
          },
          {
            "t": 4.119328,
            "value": 6397599.682846892
          },
          {
            "t": 6.140001,
            "value": 6429201.558094754
          },
          {
            "t": 8.161439,
            "value": 6413233.054884692
          },
          {
            "t": 10.186398,
            "value": 6415371.372951255
          },
          {
            "t": 12.105011,
            "value": 6761384.917125028
          },
          {
            "t": 14.124752,
            "value": 6425110.942442621
          },
          {
            "t": 16.146537,
            "value": 6421036.361433091
          },
          {
            "t": 18.172572,
            "value": 6411984.4918769915
          }
        ],
        "ram_mib": [
          {
            "t": 0.064847,
            "value": 15.22265625
          },
          {
            "t": 2.088763,
            "value": 15.0625
          },
          {
            "t": 4.119328,
            "value": 14.76953125
          },
          {
            "t": 6.140001,
            "value": 15.10546875
          },
          {
            "t": 8.161439,
            "value": 15.046875
          },
          {
            "t": 10.186398,
            "value": 15.25390625
          },
          {
            "t": 12.105011,
            "value": 15.0546875
          },
          {
            "t": 14.124752,
            "value": 15.01953125
          },
          {
            "t": 16.146537,
            "value": 15.13671875
          },
          {
            "t": 18.172572,
            "value": 15.06640625
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
