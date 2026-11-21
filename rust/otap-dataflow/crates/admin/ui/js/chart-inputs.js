// Numeric instruments consumed by the dashboard cards. The native UI guarantee
// reads the same JSON object. memory.commit is supplied by Windows.
export const chartInputs = {
  "engine": {
    "cpuUtilization": "cpu.utilization",
    "memoryRss": "memory.rss",
    "memoryCommit": "memory.commit"
  },
  "pipeline": {
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
  "tokio.runtime": {
    "workerCount": "worker.count",
    "taskActiveCount": "task.active.count",
    "globalTaskQueueSize": "global.task.queue.size",
    "workerBusyTime": "worker.busy.time",
    "workerParkCount": "worker.park.count",
    "workerParkUnparkCount": "worker.park.unpark.count"
  },
  "pipeline.runtime_control": {
    "drainActive": "drain.active",
    "drainPendingReceivers": "drain.pending_receivers",
    "pendingSendsBuffered": "pending_sends.buffered",
    "timersActive": "timers.active",
    "telemetryTimersActive": "telemetry_timers.active",
    "shutdownDeadlineForced": "shutdown.deadline_forced",
    "timerTickSent": "timer_tick.sent",
    "collectTelemetrySent": "collect_telemetry.sent"
  },
  "pipeline.completion": {
    "pendingSendsBuffered": "pending_sends.buffered",
    "deliverAckReceived": "deliver_ack.received",
    "deliverNackReceived": "deliver_nack.received",
    "ackAttempted": "ack.attempted",
    "nackAttempted": "nack.attempted",
    "ackDelivered": "ack.delivered",
    "nackDelivered": "nack.delivered",
    "ackDroppedNoInterest": "ack.dropped_no_interest",
    "nackDroppedNoInterest": "nack.dropped_no_interest"
  }
};
