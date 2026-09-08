#!/usr/bin/env python3
# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

"""Generate v1 fixtures without importing or invoking the Rust codec."""

from hashlib import sha256
from pathlib import Path
from struct import pack


OUT = Path(__file__).with_name("fixtures")
CHECKPOINT_ID = b"app-logs"
PROFILE_DOMAIN = b"otel-arrow-filelog-framing-profile-v1\0"
NAMESPACE_DOMAIN = b"otel-arrow-filelog-checkpoint-namespace-v1\0"
PATH_DOMAIN = b"otel-arrow-filelog-advisory-path-v1\0"
FRONTIER_DOMAIN = b"otel-arrow-filelog-frontier-guard-v1\0"


def u8(value):
    return pack(">B", value)


def u16(value):
    return pack(">H", value)


def u32(value):
    return pack(">I", value)


def u64(value):
    return pack(">Q", value)


def crc32c(data):
    """Compute reflected Castagnoli CRC-32C without a third-party module."""
    crc = 0xFFFFFFFF
    for byte in data:
        crc ^= byte
        for _ in range(8):
            crc = (crc >> 1) ^ (0x82F63B78 if crc & 1 else 0)
    return crc ^ 0xFFFFFFFF


def framed(payload):
    body = u32(len(payload)) + payload
    return body + u32(crc32c(body))


def variable(data):
    return u16(len(data)) + data


def namespace_digest(checkpoint_id=CHECKPOINT_ID):
    return sha256(NAMESPACE_DOMAIN + u16(len(checkpoint_id)) + checkpoint_id).digest()


def path_digest(kind, raw):
    return sha256(PATH_DOMAIN + u8(kind) + u64(len(raw)) + raw).digest()


def advisory(kind, raw):
    if kind == 0:
        raw = b""
    stored = raw[-4096:]
    truncated = len(raw) > 4096
    return (
        u8(kind)
        + u8(1 if truncated else 0)
        + u64(len(raw))
        + u16(len(stored))
        + stored
        + path_digest(kind, raw)
    )


def frontier(offset, raw):
    assert len(raw) == min(offset, 64)
    digest = sha256(FRONTIER_DOMAIN + u16(len(raw)) + raw).digest()
    return u16(len(raw)) + digest


def locator_posix(dev, ino):
    return u8(1) + u64(dev) + u64(ino)


def locator_windows(serial, file_id):
    return u8(2) + u64(serial) + file_id


def resume_clean():
    return u8(0)


def resume_continuation(start, end, index):
    return u8(1) + u64(start) + u64(end) + u32(index)


def canonical_profile(multiline=False):
    mode = 2 if multiline else 0
    regex_version = 1 if multiline else 0
    pattern = b"^END request$" if multiline else b""
    return (
        PROFILE_DOMAIN
        + u16(1)
        + u16(1000)
        + u32(0)
        + u8(1)
        + u8(1)
        + u8(mode)
        + u16(regex_version)
        + variable(pattern)
        + u64(1_048_576)
        + u64(1_048_576)
        + u8(1)
        + u32(500)
        + u64(500)
    )


PROFILE_DIGEST = sha256(canonical_profile()).digest()
UNIX_PATH = advisory(1, b"/var/log/app.log")
EMPTY_PATH = advisory(0, b"")
EMPTY_GUARD = frontier(0, b"")
FOUR_GUARD = frontier(4, b"abc\n")


def current(generation):
    prefix = b"FLOGCUR\0" + u16(1) + u16(0) + u64(generation)
    return prefix + u32(crc32c(prefix))


def record(
    file_id,
    epoch,
    offset,
    guard,
    fingerprint,
    locator,
    resume,
    lifecycle,
    path,
    quarantine=b"",
):
    payload = (
        file_id
        + u32(epoch)
        + u64(offset)
        + guard
        + variable(fingerprint)
        + u32(0)
        + locator
        + u16(1)
        + PROFILE_DIGEST
        + resume
        + u8(lifecycle)
        + quarantine
        + u64(1_700_000_000_000_000_000)
        + path
    )
    return framed(payload)


def snapshot(generation, records):
    header = (
        b"FLOGSNP\0"
        + u16(1)
        + u16(0)
        + u64(generation)
        + namespace_digest()
        + u32(len(records))
    )
    header += u32(crc32c(header))
    record_bytes = b"".join(records)
    footer = b"FLOGSFT\0" + u64(len(record_bytes)) + u32(len(records))
    footer += u32(crc32c(footer))
    return header + record_bytes + footer


def wal_header(generation):
    header = b"FLOGWAL\0" + u16(1) + u16(0) + u64(generation) + namespace_digest()
    return header + u32(crc32c(header))


def operation(code, fields):
    return framed(u8(code) + fields)


