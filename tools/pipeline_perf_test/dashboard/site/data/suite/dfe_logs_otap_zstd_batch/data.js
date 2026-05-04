window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otap_zstd_batch"] = {
  "name": "DFE OTAP Batch Processor w/ Zstd (Logs)",
  "slug": "dfe_logs_otap_zstd_batch",
  "description": "Dataflow Engine OTAP logs through a batch processor with zstd compression",
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
      "name": "100k",
      "metrics": [
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.052631575614213943
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 7.159843545267512
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 8.308670807453417
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 17.994921875
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 18.5234375
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 99270.72589889643
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/100k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 99218.47814842332
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000994
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 10779092.194510754
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 11048127.015572652
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.118728,
            "value": 7.827386716325265
          },
          {
            "t": 3.134207,
            "value": 5.746376631448104
          },
          {
            "t": 5.047797,
            "value": 6.681290322580645
          },
          {
            "t": 7.062195,
            "value": 7.001215127092375
          },
          {
            "t": 9.079132,
            "value": 6.542277227722772
          },
          {
            "t": 11.094134,
            "value": 6.930484472049689
          },
          {
            "t": 13.109249,
            "value": 6.977633872976338
          },
          {
            "t": 15.124017,
            "value": 7.962141967621419
          },
          {
            "t": 17.13735,
            "value": 8.308670807453417
          },
          {
            "t": 19.150652,
            "value": 7.620958307405103
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.111581,
            "value": 99074.25020607444
          },
          {
            "t": 1.118728,
            "value": 99290.37171336458
          },
          {
            "t": 2.126164,
            "value": 99261.88859639718
          },
          {
            "t": 3.134207,
            "value": 99202.11736999314
          },
          {
            "t": 4.141143,
            "value": 99311.1776716693
          },
          {
            "t": 5.148247,
            "value": 99294.61108286731
          },
          {
            "t": 6.155133,
            "value": 99316.10927155607
          },
          {
            "t": 7.162668,
            "value": 99252.13516155766
          },
          {
            "t": 8.17081,
            "value": 99192.37567723593
          },
          {
            "t": 9.179855,
            "value": 99103.6078668444
          },
          {
            "t": 10.186526,
            "value": 99337.32073338756
          },
          {
            "t": 11.194854,
            "value": 100165.81905887768
          },
          {
            "t": 12.202543,
            "value": 99236.96696103658
          },
          {
            "t": 13.209748,
            "value": 98291.80752676964
          },
          {
            "t": 14.217665,
            "value": 99214.51865580202
          },
          {
            "t": 15.224856,
            "value": 100278.89446986718
          },
          {
            "t": 16.231716,
            "value": 98325.48715809545
          },
          {
            "t": 17.237846,
            "value": 99390.73479570234
          },
          {
            "t": 18.245303,
            "value": 99259.81952579613
          },
          {
            "t": 19.251161,
            "value": 99417.61163106523
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.111581,
            "value": 98083.5077040137
          },
          {
            "t": 1.118728,
            "value": 98297.46799623093
          },
          {
            "t": 2.126164,
            "value": 98269.26971043322
          },
          {
            "t": 3.134207,
            "value": 98210.0961962932
          },
          {
            "t": 4.141143,
            "value": 98318.0658949526
          },
          {
            "t": 5.148247,
            "value": 98301.66497203863
          },
          {
            "t": 6.155133,
            "value": 107261.39801328056
          },
          {
            "t": 7.162668,
            "value": 98259.61380994208
          },
          {
            "t": 8.17081,
            "value": 98200.45192046357
          },
          {
            "t": 9.179855,
            "value": 98112.57178817595
          },
          {
            "t": 10.186526,
            "value": 98343.94752605369
          },
          {
            "t": 11.194854,
            "value": 98182.33749335534
          },
          {
            "t": 12.202543,
            "value": 98244.59729142622
          },
          {
            "t": 13.209748,
            "value": 98291.80752676964
          },
          {
            "t": 14.217665,
            "value": 98222.373469244
          },
          {
            "t": 15.224856,
            "value": 98293.17378729556
          },
          {
            "t": 16.231716,
            "value": 107264.16780883141
          },
          {
            "t": 17.237846,
            "value": 98396.82744774532
          },
          {
            "t": 18.245303,
            "value": 98267.22133053817
          },
          {
            "t": 19.251161,
            "value": 98423.43551475457
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.118728,
            "value": 10988208.229047392
          },
          {
            "t": 3.134207,
            "value": 10993538.508711824
          },
          {
            "t": 5.047797,
            "value": 11577477.934144724
          },
          {
            "t": 7.062195,
            "value": 10946354.692568202
          },
          {
            "t": 9.079132,
            "value": 10984315.32566461
          },
          {
            "t": 11.094134,
            "value": 10994466.506732997
          },
          {
            "t": 13.109249,
            "value": 10993078.806916725
          },
          {
            "t": 15.124017,
            "value": 10997062.689103657
          },
          {
            "t": 17.13735,
            "value": 11001726.987040892
          },
          {
            "t": 19.150652,
            "value": 11005040.475795485
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.118728,
            "value": 11155756.212152695
          },
          {
            "t": 3.134207,
            "value": 10539042.083792487
          },
          {
            "t": 5.047797,
            "value": 11188636.541787948
          },
          {
            "t": 7.062195,
            "value": 11021578.655260779
          },
          {
            "t": 9.079132,
            "value": 10531435.04234391
          },
          {
            "t": 11.094134,
            "value": 10666873.779777886
          },
          {
            "t": 13.109249,
            "value": 11016652.15136605
          },
          {
            "t": 15.124017,
            "value": 10543009.418454135
          },
          {
            "t": 17.13735,
            "value": 10549921.945351316
          },
          {
            "t": 19.150652,
            "value": 10578016.11482033
          }
        ],
        "ram_mib": [
          {
            "t": 1.118728,
            "value": 17.9296875
          },
          {
            "t": 3.134207,
            "value": 18.5234375
          },
          {
            "t": 5.047797,
            "value": 18.08984375
          },
          {
            "t": 7.062195,
            "value": 18.171875
          },
          {
            "t": 9.079132,
            "value": 18.171875
          },
          {
            "t": 11.094134,
            "value": 17.87109375
          },
          {
            "t": 13.109249,
            "value": 17.64453125
          },
          {
            "t": 15.124017,
            "value": 17.58203125
          },
          {
            "t": 17.13735,
            "value": 17.9609375
          },
          {
            "t": 19.150652,
            "value": 18.00390625
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
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -0.18464785814285278
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 12.164892066903668
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 13.693004926108374
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 18.691796875
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 19.46484375
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 197951.70375271732
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/200k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 198317.2173180745
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.00066
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 21384705.72410392
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 22034114.94910094
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.080161,
            "value": 12.49824039653036
          },
          {
            "t": 3.098248,
            "value": 11.29948084054388
          },
          {
            "t": 5.113233,
            "value": 12.100531849103277
          },
          {
            "t": 7.129271,
            "value": 12.523023543990085
          },
          {
            "t": 9.144396,
            "value": 11.975331269349846
          },
          {
            "t": 11.160202,
            "value": 12.11567701863354
          },
          {
            "t": 13.175376,
            "value": 13.693004926108374
          },
          {
            "t": 15.091325,
            "value": 12.181584158415841
          },
          {
            "t": 17.107656,
            "value": 12.337799752781212
          },
          {
            "t": 19.123368,
            "value": 10.924246913580248
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.072762,
            "value": 197541.94291931254
          },
          {
            "t": 1.080161,
            "value": 198531.06862325655
          },
          {
            "t": 2.089409,
            "value": 198167.34836234504
          },
          {
            "t": 3.098248,
            "value": 198247.6886797596
          },
          {
            "t": 4.10576,
            "value": 198508.80188027536
          },
          {
            "t": 5.113233,
            "value": 198516.4862978958
          },
          {
            "t": 6.121236,
            "value": 198412.10790047253
          },
          {
            "t": 7.129271,
            "value": 198405.80932209696
          },
          {
            "t": 8.136457,
            "value": 198573.05403371374
          },
          {
            "t": 9.144396,
            "value": 198424.70625702548
          },
          {
            "t": 10.152516,
            "value": 198389.08066500022
          },
          {
            "t": 11.160202,
            "value": 198474.52480236898
          },
          {
            "t": 12.16826,
            "value": 198401.28246588988
          },
          {
            "t": 13.175376,
            "value": 198586.85593317953
          },
          {
            "t": 14.184293,
            "value": 198232.3620277981
          },
          {
            "t": 15.191901,
            "value": 198489.88892505816
          },
          {
            "t": 16.199417,
            "value": 198508.0137685158
          },
          {
            "t": 17.208553,
            "value": 198189.34216993547
          },
          {
            "t": 18.215882,
            "value": 199537.58901014467
          },
          {
            "t": 19.223898,
            "value": 188489.07160203805
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.072762,
            "value": 196549.269839316
          },
          {
            "t": 1.080161,
            "value": 205479.65602507052
          },
          {
            "t": 2.089409,
            "value": 196185.6748787216
          },
          {
            "t": 3.098248,
            "value": 196265.211792962
          },
          {
            "t": 4.10576,
            "value": 196523.7138614726
          },
          {
            "t": 5.113233,
            "value": 196531.32143491684
          },
          {
            "t": 6.121236,
            "value": 205356.53167698905
          },
          {
            "t": 7.129271,
            "value": 196421.75122887598
          },
          {
            "t": 8.136457,
            "value": 196587.3234933766
          },
          {
            "t": 9.144396,
            "value": 196440.45919445524
          },
          {
            "t": 10.152516,
            "value": 205332.69848827523
          },
          {
            "t": 11.160202,
            "value": 196489.7795543453
          },
          {
            "t": 12.16826,
            "value": 196417.269641231
          },
          {
            "t": 13.175376,
            "value": 196600.98737384772
          },
          {
            "t": 14.184293,
            "value": 196250.03840752013
          },
          {
            "t": 15.191901,
            "value": 205437.03503743518
          },
          {
            "t": 16.199417,
            "value": 196522.93363083067
          },
          {
            "t": 17.208553,
            "value": 196207.44874823611
          },
          {
            "t": 18.215882,
            "value": 196559.41603984399
          },
          {
            "t": 19.223898,
            "value": 196425.45356422913
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.080161,
            "value": 21992061.664300818
          },
          {
            "t": 3.098248,
            "value": 21959629.094285827
          },
          {
            "t": 5.113233,
            "value": 21990614.322190985
          },
          {
            "t": 7.129271,
            "value": 21971543.195118345
          },
          {
            "t": 9.144396,
            "value": 21996247.37919484
          },
          {
            "t": 11.160202,
            "value": 21925736.90126927
          },
          {
            "t": 13.175376,
            "value": 22045446.199683003
          },
          {
            "t": 15.091325,
            "value": 23054439.340504367
          },
          {
            "t": 17.107656,
            "value": 21972654.291383706
          },
          {
            "t": 19.123368,
            "value": 21432777.103078216
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.080161,
            "value": 21513897.795292787
          },
          {
            "t": 3.098248,
            "value": 21095187.174784835
          },
          {
            "t": 5.113233,
            "value": 21510615.711779494
          },
          {
            "t": 7.129271,
            "value": 21022293.726606343
          },
          {
            "t": 9.144396,
            "value": 21507168.04168476
          },
          {
            "t": 11.160202,
            "value": 21498571.787166025
          },
          {
            "t": 13.175376,
            "value": 21036126.408935405
          },
          {
            "t": 15.091325,
            "value": 22613653.07740446
          },
          {
            "t": 17.107656,
            "value": 21493598.521274533
          },
          {
            "t": 19.123368,
            "value": 20555944.996110555
          }
        ],
        "ram_mib": [
          {
            "t": 1.080161,
            "value": 19.3203125
          },
          {
            "t": 3.098248,
            "value": 18.26953125
          },
          {
            "t": 5.113233,
            "value": 18.3046875
          },
          {
            "t": 7.129271,
            "value": 19.11328125
          },
          {
            "t": 9.144396,
            "value": 19.16796875
          },
          {
            "t": 11.160202,
            "value": 18.234375
          },
          {
            "t": 13.175376,
            "value": 18.51171875
          },
          {
            "t": 15.091325,
            "value": 18.0546875
          },
          {
            "t": 17.107656,
            "value": 18.4765625
          },
          {
            "t": 19.123368,
            "value": 19.46484375
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
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.052631575614213943
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 16.970925605272743
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 18.050891089108912
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 19.806640625
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 21.515625
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 296256.24106651003
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/300k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 296100.3167291066
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000764
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 31973528.489145357
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 32969326.41414832
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.071159,
            "value": 17.02099502487562
          },
          {
            "t": 3.091486,
            "value": 17.18031152647975
          },
          {
            "t": 5.117241,
            "value": 16.95491271820449
          },
          {
            "t": 7.142041,
            "value": 16.460349127182045
          },
          {
            "t": 9.067376,
            "value": 17.5055900621118
          },
          {
            "t": 11.090164,
            "value": 16.824592408214066
          },
          {
            "t": 13.120724,
            "value": 18.050891089108912
          },
          {
            "t": 15.148867,
            "value": 16.68012383900929
          },
          {
            "t": 17.173113,
            "value": 16.73162732919255
          },
          {
            "t": 19.199281,
            "value": 16.29986292834891
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.06161,
            "value": 297008.824132165
          },
          {
            "t": 1.071159,
            "value": 297162.39627794194
          },
          {
            "t": 2.08,
            "value": 297370.94348861714
          },
          {
            "t": 3.091486,
            "value": 296593.32902284357
          },
          {
            "t": 4.104295,
            "value": 296205.8986442656
          },
          {
            "t": 5.117241,
            "value": 296165.83707324974
          },
          {
            "t": 6.127561,
            "value": 297925.40977116156
          },
          {
            "t": 7.142041,
            "value": 294732.2766343348
          },
          {
            "t": 8.158815,
            "value": 295050.8175858155
          },
          {
            "t": 9.169749,
            "value": 296755.2777926155
          },
          {
            "t": 10.181765,
            "value": 296438.00098022167
          },
          {
            "t": 11.192898,
            "value": 296696.8737050417
          },
          {
            "t": 12.210795,
            "value": 294725.30128293927
          },
          {
            "t": 13.223948,
            "value": 296105.32663872093
          },
          {
            "t": 14.240387,
            "value": 296131.88789489574
          },
          {
            "t": 15.252183,
            "value": 295514.1154936371
          },
          {
            "t": 16.264464,
            "value": 296360.3979527424
          },
          {
            "t": 17.27695,
            "value": 296300.3932893887
          },
          {
            "t": 18.290638,
            "value": 295949.0494116533
          },
          {
            "t": 19.301711,
            "value": 296714.48055679456
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.06161,
            "value": 302949.0006148083
          },
          {
            "t": 1.071159,
            "value": 294190.7723151625
          },
          {
            "t": 2.08,
            "value": 294397.23405373096
          },
          {
            "t": 3.091486,
            "value": 302525.19560330047
          },
          {
            "t": 4.104295,
            "value": 293243.8396578229
          },
          {
            "t": 5.117241,
            "value": 293204.17870251724
          },
          {
            "t": 6.127561,
            "value": 302874.3368437722
          },
          {
            "t": 7.243756,
            "value": 266082.53934124415
          },
          {
            "t": 8.158815,
            "value": 324569.23542634957
          },
          {
            "t": 9.271583,
            "value": 258814.05647897854
          },
          {
            "t": 10.28336,
            "value": 311333.4262391812
          },
          {
            "t": 11.192898,
            "value": 326539.40791918535
          },
          {
            "t": 12.210795,
            "value": 300619.807308598
          },
          {
            "t": 13.223948,
            "value": 293144.27337233373
          },
          {
            "t": 14.240387,
            "value": 292196.58041456493
          },
          {
            "t": 15.252183,
            "value": 302432.5061573677
          },
          {
            "t": 16.264464,
            "value": 293396.7939732149
          },
          {
            "t": 17.27695,
            "value": 293337.3893564948
          },
          {
            "t": 18.290638,
            "value": 301868.03039988637
          },
          {
            "t": 19.301711,
            "value": 293747.33575122664
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.071159,
            "value": 32913820.421663478
          },
          {
            "t": 3.091486,
            "value": 32848976.428073276
          },
          {
            "t": 5.117241,
            "value": 32757149.80340663
          },
          {
            "t": 7.142041,
            "value": 32777580.007902015
          },
          {
            "t": 9.067376,
            "value": 34532141.679240234
          },
          {
            "t": 11.090164,
            "value": 32866301.362278204
          },
          {
            "t": 13.120724,
            "value": 32740543.495390434
          },
          {
            "t": 15.148867,
            "value": 32723266.061614
          },
          {
            "t": 17.173113,
            "value": 32731527.689816352
          },
          {
            "t": 19.199281,
            "value": 32801957.19209858
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.071159,
            "value": 31465021.14259118
          },
          {
            "t": 3.091486,
            "value": 31927689.923462886
          },
          {
            "t": 5.117241,
            "value": 31840924.98846109
          },
          {
            "t": 7.142041,
            "value": 31853282.29948637
          },
          {
            "t": 9.067376,
            "value": 33499247.14400351
          },
          {
            "t": 11.090164,
            "value": 31885039.361514904
          },
          {
            "t": 13.120724,
            "value": 31762719.643842094
          },
          {
            "t": 15.148867,
            "value": 31801194.984771784
          },
          {
            "t": 17.173113,
            "value": 31866147.19752441
          },
          {
            "t": 19.199281,
            "value": 31834018.20579537
          }
        ],
        "ram_mib": [
          {
            "t": 1.071159,
            "value": 20.1171875
          },
          {
            "t": 3.091486,
            "value": 19.25390625
          },
          {
            "t": 5.117241,
            "value": 19.296875
          },
          {
            "t": 7.142041,
            "value": 18.57421875
          },
          {
            "t": 9.067376,
            "value": 21.515625
          },
          {
            "t": 11.090164,
            "value": 20.42578125
          },
          {
            "t": 13.120724,
            "value": 19.25
          },
          {
            "t": 15.148867,
            "value": 20.04296875
          },
          {
            "t": 17.173113,
            "value": 19.78125
          },
          {
            "t": 19.199281,
            "value": 19.80859375
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
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.07692307978868484
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 21.659243246667074
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 22.110429447852763
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 20.73515625
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 22.24609375
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 403528.11877053464
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/400k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 403217.7125253265
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000637
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 42639865.24522508
        },
        {
          "extra": "DFE OTAP Batch Processor w/ Zstd (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 44000538.45761608
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.084417,
            "value": 21.898843788437887
          },
          {
            "t": 3.105255,
            "value": 21.683184323331293
          },
          {
            "t": 5.127753,
            "value": 21.65571166768479
          },
          {
            "t": 7.156069,
            "value": 21.822024691358024
          },
          {
            "t": 9.081203,
            "value": 22.110429447852763
          },
          {
            "t": 11.107559,
            "value": 22.033067973055726
          },
          {
            "t": 13.134915,
            "value": 21.415730886850152
          },
          {
            "t": 15.156349,
            "value": 21.206781326781325
          },
          {
            "t": 17.178008,
            "value": 21.222148557397176
          },
          {
            "t": 19.201511,
            "value": 21.544509803921567
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.073828,
            "value": 395925.9222599452
          },
          {
            "t": 1.084417,
            "value": 395808.7808198981
          },
          {
            "t": 2.094841,
            "value": 395873.4155166544
          },
          {
            "t": 3.105255,
            "value": 395877.33344945736
          },
          {
            "t": 4.116475,
            "value": 395561.79664168035
          },
          {
            "t": 5.127753,
            "value": 395539.109918341
          },
          {
            "t": 6.142879,
            "value": 394039.7546708488
          },
          {
            "t": 7.156069,
            "value": 394792.68449155637
          },
          {
            "t": 8.171358,
            "value": 393976.4933925217
          },
          {
            "t": 9.183291,
            "value": 395283.08692373906
          },
          {
            "t": 10.197379,
            "value": 394443.08580714883
          },
          {
            "t": 11.211869,
            "value": 394286.7844927008
          },
          {
            "t": 12.323692,
            "value": 359769.49568411516
          },
          {
            "t": 13.336885,
            "value": 395778.49432437844
          },
          {
            "t": 14.347625,
            "value": 394760.27465025627
          },
          {
            "t": 15.358198,
            "value": 494768.8093784417
          },
          {
            "t": 16.36936,
            "value": 395584.48596762936
          },
          {
            "t": 17.380005,
            "value": 494733.56124059384
          },
          {
            "t": 18.392737,
            "value": 394971.2263461607
          },
          {
            "t": 19.403336,
            "value": 395804.8642438791
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.073828,
            "value": 400874.9962881945
          },
          {
            "t": 1.084417,
            "value": 391850.6930116991
          },
          {
            "t": 2.094841,
            "value": 391914.68136148783
          },
          {
            "t": 3.105255,
            "value": 400825.8001175756
          },
          {
            "t": 4.116475,
            "value": 391606.17867526354
          },
          {
            "t": 5.127753,
            "value": 400483.34879232023
          },
          {
            "t": 6.142879,
            "value": 390099.35712414025
          },
          {
            "t": 7.25831,
            "value": 363088.34880866675
          },
          {
            "t": 8.171358,
            "value": 413997.9497244395
          },
          {
            "t": 9.285106,
            "value": 379798.6618157788
          },
          {
            "t": 10.29905,
            "value": 390554.1134421625
          },
          {
            "t": 11.313131,
            "value": 390501.3504838371
          },
          {
            "t": 12.323692,
            "value": 400767.4944906839
          },
          {
            "t": 13.336885,
            "value": 390843.60038018425
          },
          {
            "t": 14.347625,
            "value": 596592.5955240715
          },
          {
            "t": 15.358198,
            "value": 391856.8970277259
          },
          {
            "t": 16.36936,
            "value": 400529.2920422247
          },
          {
            "t": 17.380005,
            "value": 391828.9805025504
          },
          {
            "t": 18.392737,
            "value": 399908.3666754877
          },
          {
            "t": 19.403336,
            "value": 391846.8156014403
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.084417,
            "value": 43755139.7189045
          },
          {
            "t": 3.105255,
            "value": 43972869.17605469
          },
          {
            "t": 5.127753,
            "value": 43826769.17356655
          },
          {
            "t": 7.156069,
            "value": 43704915.80207424
          },
          {
            "t": 9.081203,
            "value": 45932465.480325006
          },
          {
            "t": 11.107559,
            "value": 43742780.14327197
          },
          {
            "t": 13.134915,
            "value": 43617260.11613155
          },
          {
            "t": 15.156349,
            "value": 43853095.87154465
          },
          {
            "t": 17.178008,
            "value": 43794513.318022475
          },
          {
            "t": 19.201511,
            "value": 43805575.77626523
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.084417,
            "value": 42389872.9216346
          },
          {
            "t": 3.105255,
            "value": 42388025.660641775
          },
          {
            "t": 5.127753,
            "value": 42830497.731023714
          },
          {
            "t": 7.156069,
            "value": 42236651.48822965
          },
          {
            "t": 9.081203,
            "value": 44500692.41933289
          },
          {
            "t": 11.107559,
            "value": 42272500.488561735
          },
          {
            "t": 13.134915,
            "value": 42247156.39483149
          },
          {
            "t": 15.156349,
            "value": 42845831.22674299
          },
          {
            "t": 17.178008,
            "value": 42362674.4173968
          },
          {
            "t": 19.201511,
            "value": 42324749.70385515
          }
        ],
        "ram_mib": [
          {
            "t": 1.084417,
            "value": 20.26953125
          },
          {
            "t": 3.105255,
            "value": 20.53125
          },
          {
            "t": 5.127753,
            "value": 20.28125
          },
          {
            "t": 7.156069,
            "value": 20.2578125
          },
          {
            "t": 9.081203,
            "value": 22.24609375
          },
          {
            "t": 11.107559,
            "value": 20.79296875
          },
          {
            "t": 13.134915,
            "value": 20.37890625
          },
          {
            "t": 15.156349,
            "value": 20.29296875
          },
          {
            "t": 17.178008,
            "value": 20.9296875
          },
          {
            "t": 19.201511,
            "value": 21.37109375
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
