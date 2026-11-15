// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Access to the profiler started by mimalloc's `MIMALLOC_PROFILE` option.

use otel_arrow_dfe_controller::HeapProfileError;
use std::ffi::{c_int, c_void};

const MAX_PROFILE_SIZE: usize = 16 * 1024 * 1024;
const MI_PPROF_OK: c_int = 0;
const MI_PPROF_DISABLED: c_int = 1;
const MI_PPROF_BUSY: c_int = 2;

unsafe extern "C" {
    // Declared in the vendored mimalloc-profile.h. The callback is synchronous.
    fn mi_pprof_snapshot_env(
        max_size: usize,
        sink: unsafe extern "C" fn(*mut c_void, *const u8, usize) -> bool,
        context: *mut c_void,
    ) -> c_int;
}

unsafe extern "C" fn copy_profile(context: *mut c_void, bytes: *const u8, len: usize) -> bool {
    if bytes.is_null() || len == 0 || len > MAX_PROFILE_SIZE {
        return false;
    }
    // SAFETY: snapshot passes an exclusive Vec pointer. Mimalloc owns the
    // encoded bytes and keeps them alive until this callback returns.
    let profile = unsafe { &mut *context.cast::<Vec<u8>>() };
    if profile.try_reserve_exact(len).is_err() {
        return false;
    }
    profile.extend_from_slice(unsafe { std::slice::from_raw_parts(bytes, len) });
    true
}

pub(crate) fn snapshot() -> Result<Vec<u8>, HeapProfileError> {
    let mut profile = Vec::<u8>::new();
    // SAFETY: the context and callback remain valid for the complete call.
    let result =
        unsafe { mi_pprof_snapshot_env(MAX_PROFILE_SIZE, copy_profile, (&raw mut profile).cast()) };
    match result {
        MI_PPROF_OK => Ok(profile),
        MI_PPROF_DISABLED => Err(HeapProfileError::Unavailable(
            "Heap profiling is inactive. Start the engine with MIMALLOC_PROFILE set to a file prefix.",
        )),
        MI_PPROF_BUSY => Err(HeapProfileError::Busy),
        _ => Err(HeapProfileError::Failed(
            "Could not encode the heap profile within the 16 MiB response limit".into(),
        )),
    }
}

#[cfg(all(test, target_os = "linux", not(feature = "dhat-heap")))]
mod tests {
    use super::*;
    use pprof::protos::{Message, Profile};
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    const CHILD_MODE: &str = "DFE_MIMALLOC_PROFILE_TEST_CHILD";

    /// Scenario: A Linux process takes an in-memory heap snapshot and exits with file profiling enabled.
    /// Guarantees: Both protobufs map sampled executable addresses to the offsets recorded by Linux,
    /// and the in-memory snapshot does not write a file or enable sampling by itself.
    #[test]
    fn protobuf_profiles_record_linux_mapping_offsets() {
        if let Some(mode) = std::env::var_os(CHILD_MODE) {
            if mode == "inactive" {
                assert!(matches!(snapshot(), Err(HeapProfileError::Unavailable(_))));
                return;
            }
            // Keep large, sampled allocations live through the process-exit snapshot.
            for _ in 0..16 {
                std::mem::forget(std::hint::black_box(vec![42_u8; 1024 * 1024]));
            }
            let directory = Path::new(&mode);
            let profile = snapshot().expect("capture in-memory heap profile");
            assert!(!directory.join("heap.0001.pb").exists());
            fs::write(directory.join("live.pb"), profile).unwrap();
            fs::write(directory.join("maps"), fs::read("/proc/self/maps").unwrap()).unwrap();
            return;
        }

        let directory = tempfile::tempdir().unwrap();
        let executable = std::env::current_exe().unwrap();
        let mut child = Command::new(&executable);
        let _ = child.args([
            "--exact",
            "mimalloc_profile::tests::protobuf_profiles_record_linux_mapping_offsets",
        ]);
        let inactive = child
            .env_remove("MIMALLOC_PROFILE")
            .env(CHILD_MODE, "inactive")
            .output()
            .unwrap();
        assert!(inactive.status.success(), "{inactive:?}");
        assert!(String::from_utf8_lossy(&inactive.stdout).contains("1 passed"));
        let active = child
            .env(CHILD_MODE, directory.path())
            .env("MIMALLOC_PROFILE", directory.path().join("heap"))
            .env("MIMALLOC_PROFILE_ALLOC_INTERVAL", "0")
            .env("MIMALLOC_PROFILE_TIME_INTERVAL", "0")
            .env("MIMALLOC_PROFILE_INUSE_INTERVAL", "0")
            .env("MIMALLOC_PROFILE_SAMPLE_RATE", "512KiB")
            .output()
            .unwrap();
        assert!(active.status.success(), "{active:?}");

        let maps = fs::read_to_string(directory.path().join("maps")).unwrap();
        let actual = maps
            .lines()
            .find_map(|line| {
                let mut fields = line.split_whitespace();
                let (start, end) = fields.next()?.split_once('-')?;
                let perms = fields.next()?;
                let offset = fields.next()?;
                if perms.as_bytes().get(2) != Some(&b'x') || !line.ends_with(executable.to_str()?) {
                    return None;
                }
                Some((
                    u64::from_str_radix(start, 16).ok()?,
                    u64::from_str_radix(end, 16).ok()?,
                    u64::from_str_radix(offset, 16).ok()?,
                ))
            })
            .expect("executable mapping in Linux process maps");
        for filename in ["live.pb", "heap.0001.pb"] {
            let data = fs::read(directory.path().join(filename)).unwrap();
            let profile = Profile::decode(data.as_slice()).unwrap();
            let mapping = profile
                .mapping
                .iter()
                .find(|mapping| {
                    (mapping.memory_start, mapping.memory_limit) == (actual.0, actual.1)
                })
                .expect("executable mapping in pprof protobuf");
            assert_eq!(mapping.file_offset, actual.2, "{filename}");
            assert!(
                profile
                    .location
                    .iter()
                    .any(|loc| loc.mapping_id == mapping.id)
            );
            let inuse_bytes = profile
                .sample_type
                .iter()
                .position(|value| profile.string_table[value.ty as usize] == "inuse_bytes")
                .expect("live byte count in profile schema");
            assert!(
                profile
                    .sample
                    .iter()
                    .any(|sample| sample.value[inuse_bytes] > 0)
            );
        }
    }
}
