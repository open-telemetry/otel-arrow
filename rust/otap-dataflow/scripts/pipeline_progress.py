#!/usr/bin/env python3
"""
Pipeline progress tracker.
Polls the df_engine metrics endpoint and shows cumulative receiver + exporter
progress across all cores.  The endpoint is queried with reset=false, so
values are cumulative (like Prometheus counters) — we display them as-is and
optionally show the delta from the previous poll.

Usage:
    python3 pipeline_progress.py [interval_secs] [url]

    interval_secs: polling interval in seconds (default: 5)
    url:           metrics endpoint (default: http://127.0.0.1:8080/metrics?reset=false)

Press Ctrl+C to stop.
"""
import json, subprocess, sys, time, urllib.request
from datetime import datetime

INTERVAL = int(sys.argv[1]) if len(sys.argv) > 1 else 5
URL = sys.argv[2] if len(sys.argv) > 2 else "http://127.0.0.1:8080/metrics?reset=false"


def attr_val(attrs, key):
    """Extract a scalar value from the typed-value attribute map."""
    v = attrs.get(key, {})
    if isinstance(v, dict):
        for _, val in v.items():
            return val
    return v


def metric_val(metrics, key):
    """Look up a metric value by name.  Handles both list and dict formats."""
    if isinstance(metrics, list):
        for m in metrics:
            if m.get('name') == key:
                return m.get('value', 0)
        return 0
    m = metrics.get(key, {})
    if isinstance(m, dict):
        return m.get('value', 0)
    return 0


def fetch():
    try:
        with urllib.request.urlopen(URL, timeout=5) as r:
            return json.loads(r.read())
    except Exception as e:
        print(f"  [error] {e}", flush=True)
        return None


def find_engine_pids():
    """Find PIDs of df_engine processes."""
    try:
        out = subprocess.check_output(
            ['pgrep', '-f', 'df_engine'], text=True, stderr=subprocess.DEVNULL
        ).strip()
        return [int(p) for p in out.splitlines() if p.strip()]
    except (subprocess.CalledProcessError, FileNotFoundError):
        return []


def get_os_rss_mb(pids):
    """Get RSS in MB for given PIDs using ps."""
    if not pids:
        return None
    try:
        out = subprocess.check_output(
            ['ps', '-o', 'rss=', '-p', ','.join(str(p) for p in pids)],
            text=True, stderr=subprocess.DEVNULL
        ).strip()
        total_kb = sum(int(x) for x in out.split() if x.strip())
        return total_kb / 1024.0 if total_kb > 0 else None
    except (subprocess.CalledProcessError, FileNotFoundError, ValueError):
        return None




