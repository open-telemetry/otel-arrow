window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["dfe_logs_otap_none_batch"] = {
  "name": "DFE OTAP Batch Processor (Logs)",
  "slug": "dfe_logs_otap_none_batch",
  "description": "Dataflow Engine OTAP logs through a batch processor with no compression",
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
  "tests": [
    {
      "name": "100k",
      "metrics": [
        {
          "extra": "DFE OTAP Batch Processor (Logs)/100k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 5.097451210021973
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 7.782957051714249
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/100k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 8.985544554455444
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 19.3421875
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/100k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 19.7421875
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/100k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 104629.70496410171
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/100k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 99296.25673504705
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/100k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000621
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/100k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 10755503.03211508
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/100k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 11051605.048028084
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.09795,
            "value": 7.943436341161929
          },
          {
            "t": 3.110338,
            "value": 7.446283240568954
          },
          {
            "t": 5.123245,
            "value": 7.827844348363187
          },
          {
            "t": 7.136518,
            "value": 8.985544554455444
          },
          {
            "t": 9.15083,
            "value": 8.216395061728395
          },
          {
            "t": 11.163768,
            "value": 7.60456009913259
          },
          {
            "t": 13.177383,
            "value": 7.663079777365492
          },
          {
            "t": 15.089887,
            "value": 7.10707768187423
          },
          {
            "t": 17.101651,
            "value": 7.583920841063699
          },
          {
            "t": 19.115532,
            "value": 7.451428571428571
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.09155,
            "value": 98394.08919871431
          },
          {
            "t": 1.09795,
            "value": 99364.06995230525
          },
          {
            "t": 2.103932,
            "value": 99405.35715350772
          },
          {
            "t": 3.110338,
            "value": 99363.47756273314
          },
          {
            "t": 4.116658,
            "value": 99371.96915494076
          },
          {
            "t": 5.123245,
            "value": 99345.61046387447
          },
          {
            "t": 6.129509,
            "value": 198754.998688217
          },
          {
            "t": 7.136518,
            "value": 99303.97841528725
          },
          {
            "t": 8.14315,
            "value": 99341.16936477282
          },
          {
            "t": 9.15083,
            "value": 99237.85328675772
          },
          {
            "t": 10.157318,
            "value": 99355.38227976885
          },
          {
            "t": 11.163768,
            "value": 99359.13358835511
          },
          {
            "t": 12.171496,
            "value": 100225.45766317895
          },
          {
            "t": 13.177383,
            "value": 98420.59793992765
          },
          {
            "t": 14.183497,
            "value": 99392.3153837438
          },
          {
            "t": 15.190329,
            "value": 99321.43594959239
          },
          {
            "t": 16.196347,
            "value": 99401.7999677938
          },
          {
            "t": 17.202123,
            "value": 99425.71705827142
          },
          {
            "t": 18.208362,
            "value": 99379.96837729405
          },
          {
            "t": 19.216138,
            "value": 100220.68396151526
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.09155,
            "value": 98394.08919871431
          },
          {
            "t": 1.09795,
            "value": 98370.4292527822
          },
          {
            "t": 2.103932,
            "value": 98411.30358197265
          },
          {
            "t": 3.110338,
            "value": 98369.84278710581
          },
          {
            "t": 4.116658,
            "value": 107321.72668733602
          },
          {
            "t": 5.123245,
            "value": 98352.15435923572
          },
          {
            "t": 6.129509,
            "value": 98383.72435066741
          },
          {
            "t": 7.136518,
            "value": 98310.93863113437
          },
          {
            "t": 8.14315,
            "value": 98347.7576711251
          },
          {
            "t": 9.15083,
            "value": 98245.47475389013
          },
          {
            "t": 10.157318,
            "value": 98361.82845697117
          },
          {
            "t": 11.163768,
            "value": 98365.54225247155
          },
          {
            "t": 12.171496,
            "value": 98240.79513519522
          },
          {
            "t": 13.177383,
            "value": 107367.92502537562
          },
          {
            "t": 14.183497,
            "value": 98398.39222990636
          },
          {
            "t": 15.190329,
            "value": 98328.22159009647
          },
          {
            "t": 16.196347,
            "value": 98407.78196811587
          },
          {
            "t": 17.202123,
            "value": 98431.4598876887
          },
          {
            "t": 18.208362,
            "value": 98386.16869352112
          },
          {
            "t": 19.216138,
            "value": 98236.11596227733
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.09795,
            "value": 11006779.431946807
          },
          {
            "t": 3.110338,
            "value": 10953106.45859546
          },
          {
            "t": 5.123245,
            "value": 11006125.469283978
          },
          {
            "t": 7.136518,
            "value": 11007389.956553334
          },
          {
            "t": 9.15083,
            "value": 10991923.793334896
          },
          {
            "t": 11.163768,
            "value": 11004479.025186071
          },
          {
            "t": 13.177383,
            "value": 11004436.299888508
          },
          {
            "t": 15.089887,
            "value": 11585996.160008032
          },
          {
            "t": 17.101651,
            "value": 11012481.583326872
          },
          {
            "t": 19.115532,
            "value": 10943332.30215688
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.09795,
            "value": 11065265.199810391
          },
          {
            "t": 3.110338,
            "value": 10558651.711300205
          },
          {
            "t": 5.123245,
            "value": 10556852.850131676
          },
          {
            "t": 7.136518,
            "value": 11030376.903678738
          },
          {
            "t": 9.15083,
            "value": 10545703.942586849
          },
          {
            "t": 11.163768,
            "value": 10555463.208504185
          },
          {
            "t": 13.177383,
            "value": 11028658.904507564
          },
          {
            "t": 15.089887,
            "value": 11108627.2237862
          },
          {
            "t": 17.101651,
            "value": 10558447.213490251
          },
          {
            "t": 19.115532,
            "value": 10546983.163354736
          }
        ],
        "ram_mib": [
          {
            "t": 1.09795,
            "value": 18.890625
          },
          {
            "t": 3.110338,
            "value": 18.75
          },
          {
            "t": 5.123245,
            "value": 19.328125
          },
          {
            "t": 7.136518,
            "value": 19.1171875
          },
          {
            "t": 9.15083,
            "value": 19.63671875
          },
          {
            "t": 11.163768,
            "value": 19.67578125
          },
          {
            "t": 13.177383,
            "value": 19.4375
          },
          {
            "t": 15.089887,
            "value": 19.234375
          },
          {
            "t": 17.101651,
            "value": 19.609375
          },
          {
            "t": 19.115532,
            "value": 19.7421875
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
          "extra": "DFE OTAP Batch Processor (Logs)/200k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 2.5904078483581543
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 12.084160055273633
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/200k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 12.744765478424014
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 18.958984375
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/200k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 19.83203125
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/200k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 203642.3835006528
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/200k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 198367.21532071795
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/200k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000782
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/200k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 21434589.16684196
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/200k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 22095154.735820893
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.099308,
            "value": 12.127279549718574
          },
          {
            "t": 3.114236,
            "value": 11.334858579509742
          },
          {
            "t": 5.128457,
            "value": 11.292388714733542
          },
          {
            "t": 7.143645,
            "value": 12.744765478424014
          },
          {
            "t": 9.059624,
            "value": 12.681656210790464
          },
          {
            "t": 11.074696,
            "value": 11.932215269086358
          },
          {
            "t": 13.088987,
            "value": 11.93318352059925
          },
          {
            "t": 15.10555,
            "value": 12.378805970149253
          },
          {
            "t": 17.121315,
            "value": 12.668728179551122
          },
          {
            "t": 19.136339,
            "value": 11.74771908017402
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.091359,
            "value": 199451.656788352
          },
          {
            "t": 1.099308,
            "value": 197430.62397006198
          },
          {
            "t": 2.106568,
            "value": 198558.4655401783
          },
          {
            "t": 3.114236,
            "value": 198478.07015802825
          },
          {
            "t": 4.121426,
            "value": 298851.25944459334
          },
          {
            "t": 5.128457,
            "value": 197610.59987229787
          },
          {
            "t": 6.136145,
            "value": 198474.13088178087
          },
          {
            "t": 7.143645,
            "value": 198511.16625310172
          },
          {
            "t": 8.152319,
            "value": 198280.11825426252
          },
          {
            "t": 9.16069,
            "value": 199331.3968767448
          },
          {
            "t": 10.167957,
            "value": 197564.30023022695
          },
          {
            "t": 11.175284,
            "value": 198545.25888812667
          },
          {
            "t": 12.182501,
            "value": 199559.7770887505
          },
          {
            "t": 13.18968,
            "value": 197581.56196664146
          },
          {
            "t": 14.198299,
            "value": 198290.9304702767
          },
          {
            "t": 15.206232,
            "value": 198425.88743497833
          },
          {
            "t": 16.214745,
            "value": 198311.77188593504
          },
          {
            "t": 17.22196,
            "value": 198567.33666595514
          },
          {
            "t": 18.229717,
            "value": 198460.54157897193
          },
          {
            "t": 19.237668,
            "value": 198422.34394330677
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.091359,
            "value": 196474.76638852587
          },
          {
            "t": 1.099308,
            "value": 196438.5102817702
          },
          {
            "t": 2.106568,
            "value": 205508.01183408455
          },
          {
            "t": 3.114236,
            "value": 196493.28945644796
          },
          {
            "t": 4.121426,
            "value": 196586.54275757304
          },
          {
            "t": 5.128457,
            "value": 196617.58178248734
          },
          {
            "t": 6.136145,
            "value": 205420.72546264323
          },
          {
            "t": 7.143645,
            "value": 196526.0545905707
          },
          {
            "t": 8.152319,
            "value": 196297.31707171988
          },
          {
            "t": 9.16069,
            "value": 196356.30140097247
          },
          {
            "t": 10.167957,
            "value": 196571.51480193436
          },
          {
            "t": 11.175284,
            "value": 205494.3429492111
          },
          {
            "t": 12.182501,
            "value": 196581.27295309748
          },
          {
            "t": 13.18968,
            "value": 196588.68979595482
          },
          {
            "t": 14.198299,
            "value": 196308.02116557394
          },
          {
            "t": 15.206232,
            "value": 205370.79349520255
          },
          {
            "t": 16.214745,
            "value": 196328.6541670757
          },
          {
            "t": 17.22196,
            "value": 196581.6632992956
          },
          {
            "t": 18.229717,
            "value": 196475.9361631822
          },
          {
            "t": 19.237668,
            "value": 196438.12050387368
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.099308,
            "value": 21987063.132034734
          },
          {
            "t": 3.114236,
            "value": 21996179.51609189
          },
          {
            "t": 5.128457,
            "value": 21893331.963076543
          },
          {
            "t": 7.143645,
            "value": 21994570.233645692
          },
          {
            "t": 9.059624,
            "value": 23137399.73141668
          },
          {
            "t": 11.074696,
            "value": 21987314.597195536
          },
          {
            "t": 13.088987,
            "value": 21998684.4006154
          },
          {
            "t": 15.10555,
            "value": 21979180.41737352
          },
          {
            "t": 17.121315,
            "value": 21987109.60851091
          },
          {
            "t": 19.136339,
            "value": 21990713.758248042
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.099308,
            "value": 21031468.285151847
          },
          {
            "t": 3.114236,
            "value": 21516392.645295516
          },
          {
            "t": 5.128457,
            "value": 21523335.820647288
          },
          {
            "t": 7.143645,
            "value": 21036887.37725711
          },
          {
            "t": 9.059624,
            "value": 22626916.57893954
          },
          {
            "t": 11.074696,
            "value": 21515131.46924775
          },
          {
            "t": 13.088987,
            "value": 21047075.621149078
          },
          {
            "t": 15.10555,
            "value": 21499604.525125176
          },
          {
            "t": 17.121315,
            "value": 21507342.373739
          },
          {
            "t": 19.136339,
            "value": 21041736.971867334
          }
        ],
        "ram_mib": [
          {
            "t": 1.099308,
            "value": 17.82421875
          },
          {
            "t": 3.114236,
            "value": 19.01171875
          },
          {
            "t": 5.128457,
            "value": 18.73046875
          },
          {
            "t": 7.143645,
            "value": 19.83203125
          },
          {
            "t": 9.059624,
            "value": 19.3125
          },
          {
            "t": 11.074696,
            "value": 18.921875
          },
          {
            "t": 13.088987,
            "value": 18.88671875
          },
          {
            "t": 15.10555,
            "value": 18.53125
          },
          {
            "t": 17.121315,
            "value": 19.0625
          },
          {
            "t": 19.136339,
            "value": 19.4765625
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
          "extra": "DFE OTAP Batch Processor (Logs)/300k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -2.6315789222717285
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 16.897157791181794
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/300k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 17.54974800245851
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 20.84609375
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/300k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 21.69140625
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/300k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 296327.2883952827
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/300k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 302532.0693043567
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/300k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000681
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/300k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 31976937.497956775
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/300k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 32992475.1744278
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.065311,
            "value": 16.95010410287814
          },
          {
            "t": 3.090073,
            "value": 16.611614487415594
          },
          {
            "t": 5.118933,
            "value": 17.54974800245851
          },
          {
            "t": 7.14434,
            "value": 16.417322640345468
          },
          {
            "t": 9.06538,
            "value": 17.176995708154507
          },
          {
            "t": 11.091615,
            "value": 16.124794099569762
          },
          {
            "t": 13.117334,
            "value": 16.486309303758475
          },
          {
            "t": 15.139606,
            "value": 17.2560987654321
          },
          {
            "t": 17.163333,
            "value": 17.29600976205003
          },
          {
            "t": 19.183317,
            "value": 17.102581039755353
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.051838,
            "value": 296159.98957516835
          },
          {
            "t": 1.065311,
            "value": 296011.8325796543
          },
          {
            "t": 2.077832,
            "value": 296290.15101909
          },
          {
            "t": 3.090073,
            "value": 296372.10901356494
          },
          {
            "t": 4.104735,
            "value": 295664.96035132883
          },
          {
            "t": 5.118933,
            "value": 295800.2283577763
          },
          {
            "t": 6.131448,
            "value": 296291.9067865661
          },
          {
            "t": 7.14434,
            "value": 296181.6264715291
          },
          {
            "t": 8.15687,
            "value": 296287.5174068917
          },
          {
            "t": 9.171201,
            "value": 295761.44276375265
          },
          {
            "t": 10.183156,
            "value": 296455.870073274
          },
          {
            "t": 11.197973,
            "value": 295619.80140261736
          },
          {
            "t": 12.20814,
            "value": 296980.5982575158
          },
          {
            "t": 13.218408,
            "value": 296950.9080758769
          },
          {
            "t": 14.231193,
            "value": 296212.9178453472
          },
          {
            "t": 15.243103,
            "value": 296469.0535719579
          },
          {
            "t": 16.254661,
            "value": 296572.2183008785
          },
          {
            "t": 17.264111,
            "value": 297191.5399474962
          },
          {
            "t": 18.275237,
            "value": 296698.9277300752
          },
          {
            "t": 19.287326,
            "value": 296416.6194870214
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.051838,
            "value": 293198.38967941666
          },
          {
            "t": 1.065311,
            "value": 301932.06923124735
          },
          {
            "t": 2.077832,
            "value": 293327.24950889905
          },
          {
            "t": 3.090073,
            "value": 293408.3879234293
          },
          {
            "t": 4.104735,
            "value": 301578.25955835544
          },
          {
            "t": 5.118933,
            "value": 292842.22607419855
          },
          {
            "t": 6.131448,
            "value": 293328.98771870043
          },
          {
            "t": 7.14434,
            "value": 302105.25900095963
          },
          {
            "t": 8.15687,
            "value": 293324.6422328228
          },
          {
            "t": 9.171201,
            "value": 292803.8283361151
          },
          {
            "t": 10.183156,
            "value": 302384.9874747395
          },
          {
            "t": 11.197973,
            "value": 292663.6033885912
          },
          {
            "t": 12.310928,
            "value": 266857.15055864793
          },
          {
            "t": 13.319923,
            "value": 303272.06775058346
          },
          {
            "t": 14.332971,
            "value": 293174.657074492
          },
          {
            "t": 15.243103,
            "value": 316437.6156425661
          },
          {
            "t": 16.254661,
            "value": 311400.8292159224
          },
          {
            "t": 17.36667,
            "value": 404672.98376182205
          },
          {
            "t": 18.376674,
            "value": 294058.24135349964
          },
          {
            "t": 19.388631,
            "value": 293490.7313255405
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.065311,
            "value": 32756372.138514902
          },
          {
            "t": 3.090073,
            "value": 32825102.407097723
          },
          {
            "t": 5.118933,
            "value": 32719399.564287335
          },
          {
            "t": 7.14434,
            "value": 32719787.183514226
          },
          {
            "t": 9.06538,
            "value": 34605777.59963353
          },
          {
            "t": 11.091615,
            "value": 32816332.755085174
          },
          {
            "t": 13.117334,
            "value": 32830027.758045413
          },
          {
            "t": 15.139606,
            "value": 32877906.631748844
          },
          {
            "t": 17.163333,
            "value": 32859486.482119378
          },
          {
            "t": 19.183317,
            "value": 32914559.224231478
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.065311,
            "value": 31840615.97709085
          },
          {
            "t": 3.090073,
            "value": 31375137.917444125
          },
          {
            "t": 5.118933,
            "value": 31787148.44789685
          },
          {
            "t": 7.14434,
            "value": 31841084.779503576
          },
          {
            "t": 9.06538,
            "value": 33569210.427685
          },
          {
            "t": 11.091615,
            "value": 31829311.999842074
          },
          {
            "t": 13.117334,
            "value": 31839272.376869645
          },
          {
            "t": 15.139606,
            "value": 31890627.966959935
          },
          {
            "t": 17.163333,
            "value": 31869697.34554117
          },
          {
            "t": 19.183317,
            "value": 31927267.74073458
          }
        ],
        "ram_mib": [
          {
            "t": 1.065311,
            "value": 21.359375
          },
          {
            "t": 3.090073,
            "value": 20.515625
          },
          {
            "t": 5.118933,
            "value": 19.66015625
          },
          {
            "t": 7.14434,
            "value": 21.69140625
          },
          {
            "t": 9.06538,
            "value": 20.1875
          },
          {
            "t": 11.091615,
            "value": 21.14453125
          },
          {
            "t": 13.117334,
            "value": 21.64453125
          },
          {
            "t": 15.139606,
            "value": 19.88671875
          },
          {
            "t": 17.163333,
            "value": 21.31640625
          },
          {
            "t": 19.183317,
            "value": 21.0546875
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
          "extra": "DFE OTAP Batch Processor (Logs)/400k - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": -0.06578947603702545
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 22.803350348611133
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/400k - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 23.52496277915633
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 20.9203125
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/400k - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 23.76171875
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/400k - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 394170.9174888242
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/400k - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 398623.18017117784
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/400k - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000653
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/400k - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 42805053.82303093
        },
        {
          "extra": "DFE OTAP Batch Processor (Logs)/400k - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 44115589.98463827
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.090339,
            "value": 23.101378026070762
          },
          {
            "t": 3.111783,
            "value": 23.52496277915633
          },
          {
            "t": 5.134037,
            "value": 23.078004987531173
          },
          {
            "t": 7.056277,
            "value": 22.245966438781853
          },
          {
            "t": 9.143077,
            "value": 22.15835103060587
          },
          {
            "t": 11.164933,
            "value": 21.879725856697817
          },
          {
            "t": 13.086602,
            "value": 22.863830318153465
          },
          {
            "t": 15.10921,
            "value": 22.585552089831566
          },
          {
            "t": 17.132363,
            "value": 23.350629283489095
          },
          {
            "t": 19.157745,
            "value": 23.2451026757934
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.079577,
            "value": 395600.52654430084
          },
          {
            "t": 1.090339,
            "value": 395741.0349815288
          },
          {
            "t": 2.100688,
            "value": 395902.8019031048
          },
          {
            "t": 3.111783,
            "value": 395610.6992913623
          },
          {
            "t": 4.123553,
            "value": 395346.7685343507
          },
          {
            "t": 5.134037,
            "value": 396839.5343221664
          },
          {
            "t": 6.146962,
            "value": 393908.7296690278
          },
          {
            "t": 7.160845,
            "value": 394522.83942032757
          },
          {
            "t": 8.176177,
            "value": 393959.8082203654
          },
          {
            "t": 9.244788,
            "value": 375253.48326004506
          },
          {
            "t": 10.255607,
            "value": 394729.4223792786
          },
          {
            "t": 11.266628,
            "value": 395639.65535829624
          },
          {
            "t": 12.278269,
            "value": 395397.1814111923
          },
          {
            "t": 13.289073,
            "value": 395724.5915132904
          },
          {
            "t": 14.299545,
            "value": 395854.6105186487
          },
          {
            "t": 15.31182,
            "value": 395149.53940381814
          },
          {
            "t": 16.323342,
            "value": 395443.6977149286
          },
          {
            "t": 17.335108,
            "value": 395348.3315312039
          },
          {
            "t": 18.34981,
            "value": 394204.40681106376
          },
          {
            "t": 19.360553,
            "value": 395748.47414228943
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.079577,
            "value": 391644.52127885784
          },
          {
            "t": 1.090339,
            "value": 400687.79791879794
          },
          {
            "t": 2.100688,
            "value": 391943.7738840737
          },
          {
            "t": 3.111783,
            "value": 400555.8330325043
          },
          {
            "t": 4.123553,
            "value": 391393.30084900715
          },
          {
            "t": 5.134037,
            "value": 400798.0334176494
          },
          {
            "t": 6.146962,
            "value": 390947.0098970802
          },
          {
            "t": 7.160845,
            "value": 390577.6110261243
          },
          {
            "t": 8.176177,
            "value": 398884.30582311994
          },
          {
            "t": 9.143077,
            "value": 409556.31399317406
          },
          {
            "t": 10.153812,
            "value": 400698.5015854799
          },
          {
            "t": 11.164933,
            "value": 391644.52127885784
          },
          {
            "t": 12.175237,
            "value": 400869.4412770809
          },
          {
            "t": 13.187511,
            "value": 391198.430464479
          },
          {
            "t": 14.197866,
            "value": 400849.2064670339
          },
          {
            "t": 15.210115,
            "value": 391208.09208011074
          },
          {
            "t": 16.221847,
            "value": 391408.001328415
          },
          {
            "t": 17.233371,
            "value": 400385.95228585775
          },
          {
            "t": 18.146567,
            "value": 433641.84687624563
          },
          {
            "t": 19.157745,
            "value": 400522.9544155431
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.090339,
            "value": 43838607.37738039
          },
          {
            "t": 3.111783,
            "value": 43797479.42559873
          },
          {
            "t": 5.134037,
            "value": 43778116.39883021
          },
          {
            "t": 7.056277,
            "value": 46112738.26369236
          },
          {
            "t": 9.143077,
            "value": 42182164.078972585
          },
          {
            "t": 11.164933,
            "value": 43966578.72766408
          },
          {
            "t": 13.086602,
            "value": 46134484.65890848
          },
          {
            "t": 15.10921,
            "value": 43824386.139083795
          },
          {
            "t": 17.132363,
            "value": 43700460.12338167
          },
          {
            "t": 19.157745,
            "value": 43820884.652870424
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.090339,
            "value": 42841376.08358149
          },
          {
            "t": 3.111783,
            "value": 42377306.024802074
          },
          {
            "t": 5.134037,
            "value": 42359896.43239672
          },
          {
            "t": 7.056277,
            "value": 44564567.37972366
          },
          {
            "t": 9.143077,
            "value": 41047383.07456392
          },
          {
            "t": 11.164933,
            "value": 42845727.88566545
          },
          {
            "t": 13.086602,
            "value": 44572803.64100165
          },
          {
            "t": 15.10921,
            "value": 42346048.76476312
          },
          {
            "t": 17.132363,
            "value": 42335157.054360196
          },
          {
            "t": 19.157745,
            "value": 42760271.889450975
          }
        ],
        "ram_mib": [
          {
            "t": 1.090339,
            "value": 19.94921875
          },
          {
            "t": 3.111783,
            "value": 20.33984375
          },
          {
            "t": 5.134037,
            "value": 19.01171875
          },
          {
            "t": 7.056277,
            "value": 23.76171875
          },
          {
            "t": 9.143077,
            "value": 21.87109375
          },
          {
            "t": 11.164933,
            "value": 19.5546875
          },
          {
            "t": 13.086602,
            "value": 21.54296875
          },
          {
            "t": 15.10921,
            "value": 20.01171875
          },
          {
            "t": 17.132363,
            "value": 20.8046875
          },
          {
            "t": 19.157745,
            "value": 22.35546875
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
