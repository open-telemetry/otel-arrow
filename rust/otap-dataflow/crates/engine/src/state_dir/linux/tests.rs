// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use nix::fcntl::{FcntlArg, FdFlag, fcntl};
use std::ffi::OsString;
use std::os::fd::{AsFd, AsRawFd};
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn fixture() -> TempDir {
    // /tmp is intentionally not trusted. Tests require a private tree beneath
    // trusted ancestors, just like production. The override supports test VMs.
    let parent = std::env::var_os("OTAP_STATE_DIR_TEST_PARENT")
        .or_else(|| std::env::var_os("HOME"))
        .expect("set OTAP_STATE_DIR_TEST_PARENT to a trusted test directory");
    let temp = tempfile::Builder::new()
        .prefix("otap-state-test-")
        .tempdir_in(parent)
        .unwrap();
    std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let _ =
        StateDirectory::provision(temp.path()).expect("test parent must have trusted ancestors");
    temp
}

fn mode(path: &Path, permissions: u32) {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(permissions)).unwrap();
}

fn subprocess(case: &str, path: &Path) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    let _ = command
        .args([
            "--exact",
            "state_dir::linux::tests::subprocess_entry",
            "--nocapture",
        ])
        .env("OTAP_STATE_TEST_CASE", case)
        .env("OTAP_STATE_TEST_PATH", path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

fn assert_success(output: std::process::Output) {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Scenario: non-UTF8 roots are reused after an ancestor rename.
/// Guarantees: clones share a close-on-exec handle that preserves directory identity.
#[test]
fn native_root_and_handle_access() {
    let temp = fixture();
    let name = OsString::from_vec(vec![b's', 0xff]);
    let path = temp.path().join("ancestor").join(name);
    let root = StateDirectory::provision(&path).unwrap();
    let again = StateDirectory::provision(&path).unwrap();
    assert_eq!(fstat(&root).unwrap().st_ino, fstat(&again).unwrap().st_ino);
    assert_eq!(root.path(), path);
    let controller = crate::context::ControllerContext::new(
        otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
    )
    .with_state_directory(root.clone());
    for generation in [0, 1, 2] {
        let context = controller.pipeline_context_with_generation(
            "g".into(),
            "p".into(),
            generation as usize,
            3,
            generation as usize,
            generation,
        );
        assert_eq!(
            context.state_directory().unwrap().as_fd().as_raw_fd(),
            root.as_fd().as_raw_fd()
        );
    }
    let clone = root.clone();
    assert_eq!(root.as_fd().as_raw_fd(), clone.as_fd().as_raw_fd());
    assert!(
        FdFlag::from_bits_truncate(fcntl(&root, FcntlArg::F_GETFD).unwrap())
            .contains(FdFlag::FD_CLOEXEC)
    );
    std::fs::rename(temp.path().join("ancestor"), temp.path().join("moved")).unwrap();
    assert!(!path.exists());
    let file = openat(
        &root,
        "sentinel",
        OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_WRONLY | OFlag::O_CLOEXEC,
        Mode::S_IRUSR | Mode::S_IWUSR,
    )
    .unwrap();
    drop(file);
    assert!(
        temp.path()
            .join("moved")
            .join(path.file_name().unwrap())
            .join("sentinel")
            .exists()
    );
    // Removing a live root does not trigger recreation by the capability.
    std::fs::remove_file(
        temp.path()
            .join("moved")
            .join(path.file_name().unwrap())
            .join("sentinel"),
    )
    .unwrap();
    std::fs::remove_dir(temp.path().join("moved").join(path.file_name().unwrap())).unwrap();
    assert!(
        openat(
            &root,
            "new",
            OFlag::O_CREAT | OFlag::O_WRONLY,
            Mode::S_IRWXU
        )
        .is_err()
    );
    assert!(!path.exists());
}

/// Scenario: traversal encounters invalid objects or permissions.
/// Guarantees: invalid objects fail without permission repair or fallback.
#[test]
fn rejects_untrusted_objects() {
    let temp = fixture();
    let target = temp.path().join("target");
    let _ = StateDirectory::provision(&target).unwrap();
    let link = temp.path().join("link");
    symlink(&target, &link).unwrap();
    assert!(StateDirectory::provision(&link).is_err());
    assert!(StateDirectory::provision(&link.join("child")).is_err());
    let file = temp.path().join("file");
    std::fs::write(&file, b"preserve").unwrap();
    assert!(StateDirectory::provision(&file).is_err());
    assert!(StateDirectory::provision(&file.join("child")).is_err());
    let fifo = temp.path().join("fifo");
    nix::unistd::mkfifo(&fifo, Mode::S_IRWXU).unwrap();
    assert!(StateDirectory::provision(&fifo).is_err());
    for permissions in [0o770, 0o707, 0o750, 0o600, 0o500] {
        mode(&target, permissions);
        let error = StateDirectory::provision(&target).unwrap_err();
        assert!(matches!(
            error,
            StateDirectoryError::Untrusted { ref path, reason }
                if path == &target && reason.contains("directory must")
                    && reason.contains("umask") && reason.contains("ACLs")
        ));
        assert_eq!(
            std::fs::metadata(&target).unwrap().permissions().mode() & 0o777,
            permissions
        );
    }
    mode(&target, 0o777);
    assert!(StateDirectory::provision(&target.join("child")).is_err());
    assert!(!target.join("child").exists());
    mode(&target, 0o700);
    assert_eq!(std::fs::read(file).unwrap(), b"preserve");
}

/// Scenario: each required sync fails during provisioning.
/// Guarantees: startup preserves OS errors and retries repeat every barrier.
#[test]
fn interrupted_provisioning_repeats_all_barriers() {
    let temp = fixture();
    let path = temp.path().join("one/two");
    let mut barriers = Vec::new();
    let _ = provision(&path, &mut |_, p| {
        barriers.push(p.to_owned());
        Ok(())
    })
    .unwrap();
    let mut expected = vec![PathBuf::from("/")];
    let mut parent = PathBuf::from("/");
    for part in path.components().skip(1) {
        expected.push(parent.clone());
        parent.push(part);
    }
    expected.push(path.clone());
    assert_eq!(barriers, expected);
    for errno in [Errno::EIO, Errno::EINVAL, Errno::EINTR] {
        for (fail_at, barrier) in barriers.iter().enumerate() {
            // Recreate only our empty fixture leaf to exercise interrupted
            // creation; production provisioning never deletes partial state.
            std::fs::remove_dir_all(temp.path().join("one")).unwrap();
            let mut calls = 0;
            let error = provision(&path, &mut |_, _| {
                let index = calls;
                calls += 1;
                if index == fail_at {
                    Err(std::io::Error::from_raw_os_error(errno as i32))
                } else {
                    Ok(())
                }
            })
            .unwrap_err();
            assert_eq!(calls, fail_at + 1);
            match error {
                StateDirectoryError::Io {
                    source,
                    path: failed,
                    ..
                } => {
                    assert_eq!(source.raw_os_error(), Some(errno as i32));
                    assert_eq!(&failed, barrier);
                }
                other => panic!("unexpected {other:?}"),
            }
            let _ = StateDirectory::provision(&path).unwrap();
        }
    }
    std::fs::write(path.join("checkpoint"), b"existing authority").unwrap();
    let _ = StateDirectory::provision(&path).unwrap();
    assert_eq!(
        std::fs::read(path.join("checkpoint")).unwrap(),
        b"existing authority"
    );
}

/// Scenario: a child is replaced during parent sync.
/// Guarantees: observed identity changes prevent capability handoff.
#[test]
fn detects_observed_substitution() {
    let temp = fixture();
    let path = temp.path().join("root");
    let error = provision(&path, &mut |_, p| {
        if p == temp.path() {
            std::fs::rename(&path, temp.path().join("moved"))?;
            std::fs::create_dir(&path)?;
            mode(&path, 0o700);
        }
        Ok(())
    })
    .unwrap_err();
    assert!(matches!(
        error,
        StateDirectoryError::Untrusted {
            reason: "directory entry changed during provisioning",
            ..
        }
    ));
}

/// Scenario: processes share a root from different working directories.
/// Guarantees: concurrent provisioning preserves the configured root identity.
#[test]
fn concurrent_processes_and_cwd_independence() {
    let temp = fixture();
    let path = temp.path().join("shared/nested/root");
    let mut children = Vec::new();
    for index in 0..6 {
        children.push(
            subprocess("provision", &path)
                .current_dir(if index % 2 == 0 {
                    Path::new("/")
                } else {
                    temp.path()
                })
                .spawn()
                .unwrap(),
        );
    }
    for child in children {
        assert_success(child.wait_with_output().unwrap());
    }
    let before = fstat(StateDirectory::provision(&path).unwrap()).unwrap();
    assert_success(
        subprocess("provision", &path)
            .current_dir("/")
            .output()
            .unwrap(),
    );
    let after = fstat(StateDirectory::provision(&path).unwrap()).unwrap();
    assert_eq!((before.st_dev, before.st_ino), (after.st_dev, after.st_ino));
}

/// Scenario: subprocesses provision directories under restrictive umasks.
/// Guarantees: 0077 succeeds, 0100 fails without repair, and descriptors close.
#[test]
fn isolated_umasks_and_descriptor_cleanup() {
    let temp = fixture();
    for case in ["umask0077", "umask0100", "fd_cleanup"] {
        assert_success(subprocess(case, &temp.path().join(case)).output().unwrap());
    }
}

/// Scenario: subprocess tests change umask or count descriptors.
/// Guarantees: process-global changes stay isolated from the parent tests.
#[test]
fn subprocess_entry() {
    let Ok(case) = std::env::var("OTAP_STATE_TEST_CASE") else {
        return;
    };
    let path = PathBuf::from(std::env::var_os("OTAP_STATE_TEST_PATH").unwrap());
    match case.as_str() {
        "interrupt" => {
            let _ = provision(&path, &mut |_, p| {
                if Some(p) == path.parent() {
                    std::process::exit(73);
                }
                Ok(())
            });
            panic!("interruption barrier was not reached");
        }
        "provision" => {
            let _ = StateDirectory::provision(&path).unwrap();
        }
        "umask0077" => {
            let _ = nix::sys::stat::umask(Mode::from_bits_truncate(0o077));
            let _ = StateDirectory::provision(&path).unwrap();
            assert_eq!(
                std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        "umask0100" => {
            let _ = nix::sys::stat::umask(Mode::from_bits_truncate(0o100));
            assert!(StateDirectory::provision(&path.join("child")).is_err());
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
            assert!(!path.join("child").exists());
            mode(&path, 0o700); // Explicit operator correction, not provisioning.
            let _ = nix::sys::stat::umask(Mode::from_bits_truncate(0o077));
            let _ = StateDirectory::provision(&path.join("child")).unwrap();
        }
        "fd_cleanup" => {
            let count = || std::fs::read_dir("/proc/self/fd").unwrap().count();
            let before = count();
            for _ in 0..30 {
                let root = StateDirectory::provision(&path).unwrap();
                assert_eq!(count(), before + 1);
                drop(root);
                assert_eq!(count(), before);
                assert!(
                    provision(&path, &mut |_, directory| {
                        assert!(count() <= before + 2);
                        if Some(directory) == path.parent() {
                            Err(std::io::Error::from_raw_os_error(5))
                        } else {
                            Ok(())
                        }
                    })
                    .is_err()
                );
                assert_eq!(count(), before);
            }
        }
        other => panic!("unknown subprocess case {other}"),
    }
}

/// Scenario: POSIX access and default ACLs affect permissions.
/// Guarantees: write-granting ACLs and non-0700 roots are rejected.
#[test]
#[ignore = "requires setfacl/getfacl and a filesystem with Linux POSIX ACL support; run explicitly"]
fn posix_and_default_acls() {
    let temp = fixture();
    let acl = |path: &Path, spec: &str| {
        let output = Command::new("setfacl")
            .args(["-m", spec])
            .arg(path)
            .output()
            .expect("setfacl required");
        assert_success(output);
    };
    // Named user write is represented in the group mode bits via ACL_MASK.
    acl(temp.path(), "u:12345:rwx,m::rwx");
    assert!(StateDirectory::provision(&temp.path().join("blocked")).is_err());
    acl(temp.path(), "m::r-x");
    let _ = StateDirectory::provision(&temp.path().join("allowed")).unwrap();
    acl(
        temp.path(),
        "m::---,d:u::rwx,d:u:12345:rwx,d:g::rwx,d:m::rwx,d:o::rwx",
    );
    let child = temp.path().join("inherited");
    let _ = StateDirectory::provision(&child).unwrap();
    let output = Command::new("getfacl")
        .args(["-cpn"])
        .arg(&child)
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("user:12345:rwx\t#effective:---"), "{text}");
    assert!(text.contains("mask::---"), "{text}");
    // An inherited default ACL can remove owner execute even with mode 0700.
    acl(temp.path(), "d:u::rw-");
    let unusable = temp.path().join("unusable");
    assert!(StateDirectory::provision(&unusable).is_err());
    assert_eq!(
        std::fs::metadata(&unusable).unwrap().permissions().mode() & 0o700,
        0o600
    );
    mode(&unusable, 0o700);
}

/// Scenario: a directory has an untrusted owner.
/// Guarantees: opened ancestors and roots reject untrusted ownership.
#[test]
#[ignore = "requires effective UID 0 and chown capability; run explicitly"]
fn rejects_foreign_owner() {
    assert_eq!(geteuid().as_raw(), 0, "root qualification requires UID 0");
    let temp = fixture();
    let path = temp.path().join("foreign");
    let _ = StateDirectory::provision(&path).unwrap();
    nix::unistd::chown(&path, Some(nix::unistd::Uid::from_raw(12345)), None).unwrap();
    assert!(StateDirectory::provision(&path).is_err());
    assert!(StateDirectory::provision(&path.join("child")).is_err());
    nix::unistd::chown(&path, Some(nix::unistd::Uid::from_raw(0)), None).unwrap();
}

/// Scenario: a process exits between root creation and parent sync.
/// Guarantees: restart validates the same location and completes all barriers.
#[test]
fn retry_after_process_interruption() {
    let temp = fixture();
    let path = temp.path().join("interrupted");
    let output = subprocess("interrupt", &path).output().unwrap();
    assert_eq!(output.status.code(), Some(73));
    assert!(path.is_dir());
    assert_success(subprocess("provision", &path).output().unwrap());
}

/// Scenario: traversal encounters an unexpected parent component.
/// Guarantees: InvalidPath prevents handoff of an incompletely traversed root.
#[test]
fn rejects_unexpected_traversal_component() {
    let temp = fixture();
    let path = temp.path().join("..");
    assert!(matches!(
        provision(&path, &mut |_, _| Ok(())),
        Err(StateDirectoryError::InvalidPath { path: rejected, .. }) if rejected == path
    ));
}
