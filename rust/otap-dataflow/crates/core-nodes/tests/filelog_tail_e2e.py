#!/usr/bin/env python3
# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

"""Black-box checks for the experimental Linux single-file executable."""

import argparse
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import unittest


class FilelogTailE2E(unittest.TestCase):
    executable: Path
    artifacts: Path
    process: subprocess.Popen | None

    def setUp(self):
        self.directory = Path(
            tempfile.mkdtemp(prefix=f"{self._testMethodName}-", dir=self.artifacts)
        )
        self.input = self.directory / "input.log"
        self.output = self.directory / "output.log"
        self.status = self.directory / "eof-offset.txt"
        self.input.write_bytes(b"ready\n")
        self.process = None
        self.stdout = (self.directory / "stdout.log").open("wb")
        self.stderr_path = self.directory / "stderr.log"
        self.stderr = self.stderr_path.open("wb")
        self.addCleanup(self.stdout.close)
        self.addCleanup(self.stderr.close)
        self.addCleanup(self._cleanup_process)

    def _start(self):
        self.process = subprocess.Popen(
            [str(self.executable), str(self.input), str(self.output), str(self.status)],
            stdout=self.stdout,
            stderr=self.stderr,
        )

    def _diagnostic(self):
        return self.stderr_path.read_text(errors="replace")

    def _cleanup_process(self):
        if self.process is not None and self.process.poll() is None:
            self.process.send_signal(signal.SIGINT)
            try:
                self.process.wait(timeout=15)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=15)
                self.fail(f"executable required forced cleanup: {self._diagnostic()}")

    def _append(self, data):
        with self.input.open("ab") as source:
            source.write(data)
            source.flush()
            os.fsync(source.fileno())

    def _wait_for_eof(self, expected, output):
        # These are real child-process/filesystem handshakes, not timer logic.
        # Poll the advertised EOF state, never infer readiness from a sleep.
        deadline = time.monotonic() + 60
        last_status = b""
        while time.monotonic() < deadline:
            code = self.process.poll()
            self.assertIsNone(
                code, f"executable exited with {code}: {self._diagnostic()}"
            )
            try:
                last_status = self.status.read_bytes()
            except FileNotFoundError:
                last_status = b""
            if (
                len(last_status) == 21
                and last_status[-1:] == b"\n"
                and last_status[:20].isdigit()
                and int(last_status[:20]) == expected
            ):
                self.assertEqual(self.output.read_bytes(), output)
                return
            time.sleep(0.02)
        self.fail(
            f"did not observe EOF offset {expected}; last status={last_status!r}; "
            f"stderr={self._diagnostic()}"
        )

    def _stop(self):
        self.process.send_signal(signal.SIGINT)
        try:
            code = self.process.wait(timeout=15)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait(timeout=15)
            self.fail(f"shutdown exceeded its guard: {self._diagnostic()}")
        self.assertEqual(code, 0, self._diagnostic())

    # Scenario: Records and partial records are appended after explicitly observed temporary EOFs.
    # Guarantees: Output preserves exact bytes and order without gaps, duplicates, or premature partial completion.
    def test_live_appends_and_partial_records(self):
        self._start()
        expected = b"ready\n"
        source_length = len(expected)
        self._wait_for_eof(source_length, expected)

        for batch in range(3):
            records = b"".join(
                f"batch-{batch}-record-{index:03d}\n".encode() for index in range(40)
            )
            self._append(records)
            source_length += len(records)
            expected += records
            self._wait_for_eof(source_length, expected)

        # Cross the source-read buffer boundary and preserve non-text payloads.
        records = b"x" * 20000 + b"\n\nraw-\x00-\xff\r\n"
        self._append(records)
        source_length += len(records)
        expected += records
        self._wait_for_eof(source_length, expected)

        prefix = b"split-" + "\u20ac".encode()[:1]
        self._append(prefix)
        source_length += len(prefix)
        self._wait_for_eof(source_length, expected)
        suffix = "\u20ac".encode()[1:] + b"-complete\n"
        self._append(suffix)
        source_length += len(suffix)
        expected += prefix + suffix
        self._wait_for_eof(source_length, expected)

        unfinished = b"not-authorized-to-complete"
        self._append(unfinished)
        self._wait_for_eof(source_length + len(unfinished), expected)
        self._stop()
        self.assertEqual(self.output.read_bytes(), expected)

    # Scenario: The requested output file already contains data.
    # Guarantees: The executable fails without overwriting existing output.
    def test_existing_output_is_not_overwritten(self):
        self.output.write_bytes(b"keep-existing-data")
        self._start()
        self.assertNotEqual(self.process.wait(timeout=30), 0)
        self.assertEqual(self.output.read_bytes(), b"keep-existing-data")
        self.assertTrue(self._diagnostic())

    # Scenario: An input record exceeds the experimental 64 KiB line bound.
    # Guarantees: The executable fails explicitly instead of silently splitting or truncating the record.
    def test_oversized_record_fails(self):
        self.input.write_bytes(b"valid\n" + b"x" * (64 * 1024 + 1) + b"\n")
        self._start()
        self.assertNotEqual(self.process.wait(timeout=30), 0)
        self.assertEqual(self.output.read_bytes(), b"valid\n")
        self.assertIn("64 KiB", self._diagnostic())

    # Scenario: The source is truncated after the reader reports a known EOF.
    # Guarantees: Unsupported truncation is an explicit failure, not silent continuation at an invalid offset.
    def test_truncation_fails(self):
        self._start()
        self._wait_for_eof(len(b"ready\n"), b"ready\n")
        self.input.write_bytes(b"x\n")
        self.assertNotEqual(self.process.wait(timeout=30), 0)
        self.assertEqual(self.output.read_bytes(), b"ready\n")
        self.assertIn("truncation", self._diagnostic())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--executable", type=Path, required=True)
    parser.add_argument("--artifacts", type=Path, required=True)
    arguments = parser.parse_args()
    FilelogTailE2E.executable = arguments.executable.resolve(strict=True)
    FilelogTailE2E.artifacts = arguments.artifacts.resolve()
    FilelogTailE2E.artifacts.mkdir(parents=True, exist_ok=True)
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(FilelogTailE2E)
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    raise SystemExit(0 if result.wasSuccessful() else 1)


if __name__ == "__main__":
    main()