def op_register(file_id):
    return operation(
        1,
        file_id
        + u32(1)
        + u64(0)
        + EMPTY_GUARD
        + variable(b"0123456789abcdef")
        + u32(0)
        + locator_posix(2049, 12345)
        + u16(1)
        + PROFILE_DIGEST
        + resume_clean()
        + u64(1_700_000_000_000_000_001)
        + UNIX_PATH,
    )


def op_progress(file_id, continuation=False):
    new_offset = 2 if continuation else 4
    guard = frontier(new_offset, b"ab" if continuation else b"abc\n")
    resume = resume_continuation(0, 3, 1) if continuation else resume_clean()
    return operation(
        2,
        file_id
        + u64(0)
        + u32(1)
        + u64(new_offset)
        + guard
        + resume
        + u64(1_700_000_000_000_000_002)
        + u8(0),
    )


def op_progress_zero_delta_finalize(file_id):
    return operation(
        2,
        file_id
        + u64(4)
        + u32(1)
        + u64(4)
        + FOUR_GUARD
        + resume_clean()
        + u64(1_700_000_000_000_000_007)
        + u8(1),
    )


def op_reset_truncate(file_id):
    return operation(
        3,
        file_id
        + u32(1)
        + u64(0)
        + u32(2)
        + u64(0)
        + resume_clean()
        + variable(b"replacement-stream")
        + u64(1_700_000_000_000_000_003)
        + u16(1),
    )


def op_update_fingerprint(file_id, minimum=False):
    old = b"" if minimum else b"0123456789abcdef"
    new = b"x" if minimum else old + b"g"
    return operation(4, file_id + u32(1) + variable(old) + variable(new))


def op_metadata(file_id):
    return operation(5, file_id + u8(1) + u32(1) + u8(1) + u64(42) + UNIX_PATH)


def op_metadata_without_path(file_id):
    return operation(5, file_id + u8(1) + u32(1) + u8(0) + u64(42))


def op_quarantine(file_id):
    return operation(
        6,
        file_id
        + u32(1)
        + u16(1)
        + locator_posix(2049, 12345)
        + u64(88)
        + u32(1)
        + u64(1_700_000_000_000_000_004),
    )


def op_keep_failed(file_id, mutated=False):
    if mutated:
        epoch = 99
        offset = 9
        guard = frontier(9, b"123456789")
        fingerprint = b"mutated-state"
    else:
        epoch = 1
        offset = 4
        guard = FOUR_GUARD
        fingerprint = b"0123456789abcdef"
    return operation(
        7,
        file_id
        + u32(1)
        + u8(3)
        + u32(epoch)
        + u64(offset)
        + guard
        + resume_clean()
        + variable(fingerprint)
        + u64(1_700_000_000_000_000_005)
        + variable(CHECKPOINT_ID)
        + variable(b"operator confirmed failure"),
    )


def op_reset_quarantined(file_id, action):
    assert action in (1, 2)
    offset = 0 if action == 1 else 4
    guard = EMPTY_GUARD if action == 1 else FOUR_GUARD
    return operation(
        7,
        file_id
        + u32(1)
        + u8(action)
        + u32(2)
        + u64(offset)
        + guard
        + resume_clean()
        + variable(b"replacement-stream")
        + u64(1_700_000_000_000_000_005)
        + variable(CHECKPOINT_ID)
        + variable(b"operator approved reset"),
    )


def op_remove(file_id):
    return operation(
        8,
        file_id
        + u32(1)
        + u8(3)
        + u16(1)
        + u64(1_700_000_000_000_000_006)
        + u8(1)
        + variable(CHECKPOINT_ID)
        + variable(b"retire quarantined file"),
    )


def op_remove_non_administrative(file_id):
    return operation(
        8,
        file_id
        + u32(1)
        + u8(1)
        + u16(1)
        + u64(1_700_000_000_000_000_006)
        + u8(0)
        + variable(b"")
        + variable(b""),
    )


def transaction(sequence, operations):
    body = b"".join(operations)
    header = (
        b"FLOGTXN\0"
        + u16(1)
        + u16(0)
        + u64(sequence)
        + u32(len(body))
        + u32(len(body) ^ 0xFFFFFFFF)
        + u16(len(operations))
        + u16(0)
    )
    header += u32(crc32c(header))
    frame = header + body
    return frame + u32(crc32c(frame))


def write(name, data):
    (OUT / name).write_bytes(data)


