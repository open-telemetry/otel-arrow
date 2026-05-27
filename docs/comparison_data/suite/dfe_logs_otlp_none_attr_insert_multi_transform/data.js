window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otlp_none_attr_insert_multi_transform"] = {
  "name": "DFE OTLP Attr Insert Multi Transform (Logs)",
  "slug": "dfe_logs_otlp_none_attr_insert_multi_transform",
  "description": "Dataflow Engine OTLP logs, attributes processor insert sweep over 1-4 insert actions at 400k signals/sec",
  "meta": {
    "binary": "dfe",
    "protocols": [
      "otlp"
    ],
    "signals": [
      "logs"
    ],
    "compression": "none"
  },
  "env": {
    "started_at": "2026-05-27T18:26:45Z",
    "ended_at": "2026-05-27T18:32:32Z",
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
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-1 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 21.973684310913086
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.36621639207486
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.57645070422537
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 109.242578125
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 116.171875
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 388175.81729013746
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 310792.42687428923
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-1 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000612
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 342114872.465608
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 343616522.8943347
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-1 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 1100.7825251932159
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.071093,
            "value": 99.99695095282725
          },
          {
            "t": 2.086481,
            "value": 100.27991263650546
          },
          {
            "t": 4.131961,
            "value": 100.57645070422537
          },
          {
            "t": 6.143251,
            "value": 100.46529540481401
          },
          {
            "t": 8.059834,
            "value": 100.57633010315723
          },
          {
            "t": 10.071743,
            "value": 100.36317301686447
          },
          {
            "t": 12.089574,
            "value": 100.21248518864982
          },
          {
            "t": 14.111754,
            "value": 100.24424204616345
          },
          {
            "t": 16.123721,
            "value": 100.4036238675414
          },
          {
            "t": 18.141909,
            "value": 100.54369999999999
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.275221,
            "value": 495732.24113605963
          },
          {
            "t": 1.382357,
            "value": 361292.5602635991
          },
          {
            "t": 2.388062,
            "value": 397730.94495900883
          },
          {
            "t": 3.394583,
            "value": 397408.49917686766
          },
          {
            "t": 4.43344,
            "value": 385038.55679848144
          },
          {
            "t": 5.439059,
            "value": 397764.9586970811
          },
          {
            "t": 6.448039,
            "value": 397431.0690003766
          },
          {
            "t": 7.45575,
            "value": 395946.85381026904
          },
          {
            "t": 8.561945,
            "value": 362503.89849890844
          },
          {
            "t": 9.5682,
            "value": 396519.7688458691
          },
          {
            "t": 10.575642,
            "value": 397045.1896982655
          },
          {
            "t": 11.585237,
            "value": 396198.475626365
          },
          {
            "t": 12.597029,
            "value": 395338.17227256193
          },
          {
            "t": 13.608143,
            "value": 395603.26530935185
          },
          {
            "t": 14.714778,
            "value": 361456.1260036055
          },
          {
            "t": 15.720407,
            "value": 397761.00331235473
          },
          {
            "t": 16.727103,
            "value": 398332.7638135048
          },
          {
            "t": 17.738498,
            "value": 394504.61985673255
          },
          {
            "t": 18.746741,
            "value": 396729.75661621254
          },
          {
            "t": 19.853978,
            "value": 361259.60386078135
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.17154,
            "value": 313190.07455912285
          },
          {
            "t": 1.181214,
            "value": 311981.88722300465
          },
          {
            "t": 2.186895,
            "value": 308248.8383493374
          },
          {
            "t": 3.192614,
            "value": 318180.32671153673
          },
          {
            "t": 4.131961,
            "value": 330016.4901788157
          },
          {
            "t": 5.137582,
            "value": 308267.229900728
          },
          {
            "t": 6.143251,
            "value": 303280.7017020511
          },
          {
            "t": 7.152352,
            "value": 307204.1351658555
          },
          {
            "t": 8.160276,
            "value": 312523.56328453333
          },
          {
            "t": 9.166615,
            "value": 313015.7928888774
          },
          {
            "t": 10.17396,
            "value": 317666.7378107798
          },
          {
            "t": 11.183169,
            "value": 312125.635027036
          },
          {
            "t": 12.192048,
            "value": 302315.73855734925
          },
          {
            "t": 13.201358,
            "value": 307140.5217425766
          },
          {
            "t": 14.212167,
            "value": 311631.5743132481
          },
          {
            "t": 15.218423,
            "value": 313041.6116773465
          },
          {
            "t": 16.224987,
            "value": 307978.4295881832
          },
          {
            "t": 17.233954,
            "value": 307244.93467080686
          },
          {
            "t": 18.242849,
            "value": 302310.94415177
          },
          {
            "t": 19.251799,
            "value": 312205.75846176717
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.071093,
            "value": 342192378.98213124
          },
          {
            "t": 2.086481,
            "value": 342669955.8596161
          },
          {
            "t": 4.131961,
            "value": 337942195.96378356
          },
          {
            "t": 6.143251,
            "value": 342437451.5857982
          },
          {
            "t": 8.059834,
            "value": 360227528.8886524
          },
          {
            "t": 10.071743,
            "value": 343578356.67517763
          },
          {
            "t": 12.089574,
            "value": 341319499.99776983
          },
          {
            "t": 14.111754,
            "value": 341405749.735434
          },
          {
            "t": 16.123721,
            "value": 343573095.88079727
          },
          {
            "t": 18.141909,
            "value": 340819015.3741872
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.071093,
            "value": 344289167.4300674
          },
          {
            "t": 2.086481,
            "value": 342667982.54231936
          },
          {
            "t": 4.131961,
            "value": 334948371.5313765
          },
          {
            "t": 6.143251,
            "value": 337918165.4560009
          },
          {
            "t": 8.059834,
            "value": 360334035.10309756
          },
          {
            "t": 10.071743,
            "value": 345983766.66141456
          },
          {
            "t": 12.089574,
            "value": 336195665.5438438
          },
          {
            "t": 14.111754,
            "value": 342145113.68918693
          },
          {
            "t": 16.123721,
            "value": 340529964.9546936
          },
          {
            "t": 18.141909,
            "value": 336136491.74407935
          }
        ],
        "ram_mib": [
          {
            "t": 0.071093,
            "value": 112.578125
          },
          {
            "t": 2.086481,
            "value": 110.40625
          },
          {
            "t": 4.131961,
            "value": 116.171875
          },
          {
            "t": 6.143251,
            "value": 109.3203125
          },
          {
            "t": 8.059834,
            "value": 105.36328125
          },
          {
            "t": 10.071743,
            "value": 112.8046875
          },
          {
            "t": 12.089574,
            "value": 105.55859375
          },
          {
            "t": 14.111754,
            "value": 104.55859375
          },
          {
            "t": 16.123721,
            "value": 108.41015625
          },
          {
            "t": 18.141909,
            "value": 107.25390625
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
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-2 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 21.38157844543457
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 99.955059880835
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.34023103340618
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 119.032421875
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 132.40234375
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 388146.0403562588
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 311627.3944001678
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-2 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000714
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 344558234.9081821
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 343700741.4477225
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-2 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 1105.6737664909108
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.086218,
            "value": 100.21897100093547
          },
          {
            "t": 2.103633,
            "value": 99.72416199376947
          },
          {
            "t": 4.123516,
            "value": 99.9347263681592
          },
          {
            "t": 6.137614,
            "value": 100.18014962593516
          },
          {
            "t": 8.155375,
            "value": 99.75481942714819
          },
          {
            "t": 10.074222,
            "value": 100.34023103340618
          },
          {
            "t": 12.089226,
            "value": 99.75723278279838
          },
          {
            "t": 14.105671,
            "value": 99.87380981976382
          },
          {
            "t": 16.133088,
            "value": 100.06937167864957
          },
          {
            "t": 18.148483,
            "value": 99.69712507778469
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.287716,
            "value": 794503.6239296547
          },
          {
            "t": 1.297955,
            "value": 395945.90982925816
          },
          {
            "t": 2.310255,
            "value": 395139.7806974217
          },
          {
            "t": 3.417678,
            "value": 361198.92760038393
          },
          {
            "t": 4.425502,
            "value": 396894.695899284
          },
          {
            "t": 5.432427,
            "value": 397249.050326489
          },
          {
            "t": 6.441321,
            "value": 396473.7623575916
          },
          {
            "t": 7.449988,
            "value": 396562.9885779945
          },
          {
            "t": 8.557611,
            "value": 361133.70704653114
          },
          {
            "t": 9.569666,
            "value": 395235.43680926436
          },
          {
            "t": 10.677778,
            "value": 360974.34194377466
          },
          {
            "t": 11.685046,
            "value": 398106.5615109385
          },
          {
            "t": 12.693081,
            "value": 395819.58959758346
          },
          {
            "t": 13.702288,
            "value": 396350.79820096376
          },
          {
            "t": 14.714063,
            "value": 396333.1768426774
          },
          {
            "t": 15.729596,
            "value": 392897.12889684533
          },
          {
            "t": 16.837105,
            "value": 361170.87987546826
          },
          {
            "t": 17.845378,
            "value": 396717.9523799606
          },
          {
            "t": 18.856114,
            "value": 395751.2149562299
          },
          {
            "t": 19.867975,
            "value": 395311.2136943711
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.086218,
            "value": 312835.4912361372
          },
          {
            "t": 1.093086,
            "value": 312851.3370173647
          },
          {
            "t": 2.103633,
            "value": 316660.18502850435
          },
          {
            "t": 3.11579,
            "value": 316156.48560450604
          },
          {
            "t": 4.123516,
            "value": 307623.3023659209
          },
          {
            "t": 5.130591,
            "value": 307822.1582305191
          },
          {
            "t": 6.137614,
            "value": 302872.9234585506
          },
          {
            "t": 7.147105,
            "value": 307085.4519753024
          },
          {
            "t": 8.155375,
            "value": 317375.3062175806
          },
          {
            "t": 9.162483,
            "value": 312776.7826290726
          },
          {
            "t": 10.174679,
            "value": 316144.3040675916
          },
          {
            "t": 11.181665,
            "value": 312814.67666879186
          },
          {
            "t": 12.189782,
            "value": 312463.73188826296
          },
          {
            "t": 13.197094,
            "value": 297822.3231729593
          },
          {
            "t": 14.206165,
            "value": 307213.26844196295
          },
          {
            "t": 15.217838,
            "value": 316307.73975385324
          },
          {
            "t": 16.233519,
            "value": 315059.5511779781
          },
          {
            "t": 17.241302,
            "value": 312567.288791337
          },
          {
            "t": 18.249059,
            "value": 317536.86652635504
          },
          {
            "t": 19.259757,
            "value": 311665.7992793099
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.086218,
            "value": 341873225.3001391
          },
          {
            "t": 2.103633,
            "value": 342216983.6151709
          },
          {
            "t": 4.123516,
            "value": 342228496.8980876
          },
          {
            "t": 6.137614,
            "value": 342750722.1594977
          },
          {
            "t": 8.155375,
            "value": 341319395.1117104
          },
          {
            "t": 10.074222,
            "value": 360229108.41771126
          },
          {
            "t": 12.089226,
            "value": 341366531.2823201
          },
          {
            "t": 14.105671,
            "value": 342792379.6582599
          },
          {
            "t": 16.133088,
            "value": 340086364.5712747
          },
          {
            "t": 18.148483,
            "value": 342144207.46305317
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.086218,
            "value": 342812529.34875304
          },
          {
            "t": 2.103633,
            "value": 347758386.8465338
          },
          {
            "t": 4.123516,
            "value": 339196960.9130826
          },
          {
            "t": 6.137614,
            "value": 336822427.7070927
          },
          {
            "t": 8.155375,
            "value": 345611846.4971818
          },
          {
            "t": 10.074222,
            "value": 362767344.1394754
          },
          {
            "t": 12.089226,
            "value": 339389598.2340482
          },
          {
            "t": 14.105671,
            "value": 340401850.28602314
          },
          {
            "t": 16.133088,
            "value": 343356450.5969912
          },
          {
            "t": 18.148483,
            "value": 347464954.512639
          }
        ],
        "ram_mib": [
          {
            "t": 0.086218,
            "value": 119.7265625
          },
          {
            "t": 2.103633,
            "value": 108.8203125
          },
          {
            "t": 4.123516,
            "value": 132.40234375
          },
          {
            "t": 6.137614,
            "value": 123.19921875
          },
          {
            "t": 8.155375,
            "value": 111.7265625
          },
          {
            "t": 10.074222,
            "value": 111.2734375
          },
          {
            "t": 12.089226,
            "value": 128.72265625
          },
          {
            "t": 14.105671,
            "value": 127.8515625
          },
          {
            "t": 16.133088,
            "value": 118.84765625
          },
          {
            "t": 18.148483,
            "value": 107.75390625
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
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-3 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 20.526315689086914
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.34547353676714
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.56100125156446
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 111.144921875
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 115.234375
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 390012.73853448505
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 314832.1983476209
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-3 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.00062
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 349952829.7029156
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 345307421.9274628
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-3 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 1111.5534927482747
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.121844,
            "value": 100.21707514811351
          },
          {
            "t": 2.038438,
            "value": 100.30991260923845
          },
          {
            "t": 4.055669,
            "value": 100.56100125156446
          },
          {
            "t": 6.078017,
            "value": 100.30711610486891
          },
          {
            "t": 8.094957,
            "value": 100.4050936329588
          },
          {
            "t": 10.113524,
            "value": 100.34103028410865
          },
          {
            "t": 12.136777,
            "value": 100.33663440524508
          },
          {
            "t": 14.154394,
            "value": 100.40432364885974
          },
          {
            "t": 16.174574,
            "value": 100.1211214953271
          },
          {
            "t": 18.096988,
            "value": 100.45142678738684
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.323562,
            "value": 793324.1770501231
          },
          {
            "t": 1.331595,
            "value": 396812.4059430594
          },
          {
            "t": 2.340795,
            "value": 397344.4312326595
          },
          {
            "t": 3.349452,
            "value": 395575.50287164026
          },
          {
            "t": 4.362978,
            "value": 394661.8044332361
          },
          {
            "t": 5.472474,
            "value": 360524.057770375
          },
          {
            "t": 6.480796,
            "value": 396698.6736379847
          },
          {
            "t": 7.489599,
            "value": 396509.5266370144
          },
          {
            "t": 8.497723,
            "value": 396776.5870071539
          },
          {
            "t": 9.506945,
            "value": 396344.9072652003
          },
          {
            "t": 10.521927,
            "value": 394095.65883927007
          },
          {
            "t": 11.631712,
            "value": 360430.1734119672
          },
          {
            "t": 12.640227,
            "value": 397614.3141153081
          },
          {
            "t": 13.649449,
            "value": 395354.0449970373
          },
          {
            "t": 14.657802,
            "value": 396686.47785051464
          },
          {
            "t": 15.669347,
            "value": 395434.7063155866
          },
          {
            "t": 16.683051,
            "value": 394592.50432078796
          },
          {
            "t": 17.792176,
            "value": 360644.6523160149
          },
          {
            "t": 18.801388,
            "value": 396348.834536252
          },
          {
            "t": 19.810105,
            "value": 396543.33177690074
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.121844,
            "value": 312110.481165371
          },
          {
            "t": 1.129741,
            "value": 317492.75967683207
          },
          {
            "t": 2.138969,
            "value": 322028.3226386902
          },
          {
            "t": 3.147439,
            "value": 322270.3699663847
          },
          {
            "t": 4.156209,
            "value": 322174.52937736054
          },
          {
            "t": 5.170345,
            "value": 315539.5331592607
          },
          {
            "t": 6.178555,
            "value": 327312.7622221561
          },
          {
            "t": 7.187459,
            "value": 312219.99318071886
          },
          {
            "t": 8.195497,
            "value": 317448.35016140266
          },
          {
            "t": 9.203388,
            "value": 322455.5036209272
          },
          {
            "t": 10.214056,
            "value": 311675.05056061933
          },
          {
            "t": 11.22892,
            "value": 315313.1848208233
          },
          {
            "t": 12.237347,
            "value": 307409.460476564
          },
          {
            "t": 13.246663,
            "value": 312092.54584292724
          },
          {
            "t": 14.254937,
            "value": 307456.1081610753
          },
          {
            "t": 15.263292,
            "value": 312389.98170287255
          },
          {
            "t": 16.275141,
            "value": 306369.823955946
          },
          {
            "t": 17.288832,
            "value": 310745.58223363926
          },
          {
            "t": 18.297987,
            "value": 307187.69663728564
          },
          {
            "t": 19.306667,
            "value": 312289.3286275132
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.121844,
            "value": 341857820.4231185
          },
          {
            "t": 2.038438,
            "value": 358948767.44892246
          },
          {
            "t": 4.055669,
            "value": 342686613.4815497
          },
          {
            "t": 6.078017,
            "value": 341823183.2503605
          },
          {
            "t": 8.094957,
            "value": 341039914.4248218
          },
          {
            "t": 10.113524,
            "value": 342443365.02082914
          },
          {
            "t": 12.136777,
            "value": 341644411.7468255
          },
          {
            "t": 14.154394,
            "value": 341327942.3200736
          },
          {
            "t": 16.174574,
            "value": 341748207.5854627
          },
          {
            "t": 18.096988,
            "value": 359553993.5726644
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.121844,
            "value": 347510680.81182176
          },
          {
            "t": 2.038438,
            "value": 373974018.4932229
          },
          {
            "t": 4.055669,
            "value": 351135686.49301934
          },
          {
            "t": 6.078017,
            "value": 352331411.3100218
          },
          {
            "t": 8.094957,
            "value": 347842295.75495553
          },
          {
            "t": 10.113524,
            "value": 347560775.5402719
          },
          {
            "t": 12.136777,
            "value": 339672053.8657301
          },
          {
            "t": 14.154394,
            "value": 341250157.9834032
          },
          {
            "t": 16.174574,
            "value": 341857970.5768793
          },
          {
            "t": 18.096988,
            "value": 356393246.19983
          }
        ],
        "ram_mib": [
          {
            "t": 0.121844,
            "value": 115.234375
          },
          {
            "t": 2.038438,
            "value": 108.7109375
          },
          {
            "t": 4.055669,
            "value": 108.890625
          },
          {
            "t": 6.078017,
            "value": 111.08203125
          },
          {
            "t": 8.094957,
            "value": 110.875
          },
          {
            "t": 10.113524,
            "value": 107.6640625
          },
          {
            "t": 12.136777,
            "value": 112.93359375
          },
          {
            "t": 14.154394,
            "value": 113.60546875
          },
          {
            "t": 16.174574,
            "value": 111.46484375
          },
          {
            "t": 18.096988,
            "value": 110.98828125
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
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-4 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 21.842105865478516
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 99.99481828697643
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.41017096673919
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 109.88984375
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 116.078125
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 386300.7388865725
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 311413.31367410097
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-4 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000601
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 342492926.51384556
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 343611606.3211921
        },
        {
          "extra": "DFE OTLP Attr Insert Multi Transform (Logs)/transform-4 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 1099.8018115316352
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.072594,
            "value": 100.09970725630643
          },
          {
            "t": 2.104837,
            "value": 99.85431500465984
          },
          {
            "t": 4.125557,
            "value": 100.28630265210607
          },
          {
            "t": 6.145299,
            "value": 100.03709928415812
          },
          {
            "t": 8.076765,
            "value": 100.41017096673919
          },
          {
            "t": 10.102699,
            "value": 99.66884184308842
          },
          {
            "t": 12.123537,
            "value": 99.88812927284027
          },
          {
            "t": 14.146227,
            "value": 99.67282689912827
          },
          {
            "t": 16.110914,
            "value": 99.97964541213064
          },
          {
            "t": 18.136606,
            "value": 100.05114427860697
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.279306,
            "value": 591394.0340169849
          },
          {
            "t": 1.396448,
            "value": 358056.5407083432
          },
          {
            "t": 2.407669,
            "value": 395561.4054692298
          },
          {
            "t": 3.417729,
            "value": 396016.0782527771
          },
          {
            "t": 4.428315,
            "value": 395809.95580781845
          },
          {
            "t": 5.438074,
            "value": 396134.1270540792
          },
          {
            "t": 6.452747,
            "value": 394215.67342385184
          },
          {
            "t": 7.469944,
            "value": 393237.4948018919
          },
          {
            "t": 8.485418,
            "value": 393904.7183876692
          },
          {
            "t": 9.596831,
            "value": 359902.2145683018
          },
          {
            "t": 10.606873,
            "value": 396023.1356715859
          },
          {
            "t": 11.616947,
            "value": 396010.58932315855
          },
          {
            "t": 12.627549,
            "value": 395803.6892861878
          },
          {
            "t": 13.640385,
            "value": 394930.66992089537
          },
          {
            "t": 14.657231,
            "value": 393373.234491752
          },
          {
            "t": 15.806134,
            "value": 349028.595103329
          },
          {
            "t": 16.916938,
            "value": 359199.28268173325
          },
          {
            "t": 17.932717,
            "value": 393786.4437047822
          },
          {
            "t": 18.942925,
            "value": 395958.0601222718
          },
          {
            "t": 19.953097,
            "value": 395972.1710758168
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.173381,
            "value": 305420.52465334773
          },
          {
            "t": 1.18791,
            "value": 310488.90667492006
          },
          {
            "t": 2.104837,
            "value": 343538.7986175562
          },
          {
            "t": 3.115012,
            "value": 311827.1586606281
          },
          {
            "t": 4.125557,
            "value": 306765.16137331835
          },
          {
            "t": 5.135287,
            "value": 311964.5845919206
          },
          {
            "t": 6.145299,
            "value": 306927.0464113298
          },
          {
            "t": 7.160608,
            "value": 310250.3769788311
          },
          {
            "t": 8.177549,
            "value": 304835.7771001464
          },
          {
            "t": 9.192755,
            "value": 310281.85412615765
          },
          {
            "t": 10.2035,
            "value": 306704.46057116287
          },
          {
            "t": 11.213579,
            "value": 311856.7953595709
          },
          {
            "t": 12.224322,
            "value": 311651.9233870529
          },
          {
            "t": 13.23426,
            "value": 311900.33447597775
          },
          {
            "t": 14.246995,
            "value": 311038.9193619259
          },
          {
            "t": 15.297022,
            "value": 295230.5035965742
          },
          {
            "t": 16.312415,
            "value": 305300.50926094624
          },
          {
            "t": 17.226649,
            "value": 339081.6793074858
          },
          {
            "t": 18.237439,
            "value": 311637.4321075594
          },
          {
            "t": 19.24771,
            "value": 301899.19338474533
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.072594,
            "value": 341289144.8138176
          },
          {
            "t": 2.104837,
            "value": 339333617.09205055
          },
          {
            "t": 4.125557,
            "value": 341261146.5220318
          },
          {
            "t": 6.145299,
            "value": 341452965.2797239
          },
          {
            "t": 8.076765,
            "value": 357503515.98216075
          },
          {
            "t": 10.102699,
            "value": 340835904.8221709
          },
          {
            "t": 12.123537,
            "value": 341232181.40197283
          },
          {
            "t": 14.146227,
            "value": 340912838.3489314
          },
          {
            "t": 16.110914,
            "value": 351871970.9551699
          },
          {
            "t": 18.136606,
            "value": 340422777.9938905
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.072594,
            "value": 340945223.475188
          },
          {
            "t": 2.104837,
            "value": 339832265.13758445
          },
          {
            "t": 4.125557,
            "value": 339057536.91753435
          },
          {
            "t": 6.145299,
            "value": 339219973.64019763
          },
          {
            "t": 8.076765,
            "value": 357560503.7831368
          },
          {
            "t": 10.102699,
            "value": 340263256.3548467
          },
          {
            "t": 12.123537,
            "value": 342372636.9951476
          },
          {
            "t": 14.146227,
            "value": 341433058.9462548
          },
          {
            "t": 16.110914,
            "value": 348725963.98306704
          },
          {
            "t": 18.136606,
            "value": 335518845.905498
          }
        ],
        "ram_mib": [
          {
            "t": 0.072594,
            "value": 106.921875
          },
          {
            "t": 2.104837,
            "value": 112.1875
          },
          {
            "t": 4.125557,
            "value": 106.75
          },
          {
            "t": 6.145299,
            "value": 111.28125
          },
          {
            "t": 8.076765,
            "value": 108.47265625
          },
          {
            "t": 10.102699,
            "value": 108.14453125
          },
          {
            "t": 12.123537,
            "value": 107.6015625
          },
          {
            "t": 14.146227,
            "value": 108.90234375
          },
          {
            "t": 16.110914,
            "value": 112.55859375
          },
          {
            "t": 18.136606,
            "value": 116.078125
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
