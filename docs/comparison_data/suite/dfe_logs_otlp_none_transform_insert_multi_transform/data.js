window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otlp_none_transform_insert_multi_transform"] = {
  "name": "DFE OTLP Transform Insert Multi Transform (Logs)",
  "slug": "dfe_logs_otlp_none_transform_insert_multi_transform",
  "description": "Dataflow Engine OTLP logs, transform processor (OPL) insert sweep over 1-4 insert actions at 400k signals/sec",
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
    "started_at": "2026-05-27T18:39:11Z",
    "ended_at": "2026-05-27T18:44:59Z",
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
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-1 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 18.026315689086914
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.27710640475634
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.55829787234043
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 106.0484375
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 115.5859375
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 389454.43480476394
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 319250.1485307473
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-1 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000616
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 342477572.8608729
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 344774721.95939225
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-1 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 1072.756189580562
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.127175,
            "value": 100.55829787234043
          },
          {
            "t": 2.039862,
            "value": 100.05509339975094
          },
          {
            "t": 4.058727,
            "value": 100.52624335314357
          },
          {
            "t": 6.114428,
            "value": 100.2212147777082
          },
          {
            "t": 8.1325,
            "value": 100.28309226932667
          },
          {
            "t": 10.144826,
            "value": 100.02982882041707
          },
          {
            "t": 12.075321,
            "value": 100.40492346141832
          },
          {
            "t": 14.093311,
            "value": 100.43230000000001
          },
          {
            "t": 16.105629,
            "value": 99.72157009345794
          },
          {
            "t": 18.133316,
            "value": 100.5385
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.32789,
            "value": 795056.3396798705
          },
          {
            "t": 1.334245,
            "value": 397474.0523970169
          },
          {
            "t": 2.342259,
            "value": 396819.8854380991
          },
          {
            "t": 3.353528,
            "value": 395542.63010138745
          },
          {
            "t": 4.364867,
            "value": 395515.2525513206
          },
          {
            "t": 5.409023,
            "value": 383084.5199376338
          },
          {
            "t": 6.420569,
            "value": 395434.31539445557
          },
          {
            "t": 7.527302,
            "value": 361424.1194579
          },
          {
            "t": 8.534026,
            "value": 397328.36407992657
          },
          {
            "t": 9.539948,
            "value": 397645.1454486531
          },
          {
            "t": 10.556488,
            "value": 393491.6481397682
          },
          {
            "t": 11.570424,
            "value": 394502.21710246016
          },
          {
            "t": 12.582129,
            "value": 395372.1687646102
          },
          {
            "t": 13.689381,
            "value": 361254.7098582798
          },
          {
            "t": 14.695414,
            "value": 397601.2715288664
          },
          {
            "t": 15.701409,
            "value": 397616.2903394152
          },
          {
            "t": 16.717278,
            "value": 393751.55654912203
          },
          {
            "t": 17.728812,
            "value": 395439.0064990401
          },
          {
            "t": 18.835709,
            "value": 361370.57016145135
          },
          {
            "t": 19.842368,
            "value": 397354.0195835929
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.32789,
            "value": 313053.433748949
          },
          {
            "t": 1.334245,
            "value": 313010.8162626508
          },
          {
            "t": 2.342259,
            "value": 307535.4112145268
          },
          {
            "t": 3.455551,
            "value": 278453.4515652677
          },
          {
            "t": 4.466629,
            "value": 311548.6639013014
          },
          {
            "t": 5.51116,
            "value": 593567.830921246
          },
          {
            "t": 6.521604,
            "value": 306795.82440986345
          },
          {
            "t": 7.527302,
            "value": 308243.6278087458
          },
          {
            "t": 8.534026,
            "value": 312896.08671294217
          },
          {
            "t": 9.539948,
            "value": 303204.42340459797
          },
          {
            "t": 10.556488,
            "value": 309874.6729100675
          },
          {
            "t": 11.672907,
            "value": 277673.5257999013
          },
          {
            "t": 12.683253,
            "value": 311774.3822413312
          },
          {
            "t": 13.689381,
            "value": 308111.8903360209
          },
          {
            "t": 14.695414,
            "value": 313111.00132898224
          },
          {
            "t": 15.701409,
            "value": 308152.6250130468
          },
          {
            "t": 16.717278,
            "value": 305157.45632556954
          },
          {
            "t": 17.830654,
            "value": 282923.2891673613
          },
          {
            "t": 18.835709,
            "value": 308440.8315962808
          },
          {
            "t": 19.842368,
            "value": 307949.36517728446
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.127175,
            "value": 341985432.38623697
          },
          {
            "t": 2.039862,
            "value": 361372009.1159714
          },
          {
            "t": 4.058727,
            "value": 341959720.93230605
          },
          {
            "t": 6.114428,
            "value": 334977368.79050016
          },
          {
            "t": 8.1325,
            "value": 342502240.75255984
          },
          {
            "t": 10.144826,
            "value": 343066459.41065216
          },
          {
            "t": 12.075321,
            "value": 357157841.38265055
          },
          {
            "t": 14.093311,
            "value": 342073380.93845856
          },
          {
            "t": 16.105629,
            "value": 343072151.6181836
          },
          {
            "t": 18.133316,
            "value": 339580614.2664031
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.127175,
            "value": 341099751.78973967
          },
          {
            "t": 2.039862,
            "value": 359963060.3439036
          },
          {
            "t": 4.058727,
            "value": 339361258.42986035
          },
          {
            "t": 6.114428,
            "value": 329995568.90812427
          },
          {
            "t": 8.1325,
            "value": 340119607.7246005
          },
          {
            "t": 10.144826,
            "value": 339831495.493275
          },
          {
            "t": 12.075321,
            "value": 354893588.43198246
          },
          {
            "t": 14.093311,
            "value": 342848670.211448
          },
          {
            "t": 16.105629,
            "value": 337740547.96508306
          },
          {
            "t": 18.133316,
            "value": 338922179.3107122
          }
        ],
        "ram_mib": [
          {
            "t": 0.127175,
            "value": 101.1640625
          },
          {
            "t": 2.039862,
            "value": 106.50390625
          },
          {
            "t": 4.058727,
            "value": 101.484375
          },
          {
            "t": 6.114428,
            "value": 115.5859375
          },
          {
            "t": 8.1325,
            "value": 106.875
          },
          {
            "t": 10.144826,
            "value": 107.35546875
          },
          {
            "t": 12.075321,
            "value": 106.25
          },
          {
            "t": 14.093311,
            "value": 100.6875
          },
          {
            "t": 16.105629,
            "value": 106.5390625
          },
          {
            "t": 18.133316,
            "value": 108.0390625
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
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-2 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 22.43421173095703
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.35429424428048
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.49991867375665
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 112.022265625
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 121.3515625
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 389387.729296216
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 308375.17148301256
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-2 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000601
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 340290511.0900749
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 343693329.36513555
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-2 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 1103.495166143168
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.040333,
            "value": 100.49991867375665
          },
          {
            "t": 2.061075,
            "value": 100.47359599749844
          },
          {
            "t": 4.079044,
            "value": 100.30511860174782
          },
          {
            "t": 6.096421,
            "value": 100.48510318949344
          },
          {
            "t": 8.114304,
            "value": 100.23449049548145
          },
          {
            "t": 10.135592,
            "value": 100.22098565190267
          },
          {
            "t": 12.152032,
            "value": 100.39992502343019
          },
          {
            "t": 14.074009,
            "value": 100.4192
          },
          {
            "t": 16.093389,
            "value": 100.0323
          },
          {
            "t": 18.107127,
            "value": 100.47230480949408
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.342369,
            "value": 396666.8088056065
          },
          {
            "t": 1.355451,
            "value": 394834.7715189886
          },
          {
            "t": 2.36422,
            "value": 396522.89077083056
          },
          {
            "t": 3.473673,
            "value": 360538.0309035173
          },
          {
            "t": 4.4817,
            "value": 396814.7678584006
          },
          {
            "t": 5.490302,
            "value": 396588.5453330451
          },
          {
            "t": 6.499286,
            "value": 396438.3974374222
          },
          {
            "t": 7.508193,
            "value": 396468.6537014809
          },
          {
            "t": 8.522225,
            "value": 394464.86895877053
          },
          {
            "t": 9.630743,
            "value": 360842.13337086095
          },
          {
            "t": 10.638663,
            "value": 396856.89340423845
          },
          {
            "t": 11.647335,
            "value": 396561.02281019004
          },
          {
            "t": 12.655308,
            "value": 396836.0263618172
          },
          {
            "t": 13.668944,
            "value": 394618.975648063
          },
          {
            "t": 14.682777,
            "value": 394542.2964137092
          },
          {
            "t": 15.789946,
            "value": 361281.79166866123
          },
          {
            "t": 16.796583,
            "value": 397362.7037353088
          },
          {
            "t": 17.803549,
            "value": 397232.8757872659
          },
          {
            "t": 18.812608,
            "value": 396408.93148963543
          },
          {
            "t": 19.86019,
            "value": 381831.6847750343
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.140877,
            "value": 312342.8989585595
          },
          {
            "t": 1.148782,
            "value": 307568.66966628795
          },
          {
            "t": 2.161603,
            "value": 311012.5086268946
          },
          {
            "t": 3.171619,
            "value": 306925.83087792667
          },
          {
            "t": 4.179581,
            "value": 302590.77227117686
          },
          {
            "t": 5.187945,
            "value": 307428.66663228755
          },
          {
            "t": 6.197054,
            "value": 307201.69971727533
          },
          {
            "t": 7.205276,
            "value": 307471.9654996618
          },
          {
            "t": 8.214893,
            "value": 311999.5008007987
          },
          {
            "t": 9.228217,
            "value": 305923.8703514375
          },
          {
            "t": 10.236138,
            "value": 307563.7872412619
          },
          {
            "t": 11.24455,
            "value": 307414.03315311595
          },
          {
            "t": 12.252676,
            "value": 307501.244884072
          },
          {
            "t": 13.260413,
            "value": 312581.55649737973
          },
          {
            "t": 14.275209,
            "value": 305480.1162006945
          },
          {
            "t": 15.287482,
            "value": 311180.87709540804
          },
          {
            "t": 16.294168,
            "value": 307941.1057668429
          },
          {
            "t": 17.301029,
            "value": 312853.5120537989
          },
          {
            "t": 18.307896,
            "value": 292988.0510534162
          },
          {
            "t": 19.257202,
            "value": 326554.3460169851
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.040333,
            "value": 342757644.82205105
          },
          {
            "t": 2.061075,
            "value": 340372502.27886593
          },
          {
            "t": 4.079044,
            "value": 342573560.8426096
          },
          {
            "t": 6.096421,
            "value": 340998024.1670248
          },
          {
            "t": 8.114304,
            "value": 342475285.2370529
          },
          {
            "t": 10.135592,
            "value": 341986727.7696201
          },
          {
            "t": 12.152032,
            "value": 342799324.0562576
          },
          {
            "t": 14.074009,
            "value": 357859775.1169759
          },
          {
            "t": 16.093389,
            "value": 342291904.44591904
          },
          {
            "t": 18.107127,
            "value": 342818544.914979
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.040333,
            "value": 341288168.81810176
          },
          {
            "t": 2.061075,
            "value": 341122358.5197913
          },
          {
            "t": 4.079044,
            "value": 334709886.52452046
          },
          {
            "t": 6.096421,
            "value": 338982273.51655143
          },
          {
            "t": 8.114304,
            "value": 340146118.4815968
          },
          {
            "t": 10.135592,
            "value": 336237402.58686537
          },
          {
            "t": 12.152032,
            "value": 339766868.3422269
          },
          {
            "t": 14.074009,
            "value": 359315179.1098437
          },
          {
            "t": 16.093389,
            "value": 339273003.0999614
          },
          {
            "t": 18.107127,
            "value": 332063851.90129006
          }
        ],
        "ram_mib": [
          {
            "t": 0.040333,
            "value": 105.73828125
          },
          {
            "t": 2.061075,
            "value": 119.76171875
          },
          {
            "t": 4.079044,
            "value": 111.265625
          },
          {
            "t": 6.096421,
            "value": 102.90234375
          },
          {
            "t": 8.114304,
            "value": 105.296875
          },
          {
            "t": 10.135592,
            "value": 121.3515625
          },
          {
            "t": 12.152032,
            "value": 118.203125
          },
          {
            "t": 14.074009,
            "value": 106.44921875
          },
          {
            "t": 16.093389,
            "value": 108.89453125
          },
          {
            "t": 18.107127,
            "value": 120.359375
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
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-3 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 22.105262756347656
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 99.82041593990341
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.07384615384616
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 111.028515625
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 116.52734375
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 387895.29848441144
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 310019.64644772996
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-3 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000607
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 340888570.7903344
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 343380040.11020327
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-3 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 1099.5708649316489
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.05242,
            "value": 100.07057071960297
          },
          {
            "t": 2.07481,
            "value": 99.80303105590063
          },
          {
            "t": 4.092163,
            "value": 99.74088144009932
          },
          {
            "t": 6.113216,
            "value": 99.46172778123058
          },
          {
            "t": 8.135887,
            "value": 100.03293532338309
          },
          {
            "t": 10.153267,
            "value": 100.07384615384616
          },
          {
            "t": 12.07383,
            "value": 99.89510724277278
          },
          {
            "t": 14.098605,
            "value": 100.05240348692404
          },
          {
            "t": 16.115748,
            "value": 99.49712154180914
          },
          {
            "t": 18.134687,
            "value": 99.57653465346534
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.259223,
            "value": 789435.7705189158
          },
          {
            "t": 1.368998,
            "value": 360433.42118897976
          },
          {
            "t": 2.377234,
            "value": 396732.51103908214
          },
          {
            "t": 3.385686,
            "value": 396647.5350338936
          },
          {
            "t": 4.394635,
            "value": 397443.2800865058
          },
          {
            "t": 5.407112,
            "value": 395070.7028406571
          },
          {
            "t": 6.420743,
            "value": 393634.36990384076
          },
          {
            "t": 7.530307,
            "value": 360501.9629331882
          },
          {
            "t": 8.538762,
            "value": 396646.35506790085
          },
          {
            "t": 9.547698,
            "value": 396457.25794302113
          },
          {
            "t": 10.556084,
            "value": 397665.17980217887
          },
          {
            "t": 11.568084,
            "value": 394268.7747035573
          },
          {
            "t": 12.585459,
            "value": 393168.69394274487
          },
          {
            "t": 13.69419,
            "value": 360772.81143938436
          },
          {
            "t": 14.702522,
            "value": 396694.7394310604
          },
          {
            "t": 15.711358,
            "value": 396496.5564274074
          },
          {
            "t": 16.719418,
            "value": 396801.77767196397
          },
          {
            "t": 17.730082,
            "value": 396768.85690991266
          },
          {
            "t": 18.743468,
            "value": 393729.5364254095
          },
          {
            "t": 19.852139,
            "value": 360792.33604919765
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.153046,
            "value": 306795.2171615307
          },
          {
            "t": 1.167239,
            "value": 310591.77099427825
          },
          {
            "t": 2.175482,
            "value": 307465.56137756474
          },
          {
            "t": 3.183995,
            "value": 312341.04072034766
          },
          {
            "t": 4.192867,
            "value": 307273.8662585541
          },
          {
            "t": 5.201089,
            "value": 312431.19074965635
          },
          {
            "t": 6.213894,
            "value": 306080.6374376114
          },
          {
            "t": 7.228122,
            "value": 310581.0527810315
          },
          {
            "t": 8.236558,
            "value": 312364.88978973374
          },
          {
            "t": 9.244744,
            "value": 307482.9446153785
          },
          {
            "t": 10.253946,
            "value": 312127.79998454225
          },
          {
            "t": 11.262393,
            "value": 312361.4825568423
          },
          {
            "t": 12.177828,
            "value": 327713.054449524
          },
          {
            "t": 13.191148,
            "value": 305925.07796155213
          },
          {
            "t": 14.199285,
            "value": 307497.88967174105
          },
          {
            "t": 15.208293,
            "value": 312187.81218781223
          },
          {
            "t": 16.216333,
            "value": 312487.59969842463
          },
          {
            "t": 17.224471,
            "value": 307497.58465606894
          },
          {
            "t": 18.235288,
            "value": 301736.1203857869
          },
          {
            "t": 19.24861,
            "value": 305924.47415530303
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.05242,
            "value": 342351165.9339184
          },
          {
            "t": 2.07481,
            "value": 341343397.1686964
          },
          {
            "t": 4.092163,
            "value": 341337171.53120947
          },
          {
            "t": 6.113216,
            "value": 341565256.82404166
          },
          {
            "t": 8.135887,
            "value": 341282861.6220829
          },
          {
            "t": 10.153267,
            "value": 341750645.39154744
          },
          {
            "t": 12.07383,
            "value": 359476678.4531411
          },
          {
            "t": 14.098605,
            "value": 340063379.88171524
          },
          {
            "t": 16.115748,
            "value": 342665485.7885634
          },
          {
            "t": 18.134687,
            "value": 341964358.50711685
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.05242,
            "value": 337234880.5984416
          },
          {
            "t": 2.07481,
            "value": 338910417.37745935
          },
          {
            "t": 4.092163,
            "value": 342193544.7093295
          },
          {
            "t": 6.113216,
            "value": 340040268.612451
          },
          {
            "t": 8.135887,
            "value": 340396351.16140985
          },
          {
            "t": 10.153267,
            "value": 342330613.9646472
          },
          {
            "t": 12.07383,
            "value": 351027031.1361825
          },
          {
            "t": 14.098605,
            "value": 338371541.0354237
          },
          {
            "t": 16.115748,
            "value": 341742482.31285536
          },
          {
            "t": 18.134687,
            "value": 336638576.9951445
          }
        ],
        "ram_mib": [
          {
            "t": 0.05242,
            "value": 106.4140625
          },
          {
            "t": 2.07481,
            "value": 107.265625
          },
          {
            "t": 4.092163,
            "value": 112.5234375
          },
          {
            "t": 6.113216,
            "value": 109.7734375
          },
          {
            "t": 8.135887,
            "value": 116.52734375
          },
          {
            "t": 10.153267,
            "value": 110.3671875
          },
          {
            "t": 12.07383,
            "value": 111.56640625
          },
          {
            "t": 14.098605,
            "value": 109.5390625
          },
          {
            "t": 16.115748,
            "value": 112.99609375
          },
          {
            "t": 18.134687,
            "value": 113.3125
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
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-4 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 18.026315689086914
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.22573487072495
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.42450000000001
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 106.912109375
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 111.03125
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 389707.9764300514
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 317786.0139161709
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-4 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000605
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 341057530.7249712
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 343327199.37861043
        },
        {
          "extra": "DFE OTLP Transform Insert Multi Transform (Logs)/transform-4 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 1073.2301479288483
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.088702,
            "value": 99.9076154806492
          },
          {
            "t": 2.107987,
            "value": 100.39332708528586
          },
          {
            "t": 4.131355,
            "value": 100.20420330526971
          },
          {
            "t": 6.150414,
            "value": 100.29872659176029
          },
          {
            "t": 8.069563,
            "value": 100.30059245400686
          },
          {
            "t": 10.092952,
            "value": 100.26853042121684
          },
          {
            "t": 12.110996,
            "value": 100.25894344313238
          },
          {
            "t": 14.132087,
            "value": 99.99565135895033
          },
          {
            "t": 16.15611,
            "value": 100.42450000000001
          },
          {
            "t": 18.174475,
            "value": 100.20525856697819
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.290001,
            "value": 792920.8030701893
          },
          {
            "t": 1.299276,
            "value": 396324.09402789135
          },
          {
            "t": 2.314311,
            "value": 394075.08115483704
          },
          {
            "t": 3.424257,
            "value": 360377.8922578215
          },
          {
            "t": 4.433203,
            "value": 396453.328523033
          },
          {
            "t": 5.443324,
            "value": 395992.16331508796
          },
          {
            "t": 6.452459,
            "value": 396379.07713041373
          },
          {
            "t": 7.461899,
            "value": 396259.3120938342
          },
          {
            "t": 8.477088,
            "value": 394015.3015842371
          },
          {
            "t": 9.587026,
            "value": 360380.48972104746
          },
          {
            "t": 10.595957,
            "value": 396459.22268222505
          },
          {
            "t": 11.605091,
            "value": 396379.4699217349
          },
          {
            "t": 12.614188,
            "value": 396394.0037479053
          },
          {
            "t": 13.625703,
            "value": 395446.4343089327
          },
          {
            "t": 14.640199,
            "value": 394284.452575466
          },
          {
            "t": 15.750068,
            "value": 361303.901631634
          },
          {
            "t": 16.759909,
            "value": 395111.70570416533
          },
          {
            "t": 17.769108,
            "value": 396353.9401049744
          },
          {
            "t": 18.780043,
            "value": 395673.312329675
          },
          {
            "t": 19.791783,
            "value": 395358.4913119971
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.290001,
            "value": 312212.56620888703
          },
          {
            "t": 1.402555,
            "value": 278638.1604847944
          },
          {
            "t": 2.415987,
            "value": 310824.99861855555
          },
          {
            "t": 3.424257,
            "value": 307457.3278982812
          },
          {
            "t": 4.433203,
            "value": 307251.32960535056
          },
          {
            "t": 5.443324,
            "value": 618737.755179825
          },
          {
            "t": 6.452459,
            "value": 302239.0463119405
          },
          {
            "t": 7.565178,
            "value": 278596.84250920493
          },
          {
            "t": 8.578731,
            "value": 305854.75056558463
          },
          {
            "t": 9.587026,
            "value": 312408.57090434845
          },
          {
            "t": 10.595957,
            "value": 307255.8975787244
          },
          {
            "t": 11.605091,
            "value": 307194.08918934455
          },
          {
            "t": 12.614188,
            "value": 312160.27795147547
          },
          {
            "t": 13.728388,
            "value": 278226.53024591634
          },
          {
            "t": 14.741638,
            "value": 305946.212681964
          },
          {
            "t": 15.750068,
            "value": 312366.74831173214
          },
          {
            "t": 16.759909,
            "value": 306979.0194694016
          },
          {
            "t": 17.769108,
            "value": 312128.7278326673
          },
          {
            "t": 18.780043,
            "value": 306646.81705549813
          },
          {
            "t": 19.894388,
            "value": 282677.2678120331
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.088702,
            "value": 341106644.33775306
          },
          {
            "t": 2.107987,
            "value": 342311575.63196874
          },
          {
            "t": 4.131355,
            "value": 341181475.1444127
          },
          {
            "t": 6.150414,
            "value": 341491490.3427785
          },
          {
            "t": 8.069563,
            "value": 359730527.9579647
          },
          {
            "t": 10.092952,
            "value": 341158519.1972478
          },
          {
            "t": 12.110996,
            "value": 342070132.76221925
          },
          {
            "t": 14.132087,
            "value": 341124314.54100776
          },
          {
            "t": 16.15611,
            "value": 341076544.5847206
          },
          {
            "t": 18.174475,
            "value": 342020769.286031
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.088702,
            "value": 339357929.31521857
          },
          {
            "t": 2.107987,
            "value": 339906154.4061388
          },
          {
            "t": 4.131355,
            "value": 339040826.9775938
          },
          {
            "t": 6.150414,
            "value": 340373223.86319566
          },
          {
            "t": 8.069563,
            "value": 354141527.83343035
          },
          {
            "t": 10.092952,
            "value": 337563355.3409651
          },
          {
            "t": 12.110996,
            "value": 340545104.0710708
          },
          {
            "t": 14.132087,
            "value": 339615191.0032749
          },
          {
            "t": 16.15611,
            "value": 338498328.82333845
          },
          {
            "t": 18.174475,
            "value": 341533665.6154858
          }
        ],
        "ram_mib": [
          {
            "t": 0.088702,
            "value": 111.03125
          },
          {
            "t": 2.107987,
            "value": 108.55859375
          },
          {
            "t": 4.131355,
            "value": 107.73046875
          },
          {
            "t": 6.150414,
            "value": 100.67578125
          },
          {
            "t": 8.069563,
            "value": 103.953125
          },
          {
            "t": 10.092952,
            "value": 108.953125
          },
          {
            "t": 12.110996,
            "value": 106.7890625
          },
          {
            "t": 14.132087,
            "value": 107.07421875
          },
          {
            "t": 16.15611,
            "value": 107.65625
          },
          {
            "t": 18.174475,
            "value": 106.69921875
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
