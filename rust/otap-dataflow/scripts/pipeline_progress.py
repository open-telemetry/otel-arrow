#!/usr/bin/env python3
"""
Pipeline progress tracker.
Polls the df_engine metrics endpoint and shows aggregated receiver + exporter
progress across all cores.  Metrics are delta-temporality, so this script
accumulates them into running totals.

Usage:
    python3 pipeline_progress.py [interval_secs] [url]

    interval_secs: polling interval in seconds (default: 5)
    url:           metrics endpoint (default: http://127.0.0.1:8080/metrics?reset=false)

Press Ctrl+C to stop.
"""
import json, sys, time, urllib.request
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


def parse(data):
    syslog_pdata_send = {}   # core -> count
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

        # Syslog receiver pdata send
        if 'syslog' in nid and 'fanout' not in nid:
            if name == 'channel.sender' and ch_kind == 'pdata':
                v = metric_val(metrics, 'send.count')
                if v:
                    syslog_pdata_send[core] = syslog_pdata_send.get(core, 0) + v

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

    return {
        'syslog_send': syslog_pdata_send,
        'fanout_recv': fanout_pdata_recv,
        'exp_recv': exp_pdata_recv,
        'exp_rows': exp_rows,
        'exp_batches': exp_batches,
        'exp_msgs': exp_msgs,
        'exp_http2xx': exp_http2xx,
        'exp_failed': exp_failed_rows,
        'exp_inflight': exp_inflight,
    }


def sum_dict(d):
    return sum(d.values())


def per_core_str(d, cores):
    return " ".join(f"c{c}={d.get(c,0):>10,}" for c in cores)


def main():
    prev = None
    cumulative = {}  # Accumulate delta metrics over time

    print(f"Polling {URL} every {INTERVAL}s  (Ctrl+C to stop)\n", flush=True)

    try:
        while True:
            data = fetch()
            if data is None:
                time.sleep(INTERVAL)
                continue

            cur = parse(data)
            now = datetime.now().strftime("%H:%M:%S")

            # Accumulate deltas into cumulative totals
            for key in cur:
                if key not in cumulative:
                    cumulative[key] = {}
                for core, val in cur[key].items():
                    cumulative[key][core] = cumulative[key].get(core, 0) + val

            cores = sorted(set(
                list(cumulative.get('syslog_send', {}).keys()) +
                list(cumulative.get('exp_rows', {}).keys())
            ))

            tot_syslog   = sum_dict(cumulative.get('syslog_send', {}))
            tot_exp_recv = sum_dict(cumulative.get('exp_recv', {}))
            tot_rows     = sum_dict(cumulative.get('exp_rows', {}))
            tot_batches  = sum_dict(cumulative.get('exp_batches', {}))
            tot_msgs     = sum_dict(cumulative.get('exp_msgs', {}))
            tot_http2xx  = sum_dict(cumulative.get('exp_http2xx', {}))
            tot_failed   = sum_dict(cumulative.get('exp_failed', {}))

            delta_syslog = ""
            delta_rows = ""
            if prev:
                ds = tot_syslog - prev['tot_syslog']
                dr = tot_rows - prev['tot_rows']
                delta_syslog = f" (+{ds:,})"
                delta_rows   = f" (+{dr:,})"

            delta_msgs = ""
            if prev and 'tot_msgs' in prev:
                dm = tot_msgs - prev['tot_msgs']
                delta_msgs = f" (+{dm:,})"

            print(f"[{now}] ─────────────────────────────────────────────────────────")
            print(f"  syslog sent (msgs):   {tot_syslog:>12,}{delta_syslog}")
            print(f"  exporter recv (msgs): {tot_exp_recv:>12,}")
            print(f"  exported (msgs):      {tot_msgs:>12,}{delta_msgs}")
            print(f"  exported (rows):      {tot_rows:>12,}{delta_rows}")
            print(f"  batches:              {tot_batches:>12,}    http.2xx: {tot_http2xx:,}    failed: {tot_failed:,}")
            if len(cores) > 1 or (len(cores) == 1 and cores[0] != 0):
                print(f"  per-core syslog: {per_core_str(cumulative['syslog_send'], cores)}")
                print(f"  per-core recv:   {per_core_str(cumulative['exp_recv'], cores)}")
                print(f"  per-core msgs:   {per_core_str(cumulative['exp_msgs'], cores)}")
                print(f"  per-core rows:   {per_core_str(cumulative['exp_rows'], cores)}")
                print(f"  per-core batches:{per_core_str(cumulative['exp_batches'], cores)}")
                print(f"  per-core http2xx:{per_core_str(cumulative['exp_http2xx'], cores)}")
                print(f"  per-core failed: {per_core_str(cumulative['exp_failed'], cores)}")
            print(flush=True)

            prev = {
                'tot_syslog': tot_syslog,
                'tot_rows': tot_rows,
                'tot_msgs': tot_msgs,
            }
            time.sleep(INTERVAL)
    except KeyboardInterrupt:
        print("\nStopped.")


if __name__ == '__main__':
    main()
