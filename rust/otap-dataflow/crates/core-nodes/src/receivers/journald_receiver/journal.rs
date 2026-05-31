// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Linux `sd-journal` reader abstraction.

#[cfg(target_os = "linux")]
mod imp {
    #![allow(unsafe_code)]

    use crate::receivers::journald_receiver::arrow_records_encoder::{JournalEntry, JournalField};
    use crate::receivers::journald_receiver::config::{
        ExtractionConfig, LargeFieldPolicy, RuntimeConfig, StartAt,
    };

    use libc::{RTLD_NOW, c_char, c_int, c_void, size_t};
    use std::ffi::{CStr, CString};
    use std::path::{Path, PathBuf};
    use std::ptr::NonNull;
    use std::str::Utf8Error;
    use std::time::Duration;

    const SD_JOURNAL_LOCAL_ONLY: c_int = 1;
    const SD_JOURNAL_OS_ROOT: c_int = 16;

    type SdJournal = c_void;
    type OpenFn = unsafe extern "C" fn(*mut *mut SdJournal, c_int) -> c_int;
    type OpenDirectoryFn = unsafe extern "C" fn(*mut *mut SdJournal, *const c_char, c_int) -> c_int;
    type CloseFn = unsafe extern "C" fn(*mut SdJournal);
    type NextFn = unsafe extern "C" fn(*mut SdJournal) -> c_int;
    type WaitFn = unsafe extern "C" fn(*mut SdJournal, u64) -> c_int;
    type SeekHeadFn = unsafe extern "C" fn(*mut SdJournal) -> c_int;
    type SeekTailFn = unsafe extern "C" fn(*mut SdJournal) -> c_int;
    type SeekCursorFn = unsafe extern "C" fn(*mut SdJournal, *const c_char) -> c_int;
    type TestCursorFn = unsafe extern "C" fn(*mut SdJournal, *const c_char) -> c_int;
    type GetCursorFn = unsafe extern "C" fn(*mut SdJournal, *mut *mut c_char) -> c_int;
    type GetRealtimeUsecFn = unsafe extern "C" fn(*mut SdJournal, *mut u64) -> c_int;
    type SetDataThresholdFn = unsafe extern "C" fn(*mut SdJournal, size_t) -> c_int;
    type RestartDataFn = unsafe extern "C" fn(*mut SdJournal);
    type EnumerateDataFn =
        unsafe extern "C" fn(*mut SdJournal, *mut *const c_void, *mut size_t) -> c_int;
    type AddMatchFn = unsafe extern "C" fn(*mut SdJournal, *const c_void, size_t) -> c_int;
    type AddDisjunctionFn = unsafe extern "C" fn(*mut SdJournal) -> c_int;
    type AddConjunctionFn = unsafe extern "C" fn(*mut SdJournal) -> c_int;

