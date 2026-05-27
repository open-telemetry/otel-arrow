window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otap_none_attr_insert_multi_transform"] = {
  "name": "OTC OTAP Attr Insert Multi Transform (Logs)",
  "slug": "otc_logs_otap_none_attr_insert_multi_transform",
  "description": "OpenTelemetry Collector OTAP logs, attributes processor insert sweep over 1-4 insert actions at 240k signals/sec",
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
  "env": {
    "started_at": "2026-05-27T23:21:33Z",
    "ended_at": "2026-05-27T23:25:46Z",
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
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-1 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 3.377265214920044
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.06244769447201
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.44056022408962
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 822.1015625
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 862.8046875
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 61332.5749784414
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 61108.47650629139
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-1 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000724
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 8621488.119847814
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9023960.623170251
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-1 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 141.08497892203536
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.054697,
            "value": 100.08458281444584
          },
          {
            "t": 3.067635,
            "value": 99.71066997518611
          },
          {
            "t": 5.086617,
            "value": 100.13766978193146
          },
          {
            "t": 7.098834,
            "value": 100.04217864923748
          },
          {
            "t": 9.048107,
            "value": 99.91450419645632
          },
          {
            "t": 11.06108,
            "value": 100.13916510903427
          },
          {
            "t": 13.079214,
            "value": 100.44056022408962
          },
          {
            "t": 15.097111,
            "value": 100.13487850467288
          },
          {
            "t": 17.115001,
            "value": 99.97964541213064
          },
          {
            "t": 19.137772,
            "value": 100.04062227753579
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.149733,
            "value": 232377.38879011478
          },
          {
            "t": 1.255913,
            "value": 69608.92440651613
          },
          {
            "t": 2.262128,
            "value": 55654.109708163756
          },
          {
            "t": 3.269618,
            "value": 83375.51737486228
          },
          {
            "t": 4.280746,
            "value": 64284.64052029021
          },
          {
            "t": 5.388055,
            "value": 57797.77821728171
          },
          {
            "t": 6.394486,
            "value": 62597.435889792745
          },
          {
            "t": 7.432299,
            "value": 61668.14252664015
          },
          {
            "t": 8.443637,
            "value": 60316.13565395545
          },
          {
            "t": 9.550252,
            "value": 52412.085503991904
          },
          {
            "t": 10.556938,
            "value": 59601.504341969594
          },
          {
            "t": 11.568405,
            "value": 58331.11708043862
          },
          {
            "t": 12.675216,
            "value": 54209.79733667266
          },
          {
            "t": 13.681966,
            "value": 63570.896448969455
          },
          {
            "t": 14.693416,
            "value": 61298.13633891938
          },
          {
            "t": 15.704416,
            "value": 63303.65974282889
          },
          {
            "t": 16.811259,
            "value": 56015.17107665676
          },
          {
            "t": 17.817904,
            "value": 67551.12278906665
          },
          {
            "t": 18.83125,
            "value": 60196.615963353084
          },
          {
            "t": 19.943456,
            "value": 55745.068809195414
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.043012,
            "value": 50660.322537386826
          },
          {
            "t": 1.054697,
            "value": 61283.89765589092
          },
          {
            "t": 2.060863,
            "value": 49693.58932820231
          },
          {
            "t": 3.067635,
            "value": 53636.771781495714
          },
          {
            "t": 4.074457,
            "value": 68532.47147956639
          },
          {
            "t": 5.086617,
            "value": 65207.08188428707
          },
          {
            "t": 6.092845,
            "value": 63603.87506608841
          },
          {
            "t": 7.098834,
            "value": 55666.61265679843
          },
          {
            "t": 8.137089,
            "value": 60678.73499284858
          },
          {
            "t": 9.148518,
            "value": 59322.0087618607
          },
          {
            "t": 10.155198,
            "value": 63575.31688322009
          },
          {
            "t": 11.161489,
            "value": 65587.38973120101
          },
          {
            "t": 12.173126,
            "value": 60298.30858301941
          },
          {
            "t": 13.179647,
            "value": 59611.274876530144
          },
          {
            "t": 14.185943,
            "value": 57637.116713173855
          },
          {
            "t": 15.197447,
            "value": 68215.25174393774
          },
          {
            "t": 16.208907,
            "value": 58331.520771953416
          },
          {
            "t": 17.215349,
            "value": 67564.74789406642
          },
          {
            "t": 18.221819,
            "value": 59614.29550806284
          },
          {
            "t": 19.238385,
            "value": 62957.05345250578
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.054697,
            "value": 10421928.864094649
          },
          {
            "t": 3.067635,
            "value": 9654848.78322134
          },
          {
            "t": 5.086617,
            "value": 8866438.135654503
          },
          {
            "t": 7.098834,
            "value": 8897568.204622066
          },
          {
            "t": 9.048107,
            "value": 8859811.32452971
          },
          {
            "t": 11.06108,
            "value": 8315177.600494392
          },
          {
            "t": 13.079214,
            "value": 9427540.985881018
          },
          {
            "t": 15.097111,
            "value": 8428360.317697087
          },
          {
            "t": 17.115001,
            "value": 8628807.81410285
          },
          {
            "t": 19.137772,
            "value": 8739124.201404905
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.054697,
            "value": 7810400.368610625
          },
          {
            "t": 3.067635,
            "value": 6792574.8334027175
          },
          {
            "t": 5.086617,
            "value": 9336604.783995103
          },
          {
            "t": 7.098834,
            "value": 8610102.687731989
          },
          {
            "t": 9.048107,
            "value": 9170839.589939428
          },
          {
            "t": 11.06108,
            "value": 8601425.354438435
          },
          {
            "t": 13.079214,
            "value": 9369422.942183224
          },
          {
            "t": 15.097111,
            "value": 8844777.012900064
          },
          {
            "t": 17.115001,
            "value": 9112619.12195412
          },
          {
            "t": 19.137772,
            "value": 8566114.503322423
          }
        ],
        "ram_mib": [
          {
            "t": 1.054697,
            "value": 605.734375
          },
          {
            "t": 3.067635,
            "value": 760.2109375
          },
          {
            "t": 5.086617,
            "value": 824.15625
          },
          {
            "t": 7.098834,
            "value": 861.28125
          },
          {
            "t": 9.048107,
            "value": 859.6328125
          },
          {
            "t": 11.06108,
            "value": 860.77734375
          },
          {
            "t": 13.079214,
            "value": 861.80859375
          },
          {
            "t": 15.097111,
            "value": 861.8984375
          },
          {
            "t": 17.115001,
            "value": 862.8046875
          },
          {
            "t": 19.137772,
            "value": 862.7109375
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
      "name": "transform-2",
      "metrics": [
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-2 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 7.014808654785156
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.0218388246789
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.21383177570094
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 394.7390625
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 438.56640625
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 65334.16541459824
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 62361.54221714362
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-2 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000894
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9443488.969416913
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 8711527.79178161
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-2 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 151.43129296794126
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.108478,
            "value": 100.01407522536525
          },
          {
            "t": 3.076525,
            "value": 99.79175077881621
          },
          {
            "t": 5.101058,
            "value": 100.21383177570094
          },
          {
            "t": 7.117995,
            "value": 99.80461059190031
          },
          {
            "t": 9.139108,
            "value": 100.14115887850468
          },
          {
            "t": 11.055248,
            "value": 100.1673769470405
          },
          {
            "t": 13.076931,
            "value": 99.75559015882902
          },
          {
            "t": 15.093361,
            "value": 100.1019358854653
          },
          {
            "t": 17.115285,
            "value": 100.0928727046374
          },
          {
            "t": 19.130287,
            "value": 100.13518530052943
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.201555,
            "value": 149745.4327643007
          },
          {
            "t": 1.2133,
            "value": 73140.95943147731
          },
          {
            "t": 2.270529,
            "value": 72831.90302195646
          },
          {
            "t": 3.281416,
            "value": 71224.57801910597
          },
          {
            "t": 4.29381,
            "value": 77045.10299349857
          },
          {
            "t": 5.402995,
            "value": 69420.34015966678
          },
          {
            "t": 6.410956,
            "value": 75399.74264877311
          },
          {
            "t": 7.424958,
            "value": 68047.20306271585
          },
          {
            "t": 8.533472,
            "value": 63147.601203052014
          },
          {
            "t": 9.542212,
            "value": 58488.807819656206
          },
          {
            "t": 10.550317,
            "value": 60509.56993567138
          },
          {
            "t": 11.563484,
            "value": 66129.27582520946
          },
          {
            "t": 12.672487,
            "value": 25247.90284606985
          },
          {
            "t": 13.680427,
            "value": 73417.06847629818
          },
          {
            "t": 14.688907,
            "value": 75360.93923528478
          },
          {
            "t": 15.701628,
            "value": 75045.348126483
          },
          {
            "t": 16.81122,
            "value": 49567.76905385043
          },
          {
            "t": 17.818804,
            "value": 64510.750468447295
          },
          {
            "t": 18.825978,
            "value": 65529.88857933187
          },
          {
            "t": 19.83906,
            "value": 62186.4765142407
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.100527,
            "value": 61487.318736376095
          },
          {
            "t": 1.108478,
            "value": 61510.9266224251
          },
          {
            "t": 2.119528,
            "value": 56377.03377676673
          },
          {
            "t": 3.076525,
            "value": 74190.41021027234
          },
          {
            "t": 4.087167,
            "value": 55410.32333902608
          },
          {
            "t": 5.101058,
            "value": 71013.55076630524
          },
          {
            "t": 6.108982,
            "value": 60520.436064623915
          },
          {
            "t": 7.117995,
            "value": 68383.65808963809
          },
          {
            "t": 8.130945,
            "value": 72066.73577175576
          },
          {
            "t": 9.139108,
            "value": 53562.767131902285
          },
          {
            "t": 10.14791,
            "value": 58485.21315382008
          },
          {
            "t": 11.155821,
            "value": 58536.91446963075
          },
          {
            "t": 12.16968,
            "value": 74961.11392215287
          },
          {
            "t": 13.177432,
            "value": 45646.151037159936
          },
          {
            "t": 14.186268,
            "value": 58483.242073042595
          },
          {
            "t": 15.193835,
            "value": 65504.32874439118
          },
          {
            "t": 16.207091,
            "value": 67110.38473988805
          },
          {
            "t": 17.215773,
            "value": 63449.13461328743
          },
          {
            "t": 18.22315,
            "value": 49633.85108057857
          },
          {
            "t": 19.230907,
            "value": 70453.49226053503
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.108478,
            "value": 8567727.146257307
          },
          {
            "t": 3.076525,
            "value": 8970187.195732621
          },
          {
            "t": 5.101058,
            "value": 9059531.753742715
          },
          {
            "t": 7.117995,
            "value": 7966744.62315878
          },
          {
            "t": 9.139108,
            "value": 8449301.449250981
          },
          {
            "t": 11.055248,
            "value": 9383066.999279803
          },
          {
            "t": 13.076931,
            "value": 8479536.109271334
          },
          {
            "t": 15.093361,
            "value": 8763561.343562633
          },
          {
            "t": 17.115285,
            "value": 8936887.340968305
          },
          {
            "t": 19.130287,
            "value": 8538733.956591606
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.108478,
            "value": 9367054.524425814
          },
          {
            "t": 3.076525,
            "value": 9602452.583703538
          },
          {
            "t": 5.101058,
            "value": 9837808.027826665
          },
          {
            "t": 7.117995,
            "value": 9375774.75151678
          },
          {
            "t": 9.139108,
            "value": 9381949.450624483
          },
          {
            "t": 11.055248,
            "value": 9672031.793084012
          },
          {
            "t": 13.076931,
            "value": 8622633.221924506
          },
          {
            "t": 15.093361,
            "value": 9358873.355385507
          },
          {
            "t": 17.115285,
            "value": 8816370.447158253
          },
          {
            "t": 19.130287,
            "value": 10399941.538519565
          }
        ],
        "ram_mib": [
          {
            "t": 1.108478,
            "value": 347.17578125
          },
          {
            "t": 3.076525,
            "value": 386.0234375
          },
          {
            "t": 5.101058,
            "value": 393.26171875
          },
          {
            "t": 7.117995,
            "value": 371.92578125
          },
          {
            "t": 9.139108,
            "value": 350.9921875
          },
          {
            "t": 11.055248,
            "value": 373.5546875
          },
          {
            "t": 13.076931,
            "value": 425.1640625
          },
          {
            "t": 15.093361,
            "value": 438.15625
          },
          {
            "t": 17.115285,
            "value": 422.5703125
          },
          {
            "t": 19.130287,
            "value": 438.56640625
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
      "name": "transform-3",
      "metrics": [
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-3 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 3.2432432174682617
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.04901324972981
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.35601990049751
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 848.738671875
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 875.7421875
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 56293.15650182153
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 56180.86849973227
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-3 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000633
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 8901289.9348417
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 8207135.1778521985
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-3 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 158.43987771182495
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.079789,
            "value": 100.12541887262536
          },
          {
            "t": 3.103449,
            "value": 100.05343292872703
          },
          {
            "t": 5.123308,
            "value": 100.09982565379825
          },
          {
            "t": 7.048084,
            "value": 99.99447589424572
          },
          {
            "t": 9.066727,
            "value": 100.35601990049751
          },
          {
            "t": 11.092108,
            "value": 99.77590031152647
          },
          {
            "t": 13.1112,
            "value": 100.27288833437305
          },
          {
            "t": 15.141464,
            "value": 99.83990021827253
          },
          {
            "t": 17.160263,
            "value": 100.15830529595016
          },
          {
            "t": 19.084322,
            "value": 99.8139650872818
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.171296,
            "value": 221628.1356918261
          },
          {
            "t": 1.186205,
            "value": 69957.01092413212
          },
          {
            "t": 2.296192,
            "value": 53153.77567485024
          },
          {
            "t": 3.306133,
            "value": 42576.74458210925
          },
          {
            "t": 4.315497,
            "value": 65387.70948835109
          },
          {
            "t": 5.330331,
            "value": 68976.79817585931
          },
          {
            "t": 6.441271,
            "value": 52208.04003816588
          },
          {
            "t": 7.450848,
            "value": 12876.680035301913
          },
          {
            "t": 8.460834,
            "value": 62377.10225686296
          },
          {
            "t": 9.475141,
            "value": 63097.26739537437
          },
          {
            "t": 10.586373,
            "value": 56693.83171110983
          },
          {
            "t": 11.595718,
            "value": 62416.71579093373
          },
          {
            "t": 12.6052,
            "value": 63398.85208453445
          },
          {
            "t": 13.61964,
            "value": 64074.760458972436
          },
          {
            "t": 14.633884,
            "value": 64087.142738828145
          },
          {
            "t": 15.745733,
            "value": 49467.1488664378
          },
          {
            "t": 16.754975,
            "value": 21798.537912611642
          },
          {
            "t": 17.765325,
            "value": 68293.1657346464
          },
          {
            "t": 18.779937,
            "value": 68006.29206041324
          },
          {
            "t": 19.889501,
            "value": 61285.33369864199
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.06818,
            "value": 40645.852685550795
          },
          {
            "t": 1.079789,
            "value": 50414.73533746734
          },
          {
            "t": 2.094413,
            "value": 54207.27284195919
          },
          {
            "t": 3.103449,
            "value": 58471.65016907226
          },
          {
            "t": 4.113718,
            "value": 57410.452067716615
          },
          {
            "t": 5.123308,
            "value": 48534.55363068176
          },
          {
            "t": 6.139006,
            "value": 63010.855588964434
          },
          {
            "t": 7.14878,
            "value": 52486.99213883503
          },
          {
            "t": 8.158534,
            "value": 47536.33063102498
          },
          {
            "t": 9.167667,
            "value": 58466.029750290596
          },
          {
            "t": 10.182376,
            "value": 61101.26154395004
          },
          {
            "t": 11.192766,
            "value": 60372.72736270154
          },
          {
            "t": 12.202137,
            "value": 59442.960021637235
          },
          {
            "t": 13.211866,
            "value": 57441.15500297604
          },
          {
            "t": 14.225904,
            "value": 55224.754890842356
          },
          {
            "t": 15.141464,
            "value": 65533.66245794923
          },
          {
            "t": 16.150879,
            "value": 53496.33203390083
          },
          {
            "t": 17.160263,
            "value": 51516.568520998946
          },
          {
            "t": 18.170609,
            "value": 60375.35656101969
          },
          {
            "t": 19.185008,
            "value": 53233.49096361491
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.079789,
            "value": 9424684.283000432
          },
          {
            "t": 3.103449,
            "value": 8082875.087712362
          },
          {
            "t": 5.123308,
            "value": 8012843.965841181
          },
          {
            "t": 7.048084,
            "value": 9064402.818821514
          },
          {
            "t": 9.066727,
            "value": 8217978.116982548
          },
          {
            "t": 11.092108,
            "value": 7228654.75680872
          },
          {
            "t": 13.1112,
            "value": 7751024.222769443
          },
          {
            "t": 15.141464,
            "value": 7572516.18508726
          },
          {
            "t": 17.160263,
            "value": 8393269.463676175
          },
          {
            "t": 19.084322,
            "value": 8323102.8778223535
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.079789,
            "value": 6758007.881911892
          },
          {
            "t": 3.103449,
            "value": 9348084.658490062
          },
          {
            "t": 5.123308,
            "value": 8836929.706479512
          },
          {
            "t": 7.048084,
            "value": 9556300.057772957
          },
          {
            "t": 9.066727,
            "value": 9372646.872180965
          },
          {
            "t": 11.092108,
            "value": 8302009.350339517
          },
          {
            "t": 13.1112,
            "value": 8847583.468212444
          },
          {
            "t": 15.141464,
            "value": 8807903.307156114
          },
          {
            "t": 17.160263,
            "value": 9361579.830384303
          },
          {
            "t": 19.084322,
            "value": 9821854.215489235
          }
        ],
        "ram_mib": [
          {
            "t": 1.079789,
            "value": 774.58203125
          },
          {
            "t": 3.103449,
            "value": 835.16015625
          },
          {
            "t": 5.123308,
            "value": 837.0703125
          },
          {
            "t": 7.048084,
            "value": 838.2109375
          },
          {
            "t": 9.066727,
            "value": 862.515625
          },
          {
            "t": 11.092108,
            "value": 862.921875
          },
          {
            "t": 13.1112,
            "value": 863.875
          },
          {
            "t": 15.141464,
            "value": 861.90234375
          },
          {
            "t": 17.160263,
            "value": 875.7421875
          },
          {
            "t": 19.084322,
            "value": 875.40625
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
      "name": "transform-4",
      "metrics": [
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-4 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 3.788546085357666
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.12748319817901
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.4070192906036
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 869.666015625
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 890.8828125
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 57806.65065770471
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 57073.947295345606
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-4 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000589
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9609447.733726945
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 8363822.115336332
        },
        {
          "extra": "OTC OTAP Attr Insert Multi Transform (Logs)/transform-4 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 168.36837452296905
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.066805,
            "value": 99.90636815920398
          },
          {
            "t": 3.092693,
            "value": 99.94786069651741
          },
          {
            "t": 5.113394,
            "value": 100.10319526627218
          },
          {
            "t": 7.043771,
            "value": 100.1672795263322
          },
          {
            "t": 9.064831,
            "value": 100.14046105919003
          },
          {
            "t": 11.091083,
            "value": 100.105686701962
          },
          {
            "t": 13.111967,
            "value": 100.4070192906036
          },
          {
            "t": 15.136302,
            "value": 100.32000000000001
          },
          {
            "t": 17.163383,
            "value": 100.1388660436137
          },
          {
            "t": 19.083215,
            "value": 100.03809523809524
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.157263,
            "value": 229600.08198595204
          },
          {
            "t": 1.173263,
            "value": 74803.14960629921
          },
          {
            "t": 2.284477,
            "value": 54894.91673071074
          },
          {
            "t": 3.294683,
            "value": 61373.620825851365
          },
          {
            "t": 4.305037,
            "value": 34641.323734057565
          },
          {
            "t": 5.320444,
            "value": 75831.66158988465
          },
          {
            "t": 6.336435,
            "value": 62992.683990310936
          },
          {
            "t": 7.447545,
            "value": 47700.0477000477
          },
          {
            "t": 8.458364,
            "value": 58368.51107864019
          },
          {
            "t": 9.468514,
            "value": 56427.26327773102
          },
          {
            "t": 10.483711,
            "value": 61071.890480369824
          },
          {
            "t": 11.595355,
            "value": 43179.29121193475
          },
          {
            "t": 12.606153,
            "value": 58369.723723236486
          },
          {
            "t": 13.616573,
            "value": 59381.247402070425
          },
          {
            "t": 14.626927,
            "value": 44538.844800931154
          },
          {
            "t": 15.645794,
            "value": 70666.73079018164
          },
          {
            "t": 16.75703,
            "value": 16198.17932464391
          },
          {
            "t": 17.768303,
            "value": 72186.24446613327
          },
          {
            "t": 18.778263,
            "value": 81191.33431026971
          },
          {
            "t": 19.791682,
            "value": 71046.62533463453
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.050934,
            "value": 45553.07874443832
          },
          {
            "t": 1.066805,
            "value": 43312.58594841274
          },
          {
            "t": 2.082232,
            "value": 53179.59833646337
          },
          {
            "t": 3.092693,
            "value": 63337.427174329336
          },
          {
            "t": 4.102941,
            "value": 61371.06928199808
          },
          {
            "t": 5.113394,
            "value": 55420.6875530084
          },
          {
            "t": 6.129406,
            "value": 61022.90130431531
          },
          {
            "t": 7.144617,
            "value": 61071.0482845438
          },
          {
            "t": 8.155445,
            "value": 63314.431337477785
          },
          {
            "t": 9.165691,
            "value": 53452.32745291741
          },
          {
            "t": 10.175879,
            "value": 55435.22591834391
          },
          {
            "t": 11.191866,
            "value": 61024.40287129658
          },
          {
            "t": 12.202668,
            "value": 56390.86586690569
          },
          {
            "t": 13.212784,
            "value": 56429.16259122715
          },
          {
            "t": 14.223376,
            "value": 55413.06481745354
          },
          {
            "t": 15.136302,
            "value": 61341.2259043997
          },
          {
            "t": 16.151957,
            "value": 59075.17808704728
          },
          {
            "t": 17.163383,
            "value": 51412.56008842961
          },
          {
            "t": 18.173875,
            "value": 58387.39940543815
          },
          {
            "t": 19.184007,
            "value": 54448.32952525016
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.066805,
            "value": 10591422.543123746
          },
          {
            "t": 3.092693,
            "value": 8533077.346822726
          },
          {
            "t": 5.113394,
            "value": 8111504.37397715
          },
          {
            "t": 7.043771,
            "value": 7952571.440708214
          },
          {
            "t": 9.064831,
            "value": 7956617.814414219
          },
          {
            "t": 11.091083,
            "value": 7593344.756723251
          },
          {
            "t": 13.111967,
            "value": 7915893.737592063
          },
          {
            "t": 15.136302,
            "value": 8248764.656047541
          },
          {
            "t": 17.163383,
            "value": 8113805.023084919
          },
          {
            "t": 19.083215,
            "value": 8621219.460869493
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.066805,
            "value": 8315974.076878798
          },
          {
            "t": 3.092693,
            "value": 9851934.065456728
          },
          {
            "t": 5.113394,
            "value": 9892001.340128995
          },
          {
            "t": 7.043771,
            "value": 9799833.400418675
          },
          {
            "t": 9.064831,
            "value": 9364738.800431458
          },
          {
            "t": 11.091083,
            "value": 9084673.574659025
          },
          {
            "t": 13.111967,
            "value": 9368511.99771981
          },
          {
            "t": 15.136302,
            "value": 9868883.85568594
          },
          {
            "t": 17.163383,
            "value": 9596925.825855011
          },
          {
            "t": 19.083215,
            "value": 10951000.400035003
          }
        ],
        "ram_mib": [
          {
            "t": 1.066805,
            "value": 751.01953125
          },
          {
            "t": 3.092693,
            "value": 882.4921875
          },
          {
            "t": 5.113394,
            "value": 883.4140625
          },
          {
            "t": 7.043771,
            "value": 883.81640625
          },
          {
            "t": 9.064831,
            "value": 884.265625
          },
          {
            "t": 11.091083,
            "value": 884.125
          },
          {
            "t": 13.111967,
            "value": 875.50390625
          },
          {
            "t": 15.136302,
            "value": 876.0390625
          },
          {
            "t": 17.163383,
            "value": 885.1015625
          },
          {
            "t": 19.083215,
            "value": 890.8828125
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
