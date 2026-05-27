window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otap_none_transform_rename_multi_transform"] = {
  "name": "DFE OTAP Transform Rename Multi Transform (Logs)",
  "slug": "dfe_logs_otap_none_transform_rename_multi_transform",
  "description": "Dataflow Engine OTAP logs, transform processor (OPL) rename sweep over 1-4 rename actions at 400k signals/sec",
  "meta": {
    "binary": "dfe",
    "protocols": [
      "otap"
    ],
    "signals": [
      "logs"
    ],
    "compression": "none"
  },
  "env": {
    "started_at": "2026-05-27T18:08:10Z",
    "ended_at": "2026-05-27T18:13:58Z",
    "cpu": {
      "model": "INTEL(R) XEON(R) PLATINUM 8581C CPU @ 2.30GHz",
      "architecture": "x86_64",
      "physical_cores": 16,
      "logical_cores": 32,
      "max_freq_mhz": null
    },
    "os": {
      "system": "Linux",
      "release": "6.1.0-45-cloud-amd64",
      "version": "#1 SMP PREEMPT_DYNAMIC Debian 6.1.170-1 (2026-04-30)",
      "distro": {
        "NAME": "Debian GNU/Linux",
        "VERSION": "12 (bookworm)",
        "VERSION_ID": "12",
        "ID": "debian"
      }
    },
    "memory": {
      "total_bytes": 126619074560,
      "total_gib_rounded": 118
    }
  },
  "tests": [
    {
      "name": "transform-1",
      "metrics": [
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-1 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -5.555555820465088
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 30.680194920535854
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 30.9862656641604
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 54.402734375
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 54.76953125
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 385899.24170799
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 397892.57246081845
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-1 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.005741
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55498587.31777725
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55444040.525543615
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-1 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 139.4813353125418
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.110837,
            "value": 30.867746390458255
          },
          {
            "t": 2.068927,
            "value": 30.648275862068964
          },
          {
            "t": 4.08376,
            "value": 30.565766217486683
          },
          {
            "t": 6.101593,
            "value": 30.9862656641604
          },
          {
            "t": 8.122061,
            "value": 30.6916
          },
          {
            "t": 10.138953,
            "value": 30.684600438733938
          },
          {
            "t": 12.057987,
            "value": 30.481376290272127
          },
          {
            "t": 14.073203,
            "value": 30.400801001251565
          },
          {
            "t": 16.089186,
            "value": 30.8034521575985
          },
          {
            "t": 18.102772,
            "value": 30.67206518332811
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.350601,
            "value": 382400.0382400038
          },
          {
            "t": 1.363112,
            "value": 395057.43641303654
          },
          {
            "t": 2.471019,
            "value": 361041.1343190358
          },
          {
            "t": 3.478568,
            "value": 397003.0241705366
          },
          {
            "t": 4.487909,
            "value": 396298.17871264514
          },
          {
            "t": 5.596658,
            "value": 360766.9544685046
          },
          {
            "t": 6.604049,
            "value": 397065.29043837
          },
          {
            "t": 7.611994,
            "value": 396847.05018627003
          },
          {
            "t": 8.626123,
            "value": 394427.13895372284
          },
          {
            "t": 9.73449,
            "value": 360891.2932268824
          },
          {
            "t": 10.741608,
            "value": 397172.92313313833
          },
          {
            "t": 11.754173,
            "value": 395036.3680356323
          },
          {
            "t": 12.762788,
            "value": 396583.433718515
          },
          {
            "t": 13.869581,
            "value": 361404.52641099103
          },
          {
            "t": 14.876238,
            "value": 398348.19605883636
          },
          {
            "t": 15.884653,
            "value": 395670.4333037489
          },
          {
            "t": 16.99247,
            "value": 361070.46560939215
          },
          {
            "t": 17.999003,
            "value": 397403.7612278982
          },
          {
            "t": 19.008321,
            "value": 396307.2094226002
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.110837,
            "value": 396050.58358053485
          },
          {
            "t": 1.156851,
            "value": 382404.05960149673
          },
          {
            "t": 2.169355,
            "value": 395060.1676635351
          },
          {
            "t": 3.176984,
            "value": 397963.9331539683
          },
          {
            "t": 4.184233,
            "value": 396128.4647589623
          },
          {
            "t": 5.194625,
            "value": 395885.95317460946
          },
          {
            "t": 6.202027,
            "value": 397060.9548124781
          },
          {
            "t": 7.210149,
            "value": 397769.31760243303
          },
          {
            "t": 8.122061,
            "value": 437542.21898604254
          },
          {
            "t": 9.131631,
            "value": 396208.2866963162
          },
          {
            "t": 10.138953,
            "value": 397092.48879702814
          },
          {
            "t": 11.145872,
            "value": 397251.4174427139
          },
          {
            "t": 12.158383,
            "value": 396045.08000406914
          },
          {
            "t": 13.16703,
            "value": 395579.42471449374
          },
          {
            "t": 14.173581,
            "value": 397396.654516264
          },
          {
            "t": 15.180021,
            "value": 398434.08449584676
          },
          {
            "t": 16.18958,
            "value": 395222.0722117281
          },
          {
            "t": 17.196009,
            "value": 397444.8272058933
          },
          {
            "t": 18.20317,
            "value": 397155.9661265677
          },
          {
            "t": 19.21147,
            "value": 396707.32916790637
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.110837,
            "value": 55046807.4515932
          },
          {
            "t": 2.068927,
            "value": 56587794.73875052
          },
          {
            "t": 4.08376,
            "value": 55152187.302868284
          },
          {
            "t": 6.101593,
            "value": 54889893.76226873
          },
          {
            "t": 8.122061,
            "value": 54953605.7982606
          },
          {
            "t": 10.138953,
            "value": 54913186.724921316
          },
          {
            "t": 12.057987,
            "value": 57857809.710510604
          },
          {
            "t": 14.073203,
            "value": 54959720.446840435
          },
          {
            "t": 16.089186,
            "value": 55076196.0790344
          },
          {
            "t": 18.102772,
            "value": 55003203.24038804
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.110837,
            "value": 55030993.93576431
          },
          {
            "t": 2.068927,
            "value": 56688618.50068178
          },
          {
            "t": 4.08376,
            "value": 55162492.375298604
          },
          {
            "t": 6.101593,
            "value": 55011602.54589949
          },
          {
            "t": 8.122061,
            "value": 55007330.479869016
          },
          {
            "t": 10.138953,
            "value": 54968249.167531036
          },
          {
            "t": 12.057987,
            "value": 57914963.46599383
          },
          {
            "t": 14.073203,
            "value": 55015585.426078394
          },
          {
            "t": 16.089186,
            "value": 55128884.51936351
          },
          {
            "t": 18.102772,
            "value": 55057152.76129254
          }
        ],
        "ram_mib": [
          {
            "t": 0.110837,
            "value": 54.16796875
          },
          {
            "t": 2.068927,
            "value": 54.12890625
          },
          {
            "t": 4.08376,
            "value": 54.4375
          },
          {
            "t": 6.101593,
            "value": 54.37109375
          },
          {
            "t": 8.122061,
            "value": 54.48046875
          },
          {
            "t": 10.138953,
            "value": 54.46484375
          },
          {
            "t": 12.057987,
            "value": 54.4609375
          },
          {
            "t": 14.073203,
            "value": 54.76953125
          },
          {
            "t": 16.089186,
            "value": 54.36328125
          },
          {
            "t": 18.102772,
            "value": 54.3828125
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
      "name": "transform-2",
      "metrics": [
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-2 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -5.555555820465088
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 29.530737215470783
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 29.979768677711782
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 58.55390625
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 58.7421875
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 388460.4164500685
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 397035.58603836905
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-2 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000589
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55309650.10040814
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55320006.917651296
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-2 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 139.30653081324323
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.095654,
            "value": 29.922901813633523
          },
          {
            "t": 2.111146,
            "value": 29.524552845528458
          },
          {
            "t": 4.092452,
            "value": 29.15785915492958
          },
          {
            "t": 6.115461,
            "value": 29.59840099937539
          },
          {
            "t": 8.137003,
            "value": 29.505779443923775
          },
          {
            "t": 10.058551,
            "value": 29.323836301155886
          },
          {
            "t": 12.076672,
            "value": 29.302428972837962
          },
          {
            "t": 14.093694,
            "value": 29.561437949359178
          },
          {
            "t": 16.110494,
            "value": 29.43040599625234
          },
          {
            "t": 18.127022,
            "value": 29.979768677711782
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.397466,
            "value": 360811.2118475969
          },
          {
            "t": 1.405583,
            "value": 396779.34208033397
          },
          {
            "t": 2.413186,
            "value": 396981.7477716918
          },
          {
            "t": 3.486981,
            "value": 372510.5816287094
          },
          {
            "t": 4.494956,
            "value": 396835.23896922043
          },
          {
            "t": 5.505133,
            "value": 395970.21116101433
          },
          {
            "t": 6.52295,
            "value": 392997.9554281369
          },
          {
            "t": 7.631142,
            "value": 360948.28332996444
          },
          {
            "t": 8.639783,
            "value": 396573.21088474494
          },
          {
            "t": 9.653142,
            "value": 394726.84408980433
          },
          {
            "t": 10.66314,
            "value": 396040.3881987885
          },
          {
            "t": 11.77198,
            "value": 360737.34713754914
          },
          {
            "t": 12.780405,
            "value": 396658.1550437564
          },
          {
            "t": 13.78859,
            "value": 397744.4615819517
          },
          {
            "t": 14.797954,
            "value": 395298.4255432134
          },
          {
            "t": 15.906873,
            "value": 360711.64801035967
          },
          {
            "t": 16.915197,
            "value": 396697.88679035706
          },
          {
            "t": 17.92313,
            "value": 396851.77486995666
          },
          {
            "t": 18.932173,
            "value": 396415.2171909423
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.095654,
            "value": 394729.5707710647
          },
          {
            "t": 1.103654,
            "value": 396825.3968253968
          },
          {
            "t": 2.111146,
            "value": 397025.48506588634
          },
          {
            "t": 3.185163,
            "value": 372433.5834535208
          },
          {
            "t": 4.193055,
            "value": 396867.91838808125
          },
          {
            "t": 5.201396,
            "value": 396691.198711547
          },
          {
            "t": 6.115461,
            "value": 437605.6407367091
          },
          {
            "t": 7.128503,
            "value": 394850.3615842186
          },
          {
            "t": 8.137003,
            "value": 396628.6564204264
          },
          {
            "t": 9.145469,
            "value": 396642.02858599095
          },
          {
            "t": 10.159155,
            "value": 394599.5110912057
          },
          {
            "t": 11.16893,
            "value": 396127.8502636726
          },
          {
            "t": 12.177169,
            "value": 396731.3305674547
          },
          {
            "t": 13.185434,
            "value": 396721.10010761063
          },
          {
            "t": 14.194288,
            "value": 396489.48212526296
          },
          {
            "t": 15.203256,
            "value": 396444.68407323124
          },
          {
            "t": 16.211028,
            "value": 396915.17525789567
          },
          {
            "t": 17.219619,
            "value": 396592.8706482608
          },
          {
            "t": 18.227557,
            "value": 396849.8062380821
          },
          {
            "t": 19.237515,
            "value": 396056.073618903
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.095654,
            "value": 54854196.54304221
          },
          {
            "t": 2.111146,
            "value": 55082279.66174016
          },
          {
            "t": 4.092452,
            "value": 55894166.27214575
          },
          {
            "t": 6.115461,
            "value": 54877932.32753784
          },
          {
            "t": 8.137003,
            "value": 54779391.17762579
          },
          {
            "t": 10.058551,
            "value": 57844508.177781664
          },
          {
            "t": 12.076672,
            "value": 54802534.63494013
          },
          {
            "t": 14.093694,
            "value": 55107258.12608886
          },
          {
            "t": 16.110494,
            "value": 54974018.74256248
          },
          {
            "t": 18.127022,
            "value": 54983783.513048165
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.095654,
            "value": 54843194.7990679
          },
          {
            "t": 2.111146,
            "value": 54940768.30868096
          },
          {
            "t": 4.092452,
            "value": 56013551.16271792
          },
          {
            "t": 6.115461,
            "value": 54866915.56982692
          },
          {
            "t": 8.137003,
            "value": 54771611.96749807
          },
          {
            "t": 10.058551,
            "value": 57763269.50979106
          },
          {
            "t": 12.076672,
            "value": 54863042.89980631
          },
          {
            "t": 14.093694,
            "value": 55087320.31678385
          },
          {
            "t": 16.110494,
            "value": 54907765.76755256
          },
          {
            "t": 18.127022,
            "value": 55039060.70235573
          }
        ],
        "ram_mib": [
          {
            "t": 0.095654,
            "value": 58.7421875
          },
          {
            "t": 2.111146,
            "value": 58.6328125
          },
          {
            "t": 4.092452,
            "value": 58.34375
          },
          {
            "t": 6.115461,
            "value": 58.359375
          },
          {
            "t": 8.137003,
            "value": 58.4375
          },
          {
            "t": 10.058551,
            "value": 58.578125
          },
          {
            "t": 12.076672,
            "value": 58.453125
          },
          {
            "t": 14.093694,
            "value": 58.59765625
          },
          {
            "t": 16.110494,
            "value": 58.66796875
          },
          {
            "t": 18.127022,
            "value": 58.7265625
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
      "name": "transform-3",
      "metrics": [
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-3 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 29.741012672749324
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 30.075499999999998
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 52.9796875
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 53.28125
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 386929.02830665855
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 397126.4349197037
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-3 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000836
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55328264.34421456
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55346457.01320167
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-3 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 139.32153460245365
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.050702,
            "value": 29.402718446601945
          },
          {
            "t": 2.075702,
            "value": 29.92030037546934
          },
          {
            "t": 4.099286,
            "value": 29.688443470090824
          },
          {
            "t": 6.124196,
            "value": 29.74108818011257
          },
          {
            "t": 8.04489,
            "value": 29.89544232572679
          },
          {
            "t": 10.068772,
            "value": 30.075499999999998
          },
          {
            "t": 12.101768,
            "value": 29.548698095535435
          },
          {
            "t": 14.12108,
            "value": 29.986466165413532
          },
          {
            "t": 16.152826,
            "value": 29.875372107567227
          },
          {
            "t": 18.077647,
            "value": 29.27609756097561
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.354858,
            "value": 394608.7933476275
          },
          {
            "t": 1.368692,
            "value": 394541.9072550339
          },
          {
            "t": 2.478699,
            "value": 360358.08783187857
          },
          {
            "t": 3.492461,
            "value": 394569.92864202836
          },
          {
            "t": 4.502322,
            "value": 396094.1159228845
          },
          {
            "t": 5.51749,
            "value": 394023.45227587945
          },
          {
            "t": 6.630638,
            "value": 359341.25561021536
          },
          {
            "t": 7.639894,
            "value": 396331.555125756
          },
          {
            "t": 8.649166,
            "value": 396325.2720772993
          },
          {
            "t": 9.663249,
            "value": 394445.03063358716
          },
          {
            "t": 10.680216,
            "value": 393326.4304544789
          },
          {
            "t": 11.797594,
            "value": 357980.916037366
          },
          {
            "t": 12.806579,
            "value": 396438.0045293042
          },
          {
            "t": 13.815899,
            "value": 397297.19018745294
          },
          {
            "t": 14.827765,
            "value": 394320.9871662849
          },
          {
            "t": 15.846887,
            "value": 392494.7160398853
          },
          {
            "t": 16.964157,
            "value": 358015.51997279085
          },
          {
            "t": 17.973419,
            "value": 396329.19895923947
          },
          {
            "t": 18.983816,
            "value": 395883.994113205
          },
          {
            "t": 19.996703,
            "value": 394910.78471734753
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.151469,
            "value": 396258.1344365159
          },
          {
            "t": 1.16259,
            "value": 395600.52654430084
          },
          {
            "t": 2.17643,
            "value": 394539.5723191036
          },
          {
            "t": 3.089373,
            "value": 438143.4547392335
          },
          {
            "t": 4.099286,
            "value": 396073.7212017272
          },
          {
            "t": 5.109625,
            "value": 395906.7204176024
          },
          {
            "t": 6.124196,
            "value": 394255.30593718926
          },
          {
            "t": 7.136365,
            "value": 395190.92167414725
          },
          {
            "t": 8.145639,
            "value": 396324.48671024916
          },
          {
            "t": 9.1561,
            "value": 395858.91983955837
          },
          {
            "t": 10.169513,
            "value": 394705.8109576254
          },
          {
            "t": 11.186839,
            "value": 393187.6311035008
          },
          {
            "t": 12.202518,
            "value": 393825.2144624434
          },
          {
            "t": 13.211761,
            "value": 396336.6602493156
          },
          {
            "t": 14.221845,
            "value": 396006.6687523018
          },
          {
            "t": 15.233072,
            "value": 395559.0584507731
          },
          {
            "t": 16.253597,
            "value": 391955.1211386297
          },
          {
            "t": 17.268818,
            "value": 394002.8821310828
          },
          {
            "t": 18.279065,
            "value": 395942.7743908173
          },
          {
            "t": 19.288951,
            "value": 396084.31050633435
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.050702,
            "value": 54841922.85523571
          },
          {
            "t": 2.075702,
            "value": 54827243.45679013
          },
          {
            "t": 4.099286,
            "value": 54728490.14421936
          },
          {
            "t": 6.124196,
            "value": 54829782.55823715
          },
          {
            "t": 8.04489,
            "value": 57661324.50041496
          },
          {
            "t": 10.068772,
            "value": 54859904.3817772
          },
          {
            "t": 12.101768,
            "value": 54476747.61780152
          },
          {
            "t": 14.12108,
            "value": 55051172.37950351
          },
          {
            "t": 16.152826,
            "value": 54579114.71217366
          },
          {
            "t": 18.077647,
            "value": 57608867.52586345
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.050702,
            "value": 54825158.110321686
          },
          {
            "t": 2.075702,
            "value": 54808304.69135803
          },
          {
            "t": 4.099286,
            "value": 54714397.32672328
          },
          {
            "t": 6.124196,
            "value": 54810587.13720609
          },
          {
            "t": 8.04489,
            "value": 57641403.055353954
          },
          {
            "t": 10.068772,
            "value": 54841097.94938638
          },
          {
            "t": 12.101768,
            "value": 54458890.72088681
          },
          {
            "t": 14.12108,
            "value": 55031376.52824327
          },
          {
            "t": 16.152826,
            "value": 54492778.62488716
          },
          {
            "t": 18.077647,
            "value": 57658649.29777886
          }
        ],
        "ram_mib": [
          {
            "t": 0.050702,
            "value": 52.82421875
          },
          {
            "t": 2.075702,
            "value": 53.0
          },
          {
            "t": 4.099286,
            "value": 52.87109375
          },
          {
            "t": 6.124196,
            "value": 52.75
          },
          {
            "t": 8.04489,
            "value": 53.28125
          },
          {
            "t": 10.068772,
            "value": 52.99609375
          },
          {
            "t": 12.101768,
            "value": 52.765625
          },
          {
            "t": 14.12108,
            "value": 53.109375
          },
          {
            "t": 16.152826,
            "value": 53.203125
          },
          {
            "t": 18.077647,
            "value": 52.99609375
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
      "name": "transform-4",
      "metrics": [
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-4 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 30.472796118698515
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 30.77641185647426
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 50.85390625
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 52.62890625
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 388370.62985948747
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 396597.5892815535
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-4 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.005284
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55319868.89291906
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55275124.76831193
        },
        {
          "extra": "DFE OTAP Transform Rename Multi Transform (Logs)/transform-4 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 139.48614512037855
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.110668,
            "value": 30.173
          },
          {
            "t": 2.03383,
            "value": 30.245796377264213
          },
          {
            "t": 4.059779,
            "value": 30.47203246955979
          },
          {
            "t": 6.086973,
            "value": 30.77641185647426
          },
          {
            "t": 8.111334,
            "value": 30.537856919712592
          },
          {
            "t": 10.149779,
            "value": 30.67411471321696
          },
          {
            "t": 12.078113,
            "value": 30.473607990012486
          },
          {
            "t": 14.101878,
            "value": 30.558449515473583
          },
          {
            "t": 16.133205,
            "value": 30.353828856964398
          },
          {
            "t": 18.156477,
            "value": 30.46286248830683
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.413794,
            "value": 359536.0187677802
          },
          {
            "t": 1.426131,
            "value": 395125.3386965013
          },
          {
            "t": 2.437662,
            "value": 395440.1792925773
          },
          {
            "t": 3.452147,
            "value": 394288.7277781337
          },
          {
            "t": 4.466661,
            "value": 394277.45698925795
          },
          {
            "t": 5.579879,
            "value": 359318.6599569895
          },
          {
            "t": 6.591688,
            "value": 395331.5299626708
          },
          {
            "t": 7.604441,
            "value": 394963.0363968312
          },
          {
            "t": 8.620976,
            "value": 393493.58359525254
          },
          {
            "t": 9.642792,
            "value": 391459.9105905564
          },
          {
            "t": 10.65948,
            "value": 393434.3672788505
          },
          {
            "t": 11.771875,
            "value": 359584.5001101227
          },
          {
            "t": 12.783957,
            "value": 395224.89284465095
          },
          {
            "t": 13.795633,
            "value": 395383.502227986
          },
          {
            "t": 14.810614,
            "value": 394096.04711812345
          },
          {
            "t": 15.827253,
            "value": 393453.33004144044
          },
          {
            "t": 16.940087,
            "value": 359442.64822965505
          },
          {
            "t": 17.951092,
            "value": 395645.91668686114
          },
          {
            "t": 18.968091,
            "value": 393314.054389434
          },
          {
            "t": 19.98273,
            "value": 394228.883376255
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.110668,
            "value": 395158.3737606956
          },
          {
            "t": 1.123139,
            "value": 395073.0440674351
          },
          {
            "t": 2.13473,
            "value": 394428.1829316394
          },
          {
            "t": 3.146083,
            "value": 395509.77749608696
          },
          {
            "t": 4.160666,
            "value": 394250.64287495456
          },
          {
            "t": 5.176085,
            "value": 394910.86930616817
          },
          {
            "t": 6.187876,
            "value": 395338.5630036242
          },
          {
            "t": 7.199937,
            "value": 394245.0109232546
          },
          {
            "t": 8.212236,
            "value": 395140.171036423
          },
          {
            "t": 9.130492,
            "value": 435608.37065045047
          },
          {
            "t": 10.149779,
            "value": 393412.2577841177
          },
          {
            "t": 11.166712,
            "value": 393339.58087700955
          },
          {
            "t": 12.179002,
            "value": 394155.8249118336
          },
          {
            "t": 13.190375,
            "value": 395501.9562515511
          },
          {
            "t": 14.202681,
            "value": 396125.28227630776
          },
          {
            "t": 15.21682,
            "value": 394423.2496728753
          },
          {
            "t": 16.234054,
            "value": 392240.13353859587
          },
          {
            "t": 17.245243,
            "value": 395573.9233713975
          },
          {
            "t": 18.257342,
            "value": 395218.2543407315
          },
          {
            "t": 19.273669,
            "value": 394558.05070612114
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.110668,
            "value": 54632651.57480315
          },
          {
            "t": 2.03383,
            "value": 57662475.13210016
          },
          {
            "t": 4.059779,
            "value": 54734764.794177935
          },
          {
            "t": 6.086973,
            "value": 54839207.29836414
          },
          {
            "t": 8.111334,
            "value": 54710756.62888191
          },
          {
            "t": 10.149779,
            "value": 54468666.55710603
          },
          {
            "t": 12.078113,
            "value": 57508287.9833058
          },
          {
            "t": 14.101878,
            "value": 54796527.2647763
          },
          {
            "t": 16.133205,
            "value": 54647539.26866526
          },
          {
            "t": 18.156477,
            "value": 54750371.1809386
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.110668,
            "value": 54682126.47637795
          },
          {
            "t": 2.03383,
            "value": 57709669.80420786
          },
          {
            "t": 4.059779,
            "value": 54778510.21916148
          },
          {
            "t": 6.086973,
            "value": 54817871.89583236
          },
          {
            "t": 8.111334,
            "value": 54822711.46302463
          },
          {
            "t": 10.149779,
            "value": 54510742.747535504
          },
          {
            "t": 12.078113,
            "value": 57482406.57479461
          },
          {
            "t": 14.101878,
            "value": 54908195.86266192
          },
          {
            "t": 16.133205,
            "value": 54635688.88711665
          },
          {
            "t": 18.156477,
            "value": 54850764.99847771
          }
        ],
        "ram_mib": [
          {
            "t": 0.110668,
            "value": 50.12890625
          },
          {
            "t": 2.03383,
            "value": 50.41015625
          },
          {
            "t": 4.059779,
            "value": 50.8828125
          },
          {
            "t": 6.086973,
            "value": 50.26953125
          },
          {
            "t": 8.111334,
            "value": 50.62890625
          },
          {
            "t": 10.149779,
            "value": 50.38671875
          },
          {
            "t": 12.078113,
            "value": 50.4921875
          },
          {
            "t": 14.101878,
            "value": 50.36328125
          },
          {
            "t": 16.133205,
            "value": 52.34765625
          },
          {
            "t": 18.156477,
            "value": 52.62890625
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
