window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otap_none_attr_insert_multi_transform"] = {
  "name": "DFE OTAP Attr Insert Multi Transform (Logs)",
  "slug": "dfe_logs_otap_none_attr_insert_multi_transform",
  "description": "Dataflow Engine OTAP logs, attributes processor insert sweep over 1-4 insert actions at 400k signals/sec",
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
    "started_at": "2026-05-27T18:02:00Z",
    "ended_at": "2026-05-27T18:07:48Z",
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
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-1 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 38.42264536980282
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 38.67113054341037
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 52.10546875
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 52.640625
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 388179.64381861716
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 409311.5799308501
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-1 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.002886
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 69024843.84208626
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55247246.6796117
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-1 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 168.63643059829252
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.105286,
            "value": 38.58377028714107
          },
          {
            "t": 2.049343,
            "value": 38.50953154278576
          },
          {
            "t": 4.063884,
            "value": 38.21143035213462
          },
          {
            "t": 6.082841,
            "value": 38.26864628820961
          },
          {
            "t": 8.098832,
            "value": 38.170146463072605
          },
          {
            "t": 10.11231,
            "value": 38.67113054341037
          },
          {
            "t": 12.126707,
            "value": 38.55635341867
          },
          {
            "t": 14.14396,
            "value": 38.45660541887263
          },
          {
            "t": 16.157977,
            "value": 38.358764430577224
          },
          {
            "t": 18.170716,
            "value": 38.44007495315428
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.406278,
            "value": 361138.92382406397
          },
          {
            "t": 1.444327,
            "value": 385338.2643786565
          },
          {
            "t": 2.45069,
            "value": 397470.8927096883
          },
          {
            "t": 3.459138,
            "value": 396649.10833280446
          },
          {
            "t": 4.471322,
            "value": 395185.0651660173
          },
          {
            "t": 5.578533,
            "value": 361268.08711257385
          },
          {
            "t": 6.584547,
            "value": 397608.780792315
          },
          {
            "t": 7.594227,
            "value": 396165.1216226924
          },
          {
            "t": 8.600867,
            "value": 397361.5195104506
          },
          {
            "t": 9.707263,
            "value": 361534.2065589536
          },
          {
            "t": 10.714194,
            "value": 397246.6832384741
          },
          {
            "t": 11.722072,
            "value": 396873.43110971764
          },
          {
            "t": 12.732905,
            "value": 395713.23848746525
          },
          {
            "t": 13.839268,
            "value": 361544.9902066501
          },
          {
            "t": 14.846211,
            "value": 397241.9491470719
          },
          {
            "t": 15.852941,
            "value": 397325.9960466064
          },
          {
            "t": 16.960533,
            "value": 361143.8146898858
          },
          {
            "t": 17.966802,
            "value": 397508.0222087732
          },
          {
            "t": 18.97404,
            "value": 397125.6048719369
          },
          {
            "t": 19.984842,
            "value": 395725.3745046013
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.406278,
            "value": 397668.8651127093
          },
          {
            "t": 1.444327,
            "value": 770676.528757313
          },
          {
            "t": 2.45069,
            "value": 397470.8927096883
          },
          {
            "t": 3.459138,
            "value": 396649.10833280446
          },
          {
            "t": 4.573117,
            "value": 359073.1961733569
          },
          {
            "t": 5.578533,
            "value": 397845.2700175847
          },
          {
            "t": 6.584547,
            "value": 397608.780792315
          },
          {
            "t": 7.594227,
            "value": 396165.1216226924
          },
          {
            "t": 8.702352,
            "value": 360970.10716300056
          },
          {
            "t": 9.707263,
            "value": 398045.20002268854
          },
          {
            "t": 10.714194,
            "value": 397246.6832384741
          },
          {
            "t": 11.722072,
            "value": 396873.43110971764
          },
          {
            "t": 12.834034,
            "value": 359724.52296031703
          },
          {
            "t": 13.839268,
            "value": 397917.3008473649
          },
          {
            "t": 14.846211,
            "value": 397241.9491470719
          },
          {
            "t": 15.955341,
            "value": 360643.02651627857
          },
          {
            "t": 16.960533,
            "value": 397933.92705075245
          },
          {
            "t": 17.966802,
            "value": 397508.0222087732
          },
          {
            "t": 18.97404,
            "value": 397125.6048719369
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.105286,
            "value": 54940874.290021725
          },
          {
            "t": 2.049343,
            "value": 57193655.330064915
          },
          {
            "t": 4.063884,
            "value": 54930662.6174399
          },
          {
            "t": 6.082841,
            "value": 55077517.74802534
          },
          {
            "t": 8.098832,
            "value": 54891765.38982565
          },
          {
            "t": 10.11231,
            "value": 55225556.9715686
          },
          {
            "t": 12.126707,
            "value": 54923674.92604486
          },
          {
            "t": 14.14396,
            "value": 55117599.52767451
          },
          {
            "t": 16.157977,
            "value": 54996558.618919306
          },
          {
            "t": 18.170716,
            "value": 55174601.37653218
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.105286,
            "value": 68876897.04110749
          },
          {
            "t": 2.049343,
            "value": 71172797.9169335
          },
          {
            "t": 4.063884,
            "value": 68850644.88635376
          },
          {
            "t": 6.082841,
            "value": 68698908.89206655
          },
          {
            "t": 8.098832,
            "value": 68629629.79497428
          },
          {
            "t": 10.11231,
            "value": 68884779.96779701
          },
          {
            "t": 12.126707,
            "value": 68679684.29261957
          },
          {
            "t": 14.14396,
            "value": 68843364.71429215
          },
          {
            "t": 16.157977,
            "value": 68610587.19961153
          },
          {
            "t": 18.170716,
            "value": 69001143.71510664
          }
        ],
        "ram_mib": [
          {
            "t": 0.105286,
            "value": 51.92578125
          },
          {
            "t": 2.049343,
            "value": 52.59375
          },
          {
            "t": 4.063884,
            "value": 52.125
          },
          {
            "t": 6.082841,
            "value": 51.76953125
          },
          {
            "t": 8.098832,
            "value": 51.94140625
          },
          {
            "t": 10.11231,
            "value": 52.01953125
          },
          {
            "t": 12.126707,
            "value": 52.0
          },
          {
            "t": 14.14396,
            "value": 51.9765625
          },
          {
            "t": 16.157977,
            "value": 52.640625
          },
          {
            "t": 18.170716,
            "value": 52.0625
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
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-2 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -11.111111640930176
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 40.17165454880301
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 40.57969422776911
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 53.219140625
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 53.765625
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 384710.2058167542
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 416243.22590165044
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-2 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000584
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 69306840.32759534
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55454695.18006821
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-2 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 166.50562943688865
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.027182,
            "value": 40.315906396255855
          },
          {
            "t": 2.048517,
            "value": 40.01067331670823
          },
          {
            "t": 4.100878,
            "value": 40.01148564294631
          },
          {
            "t": 6.116806,
            "value": 40.2955687129947
          },
          {
            "t": 8.138724,
            "value": 40.297627965043695
          },
          {
            "t": 10.155083,
            "value": 40.02375545851529
          },
          {
            "t": 12.077381,
            "value": 40.31980049875312
          },
          {
            "t": 14.099082,
            "value": 39.86382766156728
          },
          {
            "t": 16.126558,
            "value": 39.998205607476635
          },
          {
            "t": 18.143722,
            "value": 40.57969422776911
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.333881,
            "value": 394540.3506280096
          },
          {
            "t": 1.442591,
            "value": 360779.6448124396
          },
          {
            "t": 2.450292,
            "value": 396943.1408721436
          },
          {
            "t": 3.59521,
            "value": 349369.99855011445
          },
          {
            "t": 4.603132,
            "value": 396856.1059288318
          },
          {
            "t": 5.611029,
            "value": 396865.94959604007
          },
          {
            "t": 6.624348,
            "value": 394742.4256329941
          },
          {
            "t": 7.733962,
            "value": 360485.718457049
          },
          {
            "t": 8.742599,
            "value": 396574.78359409777
          },
          {
            "t": 9.750285,
            "value": 396949.04960473796
          },
          {
            "t": 10.764443,
            "value": 394415.86025057244
          },
          {
            "t": 11.87347,
            "value": 360676.52095034654
          },
          {
            "t": 12.887447,
            "value": 394486.26546756
          },
          {
            "t": 13.895389,
            "value": 396848.231346645
          },
          {
            "t": 14.909148,
            "value": 394571.0962861982
          },
          {
            "t": 15.9225,
            "value": 394729.5707710647
          },
          {
            "t": 17.032056,
            "value": 360504.56218523445
          },
          {
            "t": 18.040543,
            "value": 396633.7692007929
          },
          {
            "t": 19.049267,
            "value": 396540.5799802523
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.434957,
            "value": 396173.81063879817
          },
          {
            "t": 1.442591,
            "value": 793939.0691461385
          },
          {
            "t": 2.450292,
            "value": 396943.1408721436
          },
          {
            "t": 3.59521,
            "value": 348496.5735537392
          },
          {
            "t": 4.603132,
            "value": 397848.2461936539
          },
          {
            "t": 5.611029,
            "value": 396865.94959604007
          },
          {
            "t": 6.624348,
            "value": 394742.4256329941
          },
          {
            "t": 7.632257,
            "value": 396861.22457483766
          },
          {
            "t": 8.640843,
            "value": 396594.8367318206
          },
          {
            "t": 9.648589,
            "value": 395933.1021904329
          },
          {
            "t": 10.65764,
            "value": 397403.10450116004
          },
          {
            "t": 11.671217,
            "value": 394641.94629515073
          },
          {
            "t": 12.584619,
            "value": 437923.28022053814
          },
          {
            "t": 13.592566,
            "value": 395854.1470930515
          },
          {
            "t": 14.601242,
            "value": 397550.8488355032
          },
          {
            "t": 15.614694,
            "value": 394690.6217561364
          },
          {
            "t": 16.628857,
            "value": 394413.9157117742
          },
          {
            "t": 17.637201,
            "value": 396690.0184857549
          },
          {
            "t": 18.645962,
            "value": 396526.0354038271
          },
          {
            "t": 19.654489,
            "value": 396618.0379900588
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.027182,
            "value": 57931763.79836497
          },
          {
            "t": 2.048517,
            "value": 54850741.712778926
          },
          {
            "t": 4.100878,
            "value": 54157559.51316557
          },
          {
            "t": 6.116806,
            "value": 54997208.23362739
          },
          {
            "t": 8.138724,
            "value": 54971370.25339307
          },
          {
            "t": 10.155083,
            "value": 55123008.3531752
          },
          {
            "t": 12.077381,
            "value": 57749358.32009397
          },
          {
            "t": 14.099082,
            "value": 54978641.7477164
          },
          {
            "t": 16.126558,
            "value": 54684934.86482701
          },
          {
            "t": 18.143722,
            "value": 55102365.003539614
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.027182,
            "value": 72313891.53113054
          },
          {
            "t": 2.048517,
            "value": 68633637.67015363
          },
          {
            "t": 4.100878,
            "value": 67684956.00920111
          },
          {
            "t": 6.116806,
            "value": 68733520.24477065
          },
          {
            "t": 8.138724,
            "value": 68705865.42085288
          },
          {
            "t": 10.155083,
            "value": 68817123.33964339
          },
          {
            "t": 12.077381,
            "value": 72250235.39534453
          },
          {
            "t": 14.099082,
            "value": 68710201.95370136
          },
          {
            "t": 16.126558,
            "value": 68349800.93475829
          },
          {
            "t": 18.143722,
            "value": 68869170.77639696
          }
        ],
        "ram_mib": [
          {
            "t": 0.027182,
            "value": 53.38671875
          },
          {
            "t": 2.048517,
            "value": 53.0078125
          },
          {
            "t": 4.100878,
            "value": 53.04296875
          },
          {
            "t": 6.116806,
            "value": 53.0703125
          },
          {
            "t": 8.138724,
            "value": 53.765625
          },
          {
            "t": 10.155083,
            "value": 53.10546875
          },
          {
            "t": 12.077381,
            "value": 53.015625
          },
          {
            "t": 14.099082,
            "value": 53.10546875
          },
          {
            "t": 16.126558,
            "value": 53.3203125
          },
          {
            "t": 18.143722,
            "value": 53.37109375
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
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-3 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 39.948414709793035
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 40.21499532564662
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 51.700390625
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 51.8671875
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 387362.79839040566
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 395577.79419888294
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-3 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.006459
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 68984436.1278306
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55187940.27706164
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-3 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 174.38905100205798
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.076169,
            "value": 39.884823932689315
          },
          {
            "t": 2.100991,
            "value": 40.21499532564662
          },
          {
            "t": 4.121044,
            "value": 40.16903024633614
          },
          {
            "t": 6.143837,
            "value": 39.8685464753587
          },
          {
            "t": 8.072164,
            "value": 40.00847352024922
          },
          {
            "t": 10.091759,
            "value": 39.830611735330834
          },
          {
            "t": 12.117865,
            "value": 39.82723987538941
          },
          {
            "t": 14.137345,
            "value": 40.01248829222604
          },
          {
            "t": 16.157995,
            "value": 40.01595015576324
          },
          {
            "t": 18.178135,
            "value": 39.65198753894081
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.378642,
            "value": 395905.1530024952
          },
          {
            "t": 1.393003,
            "value": 394336.92738581233
          },
          {
            "t": 2.404224,
            "value": 395561.4054692298
          },
          {
            "t": 3.514048,
            "value": 360417.5076408512
          },
          {
            "t": 4.524186,
            "value": 395985.49901102623
          },
          {
            "t": 5.534132,
            "value": 396060.7794872201
          },
          {
            "t": 6.550918,
            "value": 393396.447236685
          },
          {
            "t": 7.566086,
            "value": 394023.45227587945
          },
          {
            "t": 8.677176,
            "value": 360006.8401299625
          },
          {
            "t": 9.686294,
            "value": 396385.754688748
          },
          {
            "t": 10.701191,
            "value": 394128.66527342185
          },
          {
            "t": 11.712338,
            "value": 395590.3543203906
          },
          {
            "t": 12.822535,
            "value": 360296.41586132906
          },
          {
            "t": 13.832771,
            "value": 395947.08563147625
          },
          {
            "t": 14.842005,
            "value": 396340.1946426696
          },
          {
            "t": 15.85266,
            "value": 395782.9328504781
          },
          {
            "t": 16.963526,
            "value": 360079.4335230352
          },
          {
            "t": 17.973914,
            "value": 395887.5204376932
          },
          {
            "t": 18.983451,
            "value": 396221.2380526915
          },
          {
            "t": 19.998492,
            "value": 394072.7517410626
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.076169,
            "value": 395882.81868566904
          },
          {
            "t": 1.08553,
            "value": 396290.32625591836
          },
          {
            "t": 2.100991,
            "value": 393909.76118235954
          },
          {
            "t": 3.111129,
            "value": 395985.49901102623
          },
          {
            "t": 4.121044,
            "value": 396072.93683131755
          },
          {
            "t": 5.131029,
            "value": 396045.48582404695
          },
          {
            "t": 6.143837,
            "value": 394941.58813911426
          },
          {
            "t": 7.158012,
            "value": 394409.2488968866
          },
          {
            "t": 8.172904,
            "value": 394130.60700054787
          },
          {
            "t": 9.182859,
            "value": 396057.2500754984
          },
          {
            "t": 10.192487,
            "value": 396185.52575800195
          },
          {
            "t": 11.208268,
            "value": 393785.66836749256
          },
          {
            "t": 12.218468,
            "value": 395961.19580281136
          },
          {
            "t": 13.228732,
            "value": 395936.1117490082
          },
          {
            "t": 14.237987,
            "value": 396331.9478228991
          },
          {
            "t": 15.248221,
            "value": 395947.86950350116
          },
          {
            "t": 16.258742,
            "value": 395835.41559255077
          },
          {
            "t": 17.269306,
            "value": 395818.5725990635
          },
          {
            "t": 18.278841,
            "value": 396222.02301059396
          },
          {
            "t": 19.288572,
            "value": 396145.11191594595
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.076169,
            "value": 54875264.68289644
          },
          {
            "t": 2.100991,
            "value": 54758915.10463636
          },
          {
            "t": 4.121044,
            "value": 55024082.53644831
          },
          {
            "t": 6.143837,
            "value": 54813983.43775166
          },
          {
            "t": 8.072164,
            "value": 57641195.7100637
          },
          {
            "t": 10.091759,
            "value": 55036481.57180029
          },
          {
            "t": 12.117865,
            "value": 54793247.73728522
          },
          {
            "t": 14.137345,
            "value": 55041230.910927564
          },
          {
            "t": 16.157995,
            "value": 54871609.13567417
          },
          {
            "t": 18.178135,
            "value": 55023391.943132654
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.076169,
            "value": 68590691.25866376
          },
          {
            "t": 2.100991,
            "value": 68449416.29437058
          },
          {
            "t": 4.121044,
            "value": 68778108.29715854
          },
          {
            "t": 6.143837,
            "value": 68520016.13610488
          },
          {
            "t": 8.072164,
            "value": 72048654.61096588
          },
          {
            "t": 10.091759,
            "value": 68710139.40913896
          },
          {
            "t": 12.117865,
            "value": 68491924.90422516
          },
          {
            "t": 14.137345,
            "value": 68888358.8844653
          },
          {
            "t": 16.157995,
            "value": 68587483.73048277
          },
          {
            "t": 18.178135,
            "value": 68779567.75273001
          }
        ],
        "ram_mib": [
          {
            "t": 0.076169,
            "value": 51.56640625
          },
          {
            "t": 2.100991,
            "value": 51.86328125
          },
          {
            "t": 4.121044,
            "value": 51.7265625
          },
          {
            "t": 6.143837,
            "value": 51.55859375
          },
          {
            "t": 8.072164,
            "value": 51.6796875
          },
          {
            "t": 10.091759,
            "value": 51.76171875
          },
          {
            "t": 12.117865,
            "value": 51.76953125
          },
          {
            "t": 14.137345,
            "value": 51.8671875
          },
          {
            "t": 16.157995,
            "value": 51.45703125
          },
          {
            "t": 18.178135,
            "value": 51.75390625
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
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-4 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 40.024925599552105
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 40.71685589519651
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 55.183203125
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 55.5234375
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 386775.43538318377
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 396894.77880740643
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-4 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.005634
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 69193029.64347954
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55368394.09310875
        },
        {
          "extra": "DFE OTAP Attr Insert Multi Transform (Logs)/transform-4 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 174.33595335114128
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.037473,
            "value": 40.0984
          },
          {
            "t": 2.065004,
            "value": 39.92505315822389
          },
          {
            "t": 4.08793,
            "value": 40.00199625701809
          },
          {
            "t": 6.119982,
            "value": 39.84811013767209
          },
          {
            "t": 8.14808,
            "value": 39.82985950671246
          },
          {
            "t": 10.070375,
            "value": 39.9844340505145
          },
          {
            "t": 12.101805,
            "value": 40.00938240798503
          },
          {
            "t": 14.124771,
            "value": 40.00749531542786
          },
          {
            "t": 16.151255,
            "value": 40.71685589519651
          },
          {
            "t": 18.173986,
            "value": 39.82766926677067
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.343615,
            "value": 393375.12903
          },
          {
            "t": 1.456508,
            "value": 359423.5923848924
          },
          {
            "t": 2.468495,
            "value": 395261.994472261
          },
          {
            "t": 3.479557,
            "value": 395623.6116083881
          },
          {
            "t": 4.496373,
            "value": 393384.84052178566
          },
          {
            "t": 5.511366,
            "value": 394091.38782237907
          },
          {
            "t": 6.528567,
            "value": 393235.9484506995
          },
          {
            "t": 7.64036,
            "value": 359779.2035028103
          },
          {
            "t": 8.652807,
            "value": 395082.4092520398
          },
          {
            "t": 9.664177,
            "value": 396491.8872420577
          },
          {
            "t": 10.6781,
            "value": 393521.0070192708
          },
          {
            "t": 11.69532,
            "value": 393228.6034486148
          },
          {
            "t": 12.808213,
            "value": 359423.5923848924
          },
          {
            "t": 13.819527,
            "value": 395525.0298126991
          },
          {
            "t": 14.831167,
            "value": 395397.57225890626
          },
          {
            "t": 15.845301,
            "value": 394425.1943037113
          },
          {
            "t": 16.958219,
            "value": 359415.51848384156
          },
          {
            "t": 17.969561,
            "value": 396502.86451071943
          },
          {
            "t": 18.980866,
            "value": 394539.7283707685
          },
          {
            "t": 19.99326,
            "value": 395103.0922743517
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.138402,
            "value": 395126.5096302208
          },
          {
            "t": 1.15349,
            "value": 394054.50561921723
          },
          {
            "t": 2.165797,
            "value": 395137.0483460057
          },
          {
            "t": 3.176666,
            "value": 395699.14598231815
          },
          {
            "t": 4.188762,
            "value": 395219.42582521815
          },
          {
            "t": 5.204981,
            "value": 393615.9430201561
          },
          {
            "t": 6.119982,
            "value": 437157.9921770578
          },
          {
            "t": 7.135738,
            "value": 393795.36030306487
          },
          {
            "t": 8.14808,
            "value": 395123.38715572405
          },
          {
            "t": 9.15982,
            "value": 395358.4913119971
          },
          {
            "t": 10.171248,
            "value": 395480.4494239827
          },
          {
            "t": 11.185799,
            "value": 394263.0779527101
          },
          {
            "t": 12.202657,
            "value": 393368.5922714872
          },
          {
            "t": 13.214596,
            "value": 395280.7432068534
          },
          {
            "t": 14.225653,
            "value": 395625.56809358916
          },
          {
            "t": 15.237736,
            "value": 395224.5023382469
          },
          {
            "t": 16.25218,
            "value": 394304.6634412546
          },
          {
            "t": 17.263906,
            "value": 395363.96217948344
          },
          {
            "t": 18.274995,
            "value": 395613.0469226745
          },
          {
            "t": 19.287054,
            "value": 395233.8747049332
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.037473,
            "value": 57656314.14421765
          },
          {
            "t": 2.065004,
            "value": 54816344.60829452
          },
          {
            "t": 4.08793,
            "value": 54805074.92612187
          },
          {
            "t": 6.119982,
            "value": 54694059.9945277
          },
          {
            "t": 8.14808,
            "value": 54797685.31895401
          },
          {
            "t": 10.070375,
            "value": 57745452.180856735
          },
          {
            "t": 12.101805,
            "value": 54642776.27090277
          },
          {
            "t": 14.124771,
            "value": 54873102.16780708
          },
          {
            "t": 16.151255,
            "value": 54844932.898557305
          },
          {
            "t": 18.173986,
            "value": 54808198.420847856
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.037473,
            "value": 72048658.9876921
          },
          {
            "t": 2.065004,
            "value": 68417781.0351605
          },
          {
            "t": 4.08793,
            "value": 68572548.37794363
          },
          {
            "t": 6.119982,
            "value": 68350068.30533864
          },
          {
            "t": 8.14808,
            "value": 68312680.1564816
          },
          {
            "t": 10.070375,
            "value": 72342331.43196023
          },
          {
            "t": 12.101805,
            "value": 68286303.73677656
          },
          {
            "t": 14.124771,
            "value": 68571985.88607027
          },
          {
            "t": 16.151255,
            "value": 68450011.94186582
          },
          {
            "t": 18.173986,
            "value": 68577926.57550609
          }
        ],
        "ram_mib": [
          {
            "t": 0.037473,
            "value": 55.02734375
          },
          {
            "t": 2.065004,
            "value": 55.2109375
          },
          {
            "t": 4.08793,
            "value": 55.13671875
          },
          {
            "t": 6.119982,
            "value": 55.25
          },
          {
            "t": 8.14808,
            "value": 55.32421875
          },
          {
            "t": 10.070375,
            "value": 54.86328125
          },
          {
            "t": 12.101805,
            "value": 55.2578125
          },
          {
            "t": 14.124771,
            "value": 55.1171875
          },
          {
            "t": 16.151255,
            "value": 55.5234375
          },
          {
            "t": 18.173986,
            "value": 55.12109375
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
