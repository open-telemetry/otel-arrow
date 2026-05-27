window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otap_none_attr_rename_multi_transform"] = {
  "name": "DFE OTAP Attr Rename Multi Transform (Logs)",
  "slug": "dfe_logs_otap_none_attr_rename_multi_transform",
  "description": "Dataflow Engine OTAP logs, attributes processor rename sweep over 1-4 rename actions at 400k signals/sec",
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
    "started_at": "2026-05-27T17:55:38Z",
    "ended_at": "2026-05-27T18:01:30Z",
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
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-1 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -5.555555820465088
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 12.707531649779307
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 12.856094822208359
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 52.992578125
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 53.29296875
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 385944.70848899323
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 398626.52182886703
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-1 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.010153
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55519539.29007592
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55448543.08217559
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-1 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 139.27708330935096
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.055777,
            "value": 12.57541055260693
          },
          {
            "t": 2.073677,
            "value": 12.686441947565545
          },
          {
            "t": 4.091778,
            "value": 12.690100000000001
          },
          {
            "t": 6.109065,
            "value": 12.700442229834943
          },
          {
            "t": 8.131684,
            "value": 12.856094822208359
          },
          {
            "t": 10.150066,
            "value": 12.657534246575342
          },
          {
            "t": 12.16355,
            "value": 12.65242282507016
          },
          {
            "t": 14.078268,
            "value": 12.6632
          },
          {
            "t": 16.096834,
            "value": 12.739138038725798
          },
          {
            "t": 18.115039,
            "value": 12.854531835205993
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.362399,
            "value": 382645.49616684875
          },
          {
            "t": 1.46887,
            "value": 361509.70066093013
          },
          {
            "t": 2.475508,
            "value": 397362.30899290513
          },
          {
            "t": 3.486803,
            "value": 395532.46085464675
          },
          {
            "t": 4.498522,
            "value": 395366.6976700052
          },
          {
            "t": 5.605178,
            "value": 361449.2669808865
          },
          {
            "t": 6.611145,
            "value": 397627.35755745467
          },
          {
            "t": 7.622378,
            "value": 395556.7114601679
          },
          {
            "t": 8.63906,
            "value": 393436.6891515734
          },
          {
            "t": 9.746011,
            "value": 361352.94154845155
          },
          {
            "t": 10.752358,
            "value": 397477.21213458176
          },
          {
            "t": 11.759495,
            "value": 397165.43032377923
          },
          {
            "t": 12.767849,
            "value": 396686.08445050055
          },
          {
            "t": 13.875349,
            "value": 361173.8148984199
          },
          {
            "t": 14.881034,
            "value": 397738.85461153346
          },
          {
            "t": 15.8933,
            "value": 395153.05265612
          },
          {
            "t": 16.9048,
            "value": 395452.2985664854
          },
          {
            "t": 18.012354,
            "value": 361156.20547621156
          },
          {
            "t": 19.017921,
            "value": 397785.5279658143
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.156175,
            "value": 383133.6887536853
          },
          {
            "t": 1.167477,
            "value": 395529.72306986444
          },
          {
            "t": 2.174037,
            "value": 397393.1012557622
          },
          {
            "t": 3.180348,
            "value": 397491.4315753281
          },
          {
            "t": 4.192192,
            "value": 395317.85532157135
          },
          {
            "t": 5.203597,
            "value": 395489.44290368346
          },
          {
            "t": 6.209466,
            "value": 397666.0976727586
          },
          {
            "t": 7.215782,
            "value": 397489.4565921639
          },
          {
            "t": 8.131684,
            "value": 436727.94687641255
          },
          {
            "t": 9.143745,
            "value": 395233.0936573981
          },
          {
            "t": 10.150066,
            "value": 397487.4816286255
          },
          {
            "t": 11.157115,
            "value": 397200.1362396467
          },
          {
            "t": 12.16355,
            "value": 397442.45778415893
          },
          {
            "t": 13.172873,
            "value": 396305.24618977273
          },
          {
            "t": 14.178654,
            "value": 397700.89114827185
          },
          {
            "t": 15.185599,
            "value": 397241.1601428082
          },
          {
            "t": 16.19724,
            "value": 395397.1814111923
          },
          {
            "t": 17.20951,
            "value": 395151.49120293994
          },
          {
            "t": 18.215444,
            "value": 397640.40185539005
          },
          {
            "t": 19.22164,
            "value": 397536.86160549236
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.055777,
            "value": 56891970.38210288
          },
          {
            "t": 2.073677,
            "value": 55056132.61311264
          },
          {
            "t": 4.091778,
            "value": 54868844.02713244
          },
          {
            "t": 6.109065,
            "value": 55030145.43790744
          },
          {
            "t": 8.131684,
            "value": 54818040.86681673
          },
          {
            "t": 10.150066,
            "value": 54930074.68358319
          },
          {
            "t": 12.16355,
            "value": 55129829.68824187
          },
          {
            "t": 14.078268,
            "value": 57832858.937974155
          },
          {
            "t": 16.096834,
            "value": 55063352.89507502
          },
          {
            "t": 18.115039,
            "value": 54864181.28980951
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.055777,
            "value": 56939024.3902439
          },
          {
            "t": 2.073677,
            "value": 55149880.56890827
          },
          {
            "t": 4.091778,
            "value": 54936607.73172403
          },
          {
            "t": 6.109065,
            "value": 55100494.37685367
          },
          {
            "t": 8.131684,
            "value": 54886260.83310796
          },
          {
            "t": 10.150066,
            "value": 55000151.60658389
          },
          {
            "t": 12.16355,
            "value": 55158644.41932491
          },
          {
            "t": 14.078268,
            "value": 57952505.277539566
          },
          {
            "t": 16.096834,
            "value": 55135891.02362767
          },
          {
            "t": 18.115039,
            "value": 54935932.67284542
          }
        ],
        "ram_mib": [
          {
            "t": 0.055777,
            "value": 52.96484375
          },
          {
            "t": 2.073677,
            "value": 53.12890625
          },
          {
            "t": 4.091778,
            "value": 52.8203125
          },
          {
            "t": 6.109065,
            "value": 53.09765625
          },
          {
            "t": 8.131684,
            "value": 52.80078125
          },
          {
            "t": 10.150066,
            "value": 52.77734375
          },
          {
            "t": 12.16355,
            "value": 52.8125
          },
          {
            "t": 14.078268,
            "value": 52.97265625
          },
          {
            "t": 16.096834,
            "value": 53.2578125
          },
          {
            "t": 18.115039,
            "value": 53.29296875
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
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-2 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -5.555555820465088
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 12.338859450691109
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 12.529684473601998
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 51.015625
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 52.84375
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 388629.5206654288
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 397214.2735813309
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-2 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.00056
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55350631.98031529
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55306457.75375581
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-2 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 139.3470367549168
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.111481,
            "value": 12.439250936329588
          },
          {
            "t": 2.097614,
            "value": 12.298696601184908
          },
          {
            "t": 4.118353,
            "value": 12.529684473601998
          },
          {
            "t": 6.135995,
            "value": 12.133391466832762
          },
          {
            "t": 8.052533,
            "value": 12.42045582266625
          },
          {
            "t": 10.069793,
            "value": 12.526185567010309
          },
          {
            "t": 12.086913,
            "value": 12.225479537644485
          },
          {
            "t": 14.102339,
            "value": 12.172484394506867
          },
          {
            "t": 16.119281,
            "value": 12.212565707133917
          },
          {
            "t": 18.13537,
            "value": 12.4304
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.413423,
            "value": 397182.3881385452
          },
          {
            "t": 1.391406,
            "value": 409005.0645052112
          },
          {
            "t": 2.404541,
            "value": 394814.1165787383
          },
          {
            "t": 3.513154,
            "value": 360811.2118475969
          },
          {
            "t": 4.520831,
            "value": 396952.59492873214
          },
          {
            "t": 5.530335,
            "value": 396234.1902558088
          },
          {
            "t": 6.640767,
            "value": 360220.166565805
          },
          {
            "t": 7.648094,
            "value": 397090.51777625334
          },
          {
            "t": 8.655901,
            "value": 396901.3908416989
          },
          {
            "t": 9.665026,
            "value": 397373.9625913539
          },
          {
            "t": 10.673661,
            "value": 395584.1310285684
          },
          {
            "t": 11.782254,
            "value": 360817.7212015591
          },
          {
            "t": 12.790454,
            "value": 396746.67724657804
          },
          {
            "t": 13.797597,
            "value": 397163.0642321895
          },
          {
            "t": 14.807029,
            "value": 396262.4525475713
          },
          {
            "t": 15.915303,
            "value": 360921.57715510787
          },
          {
            "t": 16.923214,
            "value": 397852.588174948
          },
          {
            "t": 17.931111,
            "value": 395873.78472204995
          },
          {
            "t": 18.940065,
            "value": 396450.1850431239
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.111481,
            "value": 396483.1940686114
          },
          {
            "t": 1.084462,
            "value": 411107.71947242547
          },
          {
            "t": 2.097614,
            "value": 394807.49186696566
          },
          {
            "t": 3.110893,
            "value": 394758.00840637175
          },
          {
            "t": 4.118353,
            "value": 397038.0958052925
          },
          {
            "t": 5.127126,
            "value": 396521.3184730361
          },
          {
            "t": 6.135995,
            "value": 396483.5870663089
          },
          {
            "t": 7.14538,
            "value": 396280.9037186009
          },
          {
            "t": 8.15312,
            "value": 396927.7789906126
          },
          {
            "t": 9.161421,
            "value": 396706.9357265341
          },
          {
            "t": 10.170361,
            "value": 396455.68616567884
          },
          {
            "t": 11.179335,
            "value": 396442.3265614376
          },
          {
            "t": 12.187443,
            "value": 396782.88437349966
          },
          {
            "t": 13.194719,
            "value": 397110.6231062787
          },
          {
            "t": 14.202898,
            "value": 396754.9413348225
          },
          {
            "t": 15.212145,
            "value": 396335.08942805877
          },
          {
            "t": 16.219864,
            "value": 396936.0506252239
          },
          {
            "t": 17.227212,
            "value": 397082.23970266484
          },
          {
            "t": 18.235954,
            "value": 396533.50410709577
          },
          {
            "t": 19.244731,
            "value": 396519.74618771044
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.111481,
            "value": 54934537.44497754
          },
          {
            "t": 2.097614,
            "value": 55734669.329798155
          },
          {
            "t": 4.118353,
            "value": 54978713.23312907
          },
          {
            "t": 6.135995,
            "value": 54791202.30447225
          },
          {
            "t": 8.052533,
            "value": 57970209.304485485
          },
          {
            "t": 10.069793,
            "value": 54803030.34809594
          },
          {
            "t": 12.086913,
            "value": 55081952.982470065
          },
          {
            "t": 14.102339,
            "value": 54848014.265966594
          },
          {
            "t": 16.119281,
            "value": 55083585.44767277
          },
          {
            "t": 18.13537,
            "value": 54838662.87649007
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.111481,
            "value": 54908458.86614745
          },
          {
            "t": 2.097614,
            "value": 55847303.78076393
          },
          {
            "t": 4.118353,
            "value": 55022212.66576238
          },
          {
            "t": 6.135995,
            "value": 54835928.77229955
          },
          {
            "t": 8.052533,
            "value": 58015607.83036913
          },
          {
            "t": 10.069793,
            "value": 54849628.70428205
          },
          {
            "t": 12.086913,
            "value": 55125472.95153487
          },
          {
            "t": 14.102339,
            "value": 54894798.41978817
          },
          {
            "t": 16.119281,
            "value": 55125716.059262
          },
          {
            "t": 18.13537,
            "value": 54881191.75294345
          }
        ],
        "ram_mib": [
          {
            "t": 0.111481,
            "value": 50.734375
          },
          {
            "t": 2.097614,
            "value": 50.40625
          },
          {
            "t": 4.118353,
            "value": 50.4609375
          },
          {
            "t": 6.135995,
            "value": 50.390625
          },
          {
            "t": 8.052533,
            "value": 50.828125
          },
          {
            "t": 10.069793,
            "value": 50.57421875
          },
          {
            "t": 12.086913,
            "value": 50.52734375
          },
          {
            "t": 14.102339,
            "value": 50.8125
          },
          {
            "t": 16.119281,
            "value": 52.84375
          },
          {
            "t": 18.13537,
            "value": 52.578125
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
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-3 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -5.555555820465088
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 12.883831089127911
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 13.13059007180768
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 47.988671875
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 48.4453125
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 386464.40622561955
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 397128.4893028477
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-3 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.005903
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55393650.67858936
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55324127.691482164
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-3 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 139.48546168478612
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.100938,
            "value": 12.960399750156151
          },
          {
            "t": 2.120905,
            "value": 12.961596009975063
          },
          {
            "t": 4.046476,
            "value": 13.0182318025617
          },
          {
            "t": 6.068521,
            "value": 12.861388975397073
          },
          {
            "t": 8.093344,
            "value": 12.925903276131045
          },
          {
            "t": 10.118984,
            "value": 13.13059007180768
          },
          {
            "t": 12.14485,
            "value": 12.726554096310194
          },
          {
            "t": 14.079861,
            "value": 12.947915106117353
          },
          {
            "t": 16.099614,
            "value": 12.527011852776045
          },
          {
            "t": 18.127762,
            "value": 12.778719950046833
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.403521,
            "value": 360114.7685767454
          },
          {
            "t": 1.413402,
            "value": 396086.2715508065
          },
          {
            "t": 2.42511,
            "value": 395370.99637444795
          },
          {
            "t": 3.439377,
            "value": 394373.47365141526
          },
          {
            "t": 4.550285,
            "value": 360065.8200319018
          },
          {
            "t": 5.562197,
            "value": 395291.29015171283
          },
          {
            "t": 6.572357,
            "value": 395976.8749505029
          },
          {
            "t": 7.587317,
            "value": 394104.2011507842
          },
          {
            "t": 8.602035,
            "value": 394198.1910245014
          },
          {
            "t": 9.713267,
            "value": 359960.8362610148
          },
          {
            "t": 10.723206,
            "value": 396063.52462871524
          },
          {
            "t": 11.739161,
            "value": 393718.22570881585
          },
          {
            "t": 12.75958,
            "value": 391995.83700421103
          },
          {
            "t": 13.774829,
            "value": 393992.0157518008
          },
          {
            "t": 14.885577,
            "value": 360117.6864599351
          },
          {
            "t": 15.895448,
            "value": 396090.193698007
          },
          {
            "t": 16.907337,
            "value": 395300.2750301663
          },
          {
            "t": 17.922827,
            "value": 393898.5120483707
          },
          {
            "t": 19.033956,
            "value": 359994.20409331407
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.100938,
            "value": 394022.2878707134
          },
          {
            "t": 1.110839,
            "value": 396078.4274894272
          },
          {
            "t": 2.120905,
            "value": 396013.7258357375
          },
          {
            "t": 3.132317,
            "value": 395486.7057143874
          },
          {
            "t": 4.147125,
            "value": 394163.23087717086
          },
          {
            "t": 5.159144,
            "value": 395249.49630392314
          },
          {
            "t": 6.169221,
            "value": 396009.41314375045
          },
          {
            "t": 7.179534,
            "value": 395916.9089183253
          },
          {
            "t": 8.193972,
            "value": 394306.9955975624
          },
          {
            "t": 9.209453,
            "value": 393902.0030901612
          },
          {
            "t": 10.219659,
            "value": 395958.84403775074
          },
          {
            "t": 11.230324,
            "value": 395779.0167859776
          },
          {
            "t": 12.245542,
            "value": 394004.0464215568
          },
          {
            "t": 13.165521,
            "value": 434792.5333078255
          },
          {
            "t": 14.180578,
            "value": 394066.5401056295
          },
          {
            "t": 15.190898,
            "value": 395914.1658088526
          },
          {
            "t": 16.200287,
            "value": 396279.3333392775
          },
          {
            "t": 17.212865,
            "value": 395031.29635445366
          },
          {
            "t": 18.228532,
            "value": 393829.8674664038
          },
          {
            "t": 19.238321,
            "value": 396122.35823523527
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.100938,
            "value": 54536392.006052196
          },
          {
            "t": 2.120905,
            "value": 55024311.288253725
          },
          {
            "t": 4.046476,
            "value": 57648669.92699828
          },
          {
            "t": 6.068521,
            "value": 54761576.522777684
          },
          {
            "t": 8.093344,
            "value": 54821755.28428905
          },
          {
            "t": 10.118984,
            "value": 54666276.33735511
          },
          {
            "t": 12.14485,
            "value": 54793250.39267158
          },
          {
            "t": 14.079861,
            "value": 57338893.68070776
          },
          {
            "t": 16.099614,
            "value": 54920120.925677545
          },
          {
            "t": 18.127762,
            "value": 54730030.55003876
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.100938,
            "value": 54606877.8763712
          },
          {
            "t": 2.120905,
            "value": 55094183.22180511
          },
          {
            "t": 4.046476,
            "value": 57648836.11147031
          },
          {
            "t": 6.068521,
            "value": 54898363.290629044
          },
          {
            "t": 8.093344,
            "value": 54891032.944607995
          },
          {
            "t": 10.118984,
            "value": 54735043.245591514
          },
          {
            "t": 12.14485,
            "value": 54863442.59689436
          },
          {
            "t": 14.079861,
            "value": 57368051.13769379
          },
          {
            "t": 16.099614,
            "value": 55030089.322803326
          },
          {
            "t": 18.127762,
            "value": 54800587.03802682
          }
        ],
        "ram_mib": [
          {
            "t": 0.100938,
            "value": 47.75390625
          },
          {
            "t": 2.120905,
            "value": 48.01953125
          },
          {
            "t": 4.046476,
            "value": 47.59765625
          },
          {
            "t": 6.068521,
            "value": 48.1875
          },
          {
            "t": 8.093344,
            "value": 48.4453125
          },
          {
            "t": 10.118984,
            "value": 47.80859375
          },
          {
            "t": 12.14485,
            "value": 48.0546875
          },
          {
            "t": 14.079861,
            "value": 48.0859375
          },
          {
            "t": 16.099614,
            "value": 47.90234375
          },
          {
            "t": 18.127762,
            "value": 48.03125
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
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-4 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -5.555555820465088
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 12.81356312839648
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 13.063400683866957
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 52.24375
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 52.6953125
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 385753.37768604234
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 407184.12089082244
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-4 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.002127
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55284816.453897156
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55212846.953459784
        },
        {
          "extra": "DFE OTAP Attr Rename Multi Transform (Logs)/transform-4 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 135.77350789845897
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.064331,
            "value": 13.063400683866957
          },
          {
            "t": 2.094979,
            "value": 12.9990183286735
          },
          {
            "t": 4.122698,
            "value": 12.607145076110593
          },
          {
            "t": 6.051041,
            "value": 12.505066832452597
          },
          {
            "t": 8.083075,
            "value": 12.860217729393469
          },
          {
            "t": 10.106278,
            "value": 12.858580323785803
          },
          {
            "t": 12.139762,
            "value": 12.838603491271822
          },
          {
            "t": 14.067494,
            "value": 12.820597014925372
          },
          {
            "t": 16.095602,
            "value": 12.71397635345364
          },
          {
            "t": 18.129369,
            "value": 12.869025450031035
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.371792,
            "value": 393179.5149540914
          },
          {
            "t": 1.385579,
            "value": 394560.1985426919
          },
          {
            "t": 2.503939,
            "value": 357666.5832111306
          },
          {
            "t": 3.515068,
            "value": 395597.3965735332
          },
          {
            "t": 4.526199,
            "value": 395596.61408858
          },
          {
            "t": 5.543271,
            "value": 394269.0389667595
          },
          {
            "t": 6.558055,
            "value": 393187.12159434916
          },
          {
            "t": 7.676836,
            "value": 358425.8223906198
          },
          {
            "t": 8.687816,
            "value": 394666.5611584799
          },
          {
            "t": 9.699711,
            "value": 395297.9311094531
          },
          {
            "t": 10.716219,
            "value": 393504.0353838829
          },
          {
            "t": 11.732976,
            "value": 393407.6677121476
          },
          {
            "t": 12.850783,
            "value": 357843.5275499259
          },
          {
            "t": 13.861599,
            "value": 395719.8936304926
          },
          {
            "t": 14.874109,
            "value": 395057.826589367
          },
          {
            "t": 15.890025,
            "value": 393733.34015804454
          },
          {
            "t": 16.906398,
            "value": 393556.30265660345
          },
          {
            "t": 18.02472,
            "value": 357678.73653563106
          },
          {
            "t": 19.036567,
            "value": 395316.68325349584
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.371792,
            "value": 393179.5149540914
          },
          {
            "t": 1.48816,
            "value": 716609.576770384
          },
          {
            "t": 2.503939,
            "value": 393786.4437047822
          },
          {
            "t": 3.515068,
            "value": 395597.3965735332
          },
          {
            "t": 4.526199,
            "value": 395596.61408858
          },
          {
            "t": 5.543271,
            "value": 393285.82440574514
          },
          {
            "t": 6.660666,
            "value": 357975.4697309367
          },
          {
            "t": 7.676836,
            "value": 393634.9232903943
          },
          {
            "t": 8.687816,
            "value": 395655.70040950365
          },
          {
            "t": 9.699711,
            "value": 395297.9311094531
          },
          {
            "t": 10.716219,
            "value": 393504.0353838829
          },
          {
            "t": 11.835742,
            "value": 357295.02654255426
          },
          {
            "t": 12.850783,
            "value": 394072.7517410626
          },
          {
            "t": 13.861599,
            "value": 395719.8936304926
          },
          {
            "t": 14.874109,
            "value": 395057.826589367
          },
          {
            "t": 15.890025,
            "value": 393733.34015804454
          },
          {
            "t": 17.009145,
            "value": 357423.69004217605
          },
          {
            "t": 18.02472,
            "value": 393865.54414986586
          },
          {
            "t": 19.036567,
            "value": 395316.68325349584
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.064331,
            "value": 54713593.56670532
          },
          {
            "t": 2.094979,
            "value": 54661413.00707952
          },
          {
            "t": 4.122698,
            "value": 54598848.755670786
          },
          {
            "t": 6.051041,
            "value": 57630389.92544376
          },
          {
            "t": 8.083075,
            "value": 54553890.33844906
          },
          {
            "t": 10.106278,
            "value": 54792849.25931802
          },
          {
            "t": 12.139762,
            "value": 54511086.391631305
          },
          {
            "t": 14.067494,
            "value": 57483399.14469439
          },
          {
            "t": 16.095602,
            "value": 54672843.852497004
          },
          {
            "t": 18.129369,
            "value": 54510155.2931088
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.064331,
            "value": 54844460.3557208
          },
          {
            "t": 2.094979,
            "value": 54724746.97731956
          },
          {
            "t": 4.122698,
            "value": 54664937.79463526
          },
          {
            "t": 6.051041,
            "value": 57699676.35425855
          },
          {
            "t": 8.083075,
            "value": 54620212.06338083
          },
          {
            "t": 10.106278,
            "value": 54789816.44451891
          },
          {
            "t": 12.139762,
            "value": 54643339.70663157
          },
          {
            "t": 14.067494,
            "value": 57500867.34048094
          },
          {
            "t": 16.095602,
            "value": 54786844.68479982
          },
          {
            "t": 18.129369,
            "value": 54573262.817225374
          }
        ],
        "ram_mib": [
          {
            "t": 0.064331,
            "value": 51.90625
          },
          {
            "t": 2.094979,
            "value": 52.0703125
          },
          {
            "t": 4.122698,
            "value": 52.359375
          },
          {
            "t": 6.051041,
            "value": 52.2265625
          },
          {
            "t": 8.083075,
            "value": 52.6953125
          },
          {
            "t": 10.106278,
            "value": 52.07421875
          },
          {
            "t": 12.139762,
            "value": 52.25390625
          },
          {
            "t": 14.067494,
            "value": 52.54296875
          },
          {
            "t": 16.095602,
            "value": 52.4375
          },
          {
            "t": 18.129369,
            "value": 51.87109375
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
