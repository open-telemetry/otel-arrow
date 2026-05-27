window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otlp_none_transform_rename_multi_transform"] = {
  "name": "DFE OTLP Transform Rename Multi Transform (Logs)",
  "slug": "dfe_logs_otlp_none_transform_rename_multi_transform",
  "description": "Dataflow Engine OTLP logs, transform processor (OPL) rename sweep over 1-4 rename actions at 400k signals/sec",
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
    "started_at": "2026-05-27T18:33:01Z",
    "ended_at": "2026-05-27T18:38:48Z",
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
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-1 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 97.11633783056924
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 97.5062636335307
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 67.859375
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 68.78125
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 388654.4007337795
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 396790.922091982
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-1 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000603
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 371648264.6350696
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 343844799.3261329
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-1 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 936.6349982898955
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.091131,
            "value": 97.26038581207219
          },
          {
            "t": 2.106837,
            "value": 96.62624727499221
          },
          {
            "t": 4.053773,
            "value": 97.27202740579258
          },
          {
            "t": 6.064467,
            "value": 97.2010480349345
          },
          {
            "t": 8.075755,
            "value": 97.5062636335307
          },
          {
            "t": 10.090272,
            "value": 97.07733333333334
          },
          {
            "t": 12.101527,
            "value": 97.0367601246106
          },
          {
            "t": 14.115046,
            "value": 97.10296388542965
          },
          {
            "t": 16.126279,
            "value": 96.95412021177204
          },
          {
            "t": 18.137619,
            "value": 97.12622858922454
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.292272,
            "value": 795704.7855675066
          },
          {
            "t": 1.299738,
            "value": 397035.7312306321
          },
          {
            "t": 2.408357,
            "value": 360809.2590872067
          },
          {
            "t": 3.416957,
            "value": 397580.8050763435
          },
          {
            "t": 4.455354,
            "value": 385209.1252189673
          },
          {
            "t": 5.460753,
            "value": 396857.36707516125
          },
          {
            "t": 6.466106,
            "value": 397870.2008150371
          },
          {
            "t": 7.472042,
            "value": 397639.61126751604
          },
          {
            "t": 8.48113,
            "value": 396397.53916407685
          },
          {
            "t": 9.586872,
            "value": 361748.03887344425
          },
          {
            "t": 10.592214,
            "value": 397874.5541318278
          },
          {
            "t": 11.597512,
            "value": 397891.9683516728
          },
          {
            "t": 12.603434,
            "value": 397645.1454486531
          },
          {
            "t": 13.611446,
            "value": 396820.67276976863
          },
          {
            "t": 14.717245,
            "value": 361729.39205045404
          },
          {
            "t": 15.723159,
            "value": 397648.3079070378
          },
          {
            "t": 16.728487,
            "value": 397880.09485461464
          },
          {
            "t": 17.7345,
            "value": 397609.17602456425
          },
          {
            "t": 18.741036,
            "value": 397402.57675830764
          },
          {
            "t": 19.84692,
            "value": 361701.58895508025
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.091131,
            "value": 397845.2700175847
          },
          {
            "t": 1.098057,
            "value": 397248.6558098609
          },
          {
            "t": 2.106837,
            "value": 396518.5669818989
          },
          {
            "t": 3.115544,
            "value": 396547.2629812225
          },
          {
            "t": 4.154084,
            "value": 385156.08450324496
          },
          {
            "t": 5.159413,
            "value": 397879.6990835836
          },
          {
            "t": 6.164781,
            "value": 397864.26462747966
          },
          {
            "t": 7.170776,
            "value": 397616.2903394152
          },
          {
            "t": 8.176092,
            "value": 397884.84416840074
          },
          {
            "t": 9.185364,
            "value": 396325.2720772993
          },
          {
            "t": 10.190667,
            "value": 397889.9893862845
          },
          {
            "t": 11.195933,
            "value": 397904.6341963222
          },
          {
            "t": 12.201854,
            "value": 397645.5407532003
          },
          {
            "t": 13.207128,
            "value": 397901.46765956347
          },
          {
            "t": 14.215397,
            "value": 396719.52623754175
          },
          {
            "t": 15.221317,
            "value": 397645.9360585335
          },
          {
            "t": 16.226618,
            "value": 397890.7809700776
          },
          {
            "t": 17.232028,
            "value": 397847.6442446365
          },
          {
            "t": 18.23799,
            "value": 397629.3339112213
          },
          {
            "t": 19.244795,
            "value": 397296.39801153156
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.091131,
            "value": 343646479.5618183
          },
          {
            "t": 2.106837,
            "value": 341514135.49396586
          },
          {
            "t": 4.053773,
            "value": 354454568.1008518
          },
          {
            "t": 6.064467,
            "value": 343653948.8355762
          },
          {
            "t": 8.075755,
            "value": 341826638.94976753
          },
          {
            "t": 10.090272,
            "value": 342978268.2399801
          },
          {
            "t": 12.101527,
            "value": 342257307.50203234
          },
          {
            "t": 14.115046,
            "value": 342734309.9320145
          },
          {
            "t": 16.126279,
            "value": 342700824.32020557
          },
          {
            "t": 18.137619,
            "value": 342681512.3251166
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.091131,
            "value": 370600731.19796205
          },
          {
            "t": 2.106837,
            "value": 370147300.74723196
          },
          {
            "t": 4.053773,
            "value": 382257959.6863996
          },
          {
            "t": 6.064467,
            "value": 371524001.6631074
          },
          {
            "t": 8.075755,
            "value": 370497977.91266096
          },
          {
            "t": 10.090272,
            "value": 369904665.98196983
          },
          {
            "t": 12.101527,
            "value": 370963317.4311562
          },
          {
            "t": 14.115046,
            "value": 369605125.1565046
          },
          {
            "t": 16.126279,
            "value": 370954321.0557902
          },
          {
            "t": 18.137619,
            "value": 370027245.5179134
          }
        ],
        "ram_mib": [
          {
            "t": 0.091131,
            "value": 68.421875
          },
          {
            "t": 2.106837,
            "value": 66.4609375
          },
          {
            "t": 4.053773,
            "value": 67.02734375
          },
          {
            "t": 6.064467,
            "value": 68.53125
          },
          {
            "t": 8.075755,
            "value": 67.5078125
          },
          {
            "t": 10.090272,
            "value": 67.83203125
          },
          {
            "t": 12.101527,
            "value": 67.89453125
          },
          {
            "t": 14.115046,
            "value": 68.0859375
          },
          {
            "t": 16.126279,
            "value": 68.78125
          },
          {
            "t": 18.137619,
            "value": 68.05078125
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
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-2 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 98.20120394279309
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 98.46645383411581
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 64.915625
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 65.7421875
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 389178.7815631673
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 395404.4225048173
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-2 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000599
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 372314015.2473675
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 344472603.0120799
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-2 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 941.6030627296069
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.104861,
            "value": 98.23239048811013
          },
          {
            "t": 2.118687,
            "value": 98.20810031347963
          },
          {
            "t": 4.134919,
            "value": 98.29543071161049
          },
          {
            "t": 6.048355,
            "value": 98.06466437714643
          },
          {
            "t": 8.062574,
            "value": 98.17848465873513
          },
          {
            "t": 10.081628,
            "value": 98.35207004377736
          },
          {
            "t": 12.095243,
            "value": 98.07965010934083
          },
          {
            "t": 14.115851,
            "value": 98.22310527960012
          },
          {
            "t": 16.104308,
            "value": 98.46645383411581
          },
          {
            "t": 18.118407,
            "value": 97.91168961201502
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.306233,
            "value": 794785.4129059243
          },
          {
            "t": 1.313497,
            "value": 397115.3540680497
          },
          {
            "t": 2.32234,
            "value": 396493.8052799098
          },
          {
            "t": 3.42939,
            "value": 361320.6268912876
          },
          {
            "t": 4.436649,
            "value": 397117.32533539046
          },
          {
            "t": 5.443211,
            "value": 397392.3116509465
          },
          {
            "t": 6.451175,
            "value": 396839.56966717064
          },
          {
            "t": 7.457829,
            "value": 397355.99322110676
          },
          {
            "t": 8.469878,
            "value": 395237.77998891356
          },
          {
            "t": 9.577533,
            "value": 361123.2739436015
          },
          {
            "t": 10.58421,
            "value": 397346.9146508761
          },
          {
            "t": 11.590524,
            "value": 397490.2465830745
          },
          {
            "t": 12.597783,
            "value": 397117.32533539046
          },
          {
            "t": 13.611575,
            "value": 394558.252580411
          },
          {
            "t": 14.624037,
            "value": 395076.5559596311
          },
          {
            "t": 15.800971,
            "value": 339866.12673268
          },
          {
            "t": 16.807631,
            "value": 397353.62485844275
          },
          {
            "t": 17.815041,
            "value": 397057.80168948096
          },
          {
            "t": 18.821481,
            "value": 398434.08449584676
          },
          {
            "t": 19.834533,
            "value": 393859.3477926108
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.104861,
            "value": 398354.5275823258
          },
          {
            "t": 1.111998,
            "value": 397165.43032377923
          },
          {
            "t": 2.118687,
            "value": 397342.1781702194
          },
          {
            "t": 3.127637,
            "value": 396451.75677684724
          },
          {
            "t": 4.134919,
            "value": 397108.2576676641
          },
          {
            "t": 5.141434,
            "value": 397410.8681937179
          },
          {
            "t": 6.148832,
            "value": 397062.5313927564
          },
          {
            "t": 7.15613,
            "value": 397101.94996912533
          },
          {
            "t": 8.163063,
            "value": 397245.8942154046
          },
          {
            "t": 9.175471,
            "value": 395097.628624033
          },
          {
            "t": 10.182116,
            "value": 397359.5458180391
          },
          {
            "t": 11.188609,
            "value": 397419.5548304856
          },
          {
            "t": 12.195751,
            "value": 397163.45857883
          },
          {
            "t": 13.202224,
            "value": 397427.4521025403
          },
          {
            "t": 14.216342,
            "value": 394431.41725124686
          },
          {
            "t": 15.298558,
            "value": 369611.98134198715
          },
          {
            "t": 16.305153,
            "value": 397379.2836244965
          },
          {
            "t": 17.311886,
            "value": 397324.8120405311
          },
          {
            "t": 18.3192,
            "value": 397095.6424709673
          },
          {
            "t": 19.325688,
            "value": 397421.5291190754
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.104861,
            "value": 341606843.01556826
          },
          {
            "t": 2.118687,
            "value": 343202976.8212348
          },
          {
            "t": 4.134919,
            "value": 341702130.01281595
          },
          {
            "t": 6.048355,
            "value": 360566961.7379416
          },
          {
            "t": 8.062574,
            "value": 343127807.8500897
          },
          {
            "t": 10.081628,
            "value": 340621693.1295547
          },
          {
            "t": 12.095243,
            "value": 343229515.0761193
          },
          {
            "t": 14.115851,
            "value": 341206473.9919865
          },
          {
            "t": 16.104308,
            "value": 346731279.58009654
          },
          {
            "t": 18.118407,
            "value": 342730348.90539145
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.104861,
            "value": 369628503.491446
          },
          {
            "t": 2.118687,
            "value": 370444068.65339905
          },
          {
            "t": 4.134919,
            "value": 369989012.67314476
          },
          {
            "t": 6.048355,
            "value": 389869926.1433359
          },
          {
            "t": 8.062574,
            "value": 369905856.81100214
          },
          {
            "t": 10.081628,
            "value": 369473950.6719483
          },
          {
            "t": 12.095243,
            "value": 369561461.3518473
          },
          {
            "t": 14.115851,
            "value": 369186578.99008614
          },
          {
            "t": 16.104308,
            "value": 375150085.21682894
          },
          {
            "t": 18.118407,
            "value": 369930708.47063625
          }
        ],
        "ram_mib": [
          {
            "t": 0.104861,
            "value": 64.5
          },
          {
            "t": 2.118687,
            "value": 65.03125
          },
          {
            "t": 4.134919,
            "value": 64.80859375
          },
          {
            "t": 6.048355,
            "value": 64.71875
          },
          {
            "t": 8.062574,
            "value": 65.7421875
          },
          {
            "t": 10.081628,
            "value": 65.4609375
          },
          {
            "t": 12.095243,
            "value": 64.046875
          },
          {
            "t": 14.115851,
            "value": 64.22265625
          },
          {
            "t": 16.104308,
            "value": 65.03515625
          },
          {
            "t": 18.118407,
            "value": 65.58984375
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
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-3 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -0.013157893903553486
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 97.8559741560759
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 98.34799501867994
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 67.612890625
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 71.2890625
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 390124.3110584329
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 396285.5825455413
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-3 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000598
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 371226709.2265954
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 343368140.5070235
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-3 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 936.7656194858764
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.095081,
            "value": 97.88800996574277
          },
          {
            "t": 2.111781,
            "value": 97.67197266231749
          },
          {
            "t": 4.131759,
            "value": 98.05218691588784
          },
          {
            "t": 6.052973,
            "value": 97.60507304942493
          },
          {
            "t": 8.068638,
            "value": 97.86957334163812
          },
          {
            "t": 10.090483,
            "value": 97.86904331567466
          },
          {
            "t": 12.112284,
            "value": 98.34799501867994
          },
          {
            "t": 14.128423,
            "value": 97.74456751711263
          },
          {
            "t": 16.145539,
            "value": 97.81828144458281
          },
          {
            "t": 18.166422,
            "value": 97.69303832969773
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.398144,
            "value": 396447.0416130638
          },
          {
            "t": 1.406102,
            "value": 396841.93190589297
          },
          {
            "t": 2.414079,
            "value": 396834.45157974836
          },
          {
            "t": 3.425709,
            "value": 395401.4807785455
          },
          {
            "t": 4.540295,
            "value": 358877.646049744
          },
          {
            "t": 5.547656,
            "value": 398069.80814226484
          },
          {
            "t": 6.556185,
            "value": 395625.70833362255
          },
          {
            "t": 7.564064,
            "value": 396873.03733880754
          },
          {
            "t": 8.572601,
            "value": 396614.10538235086
          },
          {
            "t": 9.585561,
            "value": 394882.32506713
          },
          {
            "t": 10.70005,
            "value": 358908.8811105358
          },
          {
            "t": 11.707933,
            "value": 396871.4622629809
          },
          {
            "t": 12.7159,
            "value": 396838.38855835557
          },
          {
            "t": 13.723629,
            "value": 396932.11170860415
          },
          {
            "t": 14.732146,
            "value": 396621.97067575454
          },
          {
            "t": 15.74099,
            "value": 396493.41226195527
          },
          {
            "t": 16.855089,
            "value": 359034.5202715378
          },
          {
            "t": 17.862562,
            "value": 397032.9725957916
          },
          {
            "t": 18.870521,
            "value": 396841.5381974862
          },
          {
            "t": 19.879114,
            "value": 396592.08422029496
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.095081,
            "value": 396784.852341477
          },
          {
            "t": 1.103829,
            "value": 396531.14553882636
          },
          {
            "t": 2.111781,
            "value": 396844.2941727384
          },
          {
            "t": 3.120265,
            "value": 396634.94909190433
          },
          {
            "t": 4.131759,
            "value": 395454.6443182066
          },
          {
            "t": 5.14521,
            "value": 394691.01120823796
          },
          {
            "t": 6.15361,
            "value": 397659.6588655296
          },
          {
            "t": 7.161499,
            "value": 395876.9269235005
          },
          {
            "t": 8.169305,
            "value": 396901.78466887475
          },
          {
            "t": 9.177825,
            "value": 397612.3428390116
          },
          {
            "t": 10.19107,
            "value": 393784.32659425907
          },
          {
            "t": 11.204926,
            "value": 394533.34595840034
          },
          {
            "t": 12.212846,
            "value": 396856.89340423845
          },
          {
            "t": 13.220682,
            "value": 396889.97019356326
          },
          {
            "t": 14.229041,
            "value": 396684.11746213405
          },
          {
            "t": 15.237004,
            "value": 396839.9633716714
          },
          {
            "t": 16.246127,
            "value": 396383.7906776478
          },
          {
            "t": 17.259116,
            "value": 396845.3754186867
          },
          {
            "t": 18.267059,
            "value": 394863.59843761
          },
          {
            "t": 19.275693,
            "value": 397567.4030421342
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.095081,
            "value": 341926313.2217506
          },
          {
            "t": 2.111781,
            "value": 341381714.18654233
          },
          {
            "t": 4.131759,
            "value": 341667623.6077819
          },
          {
            "t": 6.052973,
            "value": 359700196.8546971
          },
          {
            "t": 8.068638,
            "value": 341983439.70848334
          },
          {
            "t": 10.090483,
            "value": 340921594.38532627
          },
          {
            "t": 12.112284,
            "value": 341787434.0748669
          },
          {
            "t": 14.128423,
            "value": 341052467.1166026
          },
          {
            "t": 16.145539,
            "value": 342586247.89055264
          },
          {
            "t": 18.166422,
            "value": 340674374.0236322
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.095081,
            "value": 369570587.13801455
          },
          {
            "t": 2.111781,
            "value": 369902964.74438435
          },
          {
            "t": 4.131759,
            "value": 368760331.5481654
          },
          {
            "t": 6.052973,
            "value": 388411130.1499989
          },
          {
            "t": 8.068638,
            "value": 370114188.1215381
          },
          {
            "t": 10.090483,
            "value": 368035515.0864681
          },
          {
            "t": 12.112284,
            "value": 369408875.5520449
          },
          {
            "t": 14.128423,
            "value": 369551185.21094036
          },
          {
            "t": 16.145539,
            "value": 369372736.1242487
          },
          {
            "t": 18.166422,
            "value": 369139578.59015095
          }
        ],
        "ram_mib": [
          {
            "t": 0.095081,
            "value": 68.16796875
          },
          {
            "t": 2.111781,
            "value": 68.0703125
          },
          {
            "t": 4.131759,
            "value": 67.78515625
          },
          {
            "t": 6.052973,
            "value": 68.703125
          },
          {
            "t": 8.068638,
            "value": 67.52734375
          },
          {
            "t": 10.090483,
            "value": 68.0859375
          },
          {
            "t": 12.112284,
            "value": 71.2890625
          },
          {
            "t": 14.128423,
            "value": 66.37890625
          },
          {
            "t": 16.145539,
            "value": 65.05078125
          },
          {
            "t": 18.166422,
            "value": 65.0703125
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
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-4 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -0.03947368636727333
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 99.0035248386836
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 99.4103685196752
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 59.62890625
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 62.1484375
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 389562.8525575852
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 397925.5116729091
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-4 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000834
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 370503044.1159796
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 343089118.4231581
        },
        {
          "extra": "DFE OTLP Transform Rename Multi Transform (Logs)/transform-4 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 931.0864301169248
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.057197,
            "value": 98.55909915545824
          },
          {
            "t": 2.080843,
            "value": 98.60131291028446
          },
          {
            "t": 4.099298,
            "value": 99.4103685196752
          },
          {
            "t": 6.124867,
            "value": 99.07478260869566
          },
          {
            "t": 8.053961,
            "value": 98.9787852222918
          },
          {
            "t": 10.073065,
            "value": 98.9782
          },
          {
            "t": 12.091433,
            "value": 99.03684507042253
          },
          {
            "t": 14.115877,
            "value": 98.97992490613267
          },
          {
            "t": 16.135254,
            "value": 99.10213883677298
          },
          {
            "t": 18.153971,
            "value": 99.31379115710254
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.264339,
            "value": 591394.6169290652
          },
          {
            "t": 1.3743,
            "value": 360373.02211519144
          },
          {
            "t": 2.383431,
            "value": 396380.6483003693
          },
          {
            "t": 3.392655,
            "value": 396344.12182032934
          },
          {
            "t": 4.40208,
            "value": 396265.20048542484
          },
          {
            "t": 5.416839,
            "value": 394182.263966124
          },
          {
            "t": 6.432538,
            "value": 393817.4597001671
          },
          {
            "t": 7.549025,
            "value": 358266.59871543513
          },
          {
            "t": 8.55782,
            "value": 396512.6710580445
          },
          {
            "t": 9.566972,
            "value": 396372.3997970573
          },
          {
            "t": 10.576951,
            "value": 396047.83861842676
          },
          {
            "t": 11.586092,
            "value": 396376.7203988342
          },
          {
            "t": 12.595873,
            "value": 397115.8102598484
          },
          {
            "t": 13.711521,
            "value": 357639.68563561264
          },
          {
            "t": 14.720411,
            "value": 396475.3342782662
          },
          {
            "t": 15.730468,
            "value": 396017.25447177736
          },
          {
            "t": 16.739519,
            "value": 396412.07431537163
          },
          {
            "t": 17.748571,
            "value": 396411.6814594292
          },
          {
            "t": 18.758738,
            "value": 395974.1310100211
          },
          {
            "t": 19.773386,
            "value": 394225.3865379915
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.157971,
            "value": 395006.87549561483
          },
          {
            "t": 1.172464,
            "value": 394285.6185306355
          },
          {
            "t": 2.181549,
            "value": 396398.7176501484
          },
          {
            "t": 3.190763,
            "value": 396348.0490758154
          },
          {
            "t": 4.200084,
            "value": 393333.7362444654
          },
          {
            "t": 5.209846,
            "value": 398113.61489142984
          },
          {
            "t": 6.124867,
            "value": 440427.050308135
          },
          {
            "t": 7.140179,
            "value": 394952.4875112281
          },
          {
            "t": 8.15481,
            "value": 394231.9917290128
          },
          {
            "t": 9.163902,
            "value": 396395.9678602149
          },
          {
            "t": 10.173836,
            "value": 395075.3217536987
          },
          {
            "t": 11.183003,
            "value": 397357.4244897029
          },
          {
            "t": 12.192109,
            "value": 396390.46839479695
          },
          {
            "t": 13.202092,
            "value": 396046.2700857341
          },
          {
            "t": 14.216658,
            "value": 394257.24891234277
          },
          {
            "t": 15.226673,
            "value": 396033.7222714514
          },
          {
            "t": 16.235877,
            "value": 396351.9764091304
          },
          {
            "t": 17.2449,
            "value": 396423.0745979031
          },
          {
            "t": 18.254609,
            "value": 396153.7433062397
          },
          {
            "t": 19.264562,
            "value": 396058.0343837782
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.057197,
            "value": 341496910.4401428
          },
          {
            "t": 2.080843,
            "value": 340881975.4047892
          },
          {
            "t": 4.099298,
            "value": 342611602.9339272
          },
          {
            "t": 6.124867,
            "value": 340128397.50213397
          },
          {
            "t": 8.053961,
            "value": 358046453.4128456
          },
          {
            "t": 10.073065,
            "value": 341654856.80777216
          },
          {
            "t": 12.091433,
            "value": 341783174.326981
          },
          {
            "t": 14.115877,
            "value": 341177514.4187738
          },
          {
            "t": 16.135254,
            "value": 341172068.91036195
          },
          {
            "t": 18.153971,
            "value": 341938230.07385385
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.057197,
            "value": 369271599.07063186
          },
          {
            "t": 2.080843,
            "value": 368090797.501144
          },
          {
            "t": 4.099298,
            "value": 369487999.9801829
          },
          {
            "t": 6.124867,
            "value": 367746715.12054145
          },
          {
            "t": 8.053961,
            "value": 386648176.81253475
          },
          {
            "t": 10.073065,
            "value": 368919187.9170166
          },
          {
            "t": 12.091433,
            "value": 368601828.30881184
          },
          {
            "t": 14.115877,
            "value": 368410082.4720269
          },
          {
            "t": 16.135254,
            "value": 368402602.881978
          },
          {
            "t": 18.153971,
            "value": 369451451.0949281
          }
        ],
        "ram_mib": [
          {
            "t": 0.057197,
            "value": 56.89453125
          },
          {
            "t": 2.080843,
            "value": 58.6953125
          },
          {
            "t": 4.099298,
            "value": 62.1484375
          },
          {
            "t": 6.124867,
            "value": 59.4375
          },
          {
            "t": 8.053961,
            "value": 60.67578125
          },
          {
            "t": 10.073065,
            "value": 58.82421875
          },
          {
            "t": 12.091433,
            "value": 58.91015625
          },
          {
            "t": 14.115877,
            "value": 60.05078125
          },
          {
            "t": 16.135254,
            "value": 59.5390625
          },
          {
            "t": 18.153971,
            "value": 61.11328125
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