def parse(data):
    syslog_pdata_send = {}   # core -> count (batch messages)
    syslog_logs_forwarded = {}  # core -> count (individual log records forwarded)
    syslog_logs_total = {}      # core -> count (individual log records received)
    syslog_logs_invalid = {}    # core -> count (individual log records failed to parse)
    syslog_logs_forward_failed = {}  # core -> count (individual log records refused)
    fanout_pdata_recv = {}
    fanout_ctrl_send = {}
    fanout_ctrl_full = {}
    exp_pdata_recv = {}
    exp_ctrl_recv = {}
    exp_rows = {}
    exp_batches = {}
    exp_msgs = {}
    exp_http2xx = {}
    exp_failed_rows = {}
    exp_inflight = {}
    mem_usage = {}       # core -> bytes  (cumulative / gauge)
    mem_rss = {}         # core -> bytes  (process-wide RSS from engine)
    cpu_util = {}        # core -> ratio  (gauge, 0..1)
    cpu_time = {}        # core -> seconds (delta)
    uptime = {}          # core -> seconds (gauge)

    for ms in data.get('metric_sets', []):
        name = ms.get('name', '')
        attrs = ms.get('attributes', {})
        metrics = ms.get('metrics', {})
        nid = str(attr_val(attrs, 'node.id') or '')
        core = attr_val(attrs, 'core.id')
        if core is None:
            continue
        core = int(core)
        ch_kind = str(attr_val(attrs, 'channel.kind') or '')

        # Syslog receiver pdata send (batch messages)
        if 'syslog' in nid and 'fanout' not in nid:
            if name == 'channel.sender' and ch_kind == 'pdata':
                v = metric_val(metrics, 'send.count')
                if v:
                    syslog_pdata_send[core] = syslog_pdata_send.get(core, 0) + v
            # Syslog receiver's own metrics (individual log records)
            # Note: the proc macro converts field underscores to dots,
            # e.g. received_logs_forwarded -> received.logs.forwarded
            if name == 'syslog_cef.receiver.metrics':
                v = metric_val(metrics, 'received.logs.forwarded')
                if v:
                    syslog_logs_forwarded[core] = syslog_logs_forwarded.get(core, 0) + v
                v = metric_val(metrics, 'received.logs.total')
                if v:
                    syslog_logs_total[core] = syslog_logs_total.get(core, 0) + v
                v = metric_val(metrics, 'received.logs.invalid')
                if v:
                    syslog_logs_invalid[core] = syslog_logs_invalid.get(core, 0) + v
                v = metric_val(metrics, 'received.logs.forward.failed')
                if v:
                    syslog_logs_forward_failed[core] = syslog_logs_forward_failed.get(core, 0) + v

        # Fanout
        if 'fanout' in nid:
            if name == 'channel.receiver' and ch_kind == 'pdata':
                v = metric_val(metrics, 'recv.count')
                if v:
                    fanout_pdata_recv[core] = fanout_pdata_recv.get(core, 0) + v
            if name == 'channel.sender' and ch_kind == 'control':
                s = metric_val(metrics, 'send.count')
                f = metric_val(metrics, 'send.error_full')
                if s:
                    fanout_ctrl_send[core] = fanout_ctrl_send.get(core, 0) + s
                    fanout_ctrl_full[core] = fanout_ctrl_full.get(core, 0) + f

        # Exporter
        if 'exporter' in nid:
            if name == 'channel.receiver' and ch_kind == 'pdata':
                v = metric_val(metrics, 'recv.count')
                if v:
                    exp_pdata_recv[core] = exp_pdata_recv.get(core, 0) + v
            if name == 'channel.receiver' and ch_kind == 'control':
                v = metric_val(metrics, 'recv.count')
                if v:
                    exp_ctrl_recv[core] = exp_ctrl_recv.get(core, 0) + v
            if name == 'azure_monitor_exporter.metrics':
                exp_rows[core] = exp_rows.get(core, 0) + metric_val(metrics, 'successful.rows')
                exp_batches[core] = exp_batches.get(core, 0) + metric_val(metrics, 'successful.batches')
                exp_msgs[core] = exp_msgs.get(core, 0) + metric_val(metrics, 'successful.messages')
                exp_http2xx[core] = exp_http2xx.get(core, 0) + metric_val(metrics, 'laclient.http.2xx')
                exp_failed_rows[core] = exp_failed_rows.get(core, 0) + metric_val(metrics, 'failed.rows')
                exp_inflight[core] = exp_inflight.get(core, 0) + metric_val(metrics, 'in.flight.exports')

        # Pipeline-level metrics (memory, CPU, uptime)
        if name == 'pipeline.metrics':
            mem_usage[core] = metric_val(metrics, 'memory.usage')
            mem_rss[core] = metric_val(metrics, 'memory.rss')
            cpu_util[core] = metric_val(metrics, 'cpu.utilization')
            cpu_time[core] = metric_val(metrics, 'cpu.time')
            uptime[core] = metric_val(metrics, 'uptime')

    return {
        'syslog_send': syslog_pdata_send,
        'syslog_logs_forwarded': syslog_logs_forwarded,
        'syslog_logs_total': syslog_logs_total,
        'syslog_logs_invalid': syslog_logs_invalid,
        'syslog_logs_forward_failed': syslog_logs_forward_failed,
        'fanout_recv': fanout_pdata_recv,
        'exp_recv': exp_pdata_recv,
        'exp_rows': exp_rows,
        'exp_batches': exp_batches,
        'exp_msgs': exp_msgs,
        'exp_http2xx': exp_http2xx,
        'exp_failed': exp_failed_rows,
        'exp_inflight': exp_inflight,
        'mem_usage': mem_usage,
        'mem_rss': mem_rss,
        'cpu_util': cpu_util,
        'cpu_time': cpu_time,
        'uptime': uptime,
    }


def sum_dict(d):
    return sum(d.values())


def per_core_str(d, cores):
    return " ".join(f"c{c}={d.get(c,0):>10,}" for c in cores)


