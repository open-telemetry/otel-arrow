window.SUITE_DATA = window.SUITE_DATA || {};
window.SUITE_DATA["otc_logs_otap_none_transform_insert_multi_transform"] = {
  "name": "OTC OTAP Transform Insert Multi Transform (Logs)",
  "slug": "otc_logs_otap_none_transform_insert_multi_transform",
  "description": "OpenTelemetry Collector OTAP logs, transform processor (OTTL) insert sweep over 1-4 insert actions at 240k signals/sec",
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
    "started_at": "2026-05-27T23:40:29Z",
    "ended_at": "2026-05-27T23:44:40Z",
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
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-1 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 7.561307907104492
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.16122389607159
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-1 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.32499375780274
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 407.47421875
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-1 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 436.07421875
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 74927.01633934671
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-1 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 71098.55706080975
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-1 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000633
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 10017145.718253655
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-1 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9819412.978612704
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-1 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 140.89098474510686
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.088044,
            "value": 100.20229426433916
          },
          {
            "t": 3.105772,
            "value": 100.2561895852822
          },
          {
            "t": 5.118787,
            "value": 100.25229809791082
          },
          {
            "t": 7.038847,
            "value": 100.07322540473224
          },
          {
            "t": 9.08632,
            "value": 100.32499375780274
          },
          {
            "t": 11.099868,
            "value": 100.12571784490814
          },
          {
            "t": 13.119974,
            "value": 100.24226861950763
          },
          {
            "t": 15.134431,
            "value": 100.26081097941359
          },
          {
            "t": 17.154464,
            "value": 99.7842567778124
          },
          {
            "t": 19.167554,
            "value": 100.09018362900717
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.182851,
            "value": 155959.4386254907
          },
          {
            "t": 1.193897,
            "value": 77147.8251236838
          },
          {
            "t": 2.300257,
            "value": 68693.73440833001
          },
          {
            "t": 3.307114,
            "value": 76475.6067644164
          },
          {
            "t": 4.313435,
            "value": 77510.05891758196
          },
          {
            "t": 5.32242,
            "value": 75323.22086056779
          },
          {
            "t": 6.434946,
            "value": 68313.01021279502
          },
          {
            "t": 7.441413,
            "value": 72530.94239552812
          },
          {
            "t": 8.48174,
            "value": 77860.13436159978
          },
          {
            "t": 9.488193,
            "value": 76506.30481502862
          },
          {
            "t": 10.495555,
            "value": 78422.6524327898
          },
          {
            "t": 11.506901,
            "value": 77124.94042592743
          },
          {
            "t": 12.615313,
            "value": 68566.5618921484
          },
          {
            "t": 13.622431,
            "value": 74469.92308746344
          },
          {
            "t": 14.629666,
            "value": 75454.08966130047
          },
          {
            "t": 15.641295,
            "value": 74137.8509315174
          },
          {
            "t": 16.748455,
            "value": 74063.36934137794
          },
          {
            "t": 17.757368,
            "value": 67399.2703037824
          },
          {
            "t": 18.764085,
            "value": 89399.50353475702
          },
          {
            "t": 19.77525,
            "value": 76149.7876212092
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.081372,
            "value": 70580.04871017447
          },
          {
            "t": 1.088044,
            "value": 76489.66098192857
          },
          {
            "t": 2.099058,
            "value": 75172.05498638001
          },
          {
            "t": 3.105772,
            "value": 66553.16206986293
          },
          {
            "t": 4.112141,
            "value": 64588.63498378825
          },
          {
            "t": 5.118787,
            "value": 77485.03446097238
          },
          {
            "t": 6.127382,
            "value": 71386.43360318067
          },
          {
            "t": 7.139219,
            "value": 67204.50032959854
          },
          {
            "t": 8.180247,
            "value": 65320.04902846032
          },
          {
            "t": 9.186748,
            "value": 71534.95128171754
          },
          {
            "t": 10.193959,
            "value": 67513.16258460243
          },
          {
            "t": 11.20027,
            "value": 79498.28631506562
          },
          {
            "t": 12.112902,
            "value": 73414.0376405824
          },
          {
            "t": 13.119974,
            "value": 71494.39166216517
          },
          {
            "t": 14.127138,
            "value": 75459.40879538983
          },
          {
            "t": 15.134431,
            "value": 77435.26461516162
          },
          {
            "t": 16.145447,
            "value": 59346.241800327596
          },
          {
            "t": 17.154464,
            "value": 66401.25984002251
          },
          {
            "t": 18.161268,
            "value": 75486.3905983687
          },
          {
            "t": 19.167554,
            "value": 69562.72868747056
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.088044,
            "value": 9683809.478381932
          },
          {
            "t": 3.105772,
            "value": 10018021.259555303
          },
          {
            "t": 5.118787,
            "value": 9982589.796896694
          },
          {
            "t": 7.038847,
            "value": 10325809.089299291
          },
          {
            "t": 9.08632,
            "value": 9411936.567661697
          },
          {
            "t": 11.099868,
            "value": 9555632.147830594
          },
          {
            "t": 13.119974,
            "value": 9929544.291240161
          },
          {
            "t": 15.134431,
            "value": 10024928.305741943
          },
          {
            "t": 17.154464,
            "value": 9564391.274796005
          },
          {
            "t": 19.167554,
            "value": 9697467.574723436
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.088044,
            "value": 10411837.89106946
          },
          {
            "t": 3.105772,
            "value": 9356139.182288198
          },
          {
            "t": 5.118787,
            "value": 10420973.018084811
          },
          {
            "t": 7.038847,
            "value": 10360571.023822172
          },
          {
            "t": 9.08632,
            "value": 9713801.84256398
          },
          {
            "t": 11.099868,
            "value": 10916882.53768969
          },
          {
            "t": 13.119974,
            "value": 8562126.442869829
          },
          {
            "t": 15.134431,
            "value": 10678487.056313438
          },
          {
            "t": 17.154464,
            "value": 9855792.949917154
          },
          {
            "t": 19.167554,
            "value": 9894845.237917827
          }
        ],
        "ram_mib": [
          {
            "t": 1.088044,
            "value": 375.07421875
          },
          {
            "t": 3.105772,
            "value": 394.9375
          },
          {
            "t": 5.118787,
            "value": 393.42578125
          },
          {
            "t": 7.038847,
            "value": 405.16796875
          },
          {
            "t": 9.08632,
            "value": 435.25
          },
          {
            "t": 11.099868,
            "value": 398.12109375
          },
          {
            "t": 13.119974,
            "value": 436.07421875
          },
          {
            "t": 15.134431,
            "value": 410.0234375
          },
          {
            "t": 17.154464,
            "value": 430.08984375
          },
          {
            "t": 19.167554,
            "value": 396.578125
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
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-2 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 5.46021842956543
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.00174784310923
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-2 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.29016780609075
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 862.62421875
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-2 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 878.0703125
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 65605.97532263478
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-2 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 63343.60140777496
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-2 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.00055
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9486533.814357929
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-2 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9085550.126747629
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-2 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 149.76309530126474
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.019445,
            "value": 99.80558603491272
          },
          {
            "t": 3.087806,
            "value": 100.2052867830424
          },
          {
            "t": 5.104667,
            "value": 99.80019950124688
          },
          {
            "t": 7.126235,
            "value": 99.7740853848551
          },
          {
            "t": 9.043043,
            "value": 100.15491588785046
          },
          {
            "t": 11.064025,
            "value": 100.29016780609075
          },
          {
            "t": 13.080881,
            "value": 100.22206423448705
          },
          {
            "t": 15.103076,
            "value": 100.0408839091192
          },
          {
            "t": 17.125353,
            "value": 100.00945861854387
          },
          {
            "t": 19.144231,
            "value": 99.71483027094364
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.212065,
            "value": 230055.34219244722
          },
          {
            "t": 1.221111,
            "value": 102076.61494124151
          },
          {
            "t": 2.281145,
            "value": 67922.34966048259
          },
          {
            "t": 3.289428,
            "value": 66449.59797993222
          },
          {
            "t": 4.298409,
            "value": 59465.93642496738
          },
          {
            "t": 5.311294,
            "value": 67134.96596355955
          },
          {
            "t": 6.419594,
            "value": 57746.0976269963
          },
          {
            "t": 7.428385,
            "value": 62450.99331774372
          },
          {
            "t": 8.437312,
            "value": 67398.33506289354
          },
          {
            "t": 9.445468,
            "value": 63482.23885985899
          },
          {
            "t": 10.458641,
            "value": 64154.88766479169
          },
          {
            "t": 11.567722,
            "value": 59508.72839765536
          },
          {
            "t": 12.575643,
            "value": 63497.039946583114
          },
          {
            "t": 13.583788,
            "value": 65466.77313283307
          },
          {
            "t": 14.597949,
            "value": 66064.46116543625
          },
          {
            "t": 15.611344,
            "value": 60193.70531727509
          },
          {
            "t": 16.720699,
            "value": 55888.33150794831
          },
          {
            "t": 17.729483,
            "value": 67407.88910212692
          },
          {
            "t": 18.738513,
            "value": 66400.40434873095
          },
          {
            "t": 19.752968,
            "value": 66045.3149720786
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.11103,
            "value": 47599.40342081046
          },
          {
            "t": 1.119983,
            "value": 58476.46025136949
          },
          {
            "t": 2.079651,
            "value": 65647.70316401089
          },
          {
            "t": 3.087806,
            "value": 53563.19216787101
          },
          {
            "t": 4.095904,
            "value": 71421.62765921568
          },
          {
            "t": 5.104667,
            "value": 63444.03987854431
          },
          {
            "t": 6.117653,
            "value": 66141.09178211742
          },
          {
            "t": 7.126235,
            "value": 59489.46144190556
          },
          {
            "t": 8.135162,
            "value": 71362.94300776965
          },
          {
            "t": 9.143589,
            "value": 61481.8920953128
          },
          {
            "t": 10.151581,
            "value": 62500.49603568282
          },
          {
            "t": 11.164583,
            "value": 67127.2119897098
          },
          {
            "t": 12.173159,
            "value": 58498.31842121963
          },
          {
            "t": 13.18143,
            "value": 68433.98253049032
          },
          {
            "t": 14.189842,
            "value": 63466.122973546524
          },
          {
            "t": 15.203757,
            "value": 63121.662072264444
          },
          {
            "t": 16.217399,
            "value": 63138.662367976074
          },
          {
            "t": 17.226161,
            "value": 62452.78866571104
          },
          {
            "t": 18.23494,
            "value": 67408.22320845298
          },
          {
            "t": 19.24477,
            "value": 56445.1442321975
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.019445,
            "value": 10504282.850183453
          },
          {
            "t": 3.087806,
            "value": 8821536.956072949
          },
          {
            "t": 5.104667,
            "value": 8426777.551849136
          },
          {
            "t": 7.126235,
            "value": 8751181.261278374
          },
          {
            "t": 9.043043,
            "value": 9982335.737329977
          },
          {
            "t": 11.064025,
            "value": 8544279.958950648
          },
          {
            "t": 13.080881,
            "value": 8854232.528251892
          },
          {
            "t": 15.103076,
            "value": 9103579.031695757
          },
          {
            "t": 17.125353,
            "value": 8830071.745858753
          },
          {
            "t": 19.144231,
            "value": 9037223.646005355
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.019445,
            "value": 7955163.496416742
          },
          {
            "t": 3.087806,
            "value": 9388318.576882856
          },
          {
            "t": 5.104667,
            "value": 9365970.188327307
          },
          {
            "t": 7.126235,
            "value": 9609041.100769306
          },
          {
            "t": 9.043043,
            "value": 10424344.01358926
          },
          {
            "t": 11.064025,
            "value": 9353380.683252003
          },
          {
            "t": 13.080881,
            "value": 9369963.44805975
          },
          {
            "t": 15.103076,
            "value": 9884348.93766427
          },
          {
            "t": 17.125353,
            "value": 9605490.741377171
          },
          {
            "t": 19.144231,
            "value": 9909316.957240606
          }
        ],
        "ram_mib": [
          {
            "t": 1.019445,
            "value": 758.92578125
          },
          {
            "t": 3.087806,
            "value": 847.41015625
          },
          {
            "t": 5.104667,
            "value": 877.4375
          },
          {
            "t": 7.126235,
            "value": 876.9140625
          },
          {
            "t": 9.043043,
            "value": 877.9921875
          },
          {
            "t": 11.064025,
            "value": 877.57421875
          },
          {
            "t": 13.080881,
            "value": 878.0703125
          },
          {
            "t": 15.103076,
            "value": 877.71875
          },
          {
            "t": 17.125353,
            "value": 877.375
          },
          {
            "t": 19.144231,
            "value": 876.82421875
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
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-3 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 2.9629628658294678
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 100.12722363071387
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-3 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.47133250311333
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 867.15546875
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-3 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 887.54296875
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 61971.91891685939
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-3 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 61718.03214017562
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-3 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000579
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9784610.424010815
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-3 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 8820575.968135668
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-3 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 158.53730400521763
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.101468,
            "value": 100.16538485509506
          },
          {
            "t": 3.024074,
            "value": 99.8
          },
          {
            "t": 5.042486,
            "value": 100.0071686372122
          },
          {
            "t": 7.060581,
            "value": 100.31666770282872
          },
          {
            "t": 9.083709,
            "value": 100.16498597693986
          },
          {
            "t": 11.102336,
            "value": 100.10130177514793
          },
          {
            "t": 13.129379,
            "value": 99.97725660964231
          },
          {
            "t": 15.148491,
            "value": 100.47133250311333
          },
          {
            "t": 17.070903,
            "value": 100.19501246882794
          },
          {
            "t": 19.094206,
            "value": 100.07312577833125
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.19317,
            "value": 251687.24589744836
          },
          {
            "t": 1.20272,
            "value": 68347.28344311823
          },
          {
            "t": 2.216594,
            "value": 72001.05733059531
          },
          {
            "t": 3.326349,
            "value": 56769.28691467937
          },
          {
            "t": 4.33596,
            "value": 16838.168363854988
          },
          {
            "t": 5.344739,
            "value": 70382.1154088259
          },
          {
            "t": 6.354167,
            "value": 67364.88387482813
          },
          {
            "t": 7.367838,
            "value": 69055.93629491226
          },
          {
            "t": 8.477881,
            "value": 58556.29016173248
          },
          {
            "t": 9.486557,
            "value": 68406.50516122126
          },
          {
            "t": 10.496275,
            "value": 75268.54032512047
          },
          {
            "t": 11.507381,
            "value": 70220.1351787053
          },
          {
            "t": 12.522676,
            "value": 62050.931010199005
          },
          {
            "t": 13.633015,
            "value": 56739.4282286761
          },
          {
            "t": 14.642351,
            "value": 61426.52199069487
          },
          {
            "t": 15.65211,
            "value": 59420.11905811188
          },
          {
            "t": 16.666228,
            "value": 61136.86967394326
          },
          {
            "t": 17.680552,
            "value": 61124.453330494005
          },
          {
            "t": 18.789979,
            "value": 62194.267851782955
          },
          {
            "t": 19.798824,
            "value": 61456.41798294089
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.091962,
            "value": 46579.61893907485
          },
          {
            "t": 1.101468,
            "value": 62406.76132682718
          },
          {
            "t": 2.110373,
            "value": 59470.41594600087
          },
          {
            "t": 3.124727,
            "value": 59150.94730242105
          },
          {
            "t": 4.134312,
            "value": 57449.34799942551
          },
          {
            "t": 5.143067,
            "value": 60470.580071474236
          },
          {
            "t": 6.152493,
            "value": 71327.66542569739
          },
          {
            "t": 7.161158,
            "value": 55518.92848468025
          },
          {
            "t": 8.17555,
            "value": 60134.54364782057
          },
          {
            "t": 9.184344,
            "value": 65424.65557883969
          },
          {
            "t": 10.194152,
            "value": 54465.79943910129
          },
          {
            "t": 11.202991,
            "value": 67404.21415111826
          },
          {
            "t": 12.114817,
            "value": 62511.92661757835
          },
          {
            "t": 13.129379,
            "value": 60124.467504203785
          },
          {
            "t": 14.138824,
            "value": 64391.81926702298
          },
          {
            "t": 15.148491,
            "value": 60415.95892507133
          },
          {
            "t": 16.157603,
            "value": 63422.09784444145
          },
          {
            "t": 17.171561,
            "value": 63118.98520451538
          },
          {
            "t": 18.186123,
            "value": 62095.76152073506
          },
          {
            "t": 19.194969,
            "value": 63438.8201965414
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.101468,
            "value": 10026447.45114145
          },
          {
            "t": 3.024074,
            "value": 8576704.223330209
          },
          {
            "t": 5.042486,
            "value": 8382868.809737556
          },
          {
            "t": 7.060581,
            "value": 9102126.014880368
          },
          {
            "t": 9.083709,
            "value": 8476392.49716281
          },
          {
            "t": 11.102336,
            "value": 8743922.973387357
          },
          {
            "t": 13.129379,
            "value": 8685540.464607805
          },
          {
            "t": 15.148491,
            "value": 8726122.176481543
          },
          {
            "t": 17.070903,
            "value": 8656390.513583977
          },
          {
            "t": 19.094206,
            "value": 8829244.557043606
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.101468,
            "value": 8853647.033411922
          },
          {
            "t": 3.024074,
            "value": 10370297.918554295
          },
          {
            "t": 5.042486,
            "value": 9108848.936688842
          },
          {
            "t": 7.060581,
            "value": 10161604.87984956
          },
          {
            "t": 9.083709,
            "value": 9872607.664962376
          },
          {
            "t": 11.102336,
            "value": 9901138.248918695
          },
          {
            "t": 13.129379,
            "value": 9726116.318203412
          },
          {
            "t": 15.148491,
            "value": 9875686.44037577
          },
          {
            "t": 17.070903,
            "value": 10118434.549929984
          },
          {
            "t": 19.094206,
            "value": 9857722.249213291
          }
        ],
        "ram_mib": [
          {
            "t": 1.101468,
            "value": 810.01171875
          },
          {
            "t": 3.024074,
            "value": 853.15625
          },
          {
            "t": 5.042486,
            "value": 864.453125
          },
          {
            "t": 7.060581,
            "value": 865.046875
          },
          {
            "t": 9.083709,
            "value": 867.6171875
          },
          {
            "t": 11.102336,
            "value": 870.62890625
          },
          {
            "t": 13.129379,
            "value": 884.66796875
          },
          {
            "t": 15.148491,
            "value": 884.359375
          },
          {
            "t": 17.070903,
            "value": 884.0703125
          },
          {
            "t": 19.094206,
            "value": 887.54296875
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
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-4 - Dropped Logs %",
          "name": "dropped_logs_percentage",
          "unit": "%",
          "value": 1.908065915107727
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_avg",
          "unit": "%",
          "value": 99.99564034155031
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-4 - CPU % (Normalized)",
          "name": "cpu_percentage_normalized_max",
          "unit": "%",
          "value": 100.17276410096602
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_avg",
          "unit": "MiB",
          "value": 864.34609375
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-4 - RAM (MiB)",
          "name": "ram_mib_max",
          "unit": "MiB",
          "value": 876.625
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_produced_rate",
          "unit": "logs/sec",
          "value": 58738.21949733941
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-4 - Log Throughput",
          "name": "logs_received_rate",
          "unit": "logs/sec",
          "value": 59117.95385600644
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-4 - Test Duration",
          "name": "test_duration",
          "unit": "seconds",
          "value": 20.000587
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_tx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 9754415.117855709
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-4 - Network Utilization",
          "name": "network_rx_bytes_rate_avg",
          "unit": "bytes/sec",
          "value": 8404395.581043022
        },
        {
          "extra": "OTC OTAP Transform Insert Multi Transform (Logs)/transform-4 - Egress Bytes Per Log",
          "name": "egress_bytes_per_log",
          "unit": "bytes/log",
          "value": 164.99920043942203
        }
      ],
      "timeseries": {
        "cpu_percentage_normalized": [
          {
            "t": 1.059425,
            "value": 100.11076923076922
          },
          {
            "t": 3.084873,
            "value": 100.17276410096602
          },
          {
            "t": 5.104256,
            "value": 100.17146774696167
          },
          {
            "t": 7.034181,
            "value": 99.71203986297104
          },
          {
            "t": 9.054093,
            "value": 100.1393644859813
          },
          {
            "t": 11.079383,
            "value": 99.70964808470882
          },
          {
            "t": 13.105352,
            "value": 99.81755610972569
          },
          {
            "t": 15.128637,
            "value": 100.10897539707256
          },
          {
            "t": 17.15544,
            "value": 99.97383084577113
          },
          {
            "t": 19.074567,
            "value": 100.03998755057579
          }
        ],
        "logs_produced_rate": [
          {
            "t": 0.150426,
            "value": 239548.1117102552
          },
          {
            "t": 1.266911,
            "value": 20600.366328253403
          },
          {
            "t": 2.276064,
            "value": 70356.03124600532
          },
          {
            "t": 3.286758,
            "value": 66291.08315672202
          },
          {
            "t": 4.296358,
            "value": 64381.93343898573
          },
          {
            "t": 5.311191,
            "value": 64049.94713415901
          },
          {
            "t": 6.428021,
            "value": 56409.65948264284
          },
          {
            "t": 7.437215,
            "value": 65398.7241303456
          },
          {
            "t": 8.44757,
            "value": 63344.07213306215
          },
          {
            "t": 9.4573,
            "value": 64373.64443960266
          },
          {
            "t": 10.472936,
            "value": 63999.30683827671
          },
          {
            "t": 11.589202,
            "value": 57334.00461897075
          },
          {
            "t": 12.598466,
            "value": 60440.08307043549
          },
          {
            "t": 13.609276,
            "value": 61336.94759648203
          },
          {
            "t": 14.619072,
            "value": 57437.343780327916
          },
          {
            "t": 15.634534,
            "value": 57116.859124221286
          },
          {
            "t": 16.751008,
            "value": 54636.29247076063
          },
          {
            "t": 17.760005,
            "value": 57482.82700543213
          },
          {
            "t": 18.769965,
            "value": 58418.15517446236
          },
          {
            "t": 19.779894,
            "value": 57429.77971718804
          }
        ],
        "logs_received_rate": [
          {
            "t": 0.044121,
            "value": 36660.232960918205
          },
          {
            "t": 1.059425,
            "value": 58110.67424140947
          },
          {
            "t": 2.074249,
            "value": 51240.41213057634
          },
          {
            "t": 3.084873,
            "value": 59369.26097143943
          },
          {
            "t": 4.094446,
            "value": 63393.13749476263
          },
          {
            "t": 5.104256,
            "value": 62387.97397530228
          },
          {
            "t": 6.119832,
            "value": 56125.78477632398
          },
          {
            "t": 7.134886,
            "value": 55169.478668129974
          },
          {
            "t": 8.145145,
            "value": 55431.32998567694
          },
          {
            "t": 9.154819,
            "value": 55463.446617423055
          },
          {
            "t": 10.165263,
            "value": 63338.492781391156
          },
          {
            "t": 11.180143,
            "value": 63061.642755793786
          },
          {
            "t": 12.195308,
            "value": 57133.569419749496
          },
          {
            "t": 13.206121,
            "value": 54411.646862476045
          },
          {
            "t": 14.216069,
            "value": 63369.59922689089
          },
          {
            "t": 15.128637,
            "value": 69035.95129349265
          },
          {
            "t": 16.140654,
            "value": 63240.044386606154
          },
          {
            "t": 17.15544,
            "value": 55184.048656564046
          },
          {
            "t": 18.165477,
            "value": 55443.51345544767
          },
          {
            "t": 19.175365,
            "value": 63373.36417503724
          }
        ],
        "network_rx_bytes_rate": [
          {
            "t": 1.059425,
            "value": 8442871.876129868
          },
          {
            "t": 3.084873,
            "value": 9132383.057970386
          },
          {
            "t": 5.104256,
            "value": 8242235.8710556645
          },
          {
            "t": 7.034181,
            "value": 8573737.83955335
          },
          {
            "t": 9.054093,
            "value": 8614645.588520687
          },
          {
            "t": 11.079383,
            "value": 8180746.461000646
          },
          {
            "t": 13.105352,
            "value": 8655212.888252486
          },
          {
            "t": 15.128637,
            "value": 7765475.946295258
          },
          {
            "t": 17.15544,
            "value": 7759429.505482279
          },
          {
            "t": 19.074567,
            "value": 8677216.776169581
          }
        ],
        "network_tx_bytes_rate": [
          {
            "t": 1.059425,
            "value": 7530430.135357004
          },
          {
            "t": 3.084873,
            "value": 10370778.711672677
          },
          {
            "t": 5.104256,
            "value": 9862840.778594254
          },
          {
            "t": 7.034181,
            "value": 10336297.524515202
          },
          {
            "t": 9.054093,
            "value": 10416570.62287862
          },
          {
            "t": 11.079383,
            "value": 9335702.541364446
          },
          {
            "t": 13.105352,
            "value": 10367434.052544734
          },
          {
            "t": 15.128637,
            "value": 9591069.473652994
          },
          {
            "t": 17.15544,
            "value": 9328944.648295863
          },
          {
            "t": 19.074567,
            "value": 10404082.689681297
          }
        ],
        "ram_mib": [
          {
            "t": 1.059425,
            "value": 807.70703125
          },
          {
            "t": 3.084873,
            "value": 837.45703125
          },
          {
            "t": 5.104256,
            "value": 866.6796875
          },
          {
            "t": 7.034181,
            "value": 875.80078125
          },
          {
            "t": 9.054093,
            "value": 876.3984375
          },
          {
            "t": 11.079383,
            "value": 875.76953125
          },
          {
            "t": 13.105352,
            "value": 876.625
          },
          {
            "t": 15.128637,
            "value": 876.04296875
          },
          {
            "t": 17.15544,
            "value": 876.41015625
          },
          {
            "t": 19.074567,
            "value": 874.5703125
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
