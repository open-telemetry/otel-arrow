window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otlp_zstd_baseline"] = {
  "name": "DFE OTLP Baseline w/ Zstd (Logs)",
  "slug": "dfe_logs_otlp_zstd_baseline",
  "description": "Dataflow Engine baseline for OTLP logs with zstd compression",
  "meta": {
    "binary": "dfe",
    "protocols": [
      "otlp"
    ],
    "signals": [
      "logs"
    ],
    "compression": "zstd"
  },
  "tests": [
    {
      "name": "1000k",
      "metrics": [
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/1000k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 1.0416667461395264
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/1000k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 47.287204038513806
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/1000k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 48.67474006116208
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/1000k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 12.88125
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/1000k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 13.65234375
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/1000k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 976892.423792215
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/1000k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 992151.0927056059
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/1000k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000685
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/1000k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 17231128.93026502
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/1000k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 17203175.11093341
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.08476,
            "value": 48.160197287299624
          },
          {
            "t": 2.115989,
            "value": 47.59577395577396
          },
          {
            "t": 4.144713,
            "value": 46.4273723076923
          },
          {
            "t": 6.066854,
            "value": 46.6243137254902
          },
          {
            "t": 8.095386,
            "value": 46.369766871165645
          },
          {
            "t": 10.119619,
            "value": 47.08879852125693
          },
          {
            "t": 12.147256,
            "value": 48.67474006116208
          },
          {
            "t": 14.170652,
            "value": 46.860269938650305
          },
          {
            "t": 16.195865,
            "value": 47.50930061349693
          },
          {
            "t": 18.122648,
            "value": 47.56150710315009
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.287336,
            "value": 988493.9306472659
          },
          {
            "t": 1.307239,
            "value": 980485.3991016792
          },
          {
            "t": 2.323324,
            "value": 688918.7420343771
          },
          {
            "t": 2.424627,
            "value": 268483.2842307238
          },
          {
            "t": 3.435661,
            "value": 926031.6131665637
          },
          {
            "t": 4.448222,
            "value": 987594.8214477942
          },
          {
            "t": 5.45994,
            "value": 988417.7211436387
          },
          {
            "t": 6.475982,
            "value": 984211.2826044592
          },
          {
            "t": 7.589292,
            "value": 898222.4178351043
          },
          {
            "t": 8.600259,
            "value": 989151.9703412674
          },
          {
            "t": 9.613342,
            "value": 987085.9544578285
          },
          {
            "t": 10.624701,
            "value": 988768.5777256149
          },
          {
            "t": 11.640944,
            "value": 984016.6180726457
          },
          {
            "t": 12.753372,
            "value": 898934.5827325456
          },
          {
            "t": 13.765144,
            "value": 988364.9676013966
          },
          {
            "t": 14.776657,
            "value": 988618.0404997267
          },
          {
            "t": 15.790035,
            "value": 986798.608219243
          },
          {
            "t": 16.806821,
            "value": 983491.1180917126
          },
          {
            "t": 17.919003,
            "value": 989046.7567358578
          },
          {
            "t": 18.930321,
            "value": 988808.6635459866
          },
          {
            "t": 19.941496,
            "value": 1087843.3505575198
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.08476,
            "value": 988485.1366432428
          },
          {
            "t": 1.099815,
            "value": 985168.2913733738
          },
          {
            "t": 2.115989,
            "value": 984083.4345299132
          },
          {
            "t": 3.132458,
            "value": 983797.8334804111
          },
          {
            "t": 4.144713,
            "value": 987893.3667899888
          },
          {
            "t": 5.156226,
            "value": 988618.0404997265
          },
          {
            "t": 6.167668,
            "value": 988687.4383306211
          },
          {
            "t": 7.185203,
            "value": 982767.1775418044
          },
          {
            "t": 8.196268,
            "value": 989056.0943163891
          },
          {
            "t": 9.209441,
            "value": 986998.271766026
          },
          {
            "t": 10.220562,
            "value": 989001.3163607521
          },
          {
            "t": 11.231983,
            "value": 988707.9663166971
          },
          {
            "t": 12.147256,
            "value": 1092570.194903597
          },
          {
            "t": 13.159203,
            "value": 988194.0457355969
          },
          {
            "t": 14.170652,
            "value": 987691.9152621635
          },
          {
            "t": 15.183358,
            "value": 986465.9634681733
          },
          {
            "t": 16.195865,
            "value": 989622.787793072
          },
          {
            "t": 17.2123,
            "value": 983830.7417591878
          },
          {
            "t": 18.223511,
            "value": 988913.2930713767
          },
          {
            "t": 19.235069,
            "value": 988574.0610029282
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.08476,
            "value": 17062140.063566834
          },
          {
            "t": 2.115989,
            "value": 16973063.10612934
          },
          {
            "t": 4.144713,
            "value": 16993167.626547523
          },
          {
            "t": 6.066854,
            "value": 17961767.63307166
          },
          {
            "t": 8.095386,
            "value": 17012670.24626676
          },
          {
            "t": 10.119619,
            "value": 17040956.74756809
          },
          {
            "t": 12.147256,
            "value": 17028483.895292897
          },
          {
            "t": 14.170652,
            "value": 17038459.599603835
          },
          {
            "t": 16.195865,
            "value": 17040003.693438668
          },
          {
            "t": 18.122648,
            "value": 17881038.49784849
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.08476,
            "value": 17087973.173195146
          },
          {
            "t": 2.115989,
            "value": 17016695.31106537
          },
          {
            "t": 4.144713,
            "value": 17009938.266614877
          },
          {
            "t": 6.066854,
            "value": 17989027.339825746
          },
          {
            "t": 8.095386,
            "value": 17046470.05815043
          },
          {
            "t": 10.119619,
            "value": 17067571.272674635
          },
          {
            "t": 12.147256,
            "value": 17037012.542185806
          },
          {
            "t": 14.170652,
            "value": 17073089.499040227
          },
          {
            "t": 16.195865,
            "value": 17049650.579963688
          },
          {
            "t": 18.122648,
            "value": 17933861.259934306
          }
        ],
        "ram_mib": [
          {
            "t": 0.08476,
            "value": 11.87890625
          },
          {
            "t": 2.115989,
            "value": 12.828125
          },
          {
            "t": 4.144713,
            "value": 13.65234375
          },
          {
            "t": 6.066854,
            "value": 13.18359375
          },
          {
            "t": 8.095386,
            "value": 12.921875
          },
          {
            "t": 10.119619,
            "value": 12.5390625
          },
          {
            "t": 12.147256,
            "value": 13.16015625
          },
          {
            "t": 14.170652,
            "value": 12.59765625
          },
          {
            "t": 16.195865,
            "value": 12.89453125
          },
          {
            "t": 18.122648,
            "value": 13.15625
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
      "name": "100k",
      "metrics": [
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 0.0
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 8.96093733286509
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 9.91142679127726
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 11.0203125
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 11.23046875
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 99270.93336621465
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/100k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 99270.93336621465
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000644
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 1747048.21479172
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 1710205.5407630808
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.052699,
            "value": 8.81796631316282
          },
          {
            "t": 3.066263,
            "value": 8.41488487865588
          },
          {
            "t": 5.080106,
            "value": 8.60961394769614
          },
          {
            "t": 7.092992,
            "value": 8.84562889165629
          },
          {
            "t": 9.108296,
            "value": 8.549056839475327
          },
          {
            "t": 11.121185,
            "value": 8.729707529558182
          },
          {
            "t": 13.139007,
            "value": 9.91142679127726
          },
          {
            "t": 15.151716,
            "value": 9.15720698254364
          },
          {
            "t": 17.166246,
            "value": 9.493432835820895
          },
          {
            "t": 19.186203,
            "value": 9.080448318804484
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.046663,
            "value": 99186.66931164451
          },
          {
            "t": 1.052699,
            "value": 99400.02147040464
          },
          {
            "t": 2.058663,
            "value": 99407.13584183926
          },
          {
            "t": 3.066263,
            "value": 99245.73243350536
          },
          {
            "t": 4.072959,
            "value": 99334.85381882911
          },
          {
            "t": 5.080106,
            "value": 99290.37171336458
          },
          {
            "t": 6.08705,
            "value": 99310.38866113705
          },
          {
            "t": 7.092992,
            "value": 99409.30988068896
          },
          {
            "t": 8.100366,
            "value": 99267.9977843383
          },
          {
            "t": 9.108296,
            "value": 99213.23901461411
          },
          {
            "t": 10.114818,
            "value": 99352.02608586798
          },
          {
            "t": 11.121185,
            "value": 99367.32822121552
          },
          {
            "t": 12.131976,
            "value": 98932.42025304935
          },
          {
            "t": 13.139007,
            "value": 99301.8089810542
          },
          {
            "t": 14.144606,
            "value": 99443.21742563389
          },
          {
            "t": 15.151716,
            "value": 99294.01952120425
          },
          {
            "t": 16.159057,
            "value": 99271.24975554454
          },
          {
            "t": 17.166246,
            "value": 99286.23128330434
          },
          {
            "t": 18.172895,
            "value": 99339.49171955668
          },
          {
            "t": 19.186203,
            "value": 98686.67769325811
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.046663,
            "value": 99186.66931164451
          },
          {
            "t": 1.052699,
            "value": 99400.02147040464
          },
          {
            "t": 2.058663,
            "value": 99407.13584183926
          },
          {
            "t": 3.066263,
            "value": 99245.73243350536
          },
          {
            "t": 4.072959,
            "value": 99334.85381882911
          },
          {
            "t": 5.080106,
            "value": 99290.37171336458
          },
          {
            "t": 6.08705,
            "value": 99310.38866113705
          },
          {
            "t": 7.092992,
            "value": 99409.30988068896
          },
          {
            "t": 8.100366,
            "value": 99267.9977843383
          },
          {
            "t": 9.108296,
            "value": 99213.23901461411
          },
          {
            "t": 10.114818,
            "value": 99352.02608586798
          },
          {
            "t": 11.121185,
            "value": 99367.32822121552
          },
          {
            "t": 12.131976,
            "value": 98932.42025304935
          },
          {
            "t": 13.139007,
            "value": 99301.8089810542
          },
          {
            "t": 14.144606,
            "value": 99443.21742563389
          },
          {
            "t": 15.151716,
            "value": 99294.01952120425
          },
          {
            "t": 16.159057,
            "value": 99271.24975554454
          },
          {
            "t": 17.166246,
            "value": 99286.23128330434
          },
          {
            "t": 18.172895,
            "value": 99339.49171955668
          },
          {
            "t": 19.186203,
            "value": 98686.67769325811
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.052699,
            "value": 1711533.3059284016
          },
          {
            "t": 3.066263,
            "value": 1712054.3474158258
          },
          {
            "t": 5.080106,
            "value": 1711697.4858516776
          },
          {
            "t": 7.092992,
            "value": 1704191.3948430265
          },
          {
            "t": 9.108296,
            "value": 1710565.2546712556
          },
          {
            "t": 11.121185,
            "value": 1712771.046987688
          },
          {
            "t": 13.139007,
            "value": 1708503.5250879414
          },
          {
            "t": 15.151716,
            "value": 1712723.9953714123
          },
          {
            "t": 17.166246,
            "value": 1711337.6321027733
          },
          {
            "t": 19.186203,
            "value": 1706677.4193708086
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.052699,
            "value": 1748392.9390597725
          },
          {
            "t": 3.066263,
            "value": 1748902.4436273193
          },
          {
            "t": 5.080106,
            "value": 1740199.2111599564
          },
          {
            "t": 7.092992,
            "value": 1749462.214949083
          },
          {
            "t": 9.108296,
            "value": 1747394.4377622434
          },
          {
            "t": 11.121185,
            "value": 1749604.6726868695
          },
          {
            "t": 13.139007,
            "value": 1745246.6074807392
          },
          {
            "t": 15.151716,
            "value": 1749569.8583352088
          },
          {
            "t": 17.166246,
            "value": 1748217.6984209714
          },
          {
            "t": 19.186203,
            "value": 1743492.0644350352
          }
        ],
        "ram_mib": [
          {
            "t": 1.052699,
            "value": 11.08984375
          },
          {
            "t": 3.066263,
            "value": 10.94921875
          },
          {
            "t": 5.080106,
            "value": 11.2109375
          },
          {
            "t": 7.092992,
            "value": 11.06640625
          },
          {
            "t": 9.108296,
            "value": 10.86328125
          },
          {
            "t": 11.121185,
            "value": 10.87890625
          },
          {
            "t": 13.139007,
            "value": 11.23046875
          },
          {
            "t": 15.151716,
            "value": 10.8828125
          },
          {
            "t": 17.166246,
            "value": 10.890625
          },
          {
            "t": 19.186203,
            "value": 11.140625
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
      "name": "200k",
      "metrics": [
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.6315789222717285
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 13.060180755471334
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 13.421546134663343
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 11.15390625
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 11.66796875
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 198508.37671394352
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/200k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 203732.28136431045
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000627
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 3478722.063911325
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 3441941.160907857
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.12628,
            "value": 13.180796019900498
          },
          {
            "t": 3.042342,
            "value": 13.202442367601247
          },
          {
            "t": 5.056433,
            "value": 13.010976975731175
          },
          {
            "t": 7.070661,
            "value": 12.233279402613565
          },
          {
            "t": 9.085461,
            "value": 13.385667915106117
          },
          {
            "t": 11.100074,
            "value": 12.96708229426434
          },
          {
            "t": 13.11624,
            "value": 12.938440424204616
          },
          {
            "t": 15.131973,
            "value": 13.065071651090342
          },
          {
            "t": 17.146554,
            "value": 13.421546134663343
          },
          {
            "t": 19.160709,
            "value": 13.19650436953808
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.118728,
            "value": 198520.23020405893
          },
          {
            "t": 1.12628,
            "value": 198500.92104427365
          },
          {
            "t": 2.134602,
            "value": 198349.33681899236
          },
          {
            "t": 3.142899,
            "value": 199346.02602209468
          },
          {
            "t": 4.149997,
            "value": 197597.4532766424
          },
          {
            "t": 5.15697,
            "value": 198615.05720610186
          },
          {
            "t": 6.164144,
            "value": 198575.4199373693
          },
          {
            "t": 7.171281,
            "value": 198582.71516188962
          },
          {
            "t": 8.178828,
            "value": 198501.90611455348
          },
          {
            "t": 9.186129,
            "value": 198550.3836489788
          },
          {
            "t": 10.193675,
            "value": 198502.10312978266
          },
          {
            "t": 11.200687,
            "value": 198607.36515552943
          },
          {
            "t": 12.208365,
            "value": 198476.10050035824
          },
          {
            "t": 13.216903,
            "value": 198306.85606293468
          },
          {
            "t": 14.224257,
            "value": 198539.93730108778
          },
          {
            "t": 15.232623,
            "value": 198340.68185559608
          },
          {
            "t": 16.239719,
            "value": 198590.79968543217
          },
          {
            "t": 17.24727,
            "value": 198501.11805754743
          },
          {
            "t": 18.254436,
            "value": 198576.99723779396
          },
          {
            "t": 19.261497,
            "value": 198597.70162879906
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.118728,
            "value": 198520.23020405893
          },
          {
            "t": 1.12628,
            "value": 198500.92104427365
          },
          {
            "t": 2.134602,
            "value": 198349.33681899236
          },
          {
            "t": 3.142899,
            "value": 198354.25474835292
          },
          {
            "t": 4.149997,
            "value": 198590.4053031582
          },
          {
            "t": 5.15697,
            "value": 198615.05720610186
          },
          {
            "t": 6.164144,
            "value": 198575.4199373693
          },
          {
            "t": 7.171281,
            "value": 198582.71516188962
          },
          {
            "t": 8.178828,
            "value": 198501.90611455348
          },
          {
            "t": 9.186129,
            "value": 198550.3836489788
          },
          {
            "t": 10.193675,
            "value": 198502.10312978266
          },
          {
            "t": 11.200687,
            "value": 198607.36515552943
          },
          {
            "t": 12.208365,
            "value": 198476.10050035824
          },
          {
            "t": 13.216903,
            "value": 198306.85606293468
          },
          {
            "t": 14.224257,
            "value": 198539.93730108778
          },
          {
            "t": 15.232623,
            "value": 297511.0227833941
          },
          {
            "t": 16.239719,
            "value": 198590.79968543217
          },
          {
            "t": 17.24727,
            "value": 198501.11805754743
          },
          {
            "t": 18.254436,
            "value": 198576.99723779396
          },
          {
            "t": 19.261497,
            "value": 198597.70162879906
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.12628,
            "value": 3425294.5152520635
          },
          {
            "t": 3.042342,
            "value": 3602253.476140125
          },
          {
            "t": 5.056433,
            "value": 3429692.1042793
          },
          {
            "t": 7.070661,
            "value": 3418239.6431784285
          },
          {
            "t": 9.085461,
            "value": 3417256.7996823504
          },
          {
            "t": 11.100074,
            "value": 3426017.80093745
          },
          {
            "t": 13.11624,
            "value": 3423444.29972532
          },
          {
            "t": 15.131973,
            "value": 3424403.926512093
          },
          {
            "t": 17.146554,
            "value": 3425824.5262910747
          },
          {
            "t": 19.160709,
            "value": 3426984.5170803634
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.12628,
            "value": 3462157.432781838
          },
          {
            "t": 3.042342,
            "value": 3641002.744170074
          },
          {
            "t": 5.056433,
            "value": 3455576.2376178633
          },
          {
            "t": 7.070661,
            "value": 3463384.482789436
          },
          {
            "t": 9.085461,
            "value": 3454167.162993845
          },
          {
            "t": 11.100074,
            "value": 3462814.4462484852
          },
          {
            "t": 13.11624,
            "value": 3460290.9681048086
          },
          {
            "t": 15.131973,
            "value": 3461243.6270081406
          },
          {
            "t": 17.146554,
            "value": 3462758.7572800494
          },
          {
            "t": 19.160709,
            "value": 3463824.7801187094
          }
        ],
        "ram_mib": [
          {
            "t": 1.12628,
            "value": 11.66796875
          },
          {
            "t": 3.042342,
            "value": 11.05078125
          },
          {
            "t": 5.056433,
            "value": 11.02734375
          },
          {
            "t": 7.070661,
            "value": 11.0546875
          },
          {
            "t": 9.085461,
            "value": 11.09765625
          },
          {
            "t": 11.100074,
            "value": 11.453125
          },
          {
            "t": 13.11624,
            "value": 10.8671875
          },
          {
            "t": 15.131973,
            "value": 11.23828125
          },
          {
            "t": 17.146554,
            "value": 11.0390625
          },
          {
            "t": 19.160709,
            "value": 11.04296875
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
      "name": "300k",
      "metrics": [
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -0.017543859779834747
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 17.116916022353422
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 17.954600000000003
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 11.660546875
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 12.16796875
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 296764.9033121723
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/300k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 298390.70528393355
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000691
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 5211524.166270314
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 5173331.9715732755
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.106564,
            "value": 17.325728518057286
          },
          {
            "t": 3.123996,
            "value": 16.76519577377253
          },
          {
            "t": 5.142145,
            "value": 17.8836
          },
          {
            "t": 7.09318,
            "value": 16.844114002478314
          },
          {
            "t": 9.110991,
            "value": 15.919900124843945
          },
          {
            "t": 11.129514,
            "value": 16.31192523364486
          },
          {
            "t": 13.148996,
            "value": 17.37255292652553
          },
          {
            "t": 15.167161,
            "value": 17.954600000000003
          },
          {
            "t": 17.186019,
            "value": 17.357907845579078
          },
          {
            "t": 19.102703,
            "value": 17.43363579863269
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.097483,
            "value": 297433.7416768124
          },
          {
            "t": 1.106564,
            "value": 297300.21673185803
          },
          {
            "t": 2.115087,
            "value": 297464.7082912338
          },
          {
            "t": 3.123996,
            "value": 297350.90082455403
          },
          {
            "t": 4.133229,
            "value": 297255.44051770004
          },
          {
            "t": 5.142145,
            "value": 298340.0005550512
          },
          {
            "t": 6.151027,
            "value": 296367.6624223645
          },
          {
            "t": 7.194515,
            "value": 288455.6410806833
          },
          {
            "t": 8.203564,
            "value": 296318.61287212017
          },
          {
            "t": 9.212418,
            "value": 297367.1115939472
          },
          {
            "t": 10.222006,
            "value": 298141.4200644223
          },
          {
            "t": 11.2308,
            "value": 296393.51542534947
          },
          {
            "t": 12.240552,
            "value": 297102.65490932425
          },
          {
            "t": 13.250676,
            "value": 296993.2404338477
          },
          {
            "t": 14.259744,
            "value": 297304.0469026864
          },
          {
            "t": 15.268483,
            "value": 297401.0125513141
          },
          {
            "t": 16.276994,
            "value": 297468.24774345546
          },
          {
            "t": 17.287376,
            "value": 296917.4035166898
          },
          {
            "t": 18.296263,
            "value": 297357.38492021407
          },
          {
            "t": 19.304606,
            "value": 297517.80892017897
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.097483,
            "value": 296442.2958712231
          },
          {
            "t": 1.106564,
            "value": 298291.2174542975
          },
          {
            "t": 2.115087,
            "value": 297464.7082912338
          },
          {
            "t": 3.123996,
            "value": 297350.90082455403
          },
          {
            "t": 4.133229,
            "value": 297255.44051770004
          },
          {
            "t": 5.142145,
            "value": 297348.8377625095
          },
          {
            "t": 6.151027,
            "value": 297358.85861775704
          },
          {
            "t": 7.09318,
            "value": 318419.6197432901
          },
          {
            "t": 8.10208,
            "value": 297353.5533749629
          },
          {
            "t": 9.110991,
            "value": 297350.31137533445
          },
          {
            "t": 10.120662,
            "value": 297126.48971793783
          },
          {
            "t": 11.129514,
            "value": 297367.7011097762
          },
          {
            "t": 12.138219,
            "value": 297411.0369235802
          },
          {
            "t": 13.148996,
            "value": 296801.37161807204
          },
          {
            "t": 14.158266,
            "value": 297244.5430855965
          },
          {
            "t": 15.167161,
            "value": 297355.02703452785
          },
          {
            "t": 16.175779,
            "value": 297436.6906004057
          },
          {
            "t": 17.186019,
            "value": 296959.138422553
          },
          {
            "t": 18.195003,
            "value": 297328.7980780666
          },
          {
            "t": 19.203306,
            "value": 297529.61163459794
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.106564,
            "value": 5132722.805915025
          },
          {
            "t": 3.123996,
            "value": 5133416.144881216
          },
          {
            "t": 5.142145,
            "value": 5123462.142785294
          },
          {
            "t": 7.09318,
            "value": 5291427.8831492
          },
          {
            "t": 9.110991,
            "value": 5132577.332564844
          },
          {
            "t": 11.129514,
            "value": 5132215.981685619
          },
          {
            "t": 13.148996,
            "value": 5128623.082552853
          },
          {
            "t": 15.167161,
            "value": 5132563.492083155
          },
          {
            "t": 17.186019,
            "value": 5122422.676582504
          },
          {
            "t": 19.102703,
            "value": 5403888.173533039
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.106564,
            "value": 5170413.077406107
          },
          {
            "t": 3.123996,
            "value": 5163100.912447112
          },
          {
            "t": 5.142145,
            "value": 5169799.653048412
          },
          {
            "t": 7.09318,
            "value": 5330708.572629399
          },
          {
            "t": 9.110991,
            "value": 5170450.552603787
          },
          {
            "t": 11.129514,
            "value": 5170348.319043182
          },
          {
            "t": 13.148996,
            "value": 5166895.273144301
          },
          {
            "t": 15.167161,
            "value": 5161732.068487958
          },
          {
            "t": 17.186019,
            "value": 5168180.723953839
          },
          {
            "t": 19.102703,
            "value": 5443612.50993904
          }
        ],
        "ram_mib": [
          {
            "t": 1.106564,
            "value": 11.68359375
          },
          {
            "t": 3.123996,
            "value": 11.81640625
          },
          {
            "t": 5.142145,
            "value": 11.4453125
          },
          {
            "t": 7.09318,
            "value": 12.1171875
          },
          {
            "t": 9.110991,
            "value": 11.64453125
          },
          {
            "t": 11.129514,
            "value": 11.8671875
          },
          {
            "t": 13.148996,
            "value": 12.16796875
          },
          {
            "t": 15.167161,
            "value": 11.23046875
          },
          {
            "t": 17.186019,
            "value": 11.328125
          },
          {
            "t": 19.102703,
            "value": 11.3046875
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
      "name": "400k",
      "metrics": [
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 1.2857142686843872
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 23.40552482008079
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 23.98494117647059
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 12.21484375
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 12.5859375
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 396846.74850574316
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/400k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 395848.8212678255
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000548
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 6939369.649568543
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 6865687.276275056
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.092395,
            "value": 23.923101538461538
          },
          {
            "t": 2.106057,
            "value": 23.865771144278607
          },
          {
            "t": 4.119771,
            "value": 23.296596273291925
          },
          {
            "t": 6.086408,
            "value": 23.98494117647059
          },
          {
            "t": 8.105379,
            "value": 23.300261356565027
          },
          {
            "t": 10.118616,
            "value": 23.03861386138614
          },
          {
            "t": 12.132562,
            "value": 23.027188081936686
          },
          {
            "t": 14.14721,
            "value": 23.286169154228855
          },
          {
            "t": 16.162599,
            "value": 22.838066914498143
          },
          {
            "t": 18.181188,
            "value": 23.494538699690402
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.193245,
            "value": 398337.51206932863
          },
          {
            "t": 1.199863,
            "value": 397370.2039899942
          },
          {
            "t": 2.207003,
            "value": 496455.30909307546
          },
          {
            "t": 3.21369,
            "value": 396349.61015688087
          },
          {
            "t": 4.220639,
            "value": 397239.5821436835
          },
          {
            "t": 5.230454,
            "value": 396112.15915786556
          },
          {
            "t": 6.292326,
            "value": 376693.2360962527
          },
          {
            "t": 7.400513,
            "value": 360949.91188310273
          },
          {
            "t": 8.407353,
            "value": 397282.58710420725
          },
          {
            "t": 9.413994,
            "value": 397361.1247703998
          },
          {
            "t": 10.42066,
            "value": 398344.634665321
          },
          {
            "t": 11.4277,
            "value": 396210.6768350811
          },
          {
            "t": 12.434788,
            "value": 397184.7544603848
          },
          {
            "t": 13.441973,
            "value": 397146.5023803968
          },
          {
            "t": 14.449518,
            "value": 397004.6002908059
          },
          {
            "t": 15.457755,
            "value": 396732.1175477591
          },
          {
            "t": 16.466554,
            "value": 396511.09884129546
          },
          {
            "t": 17.47629,
            "value": 396143.15028878837
          },
          {
            "t": 18.488299,
            "value": 396241.535401365
          },
          {
            "t": 19.596201,
            "value": 361042.7637101477
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.092395,
            "value": 396349.21644044627
          },
          {
            "t": 1.098952,
            "value": 397394.28566886927
          },
          {
            "t": 2.106057,
            "value": 397178.04995506926
          },
          {
            "t": 3.112874,
            "value": 397291.66273513454
          },
          {
            "t": 4.119771,
            "value": 397260.09711023077
          },
          {
            "t": 5.128163,
            "value": 397662.8136677007
          },
          {
            "t": 6.18714,
            "value": 377723.0289231966
          },
          {
            "t": 7.198572,
            "value": 394490.18816885364
          },
          {
            "t": 8.205924,
            "value": 398073.36462328956
          },
          {
            "t": 9.212563,
            "value": 397361.91425128566
          },
          {
            "t": 10.219235,
            "value": 396355.5159972662
          },
          {
            "t": 11.226387,
            "value": 397159.5151476639
          },
          {
            "t": 12.233141,
            "value": 397316.5241955831
          },
          {
            "t": 13.240333,
            "value": 397143.74220605404
          },
          {
            "t": 14.248038,
            "value": 396941.5652398271
          },
          {
            "t": 15.256396,
            "value": 396684.51085824677
          },
          {
            "t": 16.263123,
            "value": 398320.4980098875
          },
          {
            "t": 17.27189,
            "value": 395532.36773209274
          },
          {
            "t": 18.28178,
            "value": 396082.74168473796
          },
          {
            "t": 19.29417,
            "value": 396092.41497841745
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.092395,
            "value": 6857866.7295730235
          },
          {
            "t": 2.106057,
            "value": 6857550.572042379
          },
          {
            "t": 4.119771,
            "value": 6848568.863304323
          },
          {
            "t": 6.086408,
            "value": 7013474.77953481
          },
          {
            "t": 8.105379,
            "value": 6848677.3707992835
          },
          {
            "t": 10.118616,
            "value": 6860190.827011424
          },
          {
            "t": 12.132562,
            "value": 6848596.734967075
          },
          {
            "t": 14.14721,
            "value": 6854095.603797784
          },
          {
            "t": 16.162599,
            "value": 6826640.911506414
          },
          {
            "t": 18.181188,
            "value": 6841210.370214046
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.092395,
            "value": 6921852.923283216
          },
          {
            "t": 2.106057,
            "value": 6929534.350849348
          },
          {
            "t": 4.119771,
            "value": 6928526.096555917
          },
          {
            "t": 6.086408,
            "value": 7086277.742155772
          },
          {
            "t": 8.105379,
            "value": 6911707.993824577
          },
          {
            "t": 10.118616,
            "value": 6958167.369266509
          },
          {
            "t": 12.132562,
            "value": 6920145.823175002
          },
          {
            "t": 14.14721,
            "value": 6926205.471129447
          },
          {
            "t": 16.162599,
            "value": 6898534.724561859
          },
          {
            "t": 18.181188,
            "value": 6912744.000883786
          }
        ],
        "ram_mib": [
          {
            "t": 0.092395,
            "value": 12.29296875
          },
          {
            "t": 2.106057,
            "value": 12.21484375
          },
          {
            "t": 4.119771,
            "value": 12.2421875
          },
          {
            "t": 6.086408,
            "value": 12.17578125
          },
          {
            "t": 8.105379,
            "value": 12.0
          },
          {
            "t": 10.118616,
            "value": 12.17578125
          },
          {
            "t": 12.132562,
            "value": 12.15625
          },
          {
            "t": 14.14721,
            "value": 11.98046875
          },
          {
            "t": 16.162599,
            "value": 12.5859375
          },
          {
            "t": 18.181188,
            "value": 12.32421875
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
      "name": "600k",
      "metrics": [
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/600k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.6315789222717285
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/600k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 34.44849087358919
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/600k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 35.794441687344914
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/600k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 12.343359375
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/600k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 12.9140625
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/600k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 586735.9687625888
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/600k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 608703.7139354754
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/600k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000702
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/600k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 10358357.39703664
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/600k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 10293227.867805537
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.069767,
            "value": 35.008811881188116
          },
          {
            "t": 2.089044,
            "value": 35.794441687344914
          },
          {
            "t": 4.109143,
            "value": 33.799303482587064
          },
          {
            "t": 6.146815,
            "value": 34.26271543707378
          },
          {
            "t": 8.170252,
            "value": 33.624089219330855
          },
          {
            "t": 10.09577,
            "value": 35.65746757257566
          },
          {
            "t": 12.116383,
            "value": 34.45728518057285
          },
          {
            "t": 14.136367,
            "value": 34.420657976412166
          },
          {
            "t": 16.156289,
            "value": 34.320347610180015
          },
          {
            "t": 18.175401,
            "value": 33.139788688626474
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.272174,
            "value": 593860.6684099777
          },
          {
            "t": 1.281547,
            "value": 594428.422396874
          },
          {
            "t": 2.291419,
            "value": 594134.7022196872
          },
          {
            "t": 3.300598,
            "value": 594542.6926244006
          },
          {
            "t": 4.329413,
            "value": 583195.229463023
          },
          {
            "t": 5.339284,
            "value": 594135.2905470105
          },
          {
            "t": 6.351561,
            "value": 592723.1380343522
          },
          {
            "t": 7.361365,
            "value": 594174.7111320613
          },
          {
            "t": 8.374296,
            "value": 592340.445696696
          },
          {
            "t": 9.389374,
            "value": 591087.581446943
          },
          {
            "t": 10.499553,
            "value": 540453.3863458055
          },
          {
            "t": 11.509171,
            "value": 594284.1748067092
          },
          {
            "t": 12.520203,
            "value": 593453.0262147983
          },
          {
            "t": 13.5296,
            "value": 594414.2889269533
          },
          {
            "t": 14.540822,
            "value": 593341.5214463292
          },
          {
            "t": 15.549998,
            "value": 594544.4600347213
          },
          {
            "t": 16.559863,
            "value": 594138.8205354181
          },
          {
            "t": 17.569233,
            "value": 594430.1891278718
          },
          {
            "t": 18.58418,
            "value": 592149.1467042122
          },
          {
            "t": 19.600252,
            "value": 491106.9294301978
          },
          {
            "t": 19.701697,
            "value": 89484.09733364235
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.170524,
            "value": 594097.0516943649
          },
          {
            "t": 1.179858,
            "value": 594451.3907190287
          },
          {
            "t": 2.189887,
            "value": 594042.34927908
          },
          {
            "t": 3.199267,
            "value": 594424.3000653867
          },
          {
            "t": 4.329413,
            "value": 796357.2848109889
          },
          {
            "t": 5.339284,
            "value": 594135.2905470105
          },
          {
            "t": 6.250471,
            "value": 658481.7386551828
          },
          {
            "t": 7.260059,
            "value": 594301.8340154599
          },
          {
            "t": 8.271112,
            "value": 593440.6999435242
          },
          {
            "t": 9.282839,
            "value": 592056.9481688242
          },
          {
            "t": 10.297289,
            "value": 592439.2527970821
          },
          {
            "t": 11.307112,
            "value": 594163.5316288102
          },
          {
            "t": 12.317907,
            "value": 593592.1724978853
          },
          {
            "t": 13.327603,
            "value": 594238.2657750451
          },
          {
            "t": 14.337961,
            "value": 593848.9129595648
          },
          {
            "t": 15.347781,
            "value": 594165.2967855658
          },
          {
            "t": 16.35774,
            "value": 594083.5222023864
          },
          {
            "t": 17.367321,
            "value": 594305.9546485125
          },
          {
            "t": 18.377072,
            "value": 594205.898285815
          },
          {
            "t": 19.391698,
            "value": 591350.9017115666
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.069767,
            "value": 10228747.164362768
          },
          {
            "t": 2.089044,
            "value": 10258913.957817575
          },
          {
            "t": 4.109143,
            "value": 10262477.730051843
          },
          {
            "t": 6.146815,
            "value": 10157332.976062879
          },
          {
            "t": 8.170252,
            "value": 10238367.194036681
          },
          {
            "t": 10.09577,
            "value": 10757492.788953414
          },
          {
            "t": 12.116383,
            "value": 10259989.913951855
          },
          {
            "t": 14.136367,
            "value": 10254880.236675141
          },
          {
            "t": 16.156289,
            "value": 10254979.64772897
          },
          {
            "t": 18.175401,
            "value": 10259097.068414235
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.069767,
            "value": 10285067.833084295
          },
          {
            "t": 2.089044,
            "value": 10332132.243372256
          },
          {
            "t": 4.109143,
            "value": 10319359.595742585
          },
          {
            "t": 6.146815,
            "value": 10229423.086738199
          },
          {
            "t": 8.170252,
            "value": 10303242.453310877
          },
          {
            "t": 10.09577,
            "value": 10825492.67262108
          },
          {
            "t": 12.116383,
            "value": 10316574.227721984
          },
          {
            "t": 14.136367,
            "value": 10327779.32894518
          },
          {
            "t": 16.156289,
            "value": 10320012.35691279
          },
          {
            "t": 18.175401,
            "value": 10324490.171917161
          }
        ],
        "ram_mib": [
          {
            "t": 0.069767,
            "value": 12.42578125
          },
          {
            "t": 2.089044,
            "value": 12.21875
          },
          {
            "t": 4.109143,
            "value": 12.14453125
          },
          {
            "t": 6.146815,
            "value": 12.21875
          },
          {
            "t": 8.170252,
            "value": 12.1875
          },
          {
            "t": 10.09577,
            "value": 12.1015625
          },
          {
            "t": 12.116383,
            "value": 12.9140625
          },
          {
            "t": 14.136367,
            "value": 11.7890625
          },
          {
            "t": 16.156289,
            "value": 12.734375
          },
          {
            "t": 18.175401,
            "value": 12.69921875
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
      "name": "800k",
      "metrics": [
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/800k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -0.6451612710952759
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/800k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 39.901647487536295
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/800k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 40.984965255843335
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/800k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 12.827734375
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/800k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 13.46875
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/800k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 792483.4529455025
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/800k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 814602.9058869941
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/800k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000645
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/800k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 13800154.514085934
        },
        {
          "extra": "DFE OTLP Baseline w/ Zstd (Logs)/800k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 13755297.402054638
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 0.125267,
            "value": 39.356681332495285
          },
          {
            "t": 2.098614,
            "value": 39.54459974587039
          },
          {
            "t": 4.126488,
            "value": 39.38700379266751
          },
          {
            "t": 6.144028,
            "value": 40.984965255843335
          },
          {
            "t": 8.162821,
            "value": 39.73723659305994
          },
          {
            "t": 10.088499,
            "value": 38.57417976115651
          },
          {
            "t": 12.105747,
            "value": 39.72600252206809
          },
          {
            "t": 14.124185,
            "value": 40.800504413619166
          },
          {
            "t": 16.147677,
            "value": 40.324631843927
          },
          {
            "t": 18.164956,
            "value": 40.58066961465572
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.327681,
            "value": 720019.5125287895
          },
          {
            "t": 1.291473,
            "value": 830054.6175938378
          },
          {
            "t": 2.305602,
            "value": 788854.2779074458
          },
          {
            "t": 3.319686,
            "value": 493055.8020834566
          },
          {
            "t": 3.420793,
            "value": 358682.9520683004
          },
          {
            "t": 4.429392,
            "value": 757045.5574008218
          },
          {
            "t": 5.437865,
            "value": 793278.5508387433
          },
          {
            "t": 6.446505,
            "value": 793147.2081218276
          },
          {
            "t": 7.456505,
            "value": 891089.108910891
          },
          {
            "t": 8.470343,
            "value": 789080.7012560192
          },
          {
            "t": 9.483644,
            "value": 592124.1565931544
          },
          {
            "t": 9.584834,
            "value": 179454.11851688352
          },
          {
            "t": 10.59238,
            "value": 739658.879855712
          },
          {
            "t": 11.601259,
            "value": 792959.3142487851
          },
          {
            "t": 12.609614,
            "value": 892542.80486535
          },
          {
            "t": 13.619701,
            "value": 792010.9851923648
          },
          {
            "t": 14.63338,
            "value": 789204.4720271408
          },
          {
            "t": 15.743438,
            "value": 720683.0634074976
          },
          {
            "t": 16.752273,
            "value": 792993.8989031903
          },
          {
            "t": 17.761183,
            "value": 792934.9495990723
          },
          {
            "t": 18.769874,
            "value": 793107.1061405326
          },
          {
            "t": 19.785134,
            "value": 689478.5572168706
          },
          {
            "t": 19.886449,
            "value": 89559.59071267044
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.125267,
            "value": 788399.4899055301
          },
          {
            "t": 1.089685,
            "value": 829515.8323465551
          },
          {
            "t": 2.098614,
            "value": 792920.0171667184
          },
          {
            "t": 3.112522,
            "value": 789026.2232865309
          },
          {
            "t": 4.126488,
            "value": 788981.0900957232
          },
          {
            "t": 5.135375,
            "value": 792953.0264539041
          },
          {
            "t": 6.144028,
            "value": 793136.9856630575
          },
          {
            "t": 7.153946,
            "value": 792143.5205630555
          },
          {
            "t": 8.162821,
            "value": 792962.4581836204
          },
          {
            "t": 9.175953,
            "value": 789630.5713372
          },
          {
            "t": 10.189335,
            "value": 789435.7705189158
          },
          {
            "t": 11.197884,
            "value": 793218.7727120845
          },
          {
            "t": 12.206324,
            "value": 793304.509936139
          },
          {
            "t": 13.216565,
            "value": 791890.251929985
          },
          {
            "t": 14.224861,
            "value": 793417.8058823996
          },
          {
            "t": 15.238859,
            "value": 788956.1912350912
          },
          {
            "t": 16.24835,
            "value": 1188717.8786140739
          },
          {
            "t": 17.256827,
            "value": 793275.4043969272
          },
          {
            "t": 18.266046,
            "value": 792692.1708766877
          },
          {
            "t": 19.275702,
            "value": 792349.0773094994
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 0.125267,
            "value": 13643592.967246369
          },
          {
            "t": 2.098614,
            "value": 13992879.103371074
          },
          {
            "t": 4.126488,
            "value": 13590761.55619136
          },
          {
            "t": 6.144028,
            "value": 13683142.837316733
          },
          {
            "t": 8.162821,
            "value": 13643471.618932698
          },
          {
            "t": 10.088499,
            "value": 14327644.601018446
          },
          {
            "t": 12.105747,
            "value": 13687436.051491935
          },
          {
            "t": 14.124185,
            "value": 13678621.290324498
          },
          {
            "t": 16.147677,
            "value": 13645515.771745082
          },
          {
            "t": 18.164956,
            "value": 13659908.222908186
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 0.125267,
            "value": 13686929.446528835
          },
          {
            "t": 2.098614,
            "value": 14038462.064705295
          },
          {
            "t": 4.126488,
            "value": 13635667.206147915
          },
          {
            "t": 6.144028,
            "value": 13728420.254369183
          },
          {
            "t": 8.162821,
            "value": 13687505.851268554
          },
          {
            "t": 10.088499,
            "value": 14375867.097199012
          },
          {
            "t": 12.105747,
            "value": 13732312.536683641
          },
          {
            "t": 14.124185,
            "value": 13722873.826196294
          },
          {
            "t": 16.147677,
            "value": 13689985.431125993
          },
          {
            "t": 18.164956,
            "value": 13703521.426634591
          }
        ],
        "ram_mib": [
          {
            "t": 0.125267,
            "value": 13.00390625
          },
          {
            "t": 2.098614,
            "value": 12.34375
          },
          {
            "t": 4.126488,
            "value": 11.953125
          },
          {
            "t": 6.144028,
            "value": 11.921875
          },
          {
            "t": 8.162821,
            "value": 12.75390625
          },
          {
            "t": 10.088499,
            "value": 13.08203125
          },
          {
            "t": 12.105747,
            "value": 13.46875
          },
          {
            "t": 14.124185,
            "value": 13.25
          },
          {
            "t": 16.147677,
            "value": 13.25
          },
          {
            "t": 18.164956,
            "value": 13.25
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
