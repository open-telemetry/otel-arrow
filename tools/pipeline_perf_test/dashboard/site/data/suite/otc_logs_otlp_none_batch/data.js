window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otlp_none_batch"] = {
  "name": "OTC OTLP Batch Processor (Logs)",
  "slug": "otc_logs_otlp_none_batch",
  "description": "OpenTelemetry Collector OTLP logs through a batch processor with no compression",
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
          "extra": "OTC OTLP Batch Processor (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.052631575614213943
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 18.447806336029114
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 19.64044943820225
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 65.712890625
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 69.1015625
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 99390.69320195906
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/100k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 99338.38231080014
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000613
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 36149995.22060958
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 35891984.55784427
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.088026,
            "value": 19.64044943820225
          },
          {
            "t": 3.100536,
            "value": 18.31840199750312
          },
          {
            "t": 5.11263,
            "value": 17.5890795241077
          },
          {
            "t": 7.124931,
            "value": 19.220274656679152
          },
          {
            "t": 9.13616,
            "value": 17.619699999999998
          },
          {
            "t": 11.148403,
            "value": 18.662036227357902
          },
          {
            "t": 13.161401,
            "value": 17.527528230865748
          },
          {
            "t": 15.1734,
            "value": 18.824862155388473
          },
          {
            "t": 17.185209,
            "value": 19.214846394984324
          },
          {
            "t": 19.197975,
            "value": 17.860884735202493
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.081497,
            "value": 99406.93820665908
          },
          {
            "t": 1.088026,
            "value": 100344.84848424635
          },
          {
            "t": 2.094387,
            "value": 98374.24145013568
          },
          {
            "t": 3.100536,
            "value": 99388.85791269485
          },
          {
            "t": 4.106367,
            "value": 99420.28034530653
          },
          {
            "t": 5.11263,
            "value": 99377.59810308041
          },
          {
            "t": 6.119169,
            "value": 99350.34807394446
          },
          {
            "t": 7.124931,
            "value": 99427.1010437857
          },
          {
            "t": 8.130262,
            "value": 100464.4241548306
          },
          {
            "t": 9.13616,
            "value": 98419.521661242
          },
          {
            "t": 10.142162,
            "value": 99403.38090779143
          },
          {
            "t": 11.148403,
            "value": 100373.56855862563
          },
          {
            "t": 12.155962,
            "value": 98257.27327134191
          },
          {
            "t": 13.161401,
            "value": 99459.04226909838
          },
          {
            "t": 14.16782,
            "value": 99362.19407622472
          },
          {
            "t": 15.1734,
            "value": 99445.09636229838
          },
          {
            "t": 16.179545,
            "value": 99389.25304006877
          },
          {
            "t": 17.185209,
            "value": 99436.79002131926
          },
          {
            "t": 18.191479,
            "value": 100370.67586234311
          },
          {
            "t": 19.197975,
            "value": 98361.04664101993
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.081497,
            "value": 98412.86882459249
          },
          {
            "t": 1.088026,
            "value": 98357.821781588
          },
          {
            "t": 2.094387,
            "value": 98374.24145013568
          },
          {
            "t": 3.100536,
            "value": 98394.96933356789
          },
          {
            "t": 4.106367,
            "value": 107373.90277293105
          },
          {
            "t": 5.11263,
            "value": 98383.8221220496
          },
          {
            "t": 6.119169,
            "value": 98356.84459320502
          },
          {
            "t": 7.124931,
            "value": 98432.83003334784
          },
          {
            "t": 8.130262,
            "value": 98475.02961711118
          },
          {
            "t": 9.13616,
            "value": 98419.521661242
          },
          {
            "t": 10.142162,
            "value": 98409.34709871352
          },
          {
            "t": 11.148403,
            "value": 98385.97314162314
          },
          {
            "t": 12.155962,
            "value": 98257.27327134191
          },
          {
            "t": 13.161401,
            "value": 107415.76565062624
          },
          {
            "t": 14.16782,
            "value": 98368.57213546247
          },
          {
            "t": 15.1734,
            "value": 98450.6453986754
          },
          {
            "t": 16.179545,
            "value": 98395.36050966808
          },
          {
            "t": 17.185209,
            "value": 98442.42212110606
          },
          {
            "t": 18.191479,
            "value": 98383.13772645513
          },
          {
            "t": 19.197975,
            "value": 98361.04664101993
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.088026,
            "value": 35886861.830712624
          },
          {
            "t": 3.100536,
            "value": 35887345.15604892
          },
          {
            "t": 5.11263,
            "value": 35893622.266156554
          },
          {
            "t": 7.124931,
            "value": 35890632.66380129
          },
          {
            "t": 9.13616,
            "value": 35909826.28034898
          },
          {
            "t": 11.148403,
            "value": 35891560.810498536
          },
          {
            "t": 13.161401,
            "value": 35878444.98603575
          },
          {
            "t": 15.1734,
            "value": 35897310.08812629
          },
          {
            "t": 17.185209,
            "value": 35902424.63375002
          },
          {
            "t": 19.197975,
            "value": 35881816.8629637
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.088026,
            "value": 37262196.92471286
          },
          {
            "t": 3.100536,
            "value": 35642470.844865374
          },
          {
            "t": 5.11263,
            "value": 35652518.22230969
          },
          {
            "t": 7.124931,
            "value": 37265559.67521758
          },
          {
            "t": 9.13616,
            "value": 35666692.852976955
          },
          {
            "t": 11.148403,
            "value": 35647174.819343396
          },
          {
            "t": 13.161401,
            "value": 35859720.67533102
          },
          {
            "t": 15.1734,
            "value": 37044866.821504384
          },
          {
            "t": 17.185209,
            "value": 35658474.04003064
          },
          {
            "t": 19.197975,
            "value": 35800277.32980386
          }
        ],
        "ram_mib": [
          {
            "t": 1.088026,
            "value": 68.05078125
          },
          {
            "t": 3.100536,
            "value": 63.02734375
          },
          {
            "t": 5.11263,
            "value": 64.4375
          },
          {
            "t": 7.124931,
            "value": 65.02734375
          },
          {
            "t": 9.13616,
            "value": 63.4140625
          },
          {
            "t": 11.148403,
            "value": 65.38671875
          },
          {
            "t": 13.161401,
            "value": 67.99609375
          },
          {
            "t": 15.1734,
            "value": 64.5859375
          },
          {
            "t": 17.185209,
            "value": 66.1015625
          },
          {
            "t": 19.197975,
            "value": 69.1015625
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
          "extra": "OTC OTLP Batch Processor (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.052631575614213943
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 31.791905686505086
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 33.199301745635914
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 65.82109375
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 71.55859375
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 198602.0713777935
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/200k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 197445.38527037672
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000658
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 72605938.34479286
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 72090258.08133927
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.102133,
            "value": 30.702366127023662
          },
          {
            "t": 3.115122,
            "value": 30.257711069418384
          },
          {
            "t": 5.129125,
            "value": 31.807999999999996
          },
          {
            "t": 7.142101,
            "value": 33.09182327317984
          },
          {
            "t": 9.157098,
            "value": 31.89137157107232
          },
          {
            "t": 11.170511,
            "value": 31.420149625935164
          },
          {
            "t": 13.183718,
            "value": 31.103960024984385
          },
          {
            "t": 15.096813,
            "value": 32.02502180685358
          },
          {
            "t": 17.111996,
            "value": 32.41935162094763
          },
          {
            "t": 19.125856,
            "value": 33.199301745635914
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.095742,
            "value": 198692.99746269043
          },
          {
            "t": 1.102133,
            "value": 198729.91709981507
          },
          {
            "t": 2.108533,
            "value": 198728.1399046105
          },
          {
            "t": 3.115122,
            "value": 198690.8261465206
          },
          {
            "t": 4.122376,
            "value": 198559.6483111509
          },
          {
            "t": 5.129125,
            "value": 198659.2487303191
          },
          {
            "t": 6.135626,
            "value": 198708.19800477097
          },
          {
            "t": 7.142101,
            "value": 198713.33118060557
          },
          {
            "t": 8.149882,
            "value": 198455.81530114182
          },
          {
            "t": 9.157098,
            "value": 198567.13952121494
          },
          {
            "t": 10.164156,
            "value": 198598.29324626783
          },
          {
            "t": 11.170511,
            "value": 198737.02619850845
          },
          {
            "t": 12.177493,
            "value": 198613.28206462477
          },
          {
            "t": 13.183718,
            "value": 198762.70217893613
          },
          {
            "t": 14.190175,
            "value": 198716.8850730831
          },
          {
            "t": 15.197305,
            "value": 198584.0953997994
          },
          {
            "t": 16.205239,
            "value": 198425.6905710096
          },
          {
            "t": 17.213271,
            "value": 198406.39979683183
          },
          {
            "t": 18.219773,
            "value": 198708.00058022735
          },
          {
            "t": 19.22948,
            "value": 198077.26399836785
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.095742,
            "value": 196706.06748806353
          },
          {
            "t": 1.102133,
            "value": 196742.61792881694
          },
          {
            "t": 2.108533,
            "value": 196740.8585055644
          },
          {
            "t": 3.115122,
            "value": 205645.00506164882
          },
          {
            "t": 4.122376,
            "value": 196574.05182803937
          },
          {
            "t": 5.129125,
            "value": 196672.6562430159
          },
          {
            "t": 6.135626,
            "value": 196721.11602472325
          },
          {
            "t": 7.142101,
            "value": 205668.29777192677
          },
          {
            "t": 8.149882,
            "value": 196471.2571481304
          },
          {
            "t": 9.157098,
            "value": 196581.4681260028
          },
          {
            "t": 10.164156,
            "value": 196612.31031380515
          },
          {
            "t": 11.170511,
            "value": 196749.65593652337
          },
          {
            "t": 12.177493,
            "value": 205564.74693688666
          },
          {
            "t": 13.183718,
            "value": 196775.07515714678
          },
          {
            "t": 14.190175,
            "value": 196729.7162223523
          },
          {
            "t": 15.197305,
            "value": 196598.25444580143
          },
          {
            "t": 16.205239,
            "value": 205370.5897409949
          },
          {
            "t": 17.213271,
            "value": 196422.33579886353
          },
          {
            "t": 18.320822,
            "value": 178772.80594753652
          },
          {
            "t": 19.331441,
            "value": 195919.53050556144
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.102133,
            "value": 71756409.06541531
          },
          {
            "t": 3.115122,
            "value": 71577149.70126513
          },
          {
            "t": 5.129125,
            "value": 71721177.1779883
          },
          {
            "t": 7.142101,
            "value": 71758822.75794645
          },
          {
            "t": 9.157098,
            "value": 71686809.95554832
          },
          {
            "t": 11.170511,
            "value": 71744696.19496845
          },
          {
            "t": 13.183718,
            "value": 71751113.52185841
          },
          {
            "t": 15.096813,
            "value": 75504078.99241805
          },
          {
            "t": 17.111996,
            "value": 71679444.49710026
          },
          {
            "t": 19.125856,
            "value": 71722878.94888422
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.102133,
            "value": 71281830.47031525
          },
          {
            "t": 3.115122,
            "value": 72896635.79880466
          },
          {
            "t": 5.129125,
            "value": 72865414.30176617
          },
          {
            "t": 7.142101,
            "value": 71281179.20929012
          },
          {
            "t": 9.157098,
            "value": 72822142.16696104
          },
          {
            "t": 11.170511,
            "value": 72887891.85328594
          },
          {
            "t": 13.183718,
            "value": 71277397.20754
          },
          {
            "t": 15.096813,
            "value": 76698463.48456298
          },
          {
            "t": 17.111996,
            "value": 72816223.63825023
          },
          {
            "t": 19.125856,
            "value": 71232205.31715213
          }
        ],
        "ram_mib": [
          {
            "t": 1.102133,
            "value": 66.9453125
          },
          {
            "t": 3.115122,
            "value": 62.67578125
          },
          {
            "t": 5.129125,
            "value": 67.51953125
          },
          {
            "t": 7.142101,
            "value": 64.56640625
          },
          {
            "t": 9.157098,
            "value": 64.890625
          },
          {
            "t": 11.170511,
            "value": 61.3203125
          },
          {
            "t": 13.183718,
            "value": 71.55859375
          },
          {
            "t": 15.096813,
            "value": 61.0625
          },
          {
            "t": 17.111996,
            "value": 71.515625
          },
          {
            "t": 19.125856,
            "value": 66.15625
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
          "extra": "OTC OTLP Batch Processor (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 2.4837472438812256
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 43.67975426347955
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 44.461621621621624
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 71.9125
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 77.4765625
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 312828.6388960982
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/300k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 303457.91067309194
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000612
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 108264871.49097458
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 107341045.79485634
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.032621,
            "value": 44.01652120467117
          },
          {
            "t": 3.048467,
            "value": 43.560049079754606
          },
          {
            "t": 5.064317,
            "value": 43.52147510755992
          },
          {
            "t": 7.080405,
            "value": 43.2660333127699
          },
          {
            "t": 9.097011,
            "value": 43.71109471094711
          },
          {
            "t": 11.113161,
            "value": 44.368
          },
          {
            "t": 13.129059,
            "value": 43.86049261083744
          },
          {
            "t": 15.146832,
            "value": 42.605333333333334
          },
          {
            "t": 17.168846,
            "value": 44.461621621621624
          },
          {
            "t": 19.196849,
            "value": 43.42692165330043
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.12551,
            "value": 298787.18212915154
          },
          {
            "t": 1.133301,
            "value": 296688.49989730015
          },
          {
            "t": 2.14156,
            "value": 297542.59570209635
          },
          {
            "t": 3.149174,
            "value": 297733.0604775241
          },
          {
            "t": 4.156903,
            "value": 298691.41406072467
          },
          {
            "t": 5.164986,
            "value": 296602.5614954324
          },
          {
            "t": 6.172612,
            "value": 297729.5147207397
          },
          {
            "t": 7.181097,
            "value": 297475.9168455653
          },
          {
            "t": 8.189917,
            "value": 297377.13368093414
          },
          {
            "t": 9.197864,
            "value": 396846.26274992636
          },
          {
            "t": 10.206166,
            "value": 297529.9067144566
          },
          {
            "t": 11.213858,
            "value": 297710.0145679434
          },
          {
            "t": 12.221522,
            "value": 396957.7160640849
          },
          {
            "t": 13.229753,
            "value": 297550.8588805541
          },
          {
            "t": 14.239848,
            "value": 297991.7730510497
          },
          {
            "t": 15.249151,
            "value": 296244.0416802486
          },
          {
            "t": 16.261881,
            "value": 297216.4347851846
          },
          {
            "t": 17.27631,
            "value": 294747.09417810413
          },
          {
            "t": 18.288391,
            "value": 296418.96251387
          },
          {
            "t": 19.302143,
            "value": 394573.8208161365
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.12551,
            "value": 294816.5883467043
          },
          {
            "t": 1.133301,
            "value": 303634.3845102804
          },
          {
            "t": 2.14156,
            "value": 294567.1697450754
          },
          {
            "t": 3.149174,
            "value": 294755.7298727489
          },
          {
            "t": 4.156903,
            "value": 303653.0654570822
          },
          {
            "t": 5.164986,
            "value": 294618.5978733894
          },
          {
            "t": 6.172612,
            "value": 294752.2195735323
          },
          {
            "t": 7.181097,
            "value": 303425.43518247665
          },
          {
            "t": 8.189917,
            "value": 294403.3623441248
          },
          {
            "t": 9.197864,
            "value": 294658.35009182035
          },
          {
            "t": 10.206166,
            "value": 294554.607647312
          },
          {
            "t": 11.213858,
            "value": 303664.2148593022
          },
          {
            "t": 12.221522,
            "value": 294741.104177583
          },
          {
            "t": 13.229753,
            "value": 446326.2883208312
          },
          {
            "t": 14.239848,
            "value": 294031.7494889095
          },
          {
            "t": 15.350508,
            "value": 275511.8578142726
          },
          {
            "t": 16.363282,
            "value": 293253.9737394522
          },
          {
            "t": 17.377618,
            "value": 292802.38500851794
          },
          {
            "t": 18.389613,
            "value": 302373.03543989843
          },
          {
            "t": 19.403307,
            "value": 292987.8247281724
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.032621,
            "value": 107335105.63595212
          },
          {
            "t": 3.048467,
            "value": 107472720.63441356
          },
          {
            "t": 5.064317,
            "value": 107469897.0657539
          },
          {
            "t": 7.080405,
            "value": 107458520.65981248
          },
          {
            "t": 9.097011,
            "value": 107426472.49884212
          },
          {
            "t": 11.113161,
            "value": 107450941.15021203
          },
          {
            "t": 13.129059,
            "value": 107464871.23852497
          },
          {
            "t": 15.146832,
            "value": 107366878.73214677
          },
          {
            "t": 17.168846,
            "value": 107139751.25790425
          },
          {
            "t": 19.196849,
            "value": 106825299.07500137
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.032621,
            "value": 108430622.39185423
          },
          {
            "t": 3.048467,
            "value": 108377717.84154148
          },
          {
            "t": 5.064317,
            "value": 108374558.12684476
          },
          {
            "t": 7.080405,
            "value": 108361380.05880697
          },
          {
            "t": 9.097011,
            "value": 108329609.75024374
          },
          {
            "t": 11.113161,
            "value": 108361147.23606874
          },
          {
            "t": 13.129059,
            "value": 108365055.67245962
          },
          {
            "t": 15.146832,
            "value": 108271569.20030151
          },
          {
            "t": 17.168846,
            "value": 108046272.18209171
          },
          {
            "t": 19.196849,
            "value": 107730782.44953287
          }
        ],
        "ram_mib": [
          {
            "t": 1.032621,
            "value": 77.4765625
          },
          {
            "t": 3.048467,
            "value": 71.1796875
          },
          {
            "t": 5.064317,
            "value": 69.640625
          },
          {
            "t": 7.080405,
            "value": 70.9765625
          },
          {
            "t": 9.097011,
            "value": 71.34765625
          },
          {
            "t": 11.113161,
            "value": 71.1484375
          },
          {
            "t": 13.129059,
            "value": 74.734375
          },
          {
            "t": 15.146832,
            "value": 72.55078125
          },
          {
            "t": 17.168846,
            "value": 70.1328125
          },
          {
            "t": 19.196849,
            "value": 69.9375
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
          "extra": "OTC OTLP Batch Processor (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 2.4624998569488525
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 56.005784100256705
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 57.49750782717595
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 79.597265625
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 81.28125
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 412493.6914246068
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/400k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 406582.9441711544
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000688
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 144844628.94584197
        },
        {
          "extra": "OTC OTLP Batch Processor (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 143633859.0352406
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.061741,
            "value": 55.91080174021131
          },
          {
            "t": 3.080285,
            "value": 56.94171143035602
          },
          {
            "t": 5.10076,
            "value": 56.491485642946316
          },
          {
            "t": 7.118999,
            "value": 57.49750782717595
          },
          {
            "t": 9.137399,
            "value": 56.4652
          },
          {
            "t": 11.155704,
            "value": 56.090300000000006
          },
          {
            "t": 13.083555,
            "value": 54.21474734872115
          },
          {
            "t": 15.104921,
            "value": 55.891503431066745
          },
          {
            "t": 17.123744,
            "value": 55.08328358208955
          },
          {
            "t": 19.143747,
            "value": 55.4713
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.052254,
            "value": 295589.8001812951
          },
          {
            "t": 0.153609,
            "value": 89583.65994042688
          },
          {
            "t": 1.162547,
            "value": 369313.0654982312
          },
          {
            "t": 2.171954,
            "value": 396272.2667863409
          },
          {
            "t": 3.181326,
            "value": 396286.0075373599
          },
          {
            "t": 4.19212,
            "value": 494660.63312603754
          },
          {
            "t": 5.201617,
            "value": 594355.4067025458
          },
          {
            "t": 6.210621,
            "value": 396430.539423035
          },
          {
            "t": 7.219839,
            "value": 396346.4781642817
          },
          {
            "t": 8.229022,
            "value": 396360.2240624347
          },
          {
            "t": 9.23818,
            "value": 396370.0431448792
          },
          {
            "t": 10.247505,
            "value": 396304.46090208803
          },
          {
            "t": 11.26171,
            "value": 394397.58234282024
          },
          {
            "t": 12.27593,
            "value": 394391.74932460417
          },
          {
            "t": 13.386268,
            "value": 360250.66241090547
          },
          {
            "t": 14.397615,
            "value": 395512.1239297689
          },
          {
            "t": 15.407739,
            "value": 395990.9872451303
          },
          {
            "t": 16.417056,
            "value": 396307.6020714998
          },
          {
            "t": 17.426611,
            "value": 396214.1735715241
          },
          {
            "t": 18.437106,
            "value": 494807.00052944355
          },
          {
            "t": 19.44649,
            "value": 396281.2963153765
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.153609,
            "value": 390152.7802819149
          },
          {
            "t": 1.162547,
            "value": 392491.90733226424
          },
          {
            "t": 2.171954,
            "value": 401225.6701211702
          },
          {
            "t": 3.181326,
            "value": 392323.1474619863
          },
          {
            "t": 4.19212,
            "value": 400675.1128320904
          },
          {
            "t": 5.201617,
            "value": 392274.56842368026
          },
          {
            "t": 6.210621,
            "value": 401385.92116582295
          },
          {
            "t": 7.219839,
            "value": 392383.01338263886
          },
          {
            "t": 8.229022,
            "value": 401314.7268632151
          },
          {
            "t": 9.23818,
            "value": 588609.5140701456
          },
          {
            "t": 10.247505,
            "value": 401258.26666336413
          },
          {
            "t": 11.26171,
            "value": 390453.60651939205
          },
          {
            "t": 12.27593,
            "value": 399321.6461911617
          },
          {
            "t": 13.28501,
            "value": 392436.674991081
          },
          {
            "t": 14.294267,
            "value": 392367.8508050972
          },
          {
            "t": 15.306475,
            "value": 400115.3913029733
          },
          {
            "t": 16.315822,
            "value": 392332.8647135227
          },
          {
            "t": 17.32536,
            "value": 401173.60614459286
          },
          {
            "t": 18.335719,
            "value": 391939.8946315122
          },
          {
            "t": 19.345265,
            "value": 401170.4271028759
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.061741,
            "value": 142700470.80453682
          },
          {
            "t": 3.080285,
            "value": 143111413.9696732
          },
          {
            "t": 5.10076,
            "value": 142981602.84091613
          },
          {
            "t": 7.118999,
            "value": 143113881.95352483
          },
          {
            "t": 9.137399,
            "value": 142967332.54062623
          },
          {
            "t": 11.155704,
            "value": 143290708.29235426
          },
          {
            "t": 13.083555,
            "value": 149498640.71445355
          },
          {
            "t": 15.104921,
            "value": 143015874.90835404
          },
          {
            "t": 17.123744,
            "value": 142977897.02217582
          },
          {
            "t": 19.143747,
            "value": 142680767.30579114
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.061741,
            "value": 144957025.93401426
          },
          {
            "t": 3.080285,
            "value": 143762148.85580894
          },
          {
            "t": 5.10076,
            "value": 143638893.32953885
          },
          {
            "t": 7.118999,
            "value": 145403257.49328995
          },
          {
            "t": 9.137399,
            "value": 143768046.96789536
          },
          {
            "t": 11.155704,
            "value": 143786546.6319511
          },
          {
            "t": 13.083555,
            "value": 150528768.0427585
          },
          {
            "t": 15.104921,
            "value": 145191392.85018152
          },
          {
            "t": 17.123744,
            "value": 143755189.53370357
          },
          {
            "t": 19.143747,
            "value": 143655019.8192775
          }
        ],
        "ram_mib": [
          {
            "t": 1.061741,
            "value": 81.28125
          },
          {
            "t": 3.080285,
            "value": 78.08203125
          },
          {
            "t": 5.10076,
            "value": 79.3203125
          },
          {
            "t": 7.118999,
            "value": 80.55078125
          },
          {
            "t": 9.137399,
            "value": 79.71484375
          },
          {
            "t": 11.155704,
            "value": 78.75
          },
          {
            "t": 13.083555,
            "value": 79.5859375
          },
          {
            "t": 15.104921,
            "value": 80.9296875
          },
          {
            "t": 17.123744,
            "value": 78.79296875
          },
          {
            "t": 19.143747,
            "value": 78.96484375
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
