from unittest.mock import MagicMock, patch

from lib.impl.strategies.monitoring import docker_component
from lib.impl.strategies.monitoring.docker_component import (
    _collect_container_perf_counters,
    _parse_perf_stat_output as parse_container_perf_stat_output,
)
from lib.impl.strategies.monitoring.perf_cycles_process import (
    _collect_perf_counters,
    _parse_perf_stat_output as parse_process_perf_stat_output,
)
from lib.impl.strategies.monitoring import perf_support


def test_parse_perf_stat_output_parses_supported_metrics():
    stderr = """
      12,345;cycles;cycles;100.00%;
      9,876;instructions;instructions;100.00%;
      <not supported>;ref-cycles;ref-cycles;100.00%;
    """

    result = parse_process_perf_stat_output(stderr)

    assert result["cycles"] == 12345.0
    assert result["instructions"] == 9876.0
    assert "ref-cycles" not in result


def test_collect_perf_counters_handles_perf_output():
    fake_completed = MagicMock()
    fake_completed.stderr = """
      1,234;cycles;cycles;100.00%;
      2,345;ref-cycles;ref-cycles;100.00%;
      3,456;instructions;instructions;100.00%;
    """

    with patch(
        "lib.impl.strategies.monitoring.perf_cycles_process.ensure_perf_binary",
        return_value="/usr/bin/perf",
    ), patch(
        "lib.impl.strategies.monitoring.perf_cycles_process.perf_privilege_prefix",
        return_value=[],
    ), patch(
        "lib.impl.strategies.monitoring.perf_cycles_process.subprocess.run",
        return_value=fake_completed,
    ):
        result = _collect_perf_counters(
            pid=1234,
            interval=0.1,
            events=["cycles", "ref-cycles", "instructions"],
            logger=MagicMock(),
        )

    assert result == {
        "cycles": 1234.0,
        "ref-cycles": 2345.0,
        "instructions": 3456.0,
    }


def test_collect_container_perf_counters_uses_container_pid():
    fake_completed = MagicMock()
    fake_completed.stderr = """
      11;cycles;cycles;100.00%;
      22;ref-cycles;ref-cycles;100.00%;
    """

    fake_container = MagicMock()
    fake_container.attrs = {"State": {"Pid": 4321}}

    with patch(
        "lib.impl.strategies.monitoring.docker_component.ensure_perf_binary",
        return_value="/usr/bin/perf",
    ), patch(
        "lib.impl.strategies.monitoring.docker_component.perf_privilege_prefix",
        return_value=[],
    ), patch(
        "lib.impl.strategies.monitoring.docker_component.subprocess.run",
        return_value=fake_completed,
    ), patch.object(
        fake_container, "reload", return_value=None
    ):
        result = _collect_container_perf_counters(
            container=fake_container,
            interval=0.1,
            events=["cycles", "ref-cycles"],
            logger=MagicMock(),
        )

    assert result == {"cycles": 11.0, "ref-cycles": 22.0}


def test_container_perf_parser_skips_unsupported_values():
    stderr = """
      <not counted>;cycles;cycles;100.00%;
      44;instructions;instructions;100.00%;
    """

    result = parse_container_perf_stat_output(stderr)

    assert result == {"instructions": 44.0}


def test_collect_container_perf_counters_prepends_privilege_prefix():
    fake_completed = MagicMock()
    fake_completed.stderr = "10;cycles;cycles;100.00%;"

    fake_container = MagicMock()
    fake_container.attrs = {"State": {"Pid": 4321}}

    with patch(
        "lib.impl.strategies.monitoring.docker_component.ensure_perf_binary",
        return_value="/usr/bin/perf",
    ), patch(
        "lib.impl.strategies.monitoring.docker_component.perf_privilege_prefix",
        return_value=["sudo", "-n"],
    ), patch(
        "lib.impl.strategies.monitoring.docker_component.subprocess.run",
        return_value=fake_completed,
    ) as mock_run, patch.object(
        fake_container, "reload", return_value=None
    ):
        _collect_container_perf_counters(
            container=fake_container,
            interval=0.1,
            events=["cycles"],
            logger=MagicMock(),
        )

    command = mock_run.call_args.args[0]
    assert command[:2] == ["sudo", "-n"]
    assert command[2] == "/usr/bin/perf"


def test_collect_container_perf_counters_warns_once_when_empty():
    fake_completed = MagicMock()
    fake_completed.stderr = "<not supported>;cycles;cycles;100.00%;"
    fake_completed.returncode = 0

    fake_container = MagicMock()
    fake_container.attrs = {"State": {"Pid": 4321}}

    docker_component._perf_failure_warned = False
    logger = MagicMock()

    with patch(
        "lib.impl.strategies.monitoring.docker_component.ensure_perf_binary",
        return_value="/usr/bin/perf",
    ), patch(
        "lib.impl.strategies.monitoring.docker_component.perf_privilege_prefix",
        return_value=[],
    ), patch(
        "lib.impl.strategies.monitoring.docker_component.read_perf_event_paranoid",
        return_value="2",
    ), patch(
        "lib.impl.strategies.monitoring.docker_component.subprocess.run",
        return_value=fake_completed,
    ), patch.object(
        fake_container, "reload", return_value=None
    ):
        for _ in range(3):
            result = _collect_container_perf_counters(
                container=fake_container,
                interval=0.1,
                events=["cycles"],
                logger=logger,
            )

    assert result == {}
    assert logger.warning.call_count == 1


def test_perf_privilege_prefix_reflects_sudo_availability():
    with patch(
        "lib.impl.strategies.monitoring.perf_support.passwordless_sudo_available",
        return_value=True,
    ):
        assert perf_support.perf_privilege_prefix() == ["sudo", "-n"]

    with patch(
        "lib.impl.strategies.monitoring.perf_support.passwordless_sudo_available",
        return_value=False,
    ):
        assert perf_support.perf_privilege_prefix() == []
