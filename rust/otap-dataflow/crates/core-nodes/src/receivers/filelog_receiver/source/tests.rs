// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::ffi::CString;
use std::fs;
use std::io::{Seek, SeekFrom};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::os::unix::net::UnixListener;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use nix::fcntl::{FcntlArg, FdFlag, fcntl};
use nix::unistd::mkfifo;
use tempfile::tempdir;

fn open(directory: &File, name: &CStr) -> SourceFile {
    SourceFile::open_at(directory, name, SymlinkPolicy::Reject, None, || false)
        .expect("open regular source")
}

fn descriptors_for(locator: FileLocator) -> usize {
    fs::read_dir("/proc/self/fd")
        .expect("Linux descriptor inventory")
        .filter_map(Result::ok)
        .filter_map(|entry| fs::metadata(entry.path()).ok())
        .filter(|metadata| FileLocator::from_metadata(metadata) == locator)
        .count()
}

/// Scenario: A regular file is opened relative to a directory handle.
/// Guarantees: The descriptor is read-only, nonblocking and close-on-exec, with handle-derived evidence.
#[test]
fn regular_file_flags_and_metadata() {
    let dir = tempdir().expect("create test directory");
    fs::write(dir.path().join("log"), b"hello").expect("fixture operation succeeds");
    let parent = File::open(dir.path()).expect("fixture operation succeeds");
    let source = open(&parent, c"log");
    let flags = fcntl(&source.file, FcntlArg::F_GETFL).expect("fixture operation succeeds");
    assert_eq!(flags & libc::O_ACCMODE, libc::O_RDONLY);
    assert_ne!(flags & libc::O_NONBLOCK, 0);
    let fd_flags = FdFlag::from_bits_truncate(
        fcntl(&source.file, FcntlArg::F_GETFD).expect("fixture operation succeeds"),
    );
    assert!(fd_flags.contains(FdFlag::FD_CLOEXEC));
    assert_eq!(source.opened_metadata().len(), 5);
    assert_eq!(
        source.locator(),
        FileLocator::from_metadata(&source.file.metadata().expect("fixture operation succeeds"))
    );
    assert!(source.opened_metadata().is_file());
}

/// Scenario: A caller supplies empty, dot, parent, absolute or multi-component names.
/// Guarantees: Opens cannot resolve ancestor components through the supplied filename.
#[test]
fn invalid_names_are_rejected() {
    let dir = tempdir().expect("create test directory");
    let parent = File::open(dir.path()).expect("fixture operation succeeds");
    for name in [c"", c".", c"..", c"a/b", c"/tmp", c"../log", c"log/"] {
        assert!(matches!(
            SourceFile::open_at(&parent, name, SymlinkPolicy::Follow, None, || false),
            Err(FileAccessError::InvalidName)
        ));
    }
}

/// Scenario: A source filename contains a byte that is not valid UTF-8.
/// Guarantees: Native filename bytes reach openat unchanged, without lossy conversion.
#[test]
fn native_filename_bytes_are_preserved() {
    let dir = tempdir().expect("create test directory");
    let name = CString::new(b"log-\xff".to_vec()).expect("fixture operation succeeds");
    fs::write(
        dir.path()
            .join(std::ffi::OsStr::from_bytes(name.to_bytes())),
        b"native",
    )
    .expect("fixture operation succeeds");
    let parent = File::open(dir.path()).expect("fixture operation succeeds");
    let source = open(&parent, &name);
    let mut buf = [0; 6];
    assert_eq!(
        source
            .read_at(0, &mut buf, || false)
            .expect("fixture operation succeeds"),
        6
    );
    assert_eq!(&buf, b"native");
}

/// Scenario: A final symlink names a regular file, or is dangling or cyclic.
/// Guarantees: Reject never follows a final link; Follow validates the target and preserves OS failures.
#[test]
fn final_symlink_policy_is_explicit() {
    let dir = tempdir().expect("create test directory");
    fs::write(dir.path().join("log"), b"target").expect("fixture operation succeeds");
    symlink("log", dir.path().join("alias")).expect("fixture operation succeeds");
    symlink("missing", dir.path().join("dangling")).expect("fixture operation succeeds");
    symlink("cycle", dir.path().join("cycle")).expect("fixture operation succeeds");
    let parent = File::open(dir.path()).expect("fixture operation succeeds");
    assert!(matches!(
        SourceFile::open_at(&parent, c"alias", SymlinkPolicy::Reject, None, || false),
        Err(FileAccessError::NotRegular)
    ));
    let target = open(&parent, c"log");
    let followed = SourceFile::open_at(
        &parent,
        c"alias",
        SymlinkPolicy::Follow,
        Some(target.locator()),
        || false,
    )
    .expect("fixture operation succeeds");
    assert_eq!(followed.locator(), target.locator());
    for (name, errno) in [(c"dangling", libc::ENOENT), (c"cycle", libc::ELOOP)] {
        assert!(
            matches!(SourceFile::open_at(&parent, name, SymlinkPolicy::Follow, None, || false),
            Err(FileAccessError::Io(error)) if error.raw_os_error() == Some(errno))
        );
    }
}

