#!/usr/bin/env python3
"""
Engine metrics console viewer.

Polls the otap-dataflow engine metrics endpoint and displays all reported
metrics with per-core breakdown and aggregated totals.  Metric sets,
metric names, and attributes are discovered dynamically from the JSON
response -- no hard-coded metric names -- so the script stays current
as the engine evolves.

Usage:
    python3 engine-metrics.py [-i SECS] [-u URL] [-k KINDS] [--per-core]
    python3 engine-metrics.py [INTERVAL] [URL]        (legacy positional)

Kinds (for -k / --kind):
    engine     engine-wide metrics (RSS, CPU)
    pipeline   per-pipeline metrics (memory, CPU, uptime, context switches)
    tokio      per-pipeline Tokio runtime stats
    channel    inter-node channel sender/receiver counters
    receiver   receiver component metrics (otap, syslog/cef, traffic_generator, ...)
    processor  processor component metrics (batch, retry, router, filter, ...)
    exporter   exporter component metrics (otap, parquet, geneva, ...)

    Combine with commas:  -k receiver,exporter  (default)
    Use 'all' to show everything.

Examples:
    python3 engine-metrics.py                          # receivers + exporters, 5s
    python3 engine-metrics.py -k all                   # all metric sets
    python3 engine-metrics.py -k pipeline,tokio        # runtime metrics only
    python3 engine-metrics.py -k exporter --per-core   # exporters with per-core
    python3 engine-metrics.py -i 10 -f syslog          # 10s, filter to syslog

Press Ctrl+C to stop.
"""

import argparse
import json
import os
import subprocess
import sys
import time
import urllib.request
from collections import OrderedDict
from datetime import datetime


# --- value helpers --------------------------------------------------


def attr_val(attrs, key):
    """Unwrap typed attribute value, e.g. ``{"UInt": 0}`` -> ``0``.

    Returns ``None`` when *key* is absent.
    """
    if key not in attrs:
        return None
    v = attrs[key]
    if isinstance(v, dict):
        for _, val in v.items():
            return val
    return v


def is_mmsc(v):
    """True when *v* is an MMSC snapshot dict (min/max/sum/count)."""
    return isinstance(v, dict) and "min" in v and "count" in v


def _fmt_num(n):
    """Number with commas; keep decimals for non-integer floats."""
    if isinstance(n, float) and n != int(n):
        return f"{n:,.2f}"
    return f"{int(n):,}"


def fmt(v, unit="", instrument=""):
    """Smart-format a value based on its unit and instrument type."""
    if v is None:
        return "-"
    if is_mmsc(v):
        c = v["count"]
        if c == 0:
            return "n=0"
        avg = v["sum"] / c
        return (
            f"min={v['min']:.1f} max={v['max']:.1f} "
            f"avg={avg:.1f} n={_fmt_num(c)}"
        )
    # Bytes -> human-readable
    if unit and "By" in unit:
        mb = v / (1024 * 1024)
        if mb >= 1024:
            return f"{mb / 1024:.1f} GB"
        if mb >= 1:
            return f"{mb:.1f} MB"
        return f"{v:,} B"
    # Ratio [0,1] -> percentage
    if instrument == "gauge" and unit in ("1", "{1}"):
        return f"{v * 100:.1f}%"
    # Seconds
    if unit in ("s", "{s}"):
        return f"{v:.0f}s" if v >= 10 else f"{v:.1f}s"
    return _fmt_num(v)


# --- OS-level stats (optional enrichment) --------------------------


def find_engine_pids():
    """Find PIDs of df_engine processes."""
    try:
        out = subprocess.check_output(
            ["pgrep", "-f", "df_engine"],
            text=True,
            stderr=subprocess.DEVNULL,
        ).strip()
        return [int(p) for p in out.splitlines() if p.strip()]
    except (subprocess.CalledProcessError, FileNotFoundError):
        return []


def get_os_rss_mb(pids):
    """Get RSS in MB for given PIDs via ``ps``."""
    if not pids:
        return None
    try:
        out = subprocess.check_output(
            ["ps", "-o", "rss=", "-p", ",".join(str(p) for p in pids)],
            text=True,
            stderr=subprocess.DEVNULL,
        ).strip()
        kb = sum(int(x) for x in out.split() if x.strip())
        return kb / 1024.0 if kb > 0 else None
    except (subprocess.CalledProcessError, FileNotFoundError, ValueError):
        return None


