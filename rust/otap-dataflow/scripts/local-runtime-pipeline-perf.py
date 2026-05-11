#!/usr/bin/env python3
# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

"""Run real OTAP pipeline performance tests for LocalRuntime comparisons."""

from __future__ import annotations

import argparse
import csv
import json
import os
import re
import shlex
import shutil
import signal
import socket
import subprocess
import sys
import threading
import time
import urllib.error
import urllib.request
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, TextIO


PRE_LOCAL_RUNTIME_COMMIT = "c3cf2beba17f14a89341af778253c77c1a0a4346"
INITIAL_LOCAL_RUNTIME_COMMIT = "cdcf14e076a37cc50c6311536ac6b61b102bac2c"

DEFAULT_VARIANTS = {
    "pre-local-runtime": PRE_LOCAL_RUNTIME_COMMIT,
    "local-runtime-initial": INITIAL_LOCAL_RUNTIME_COMMIT,
    "local-runtime-optimized": "WORKTREE",
}

SCHEDULER_SWEEP = {
    "event31": 31,
    "event127": 127,
}

LABEL_RE = re.compile(r"^[A-Za-z0-9_.-]+$")
EPOLL_WAIT_RE = re.compile(r"\b(epoll_wait|epoll_pwait|epoll_pwait2)\b.*<([0-9.]+)>")


@dataclass
class CoreLayout:
    sut: int
    backend: int
    traffic: list[int]


@dataclass
class RunCase:
    label: str
    binary: Path
    source: str
    commit: str | None = None
    sut_env: dict[str, str] = field(default_factory=dict)
    sut_local_runtime_event_interval: int | None = None


@dataclass
class EngineInstance:
    name: str
    config: Path
    core: int
    admin_port: int
    process: subprocess.Popen | None = None
    stdout: TextIO | None = None
    stderr: TextIO | None = None

    @property
    def pid(self) -> int:
        if self.process is None:
            raise RuntimeError(f"{self.name} has not been started")
        return self.process.pid

    def close_logs(self) -> None:
        for handle in (self.stdout, self.stderr):
            if handle is not None:
                handle.close()
        self.stdout = None
        self.stderr = None


class ProcSampler:
    def __init__(self, pid: int, output: Path, interval: float) -> None:
        self.pid = pid
        self.output = output
        self.interval = interval
        self.stop_event = threading.Event()
        self.thread = threading.Thread(target=self._run, name=f"proc-sampler-{pid}")
        self.hz = os.sysconf(os.sysconf_names["SC_CLK_TCK"])

    def start(self) -> None:
        self.thread.start()

    def stop(self) -> None:
        self.stop_event.set()
        self.thread.join(timeout=max(2.0, self.interval * 2))

    def _run(self) -> None:
        start = time.time()
        last_time: float | None = None
        last_cpu: float | None = None
        fields = [
            "timestamp",
            "elapsed_s",
            "cpu_pct",
            "utime_s",
            "stime_s",
            "vmrss_kb",
            "vmhwm_kb",
            "threads",
            "voluntary_ctxt_switches",
            "nonvoluntary_ctxt_switches",
            "task_nr_switches",
            "task_nr_voluntary_switches",
            "task_nr_involuntary_switches",
        ]
        with self.output.open("w", newline="", encoding="utf-8") as handle:
            writer = csv.DictWriter(handle, fieldnames=fields)
            writer.writeheader()
            while not self.stop_event.is_set():
                now = time.time()
                sample = read_proc_sample(self.pid, self.hz)
                if sample is None:
                    return

                total_cpu = sample["utime_s"] + sample["stime_s"]
                if last_time is None or last_cpu is None:
                    cpu_pct = 0.0
                else:
                    elapsed = max(now - last_time, 1e-9)
                    cpu_pct = ((total_cpu - last_cpu) / elapsed) * 100.0

                last_time = now
                last_cpu = total_cpu
                sample.update(
                    {
                        "timestamp": f"{now:.6f}",
                        "elapsed_s": f"{now - start:.6f}",
                        "cpu_pct": f"{cpu_pct:.6f}",
                    }
                )
                writer.writerow(sample)
                handle.flush()
                self.stop_event.wait(self.interval)


