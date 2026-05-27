window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otap_none_transform_insert_multi_transform"] = {
  "name": "DFE OTAP Transform Insert Multi Transform (Logs)",
  "slug": "dfe_logs_otap_none_transform_insert_multi_transform",
  "description": "Dataflow Engine OTAP logs, transform processor (OPL) insert sweep over 1-4 insert actions at 400k signals/sec",
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
    "started_at": "2026-05-27T18:14:20Z",
    "ended_at": "2026-05-27T18:20:05Z",
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
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-1 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -5.555555820465088
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 41.73842923294049
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 42.452902021772935
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 57.5875
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 57.7578125
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 388314.24373320135
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 396618.5347460538
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-1 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.01083
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 69185268.99302927
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55345653.65070593
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-1 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 174.43781097453524
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.028751,
            "value": 41.68443613707165
          },
          {
            "t": 2.057049,
            "value": 41.65168693483005
          },
          {
            "t": 4.074963,
            "value": 42.452902021772935
          },
          {
            "t": 6.088352,
            "value": 41.55968895800933
          },
          {
            "t": 8.106779,
            "value": 41.67857899657214
          },
          {
            "t": 10.120759,
            "value": 41.87594130502654
          },
          {
            "t": 12.139238,
            "value": 41.73626127527216
          },
          {
            "t": 14.152827,
            "value": 41.339469909572806
          },
          {
            "t": 16.071028,
            "value": 42.1659
          },
          {
            "t": 18.084386,
            "value": 41.23942679127725
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.446737,
            "value": 356224.62456150976
          },
          {
            "t": 1.452704,
            "value": 397627.35755745467
          },
          {
            "t": 2.458688,
            "value": 397620.63810160005
          },
          {
            "t": 3.572186,
            "value": 359228.3057535801
          },
          {
            "t": 4.577678,
            "value": 397815.1989274902
          },
          {
            "t": 5.584804,
            "value": 397169.7682315818
          },
          {
            "t": 6.591274,
            "value": 397428.6367204189
          },
          {
            "t": 7.704349,
            "value": 360263.2347326101
          },
          {
            "t": 8.709823,
            "value": 396827.76481540053
          },
          {
            "t": 9.717209,
            "value": 397067.2612087125
          },
          {
            "t": 10.723853,
            "value": 397359.94055495283
          },
          {
            "t": 11.836988,
            "value": 359345.45225871075
          },
          {
            "t": 12.842612,
            "value": 397762.98099488474
          },
          {
            "t": 13.849674,
            "value": 397195.0088475189
          },
          {
            "t": 14.856302,
            "value": 397366.2564522346
          },
          {
            "t": 15.969258,
            "value": 360301.75496605434
          },
          {
            "t": 16.974997,
            "value": 396723.2055235007
          },
          {
            "t": 17.982093,
            "value": 397181.59937086434
          },
          {
            "t": 18.988421,
            "value": 397484.71671264246
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.129856,
            "value": 396962.8373315761
          },
          {
            "t": 1.151316,
            "value": 391596.3424901611
          },
          {
            "t": 2.157392,
            "value": 397584.27792731364
          },
          {
            "t": 3.163813,
            "value": 397447.9864788195
          },
          {
            "t": 4.175343,
            "value": 395440.57022530225
          },
          {
            "t": 5.182433,
            "value": 397183.96568330535
          },
          {
            "t": 6.188748,
            "value": 397489.8515872266
          },
          {
            "t": 7.195678,
            "value": 396253.9600568063
          },
          {
            "t": 8.20713,
            "value": 396459.7430229017
          },
          {
            "t": 9.214401,
            "value": 397112.5943266509
          },
          {
            "t": 10.221188,
            "value": 397303.5011377779
          },
          {
            "t": 11.228058,
            "value": 397270.7499478582
          },
          {
            "t": 12.239628,
            "value": 394436.37118538504
          },
          {
            "t": 13.246683,
            "value": 398190.7641588592
          },
          {
            "t": 14.253242,
            "value": 397393.49605934677
          },
          {
            "t": 15.260344,
            "value": 397179.2330866188
          },
          {
            "t": 16.271753,
            "value": 395487.8787908749
          },
          {
            "t": 17.278864,
            "value": 397175.683713116
          },
          {
            "t": 18.285045,
            "value": 397542.7880272039
          },
          {
            "t": 19.291845,
            "value": 397298.3710766786
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.028751,
            "value": 55212814.27590068
          },
          {
            "t": 2.057049,
            "value": 54734105.14628521
          },
          {
            "t": 4.074963,
            "value": 55025597.22564985
          },
          {
            "t": 6.088352,
            "value": 55200853.88367573
          },
          {
            "t": 8.106779,
            "value": 55003222.311235435
          },
          {
            "t": 10.120759,
            "value": 55125289.22829422
          },
          {
            "t": 12.139238,
            "value": 55001713.66657765
          },
          {
            "t": 14.152827,
            "value": 55134479.28052845
          },
          {
            "t": 16.071028,
            "value": 57877051.46645216
          },
          {
            "t": 18.084386,
            "value": 55141410.022459984
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.028751,
            "value": 68933056.13491493
          },
          {
            "t": 2.057049,
            "value": 68423877.55645373
          },
          {
            "t": 4.074963,
            "value": 68857851.22656366
          },
          {
            "t": 6.088352,
            "value": 69013019.34201488
          },
          {
            "t": 8.106779,
            "value": 68672635.671243
          },
          {
            "t": 10.120759,
            "value": 68995510.8789561
          },
          {
            "t": 12.139238,
            "value": 68755223.11601953
          },
          {
            "t": 14.152827,
            "value": 68921233.18115067
          },
          {
            "t": 16.071028,
            "value": 72349891.90392455
          },
          {
            "t": 18.084386,
            "value": 68930390.91905165
          }
        ],
        "ram_mib": [
          {
            "t": 0.028751,
            "value": 57.55078125
          },
          {
            "t": 2.057049,
            "value": 57.55078125
          },
          {
            "t": 4.074963,
            "value": 57.66015625
          },
          {
            "t": 6.088352,
            "value": 57.6171875
          },
          {
            "t": 8.106779,
            "value": 57.7578125
          },
          {
            "t": 10.120759,
            "value": 57.5546875
          },
          {
            "t": 12.139238,
            "value": 57.515625
          },
          {
            "t": 14.152827,
            "value": 57.31640625
          },
          {
            "t": 16.071028,
            "value": 57.69140625
          },
          {
            "t": 18.084386,
            "value": 57.66015625
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
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-2 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -5.570218086242676
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 41.77760737924239
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 43.03638725641819
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 53.298046875
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 53.6796875
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 384656.2394672369
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 408347.9210255121
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-2 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000604
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 69301176.4530877
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55447774.841597594
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-2 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 169.71110390141553
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.040301,
            "value": 41.43841092489137
          },
          {
            "t": 2.051771,
            "value": 43.03638725641819
          },
          {
            "t": 4.074082,
            "value": 41.367841191066994
          },
          {
            "t": 6.098117,
            "value": 41.169101053936764
          },
          {
            "t": 8.123158,
            "value": 41.86444720496895
          },
          {
            "t": 10.149423,
            "value": 42.05431853461658
          },
          {
            "t": 12.081023,
            "value": 41.36203600248293
          },
          {
            "t": 14.102247,
            "value": 41.5179919429811
          },
          {
            "t": 16.134264,
            "value": 41.859931740614336
          },
          {
            "t": 18.156242,
            "value": 42.105607940446646
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.346823,
            "value": 394854.87360213004
          },
          {
            "t": 1.364401,
            "value": 392107.53377136687
          },
          {
            "t": 2.555487,
            "value": 335827.9754778412
          },
          {
            "t": 3.565968,
            "value": 395851.08478041645
          },
          {
            "t": 4.57812,
            "value": 395197.55925987405
          },
          {
            "t": 5.589261,
            "value": 395592.7017102461
          },
          {
            "t": 6.605575,
            "value": 393579.1497509628
          },
          {
            "t": 7.716198,
            "value": 360158.21750494995
          },
          {
            "t": 8.727271,
            "value": 395619.30740905943
          },
          {
            "t": 9.741869,
            "value": 394244.8142022751
          },
          {
            "t": 10.758343,
            "value": 393517.1976853318
          },
          {
            "t": 11.774853,
            "value": 393503.2611582768
          },
          {
            "t": 12.886673,
            "value": 359770.4664424097
          },
          {
            "t": 13.898168,
            "value": 395454.25335765374
          },
          {
            "t": 14.912654,
            "value": 394288.3391195147
          },
          {
            "t": 15.929039,
            "value": 393551.6561145628
          },
          {
            "t": 17.040266,
            "value": 359962.45591584797
          },
          {
            "t": 18.051197,
            "value": 395674.87790957047
          },
          {
            "t": 19.062235,
            "value": 395633.00291383703
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.450655,
            "value": 357335.882329294
          },
          {
            "t": 1.547015,
            "value": 729687.3289795322
          },
          {
            "t": 2.555487,
            "value": 396639.66872654867
          },
          {
            "t": 3.565968,
            "value": 395851.08478041645
          },
          {
            "t": 4.57812,
            "value": 396185.55315802374
          },
          {
            "t": 5.693378,
            "value": 357764.75039856246
          },
          {
            "t": 6.707285,
            "value": 395499.7844970003
          },
          {
            "t": 7.716198,
            "value": 396466.2959046023
          },
          {
            "t": 8.727271,
            "value": 395619.30740905943
          },
          {
            "t": 9.741869,
            "value": 393259.2021667695
          },
          {
            "t": 10.861565,
            "value": 357239.8222374644
          },
          {
            "t": 11.87594,
            "value": 395317.3136167591
          },
          {
            "t": 12.886673,
            "value": 395752.38960239745
          },
          {
            "t": 13.898168,
            "value": 395454.25335765374
          },
          {
            "t": 14.912654,
            "value": 393302.61827171594
          },
          {
            "t": 16.0315,
            "value": 357511.22138346115
          },
          {
            "t": 17.040266,
            "value": 397515.3801773652
          },
          {
            "t": 18.051197,
            "value": 394685.69071479654
          },
          {
            "t": 19.062235,
            "value": 395633.00291383703
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.040301,
            "value": 57848914.22124011
          },
          {
            "t": 2.051771,
            "value": 55192523.87557358
          },
          {
            "t": 4.074082,
            "value": 54897695.26052125
          },
          {
            "t": 6.098117,
            "value": 54918751.89905313
          },
          {
            "t": 8.123158,
            "value": 54821893.97646764
          },
          {
            "t": 10.149423,
            "value": 54858078.78041618
          },
          {
            "t": 12.081023,
            "value": 57472498.44688341
          },
          {
            "t": 14.102247,
            "value": 54927428.13265625
          },
          {
            "t": 16.134264,
            "value": 54633532.10135545
          },
          {
            "t": 18.156242,
            "value": 54906431.721809044
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.040301,
            "value": 72217787.03685392
          },
          {
            "t": 2.051771,
            "value": 69066472.77861464
          },
          {
            "t": 4.074082,
            "value": 68611976.10060965
          },
          {
            "t": 6.098117,
            "value": 68639039.34467536
          },
          {
            "t": 8.123158,
            "value": 68518951.46814312
          },
          {
            "t": 10.149423,
            "value": 68563333.0289967
          },
          {
            "t": 12.081023,
            "value": 71746802.13294677
          },
          {
            "t": 14.102247,
            "value": 68736326.10734881
          },
          {
            "t": 16.134264,
            "value": 68259118.89516671
          },
          {
            "t": 18.156242,
            "value": 68651957.63752128
          }
        ],
        "ram_mib": [
          {
            "t": 0.040301,
            "value": 53.02734375
          },
          {
            "t": 2.051771,
            "value": 53.2421875
          },
          {
            "t": 4.074082,
            "value": 53.046875
          },
          {
            "t": 6.098117,
            "value": 53.43359375
          },
          {
            "t": 8.123158,
            "value": 53.5859375
          },
          {
            "t": 10.149423,
            "value": 53.10546875
          },
          {
            "t": 12.081023,
            "value": 53.50390625
          },
          {
            "t": 14.102247,
            "value": 53.28125
          },
          {
            "t": 16.134264,
            "value": 53.6796875
          },
          {
            "t": 18.156242,
            "value": 53.07421875
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
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-3 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 36.5073062681769
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 36.757760400375346
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 58.283203125
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 58.54296875
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 387784.12649559026
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 397930.38501126086
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-3 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000567
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 69331196.02432224
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55496097.2245425
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-3 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 174.22945981458608
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.115343,
            "value": 36.28235294117647
          },
          {
            "t": 2.034205,
            "value": 36.57941747572816
          },
          {
            "t": 4.050971,
            "value": 36.34694158075602
          },
          {
            "t": 6.068223,
            "value": 36.338133000312205
          },
          {
            "t": 8.090648,
            "value": 36.757760400375346
          },
          {
            "t": 10.112004,
            "value": 36.306019417475724
          },
          {
            "t": 12.142216,
            "value": 36.5395675336885
          },
          {
            "t": 14.060465,
            "value": 36.64923462986198
          },
          {
            "t": 16.077263,
            "value": 36.64120225422668
          },
          {
            "t": 18.104453,
            "value": 36.632433448167866
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.319579,
            "value": 395655.3090513076
          },
          {
            "t": 1.429318,
            "value": 360445.1136708722
          },
          {
            "t": 2.436894,
            "value": 396992.38568604254
          },
          {
            "t": 3.444887,
            "value": 396828.1525764564
          },
          {
            "t": 4.45363,
            "value": 396533.1110104358
          },
          {
            "t": 5.465928,
            "value": 395140.5613761956
          },
          {
            "t": 6.476029,
            "value": 396000.00396000006
          },
          {
            "t": 7.584892,
            "value": 360729.864735319
          },
          {
            "t": 8.594125,
            "value": 396340.5873569334
          },
          {
            "t": 9.606862,
            "value": 394969.2763274177
          },
          {
            "t": 10.619451,
            "value": 395027.00503363163
          },
          {
            "t": 11.63701,
            "value": 393097.5992546869
          },
          {
            "t": 12.749271,
            "value": 359627.8211678734
          },
          {
            "t": 13.756354,
            "value": 397186.72641678987
          },
          {
            "t": 14.764562,
            "value": 396743.5291130402
          },
          {
            "t": 15.773194,
            "value": 396576.7494983304
          },
          {
            "t": 16.785589,
            "value": 395102.7020086034
          },
          {
            "t": 17.799823,
            "value": 394386.3053299337
          },
          {
            "t": 18.909203,
            "value": 360561.7552146244
          },
          {
            "t": 19.918112,
            "value": 396467.86776607204
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.115343,
            "value": 396739.9875225274
          },
          {
            "t": 1.126228,
            "value": 395692.88296888367
          },
          {
            "t": 2.134832,
            "value": 396587.7589222331
          },
          {
            "t": 3.142977,
            "value": 396768.32201717014
          },
          {
            "t": 4.1516,
            "value": 396580.2881750664
          },
          {
            "t": 5.159761,
            "value": 396762.02511305234
          },
          {
            "t": 6.168881,
            "value": 396384.9690819724
          },
          {
            "t": 7.182477,
            "value": 394634.5486761984
          },
          {
            "t": 8.191292,
            "value": 396504.8100989775
          },
          {
            "t": 9.199671,
            "value": 396676.24970373244
          },
          {
            "t": 10.212636,
            "value": 394880.37592611794
          },
          {
            "t": 11.128989,
            "value": 436513.0031767234
          },
          {
            "t": 12.142216,
            "value": 394778.267851133
          },
          {
            "t": 13.153001,
            "value": 395732.03005584766
          },
          {
            "t": 14.161096,
            "value": 396788.0011308458
          },
          {
            "t": 15.169816,
            "value": 396542.1524308034
          },
          {
            "t": 16.177916,
            "value": 396786.03313163377
          },
          {
            "t": 17.190884,
            "value": 394879.2064507467
          },
          {
            "t": 18.205175,
            "value": 394364.1420460203
          },
          {
            "t": 19.214161,
            "value": 396437.6116219651
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.115343,
            "value": 55105618.568204924
          },
          {
            "t": 2.034205,
            "value": 57766350.05539741
          },
          {
            "t": 4.050971,
            "value": 55097733.69840626
          },
          {
            "t": 6.068223,
            "value": 54947331.320033394
          },
          {
            "t": 8.090648,
            "value": 54940437.34625511
          },
          {
            "t": 10.112004,
            "value": 54828508.684269376
          },
          {
            "t": 12.142216,
            "value": 54719423.88282603
          },
          {
            "t": 14.060465,
            "value": 57782014.22234548
          },
          {
            "t": 16.077263,
            "value": 55096023.99447044
          },
          {
            "t": 18.104453,
            "value": 54677530.473216616
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.115343,
            "value": 68796729.23847546
          },
          {
            "t": 2.034205,
            "value": 72208136.90614541
          },
          {
            "t": 4.050971,
            "value": 68691637.99865726
          },
          {
            "t": 6.068223,
            "value": 68779011.24896641
          },
          {
            "t": 8.090648,
            "value": 68638220.45316884
          },
          {
            "t": 10.112004,
            "value": 68503141.45553777
          },
          {
            "t": 12.142216,
            "value": 68370207.64333971
          },
          {
            "t": 14.060465,
            "value": 72186879.28418052
          },
          {
            "t": 16.077263,
            "value": 68828824.20549802
          },
          {
            "t": 18.104453,
            "value": 68309171.8092532
          }
        ],
        "ram_mib": [
          {
            "t": 0.115343,
            "value": 58.2890625
          },
          {
            "t": 2.034205,
            "value": 57.87109375
          },
          {
            "t": 4.050971,
            "value": 58.51171875
          },
          {
            "t": 6.068223,
            "value": 58.29296875
          },
          {
            "t": 8.090648,
            "value": 58.43359375
          },
          {
            "t": 10.112004,
            "value": 58.19140625
          },
          {
            "t": 12.142216,
            "value": 58.375
          },
          {
            "t": 14.060465,
            "value": 58.32421875
          },
          {
            "t": 16.077263,
            "value": 58.0
          },
          {
            "t": 18.104453,
            "value": 58.54296875
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
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-4 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 35.71237207732138
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 35.85597503900156
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 57.534375
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 57.93359375
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 388954.68990049977
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 397137.39187542594
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-4 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000548
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 69167960.62412703
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 55403663.23951721
        },
        {
          "extra": "DFE OTAP Transform Insert Multi Transform (Logs)/transform-4 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 174.16632641285926
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.107478,
            "value": 35.72700593193881
          },
          {
            "t": 2.036627,
            "value": 35.81497039576192
          },
          {
            "t": 4.062371,
            "value": 35.55599875350576
          },
          {
            "t": 6.083049,
            "value": 35.85597503900156
          },
          {
            "t": 8.109652,
            "value": 35.72610677489853
          },
          {
            "t": 10.131798,
            "value": 35.55376834217921
          },
          {
            "t": 12.15147,
            "value": 35.72507183010618
          },
          {
            "t": 14.080496,
            "value": 35.73876327295441
          },
          {
            "t": 16.10038,
            "value": 35.77462472749922
          },
          {
            "t": 18.123725,
            "value": 35.65143570536829
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.314464,
            "value": 394066.5401056295
          },
          {
            "t": 1.32902,
            "value": 394261.13492010295
          },
          {
            "t": 2.344281,
            "value": 393987.3589155892
          },
          {
            "t": 3.45493,
            "value": 360149.78629612055
          },
          {
            "t": 4.465845,
            "value": 395681.1403530465
          },
          {
            "t": 5.475781,
            "value": 396064.70112957654
          },
          {
            "t": 6.491419,
            "value": 393841.11267991154
          },
          {
            "t": 7.502258,
            "value": 395710.88966690045
          },
          {
            "t": 8.613748,
            "value": 359877.28184689017
          },
          {
            "t": 9.625234,
            "value": 395457.7720304581
          },
          {
            "t": 10.635663,
            "value": 395871.4565793341
          },
          {
            "t": 11.64543,
            "value": 396130.9886340116
          },
          {
            "t": 12.659859,
            "value": 394310.49388375133
          },
          {
            "t": 13.675075,
            "value": 394989.8346755765
          },
          {
            "t": 14.785991,
            "value": 359163.0690349225
          },
          {
            "t": 15.795705,
            "value": 396151.78159359976
          },
          {
            "t": 16.805549,
            "value": 396100.7838834513
          },
          {
            "t": 17.819017,
            "value": 394684.39062703506
          },
          {
            "t": 18.838033,
            "value": 393516.8829537515
          },
          {
            "t": 19.854015,
            "value": 392723.49313275237
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.107478,
            "value": 396066.2698082643
          },
          {
            "t": 1.122476,
            "value": 394089.4464816679
          },
          {
            "t": 2.137358,
            "value": 394134.49051219743
          },
          {
            "t": 3.15233,
            "value": 394099.54166223307
          },
          {
            "t": 4.163188,
            "value": 395703.4519190628
          },
          {
            "t": 5.173271,
            "value": 396007.06080589414
          },
          {
            "t": 6.183879,
            "value": 395801.33939173253
          },
          {
            "t": 7.198883,
            "value": 394087.1168980615
          },
          {
            "t": 8.210449,
            "value": 395426.49713414646
          },
          {
            "t": 9.121323,
            "value": 439138.6734059815
          },
          {
            "t": 10.131798,
            "value": 395853.4352655929
          },
          {
            "t": 11.141651,
            "value": 396097.2537587154
          },
          {
            "t": 12.15147,
            "value": 396110.5901156544
          },
          {
            "t": 13.166279,
            "value": 394162.8424659221
          },
          {
            "t": 14.181353,
            "value": 394059.940457543
          },
          {
            "t": 15.191262,
            "value": 396075.28995186696
          },
          {
            "t": 16.201163,
            "value": 396078.4274894272
          },
          {
            "t": 17.211529,
            "value": 395896.14060647326
          },
          {
            "t": 18.224529,
            "value": 394866.7324777888
          },
          {
            "t": 19.244432,
            "value": 392194.1596406717
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.107478,
            "value": 54792560.861337475
          },
          {
            "t": 2.036627,
            "value": 57668057.780917905
          },
          {
            "t": 4.062371,
            "value": 54781275.42275826
          },
          {
            "t": 6.083049,
            "value": 54917059.52160611
          },
          {
            "t": 8.109652,
            "value": 54827244.90193688
          },
          {
            "t": 10.131798,
            "value": 54806786.948123425
          },
          {
            "t": 12.15147,
            "value": 55084503.82042233
          },
          {
            "t": 14.080496,
            "value": 57384848.62308751
          },
          {
            "t": 16.10038,
            "value": 55069614.393697865
          },
          {
            "t": 18.123725,
            "value": 54704680.12128431
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.107478,
            "value": 68404551.61278027
          },
          {
            "t": 2.036627,
            "value": 71988541.57973282
          },
          {
            "t": 4.062371,
            "value": 68283123.63260116
          },
          {
            "t": 6.083049,
            "value": 68664082.55051027
          },
          {
            "t": 8.109652,
            "value": 68189016.79312623
          },
          {
            "t": 10.131798,
            "value": 68679130.48810521
          },
          {
            "t": 12.15147,
            "value": 68675898.3636947
          },
          {
            "t": 14.080496,
            "value": 71727405.9551297
          },
          {
            "t": 16.10038,
            "value": 68590757.68707511
          },
          {
            "t": 18.123725,
            "value": 68477097.57851478
          }
        ],
        "ram_mib": [
          {
            "t": 0.107478,
            "value": 57.37890625
          },
          {
            "t": 2.036627,
            "value": 57.17578125
          },
          {
            "t": 4.062371,
            "value": 57.6953125
          },
          {
            "t": 6.083049,
            "value": 57.671875
          },
          {
            "t": 8.109652,
            "value": 57.93359375
          },
          {
            "t": 10.131798,
            "value": 57.66015625
          },
          {
            "t": 12.15147,
            "value": 57.41796875
          },
          {
            "t": 14.080496,
            "value": 57.37890625
          },
          {
            "t": 16.10038,
            "value": 57.24609375
          },
          {
            "t": 18.123725,
            "value": 57.78515625
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