def get_os_cpu_pct(pids):
    """Get total CPU% for given PIDs via ``ps``."""
    if not pids:
        return None
    try:
        out = subprocess.check_output(
            ["ps", "-o", "%cpu=", "-p", ",".join(str(p) for p in pids)],
            text=True,
            stderr=subprocess.DEVNULL,
        ).strip()
        t = sum(float(x) for x in out.split() if x.strip())
        return t if t > 0 else None
    except (subprocess.CalledProcessError, FileNotFoundError, ValueError):
        return None


# --- fetch ----------------------------------------------------------


def fetch(url):
    """Fetch and parse JSON from the metrics endpoint."""
    try:
        with urllib.request.urlopen(url, timeout=5) as r:
            body = r.read()
            return json.loads(body) if body else None
    except urllib.error.URLError:
        return None
    except json.JSONDecodeError as e:
        print(f"  [warn] bad JSON: {e}", flush=True)
        return None
    except Exception as e:
        print(f"  [warn] {e}", flush=True)
        return None


# --- parse & group --------------------------------------------------


def _gkey(ms):
    """Group key = (metric_set name, non-core-id attributes as tuple).

    Metric-set instances that differ only in ``core.id`` are collapsed
    into the same group so we can show per-core breakdown.
    """
    name = ms.get("name", "")
    attrs = ms.get("attributes", {})
    extras = tuple(
        sorted((k, str(attr_val(attrs, k))) for k in attrs if k != "core.id")
    )
    return (name, extras)


def parse(data):
    """Parse the JSON response into groups keyed by :func:`_gkey`.

    Returns ``OrderedDict[key] -> {name, attrs, cores, meta}`` where
    *cores* maps ``core_id -> {metric_name: value}`` and *meta* maps
    ``metric_name -> {instrument, unit}``.
    """
    groups = OrderedDict()
    for ms in data.get("metric_sets", []):
        key = _gkey(ms)
        attrs = ms.get("attributes", {})
        core = attr_val(attrs, "core.id")
        raw = ms.get("metrics", [])

        if key not in groups:
            groups[key] = dict(
                name=ms.get("name", ""),
                attrs={k: attr_val(attrs, k) for k in attrs if k != "core.id"},
                cores={},
                meta={},
            )

        g = groups[key]
        vals = {}
        if isinstance(raw, list):  # format=json (with metadata)
            for m in raw:
                mn = m.get("name", "")
                vals[mn] = m.get("value", 0)
                g["meta"][mn] = dict(
                    instrument=m.get("instrument", ""),
                    unit=m.get("unit", ""),
                )
        elif isinstance(raw, dict):  # format=json_compact
            for mn, v in raw.items():
                vals[mn] = (
                    v.get("value", v) if isinstance(v, dict) and "value" in v else v
                )

        ck = int(core) if core is not None else None
        g["cores"][ck] = vals
    return groups


def _agg(group):
    """Aggregate metrics across cores -> ``{metric_name: total}``.

    Counters / up-down-counters -> sum.  Gauges -> sum (useful for
    CPU-utilization; per-core breakdown shows individual values).
    MMSC snapshots -> merged (min of mins, max of maxes, sum of sums,
    sum of counts).
    """
    cores = group["cores"]
    names = sorted({n for v in cores.values() for n in v})
    out = {}
    for mn in names:
        raws = [c.get(mn) for c in cores.values() if c.get(mn) is not None]
        ms = [v for v in raws if is_mmsc(v)]
        if ms:
            out[mn] = dict(
                min=min(v["min"] for v in ms),
                max=max(v["max"] for v in ms),
                sum=sum(v["sum"] for v in ms),
                count=sum(v["count"] for v in ms),
            )
        else:
            nums = [v for v in raws if isinstance(v, (int, float))]
            out[mn] = sum(nums)
    return out


# --- kind classification --------------------------------------------

ALL_KINDS = ("engine", "pipeline", "tokio", "channel", "receiver", "processor", "exporter")


def classify(name):
    """Map a metric-set *name* to a kind string.

    >>> classify('engine.metrics')
    'engine'
    >>> classify('channel.sender')
    'channel'
    >>> classify('syslog_cef.receiver.metrics')
    'receiver'
    >>> classify('otap.processor.batch')
    'processor'
    >>> classify('azure_monitor_exporter.metrics')
    'exporter'
    """
    n = name.lower()
    if n == "engine.metrics":
        return "engine"
    if n == "pipeline.metrics":
        return "pipeline"
    if n == "tokio.runtime":
        return "tokio"
    if n.startswith("channel."):
        return "channel"
    if "receiver" in n:
        return "receiver"
    if "processor" in n:
        return "processor"
    if "exporter" in n:
        return "exporter"
    # Fallback: anything unrecognised is grouped as 'other'
    return "other"