/// Scenario: The selected pathname is rebound to another inode before the open.
/// Guarantees: An expected locator rejects replacement and closes the rejected descriptor.
#[test]
fn selected_locator_rejects_replacement() {
    let dir = tempdir().expect("create test directory");
    fs::write(dir.path().join("log"), b"old").expect("fixture operation succeeds");
    let parent = File::open(dir.path()).expect("fixture operation succeeds");
    let original = open(&parent, c"log");
    fs::rename(dir.path().join("log"), dir.path().join("old")).expect("fixture operation succeeds");
    fs::write(dir.path().join("log"), b"new").expect("fixture operation succeeds");
    let replacement = FileLocator::from_metadata(
        &fs::metadata(dir.path().join("log")).expect("fixture operation succeeds"),
    );
    assert_ne!(original.locator(), replacement);
    assert!(
        matches!(SourceFile::open_at(&parent, c"log", SymlinkPolicy::Reject, Some(original.locator()), || false),
        Err(FileAccessError::LocatorChanged { actual })
            if actual == replacement)
    );
    assert_eq!(descriptors_for(replacement), 0);
}

/// Scenario: The source name changes between open and the metadata query.
/// Guarantees: Acceptance evidence and reads refer to the opened object, not replacement path metadata.
#[test]
fn substitution_after_open_cannot_change_evidence() {
    let dir = tempdir().expect("create test directory");
    fs::write(dir.path().join("log"), b"old").expect("fixture operation succeeds");
    let expected = FileLocator::from_metadata(
        &fs::metadata(dir.path().join("log")).expect("fixture operation succeeds"),
    );
    let parent = File::open(dir.path()).expect("fixture operation succeeds");
    let mut calls = 0;
    let source = SourceFile::open_at(
        &parent,
        c"log",
        SymlinkPolicy::Reject,
        Some(expected),
        || {
            calls += 1;
            if calls == 2 {
                fs::rename(dir.path().join("log"), dir.path().join("old"))
                    .expect("fixture operation succeeds");
                fs::write(dir.path().join("log"), b"replacement")
                    .expect("fixture operation succeeds");
            }
            false
        },
    )
    .expect("fixture operation succeeds");
    assert_eq!(source.locator(), expected);
    assert_eq!(source.opened_metadata().len(), 3);
    let mut buf = [0; 3];
    assert_eq!(
        source
            .read_at(0, &mut buf, || false)
            .expect("fixture operation succeeds"),
        3
    );
    assert_eq!(&buf, b"old");
}

/// Scenario: The directory pathname is renamed and replaced after its handle was selected.
/// Guarantees: Descriptor-relative opens remain anchored to the selected directory object.
#[test]
fn directory_handle_survives_path_replacement() {
    let dir = tempdir().expect("create test directory");
    let current = dir.path().join("current");
    fs::create_dir(&current).expect("fixture operation succeeds");
    fs::write(current.join("log"), b"old").expect("fixture operation succeeds");
    let parent = File::open(&current).expect("fixture operation succeeds");
    fs::rename(&current, dir.path().join("moved")).expect("fixture operation succeeds");
    fs::create_dir(&current).expect("fixture operation succeeds");
    fs::write(current.join("log"), b"new").expect("fixture operation succeeds");
    let source = open(&parent, c"log");
    let mut buf = [0; 3];
    assert_eq!(
        source
            .read_at(0, &mut buf, || false)
            .expect("fixture operation succeeds"),
        3
    );
    assert_eq!(&buf, b"old");
}

