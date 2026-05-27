window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otap_none_attr_rename_multi_transform"] = {
  "name": "OTC OTAP Attr Rename Multi Transform (Logs)",
  "slug": "otc_logs_otap_none_attr_rename_multi_transform",
  "description": "OpenTelemetry Collector OTAP logs, attributes processor rename sweep over 1-4 rename actions at 240k signals/sec",
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
    "started_at": "2026-05-27T23:16:35Z",
    "ended_at": "2026-05-27T23:20:48Z",
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
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-1 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 7.018813610076904
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.1370267045568
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.243018397256
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 321.7375
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 354.68359375
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 70461.88735887228
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 67245.83911791461
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-1 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000573
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 8950869.352923838
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9427201.481104603
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-1 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 133.10666459568773
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.028873,
            "value": 99.86504521359527
          },
          {
            "t": 3.048014,
            "value": 100.17914615144905
          },
          {
            "t": 5.067772,
            "value": 100.10362900715842
          },
          {
            "t": 7.087279,
            "value": 100.14594392523364
          },
          {
            "t": 9.138353,
            "value": 100.20347771891555
          },
          {
            "t": 11.052714,
            "value": 100.17465877220317
          },
          {
            "t": 13.078063,
            "value": 100.23620947630923
          },
          {
            "t": 15.093071,
            "value": 99.98711843332298
          },
          {
            "t": 17.11719,
            "value": 100.243018397256
          },
          {
            "t": 19.131414,
            "value": 100.2320199501247
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.22363,
            "value": 156994.60653971962
          },
          {
            "t": 1.230135,
            "value": 78489.42628203536
          },
          {
            "t": 2.241736,
            "value": 75128.43502527181
          },
          {
            "t": 3.355311,
            "value": 68248.65859955549
          },
          {
            "t": 4.362031,
            "value": 78472.66369993643
          },
          {
            "t": 5.369567,
            "value": 76424.06822187992
          },
          {
            "t": 6.381518,
            "value": 79055.21117129188
          },
          {
            "t": 7.421279,
            "value": 74055.48005743627
          },
          {
            "t": 8.534509,
            "value": 68269.80947333436
          },
          {
            "t": 9.540451,
            "value": 77539.26170693738
          },
          {
            "t": 10.548303,
            "value": 74415.68801768514
          },
          {
            "t": 11.560459,
            "value": 59279.39961824066
          },
          {
            "t": 12.674299,
            "value": 68232.42117359766
          },
          {
            "t": 13.681173,
            "value": 68528.93212060297
          },
          {
            "t": 14.689166,
            "value": 59524.22288646846
          },
          {
            "t": 15.700282,
            "value": 35604.22345210639
          },
          {
            "t": 16.813353,
            "value": 67381.14639587233
          },
          {
            "t": 17.820534,
            "value": 78436.7457289206
          },
          {
            "t": 18.827462,
            "value": 73490.85535410675
          },
          {
            "t": 19.83707,
            "value": 79238.67481240243
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.122851,
            "value": 84443.60111227125
          },
          {
            "t": 1.129279,
            "value": 61604.00942740066
          },
          {
            "t": 2.135911,
            "value": 54637.64315062506
          },
          {
            "t": 3.148681,
            "value": 82940.84540418851
          },
          {
            "t": 4.160639,
            "value": 75101.93110781278
          },
          {
            "t": 5.168282,
            "value": 65499.38817616953
          },
          {
            "t": 6.175275,
            "value": 67527.77824672069
          },
          {
            "t": 7.1877,
            "value": 55312.73921525052
          },
          {
            "t": 8.126419,
            "value": 76700.26919663926
          },
          {
            "t": 9.138353,
            "value": 72139.09207517488
          },
          {
            "t": 10.146303,
            "value": 69447.88928022224
          },
          {
            "t": 11.153095,
            "value": 64561.49830352248
          },
          {
            "t": 12.165998,
            "value": 69108.29566108501
          },
          {
            "t": 13.178659,
            "value": 63199.82699047361
          },
          {
            "t": 14.185875,
            "value": 59570.141856364484
          },
          {
            "t": 15.193498,
            "value": 64508.25358293727
          },
          {
            "t": 16.204941,
            "value": 63275.93349303915
          },
          {
            "t": 17.217654,
            "value": 80970.62050156362
          },
          {
            "t": 18.224675,
            "value": 70504.98450379883
          },
          {
            "t": 19.23184,
            "value": 61558.93026465375
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.028873,
            "value": 9319797.68046806
          },
          {
            "t": 3.048014,
            "value": 9398027.180865528
          },
          {
            "t": 5.067772,
            "value": 9625515.532058792
          },
          {
            "t": 7.087279,
            "value": 9497550.639834376
          },
          {
            "t": 9.138353,
            "value": 8933391.969280485
          },
          {
            "t": 11.052714,
            "value": 9728593.509792563
          },
          {
            "t": 13.078063,
            "value": 9147405.706374556
          },
          {
            "t": 15.093071,
            "value": 9748987.10079563
          },
          {
            "t": 17.11719,
            "value": 9155743.807552818
          },
          {
            "t": 19.131414,
            "value": 9717001.684023228
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.028873,
            "value": 8276673.897094799
          },
          {
            "t": 3.048014,
            "value": 9751594.86137917
          },
          {
            "t": 5.067772,
            "value": 8446252.966939604
          },
          {
            "t": 7.087279,
            "value": 9622868.848684357
          },
          {
            "t": 9.138353,
            "value": 8640513.701602185
          },
          {
            "t": 11.052714,
            "value": 8838984.392180994
          },
          {
            "t": 13.078063,
            "value": 9205533.959826184
          },
          {
            "t": 15.093071,
            "value": 8914661.877272945
          },
          {
            "t": 17.11719,
            "value": 9215121.245341802
          },
          {
            "t": 19.131414,
            "value": 8596487.778916348
          }
        ],
        "ram_mib": [
          {
            "t": 1.028873,
            "value": 328.85546875
          },
          {
            "t": 3.048014,
            "value": 322.12109375
          },
          {
            "t": 5.067772,
            "value": 308.8671875
          },
          {
            "t": 7.087279,
            "value": 343.84375
          },
          {
            "t": 9.138353,
            "value": 299.890625
          },
          {
            "t": 11.052714,
            "value": 323.78125
          },
          {
            "t": 13.078063,
            "value": 318.23828125
          },
          {
            "t": 15.093071,
            "value": 354.68359375
          },
          {
            "t": 17.11719,
            "value": 297.64453125
          },
          {
            "t": 19.131414,
            "value": 319.44921875
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
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-2 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -4.724409580230713
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.28680926071966
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.47371766905577
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 255.545703125
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 331.546875
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 64591.61771690094
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 67297.34908658016
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-2 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000619
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 8740941.657579288
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9164675.292706471
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-2 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 129.88537849141414
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.09015,
            "value": 100.29695760598503
          },
          {
            "t": 3.064667,
            "value": 100.32039900249376
          },
          {
            "t": 5.086492,
            "value": 100.18102771722207
          },
          {
            "t": 7.104611,
            "value": 100.47371766905577
          },
          {
            "t": 9.127202,
            "value": 100.32748129675811
          },
          {
            "t": 11.148199,
            "value": 100.22246575342466
          },
          {
            "t": 13.07113,
            "value": 100.0418018018018
          },
          {
            "t": 15.092734,
            "value": 100.43499688861233
          },
          {
            "t": 17.115319,
            "value": 100.40396138274681
          },
          {
            "t": 19.135068,
            "value": 100.16528348909657
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.182206,
            "value": 133786.68852179585
          },
          {
            "t": 1.195116,
            "value": 79967.61805096209
          },
          {
            "t": 2.257682,
            "value": 72466.08681248978
          },
          {
            "t": 3.269417,
            "value": 60292.46788931884
          },
          {
            "t": 4.378728,
            "value": 58594.9296455187
          },
          {
            "t": 5.388233,
            "value": 77265.59056171092
          },
          {
            "t": 6.396798,
            "value": 47592.37133947738
          },
          {
            "t": 7.411616,
            "value": 86715.05629580871
          },
          {
            "t": 8.520901,
            "value": 53187.41351411044
          },
          {
            "t": 9.529523,
            "value": 62461.45731502981
          },
          {
            "t": 10.54137,
            "value": 72145.294693763
          },
          {
            "t": 11.555473,
            "value": 64096.05335947138
          },
          {
            "t": 12.665633,
            "value": 54947.03466167038
          },
          {
            "t": 13.674164,
            "value": 73374.04601345917
          },
          {
            "t": 14.686975,
            "value": 65165.16901968877
          },
          {
            "t": 15.701152,
            "value": 54231.16477695707
          },
          {
            "t": 16.810178,
            "value": 71233.67711848054
          },
          {
            "t": 17.819551,
            "value": 67368.5545383124
          },
          {
            "t": 18.829962,
            "value": 51464.20614977469
          },
          {
            "t": 19.8442,
            "value": 56199.82686509479
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.182206,
            "value": 71352.90054495778
          },
          {
            "t": 1.297386,
            "value": 58286.554636919594
          },
          {
            "t": 2.257682,
            "value": 63522.080691786694
          },
          {
            "t": 3.371425,
            "value": 67340.49057996324
          },
          {
            "t": 4.378728,
            "value": 64528.746563844245
          },
          {
            "t": 5.388233,
            "value": 64387.99213475911
          },
          {
            "t": 6.500763,
            "value": 61121.947273331956
          },
          {
            "t": 7.513041,
            "value": 67175.22261671201
          },
          {
            "t": 8.520901,
            "value": 132954.97390510587
          },
          {
            "t": 9.529523,
            "value": 63452.90901844299
          },
          {
            "t": 10.644733,
            "value": 60975.062992620224
          },
          {
            "t": 11.656786,
            "value": 64225.8854032348
          },
          {
            "t": 12.665633,
            "value": 65421.218480106494
          },
          {
            "t": 13.674164,
            "value": 67424.79903939491
          },
          {
            "t": 14.790273,
            "value": 57342.06963656775
          },
          {
            "t": 15.80256,
            "value": 62235.31468842334
          },
          {
            "t": 16.810178,
            "value": 54584.177734022225
          },
          {
            "t": 17.819551,
            "value": 71331.41068762488
          },
          {
            "t": 18.932587,
            "value": 67383.26523131327
          },
          {
            "t": 19.945243,
            "value": 68137.64990282978
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.09015,
            "value": 8968793.508597756
          },
          {
            "t": 3.064667,
            "value": 9179878.927352866
          },
          {
            "t": 5.086492,
            "value": 9308285.336267974
          },
          {
            "t": 7.104611,
            "value": 9493530.857199203
          },
          {
            "t": 9.127202,
            "value": 8793428.330295151
          },
          {
            "t": 11.148199,
            "value": 9144090.76312335
          },
          {
            "t": 13.07113,
            "value": 9736645.776681535
          },
          {
            "t": 15.092734,
            "value": 8717092.961826352
          },
          {
            "t": 17.115319,
            "value": 9594674.142248658
          },
          {
            "t": 19.135068,
            "value": 8710332.323471878
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.09015,
            "value": 8709630.092968687
          },
          {
            "t": 3.064667,
            "value": 8860096.418516528
          },
          {
            "t": 5.086492,
            "value": 8900511.666439973
          },
          {
            "t": 7.104611,
            "value": 8785368.454486579
          },
          {
            "t": 9.127202,
            "value": 8766471.817584476
          },
          {
            "t": 11.148199,
            "value": 8774080.317783747
          },
          {
            "t": 13.07113,
            "value": 8950740.82221359
          },
          {
            "t": 15.092734,
            "value": 8772323.857689241
          },
          {
            "t": 17.115319,
            "value": 8533963.220334375
          },
          {
            "t": 19.135068,
            "value": 8356229.907775669
          }
        ],
        "ram_mib": [
          {
            "t": 1.09015,
            "value": 232.75
          },
          {
            "t": 3.064667,
            "value": 244.0390625
          },
          {
            "t": 5.086492,
            "value": 251.99609375
          },
          {
            "t": 7.104611,
            "value": 255.3203125
          },
          {
            "t": 9.127202,
            "value": 241.69140625
          },
          {
            "t": 11.148199,
            "value": 235.296875
          },
          {
            "t": 13.07113,
            "value": 241.04296875
          },
          {
            "t": 15.092734,
            "value": 235.296875
          },
          {
            "t": 17.115319,
            "value": 331.546875
          },
          {
            "t": 19.135068,
            "value": 286.4765625
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
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-3 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.5177514553070068
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.1748652741159
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.33796631316282
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 214.228515625
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 222.5859375
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 68581.17718576125
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 68226.09712636752
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-3 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000618
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 8936854.391256485
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9337043.319604559
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-3 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 130.98879706842612
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.031587,
            "value": 100.19327521793275
          },
          {
            "t": 3.050428,
            "value": 100.14296918767506
          },
          {
            "t": 5.068141,
            "value": 100.18532087227415
          },
          {
            "t": 7.087529,
            "value": 99.9897046938141
          },
          {
            "t": 9.104893,
            "value": 100.3059806672903
          },
          {
            "t": 11.136898,
            "value": 100.23568712994701
          },
          {
            "t": 13.060265,
            "value": 99.88001247271593
          },
          {
            "t": 15.082298,
            "value": 100.33796631316282
          },
          {
            "t": 17.105888,
            "value": 100.19228900654002
          },
          {
            "t": 19.127348,
            "value": 100.28544717980678
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.223509,
            "value": 58514.9113846215
          },
          {
            "t": 1.232776,
            "value": 145650.25904938934
          },
          {
            "t": 2.242522,
            "value": 58430.53599618122
          },
          {
            "t": 3.352565,
            "value": 53151.09414680333
          },
          {
            "t": 4.361303,
            "value": 66419.62531400622
          },
          {
            "t": 5.369857,
            "value": 65440.22432115683
          },
          {
            "t": 6.380384,
            "value": 77187.4477376656
          },
          {
            "t": 7.489794,
            "value": 55885.560793574965
          },
          {
            "t": 8.49855,
            "value": 78314.2801628937
          },
          {
            "t": 9.507116,
            "value": 39660.27012609983
          },
          {
            "t": 10.529192,
            "value": 93926.47904852476
          },
          {
            "t": 11.645508,
            "value": 58227.24031546623
          },
          {
            "t": 12.653643,
            "value": 68443.21445044562
          },
          {
            "t": 13.663416,
            "value": 68332.1895119002
          },
          {
            "t": 14.676213,
            "value": 49368.23470053723
          },
          {
            "t": 15.792323,
            "value": 64509.770542330065
          },
          {
            "t": 16.800937,
            "value": 60479.033604530574
          },
          {
            "t": 17.80962,
            "value": 79311.33963792391
          },
          {
            "t": 18.822369,
            "value": 57269.86647234409
          },
          {
            "t": 19.937373,
            "value": 67264.3326840083
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.223509,
            "value": 65457.35849805116
          },
          {
            "t": 1.232776,
            "value": 67375.6300364522
          },
          {
            "t": 2.345541,
            "value": 60210.376854052745
          },
          {
            "t": 3.352565,
            "value": 65539.64950189868
          },
          {
            "t": 4.361303,
            "value": 66419.62531400622
          },
          {
            "t": 5.369857,
            "value": 67423.26142179794
          },
          {
            "t": 6.482481,
            "value": 60218.00716144897
          },
          {
            "t": 7.489794,
            "value": 68499.06632794376
          },
          {
            "t": 8.49855,
            "value": 134819.5202804246
          },
          {
            "t": 9.507116,
            "value": 66430.95246121721
          },
          {
            "t": 10.633334,
            "value": 61267.00159294205
          },
          {
            "t": 11.645508,
            "value": 68170.09723624594
          },
          {
            "t": 12.653643,
            "value": 67451.28380623627
          },
          {
            "t": 13.663416,
            "value": 63380.58157625525
          },
          {
            "t": 14.77958,
            "value": 60027.02111876033
          },
          {
            "t": 15.792323,
            "value": 63194.709812854795
          },
          {
            "t": 16.800937,
            "value": 67419.25057554229
          },
          {
            "t": 17.80962,
            "value": 67414.63869223533
          },
          {
            "t": 18.925252,
            "value": 59159.29266998437
          },
          {
            "t": 19.937373,
            "value": 66197.61866417158
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.031587,
            "value": 9710953.29820174
          },
          {
            "t": 3.050428,
            "value": 9335183.900069395
          },
          {
            "t": 5.068141,
            "value": 9285401.34300567
          },
          {
            "t": 7.087529,
            "value": 9300594.536562562
          },
          {
            "t": 9.104893,
            "value": 9253526.383934679
          },
          {
            "t": 11.136898,
            "value": 9405574.789432114
          },
          {
            "t": 13.060265,
            "value": 9764979.330517784
          },
          {
            "t": 15.082298,
            "value": 8896962.116839834
          },
          {
            "t": 17.105888,
            "value": 8864628.704431234
          },
          {
            "t": 19.127348,
            "value": 9552628.793050569
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.031587,
            "value": 9255320.058694026
          },
          {
            "t": 3.050428,
            "value": 8921087.891517954
          },
          {
            "t": 5.068141,
            "value": 8797746.26024613
          },
          {
            "t": 7.087529,
            "value": 8983485.095484374
          },
          {
            "t": 9.104893,
            "value": 8926385.620046753
          },
          {
            "t": 11.136898,
            "value": 8928169.960211713
          },
          {
            "t": 13.060265,
            "value": 9365243.346693585
          },
          {
            "t": 15.082298,
            "value": 8513378.861769319
          },
          {
            "t": 17.105888,
            "value": 8507205.01682653
          },
          {
            "t": 19.127348,
            "value": 9170521.801074471
          }
        ],
        "ram_mib": [
          {
            "t": 1.031587,
            "value": 210.66796875
          },
          {
            "t": 3.050428,
            "value": 211.734375
          },
          {
            "t": 5.068141,
            "value": 211.76171875
          },
          {
            "t": 7.087529,
            "value": 215.83203125
          },
          {
            "t": 9.104893,
            "value": 220.0625
          },
          {
            "t": 11.136898,
            "value": 214.04296875
          },
          {
            "t": 13.060265,
            "value": 210.7890625
          },
          {
            "t": 15.082298,
            "value": 222.5859375
          },
          {
            "t": 17.105888,
            "value": 212.58984375
          },
          {
            "t": 19.127348,
            "value": 212.21875
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
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-4 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 14.243973731994629
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.05289702714961
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.22805113813533
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 806.390625
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 849.20703125
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 69625.06925634628
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 61274.12283666203
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-4 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000631
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 8196459.004024635
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9111710.836635575
        },
        {
          "extra": "OTC OTAP Attr Rename Multi Transform (Logs)/transform-4 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 133.76705572552828
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.071219,
            "value": 100.13856697819314
          },
          {
            "t": 3.098853,
            "value": 99.95898844832969
          },
          {
            "t": 5.120812,
            "value": 100.10488944254125
          },
          {
            "t": 7.051892,
            "value": 99.64359788359788
          },
          {
            "t": 9.073681,
            "value": 99.97765474339036
          },
          {
            "t": 11.100704,
            "value": 100.07163138231631
          },
          {
            "t": 13.126539,
            "value": 100.16418822062947
          },
          {
            "t": 15.152772,
            "value": 100.22805113813533
          },
          {
            "t": 17.095334,
            "value": 100.07362391033622
          },
          {
            "t": 19.117991,
            "value": 100.16777812402617
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.162496,
            "value": 125464.68860553407
          },
          {
            "t": 1.279212,
            "value": 170141.7370217674
          },
          {
            "t": 2.289377,
            "value": 70285.54741057154
          },
          {
            "t": 3.301176,
            "value": 77090.41024946656
          },
          {
            "t": 4.311669,
            "value": 63335.42142300837
          },
          {
            "t": 5.328001,
            "value": 60019.757323394326
          },
          {
            "t": 6.445366,
            "value": 42958.20971661006
          },
          {
            "t": 7.455522,
            "value": 69296.22751337418
          },
          {
            "t": 8.466853,
            "value": 73170.90052613833
          },
          {
            "t": 9.477256,
            "value": 63341.06292241809
          },
          {
            "t": 10.595744,
            "value": 57220.10428364006
          },
          {
            "t": 11.605004,
            "value": 76293.52198640587
          },
          {
            "t": 12.620321,
            "value": 60079.7583414835
          },
          {
            "t": 13.631043,
            "value": 62331.679729935626
          },
          {
            "t": 14.646642,
            "value": 65970.91962477317
          },
          {
            "t": 15.763815,
            "value": 55497.22379613543
          },
          {
            "t": 16.773958,
            "value": 64347.325081696355
          },
          {
            "t": 17.801379,
            "value": 59371.96144521088
          },
          {
            "t": 18.812103,
            "value": 67278.50530906559
          },
          {
            "t": 19.824954,
            "value": 60226.03522137018
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.059194,
            "value": 61361.71543603437
          },
          {
            "t": 1.071219,
            "value": 57310.83718287592
          },
          {
            "t": 2.087311,
            "value": 46255.65401558127
          },
          {
            "t": 3.098853,
            "value": 56349.61276941541
          },
          {
            "t": 4.10932,
            "value": 54430.2782772718
          },
          {
            "t": 5.120812,
            "value": 63272.86819866098
          },
          {
            "t": 6.136798,
            "value": 62992.99399794879
          },
          {
            "t": 7.15269,
            "value": 64967.53591917251
          },
          {
            "t": 8.163706,
            "value": 63302.65792034943
          },
          {
            "t": 9.174491,
            "value": 62327.79473379601
          },
          {
            "t": 10.185845,
            "value": 63281.50182824214
          },
          {
            "t": 11.201634,
            "value": 63005.21072781847
          },
          {
            "t": 12.116275,
            "value": 69972.8090037512
          },
          {
            "t": 13.126539,
            "value": 62359.93760046879
          },
          {
            "t": 14.137066,
            "value": 61354.125124811115
          },
          {
            "t": 15.152772,
            "value": 63010.35929688315
          },
          {
            "t": 16.16913,
            "value": 62969.93775815214
          },
          {
            "t": 17.196322,
            "value": 66199.89252252743
          },
          {
            "t": 18.207303,
            "value": 58359.15808506787
          },
          {
            "t": 19.218995,
            "value": 63260.359872372224
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.071219,
            "value": 10227216.112126667
          },
          {
            "t": 3.098853,
            "value": 9833497.070970403
          },
          {
            "t": 5.120812,
            "value": 8422837.95072007
          },
          {
            "t": 7.051892,
            "value": 9100610.021335213
          },
          {
            "t": 9.073681,
            "value": 9125745.56494273
          },
          {
            "t": 11.100704,
            "value": 9460511.794883434
          },
          {
            "t": 13.126539,
            "value": 8314660.86823458
          },
          {
            "t": 15.152772,
            "value": 9162795.196801158
          },
          {
            "t": 17.095334,
            "value": 9023959.080842722
          },
          {
            "t": 19.117991,
            "value": 8445274.705498757
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.071219,
            "value": 7021030.178097899
          },
          {
            "t": 3.098853,
            "value": 7264581.773633704
          },
          {
            "t": 5.120812,
            "value": 8311866.363264537
          },
          {
            "t": 7.051892,
            "value": 8426427.698489964
          },
          {
            "t": 9.073681,
            "value": 8835910.176581236
          },
          {
            "t": 11.100704,
            "value": 8295037.106140385
          },
          {
            "t": 13.126539,
            "value": 8307326.60853426
          },
          {
            "t": 15.152772,
            "value": 8820517.186325561
          },
          {
            "t": 17.095334,
            "value": 8637899.330883648
          },
          {
            "t": 19.117991,
            "value": 8043993.618295142
          }
        ],
        "ram_mib": [
          {
            "t": 1.071219,
            "value": 573.78125
          },
          {
            "t": 3.098853,
            "value": 710.9375
          },
          {
            "t": 5.120812,
            "value": 841.953125
          },
          {
            "t": 7.051892,
            "value": 844.69921875
          },
          {
            "t": 9.073681,
            "value": 848.83984375
          },
          {
            "t": 11.100704,
            "value": 849.1953125
          },
          {
            "t": 13.126539,
            "value": 847.6953125
          },
          {
            "t": 15.152772,
            "value": 848.68359375
          },
          {
            "t": 17.095334,
            "value": 848.9140625
          },
          {
            "t": 19.117991,
            "value": 849.20703125
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