# --- display ordering ----------------------------------------------

# Controls the order kinds are printed in.
_KIND_ORDER = ["engine", "pipeline", "tokio", "channel", "receiver", "processor", "exporter", "other"]

_KIND_LABELS = {
    "engine": "Engine",
    "pipeline": "Pipeline",
    "tokio": "Tokio Runtime",
    "channel": "Channels",
    "receiver": "Receivers",
    "processor": "Processors",
    "exporter": "Exporters",
    "other": "Other",
}


def _sortkey(item):
    """Sort groups: by kind order, then alphabetically within kind."""
    _, g = item
    n = g["name"]
    kind = classify(n)
    try:
        ki = _KIND_ORDER.index(kind)
    except ValueError:
        ki = len(_KIND_ORDER)
    return (ki, n, item[0][1])


# --- printing -------------------------------------------------------


def _print_section(grp, prev, key, show_pc):
    """Print one metric-set section with aggregated totals and per-core."""
    name = grp["name"]
    attrs = grp["attrs"]
    cores = grp["cores"]
    meta = grp["meta"]
    totals = _agg(grp)

    real_cores = sorted(c for c in cores if c is not None)
    nc = len(real_cores)

    # -- header -- show node.id (or metric-set name) plus core count
    node_id = attrs.get("node.id", "")
    hdr = f"{name}  [{node_id}]" if node_id else name
    if nc > 1:
        hdr += f"  ({nc} cores)"
    print(f"  {hdr}")

    # -- aggregated metrics, two per line for compactness
    entries = []
    for mn in sorted(totals):
        v = totals[mn]
        m = meta.get(mn, {})
        inst = m.get("instrument", "")
        unit = m.get("unit", "")
        txt = fmt(v, unit, inst)

        # delta for counters
        pk = (key, mn)
        delta = ""
        if inst == "counter" and not is_mmsc(v) and pk in prev:
            d = v - prev[pk]
            if d > 0:
                delta = f" (+{_fmt_num(d)})"
            elif d < 0:
                delta = f" ({_fmt_num(d)})"
        prev[pk] = v
        entries.append((mn, f"{txt}{delta}"))

    i = 0
    while i < len(entries):
        n1, v1 = entries[i]
        col = f"    {n1}: {v1}"
        if i + 1 < len(entries):
            n2, v2 = entries[i + 1]
            print(f"{col:<48}{n2}: {v2}")
            i += 2
        else:
            print(col)
            i += 1

    # -- per-core breakdown (skip zero-valued and MMSC metrics)
    if show_pc and nc > 1:
        for mn in sorted(totals):
            v = totals[mn]
            if is_mmsc(v) or (isinstance(v, (int, float)) and v == 0):
                continue
            m = meta.get(mn, {})
            unit = m.get("unit", "")
            inst = m.get("instrument", "")
            parts = []
            for c in real_cores:
                cv = cores[c].get(mn, 0)
                fv = fmt(cv, unit, inst)
                parts.append((c, fv))
            w = max(len(fv) for _, fv in parts)
            pc = "  ".join(f"c{c}={fv:>{w}}" for c, fv in parts)
            print(f"    pc {mn}: {pc}")

    print()


# --- main -----------------------------------------------------------