def main():
    prev = None
    engine_pids = find_engine_pids()

    print(f"Polling {URL} every {INTERVAL}s  (Ctrl+C to stop)", flush=True)
    print(flush=True)

    try:
        while True:
            data = fetch()
            if data is None:
                time.sleep(INTERVAL)
                continue

            cur = parse(data)
            now = datetime.now().strftime("%H:%M:%S")

            # Values from the endpoint are already cumulative (reset=false),
            # so just use them directly.
            cores = sorted(set(
                list(cur.get('syslog_send', {}).keys()) +
                list(cur.get('exp_rows', {}).keys())
            ))

            tot_syslog   = sum_dict(cur.get('syslog_send', {}))
            tot_syslog_logs = sum_dict(cur.get('syslog_logs_forwarded', {}))
            tot_syslog_total = sum_dict(cur.get('syslog_logs_total', {}))
            tot_syslog_invalid = sum_dict(cur.get('syslog_logs_invalid', {}))
            tot_syslog_refused = sum_dict(cur.get('syslog_logs_forward_failed', {}))
            tot_exp_recv = sum_dict(cur.get('exp_recv', {}))
            tot_rows     = sum_dict(cur.get('exp_rows', {}))
            tot_batches  = sum_dict(cur.get('exp_batches', {}))
            tot_msgs     = sum_dict(cur.get('exp_msgs', {}))
            tot_http2xx  = sum_dict(cur.get('exp_http2xx', {}))
            tot_failed   = sum_dict(cur.get('exp_failed', {}))

            # Compute deltas from previous poll
            delta_syslog_logs = ""
            delta_syslog = ""
            delta_rows = ""
            delta_msgs = ""
            if prev:
                ds = tot_syslog - prev.get('tot_syslog', 0)
                dsl = tot_syslog_logs - prev.get('tot_syslog_logs', 0)
                dr = tot_rows - prev.get('tot_rows', 0)
                dm = tot_msgs - prev.get('tot_msgs', 0)
                delta_syslog = f" (+{ds:,})"
                delta_syslog_logs = f" (+{dsl:,})"
                delta_rows   = f" (+{dr:,})"
                delta_msgs   = f" (+{dm:,})"

            print(f"[{now}] ─────────────────────────────────────────────────────────")
            print(f"  syslog recv (logs):   {tot_syslog_total:>12,}    invalid: {tot_syslog_invalid:,}    refused: {tot_syslog_refused:,}")
            print(f"  syslog fwd  (logs):   {tot_syslog_logs:>12,}{delta_syslog_logs}")
            print(f"  syslog sent (msgs):   {tot_syslog:>12,}{delta_syslog}")
            print(f"  exporter recv (msgs): {tot_exp_recv:>12,}")
            print(f"  exported (msgs):      {tot_msgs:>12,}{delta_msgs}")
            print(f"  exported (rows):      {tot_rows:>12,}{delta_rows}")
            print(f"  batches:              {tot_batches:>12,}    http.2xx: {tot_http2xx:,}    failed: {tot_failed:,}")

            # CPU and memory from pipeline.metrics (gauges — always use current values)
            mem = cur.get('mem_usage', {})
            rss = cur.get('mem_rss', {})
            cpu_u = cur.get('cpu_util', {})
            up = cur.get('uptime', {})
            if mem:
                alloc_mb = max(mem.values()) / (1024 * 1024)
                rss_mb = max(rss.values()) / (1024 * 1024) if rss and max(rss.values()) > 0 else 0
                total_cpu_pct = sum(cpu_u.values()) * 100  # ratio → %
                uptime_s = max(up.values()) if up else 0
                if not engine_pids:
                    engine_pids = find_engine_pids()
                os_rss = get_os_rss_mb(engine_pids)
                if os_rss is None:
                    engine_pids = []  # retry next poll
                rss_str = f"rss(engine)={rss_mb:.1f} MB  " if rss_mb > 0 else ""
                os_str = f"rss(os)={os_rss:.1f} MB  " if os_rss else ""
                print(f"  memory: {rss_str}{os_str}heap={alloc_mb:.1f} MB    cpu: {total_cpu_pct:.1f}%    uptime: {uptime_s:.0f}s")

            if len(cores) > 1 or (len(cores) == 1 and cores[0] != 0):
                print(f"  per-core syslog: {per_core_str(cur.get('syslog_logs_forwarded', {}), cores)}")
                print(f"  per-core recv:   {per_core_str(cur.get('exp_recv', {}), cores)}")
                print(f"  per-core msgs:   {per_core_str(cur.get('exp_msgs', {}), cores)}")
                print(f"  per-core rows:   {per_core_str(cur.get('exp_rows', {}), cores)}")
                print(f"  per-core batches:{per_core_str(cur.get('exp_batches', {}), cores)}")
                print(f"  per-core http2xx:{per_core_str(cur.get('exp_http2xx', {}), cores)}")
                print(f"  per-core failed: {per_core_str(cur.get('exp_failed', {}), cores)}")
                if cpu_u:
                    cpu_str = " ".join(f"c{c}={cpu_u.get(c,0)*100:>7.1f}%" for c in cores)
                    print(f"  per-core cpu:    {cpu_str}")
            print(flush=True)

            prev = {
                'tot_syslog': tot_syslog,
                'tot_syslog_logs': tot_syslog_logs,
                'tot_rows': tot_rows,
                'tot_msgs': tot_msgs,
            }
            time.sleep(INTERVAL)
    except KeyboardInterrupt:
        print("\nStopped.")


if __name__ == '__main__':
    main()
