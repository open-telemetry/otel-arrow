// The shipped dashboard and deployment probe share this JSON-compatible catalog.
export const telemetryInputs = {
  "sets": {
    "engine": {
      "name": "engine",
      "consumer": "Engine cards",
      "metrics": {
        "cpuUtilization": "cpu.utilization",
        "memoryRss": "memory.rss",
        "memoryCommit": "memory.commit"
      },
      "platform": {
        "memoryCommit": "windows"
      }
    },
    "pipeline": {
      "name": "pipeline",
      "consumer": "Pipeline charts and selectors",
      "metrics": {
        "uptime": "uptime",
        "cpuUtilization": "cpu.utilization",
        "cpuTime": "cpu.time",
        "memoryUsage": "memory.usage",
        "memoryAllocatedDelta": "memory.allocated.delta",
        "memoryFreedDelta": "memory.freed.delta",
        "contextSwitchesVoluntary": "context.switches.voluntary",
        "contextSwitchesInvoluntary": "context.switches.involuntary",
        "pageFaultsMinor": "page.faults.minor",
        "pageFaultsMajor": "page.faults.major"
      },
      "dimensions": [
        "group",
        "pipeline",
        "core"
      ],
      "platform": {
        "memoryUsage": "non-windows",
        "memoryAllocatedDelta": "non-windows",
        "memoryFreedDelta": "non-windows",
        "contextSwitchesVoluntary": "unix-rusage",
        "contextSwitchesInvoluntary": "unix-rusage",
        "pageFaultsMinor": "unix-rusage",
        "pageFaultsMajor": "unix-rusage"
      }
    },
    "tokio": {
      "name": "tokio.runtime",
      "consumer": "Runtime charts",
      "metrics": {
        "workerCount": "worker.count",
        "taskActiveCount": "task.active.count",
        "globalTaskQueueSize": "global.task.queue.size",
        "workerBusyTime": "worker.busy.time",
        "workerParkCount": "worker.park.count",
        "workerParkUnparkCount": "worker.park.unpark.count"
      },
      "dimensions": [
        "group",
        "pipeline",
        "core"
      ],
      "platform": {
        "workerBusyTime": "atomic64",
        "workerParkCount": "atomic64",
        "workerParkUnparkCount": "atomic64"
      }
    },
    "control": {
      "name": "pipeline.runtime_control",
      "consumer": "Runtime control charts",
      "metrics": {
        "drainActive": "drain.active",
        "drainPendingReceivers": "drain.pending_receivers",
        "pendingSendsBuffered": "pending_sends.buffered",
        "timersActive": "timers.active",
        "telemetryTimersActive": "telemetry_timers.active",
        "shutdownDeadlineForced": "shutdown.deadline_forced",
        "timerTickSent": "timer_tick.sent",
        "collectTelemetrySent": "collect_telemetry.sent"
      },
      "dimensions": [
        "group",
        "pipeline",
        "core"
      ]
    },
    "completion": {
      "name": "pipeline.completion",
      "consumer": "Completion charts",
      "metrics": {
        "pendingSendsBuffered": "pending_sends.buffered",
        "deliverAckReceived": "deliver_ack.received",
        "deliverNackReceived": "deliver_nack.received",
        "ackAttempted": "ack.attempted",
        "nackAttempted": "nack.attempted",
        "ackDelivered": "ack.delivered",
        "nackDelivered": "nack.delivered",
        "ackDroppedNoInterest": "ack.dropped_no_interest",
        "nackDroppedNoInterest": "nack.dropped_no_interest"
      },
      "dimensions": [
        "group",
        "pipeline",
        "core"
      ]
    },
    "sender": {
      "name": "channel.sender",
      "consumer": "Channel charts and DAG",
      "metrics": {
        "messages": "messages",
        "failures": "failures"
      },
      "dimensions": [
        "group",
        "pipeline",
        "core",
        "node",
        "channel",
        "kind",
        "port",
        "nodeType",
        "nodeUrn",
        "channelType",
        "mode",
        "implementation"
      ],
      "legacy": {
        "count": "send.count",
        "full": "send.error_full",
        "closed": "send.error_closed",
        "capacity": "capacity"
      },
      "series": {
        "success": {
          "metric": "messages",
          "attributes": {
            "outcome": "success"
          },
          "traffic": true
        },
        "full": {
          "metric": "failures",
          "attributes": {
            "error.type": "full"
          }
        },
        "closed": {
          "metric": "failures",
          "attributes": {
            "error.type": "closed"
          }
        }
      }
    },
    "receiver": {
      "name": "channel.receiver",
      "consumer": "Channel details and DAG",
      "metrics": {
        "messages": "messages",
        "capacity": "capacity",
        "depth": "queue.depth"
      },
      "dimensions": [
        "group",
        "pipeline",
        "core",
        "node",
        "channel",
        "kind",
        "port",
        "nodeType",
        "nodeUrn",
        "channelType",
        "mode",
        "implementation"
      ],
      "legacy": {
        "count": "recv.count",
        "empty": "recv.error_empty",
        "closed": "recv.error_closed"
      },
      "series": {
        "received": {
          "metric": "messages",
          "attributes": {},
          "traffic": true
        }
      }
    },
    "publisher": {
      "name": "exporter.topic",
      "consumer": "Topic topology",
      "metrics": {},
      "dimensions": [
        "group",
        "pipeline",
        "core",
        "node",
        "topic"
      ],
      "fallbackDimensions": { "process": "" }
    },
    "subscriber": {
      "name": "receiver.topic",
      "consumer": "Topic topology",
      "metrics": {},
      "dimensions": [
        "group",
        "pipeline",
        "core",
        "node",
        "topic"
      ],
      "fallbackDimensions": { "process": "" }
    }
  },
  "dimensions": {
    "group": "pipeline.group.id",
    "pipeline": "pipeline.id",
    "core": "core.id",
    "node": "node.id",
    "channel": "channel.id",
    "kind": "channel.kind",
    "process": "process.instance.id",
    "topic": "topic",
    "port": "node.port",
    "nodeType": "node.type",
    "nodeUrn": "node.urn",
    "channelType": "channel.type",
    "mode": "channel.mode",
    "implementation": "channel.impl",
    "numa": "numa.node.id",
    "thread": "thread.id"
  },
  "legacyDimensions": {
    "group": ["pipeline_group_id", "otelcol.pipeline_group.id"],
    "pipeline": ["pipeline_id", "otelcol.pipeline.id"]
  },
  "events": {
    "kinds": {
      "request": "Request",
      "success": "Success",
      "error": "Error"
    },
    "conditions": {
      "ready": "Ready",
      "accepted": "Accepted"
    },
    "logFields": {
      "timestamp": "timestamp",
      "level": "level",
      "target": "target",
      "rendered": "rendered",
      "name": "event_name"
    },
    "namedLogs": []
  }
};

export const { sets, dimensions, legacyDimensions, events } = telemetryInputs;