def main():
    ap = argparse.ArgumentParser(
        description="otap-dataflow engine metrics -- console viewer",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    ap.add_argument(
        "-i",
        "--interval",
        type=int,
        default=5,
        help="polling interval in seconds (default: 5)",
    )
    ap.add_argument(
        "-u",
        "--url",
        default="http://127.0.0.1:8080/metrics?reset=false&format=json",
        help="metrics endpoint URL",
    )
    ap.add_argument(
        "-k",
        "--kind",
        default="receiver,exporter",
        help=(
            "comma-separated component kinds to show. "
            "Kinds: engine, pipeline, tokio, channel, receiver, processor, "
            "exporter.  Use 'all' to show everything. "
            "Default: receiver,exporter"
        ),
    )
    ap.add_argument(
        "-f",
        "--filter",
        default="",
        help=(
            "comma-separated substrings; only metric sets whose name "
            "or attributes match are shown (applied after --kind)"
        ),
    )
    ap.add_argument(
        "--per-core",
        action="store_true",
        help="show per-core breakdown (off by default)",
    )
    args, extra = ap.parse_known_args()

    # backward compat: engine-metrics.py [interval] [url]
    if extra:
        try:
            args.interval = int(extra[0])
        except ValueError:
            pass
        if len(extra) > 1:
            args.url = extra[1]

    url = args.url
    interval = args.interval
    filters = [s.strip().lower() for s in args.filter.split(",") if s.strip()]
    show_pc = args.per_core

    # Parse --kind
    kind_raw = args.kind.strip().lower()
    if kind_raw in ("", "all"):
        kinds = None  # show everything
    else:
        kinds = set(
            s.strip() for s in kind_raw.replace(";", ",").split(",") if s.strip()
        )
        unknown = kinds - set(ALL_KINDS) - {"other"}
        if unknown:
            print(
                f"Warning: unknown kind(s): {', '.join(sorted(unknown))}\n"
                f"  Valid kinds: {', '.join(ALL_KINDS)}\n",
                file=sys.stderr,
            )

    prev = {}  # (group_key, metric_name) -> previous value (for deltas)
    pids = find_engine_pids()

    kind_msg = f"  kinds={','.join(sorted(kinds))}" if kinds else ""
    print(f"Polling {url} every {interval}s{kind_msg}  (Ctrl+C to stop)\n", flush=True)

    try:
        while True:
            data = fetch(url)
            if data is None:
                time.sleep(interval)
                continue

            all_groups = parse(data)
            now = datetime.now().strftime("%H:%M:%S")

            # apply --kind filter
            if kinds is not None:
                groups = OrderedDict(
                    (k, g)
                    for k, g in all_groups.items()
                    if classify(g["name"]) in kinds
                )
            else:
                groups = all_groups

            # apply --filter (substring match on name or attributes)
            if filters:
                groups = OrderedDict(
                    (k, g)
                    for k, g in groups.items()
                    if any(
                        f in g["name"].lower()
                        or any(f in str(av).lower() for av in g["attrs"].values())
                        for f in filters
                    )
                )

            print(f"[{now}] {'-' * 60}")

            # -- compact summary line (engine + pipeline + OS) -------
            # Always consult *all_groups* so the summary shows regardless
            # of --kind / --filter selection.
            parts = []
            eng = next(
                (g for g in all_groups.values() if g["name"] == "engine.metrics"),
                None,
            )
            if eng:
                et = _agg(eng)
                rss = et.get("memory.rss", 0)
                cpu = et.get("cpu.utilization")
                if rss:
                    parts.append(f"rss={fmt(rss, '{By}')}")
                if cpu is not None:
                    parts.append(f"cpu={cpu * 100:.1f}%")

            if not pids:
                pids = find_engine_pids()
            os_rss = get_os_rss_mb(pids)
            os_cpu = get_os_cpu_pct(pids)
            if os_rss is None and pids:
                pids = []
            if os_rss:
                parts.append(f"rss(os)={os_rss:.1f} MB")
            if os_cpu is not None:
                parts.append(f"cpu(os)={os_cpu / (os.cpu_count() or 1):.1f}%")

            for pg in (
                g for g in all_groups.values() if g["name"] == "pipeline.metrics"
            ):
                pt = _agg(pg)
                heap = pt.get("memory.usage", 0)
                up = pt.get("uptime", 0)
                nc = len([c for c in pg["cores"] if c is not None])
                if heap:
                    parts.append(f"heap={fmt(heap, '{By}')}")
                if up:
                    parts.append(f"uptime={up:.0f}s")
                if nc > 1:
                    parts.append(f"{nc} cores")

            if parts:
                print(f"  {'  '.join(parts)}")
            print()

            # -- all metric-set sections, grouped by kind -----------
            sorted_items = sorted(groups.items(), key=_sortkey)
            last_kind = None
            for key, grp in sorted_items:
                kind = classify(grp["name"])
                if kind != last_kind:
                    label = _KIND_LABELS.get(kind, kind.title())
                    print(f"  -- {label} {'-' * max(1, 56 - len(label))}")
                    last_kind = kind
                _print_section(grp, prev, key, show_pc)

            print(f"{'=' * 64}")
            sys.stdout.flush()
            time.sleep(interval)
    except KeyboardInterrupt:
        print("\nStopped.")


if __name__ == "__main__":
    main()