class AdminSampler:
    def __init__(self, url: str, output: Path, interval: float) -> None:
        self.url = url
        self.output = output
        self.interval = interval
        self.stop_event = threading.Event()
        self.thread = threading.Thread(target=self._run, name="admin-sampler")

    def start(self) -> None:
        self.thread.start()

    def stop(self) -> None:
        self.stop_event.set()
        self.thread.join(timeout=max(2.0, self.interval * 2))

    def _run(self) -> None:
        with self.output.open("w", encoding="utf-8") as handle:
            while not self.stop_event.is_set():
                record: dict[str, Any] = {"timestamp": time.time()}
                try:
                    with urllib.request.urlopen(self.url, timeout=5) as response:
                        body = response.read()
                    record["data"] = json.loads(body) if body else None
                except (urllib.error.URLError, TimeoutError, json.JSONDecodeError) as exc:
                    record["error"] = str(exc)

                handle.write(json.dumps(record, sort_keys=True) + "\n")
                handle.flush()
                self.stop_event.wait(self.interval)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run six df_engine processes for OTAP traffic -> SUT -> noop backend "
            "and collect SUT CPU, memory, scheduler, perf, and epoll measurements."
        )
    )
    parser.add_argument(
        "--variant",
        action="append",
        metavar="LABEL=GIT_REF",
        help=(
            "Build a df_engine binary from a git ref. Defaults to the pre-LocalRuntime, "
            "initial LocalRuntime, and current working tree variants when no binaries are provided. "
            "Use WORKTREE to build the current checkout."
        ),
    )
    parser.add_argument(
        "--binary",
        action="append",
        metavar="LABEL=PATH",
        help="Run an existing df_engine binary. May be repeated for experiments.",
    )
    parser.add_argument(
        "--skip-build",
        action="store_true",
        help="Do not build --variant entries. Use only --binary entries.",
    )
    parser.add_argument(
        "--build-only",
        action="store_true",
        help="Build requested variants and exit before running pipelines.",
    )
    parser.add_argument(
        "--generate-only",
        action="store_true",
        help="Generate one set of pipeline configs and exit without building or running.",
    )
    parser.add_argument(
        "--validate-only",
        action="store_true",
        help="Generate and validate configs for all cases, then exit without binding sockets.",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        help="Directory for worktrees, configs, logs, and summaries.",
    )
    parser.add_argument(
        "--core-layout",
        default="0,1,2,3,4,5",
        help="Comma-separated cores in order: SUT,backend,traffic1,traffic2,traffic3,traffic4.",
    )
    parser.add_argument("--sut-port", type=int, default=4327)
    parser.add_argument("--backend-port", type=int, default=4328)
    parser.add_argument("--admin-base-port", type=int, default=18080)
    parser.add_argument("--traffic-rate", type=int, default=100_000)
    parser.add_argument("--traffic-batch-size", type=int, default=1000)
    parser.add_argument("--warmup-seconds", type=float, default=20.0)
    parser.add_argument("--duration-seconds", type=float, default=60.0)
    parser.add_argument("--sample-interval", type=float, default=1.0)
    parser.add_argument("--startup-timeout", type=float, default=30.0)
    parser.add_argument(
        "--compression-method",
        choices=("zstd", "gzip", "none"),
        default="zstd",
        help="gRPC compression for OTAP exporter/receiver links.",
    )
    parser.add_argument(
        "--sut-streams-per-signal",
        type=int,
        help="Set SUT OTAP exporter streams_per_signal for exporter experiments.",
    )
    parser.add_argument(
        "--sut-stream-queue-capacity",
        type=int,
        help="Set SUT OTAP exporter stream_queue_capacity for exporter experiments.",
    )
    parser.add_argument(
        "--sut-env",
        action="append",
        default=[],
        metavar="KEY=VALUE",
        help="Environment variable to apply only to the SUT process for every case.",
    )
    parser.add_argument(
        "--variant-env",
        action="append",
        default=[],
        metavar="LABEL:KEY=VALUE",
        help="Environment variable to apply only to the SUT process for one case.",
    )
    parser.add_argument(
        "--scheduler-sweep",
        action="store_true",
        help=(
            "Clone the local-runtime-optimized case into scheduler interval experiments "
            "using engine.runtime.local_runtime.event_interval in the SUT config."
        ),
    )
    parser.add_argument(
        "--tokio-unstable",
        action="store_true",
        help="Build variants with RUSTFLAGS='--cfg tokio_unstable' for extra Tokio metrics.",
    )
    parser.add_argument(
        "--build-rustflags",
        default="",
        help="Extra RUSTFLAGS to append when building --variant entries.",
    )
    parser.add_argument(
        "--no-taskset",
        action="store_true",
        help="Do not wrap engine processes in taskset -c CORE.",
    )
    parser.add_argument("--no-validate", action="store_true")
    parser.add_argument("--no-perf", action="store_true")
    parser.add_argument("--no-pidstat", action="store_true")
    parser.add_argument(
        "--with-strace",
        action="store_true",
        help="Attach strace to the SUT to collect epoll wait durations. Intrusive.",
    )
    parser.add_argument(
        "--keep-going",
        action="store_true",
        help="Continue with later cases if one case fails.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    engine_root = Path(__file__).resolve().parents[1]
    git_root = find_git_root(engine_root)
    engine_rel = engine_root.relative_to(git_root)
    output_dir = prepare_output_dir(args.output_dir)
    core_layout = parse_core_layout(args.core_layout)

    if args.generate_only:
        configs_dir = output_dir / "config-preview"
        write_pipeline_configs(args, configs_dir, core_layout)
        print(f"Generated config preview in {configs_dir}")
        return 0

    cases = collect_cases(args, git_root, engine_rel, output_dir)
    if args.scheduler_sweep:
        cases = expand_scheduler_sweep(cases)

    global_sut_env = parse_env_entries(args.sut_env)
    variant_sut_env = parse_variant_env_entries(args.variant_env)
    for case in cases:
        specific_env = variant_sut_env.get(case.label, {})
        case.sut_env = {**global_sut_env, **case.sut_env, **specific_env}

    if args.validate_only:
        validate_cases_only(args, output_dir, cases, core_layout)
        write_matrix_metadata(output_dir, cases, args, core_layout)
        print(f"Validated {len(cases)} case(s). Results: {output_dir}")
        return 0

    if args.build_only:
        write_matrix_metadata(output_dir, cases, args, core_layout)
        print(f"Built {len(cases)} case(s) under {output_dir}")
        return 0

    summaries = []
    for case in cases:
        try:
            summaries.append(run_case(args, output_dir, case, core_layout))
        except Exception as exc:
            summary = {
                "label": case.label,
                "status": "failed",
                "error": str(exc),
                "binary": str(case.binary),
                "commit": case.commit,
            }
            summaries.append(summary)
            write_json(output_dir / "runs" / case.label / "summary.json", summary)
            if not args.keep_going:
                write_matrix_summary(output_dir, summaries)
                raise
            print(f"[warn] {case.label} failed: {exc}", file=sys.stderr)

    write_matrix_metadata(output_dir, cases, args, core_layout)
    write_matrix_summary(output_dir, summaries)
    print(f"Completed {len(summaries)} case(s). Results: {output_dir}")
    return 0


def find_git_root(engine_root: Path) -> Path:
    result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        cwd=engine_root,
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    return Path(result.stdout.strip())


def prepare_output_dir(output_dir: Path | None) -> Path:
    if output_dir is None:
        stamp = time.strftime("%Y%m%d-%H%M%S")
        output_dir = Path("/tmp/otap-local-runtime-perf") / stamp
    output_dir.mkdir(parents=True, exist_ok=True)
    return output_dir.resolve()


def parse_core_layout(raw: str) -> CoreLayout:
    parts = [part.strip() for part in raw.split(",") if part.strip()]
    if len(parts) != 6:
        raise ValueError("--core-layout must contain exactly six CPU core IDs")
    cores = [int(part) for part in parts]
    if len(set(cores)) != len(cores):
        raise ValueError("--core-layout CPU core IDs must be distinct")
    cpu_count = os.cpu_count()
    if cpu_count is not None and max(cores) >= cpu_count:
        raise ValueError(f"--core-layout references core {max(cores)}, but only {cpu_count} cores exist")
    return CoreLayout(sut=cores[0], backend=cores[1], traffic=cores[2:])


def collect_cases(
    args: argparse.Namespace,
    git_root: Path,
    engine_rel: Path,
    output_dir: Path,
) -> list[RunCase]:
    variant_specs = parse_label_specs(args.variant or [])
    binary_specs = parse_label_specs(args.binary or [])

    if not variant_specs and not binary_specs:
        variant_specs = DEFAULT_VARIANTS.copy()
    if args.skip_build and variant_specs:
        raise ValueError("--skip-build requires using --binary instead of --variant")

    cases: list[RunCase] = []
    if variant_specs:
        cases.extend(build_variants(args, git_root, engine_rel, output_dir, variant_specs))
    for label, path in binary_specs.items():
        validate_label(label)
        binary = Path(path).expanduser().resolve()
        if not binary.exists():
            raise FileNotFoundError(binary)
        cases.append(RunCase(label=label, binary=binary, source=f"binary:{binary}"))

    labels = [case.label for case in cases]
    if len(set(labels)) != len(labels):
        raise ValueError("case labels must be unique")
    return cases


def parse_label_specs(entries: list[str]) -> dict[str, str]:
    specs = {}
    for entry in entries:
        if "=" not in entry:
            raise ValueError(f"expected LABEL=VALUE, got {entry!r}")
        label, value = entry.split("=", 1)
        validate_label(label)
        specs[label] = value
    return specs


def validate_label(label: str) -> None:
    if not LABEL_RE.match(label):
        raise ValueError(f"invalid label {label!r}; use letters, numbers, '.', '_', '-'")


def build_variants(
    args: argparse.Namespace,
    git_root: Path,
    engine_rel: Path,
    output_dir: Path,
    variants: dict[str, str],
) -> list[RunCase]:
    bin_dir = output_dir / "bin"
    worktree_dir = output_dir / "worktrees"
    bin_dir.mkdir(parents=True, exist_ok=True)
    worktree_dir.mkdir(parents=True, exist_ok=True)
    cases = []
    build_env = os.environ.copy()
    rustflags = []
    if build_env.get("RUSTFLAGS"):
        rustflags.append(build_env["RUSTFLAGS"])
    if args.tokio_unstable:
        rustflags.append("--cfg tokio_unstable")
    if args.build_rustflags:
        rustflags.append(args.build_rustflags)
    if rustflags:
        build_env["RUSTFLAGS"] = " ".join(rustflags)

    for label, git_ref in variants.items():
        validate_label(label)
        binary = bin_dir / f"df_engine-{label}"
        if git_ref == "WORKTREE":
            worktree = git_root
            worktree_engine_root = git_root / engine_rel
        else:
            worktree = worktree_dir / label
            run_cmd(["git", "worktree", "add", "--detach", str(worktree), git_ref], cwd=git_root)
            worktree_engine_root = worktree / engine_rel

        run_cmd(
            ["cargo", "build", "--release", "--bin", "df_engine"],
            cwd=worktree_engine_root,
            env=build_env,
        )
        shutil.copy2(worktree_engine_root / "target" / "release" / "df_engine", binary)
        commit = run_output(["git", "rev-parse", "HEAD"], cwd=worktree).strip()
        if git_ref == "WORKTREE" and run_output(["git", "status", "--porcelain"], cwd=worktree):
            commit = f"{commit}-dirty"
        cases.append(
            RunCase(
                label=label,
                binary=binary,
                source=f"git:{git_ref}",
                commit=commit,
            )
        )
    return cases


def expand_scheduler_sweep(cases: list[RunCase]) -> list[RunCase]:
    expanded = list(cases)
    base = next((case for case in cases if case.label == "local-runtime-optimized"), None)
    if base is None:
        print(
            "[warn] --scheduler-sweep skipped: no local-runtime-optimized case",
            file=sys.stderr,
        )
        return expanded
    for suffix, event_interval in SCHEDULER_SWEEP.items():
        expanded.append(
            RunCase(
                label=f"{base.label}-{suffix}",
                binary=base.binary,
                source=f"{base.source}+scheduler-sweep:{suffix}",
                commit=base.commit,
                sut_local_runtime_event_interval=event_interval,
            )
        )
    return expanded


def parse_env_entries(entries: list[str]) -> dict[str, str]:
    env = {}
    for entry in entries:
        if "=" not in entry:
            raise ValueError(f"expected KEY=VALUE, got {entry!r}")
        key, value = entry.split("=", 1)
        if not key:
            raise ValueError(f"empty env key in {entry!r}")
        env[key] = value
    return env


def parse_variant_env_entries(entries: list[str]) -> dict[str, dict[str, str]]:
    result: dict[str, dict[str, str]] = {}
    for entry in entries:
        if ":" not in entry:
            raise ValueError(f"expected LABEL:KEY=VALUE, got {entry!r}")
        label, env_entry = entry.split(":", 1)
        validate_label(label)
        result.setdefault(label, {}).update(parse_env_entries([env_entry]))
    return result


def run_case(
    args: argparse.Namespace,
    output_dir: Path,
    case: RunCase,
    core_layout: CoreLayout,
) -> dict[str, Any]:
    print(f"[case] {case.label}")
    case_dir = output_dir / "runs" / case.label
    configs_dir = case_dir / "configs"
    logs_dir = case_dir / "logs"
    case_dir.mkdir(parents=True, exist_ok=True)
    logs_dir.mkdir(parents=True, exist_ok=True)
    configs = write_pipeline_configs(args, configs_dir, core_layout, case)
    instances = build_instances(configs, core_layout, args.admin_base_port)
    samplers: list[ProcSampler | AdminSampler] = []
    monitor_procs: list[subprocess.Popen] = []
    started: list[EngineInstance] = []
    status = "passed"
    error: str | None = None

    try:
        if not args.no_validate:
            for instance in instances:
                validate_engine_config(case, instance, logs_dir)

        backend = instances[0]
        sut = instances[1]
        traffic = instances[2:]

        start_engine(case, backend, logs_dir, use_taskset=not args.no_taskset)
        started.append(backend)
        wait_for_tcp("127.0.0.1", args.backend_port, args.startup_timeout, backend)
        wait_for_tcp("127.0.0.1", backend.admin_port, args.startup_timeout, backend)

        start_engine(case, sut, logs_dir, use_taskset=not args.no_taskset, extra_env=case.sut_env)
        started.append(sut)
        wait_for_tcp("127.0.0.1", args.sut_port, args.startup_timeout, sut)
        wait_for_tcp("127.0.0.1", sut.admin_port, args.startup_timeout, sut)

        for instance in traffic:
            start_engine(case, instance, logs_dir, use_taskset=not args.no_taskset)
            started.append(instance)
            wait_for_tcp("127.0.0.1", instance.admin_port, args.startup_timeout, instance)

        time.sleep(args.warmup_seconds)

        proc_sampler = ProcSampler(
            sut.pid,
            logs_dir / "sut-proc-samples.csv",
            args.sample_interval,
        )
        admin_sampler = AdminSampler(
            metrics_url(sut.admin_port),
            logs_dir / "sut-admin-metrics.jsonl",
            args.sample_interval,
        )
        samplers.extend([proc_sampler, admin_sampler])
        for sampler in samplers:
            sampler.start()

        monitor_procs.extend(start_external_monitors(args, sut.pid, logs_dir))
        time.sleep(args.duration_seconds)
    except Exception as exc:
        status = "failed"
        error = str(exc)
        raise
    finally:
        for sampler in samplers:
            sampler.stop()
        stop_monitor_processes(monitor_procs)
        for instance in reversed(started):
            stop_engine(instance)
        for instance in instances:
            instance.close_logs()

    summary = summarize_case(case, case_dir, status, error)
    write_json(case_dir / "summary.json", summary)
    return summary


def validate_cases_only(
    args: argparse.Namespace,
    output_dir: Path,
    cases: list[RunCase],
    core_layout: CoreLayout,
) -> None:
    for case in cases:
        case_dir = output_dir / "runs" / case.label
        configs_dir = case_dir / "configs"
        logs_dir = case_dir / "logs"
        logs_dir.mkdir(parents=True, exist_ok=True)
        configs = write_pipeline_configs(args, configs_dir, core_layout, case)
        for instance in build_instances(configs, core_layout, args.admin_base_port):
            validate_engine_config(case, instance, logs_dir)


def build_instances(
    configs: dict[str, Path],
    core_layout: CoreLayout,
    admin_base_port: int,
) -> list[EngineInstance]:
    return [
        EngineInstance("backend", configs["backend"], core_layout.backend, admin_base_port + 1),
        EngineInstance("sut", configs["sut"], core_layout.sut, admin_base_port + 2),
        EngineInstance("traffic1", configs["traffic1"], core_layout.traffic[0], admin_base_port + 3),
        EngineInstance("traffic2", configs["traffic2"], core_layout.traffic[1], admin_base_port + 4),
        EngineInstance("traffic3", configs["traffic3"], core_layout.traffic[2], admin_base_port + 5),
        EngineInstance("traffic4", configs["traffic4"], core_layout.traffic[3], admin_base_port + 6),
    ]


def write_pipeline_configs(
    args: argparse.Namespace,
    configs_dir: Path,
    core_layout: CoreLayout,
    case: RunCase | None = None,
) -> dict[str, Path]:
    configs_dir.mkdir(parents=True, exist_ok=True)
    configs = {
        "backend": configs_dir / "backend-otap-noop.yaml",
        "sut": configs_dir / "sut-otap-forward.yaml",
    }
    configs["backend"].write_text(
        render_backend_config(args, core_layout.backend, args.admin_base_port + 1),
        encoding="utf-8",
    )
    configs["sut"].write_text(
        render_sut_config(
            args,
            core_layout.sut,
            args.admin_base_port + 2,
            None if case is None else case.sut_local_runtime_event_interval,
        ),
        encoding="utf-8",
    )
    for index, core in enumerate(core_layout.traffic, start=1):
        path = configs_dir / f"traffic-gen-{index}.yaml"
        path.write_text(
            render_traffic_config(args, index, core, args.admin_base_port + 2 + index),
            encoding="utf-8",
        )
        configs[f"traffic{index}"] = path
    return configs


def render_common_header(
    name: str,
    core: int,
    admin_port: int,
    local_runtime_event_interval: int | None = None,
) -> str:
    local_runtime_config = ""
    if local_runtime_event_interval is not None:
        local_runtime_config = f"""  runtime:
    local_runtime:
      event_interval: {local_runtime_event_interval}
"""
    return f"""version: otel_dataflow/v1

policies:
  channel_capacity:
    control:
      node: 256
      pipeline: 256
    pdata: 128
  telemetry:
    pipeline_metrics: true
    tokio_metrics: true
    runtime_metrics: detailed
  resources:
    core_allocation:
      type: core_set
      set:
        - start: {core}
          end: {core}

engine:
  http_admin:
    bind_address: 127.0.0.1:{admin_port}
{local_runtime_config}\
  telemetry:
    logs:
      level: warn

groups:
  local_runtime_perf:
    pipelines:
      {name}:
        policies:
          resources:
            core_allocation:
              type: core_set
              set:
                - start: {core}
                  end: {core}
"""


def render_receiver_compression(args: argparse.Namespace, indent: str = "              ") -> str:
    if args.compression_method == "none":
        return ""
    return f"{indent}compression_method: {args.compression_method}\n"


def render_exporter_compression(args: argparse.Namespace, indent: str = "              ") -> str:
    return f"{indent}compression_method: {args.compression_method}\n"


def render_sut_exporter_options(args: argparse.Namespace) -> str:
    lines = []
    if args.sut_streams_per_signal is not None:
        lines.append(f"              streams_per_signal: {args.sut_streams_per_signal}")
    if args.sut_stream_queue_capacity is not None:
        lines.append(f"              stream_queue_capacity: {args.sut_stream_queue_capacity}")
    return "\n".join(lines) + ("\n" if lines else "")


def render_backend_config(args: argparse.Namespace, core: int, admin_port: int) -> str:
    compression = render_receiver_compression(args)
    return (
        render_common_header("backend", core, admin_port)
        + f"""        nodes:
          receiver:
            type: receiver:otap
            config:
              listening_addr: "127.0.0.1:{args.backend_port}"
{compression}              response_stream_channel_size: 256
              wait_for_result: true
              tls: null
          exporter:
            type: exporter:noop
            config: null
        connections:
          - from: receiver
            to: exporter
"""
    )


def render_sut_config(
    args: argparse.Namespace,
    core: int,
    admin_port: int,
    local_runtime_event_interval: int | None,
) -> str:
    receiver_compression = render_receiver_compression(args)
    exporter_compression = render_exporter_compression(args)
    exporter_options = render_sut_exporter_options(args)
    return (
        render_common_header("sut", core, admin_port, local_runtime_event_interval)
        + f"""        nodes:
          receiver:
            type: receiver:otap
            config:
              listening_addr: "127.0.0.1:{args.sut_port}"
{receiver_compression}              response_stream_channel_size: 256
              wait_for_result: true
              tls: null
          exporter:
            type: exporter:otap
            config:
              grpc_endpoint: "http://127.0.0.1:{args.backend_port}"
{exporter_compression}{exporter_options}        connections:
          - from: receiver
            to: exporter
"""
    )


def render_traffic_config(
    args: argparse.Namespace,
    index: int,
    core: int,
    admin_port: int,
) -> str:
    exporter_compression = render_exporter_compression(args)
    return (
        render_common_header(f"traffic_gen_{index}", core, admin_port)
        + f"""        nodes:
          receiver:
            type: receiver:traffic_generator
            config:
              data_source: static
              generation_strategy: pre_generated
              traffic_config:
                production_mode: smooth
                signals_per_second: {args.traffic_rate}
                max_signal_count: null
                max_batch_size: {args.traffic_batch_size}
                metric_weight: 0
                trace_weight: 0
                log_weight: 30
          exporter:
            type: exporter:otap
            config:
              grpc_endpoint: "http://127.0.0.1:{args.sut_port}"
{exporter_compression}        connections:
          - from: receiver
            to: exporter
"""
    )


def validate_engine_config(case: RunCase, instance: EngineInstance, logs_dir: Path) -> None:
    env = os.environ.copy()
    if instance.name == "sut":
        env.update(case.sut_env)
    cmd = [
        str(case.binary),
        "--config",
        str(instance.config),
        "--core-id-range",
        str(instance.core),
        "--http-admin-bind",
        f"127.0.0.1:{instance.admin_port}",
        "--validate-and-exit",
    ]
    result = subprocess.run(
        cmd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    log_path = logs_dir / f"{instance.name}-validate.log"
    log_path.write_text(result.stdout, encoding="utf-8")
    if result.returncode != 0:
        raise RuntimeError(f"{instance.name} config validation failed; see {log_path}")


def start_engine(
    case: RunCase,
    instance: EngineInstance,
    logs_dir: Path,
    *,
    use_taskset: bool,
    extra_env: dict[str, str] | None = None,
) -> None:
    env = os.environ.copy()
    if extra_env:
        env.update(extra_env)
    cmd = [
        str(case.binary),
        "--config",
        str(instance.config),
        "--core-id-range",
        str(instance.core),
        "--http-admin-bind",
        f"127.0.0.1:{instance.admin_port}",
    ]
    if use_taskset and shutil.which("taskset"):
        cmd = ["taskset", "-c", str(instance.core), *cmd]

    instance.stdout = (logs_dir / f"{instance.name}.stdout.log").open("w", encoding="utf-8")
    instance.stderr = (logs_dir / f"{instance.name}.stderr.log").open("w", encoding="utf-8")
    instance.process = subprocess.Popen(
        cmd,
        env=env,
        stdout=instance.stdout,
        stderr=instance.stderr,
        start_new_session=True,
    )


def wait_for_tcp(
    host: str,
    port: int,
    timeout: float,
    instance: EngineInstance,
) -> None:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if instance.process is not None and instance.process.poll() is not None:
            raise RuntimeError(
                f"{instance.name} exited before {host}:{port} became ready "
                f"(exit={instance.process.returncode})"
            )
        try:
            with socket.create_connection((host, port), timeout=0.5):
                return
        except OSError:
            time.sleep(0.2)
    raise TimeoutError(f"{instance.name} did not open {host}:{port} within {timeout}s")


def metrics_url(admin_port: int) -> str:
    return (
        f"http://127.0.0.1:{admin_port}/api/v1/telemetry/metrics"
        "?format=json&reset=false&keep_all_zeroes=false"
    )


def start_external_monitors(
    args: argparse.Namespace,
    sut_pid: int,
    logs_dir: Path,
) -> list[subprocess.Popen]:
    procs = []
    if not args.no_pidstat and shutil.which("pidstat"):
        procs.append(
            start_monitor(
                [
                    "pidstat",
                    "-h",
                    "-u",
                    "-r",
                    "-w",
                    "-t",
                    "-p",
                    str(sut_pid),
                    str(max(1, int(args.sample_interval))),
                ],
                logs_dir / "sut-pidstat.log",
            )
        )
    if not args.no_perf and shutil.which("perf"):
        events = ",".join(
            [
                "task-clock",
                "cpu-clock",
                "context-switches",
                "cpu-migrations",
                "page-faults",
                "cycles",
                "instructions",
                "cache-misses",
            ]
        )
        procs.append(
            subprocess.Popen(
                [
                    "perf",
                    "stat",
                    "-x",
                    ",",
                    "-e",
                    events,
                    "-p",
                    str(sut_pid),
                    "-o",
                    str(logs_dir / "sut-perf-stat.csv"),
                    "--",
                    "sleep",
                    str(args.duration_seconds),
                ],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.PIPE,
                text=True,
                start_new_session=True,
            )
        )
    if args.with_strace and shutil.which("strace"):
        procs.append(
            subprocess.Popen(
                [
                    "strace",
                    "-f",
                    "-ttT",
                    "-e",
                    "trace=epoll_wait,epoll_pwait,epoll_pwait2",
                    "-p",
                    str(sut_pid),
                    "-o",
                    str(logs_dir / "sut-strace-epoll.log"),
                ],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.PIPE,
                text=True,
                start_new_session=True,
            )
        )
    return procs


def start_monitor(cmd: list[str], output: Path) -> subprocess.Popen:
    handle = output.open("w", encoding="utf-8")
    return subprocess.Popen(
        cmd,
        stdout=handle,
        stderr=subprocess.STDOUT,
        text=True,
        start_new_session=True,
    )


def stop_monitor_processes(procs: list[subprocess.Popen]) -> None:
    for proc in procs:
        if proc.poll() is not None:
            continue
        signal_process_group(proc, signal.SIGINT)
    deadline = time.time() + 5
    for proc in procs:
        remaining = max(0.0, deadline - time.time())
        try:
            proc.wait(timeout=remaining)
        except subprocess.TimeoutExpired:
            signal_process_group(proc, signal.SIGTERM)


def stop_engine(instance: EngineInstance) -> None:
    proc = instance.process
    if proc is None or proc.poll() is not None:
        return
    signal_process_group(proc, signal.SIGINT)
    try:
        proc.wait(timeout=10)
        return
    except subprocess.TimeoutExpired:
        signal_process_group(proc, signal.SIGTERM)
    try:
        proc.wait(timeout=5)
        return
    except subprocess.TimeoutExpired:
        signal_process_group(proc, signal.SIGKILL)
        proc.wait(timeout=5)


def signal_process_group(proc: subprocess.Popen, sig: signal.Signals) -> None:
    try:
        os.killpg(proc.pid, sig)
    except ProcessLookupError:
        return


def read_proc_sample(pid: int, hz: int) -> dict[str, Any] | None:
    status = read_proc_status(pid)
    stat = read_proc_stat(pid, hz)
    task_sched = read_task_sched(pid)
    if status is None or stat is None:
        return None
    return {
        **stat,
        "vmrss_kb": status.get("VmRSS", 0),
        "vmhwm_kb": status.get("VmHWM", 0),
        "threads": status.get("Threads", 0),
        "voluntary_ctxt_switches": status.get("voluntary_ctxt_switches", 0),
        "nonvoluntary_ctxt_switches": status.get("nonvoluntary_ctxt_switches", 0),
        **task_sched,
    }


def read_proc_status(pid: int) -> dict[str, int] | None:
    path = Path("/proc") / str(pid) / "status"
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except FileNotFoundError:
        return None
    result = {}
    for line in lines:
        if ":" not in line:
            continue
        key, raw = line.split(":", 1)
        if key in {
            "VmRSS",
            "VmHWM",
            "Threads",
            "voluntary_ctxt_switches",
            "nonvoluntary_ctxt_switches",
        }:
            match = re.search(r"\d+", raw)
            if match:
                result[key] = int(match.group(0))
    return result


def read_proc_stat(pid: int, hz: int) -> dict[str, float] | None:
    path = Path("/proc") / str(pid) / "stat"
    try:
        raw = path.read_text(encoding="utf-8")
    except FileNotFoundError:
        return None
    rparen = raw.rfind(")")
    if rparen == -1:
        return None
    fields = raw[rparen + 2 :].split()
    if len(fields) < 13:
        return None
    return {
        "utime_s": int(fields[11]) / hz,
        "stime_s": int(fields[12]) / hz,
    }


def read_task_sched(pid: int) -> dict[str, int]:
    totals = {
        "task_nr_switches": 0,
        "task_nr_voluntary_switches": 0,
        "task_nr_involuntary_switches": 0,
    }
    task_dir = Path("/proc") / str(pid) / "task"
    try:
        tids = list(task_dir.iterdir())
    except FileNotFoundError:
        return totals
    for tid_dir in tids:
        sched_path = tid_dir / "sched"
        try:
            lines = sched_path.read_text(encoding="utf-8").splitlines()
        except FileNotFoundError:
            continue
        for line in lines:
            if ":" not in line:
                continue
            key, raw = line.split(":", 1)
            key = key.strip()
            if key == "nr_switches":
                totals["task_nr_switches"] += int(raw.strip().split()[0])
            elif key == "nr_voluntary_switches":
                totals["task_nr_voluntary_switches"] += int(raw.strip().split()[0])
            elif key == "nr_involuntary_switches":
                totals["task_nr_involuntary_switches"] += int(raw.strip().split()[0])
    return totals


def summarize_case(
    case: RunCase,
    case_dir: Path,
    status: str,
    error: str | None,
) -> dict[str, Any]:
    logs_dir = case_dir / "logs"
    proc_summary = summarize_proc_samples(logs_dir / "sut-proc-samples.csv")
    metrics_summary = summarize_admin_metrics(logs_dir / "sut-admin-metrics.jsonl")
    epoll_summary = summarize_epoll_strace(logs_dir / "sut-strace-epoll.log")
    summary = {
        "label": case.label,
        "status": status,
        "error": error,
        "binary": str(case.binary),
        "source": case.source,
        "commit": case.commit,
        "sut_env": case.sut_env,
        "sut_local_runtime_event_interval": case.sut_local_runtime_event_interval,
        "proc": proc_summary,
        "metrics": metrics_summary,
        "epoll": epoll_summary,
        "logs_dir": str(logs_dir),
    }
    return summary


def summarize_proc_samples(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    with path.open(encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    if len(rows) < 2:
        return {"sample_count": len(rows)}

    def f(row: dict[str, str], key: str) -> float:
        raw = row.get(key, "")
        return float(raw) if raw else 0.0

    first = rows[0]
    last = rows[-1]
    cpu_values = [f(row, "cpu_pct") for row in rows[1:]]
    rss_values = [f(row, "vmrss_kb") for row in rows]
    return {
        "sample_count": len(rows),
        "avg_cpu_pct": sum(cpu_values) / len(cpu_values),
        "max_cpu_pct": max(cpu_values),
        "max_rss_kb": max(rss_values),
        "final_rss_kb": f(last, "vmrss_kb"),
        "voluntary_context_switches_delta": f(last, "voluntary_ctxt_switches")
        - f(first, "voluntary_ctxt_switches"),
        "nonvoluntary_context_switches_delta": f(last, "nonvoluntary_ctxt_switches")
        - f(first, "nonvoluntary_ctxt_switches"),
        "task_switches_delta": f(last, "task_nr_switches") - f(first, "task_nr_switches"),
        "task_voluntary_switches_delta": f(last, "task_nr_voluntary_switches")
        - f(first, "task_nr_voluntary_switches"),
        "task_involuntary_switches_delta": f(last, "task_nr_involuntary_switches")
        - f(first, "task_nr_involuntary_switches"),
    }


def summarize_admin_metrics(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    snapshots = []
    with path.open(encoding="utf-8") as handle:
        for line in handle:
            record = json.loads(line)
            if record.get("data"):
                snapshots.append(record["data"])
    if len(snapshots) < 2:
        return {"sample_count": len(snapshots)}

    first = flatten_metrics(snapshots[0])
    last = flatten_metrics(snapshots[-1])
    deltas = {
        key: last[key] - first.get(key, 0.0)
        for key in sorted(last)
        if key in first or last[key] != 0.0
    }
    return {
        "sample_count": len(snapshots),
        "last": last,
        "delta": deltas,
    }


def flatten_metrics(data: dict[str, Any]) -> dict[str, float]:
    result: dict[str, float] = {}
    for metric_set in data.get("metric_sets", []):
        set_name = metric_set.get("name", "")
        if set_name not in {"pipeline", "tokio.runtime"}:
            continue
        for metric in metric_set.get("metrics", []):
            metric_name = metric.get("name", "")
            number = metric_number(metric.get("value"))
            if number is None:
                continue
            key = f"{set_name}.{metric_name}"
            result[key] = result.get(key, 0.0) + number
    return result


def metric_number(value: Any) -> float | None:
    if isinstance(value, (int, float)):
        return float(value)
    if isinstance(value, dict) and isinstance(value.get("sum"), (int, float)):
        return float(value["sum"])
    return None


def summarize_epoll_strace(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    durations = []
    with path.open(encoding="utf-8", errors="replace") as handle:
        for line in handle:
            match = EPOLL_WAIT_RE.search(line)
            if match:
                durations.append(float(match.group(2)))
    if not durations:
        return {"count": 0}
    return {
        "count": len(durations),
        "total_s": sum(durations),
        "avg_s": sum(durations) / len(durations),
        "max_s": max(durations),
    }


def write_matrix_metadata(
    output_dir: Path,
    cases: list[RunCase],
    args: argparse.Namespace,
    core_layout: CoreLayout,
) -> None:
    metadata = {
        "cases": [
            {
                "label": case.label,
                "binary": str(case.binary),
                "source": case.source,
                "commit": case.commit,
                "sut_env": case.sut_env,
                "sut_local_runtime_event_interval": case.sut_local_runtime_event_interval,
            }
            for case in cases
        ],
        "core_layout": {
            "sut": core_layout.sut,
            "backend": core_layout.backend,
            "traffic": core_layout.traffic,
        },
        "args": vars(args),
    }
    write_json(output_dir / "matrix-metadata.json", metadata)


def write_matrix_summary(output_dir: Path, summaries: list[dict[str, Any]]) -> None:
    write_json(output_dir / "matrix-summary.json", summaries)
    csv_path = output_dir / "matrix-summary.csv"
    fields = [
        "label",
        "status",
        "commit",
        "avg_cpu_pct",
        "max_cpu_pct",
        "max_rss_kb",
        "voluntary_context_switches_delta",
        "nonvoluntary_context_switches_delta",
        "task_switches_delta",
        "tokio_worker_busy_time_delta",
        "tokio_worker_park_count_delta",
        "tokio_worker_poll_count_delta",
        "epoll_wait_count",
        "epoll_wait_avg_s",
        "error",
    ]
    with csv_path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields)
        writer.writeheader()
        for summary in summaries:
            proc = summary.get("proc", {})
            metrics_delta = summary.get("metrics", {}).get("delta", {})
            epoll = summary.get("epoll", {})
            writer.writerow(
                {
                    "label": summary.get("label"),
                    "status": summary.get("status"),
                    "commit": summary.get("commit"),
                    "avg_cpu_pct": proc.get("avg_cpu_pct"),
                    "max_cpu_pct": proc.get("max_cpu_pct"),
                    "max_rss_kb": proc.get("max_rss_kb"),
                    "voluntary_context_switches_delta": proc.get(
                        "voluntary_context_switches_delta"
                    ),
                    "nonvoluntary_context_switches_delta": proc.get(
                        "nonvoluntary_context_switches_delta"
                    ),
                    "task_switches_delta": proc.get("task_switches_delta"),
                    "tokio_worker_busy_time_delta": metrics_delta.get(
                        "tokio.runtime.worker.busy.time"
                    ),
                    "tokio_worker_park_count_delta": metrics_delta.get(
                        "tokio.runtime.worker.park.count"
                    ),
                    "tokio_worker_poll_count_delta": metrics_delta.get(
                        "tokio.runtime.worker.poll.count"
                    ),
                    "epoll_wait_count": epoll.get("count"),
                    "epoll_wait_avg_s": epoll.get("avg_s"),
                    "error": summary.get("error"),
                }
            )


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(value, handle, indent=2, sort_keys=True, default=str)
        handle.write("\n")


def run_cmd(cmd: list[str], *, cwd: Path, env: dict[str, str] | None = None) -> None:
    print(f"[cmd] {shlex.join(cmd)}")
    subprocess.run(cmd, cwd=cwd, env=env, check=True)


def run_output(cmd: list[str], *, cwd: Path) -> str:
    result = subprocess.run(
        cmd,
        cwd=cwd,
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    return result.stdout


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)