    #[derive(Debug, thiserror::Error, Clone)]
    pub(crate) enum JournalError {
        #[error("failed to load libsystemd.so.0")]
        LoadLibSystemd,
        #[error("missing libsystemd symbol {symbol}")]
        MissingSymbol { symbol: &'static str },
        #[error("{operation} failed with {rc}")]
        SystemdCall { operation: &'static str, rc: c_int },
        #[error("sd_journal_open returned null")]
        OpenReturnedNull,
        #[error("{field} contains NUL")]
        Nul { field: &'static str },
        #[error("checkpoint cursor is no longer present in journal")]
        CheckpointCursorMissing,
        #[error("sd_journal_get_cursor returned non-UTF-8 cursor: {source}")]
        CursorUtf8 { source: Utf8Error },
        #[error(
            "selected journal root {root_path} is not readable \
             (journal_files={journal_files}, unreadable_files={unreadable_files}, \
             unreadable_directories={unreadable_directories}, first_error={first_error}); \
             run as root or grant access to the systemd-journal group, and ensure container \
             host-root mounts expose readable journal files"
        )]
        JournalAccess {
            root_path: PathBuf,
            journal_files: usize,
            unreadable_files: usize,
            unreadable_directories: usize,
            first_error: String,
        },
        #[error(
            "no systemd journal directories are visible under {root_path}; mount \
             /run/log/journal or /var/log/journal below journal.root_path"
        )]
        JournalDirectoriesMissing { root_path: PathBuf },
    }

    struct LibSystemd {
        _handle: NonNull<c_void>,
        open: OpenFn,
        open_directory: OpenDirectoryFn,
        close: CloseFn,
        next: NextFn,
        wait: WaitFn,
        seek_head: SeekHeadFn,
        seek_tail: SeekTailFn,
        seek_cursor: SeekCursorFn,
        test_cursor: TestCursorFn,
        get_cursor: GetCursorFn,
        get_realtime_usec: GetRealtimeUsecFn,
        set_data_threshold: SetDataThresholdFn,
        restart_data: RestartDataFn,
        enumerate_data: EnumerateDataFn,
        add_match: AddMatchFn,
        add_disjunction: AddDisjunctionFn,
        add_conjunction: AddConjunctionFn,
    }

    // Function pointers are immutable after load and libsystemd is process-global.
    unsafe impl Send for LibSystemd {}
    unsafe impl Sync for LibSystemd {}

    impl LibSystemd {
        fn load() -> Result<&'static Self, JournalError> {
            static LIB: std::sync::OnceLock<Result<LibSystemd, JournalError>> =
                std::sync::OnceLock::new();
            LIB.get_or_init(Self::load_inner)
                .as_ref()
                .map_err(Clone::clone)
        }

        fn load_inner() -> Result<Self, JournalError> {
            let name = CString::new("libsystemd.so.0").expect("static string");
            let handle = unsafe { libc::dlopen(name.as_ptr(), RTLD_NOW) };
            let handle = NonNull::new(handle).ok_or(JournalError::LoadLibSystemd)?;

            macro_rules! sym {
                ($name:literal, $ty:ty) => {{
                    let cname = CString::new($name).expect("static string");
                    let ptr = unsafe { libc::dlsym(handle.as_ptr(), cname.as_ptr()) };
                    if ptr.is_null() {
                        return Err(JournalError::MissingSymbol { symbol: $name });
                    }
                    unsafe { std::mem::transmute::<*mut c_void, $ty>(ptr) }
                }};
            }

            Ok(Self {
                _handle: handle,
                open: sym!("sd_journal_open", OpenFn),
                open_directory: sym!("sd_journal_open_directory", OpenDirectoryFn),
                close: sym!("sd_journal_close", CloseFn),
                next: sym!("sd_journal_next", NextFn),
                wait: sym!("sd_journal_wait", WaitFn),
                seek_head: sym!("sd_journal_seek_head", SeekHeadFn),
                seek_tail: sym!("sd_journal_seek_tail", SeekTailFn),
                seek_cursor: sym!("sd_journal_seek_cursor", SeekCursorFn),
                test_cursor: sym!("sd_journal_test_cursor", TestCursorFn),
                get_cursor: sym!("sd_journal_get_cursor", GetCursorFn),
                get_realtime_usec: sym!("sd_journal_get_realtime_usec", GetRealtimeUsecFn),
                set_data_threshold: sym!("sd_journal_set_data_threshold", SetDataThresholdFn),
                restart_data: sym!("sd_journal_restart_data", RestartDataFn),
                enumerate_data: sym!("sd_journal_enumerate_data", EnumerateDataFn),
                add_match: sym!("sd_journal_add_match", AddMatchFn),
                add_disjunction: sym!("sd_journal_add_disjunction", AddDisjunctionFn),
                add_conjunction: sym!("sd_journal_add_conjunction", AddConjunctionFn),
            })
        }
    }

    pub(crate) struct SdJournalReader {
        lib: &'static LibSystemd,
        journal: NonNull<SdJournal>,
        extraction: ExtractionConfig,
    }

    impl SdJournalReader {
        pub(crate) fn open(
            config: &RuntimeConfig,
            checkpoint: Option<&str>,
        ) -> Result<Self, JournalError> {
            preflight_journal_access(&config.journal.root_path)?;
            let lib = LibSystemd::load()?;
            let mut raw = std::ptr::null_mut();
            if config.journal.root_path == Path::new("/") {
                check(
                    unsafe { (lib.open)(&mut raw, SD_JOURNAL_LOCAL_ONLY) },
                    "sd_journal_open",
                )?;
            } else {
                let root_path = CString::new(config.journal.root_path.to_string_lossy().as_bytes())
                    .map_err(|_| JournalError::Nul {
                        field: "journal.root_path",
                    })?;
                check(
                    unsafe {
                        (lib.open_directory)(&mut raw, root_path.as_ptr(), SD_JOURNAL_OS_ROOT)
                    },
                    "sd_journal_open_directory",
                )?;
            }
            let journal = NonNull::new(raw).ok_or(JournalError::OpenReturnedNull)?;
            let mut reader = Self {
                lib,
                journal,
                extraction: config.extraction.clone(),
            };
            check(
                unsafe { (reader.lib.set_data_threshold)(reader.journal.as_ptr(), 0) },
                "sd_journal_set_data_threshold",
            )?;
            reader.configure(config, checkpoint)?;
            Ok(reader)
        }

        fn configure(
            &mut self,
            config: &RuntimeConfig,
            checkpoint: Option<&str>,
        ) -> Result<(), JournalError> {
            let mut has_match_group = false;
            has_match_group |=
                self.add_match_group("_SYSTEMD_UNIT", config.units.iter().map(String::as_str))?;
            if has_match_group && !config.identifiers.is_empty() {
                self.add_conjunction()?;
            }
            has_match_group |= self.add_match_group(
                "SYSLOG_IDENTIFIER",
                config.identifiers.iter().map(String::as_str),
            )?;
            if config.priority_filter_enabled {
                if has_match_group {
                    self.add_conjunction()?;
                }
                let _ = self.add_match_group(
                    "PRIORITY",
                    config
                        .priorities
                        .iter()
                        .map(|priority| priority.to_string()),
                )?;
            }

            if let Some(cursor) = checkpoint {
                let c = CString::new(cursor).map_err(|_| JournalError::Nul {
                    field: "checkpoint cursor",
                })?;
                check(
                    unsafe { (self.lib.seek_cursor)(self.journal.as_ptr(), c.as_ptr()) },
                    "sd_journal_seek_cursor",
                )?;
                let next = unsafe { (self.lib.next)(self.journal.as_ptr()) };
                if next < 0 {
                    return Err(JournalError::SystemdCall {
                        operation: "sd_journal_next",
                        rc: next,
                    });
                }
                if next == 0 {
                    return Err(JournalError::CheckpointCursorMissing);
                }
                let matches = unsafe { (self.lib.test_cursor)(self.journal.as_ptr(), c.as_ptr()) };
                if matches < 0 {
                    return Err(JournalError::SystemdCall {
                        operation: "sd_journal_test_cursor",
                        rc: matches,
                    });
                }
                if matches == 0 {
                    return Err(JournalError::CheckpointCursorMissing);
                }
                return Ok(());
            }

            match config.start_at {
                StartAt::Beginning => check(
                    unsafe { (self.lib.seek_head)(self.journal.as_ptr()) },
                    "sd_journal_seek_head",
                ),
                StartAt::End => check(
                    unsafe { (self.lib.seek_tail)(self.journal.as_ptr()) },
                    "sd_journal_seek_tail",
                ),
            }
        }

        fn add_match_group<I, V>(&mut self, field: &str, values: I) -> Result<bool, JournalError>
        where
            I: IntoIterator<Item = V>,
            V: AsRef<str>,
        {
            let mut added = false;
            for value in values {
                if added {
                    check(
                        unsafe { (self.lib.add_disjunction)(self.journal.as_ptr()) },
                        "sd_journal_add_disjunction",
                    )?;
                }
                self.add_match(field, value.as_ref())?;
                added = true;
            }
            Ok(added)
        }

        fn add_conjunction(&mut self) -> Result<(), JournalError> {
            check(
                unsafe { (self.lib.add_conjunction)(self.journal.as_ptr()) },
                "sd_journal_add_conjunction",
            )
        }

        fn add_match(&mut self, field: &str, value: &str) -> Result<(), JournalError> {
            let matcher = format!("{field}={value}");
            check(
                unsafe {
                    (self.lib.add_match)(
                        self.journal.as_ptr(),
                        matcher.as_ptr().cast(),
                        matcher.len(),
                    )
                },
                "sd_journal_add_match",
            )
        }

        pub(crate) fn next_entry_with_wait_timeout(
            &mut self,
            wait_timeout: Duration,
        ) -> Result<Option<JournalEntry>, JournalError> {
            loop {
                let next = unsafe { (self.lib.next)(self.journal.as_ptr()) };
                if next < 0 {
                    return Err(JournalError::SystemdCall {
                        operation: "sd_journal_next",
                        rc: next,
                    });
                }
                if next > 0 {
                    return self.current_entry().map(Some);
                }
                let timeout = duration_to_usec(wait_timeout);
                let waited = unsafe { (self.lib.wait)(self.journal.as_ptr(), timeout) };
                if waited < 0 {
                    return Err(JournalError::SystemdCall {
                        operation: "sd_journal_wait",
                        rc: waited,
                    });
                }
                if waited == 0 {
                    return Ok(None);
                }
            }
        }

        fn current_entry(&mut self) -> Result<JournalEntry, JournalError> {
            let mut cursor_ptr: *mut c_char = std::ptr::null_mut();
            check(
                unsafe { (self.lib.get_cursor)(self.journal.as_ptr(), &mut cursor_ptr) },
                "sd_journal_get_cursor",
            )?;
            let cursor = unsafe { CStr::from_ptr(cursor_ptr) }
                .to_str()
                .map(str::to_owned)
                .map_err(|source| JournalError::CursorUtf8 { source });
            unsafe { libc::free(cursor_ptr.cast()) };
            let cursor = cursor?;

            let mut realtime_usec = 0u64;
            check(
                unsafe { (self.lib.get_realtime_usec)(self.journal.as_ptr(), &mut realtime_usec) },
                "sd_journal_get_realtime_usec",
            )?;

            unsafe { (self.lib.restart_data)(self.journal.as_ptr()) };
            let mut fields = Vec::with_capacity(self.extraction.max_fields_per_entry.min(64));
            let mut copied_entry_bytes = 0u64;
            let mut copied_field_count = 0usize;
            let mut dropped_fields = 0u64;
            let mut message_seen = false;
            let mut message_body = None;
            loop {
                let mut data: *const c_void = std::ptr::null();
                let mut len: size_t = 0;
                let rc = unsafe {
                    (self.lib.enumerate_data)(self.journal.as_ptr(), &mut data, &mut len)
                };
                if rc < 0 {
                    return Err(JournalError::SystemdCall {
                        operation: "sd_journal_enumerate_data",
                        rc,
                    });
                }
                if rc == 0 {
                    break;
                }
                let bytes = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), len) };
                if let Some(eq) = bytes.iter().position(|b| *b == b'=') {
                    let is_first_message = if !message_seen && &bytes[..eq] == b"MESSAGE" {
                        message_seen = true;
                        true
                    } else {
                        false
                    };
                    let value_len = bytes.len().saturating_sub(eq + 1) as u64;
                    let field_len = bytes.len() as u64;
                    let would_exceed_entry = copied_entry_bytes.saturating_add(field_len)
                        > self.extraction.max_entry_bytes;
                    let should_drop = value_len > self.extraction.max_field_bytes
                        || copied_field_count >= self.extraction.max_fields_per_entry
                        || would_exceed_entry;
                    if should_drop {
                        match self.extraction.large_field_policy {
                            LargeFieldPolicy::DropAndCount => {
                                dropped_fields = dropped_fields.saturating_add(1);
                                continue;
                            }
                        }
                    }
                    let name = match std::str::from_utf8(&bytes[..eq]) {
                        Ok(name) => name.to_owned(),
                        Err(_) => {
                            dropped_fields = dropped_fields.saturating_add(1);
                            continue;
                        }
                    };
                    let value = bytes[eq + 1..].to_vec();
                    if is_first_message {
                        message_body = Some(value.clone());
                    }
                    fields.push(JournalField { name, value });
                    copied_entry_bytes = copied_entry_bytes.saturating_add(field_len);
                    copied_field_count = copied_field_count.saturating_add(1);
                }
            }

            Ok(JournalEntry {
                cursor,
                message_body,
                realtime_unix_nano: realtime_usec.saturating_mul(1000),
                fields,
                dropped_fields,
            })
        }
    }

    fn duration_to_usec(duration: Duration) -> u64 {
        if duration.is_zero() {
            return 0;
        }
        let usec = duration.as_micros().min(u64::MAX as u128) as u64;
        usec.max(1)
    }

    impl Drop for SdJournalReader {
        fn drop(&mut self) {
            unsafe { (self.lib.close)(self.journal.as_ptr()) };
        }
    }

    fn check(rc: c_int, operation: &'static str) -> Result<(), JournalError> {
        if rc < 0 {
            Err(JournalError::SystemdCall { operation, rc })
        } else {
            Ok(())
        }
    }

    #[derive(Default)]
    struct JournalAccessSummary {
        journal_files: usize,
        readable_files: usize,
        visible_directories: usize,
        unreadable_files: usize,
        unreadable_directories: usize,
        first_error: Option<String>,
    }

    fn preflight_journal_access(root_path: &Path) -> Result<(), JournalError> {
        let mut summary = JournalAccessSummary::default();
        for relative in ["run/log/journal", "var/log/journal"] {
            inspect_journal_path(&root_path.join(relative), 0, &mut summary);
        }

        if root_path != Path::new("/") && summary.visible_directories == 0 {
            return Err(JournalError::JournalDirectoriesMissing {
                root_path: root_path.to_path_buf(),
            });
        }

        if (summary.journal_files > 0 || summary.unreadable_directories > 0)
            && summary.readable_files == 0
            && (summary.unreadable_files > 0 || summary.unreadable_directories > 0)
        {
            return Err(JournalError::JournalAccess {
                root_path: root_path.to_path_buf(),
                journal_files: summary.journal_files,
                unreadable_files: summary.unreadable_files,
                unreadable_directories: summary.unreadable_directories,
                first_error: summary
                    .first_error
                    .unwrap_or_else(|| "permission denied".to_owned()),
            });
        }
        Ok(())
    }

    fn inspect_journal_path(path: &Path, depth: usize, summary: &mut JournalAccessSummary) {
        const MAX_DEPTH: usize = 4;
        if depth > MAX_DEPTH {
            return;
        }

        let metadata = match std::fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::PermissionDenied {
                    summary.unreadable_directories =
                        summary.unreadable_directories.saturating_add(1);
                }
                record_first_error(summary, path, &err);
                return;
            }
        };

        if metadata.is_file() {
            inspect_journal_file(path, summary);
            return;
        }
        if !metadata.is_dir() {
            return;
        }
        summary.visible_directories = summary.visible_directories.saturating_add(1);

        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::PermissionDenied {
                    summary.unreadable_directories =
                        summary.unreadable_directories.saturating_add(1);
                }
                record_first_error(summary, path, &err);
                return;
            }
        };

        for entry in entries {
            match entry {
                Ok(entry) => inspect_journal_path(&entry.path(), depth + 1, summary),
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::PermissionDenied {
                        summary.unreadable_directories =
                            summary.unreadable_directories.saturating_add(1);
                    }
                    if summary.first_error.is_none() {
                        summary.first_error = Some(err.to_string());
                    }
                }
            }
        }
    }

    fn inspect_journal_file(path: &Path, summary: &mut JournalAccessSummary) {
        if !is_journal_file(path) {
            return;
        }
        summary.journal_files = summary.journal_files.saturating_add(1);
        match std::fs::File::open(path) {
            Ok(_) => {
                summary.readable_files = summary.readable_files.saturating_add(1);
            }
            Err(err) => {
                if err.kind() == std::io::ErrorKind::PermissionDenied {
                    summary.unreadable_files = summary.unreadable_files.saturating_add(1);
                }
                record_first_error(summary, path, &err);
            }
        }
    }

    fn is_journal_file(path: &Path) -> bool {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return false;
        };
        name.ends_with(".journal") || name.ends_with(".journal~")
    }

    fn record_first_error(summary: &mut JournalAccessSummary, path: &Path, err: &std::io::Error) {
        if summary.first_error.is_none() {
            summary.first_error = Some(format!("{}: {err}", path.display()));
        }
    }
}

#[cfg(target_os = "linux")]
pub(crate) use imp::{JournalError, SdJournalReader};