/// Scenario: A writerless FIFO is offered as a source candidate.
/// Guarantees: The probe rejects it without waiting for a writer; a child-process deadline detects blocking regressions.
#[test]
fn fifo_without_writer_is_rejected() {
    const CHILD: &str = "OTAP_FILELOG_FIFO_TEST_DIRECTORY";
    if let Some(path) = std::env::var_os(CHILD) {
        let parent = File::open(path).expect("fixture operation succeeds");
        assert!(matches!(
            SourceFile::open_at(&parent, c"fifo", SymlinkPolicy::Reject, None, || false),
            Err(FileAccessError::NotRegular)
        ));
        return;
    }
    let dir = tempdir().expect("create test directory");
    mkfifo(&dir.path().join("fifo"), Mode::S_IRUSR | Mode::S_IWUSR)
        .expect("fixture operation succeeds");
    let mut child = Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "receivers::filelog_receiver::source::tests::fifo_without_writer_is_rejected",
        ])
        .env(CHILD, dir.path())
        .stdout(Stdio::null())
        .spawn()
        .expect("fixture operation succeeds");
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Some(status) = child.try_wait().expect("poll FIFO probe child") {
            assert!(status.success());
            break;
        }
        if Instant::now() >= deadline {
            child.kill().expect("kill blocked FIFO probe");
            let _ = child.wait().expect("reap FIFO probe child");
            panic!("source probe blocked on a writerless FIFO");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

/// Scenario: Directory, socket, device and followed non-regular targets are probed.
/// Guarantees: No non-regular object becomes readable through SourceFile and failed probes release handles.
#[test]
fn non_regular_candidates_are_rejected() {
    let dir = tempdir().expect("create test directory");
    let parent = File::open(dir.path()).expect("fixture operation succeeds");
    fs::create_dir(dir.path().join("directory")).expect("fixture operation succeeds");
    let _listener =
        UnixListener::bind(dir.path().join("socket")).expect("fixture operation succeeds");
    symlink("directory", dir.path().join("alias")).expect("fixture operation succeeds");
    for name in [c"directory", c"alias"] {
        let locator = FileLocator::from_metadata(
            &fs::metadata(
                dir.path()
                    .join(std::ffi::OsStr::from_bytes(name.to_bytes())),
            )
            .expect("fixture operation succeeds"),
        );
        assert!(matches!(
            SourceFile::open_at(&parent, name, SymlinkPolicy::Follow, None, || false),
            Err(FileAccessError::NotRegular)
        ));
        assert_eq!(descriptors_for(locator), 0);
    }
    assert!(matches!(
        SourceFile::open_at(&parent, c"socket", SymlinkPolicy::Reject, None, || false),
        Err(FileAccessError::NotRegular)
    ));
    let devices = File::open("/dev").expect("fixture operation succeeds");
    assert!(matches!(
        SourceFile::open_at(&devices, c"null", SymlinkPolicy::Reject, None, || false),
        Err(FileAccessError::NotRegular)
    ));
}

/// Scenario: An opened file is renamed, unlinked and its original name is reused.
/// Guarantees: Resident reads continue from the old inode until drop closes its descriptor.
#[test]
fn resident_handle_survives_rename_and_unlink() {
    let dir = tempdir().expect("create test directory");
    fs::write(dir.path().join("log"), b"old").expect("fixture operation succeeds");
    let parent = File::open(dir.path()).expect("fixture operation succeeds");
    let source = open(&parent, c"log");
    let locator = source.locator();
    fs::rename(dir.path().join("log"), dir.path().join("old")).expect("fixture operation succeeds");
    fs::remove_file(dir.path().join("old")).expect("fixture operation succeeds");
    fs::write(dir.path().join("log"), b"replacement").expect("fixture operation succeeds");
    let mut buf = [0; 3];
    assert_eq!(
        source
            .read_at(0, &mut buf, || false)
            .expect("fixture operation succeeds"),
        3
    );
    assert_eq!(&buf, b"old");
    assert_eq!(
        FileLocator::from_metadata(
            &source
                .metadata(|| false)
                .expect("fixture operation succeeds")
        ),
        locator
    );
    assert_eq!(descriptors_for(locator), 1);
    drop(source);
    assert_eq!(descriptors_for(locator), 0);
}

/// Scenario: Reads use independent offsets across short input, temporary EOF, append and truncation.
/// Guarantees: Reads preserve source bytes and cursor independence; EOF and metadata are not cached as final state.
#[test]
fn positional_reads_observe_append_and_truncate() {
    let dir = tempdir().expect("create test directory");
    let path = dir.path().join("log");
    fs::write(&path, b"abc").expect("fixture operation succeeds");
    let parent = File::open(dir.path()).expect("fixture operation succeeds");
    let source = open(&parent, c"log");
    let mut shared = source
        .file
        .try_clone()
        .expect("duplicate source descriptor");
    assert_eq!(
        shared.seek(SeekFrom::Start(2)).expect("set shared cursor"),
        2
    );
    let mut buf = [0xaa; 8];
    assert_eq!(
        source
            .read_at(1, &mut buf, || false)
            .expect("fixture operation succeeds"),
        2
    );
    assert_eq!(&buf[..2], b"bc");
    assert_eq!(&buf[2..], &[0xaa; 6]);
    assert_eq!(
        source
            .read_at(3, &mut buf, || false)
            .expect("fixture operation succeeds"),
        0
    );
    let writer = fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .expect("fixture operation succeeds");
    assert_eq!(
        writer
            .write_at(b"def", 3)
            .expect("fixture operation succeeds"),
        3
    );
    assert_eq!(
        source
            .read_at(3, &mut buf, || false)
            .expect("fixture operation succeeds"),
        3
    );
    assert_eq!(&buf[..3], b"def");
    assert_eq!(source.opened_metadata().len(), 3);
    assert_eq!(
        source
            .metadata(|| false)
            .expect("fixture operation succeeds")
            .len(),
        6
    );
    writer.set_len(1).expect("fixture operation succeeds");
    assert_eq!(
        source
            .read_at(3, &mut buf, || false)
            .expect("fixture operation succeeds"),
        0
    );
    assert_eq!(
        source
            .read_at(0, &mut buf, || false)
            .expect("fixture operation succeeds"),
        1
    );
    assert_eq!(buf[0], b'a');
    assert_eq!(
        source
            .metadata(|| false)
            .expect("fixture operation succeeds")
            .len(),
        1
    );
    assert_eq!(shared.stream_position().expect("query shared cursor"), 2);
}

/// Scenario: Empty reads and unrepresentable offsets are requested.
/// Guarantees: Empty reads do not imply permanent EOF, and invalid ranges cannot wrap into valid file offsets.
#[test]
fn empty_and_overflowing_read_ranges() {
    let dir = tempdir().expect("create test directory");
    fs::write(dir.path().join("log"), b"x").expect("fixture operation succeeds");
    let source = open(
        &File::open(dir.path()).expect("fixture operation succeeds"),
        c"log",
    );
    assert_eq!(
        source
            .read_at(i64::MAX as u64, &mut [], || false)
            .expect("fixture operation succeeds"),
        0
    );
    assert!(matches!(
        source.read_at(i64::MAX as u64, &mut [0; 1], || false),
        Err(FileAccessError::InvalidRange)
    ));
    let mut buf = [0xaa; 2];
    for offset in [i64::MAX as u64, u64::MAX] {
        assert!(matches!(
            source.read_at(offset, &mut buf, || false),
            Err(FileAccessError::InvalidRange)
        ));
        assert_eq!(buf, [0xaa; 2]);
    }
    assert_eq!(
        source
            .read_at(0, &mut [], || false)
            .expect("fixture operation succeeds"),
        0
    );
    assert_eq!(
        source
            .read_at(0, &mut buf, || false)
            .expect("fixture operation succeeds"),
        1
    );
}

/// Scenario: Cancellation occurs before open or between open and metadata.
/// Guarantees: No candidate escapes cancellation and every temporarily acquired descriptor is closed.
#[test]
fn cancelled_open_releases_descriptor() {
    let dir = tempdir().expect("create test directory");
    fs::write(dir.path().join("log"), b"x").expect("fixture operation succeeds");
    let parent = File::open(dir.path()).expect("fixture operation succeeds");
    assert!(matches!(
        SourceFile::open_at(&parent, c"missing", SymlinkPolicy::Reject, None, || true),
        Err(FileAccessError::Cancelled)
    ));
    let locator = FileLocator::from_metadata(
        &fs::metadata(dir.path().join("log")).expect("fixture operation succeeds"),
    );
    let mut calls = 0;
    assert!(matches!(
        SourceFile::open_at(&parent, c"log", SymlinkPolicy::Reject, None, || {
            calls += 1;
            if calls == 2 {
                assert_eq!(descriptors_for(locator), 1);
            }
            calls == 2
        }),
        Err(FileAccessError::Cancelled)
    ));
    assert_eq!(calls, 2);
    assert_eq!(descriptors_for(locator), 0);
}

/// Scenario: A read or metadata operation is cancelled before it starts; a later read succeeds.
/// Guarantees: Cancellation preserves the buffer and handle, and a completed read is not hidden by a second cancellation check.
#[test]
fn cancelled_operations_preserve_source_state() {
    let dir = tempdir().expect("create test directory");
    fs::write(dir.path().join("log"), b"data").expect("fixture operation succeeds");
    let source = open(
        &File::open(dir.path()).expect("fixture operation succeeds"),
        c"log",
    );
    let mut buf = [0xaa; 4];
    assert!(matches!(
        source.read_at(0, &mut buf, || true),
        Err(FileAccessError::Cancelled)
    ));
    assert_eq!(buf, [0xaa; 4]);
    assert!(matches!(
        source.metadata(|| true),
        Err(FileAccessError::Cancelled)
    ));
    let mut calls = 0;
    assert_eq!(
        source
            .read_at(0, &mut buf, || {
                calls += 1;
                calls > 1
            })
            .expect("fixture operation succeeds"),
        4
    );
    assert_eq!(calls, 1);
    assert_eq!(&buf, b"data");
}

/// Scenario: The source does not exist or a non-directory is supplied as the anchor.
/// Guarantees: Original OS errors remain available for caller-owned environmental retry policy.
#[test]
fn operating_system_errors_are_preserved() {
    let dir = tempdir().expect("create test directory");
    let parent = File::open(dir.path()).expect("fixture operation succeeds");
    assert!(
        matches!(SourceFile::open_at(&parent, c"missing", SymlinkPolicy::Reject, None, || false),
        Err(FileAccessError::Io(error)) if error.kind() == io::ErrorKind::NotFound)
    );
    fs::write(dir.path().join("log"), b"x").expect("fixture operation succeeds");
    let ordinary_file = File::open(dir.path().join("log")).expect("fixture operation succeeds");
    assert!(
        matches!(SourceFile::open_at(&ordinary_file, c"child", SymlinkPolicy::Reject, None, || false),
        Err(FileAccessError::Io(error)) if error.raw_os_error() == Some(libc::ENOTDIR))
    );
}

/// Scenario: A sparse source contains data beyond the 32-bit offset domain.
/// Guarantees: Positional reads preserve the full source offset without wrapping or seeking.
#[test]
fn large_source_offsets_are_preserved() {
    let dir = tempdir().expect("create test directory");
    let path = dir.path().join("large");
    let writer = File::create(&path).expect("create sparse source");
    let offset = (1_u64 << 32) + 7;
    assert_eq!(
        writer
            .write_at(b"large", offset)
            .expect("write sparse tail"),
        5
    );
    let parent = File::open(dir.path()).expect("open test directory");
    let source = open(&parent, c"large");
    assert_eq!(source.opened_metadata().len(), offset + 5);
    let mut buf = [0; 5];
    assert_eq!(
        source
            .read_at(offset, &mut buf, || false)
            .expect("read sparse tail"),
        5
    );
    assert_eq!(&buf, b"large");
    assert_eq!(
        source
            .read_at(7, &mut buf, || false)
            .expect("read sparse hole"),
        5
    );
    assert_eq!(buf, [0; 5]);
}

/// Scenario: A FIFO is pinned and the callback tries to connect a nonblocking writer while the pin is live.
/// Guarantees: The probe never registers a reader, unlike a normal O_RDONLY FIFO open even if later rejected.
#[test]
fn pinned_fifo_never_registers_a_reader() {
    let dir = tempdir().expect("create test directory");
    mkfifo(&dir.path().join("fifo"), Mode::S_IRUSR | Mode::S_IWUSR).expect("create FIFO");
    let parent = File::open(dir.path()).expect("open directory");
    let mut calls = 0;
    let result = SourceFile::open_at(&parent, c"fifo", SymlinkPolicy::Reject, None, || {
        calls += 1;
        if calls == 2 {
            let writer = openat(
                &parent,
                c"fifo",
                OFlag::O_WRONLY | OFlag::O_NONBLOCK | OFlag::O_CLOEXEC,
                Mode::empty(),
            );
            assert!(
                matches!(writer, Err(nix::errno::Errno::ENXIO)),
                "an O_PATH pin must not register a FIFO reader"
            );
        }
        false
    });
    assert!(matches!(result, Err(FileAccessError::NotRegular)));
    assert_eq!(calls, 2);
}

/// Scenario: A followed symlink names /dev/null.
/// Guarantees: The device is rejected at the pin stage before the read-reopen operation is invoked.
#[test]
fn followed_device_is_rejected_before_read_open() {
    let dir = tempdir().expect("create test directory");
    symlink("/dev/null", dir.path().join("device")).expect("create device symlink");
    let parent = File::open(dir.path()).expect("open directory");
    let result = SourceFile::open_at_with_reopen(
        &parent,
        c"device",
        SymlinkPolicy::Follow,
        None,
        || false,
        |_| panic!("a non-regular pin must never be reopened for reading"),
    );
    assert!(matches!(result, Err(FileAccessError::NotRegular)));
}

/// Scenario: Two regular-file names are hardlinks to the same inode under Reject policy.
/// Guarantees: Handle-derived locators agree without treating hardlinks as symbolic links.
#[test]
fn hardlinks_share_a_locator() {
    let dir = tempdir().expect("create test directory");
    fs::write(dir.path().join("log"), b"data").expect("write source");
    fs::hard_link(dir.path().join("log"), dir.path().join("alias")).expect("create hardlink");
    let parent = File::open(dir.path()).expect("open directory");
    let first = open(&parent, c"log");
    let second = SourceFile::open_at(
        &parent,
        c"alias",
        SymlinkPolicy::Reject,
        Some(first.locator()),
        || false,
    )
    .expect("open hardlink");
    assert_eq!(first.locator(), second.locator());
}

/// Scenario: A stale expected locator is supplied for an otherwise valid regular source.
/// Guarantees: Selection mismatches are rejected before a normal read-open can happen.
#[test]
fn locator_mismatch_precedes_read_reopen() {
    let dir = tempdir().expect("create test directory");
    fs::write(dir.path().join("log"), b"data").expect("write source");
    let parent = File::open(dir.path()).expect("open directory");
    let actual =
        FileLocator::from_metadata(&fs::metadata(dir.path().join("log")).expect("source metadata"));
    let expected = FileLocator {
        device: actual.device,
        inode: actual.inode ^ 1,
    };
    let result = SourceFile::open_at_with_reopen(
        &parent,
        c"log",
        SymlinkPolicy::Reject,
        Some(expected),
        || false,
        |_| panic!("a mismatched pin must not be reopened"),
    );
    assert!(
        matches!(result, Err(FileAccessError::LocatorChanged { actual: found }) if found == actual)
    );
    assert_eq!(descriptors_for(actual), 0);
}

/// Scenario: Reopening the pinned file fails with missing-procfs, permission or descriptor-pressure errors.
/// Guarantees: The distinct reopen error retains the OS errno, closes the pin, and never falls back to the readable pathname.
#[test]
fn pinned_reopen_failure_preserves_errno_without_fallback() {
    let dir = tempdir().expect("create test directory");
    fs::write(dir.path().join("log"), b"data").expect("write source");
    let parent = File::open(dir.path()).expect("open directory");
    let locator =
        FileLocator::from_metadata(&fs::metadata(dir.path().join("log")).expect("source metadata"));
    for errno in [libc::ENOENT, libc::EACCES, libc::EMFILE] {
        let result = SourceFile::open_at_with_reopen(
            &parent,
            c"log",
            SymlinkPolicy::Reject,
            None,
            || false,
            |_| Err(io::Error::from_raw_os_error(errno)),
        );
        assert!(
            matches!(result, Err(FileAccessError::PinnedReopen(source)) if source.raw_os_error() == Some(errno))
        );
        assert_eq!(descriptors_for(locator), 0);
    }
}

/// Scenario: The reopen operation unexpectedly returns a different regular file or a directory.
/// Guarantees: Fresh handle validation rejects both locator and type mismatches and closes every acquired descriptor.
#[test]
fn reopened_handle_is_revalidated() {
    let dir = tempdir().expect("create test directory");
    fs::write(dir.path().join("log"), b"data").expect("write source");
    fs::write(dir.path().join("other"), b"other").expect("write replacement");
    fs::create_dir(dir.path().join("directory")).expect("create non-regular replacement");
    let parent = File::open(dir.path()).expect("open directory");
    let source_locator =
        FileLocator::from_metadata(&fs::metadata(dir.path().join("log")).expect("source metadata"));
    for name in ["other", "directory"] {
        let path = dir.path().join(name);
        let actual =
            FileLocator::from_metadata(&fs::metadata(&path).expect("replacement metadata"));
        let result = SourceFile::open_at_with_reopen(
            &parent,
            c"log",
            SymlinkPolicy::Reject,
            None,
            || false,
            |_| File::open(&path),
        );
        assert!(
            matches!(result, Err(FileAccessError::ReopenedMismatch { actual: found }) if found == actual)
        );
        assert_eq!(descriptors_for(source_locator), 0);
        assert_eq!(descriptors_for(actual), 0);
    }
}

/// Scenario: The source is unlinked and its name reused between pinning and the procfs reopen.
/// Guarantees: The reopened descriptor reads the pinned inode, not the replacement pathname.
#[test]
fn procfs_reopen_survives_unlink_and_name_reuse() {
    let dir = tempdir().expect("create test directory");
    let path = dir.path().join("log");
    fs::write(&path, b"old").expect("write source");
    let parent = File::open(dir.path()).expect("open directory");
    let mut calls = 0;
    let source = SourceFile::open_at(&parent, c"log", SymlinkPolicy::Reject, None, || {
        calls += 1;
        if calls == 3 {
            fs::remove_file(&path).expect("unlink pinned source");
            fs::write(&path, b"replacement").expect("reuse source name");
        }
        false
    })
    .expect("reopen pinned inode");
    assert_eq!(source.opened_metadata().len(), 3);
    let mut buf = [0; 3];
    assert_eq!(
        source
            .read_at(0, &mut buf, || false)
            .expect("read pinned file"),
        3
    );
    assert_eq!(&buf, b"old");
}

/// Scenario: File length changes after pin metadata was captured but before the read handle's metadata query.
/// Guarantees: The opening snapshot comes from the final read handle rather than the earlier pin observation.
#[test]
fn opening_snapshot_uses_fresh_read_handle_metadata() {
    let dir = tempdir().expect("create test directory");
    let path = dir.path().join("log");
    fs::write(&path, b"old").expect("write source");
    let parent = File::open(dir.path()).expect("open directory");
    let mut calls = 0;
    let source = SourceFile::open_at(&parent, c"log", SymlinkPolicy::Reject, None, || {
        calls += 1;
        if calls == 5 {
            fs::write(&path, b"longer content").expect("change file length");
        }
        false
    })
    .expect("open changed source");
    assert_eq!(source.opened_metadata().len(), 14);
}

/// Scenario: Cancellation occurs before the read reopen or while both pin and read descriptors are held.
/// Guarantees: Conversion peaks at two descriptors, and cancellation releases both while preserving the borrowed directory.
#[test]
fn cancellation_releases_conversion_descriptors() {
    let dir = tempdir().expect("create test directory");
    fs::write(dir.path().join("log"), b"data").expect("write source");
    let parent = File::open(dir.path()).expect("open directory");
    let locator =
        FileLocator::from_metadata(&fs::metadata(dir.path().join("log")).expect("source metadata"));
    for cancel_at in [3, 4, 5] {
        let mut calls = 0;
        let result = SourceFile::open_at(&parent, c"log", SymlinkPolicy::Reject, None, || {
            calls += 1;
            assert_eq!(
                descriptors_for(locator),
                match calls {
                    1 => 0,
                    2..=4 => 1,
                    5 => 2,
                    _ => panic!("unexpected I/O stage"),
                }
            );
            calls == cancel_at
        });
        assert!(matches!(result, Err(FileAccessError::Cancelled)));
        assert_eq!(descriptors_for(locator), 0);
    }
    let source = open(&parent, c"log");
    assert_eq!(descriptors_for(locator), 1);
    drop(source);
    assert_eq!(descriptors_for(locator), 0);
}

/// Scenario: A followed log name resolves to the process environment in procfs.
/// Guarantees: Filesystem eligibility rejects the pinned regular file before any read-open; environment contents are never read.
#[test]
fn followed_procfs_file_is_rejected_before_reopen() {
    let dir = tempdir().expect("create fixture directory");
    symlink("/proc/self/environ", dir.path().join("log")).expect("create procfs alias");
    let parent = File::open(dir.path()).expect("open fixture directory");
    let result = SourceFile::open_at_with_reopen(
        &parent,
        c"log",
        SymlinkPolicy::Follow,
        None,
        || false,
        |_| panic!("procfs source must not be reopened for reading"),
    );
    assert!(matches!(
        result,
        Err(FileAccessError::UnsupportedFilesystem {
            filesystem: "procfs"
        })
    ));
}

/// Scenario: Procfs is selected directly, without following the final entry.
/// Guarantees: Kernel-control filesystem rejection is independent of symlink policy and candidate path spelling.
#[test]
fn direct_procfs_file_is_rejected_before_reopen() {
    let parent = File::open("/proc/self").expect("open own proc directory");
    for policy in [SymlinkPolicy::Reject, SymlinkPolicy::Follow] {
        let result = SourceFile::open_at_with_reopen(
            &parent,
            c"environ",
            policy,
            None,
            || false,
            |_| panic!("direct procfs source must not be reopened"),
        );
        assert!(matches!(
            result,
            Err(FileAccessError::UnsupportedFilesystem {
                filesystem: "procfs"
            })
        ));
    }
}

/// Scenario: Filesystem observations cover each forbidden family and ordinary log-storage filesystems.
/// Guarantees: The fixed rejection policy covers both cgroup versions while preserving tmpfs, disk and overlay eligibility.
#[test]
fn kernel_control_filesystem_policy_is_explicit() {
    use nix::sys::statfs::{
        BTRFS_SUPER_MAGIC, EXT4_SUPER_MAGIC, OVERLAYFS_SUPER_MAGIC, TMPFS_MAGIC, XFS_SUPER_MAGIC,
    };
    for (kind, expected) in [
        (PROC_SUPER_MAGIC, "procfs"),
        (SYSFS_MAGIC, "sysfs"),
        (DEBUGFS_MAGIC, "debugfs"),
        (TRACEFS_MAGIC, "tracefs"),
        (SECURITYFS_MAGIC, "securityfs"),
        (CGROUP_SUPER_MAGIC, "cgroup"),
        (CGROUP2_SUPER_MAGIC, "cgroup2"),
    ] {
        assert!(matches!(reject_kernel_control_filesystem(kind),
            Err(FileAccessError::UnsupportedFilesystem { filesystem }) if filesystem == expected));
    }
    for kind in [
        TMPFS_MAGIC,
        EXT4_SUPER_MAGIC,
        XFS_SUPER_MAGIC,
        BTRFS_SUPER_MAGIC,
        OVERLAYFS_SUPER_MAGIC,
    ] {
        reject_kernel_control_filesystem(kind)
            .expect("ordinary storage is not a kernel-control source");
    }
}

/// Scenario: Identical OS failures originate at the pin stage or the procfs read-reopen stage.
/// Guarantees: Callers classify descriptor pressure, WouldBlock, interruption and permission failures without matching the I/O phase.
#[test]
fn os_error_accessor_preserves_io_classification() {
    for errno in [
        libc::EMFILE,
        libc::ENFILE,
        libc::EAGAIN,
        libc::EACCES,
        libc::EINTR,
        libc::ENOENT,
    ] {
        for error in [
            FileAccessError::Io(io::Error::from_raw_os_error(errno)),
            FileAccessError::PinnedReopen(io::Error::from_raw_os_error(errno)),
        ] {
            let underlying = error.os_error().expect("underlying OS error");
            assert_eq!(underlying.raw_os_error(), Some(errno));
            assert_eq!(
                underlying.kind(),
                io::Error::from_raw_os_error(errno).kind()
            );
        }
    }
    let actual = FileLocator {
        device: 1,
        inode: 2,
    };
    for error in [
        FileAccessError::Cancelled,
        FileAccessError::InvalidName,
        FileAccessError::NotRegular,
        FileAccessError::InvalidRange,
        FileAccessError::LocatorChanged { actual },
        FileAccessError::ReopenedMismatch { actual },
        FileAccessError::UnsupportedFilesystem {
            filesystem: "procfs",
        },
    ] {
        assert!(error.os_error().is_none());
    }
}

/// Scenario: A mode-000 regular file is accessible to O_PATH but unreadable under the runner's credentials.
/// Guarantees: Read permission is enforced at procfs reopening and EACCES is preserved; runners with DAC override skip this case.
#[test]
fn read_permission_is_checked_at_procfs_reopen() {
    let dir = tempdir().expect("create fixture directory");
    let path = dir.path().join("private");
    fs::write(&path, b"fixture").expect("create permission fixture");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000))
        .expect("remove access permissions");
    // Detect actual DAC override rather than assuming only UID 0 can bypass it.
    match File::open(&path) {
        Ok(_) => return,
        Err(error) => assert_eq!(error.raw_os_error(), Some(libc::EACCES)),
    }
    let parent = File::open(dir.path()).expect("open fixture directory");
    let locator =
        FileLocator::from_metadata(&fs::metadata(&path).expect("query permission fixture"));
    let result = SourceFile::open_at(&parent, c"private", SymlinkPolicy::Reject, None, || false);
    assert!(
        matches!(result, Err(FileAccessError::PinnedReopen(error)) if error.raw_os_error() == Some(libc::EACCES))
    );
    assert_eq!(descriptors_for(locator), 0);
}