def main():
    assert crc32c(b"123456789") == 0xE3069283
    OUT.mkdir(exist_ok=True)
    file_ids = [number.to_bytes(16, "big") for number in range(1, 5000)]

    active = record(
        file_ids[0],
        2,
        4,
        FOUR_GUARD,
        b"0123456789abcdef",
        locator_posix(2049, 12345),
        resume_clean(),
        1,
        UNIX_PATH,
    )
    quarantine = u16(1) + u64(88) + u32(4) + u64(1_700_000_000_000_000_010)
    quarantined = record(
        file_ids[1],
        4,
        4,
        FOUR_GUARD,
        b"fedcba9876543210",
        locator_windows(0x1122334455667788, bytes(range(16))),
        resume_clean(),
        3,
        advisory(2, "C:\\logs\\app.log".encode("utf-16le")),
        quarantine,
    )
    finalized = record(
        file_ids[2],
        1,
        0,
        EMPTY_GUARD,
        b"finalized-stream",
        locator_posix(2049, 12345),
        resume_clean(),
        2,
        EMPTY_PATH,
    )
    long_path = record(
        file_ids[3],
        1,
        0,
        EMPTY_GUARD,
        b"long-path-stream",
        locator_posix(2050, 99999),
        resume_clean(),
        1,
        advisory(1, b"x" * 5000),
    )

    write("current-generation-42.bin", current(42))
    write("snapshot-empty.bin", snapshot(0, []))
    write("snapshot-active.bin", snapshot(7, [active]))
    write("snapshot-quarantined.bin", snapshot(7, [quarantined]))
    write("snapshot-rotated-finalized.bin", snapshot(7, [finalized]))
    write("snapshot-long-path.bin", snapshot(7, [long_path]))
    write("wal-header.bin", wal_header(7))
    write("advisory-unix.bin", UNIX_PATH)
    write("advisory-windows-utf16le.bin", advisory(2, "C:\\logs\\app.log".encode("utf-16le")))
    write("advisory-long-truncated.bin", advisory(1, b"x" * 5000))
    write("frontier-empty.bin", EMPTY_GUARD)
    write("frontier-nonempty.bin", FOUR_GUARD)
    write("framing-profile-default.bin", canonical_profile())
    write("framing-profile-multiline.bin", canonical_profile(True))

    operations = {
        "register-file": op_register(file_ids[10]),
        "update-progress": op_progress(file_ids[11]),
        "reset-after-truncate": op_reset_truncate(file_ids[12]),
        "update-fingerprint": op_update_fingerprint(file_ids[13]),
        "update-metadata": op_metadata(file_ids[14]),
        "quarantine-file": op_quarantine(file_ids[15]),
        "reset-quarantined-keep-failed": op_keep_failed(file_ids[16]),
        "remove-file": op_remove(file_ids[17]),
    }
    for name, encoded in operations.items():
        write(f"operation-{name}.bin", encoded)
        write(f"transaction-{name}.bin", transaction(1, [encoded]))
    zero_delta_finalize = op_progress_zero_delta_finalize(file_ids[18])
    write("operation-update-progress-zero-delta-finalize.bin", zero_delta_finalize)
    write(
        "transaction-update-progress-zero-delta-finalize.bin",
        transaction(1, [zero_delta_finalize]),
    )
    mutated = op_keep_failed(file_ids[16], True)
    write("operation-keep-failed-mutation.bin", mutated)
    write("transaction-keep-failed-mutation.bin", transaction(1, [mutated]))
    reset_beginning = op_reset_quarantined(file_ids[25], 1)
    reset_end = op_reset_quarantined(file_ids[26], 2)
    metadata_without_path = op_metadata_without_path(file_ids[27])
    remove_non_administrative = op_remove_non_administrative(file_ids[28])
    extra_operations = {
        "reset-quarantined-beginning": reset_beginning,
        "reset-quarantined-end": reset_end,
        "update-metadata-without-path": metadata_without_path,
        "remove-file-non-administrative": remove_non_administrative,
    }
    for name, encoded in extra_operations.items():
        write(f"operation-{name}.bin", encoded)
        write(f"transaction-{name}.bin", transaction(1, [encoded]))
    write("transaction-minimum.bin", transaction(1, [op_update_fingerprint(file_ids[20], True)]))
    write(
        "transaction-progress-class.bin",
        transaction(1, [op_progress(file_ids[21]), op_progress(file_ids[22])]),
    )
    write(
        "transaction-non-progress-class.bin",
        transaction(1, [op_metadata(file_ids[23]), op_remove(file_ids[24])]),
    )
    max_progress = [op_progress(file_ids[index + 100], True) for index in range(4096)]
    write("transaction-max-progress.bin", transaction(1, max_progress))

    values = {
        "crc32c_123456789": f"{crc32c(b'123456789'):08x}",
        "namespace_app_logs": namespace_digest().hex(),
        "advisory_unix": path_digest(1, b"/var/log/app.log").hex(),
        "advisory_windows_utf16le": path_digest(
            2, "C:\\logs\\app.log".encode("utf-16le")
        ).hex(),
        "advisory_long": path_digest(1, b"x" * 5000).hex(),
        "frontier_empty": EMPTY_GUARD[2:].hex(),
        "frontier_nonempty": FOUR_GUARD[2:].hex(),
        "framing_profile_default": sha256(canonical_profile()).hexdigest(),
        "framing_profile_multiline": sha256(canonical_profile(True)).hexdigest(),
    }
    text = "".join(f"{key}={value}\n" for key, value in values.items())
    (OUT / "expected-values.txt").write_text(text, encoding="ascii")


if __name__ == "__main__":
    main()
