window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otap_none_baseline"] = {
  "name": "OTC OTAP Baseline (Logs)",
  "slug": "otc_logs_otap_none_baseline",
  "description": "OpenTelemetry Collector baseline for OTAP logs with no compression",
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
          "extra": "OTC OTAP Baseline (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 5.0
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 47.066263123262665
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 48.48586335403727
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 80.876171875
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 83.5
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 104684.08958864387
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/100k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 99449.88510921168
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000617
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 10405497.304260913
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 11075333.438380124
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.118809,
            "value": 48.48586335403727
          },
          {
            "t": 3.130003,
            "value": 47.76753407682776
          },
          {
            "t": 5.040583,
            "value": 46.53285448916409
          },
          {
            "t": 7.051091,
            "value": 48.34086956521739
          },
          {
            "t": 9.061571,
            "value": 46.26190416925949
          },
          {
            "t": 11.072377,
            "value": 47.72503409795412
          },
          {
            "t": 13.082694,
            "value": 47.47922933499068
          },
          {
            "t": 15.095306,
            "value": 45.238407960199005
          },
          {
            "t": 17.107957,
            "value": 46.53219118559901
          },
          {
            "t": 19.118188,
            "value": 46.29874299937772
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.113584,
            "value": 99486.94582040417
          },
          {
            "t": 1.118809,
            "value": 99480.21587206844
          },
          {
            "t": 2.124864,
            "value": 99398.14423664712
          },
          {
            "t": 3.130003,
            "value": 99488.72742973857
          },
          {
            "t": 4.135475,
            "value": 99455.7779828777
          },
          {
            "t": 5.141023,
            "value": 99448.2610477073
          },
          {
            "t": 6.146301,
            "value": 99474.97110252091
          },
          {
            "t": 7.151546,
            "value": 99478.23664877717
          },
          {
            "t": 8.15703,
            "value": 99454.59102283079
          },
          {
            "t": 9.162085,
            "value": 99497.04245041315
          },
          {
            "t": 10.167411,
            "value": 99470.22159975968
          },
          {
            "t": 11.172973,
            "value": 99446.87647305685
          },
          {
            "t": 12.178043,
            "value": 99495.55752335659
          },
          {
            "t": 13.183182,
            "value": 99488.72742973857
          },
          {
            "t": 14.190586,
            "value": 99265.04163175846
          },
          {
            "t": 15.195735,
            "value": 99487.7376388973
          },
          {
            "t": 16.20209,
            "value": 99368.51309925423
          },
          {
            "t": 17.208438,
            "value": 99369.20429115971
          },
          {
            "t": 18.213493,
            "value": 198994.0849008263
          },
          {
            "t": 19.218684,
            "value": 99483.58073241802
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.113584,
            "value": 99486.94582040417
          },
          {
            "t": 1.118809,
            "value": 99480.21587206844
          },
          {
            "t": 2.124864,
            "value": 99398.14423664712
          },
          {
            "t": 3.130003,
            "value": 99488.72742973857
          },
          {
            "t": 4.135475,
            "value": 99455.7779828777
          },
          {
            "t": 5.141023,
            "value": 99448.2610477073
          },
          {
            "t": 6.146301,
            "value": 99474.97110252091
          },
          {
            "t": 7.151546,
            "value": 99478.23664877717
          },
          {
            "t": 8.15703,
            "value": 99454.59102283079
          },
          {
            "t": 9.162085,
            "value": 99497.04245041315
          },
          {
            "t": 10.167411,
            "value": 99470.22159975968
          },
          {
            "t": 11.172973,
            "value": 99446.87647305685
          },
          {
            "t": 12.178043,
            "value": 99495.55752335659
          },
          {
            "t": 13.183182,
            "value": 99488.72742973857
          },
          {
            "t": 14.190586,
            "value": 98272.39121544088
          },
          {
            "t": 15.195735,
            "value": 100482.61501528628
          },
          {
            "t": 16.20209,
            "value": 99368.51309925423
          },
          {
            "t": 17.208438,
            "value": 98375.51224824812
          },
          {
            "t": 18.213493,
            "value": 100492.01287491729
          },
          {
            "t": 19.218684,
            "value": 99483.58073241802
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.118809,
            "value": 11026176.119762314
          },
          {
            "t": 3.130003,
            "value": 11021915.339842899
          },
          {
            "t": 5.040583,
            "value": 11602166.881261189
          },
          {
            "t": 7.051091,
            "value": 11025268.738050282
          },
          {
            "t": 9.061571,
            "value": 10971251.144005412
          },
          {
            "t": 11.072377,
            "value": 11078689.34148794
          },
          {
            "t": 13.082694,
            "value": 10972357.095920693
          },
          {
            "t": 15.095306,
            "value": 11013993.258511825
          },
          {
            "t": 17.107957,
            "value": 11014179.805639427
          },
          {
            "t": 19.118188,
            "value": 11027336.659319252
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.118809,
            "value": 10360705.079930084
          },
          {
            "t": 3.130003,
            "value": 10356708.99972852
          },
          {
            "t": 5.040583,
            "value": 10901493.787226915
          },
          {
            "t": 7.051091,
            "value": 10308744.854534276
          },
          {
            "t": 9.061571,
            "value": 10360152.799331503
          },
          {
            "t": 11.072377,
            "value": 10358134.996613298
          },
          {
            "t": 13.082694,
            "value": 10360893.331748176
          },
          {
            "t": 15.095306,
            "value": 10349142.308601957
          },
          {
            "t": 17.107957,
            "value": 10344542.595810203
          },
          {
            "t": 19.118188,
            "value": 10354454.289084189
          }
        ],
        "ram_mib": [
          {
            "t": 1.118809,
            "value": 80.35546875
          },
          {
            "t": 3.130003,
            "value": 81.56640625
          },
          {
            "t": 5.040583,
            "value": 80.1171875
          },
          {
            "t": 7.051091,
            "value": 77.140625
          },
          {
            "t": 9.061571,
            "value": 81.08984375
          },
          {
            "t": 11.072377,
            "value": 82.52734375
          },
          {
            "t": 13.082694,
            "value": 81.484375
          },
          {
            "t": 15.095306,
            "value": 78.765625
          },
          {
            "t": 17.107957,
            "value": 82.21484375
          },
          {
            "t": 19.118188,
            "value": 83.5
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
          "extra": "OTC OTAP Baseline (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.026315787807106972
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 83.65064167441749
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 85.7335495945103
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 99.124609375
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 104.19921875
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 198117.55993616654
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/200k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 197029.63945028163
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000595
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 20727231.28787933
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 22072093.952945154
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.048734,
            "value": 85.7335495945103
          },
          {
            "t": 3.062688,
            "value": 84.55370786516853
          },
          {
            "t": 5.080868,
            "value": 83.88579962663349
          },
          {
            "t": 7.098381,
            "value": 81.72308457711442
          },
          {
            "t": 9.118513,
            "value": 82.86785314250156
          },
          {
            "t": 11.141333,
            "value": 82.34120422098076
          },
          {
            "t": 13.161966,
            "value": 84.51714463066419
          },
          {
            "t": 15.181774,
            "value": 82.92224719101124
          },
          {
            "t": 17.097839,
            "value": 84.79404466501241
          },
          {
            "t": 19.118027,
            "value": 83.167781230578
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.042242,
            "value": 198776.3328947002
          },
          {
            "t": 1.048734,
            "value": 198709.9748433172
          },
          {
            "t": 2.055068,
            "value": 199734.87927467423
          },
          {
            "t": 3.062688,
            "value": 198487.52505905004
          },
          {
            "t": 4.071944,
            "value": 197174.9486750636
          },
          {
            "t": 5.080868,
            "value": 198230.9866749131
          },
          {
            "t": 6.08962,
            "value": 198264.7865877837
          },
          {
            "t": 7.098381,
            "value": 199254.33279042313
          },
          {
            "t": 8.107231,
            "value": 197254.29944986865
          },
          {
            "t": 9.118513,
            "value": 198757.61656985886
          },
          {
            "t": 10.130591,
            "value": 196625.1613017969
          },
          {
            "t": 11.141333,
            "value": 198863.8050066189
          },
          {
            "t": 12.152524,
            "value": 196797.63763720207
          },
          {
            "t": 13.161966,
            "value": 198129.2634940888
          },
          {
            "t": 14.173,
            "value": 197817.28408737984
          },
          {
            "t": 15.181774,
            "value": 198260.46270026785
          },
          {
            "t": 16.192245,
            "value": 198917.1386412871
          },
          {
            "t": 17.201989,
            "value": 197079.65583355783
          },
          {
            "t": 18.212185,
            "value": 197981.3818308526
          },
          {
            "t": 19.222773,
            "value": 197904.58624088153
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.042242,
            "value": 198776.3328947002
          },
          {
            "t": 1.048734,
            "value": 198709.9748433172
          },
          {
            "t": 2.157151,
            "value": 178633.1317545653
          },
          {
            "t": 3.163603,
            "value": 199711.46164943784
          },
          {
            "t": 4.172862,
            "value": 198165.18851949798
          },
          {
            "t": 5.181921,
            "value": 199195.4880735418
          },
          {
            "t": 6.190574,
            "value": 193327.14025537026
          },
          {
            "t": 7.199415,
            "value": 203203.47805055502
          },
          {
            "t": 8.208102,
            "value": 196294.7871837349
          },
          {
            "t": 9.219622,
            "value": 198710.85099652008
          },
          {
            "t": 10.231477,
            "value": 198645.0627807344
          },
          {
            "t": 11.242258,
            "value": 195888.13006971838
          },
          {
            "t": 12.253425,
            "value": 197791.26494436627
          },
          {
            "t": 13.262906,
            "value": 200102.82511508386
          },
          {
            "t": 14.273936,
            "value": 197818.06672403388
          },
          {
            "t": 15.282716,
            "value": 198259.28349094946
          },
          {
            "t": 16.293266,
            "value": 197912.02810350797
          },
          {
            "t": 17.303105,
            "value": 196070.85882006935
          },
          {
            "t": 18.31317,
            "value": 198007.05895165162
          },
          {
            "t": 19.323605,
            "value": 198924.2257047707
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.048734,
            "value": 22032997.324917223
          },
          {
            "t": 3.062688,
            "value": 22013889.095778752
          },
          {
            "t": 5.080868,
            "value": 21965701.275406554
          },
          {
            "t": 7.098381,
            "value": 21975268.065187186
          },
          {
            "t": 9.118513,
            "value": 21897280.474741258
          },
          {
            "t": 11.141333,
            "value": 21858890.55872495
          },
          {
            "t": 13.161966,
            "value": 21943451.383799035
          },
          {
            "t": 15.181774,
            "value": 21950447.270235587
          },
          {
            "t": 17.097839,
            "value": 23137203.59173619
          },
          {
            "t": 19.118027,
            "value": 21945810.488924794
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.048734,
            "value": 20742960.02082828
          },
          {
            "t": 3.062688,
            "value": 20677513.984927163
          },
          {
            "t": 5.080868,
            "value": 20533218.047944188
          },
          {
            "t": 7.098381,
            "value": 20693052.28764325
          },
          {
            "t": 9.118513,
            "value": 20614992.485639554
          },
          {
            "t": 11.141333,
            "value": 20485649.736506462
          },
          {
            "t": 13.161966,
            "value": 20660067.41451812
          },
          {
            "t": 15.181774,
            "value": 20566858.830146234
          },
          {
            "t": 17.097839,
            "value": 21734056.516871817
          },
          {
            "t": 19.118027,
            "value": 20563943.553768262
          }
        ],
        "ram_mib": [
          {
            "t": 1.048734,
            "value": 99.87890625
          },
          {
            "t": 3.062688,
            "value": 99.70703125
          },
          {
            "t": 5.080868,
            "value": 90.1796875
          },
          {
            "t": 7.098381,
            "value": 102.09375
          },
          {
            "t": 9.118513,
            "value": 100.984375
          },
          {
            "t": 11.141333,
            "value": 98.3203125
          },
          {
            "t": 13.161966,
            "value": 104.19921875
          },
          {
            "t": 15.181774,
            "value": 99.8203125
          },
          {
            "t": 17.097839,
            "value": 96.0859375
          },
          {
            "t": 19.118027,
            "value": 99.9765625
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
          "extra": "OTC OTAP Baseline (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 1.363451361656189
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 99.94648684528512
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.58514001244554
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 108.178125
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 111.4765625
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 256079.79767507268
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/300k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 251263.39298164888
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000603
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 25745439.73134666
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 27412664.033624254
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.028623,
            "value": 99.6147120743034
          },
          {
            "t": 3.044302,
            "value": 99.77022332506203
          },
          {
            "t": 5.061075,
            "value": 100.1746908182386
          },
          {
            "t": 7.0768,
            "value": 99.6858452012384
          },
          {
            "t": 9.091781,
            "value": 98.48009913258984
          },
          {
            "t": 11.107021,
            "value": 100.25137157107231
          },
          {
            "t": 13.124999,
            "value": 100.12077162414437
          },
          {
            "t": 15.145077,
            "value": 100.2534574798262
          },
          {
            "t": 17.172011,
            "value": 100.58514001244554
          },
          {
            "t": 19.206083,
            "value": 100.52855721393034
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.121278,
            "value": 297655.36866105685
          },
          {
            "t": 1.129248,
            "value": 297627.90559242835
          },
          {
            "t": 2.136585,
            "value": 297814.93184505287
          },
          {
            "t": 3.144933,
            "value": 297516.3336467172
          },
          {
            "t": 4.154085,
            "value": 268542.3008625063
          },
          {
            "t": 5.161736,
            "value": 243139.73786559037
          },
          {
            "t": 6.169686,
            "value": 252988.7395208096
          },
          {
            "t": 7.177436,
            "value": 251054.3289506326
          },
          {
            "t": 8.185068,
            "value": 252076.15478666814
          },
          {
            "t": 9.192622,
            "value": 245148.1508683405
          },
          {
            "t": 10.20043,
            "value": 239132.85070172095
          },
          {
            "t": 11.207684,
            "value": 253163.55159671739
          },
          {
            "t": 12.218354,
            "value": 249339.54703315624
          },
          {
            "t": 13.226299,
            "value": 252989.99449374716
          },
          {
            "t": 14.2384,
            "value": 253927.22663054382
          },
          {
            "t": 15.25087,
            "value": 250871.63076436834
          },
          {
            "t": 16.26422,
            "value": 246706.4686436078
          },
          {
            "t": 17.278505,
            "value": 236619.8849435809
          },
          {
            "t": 18.29747,
            "value": 244365.60627695755
          },
          {
            "t": 19.310609,
            "value": 232939.40910378538
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.121278,
            "value": 243085.21773986312
          },
          {
            "t": 1.129248,
            "value": 248023.25466035694
          },
          {
            "t": 2.136585,
            "value": 256120.84138674548
          },
          {
            "t": 3.144933,
            "value": 243963.39359030811
          },
          {
            "t": 4.154085,
            "value": 240796.2328767123
          },
          {
            "t": 5.161736,
            "value": 239170.10949227458
          },
          {
            "t": 6.169686,
            "value": 251004.51411280324
          },
          {
            "t": 7.177436,
            "value": 255023.56735301416
          },
          {
            "t": 8.185068,
            "value": 249098.87736792795
          },
          {
            "t": 9.192622,
            "value": 248125.65877362402
          },
          {
            "t": 10.20043,
            "value": 238140.5982091827
          },
          {
            "t": 11.207684,
            "value": 250185.15687205014
          },
          {
            "t": 12.218354,
            "value": 250328.9896801132
          },
          {
            "t": 13.327541,
            "value": 230799.67579858043
          },
          {
            "t": 14.339533,
            "value": 381425.9401260089
          },
          {
            "t": 15.351927,
            "value": 248914.94813284156
          },
          {
            "t": 16.365327,
            "value": 238800.07894217485
          },
          {
            "t": 17.379519,
            "value": 242557.62222537745
          },
          {
            "t": 18.398708,
            "value": 237443.6929754933
          },
          {
            "t": 19.411792,
            "value": 226042.46044750483
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.028623,
            "value": 28892020.872776106
          },
          {
            "t": 3.044302,
            "value": 27439086.283083767
          },
          {
            "t": 5.061075,
            "value": 26693694.828322273
          },
          {
            "t": 7.0768,
            "value": 27831102.953031786
          },
          {
            "t": 9.091781,
            "value": 27319851.15492404
          },
          {
            "t": 11.107021,
            "value": 27547999.741966218
          },
          {
            "t": 13.124999,
            "value": 27739724.615431886
          },
          {
            "t": 15.145077,
            "value": 28055680.028196935
          },
          {
            "t": 17.172011,
            "value": 26619749.335696183
          },
          {
            "t": 19.206083,
            "value": 25987730.52281335
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.028623,
            "value": 27076572.32425275
          },
          {
            "t": 3.044302,
            "value": 25780302.81607339
          },
          {
            "t": 5.061075,
            "value": 25046226.818784263
          },
          {
            "t": 7.0768,
            "value": 26188818.911309823
          },
          {
            "t": 9.091781,
            "value": 25582297.302058928
          },
          {
            "t": 11.107021,
            "value": 25988898.592723448
          },
          {
            "t": 13.124999,
            "value": 25954872.154205848
          },
          {
            "t": 15.145077,
            "value": 26389911.676677834
          },
          {
            "t": 17.172011,
            "value": 25072635.32014363
          },
          {
            "t": 19.206083,
            "value": 24373861.397236675
          }
        ],
        "ram_mib": [
          {
            "t": 1.028623,
            "value": 105.984375
          },
          {
            "t": 3.044302,
            "value": 108.72265625
          },
          {
            "t": 5.061075,
            "value": 105.30078125
          },
          {
            "t": 7.0768,
            "value": 109.14453125
          },
          {
            "t": 9.091781,
            "value": 111.15234375
          },
          {
            "t": 11.107021,
            "value": 110.125
          },
          {
            "t": 13.124999,
            "value": 107.453125
          },
          {
            "t": 15.145077,
            "value": 103.01953125
          },
          {
            "t": 17.172011,
            "value": 111.4765625
          },
          {
            "t": 19.206083,
            "value": 109.40234375
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
          "extra": "OTC OTAP Baseline (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -0.3109452724456787
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.67112102050608
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 101.42528967254407
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 137.241015625
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 174.4375
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 249791.33568926333
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/400k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 253216.487799926
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.00063
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 26389753.94797593
        },
        {
          "extra": "OTC OTAP Baseline (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 28057306.29037981
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.108245,
            "value": 100.35857943925232
          },
          {
            "t": 3.126582,
            "value": 100.85366771159873
          },
          {
            "t": 5.044052,
            "value": 100.77797246558198
          },
          {
            "t": 7.076907,
            "value": 100.6567209011264
          },
          {
            "t": 9.105932,
            "value": 100.908671679198
          },
          {
            "t": 11.129432,
            "value": 101.42528967254407
          },
          {
            "t": 13.148682,
            "value": 100.8716104868914
          },
          {
            "t": 15.167745,
            "value": 100.3685607008761
          },
          {
            "t": 17.085947,
            "value": 99.88591478696742
          },
          {
            "t": 19.106794,
            "value": 100.60422236102436
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.097686,
            "value": 247560.2933094355
          },
          {
            "t": 1.108245,
            "value": 258272.8964860043
          },
          {
            "t": 2.117305,
            "value": 255683.50742274986
          },
          {
            "t": 3.126582,
            "value": 250674.4927309351
          },
          {
            "t": 4.135771,
            "value": 250696.35122856076
          },
          {
            "t": 5.148807,
            "value": 248757.20112611988
          },
          {
            "t": 6.163665,
            "value": 248310.60108901933
          },
          {
            "t": 7.181197,
            "value": 246675.28883612505
          },
          {
            "t": 8.196426,
            "value": 249204.85919925457
          },
          {
            "t": 9.212171,
            "value": 249078.26275295473
          },
          {
            "t": 10.32204,
            "value": 229756.84517722361
          },
          {
            "t": 11.331603,
            "value": 253575.06168510535
          },
          {
            "t": 12.341082,
            "value": 252605.55197284935
          },
          {
            "t": 13.350656,
            "value": 256543.84918787528
          },
          {
            "t": 14.360401,
            "value": 252539.0073731487
          },
          {
            "t": 15.370064,
            "value": 258502.09426313534
          },
          {
            "t": 16.378871,
            "value": 243852.39198379868
          },
          {
            "t": 17.388677,
            "value": 246582.0167438102
          },
          {
            "t": 18.398016,
            "value": 248677.5998945845
          },
          {
            "t": 19.409805,
            "value": 248075.4386537114
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.097686,
            "value": 222804.26397849197
          },
          {
            "t": 1.108245,
            "value": 284990.78232938406
          },
          {
            "t": 2.117305,
            "value": 254692.48607615006
          },
          {
            "t": 3.126582,
            "value": 252656.10927426268
          },
          {
            "t": 4.135771,
            "value": 251687.24589744836
          },
          {
            "t": 5.148807,
            "value": 230988.82961711133
          },
          {
            "t": 6.163665,
            "value": 259149.55589846068
          },
          {
            "t": 7.076907,
            "value": 284700.002847
          },
          {
            "t": 8.08956,
            "value": 247863.77959676215
          },
          {
            "t": 9.105932,
            "value": 248924.60634492093
          },
          {
            "t": 10.120043,
            "value": 238632.6546107872
          },
          {
            "t": 11.129432,
            "value": 265507.1533373159
          },
          {
            "t": 12.138807,
            "value": 251640.86687306504
          },
          {
            "t": 13.148682,
            "value": 258447.82770144817
          },
          {
            "t": 14.158501,
            "value": 253510.7776740188
          },
          {
            "t": 15.167745,
            "value": 254646.05189627086
          },
          {
            "t": 16.17703,
            "value": 247700.10452944413
          },
          {
            "t": 17.186719,
            "value": 237696.95421065297
          },
          {
            "t": 18.19596,
            "value": 255637.65245367558
          },
          {
            "t": 19.207816,
            "value": 235211.33441912683
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.108245,
            "value": 28073443.363587856
          },
          {
            "t": 3.126582,
            "value": 27945261.8665763
          },
          {
            "t": 5.044052,
            "value": 29301669.908786055
          },
          {
            "t": 7.076907,
            "value": 27264480.25068192
          },
          {
            "t": 9.105932,
            "value": 27514914.306132257
          },
          {
            "t": 11.129432,
            "value": 27980841.1168767
          },
          {
            "t": 13.148682,
            "value": 28150318.187445834
          },
          {
            "t": 15.167745,
            "value": 28312812.428339284
          },
          {
            "t": 17.085947,
            "value": 28383430.42077946
          },
          {
            "t": 19.106794,
            "value": 27645891.054592457
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.108245,
            "value": 27144904.460174993
          },
          {
            "t": 3.126582,
            "value": 26329683.298676092
          },
          {
            "t": 5.044052,
            "value": 27404390.681470897
          },
          {
            "t": 7.076907,
            "value": 25540153.134384893
          },
          {
            "t": 9.105932,
            "value": 25763941.794704355
          },
          {
            "t": 11.129432,
            "value": 26544732.888559427
          },
          {
            "t": 13.148682,
            "value": 26324230.778754488
          },
          {
            "t": 15.167745,
            "value": 26582486.52964271
          },
          {
            "t": 17.085947,
            "value": 26538622.626814067
          },
          {
            "t": 19.106794,
            "value": 25724393.286577363
          }
        ],
        "ram_mib": [
          {
            "t": 1.108245,
            "value": 147.48046875
          },
          {
            "t": 3.126582,
            "value": 111.53515625
          },
          {
            "t": 5.044052,
            "value": 174.4375
          },
          {
            "t": 7.076907,
            "value": 118.44140625
          },
          {
            "t": 9.105932,
            "value": 150.0
          },
          {
            "t": 11.129432,
            "value": 125.65625
          },
          {
            "t": 13.148682,
            "value": 126.265625
          },
          {
            "t": 15.167745,
            "value": 119.9140625
          },
          {
            "t": 17.085947,
            "value": 128.453125
          },
          {
            "t": 19.106794,
            "value": 170.2265625
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