/// Scenario: Descriptor formatting crosses decimal-width boundaries and reuses a nonzero buffer.
/// Guarantees: Procfs paths are exact and NUL-terminated, with no stale digits or prefix underflow.
#[test]
fn procfs_descriptor_paths_cover_decimal_boundaries() {
    let mut buffer = [0xff; 32];
    for (fd, expected) in [
        (0, c"/proc/self/fd/0"),
        (9, c"/proc/self/fd/9"),
        (10, c"/proc/self/fd/10"),
        (i32::MAX as u32, c"/proc/self/fd/2147483647"),
        (u32::MAX, c"/proc/self/fd/4294967295"),
        (0, c"/proc/self/fd/0"),
    ] {
        assert_eq!(proc_fd_path(fd, &mut buffer), expected);
    }
}

/// Scenario: A Linux qualification host provides writable tmpfs storage at /dev/shm.
/// Guarantees: Real regular files on tmpfs survive the filesystem check and the procfs transport reopen.
#[test]
#[ignore = "requires writable tmpfs at /dev/shm; run in Linux qualification"]
fn tmpfs_log_files_remain_readable() {
    use nix::sys::statfs::TMPFS_MAGIC;
    let dir = tempfile::tempdir_in("/dev/shm").expect("writable tmpfs qualification fixture");
    let parent = File::open(dir.path()).expect("open tmpfs fixture directory");
    assert_eq!(
        fstatfs(&parent).expect("query tmpfs").filesystem_type(),
        TMPFS_MAGIC
    );
    fs::write(dir.path().join("log"), b"tmpfs log").expect("write tmpfs log");
    let source = open(&parent, c"log");
    let mut buffer = [0; 9];
    assert_eq!(
        source
            .read_at(0, &mut buffer, || false)
            .expect("read tmpfs log"),
        9
    );
    assert_eq!(&buffer, b"tmpfs log");
}
