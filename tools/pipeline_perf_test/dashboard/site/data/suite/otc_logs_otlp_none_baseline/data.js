window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otlp_none_baseline"] = {
  "name": "OTC OTLP Baseline (Logs)",
  "slug": "otc_logs_otlp_none_baseline",
  "description": "OpenTelemetry Collector baseline for OTLP logs with no compression",
  "meta": {
    "binary": "otc",
    "protocols": [
      "otlp"
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
          "extra": "OTC OTLP Baseline (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.6315789222717285
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 19.09686958371338
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 21.22330630068621
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 51.447265625
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 53.05078125
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 99436.65992049986
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/100k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 102053.41412893406
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000634
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 36187256.90423184
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 35897516.80886691
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.0766,
            "value": 19.633187772925766
          },
          {
            "t": 3.088573,
            "value": 21.22330630068621
          },
          {
            "t": 5.100077,
            "value": 18.504837905236908
          },
          {
            "t": 7.112136,
            "value": 18.83223880597015
          },
          {
            "t": 9.123138,
            "value": 17.736660459342023
          },
          {
            "t": 11.133852,
            "value": 18.622643391521194
          },
          {
            "t": 13.14528,
            "value": 20.133084112149533
          },
          {
            "t": 15.156766,
            "value": 18.192359550561797
          },
          {
            "t": 17.167784,
            "value": 18.611770573566087
          },
          {
            "t": 19.178805,
            "value": 19.47860696517413
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.071164,
            "value": 99412.57111726807
          },
          {
            "t": 1.0766,
            "value": 99459.33903301653
          },
          {
            "t": 2.082651,
            "value": 99398.5394378615
          },
          {
            "t": 3.088573,
            "value": 99411.28636216327
          },
          {
            "t": 4.094229,
            "value": 99437.58104162854
          },
          {
            "t": 5.100077,
            "value": 99418.60002704186
          },
          {
            "t": 6.106069,
            "value": 100398.41271103547
          },
          {
            "t": 7.112136,
            "value": 98402.98906534057
          },
          {
            "t": 8.117551,
            "value": 99461.41643003139
          },
          {
            "t": 9.123138,
            "value": 99444.40411421389
          },
          {
            "t": 10.12866,
            "value": 99450.83250291889
          },
          {
            "t": 11.133852,
            "value": 99483.48176268811
          },
          {
            "t": 12.139866,
            "value": 99402.19519807876
          },
          {
            "t": 13.14528,
            "value": 99461.51535586335
          },
          {
            "t": 14.150646,
            "value": 99466.26402722989
          },
          {
            "t": 15.156766,
            "value": 99391.7226573371
          },
          {
            "t": 16.162187,
            "value": 99460.82287917202
          },
          {
            "t": 17.167784,
            "value": 99443.41520509706
          },
          {
            "t": 18.173456,
            "value": 99435.9990135949
          },
          {
            "t": 19.178805,
            "value": 99467.9459570756
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.071164,
            "value": 99412.57111726807
          },
          {
            "t": 1.0766,
            "value": 99459.33903301653
          },
          {
            "t": 2.082651,
            "value": 99398.5394378615
          },
          {
            "t": 3.088573,
            "value": 99411.28636216327
          },
          {
            "t": 4.094229,
            "value": 149156.3715624428
          },
          {
            "t": 5.100077,
            "value": 99418.60002704186
          },
          {
            "t": 6.106069,
            "value": 99404.3690208272
          },
          {
            "t": 7.112136,
            "value": 99396.95865185917
          },
          {
            "t": 8.117551,
            "value": 99461.41643003139
          },
          {
            "t": 9.123138,
            "value": 99444.40411421389
          },
          {
            "t": 10.12866,
            "value": 99450.83250291889
          },
          {
            "t": 11.133852,
            "value": 99483.48176268811
          },
          {
            "t": 12.139866,
            "value": 99402.19519807876
          },
          {
            "t": 13.14528,
            "value": 99461.51535586335
          },
          {
            "t": 14.150646,
            "value": 99466.26402722989
          },
          {
            "t": 15.156766,
            "value": 99391.7226573371
          },
          {
            "t": 16.162187,
            "value": 99460.82287917202
          },
          {
            "t": 17.167784,
            "value": 99443.41520509706
          },
          {
            "t": 18.173456,
            "value": 99435.9990135949
          },
          {
            "t": 19.178805,
            "value": 99467.9459570756
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.0766,
            "value": 35915135.89165459
          },
          {
            "t": 3.088573,
            "value": 35904169.19113726
          },
          {
            "t": 5.100077,
            "value": 35909715.317493774
          },
          {
            "t": 7.112136,
            "value": 35722546.40644236
          },
          {
            "t": 9.123138,
            "value": 35923438.16664529
          },
          {
            "t": 11.133852,
            "value": 35927899.741087
          },
          {
            "t": 13.14528,
            "value": 35913959.63464762
          },
          {
            "t": 15.156766,
            "value": 35914031.71585584
          },
          {
            "t": 17.167784,
            "value": 35923127.98791458
          },
          {
            "t": 19.178805,
            "value": 35921144.03579078
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.0766,
            "value": 36203237.6345182
          },
          {
            "t": 3.088573,
            "value": 36190871.34867118
          },
          {
            "t": 5.100077,
            "value": 36199936.96259118
          },
          {
            "t": 7.112136,
            "value": 36011922.11560397
          },
          {
            "t": 9.123138,
            "value": 36212524.40325768
          },
          {
            "t": 11.133852,
            "value": 36221463.61939092
          },
          {
            "t": 13.14528,
            "value": 36202890.6826394
          },
          {
            "t": 15.156766,
            "value": 36202968.35275015
          },
          {
            "t": 17.167784,
            "value": 36033591.94199157
          },
          {
            "t": 19.178805,
            "value": 36393161.98090423
          }
        ],
        "ram_mib": [
          {
            "t": 1.0766,
            "value": 49.546875
          },
          {
            "t": 3.088573,
            "value": 50.31640625
          },
          {
            "t": 5.100077,
            "value": 52.078125
          },
          {
            "t": 7.112136,
            "value": 51.4140625
          },
          {
            "t": 9.123138,
            "value": 51.43359375
          },
          {
            "t": 11.133852,
            "value": 51.55078125
          },
          {
            "t": 13.14528,
            "value": 51.50390625
          },
          {
            "t": 15.156766,
            "value": 51.80078125
          },
          {
            "t": 17.167784,
            "value": 51.77734375
          },
          {
            "t": 19.178805,
            "value": 53.05078125
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
          "extra": "OTC OTLP Baseline (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.6315789222717285
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 32.01468313931408
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 33.63741293532338
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 51.381640625
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 52.87890625
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 198408.7824498869
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/200k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 203630.06619856815
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000602
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 72590366.20833948
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 71975422.38482442
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.096677,
            "value": 32.58406991260924
          },
          {
            "t": 3.109808,
            "value": 33.63741293532338
          },
          {
            "t": 5.122635,
            "value": 32.73124378109453
          },
          {
            "t": 7.135735,
            "value": 30.717432098765435
          },
          {
            "t": 9.14993,
            "value": 33.282490660024905
          },
          {
            "t": 11.164844,
            "value": 32.059329192546585
          },
          {
            "t": 13.078927,
            "value": 32.71432835820895
          },
          {
            "t": 15.100441,
            "value": 31.329051456912588
          },
          {
            "t": 17.119548,
            "value": 30.239103920348477
          },
          {
            "t": 19.139379,
            "value": 30.852369077306737
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.09039,
            "value": 198645.83136756727
          },
          {
            "t": 1.096677,
            "value": 198750.4558838582
          },
          {
            "t": 2.103386,
            "value": 198667.1421433602
          },
          {
            "t": 3.109808,
            "value": 198723.7957834785
          },
          {
            "t": 4.116184,
            "value": 198732.87916246016
          },
          {
            "t": 5.122635,
            "value": 198718.06973215786
          },
          {
            "t": 6.129402,
            "value": 198655.6968990839
          },
          {
            "t": 7.135735,
            "value": 198741.37089810232
          },
          {
            "t": 8.141942,
            "value": 199760.08912679
          },
          {
            "t": 9.14993,
            "value": 197422.98519426817
          },
          {
            "t": 10.156166,
            "value": 198760.52933904176
          },
          {
            "t": 11.164844,
            "value": 198279.3319572748
          },
          {
            "t": 12.173092,
            "value": 198363.89459736095
          },
          {
            "t": 13.184563,
            "value": 197731.81831214143
          },
          {
            "t": 14.194816,
            "value": 197970.21142228728
          },
          {
            "t": 15.205346,
            "value": 197915.94509811685
          },
          {
            "t": 16.213751,
            "value": 198333.0110421904
          },
          {
            "t": 17.224137,
            "value": 197944.1520369443
          },
          {
            "t": 18.233092,
            "value": 198224.89605582014
          },
          {
            "t": 19.242768,
            "value": 198083.34554847298
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.09039,
            "value": 198645.83136756727
          },
          {
            "t": 1.096677,
            "value": 198750.4558838582
          },
          {
            "t": 2.103386,
            "value": 198667.1421433602
          },
          {
            "t": 3.109808,
            "value": 198723.7957834785
          },
          {
            "t": 4.116184,
            "value": 198732.87916246016
          },
          {
            "t": 5.122635,
            "value": 198718.06973215786
          },
          {
            "t": 6.129402,
            "value": 198655.6968990839
          },
          {
            "t": 7.135735,
            "value": 198741.37089810232
          },
          {
            "t": 8.141942,
            "value": 198766.257837602
          },
          {
            "t": 9.14993,
            "value": 198415.06049675192
          },
          {
            "t": 10.156166,
            "value": 198760.52933904176
          },
          {
            "t": 11.164844,
            "value": 198279.3319572748
          },
          {
            "t": 12.173092,
            "value": 198363.89459736095
          },
          {
            "t": 13.184563,
            "value": 197731.81831214143
          },
          {
            "t": 14.194816,
            "value": 197970.21142228728
          },
          {
            "t": 15.205346,
            "value": 197915.94509811685
          },
          {
            "t": 16.213751,
            "value": 198333.0110421904
          },
          {
            "t": 17.224137,
            "value": 296916.22805541643
          },
          {
            "t": 18.233092,
            "value": 198224.89605582014
          },
          {
            "t": 19.242768,
            "value": 198083.34554847298
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.096677,
            "value": 71386484.75190552
          },
          {
            "t": 3.109808,
            "value": 71737834.24923664
          },
          {
            "t": 5.122635,
            "value": 71751980.67196038
          },
          {
            "t": 7.135735,
            "value": 71742886.59281704
          },
          {
            "t": 9.14993,
            "value": 71701569.6096952
          },
          {
            "t": 11.164844,
            "value": 71680296.52878484
          },
          {
            "t": 13.078927,
            "value": 75454895.63409737
          },
          {
            "t": 15.100441,
            "value": 71444317.97157973
          },
          {
            "t": 17.119548,
            "value": 71350157.76776566
          },
          {
            "t": 19.139379,
            "value": 71503800.07040194
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.096677,
            "value": 71997901.74774876
          },
          {
            "t": 3.109808,
            "value": 72347197.97171669
          },
          {
            "t": 5.122635,
            "value": 72360485.52607849
          },
          {
            "t": 7.135735,
            "value": 72356483.03611346
          },
          {
            "t": 9.14993,
            "value": 72312039.30106072
          },
          {
            "t": 11.164844,
            "value": 72294334.64654075
          },
          {
            "t": 13.078927,
            "value": 76099258.49610493
          },
          {
            "t": 15.100441,
            "value": 72049727.58041745
          },
          {
            "t": 17.119548,
            "value": 71962233.79939745
          },
          {
            "t": 19.139379,
            "value": 72123999.978216
          }
        ],
        "ram_mib": [
          {
            "t": 1.096677,
            "value": 50.36328125
          },
          {
            "t": 3.109808,
            "value": 52.6953125
          },
          {
            "t": 5.122635,
            "value": 50.671875
          },
          {
            "t": 7.135735,
            "value": 50.578125
          },
          {
            "t": 9.14993,
            "value": 51.6640625
          },
          {
            "t": 11.164844,
            "value": 51.58984375
          },
          {
            "t": 13.078927,
            "value": 51.13671875
          },
          {
            "t": 15.100441,
            "value": 52.87890625
          },
          {
            "t": 17.119548,
            "value": 50.8203125
          },
          {
            "t": 19.139379,
            "value": 51.41796875
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
          "extra": "OTC OTLP Baseline (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.6135764122009277
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 43.01495910321453
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 44.47629304946775
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 54.75546875
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 56.62109375
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 296191.0331892417
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/300k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 305615.98364280566
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000759
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 108874744.03119686
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 107983698.05148897
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.07479,
            "value": 43.21256074766355
          },
          {
            "t": 3.09955,
            "value": 44.47629304946775
          },
          {
            "t": 5.125863,
            "value": 41.52
          },
          {
            "t": 7.151254,
            "value": 41.733233830845776
          },
          {
            "t": 9.071252,
            "value": 42.293896830329395
          },
          {
            "t": 11.098008,
            "value": 44.01423771001867
          },
          {
            "t": 13.124632,
            "value": 42.36019950124688
          },
          {
            "t": 15.151258,
            "value": 42.90267246737104
          },
          {
            "t": 17.177859,
            "value": 43.4839549436796
          },
          {
            "t": 19.103063,
            "value": 44.152541951522686
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.062191,
            "value": 296688.9513034534
          },
          {
            "t": 1.07479,
            "value": 296267.3279353426
          },
          {
            "t": 2.087045,
            "value": 296368.0100369966
          },
          {
            "t": 3.09955,
            "value": 297282.48255564173
          },
          {
            "t": 4.11267,
            "value": 295127.9216677195
          },
          {
            "t": 5.125863,
            "value": 296093.6366516547
          },
          {
            "t": 6.138444,
            "value": 297260.1698036997
          },
          {
            "t": 7.151254,
            "value": 295218.25416415714
          },
          {
            "t": 8.1639,
            "value": 296253.57726194547
          },
          {
            "t": 9.176937,
            "value": 296139.2328217035
          },
          {
            "t": 10.189745,
            "value": 296206.1911043357
          },
          {
            "t": 11.203743,
            "value": 295858.5717131592
          },
          {
            "t": 12.216687,
            "value": 296166.4218357579
          },
          {
            "t": 13.230563,
            "value": 295894.1724629047
          },
          {
            "t": 14.243843,
            "value": 296068.2141165325
          },
          {
            "t": 15.257425,
            "value": 295979.999644824
          },
          {
            "t": 16.271205,
            "value": 295922.1921915998
          },
          {
            "t": 17.284477,
            "value": 296070.5516386518
          },
          {
            "t": 18.297353,
            "value": 296186.3051350807
          },
          {
            "t": 19.309904,
            "value": 297268.9770688094
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.062191,
            "value": 297677.91447446495
          },
          {
            "t": 1.07479,
            "value": 296267.3279353426
          },
          {
            "t": 2.087045,
            "value": 296368.0100369966
          },
          {
            "t": 3.09955,
            "value": 296294.83311193524
          },
          {
            "t": 4.11267,
            "value": 296114.9715729627
          },
          {
            "t": 5.125863,
            "value": 296093.6366516547
          },
          {
            "t": 6.138444,
            "value": 296272.59448873723
          },
          {
            "t": 7.151254,
            "value": 296205.6061847731
          },
          {
            "t": 8.1639,
            "value": 296253.57726194547
          },
          {
            "t": 9.176937,
            "value": 296139.2328217035
          },
          {
            "t": 10.189745,
            "value": 296206.1911043357
          },
          {
            "t": 11.203743,
            "value": 295858.5717131592
          },
          {
            "t": 12.216687,
            "value": 296166.4218357579
          },
          {
            "t": 13.124632,
            "value": 330416.4899856269
          },
          {
            "t": 14.137579,
            "value": 296165.5446928615
          },
          {
            "t": 15.151258,
            "value": 295951.67701017775
          },
          {
            "t": 16.164593,
            "value": 296052.14465107786
          },
          {
            "t": 17.177859,
            "value": 296072.304804464
          },
          {
            "t": 18.191063,
            "value": 296090.42206702696
          },
          {
            "t": 19.20386,
            "value": 444314.11230483506
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.07479,
            "value": 107053447.07546699
          },
          {
            "t": 3.09955,
            "value": 106976813.05438669
          },
          {
            "t": 5.125863,
            "value": 106407489.85966137
          },
          {
            "t": 7.151254,
            "value": 106967951.37333977
          },
          {
            "t": 9.071252,
            "value": 112838377.9566437
          },
          {
            "t": 11.098008,
            "value": 106894330.15123677
          },
          {
            "t": 13.124632,
            "value": 106544278.56375924
          },
          {
            "t": 15.151258,
            "value": 107076982.63024358
          },
          {
            "t": 17.177859,
            "value": 106546187.43403365
          },
          {
            "t": 19.103063,
            "value": 112531122.41611798
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.07479,
            "value": 107936812.14018072
          },
          {
            "t": 3.09955,
            "value": 107525176.81107883
          },
          {
            "t": 5.125863,
            "value": 107627177.53871194
          },
          {
            "t": 7.151254,
            "value": 107849369.82538188
          },
          {
            "t": 9.071252,
            "value": 113772858.61756106
          },
          {
            "t": 11.098008,
            "value": 107415277.91209204
          },
          {
            "t": 13.124632,
            "value": 107599402.25715278
          },
          {
            "t": 15.151258,
            "value": 107780303.81530683
          },
          {
            "t": 17.177859,
            "value": 107784003.85670392
          },
          {
            "t": 19.103063,
            "value": 113457057.5377986
          }
        ],
        "ram_mib": [
          {
            "t": 1.07479,
            "value": 54.5
          },
          {
            "t": 3.09955,
            "value": 56.1640625
          },
          {
            "t": 5.125863,
            "value": 51.140625
          },
          {
            "t": 7.151254,
            "value": 53.21875
          },
          {
            "t": 9.071252,
            "value": 54.29296875
          },
          {
            "t": 11.098008,
            "value": 56.62109375
          },
          {
            "t": 13.124632,
            "value": 55.56640625
          },
          {
            "t": 15.151258,
            "value": 55.65234375
          },
          {
            "t": 17.177859,
            "value": 55.24609375
          },
          {
            "t": 19.103063,
            "value": 55.15234375
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
          "extra": "OTC OTLP Baseline (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.6184210777282715
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 55.71941419117936
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 58.51747972551466
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 61.57734375
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 63.85546875
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 394099.0716322251
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/400k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 404418.2446920689
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000585
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 144953180.43440685
        },
        {
          "extra": "OTC OTLP Baseline (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 143734366.34479007
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.120113,
            "value": 54.087386150966935
          },
          {
            "t": 3.138356,
            "value": 55.313246105919
          },
          {
            "t": 5.055957,
            "value": 55.36407453416149
          },
          {
            "t": 7.074075,
            "value": 55.38420921544209
          },
          {
            "t": 9.092323,
            "value": 55.3234456928839
          },
          {
            "t": 11.113903,
            "value": 56.31979987492183
          },
          {
            "t": 13.1376,
            "value": 56.67578947368421
          },
          {
            "t": 15.156096,
            "value": 55.55544430538173
          },
          {
            "t": 17.174217,
            "value": 54.6532668329177
          },
          {
            "t": 19.192968,
            "value": 58.51747972551466
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.110227,
            "value": 396024.3119325096
          },
          {
            "t": 1.120113,
            "value": 396084.31050633435
          },
          {
            "t": 2.129367,
            "value": 396332.3405208203
          },
          {
            "t": 3.138356,
            "value": 396436.4329046204
          },
          {
            "t": 4.147415,
            "value": 396408.93148963543
          },
          {
            "t": 5.15671,
            "value": 396316.2405441422
          },
          {
            "t": 6.16558,
            "value": 396483.1940686114
          },
          {
            "t": 7.174927,
            "value": 396295.8229429522
          },
          {
            "t": 8.183922,
            "value": 396434.0754909588
          },
          {
            "t": 9.19317,
            "value": 396334.6967246901
          },
          {
            "t": 10.203208,
            "value": 396024.70402103686
          },
          {
            "t": 11.220403,
            "value": 393238.2679820486
          },
          {
            "t": 12.330223,
            "value": 360418.80665333115
          },
          {
            "t": 13.339324,
            "value": 396392.43247207167
          },
          {
            "t": 14.348975,
            "value": 396176.5005927791
          },
          {
            "t": 15.357686,
            "value": 396545.6904901404
          },
          {
            "t": 16.366773,
            "value": 396397.9319919888
          },
          {
            "t": 17.375975,
            "value": 396352.76188513305
          },
          {
            "t": 18.385157,
            "value": 396360.6168163919
          },
          {
            "t": 19.394718,
            "value": 396211.81880044896
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.110227,
            "value": 396024.3119325096
          },
          {
            "t": 1.120113,
            "value": 395094.0997300685
          },
          {
            "t": 2.129367,
            "value": 396332.3405208203
          },
          {
            "t": 3.138356,
            "value": 396436.4329046204
          },
          {
            "t": 4.147415,
            "value": 397399.9538183595
          },
          {
            "t": 5.15671,
            "value": 396316.2405441422
          },
          {
            "t": 6.16558,
            "value": 396483.1940686114
          },
          {
            "t": 7.174927,
            "value": 396295.8229429522
          },
          {
            "t": 8.183922,
            "value": 395442.9903022314
          },
          {
            "t": 9.19317,
            "value": 594502.0450870352
          },
          {
            "t": 10.307417,
            "value": 358986.8314655548
          },
          {
            "t": 11.321829,
            "value": 394317.10192702763
          },
          {
            "t": 12.330223,
            "value": 397662.02496246505
          },
          {
            "t": 13.339324,
            "value": 395401.4513908915
          },
          {
            "t": 14.348975,
            "value": 396176.5005927791
          },
          {
            "t": 15.357686,
            "value": 397537.05471636576
          },
          {
            "t": 16.366773,
            "value": 396397.9319919888
          },
          {
            "t": 17.375975,
            "value": 396352.76188513305
          },
          {
            "t": 18.385157,
            "value": 396360.6168163919
          },
          {
            "t": 19.394718,
            "value": 395221.2892534478
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.120113,
            "value": 143036953.3522284
          },
          {
            "t": 3.138356,
            "value": 143131249.3094241
          },
          {
            "t": 5.055957,
            "value": 150618282.9483297
          },
          {
            "t": 7.074075,
            "value": 142982133.3539466
          },
          {
            "t": 9.092323,
            "value": 142949170.2704524
          },
          {
            "t": 11.113903,
            "value": 142894090.76069212
          },
          {
            "t": 13.1376,
            "value": 142743571.29550523
          },
          {
            "t": 15.156096,
            "value": 142758574.70116365
          },
          {
            "t": 17.174217,
            "value": 143137848.02794284
          },
          {
            "t": 19.192968,
            "value": 143091789.42821577
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.120113,
            "value": 144071726.4254861
          },
          {
            "t": 3.138356,
            "value": 144541552.72680247
          },
          {
            "t": 5.055957,
            "value": 151562299.9779412
          },
          {
            "t": 7.074075,
            "value": 144378658.73055986
          },
          {
            "t": 9.092323,
            "value": 144361350.04221484
          },
          {
            "t": 11.113903,
            "value": 144125724.9280266
          },
          {
            "t": 13.1376,
            "value": 143790968.7072719
          },
          {
            "t": 15.156096,
            "value": 144178448.70636356
          },
          {
            "t": 17.174217,
            "value": 144375967.0505386
          },
          {
            "t": 19.192968,
            "value": 144145107.04886338
          }
        ],
        "ram_mib": [
          {
            "t": 1.120113,
            "value": 59.58203125
          },
          {
            "t": 3.138356,
            "value": 62.36328125
          },
          {
            "t": 5.055957,
            "value": 61.0
          },
          {
            "t": 7.074075,
            "value": 63.85546875
          },
          {
            "t": 9.092323,
            "value": 58.546875
          },
          {
            "t": 11.113903,
            "value": 62.96484375
          },
          {
            "t": 13.1376,
            "value": 60.24609375
          },
          {
            "t": 15.156096,
            "value": 62.27734375
          },
          {
            "t": 17.174217,
            "value": 61.42578125
          },
          {
            "t": 19.192968,
            "value": 63.51171875
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
