// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded native path conversion for allocation-free repeated Linux syscalls.

use std::ffi::CStr;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// Shared native byte bound, including room for the syscall terminator.
pub(super) fn check_path_bytes(path: &Path, limit: usize) -> io::Result<()> {
    let bytes = path.as_os_str().as_bytes();
    if bytes.len() >= limit {
        return Err(io::Error::from_raw_os_error(libc::ENAMETOOLONG));
    }
    if bytes.contains(&0) {
        return Err(io::Error::from_raw_os_error(libc::EINVAL));
    }
    Ok(())
}

pub(super) fn valid_absolute_path(path: &Path, limit: usize) -> bool {
    path.is_absolute()
        && check_path_bytes(path, limit).is_ok()
        && !path
            .components()
            .any(|c| c == std::path::Component::ParentDir)
}

/// Complete filename accepted by targeted checks; directory observations use
/// `valid_absolute_path` instead. Inspect native bytes because Path normalizes
/// trailing separators and dot components when reporting file_name.
pub(super) fn valid_binding_path(path: &Path, limit: usize) -> bool {
    valid_absolute_path(path, limit)
        && path.file_name().is_some()
        && !path.as_os_str().as_bytes().ends_with(b"/")
        && !path.as_os_str().as_bytes().ends_with(b"/.")
}

pub(super) struct NativePath {
    bytes: Vec<u8>,
    len: usize,
}

impl NativePath {
    pub fn new(path_bytes: usize) -> Self {
        Self {
            bytes: vec![0; path_bytes + 1],
            len: 0,
        }
    }

    pub fn heap_bytes(&self) -> usize {
        self.bytes.capacity()
    }

    pub fn load(&mut self, path: &Path) -> io::Result<()> {
        let bytes = path.as_os_str().as_bytes();
        check_path_bytes(path, self.bytes.len() - 1)?;
        self.bytes[..bytes.len()].copy_from_slice(bytes);
        self.bytes[bytes.len()] = 0;
        self.len = bytes.len();
        Ok(())
    }

    pub fn as_c_str(&self) -> &CStr {
        CStr::from_bytes_with_nul(&self.bytes[..=self.len]).expect("validated native path")
    }

    /// Temporarily terminates an ancestor prefix, restoring the full path on
    /// success or a returned error. The borrowed C string cannot escape the call.
    pub fn with_prefix<T>(
        &mut self,
        len: usize,
        operation: impl FnOnce(&CStr) -> io::Result<T>,
    ) -> io::Result<T> {
        if len > self.len {
            return Err(io::Error::from_raw_os_error(libc::EINVAL));
        }
        let saved = self.bytes[len];
        self.bytes[len] = 0;
        let result = CStr::from_bytes_with_nul(&self.bytes[..=len])
            .map_err(|_| io::Error::from_raw_os_error(libc::EINVAL))
            .and_then(operation);
        self.bytes[len] = saved;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    /// Scenario: Native paths cross the short-C-string threshold and contain non-UTF-8 bytes.
    /// Guarantees: Conversion preserves every byte and reuses fixed capacity at 255, 256, 257, and 1024 bytes.
    #[test]
    fn long_native_paths_preserve_bytes_and_capacity() {
        let mut scratch = NativePath::new(2048);
        let capacity = scratch.heap_bytes();
        for len in [255, 256, 257, 1024] {
            let mut bytes = vec![b'x'; len];
            bytes[0] = b'/';
            bytes[len - 1] = 0xff;
            scratch.load(Path::new(OsStr::from_bytes(&bytes))).unwrap();
            assert_eq!(scratch.as_c_str().to_bytes(), bytes);
            assert_eq!(scratch.heap_bytes(), capacity);
        }
    }

    /// Scenario: Ancestor lookup succeeds or fails, including the root prefix and an invalid length.
    /// Guarantees: Temporary termination never corrupts the full path used by later verification.
    #[test]
    fn ancestor_lookup_restores_path_on_return() {
        let mut scratch = NativePath::new(64);
        scratch.load(Path::new("/parent/child")).unwrap();
        scratch
            .with_prefix(7, |prefix| {
                assert_eq!(prefix, c"/parent");
                Ok(())
            })
            .unwrap();
        let error = scratch
            .with_prefix::<()>(1, |prefix| {
                assert_eq!(prefix, c"/");
                Err(io::Error::from_raw_os_error(libc::EACCES))
            })
            .unwrap_err();
        assert_eq!(error.raw_os_error(), Some(libc::EACCES));
        assert!(scratch.with_prefix(65, |_| Ok(())).is_err());
        assert_eq!(scratch.as_c_str(), c"/parent/child");
    }

    /// Scenario: A conversion reaches the configured bound or includes an interior NUL.
    /// Guarantees: Invalid input is rejected without truncation or overwriting earlier valid evidence.
    #[test]
    fn invalid_paths_preserve_previous_value() {
        let mut scratch = NativePath::new(8);
        scratch.load(Path::new("/123456")).unwrap();
        assert_eq!(
            scratch
                .load(Path::new("/1234567"))
                .unwrap_err()
                .raw_os_error(),
            Some(libc::ENAMETOOLONG)
        );
        assert_eq!(
            scratch
                .load(Path::new(OsStr::from_bytes(b"/a\0b")))
                .unwrap_err()
                .raw_os_error(),
            Some(libc::EINVAL)
        );
        assert_eq!(scratch.as_c_str(), c"/123456");
    }
}
