// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Linux `sd-journal` reader abstraction.

#[cfg(any(target_os = "linux", test))]
use crate::receivers::journald_receiver::config::StartAt;
#[cfg(any(target_os = "linux", test))]
use std::{ffi::c_int, time::Duration};

/// Minimal seam over the `sd-journal` seek/step calls used to position a freshly
/// opened journal when no checkpoint cursor exists. Abstracting the three raw
/// FFI calls lets [`position_for_fresh_start`] -- including its empty-journal
/// handling -- be unit-tested without a live `sd-journal`. Each method returns
/// the raw libsystemd code (`>= 0` on success, `< 0` is `-errno`).
#[cfg(any(target_os = "linux", test))]
trait JournalSeek {
    /// `sd_journal_seek_head`: position before the first entry.
    fn seek_head(&mut self) -> c_int;
    /// `sd_journal_seek_tail`: position after the most recent entry.
    fn seek_tail(&mut self) -> c_int;
    /// `sd_journal_previous`: step to the previous entry. Returns `1` when an
    /// entry is now current, `0` when there is none, `< 0` on error.
    fn previous(&mut self) -> c_int;
}

/// Minimal seam over the `sd-journal` calls used by the follow loop.
#[cfg(any(target_os = "linux", test))]
trait JournalFollow {
    /// `sd_journal_next`: advance to the next matching entry.
    fn next(&mut self) -> c_int;
    /// `sd_journal_wait`: wait for a journal change or timeout.
    fn wait(&mut self, wait_timeout: Duration) -> c_int;
}

#[cfg(any(target_os = "linux", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FreshStartPosition {
    Beginning,
    EndAnchored,
    EndHeadRecovery,
}

/// Advances through at most one blocking `next()`/`wait()` cycle and returns
/// the next current entry.
///
/// A positive wait result may be caused by an append that does not match the
/// configured filters. If the retrying `next()` still finds no matching entry,
/// this returns `None` so the worker can poll control commands and recompute its
/// batch flush deadline instead of re-arming the full timeout indefinitely.
#[cfg(any(target_os = "linux", test))]
fn follow_next_entry<J, T, E>(
    journal: &mut J,
    wait_timeout: Duration,
    mut current_entry: impl FnMut(&mut J) -> Result<T, E>,
    mut systemd_error: impl FnMut(&'static str, c_int) -> E,
) -> Result<Option<T>, E>
where
    J: JournalFollow,
{
    let mut has_waited = false;
    loop {
        let next = journal.next();
        if next < 0 {
            return Err(systemd_error("sd_journal_next", next));
        }
        if next > 0 {
            return current_entry(journal).map(Some);
        }
        if has_waited {
            return Ok(None);
        }

        let waited = journal.wait(wait_timeout);
        if waited < 0 {
            return Err(systemd_error("sd_journal_wait", waited));
        }
        if waited == 0 {
            return Ok(None);
        }
        has_waited = true;
    }
}

/// Positions a freshly opened journal (no checkpoint cursor) for `start_at`.
///
/// `StartAt::Beginning` seeks the head; the worker's `next()` then iterates
/// forward from the first entry (the documented `SD_JOURNAL_FOREACH` idiom).
///
/// `StartAt::End` is subtle. `sd_journal_seek_tail()` parks the read head
/// *after* the most recent entry without making any entry current. From there a
/// bare `sd_journal_next()` advances toward a *following* entry, finds none, and
/// returns `0` (the documented EOF marker) without anchoring -- so a plain
/// `next()`/`wait()` follow loop started at the raw tail never advances onto
/// entries appended after startup (verified against real journald). (It is
/// `sd_journal_step_one()`, not `sd_journal_next()`, that libsystemd documents as
/// behaving like `sd_journal_previous()` at the tail.) So we step back once with
/// `previous()` -- documented to seek the closest *preceding* entry -- to anchor on
/// the last existing entry; the worker's first `next()` then steps forward onto
/// genuinely new entries.
///
/// When the journal is empty -- or a filter matches none of the existing
/// entries -- `previous()` returns `0` and anchors nothing, leaving the read head
/// parked at the tail where `next()` keeps returning `0` even after `wait()`
/// reports appends (the same permanent stall the tail branch exists to avoid,
/// verified against real journald). In that case we reposition to the head with
/// `seek_head()` so the follow loop can make progress. This does not replay
/// history: `sd-journal` merges *all* open journal files (active plus
/// rotated/archived) into a single view, and with the receiver's matches
/// installed a well-behaved `previous()` returns `0` only when no matching entry
/// exists in ANY of them yet -- so `seek_head()` + `next()` land on the first
/// matching entry appended *after* startup, with nothing older to replay. Replay
/// would require `seek_tail()` + `previous()` to spuriously report `0` while
/// matching entries actually exist (the multi-file positioning bug of the
/// systemd#17662 class noted below); on such an old/buggy libsystemd it would
/// replay that history once at startup (not observed on modern systemd). A later
/// `SD_JOURNAL_INVALIDATE` can likewise make older matching entries visible after
/// the head seek. Bounding replay across either case needs a durable tail-boundary
/// guard, which `start_at: end` deliberately omits (see the resume-anchor note
/// below and issue #3399).
///
/// Returns `Err((operation, rc))` with the failing call's name and its negative
/// return code. `seek_tail`/`seek_head` return `0` on success; `previous`
/// returning `0` is the empty/no-match case above, not an error.
///
/// `seek_tail()` + `previous()` is the accepted best-effort idiom (see
/// systemd/systemd#17662, coreos/go-systemd `sdjournal`). Across rotated or
/// multi-file journals the tail position is approximate, and an entry appended
/// in the race window between `seek_tail()` and `previous()` may be anchored and
/// skipped -- acceptable under `start_at: end`, which has no durable resume
/// anchor until the first checkpoint commit.
#[cfg(any(target_os = "linux", test))]
fn position_for_fresh_start<J: JournalSeek>(
    seek: &mut J,
    start_at: StartAt,
) -> Result<FreshStartPosition, (&'static str, c_int)> {
    fn require_nonneg(rc: c_int, operation: &'static str) -> Result<(), (&'static str, c_int)> {
        if rc < 0 { Err((operation, rc)) } else { Ok(()) }
    }

    match start_at {
        StartAt::Beginning => {
            require_nonneg(seek.seek_head(), "sd_journal_seek_head")?;
            Ok(FreshStartPosition::Beginning)
        }
        StartAt::End => {
            require_nonneg(seek.seek_tail(), "sd_journal_seek_tail")?;
            let anchored = seek.previous();
            require_nonneg(anchored, "sd_journal_previous")?;
            if anchored == 0 {
                require_nonneg(seek.seek_head(), "sd_journal_seek_head")?;
                return Ok(FreshStartPosition::EndHeadRecovery);
            }
            Ok(FreshStartPosition::EndAnchored)
        }
    }
}

#[cfg(test)]
mod fresh_start_tests {
    use super::{
        FreshStartPosition, JournalFollow, JournalSeek, follow_next_entry, position_for_fresh_start,
    };
    use crate::receivers::journald_receiver::config::StartAt;
    use std::ffi::c_int;
    use std::time::Duration;

    #[derive(Default)]
    struct RecordingSeek {
        calls: Vec<&'static str>,
        seek_head_rc: c_int,
        seek_tail_rc: c_int,
        previous_rc: c_int,
    }

    impl JournalSeek for RecordingSeek {
        fn seek_head(&mut self) -> c_int {
            self.calls.push("seek_head");
            self.seek_head_rc
        }
        fn seek_tail(&mut self) -> c_int {
            self.calls.push("seek_tail");
            self.seek_tail_rc
        }
        fn previous(&mut self) -> c_int {
            self.calls.push("previous");
            self.previous_rc
        }
    }

    #[derive(Clone, Copy)]
    struct FakeEntry {
        cursor: &'static str,
        matches: bool,
    }

    #[derive(Clone, Copy)]
    enum FakePosition {
        BeforeHead,
        At(usize),
        After(usize),
        RawTail,
    }

    struct FakeJournal {
        entries: Vec<FakeEntry>,
        position: FakePosition,
        append_on_wait: Option<FakeEntry>,
        current_error: Option<String>,
        next_error: Option<c_int>,
        wait_error: Option<c_int>,
        wait_change_rc: c_int,
        wait_timeouts: Vec<Duration>,
        calls: Vec<&'static str>,
    }

    impl FakeJournal {
        fn new(entries: Vec<FakeEntry>, append_on_wait: Option<FakeEntry>) -> Self {
            Self {
                entries,
                position: FakePosition::BeforeHead,
                append_on_wait,
                current_error: None,
                next_error: None,
                wait_error: None,
                wait_change_rc: 1,
                wait_timeouts: Vec::new(),
                calls: Vec::new(),
            }
        }

        fn current_cursor(&mut self) -> Result<&'static str, String> {
            if let Some(error) = self.current_error.take() {
                return Err(error);
            }
            let FakePosition::At(index) = self.position else {
                return Err("no current entry".to_owned());
            };
            Ok(self.entries[index].cursor)
        }
    }

    fn follow(
        journal: &mut FakeJournal,
        wait_timeout: Duration,
    ) -> Result<Option<&'static str>, String> {
        follow_next_entry(
            journal,
            wait_timeout,
            FakeJournal::current_cursor,
            |operation, rc| format!("{operation} failed with {rc}"),
        )
    }

    impl JournalSeek for FakeJournal {
        fn seek_head(&mut self) -> c_int {
            self.calls.push("seek_head");
            self.position = FakePosition::BeforeHead;
            0
        }

        fn seek_tail(&mut self) -> c_int {
            self.calls.push("seek_tail");
            self.position = FakePosition::RawTail;
            0
        }

        fn previous(&mut self) -> c_int {
            self.calls.push("previous");
            let end = match self.position {
                FakePosition::BeforeHead => 0,
                FakePosition::At(index) => index,
                FakePosition::After(index) => index,
                FakePosition::RawTail => self.entries.len(),
            };
            if let Some(index) = (0..end).rev().find(|index| self.entries[*index].matches) {
                self.position = FakePosition::At(index);
                1
            } else {
                0
            }
        }
    }

    impl JournalFollow for FakeJournal {
        fn next(&mut self) -> c_int {
            self.calls.push("next");
            if let Some(rc) = self.next_error.take() {
                return rc;
            }
            let start = match self.position {
                FakePosition::BeforeHead => 0,
                FakePosition::At(index) => index.saturating_add(1),
                FakePosition::After(index) => index,
                // Model libsystemd's raw-tail behavior: without an anchor or
                // seek_head recovery, appends remain invisible to bare next().
                FakePosition::RawTail => return 0,
            };
            if let Some(index) =
                (start..self.entries.len()).find(|index| self.entries[*index].matches)
            {
                self.position = FakePosition::At(index);
                1
            } else {
                self.position = FakePosition::After(self.entries.len());
                0
            }
        }

        fn wait(&mut self, wait_timeout: Duration) -> c_int {
            self.calls.push("wait");
            self.wait_timeouts.push(wait_timeout);
            if let Some(rc) = self.wait_error.take() {
                return rc;
            }
            if let Some(entry) = self.append_on_wait.take() {
                self.entries.push(entry);
                self.wait_change_rc
            } else {
                0
            }
        }
    }

    /// Scenario: a fresh `start_at: beginning` start is positioned.
    /// Guarantees: only `seek_head` is issued (no tail/previous stepping), so
    /// iteration begins at the oldest retained entry.
    #[test]
    fn beginning_seeks_head_only() {
        let mut seek = RecordingSeek::default();
        assert_eq!(
            position_for_fresh_start(&mut seek, StartAt::Beginning),
            Ok(FreshStartPosition::Beginning)
        );
        assert_eq!(seek.calls, ["seek_head"]);
    }

    /// Scenario: a fresh `start_at: end` start on a journal that already has
    /// entries (`previous` returns 1).
    /// Guarantees: the read head is anchored with `seek_tail` + `previous` and
    /// never `seek_head`, so existing historical records are not replayed.
    #[test]
    fn end_with_existing_entries_anchors_with_previous() {
        // previous() finds the last existing entry (rc = 1): tail + previous and
        // NO seek_head -- rewinding to the head would replay historical records.
        let mut seek = RecordingSeek {
            previous_rc: 1,
            ..Default::default()
        };
        assert_eq!(
            position_for_fresh_start(&mut seek, StartAt::End),
            Ok(FreshStartPosition::EndAnchored)
        );
        assert_eq!(seek.calls, ["seek_tail", "previous"]);
    }

    /// Scenario: a fresh `start_at: end` start on an empty journal where
    /// `previous` returns 0 (nothing to anchor on).
    /// Guarantees: the read head is unparked with a recovery `seek_head`, so the
    /// later `next`/`wait` loop still advances onto newly appended entries
    /// instead of stalling forever.
    #[test]
    fn end_on_empty_journal_repositions_to_head() {
        // Regression guard for the empty/no-match permanent stall: previous()
        // returns 0 (nothing to anchor), so the read head must be unparked with
        // seek_head() or the next()/wait() loop never advances onto new entries.
        let mut seek = RecordingSeek {
            previous_rc: 0,
            ..Default::default()
        };
        assert_eq!(
            position_for_fresh_start(&mut seek, StartAt::End),
            Ok(FreshStartPosition::EndHeadRecovery)
        );
        assert_eq!(seek.calls, ["seek_tail", "previous", "seek_head"]);
    }

    /// Scenario: `start_at: end` starts with historical entries that do not
    /// match the configured filters, then a matching entry is appended.
    /// Guarantees: the follow loop delivers the first matching append instead
    /// of remaining parked at the raw tail when existing history has no match.
    #[test]
    fn end_with_no_existing_match_follows_first_matching_append() {
        let mut journal = FakeJournal::new(
            vec![FakeEntry {
                cursor: "historical-unmatched",
                matches: false,
            }],
            Some(FakeEntry {
                cursor: "new-matching",
                matches: true,
            }),
        );

        assert_eq!(
            position_for_fresh_start(&mut journal, StartAt::End),
            Ok(FreshStartPosition::EndHeadRecovery)
        );
        let delivered =
            follow(&mut journal, Duration::from_millis(1)).expect("follow newly appended entry");

        assert_eq!(delivered, Some("new-matching"));
        assert_eq!(
            journal.calls,
            ["seek_tail", "previous", "seek_head", "next", "wait", "next"]
        );
    }

    /// Scenario: `start_at: end` starts with an existing matching entry, then
    /// another matching entry is appended after startup positioning.
    /// Guarantees: the existing entry is used only as the tail anchor and the
    /// follow loop delivers the newly appended entry without replay.
    #[test]
    fn end_with_existing_match_follows_append_without_replay() {
        let mut journal = FakeJournal::new(
            vec![FakeEntry {
                cursor: "historical-matching",
                matches: true,
            }],
            Some(FakeEntry {
                cursor: "new-matching",
                matches: true,
            }),
        );

        assert_eq!(
            position_for_fresh_start(&mut journal, StartAt::End),
            Ok(FreshStartPosition::EndAnchored)
        );
        let delivered =
            follow(&mut journal, Duration::from_millis(1)).expect("follow newly appended entry");

        assert_eq!(delivered, Some("new-matching"));
        assert_eq!(
            journal.calls,
            ["seek_tail", "previous", "next", "wait", "next"]
        );

        let exhausted = follow(&mut journal, Duration::from_millis(1))
            .expect("observe end after delivered entry");
        assert_eq!(exhausted, None);
        assert_eq!(
            journal.calls,
            [
                "seek_tail",
                "previous",
                "next",
                "wait",
                "next",
                "next",
                "wait"
            ]
        );
    }

    /// Scenario: `start_at: beginning` has an existing matching entry.
    /// Guarantees: the follow loop returns the current entry immediately without
    /// waiting when `next()` succeeds on its first attempt.
    #[test]
    fn beginning_with_existing_match_delivers_immediately() {
        let mut journal = FakeJournal::new(
            vec![FakeEntry {
                cursor: "historical-matching",
                matches: true,
            }],
            None,
        );

        assert_eq!(
            position_for_fresh_start(&mut journal, StartAt::Beginning),
            Ok(FreshStartPosition::Beginning)
        );
        let delivered =
            follow(&mut journal, Duration::from_millis(17)).expect("read existing entry");

        assert_eq!(delivered, Some("historical-matching"));
        assert_eq!(journal.calls, ["seek_head", "next"]);
        assert!(journal.wait_timeouts.is_empty());
    }

    /// Scenario: the raw-tail position receives an append without the
    /// `previous()` or `seek_head()` recovery used by the production fix.
    /// Guarantees: the fake preserves the pre-fix stall and the follow helper
    /// yields after one wake instead of re-arming the timeout indefinitely.
    #[test]
    fn raw_tail_without_recovery_stays_stalled_after_append() {
        let mut journal = FakeJournal::new(
            Vec::new(),
            Some(FakeEntry {
                cursor: "new-matching",
                matches: true,
            }),
        );
        assert_eq!(journal.seek_tail(), 0);

        let delivered = follow(&mut journal, Duration::from_millis(1)).expect("follow raw tail");

        assert_eq!(delivered, None);
        assert_eq!(journal.calls, ["seek_tail", "next", "wait", "next"]);
    }

    /// Scenario: a journal change contains no entry matching the configured
    /// filters.
    /// Guarantees: one non-matching wake yields to the worker command loop
    /// instead of restarting the full blocking timeout.
    #[test]
    fn follow_yields_after_nonmatching_append() {
        let mut journal = FakeJournal::new(
            Vec::new(),
            Some(FakeEntry {
                cursor: "new-unmatched",
                matches: false,
            }),
        );

        let delivered =
            follow(&mut journal, Duration::from_millis(3)).expect("follow non-matching append");

        assert_eq!(delivered, None);
        assert_eq!(journal.calls, ["next", "wait", "next"]);
        assert_eq!(journal.wait_timeouts, [Duration::from_millis(3)]);
        assert_eq!(journal.current_cursor(), Err("no current entry".to_owned()));
    }

    /// Scenario: journal rotation invalidates the file set without exposing a
    /// new matching entry at the current cursor.
    /// Guarantees: `SD_JOURNAL_INVALIDATE` is treated as a wakeup, followed by
    /// one `next()` retry and a yield to the worker command loop.
    #[test]
    fn follow_yields_after_invalidation_without_match() {
        let mut journal = FakeJournal::new(
            Vec::new(),
            Some(FakeEntry {
                cursor: "rotated-unmatched",
                matches: false,
            }),
        );
        journal.wait_change_rc = 2;

        let delivered =
            follow(&mut journal, Duration::from_millis(5)).expect("follow invalidation");

        assert_eq!(delivered, None);
        assert_eq!(journal.calls, ["next", "wait", "next"]);
        assert_eq!(journal.wait_timeouts, [Duration::from_millis(5)]);
    }

    /// Scenario: the journal remains idle until the configured wait expires.
    /// Guarantees: the follow helper returns `None` and forwards the exact
    /// timeout to the underlying journal wait call.
    #[test]
    fn follow_returns_none_when_wait_times_out() {
        let mut journal = FakeJournal::new(Vec::new(), None);

        let delivered =
            follow(&mut journal, Duration::from_millis(17)).expect("observe wait timeout");

        assert_eq!(delivered, None);
        assert_eq!(journal.calls, ["next", "wait"]);
        assert_eq!(journal.wait_timeouts, [Duration::from_millis(17)]);
    }

    /// Scenario: `sd_journal_next` returns a negative systemd error.
    /// Guarantees: the follow helper preserves the operation name and return
    /// code without calling `wait()`.
    #[test]
    fn follow_propagates_next_error() {
        let mut journal = FakeJournal::new(Vec::new(), None);
        journal.next_error = Some(-5);

        let error = follow(&mut journal, Duration::from_millis(1))
            .expect_err("next error should propagate");

        assert_eq!(error, "sd_journal_next failed with -5");
        assert_eq!(journal.calls, ["next"]);
    }

    /// Scenario: `sd_journal_wait` returns a negative systemd error after EOF.
    /// Guarantees: the follow helper preserves the operation name and return
    /// code after exactly one `next()` attempt.
    #[test]
    fn follow_propagates_wait_error() {
        let mut journal = FakeJournal::new(Vec::new(), None);
        journal.wait_error = Some(-22);

        let error = follow(&mut journal, Duration::from_millis(1))
            .expect_err("wait error should propagate");

        assert_eq!(error, "sd_journal_wait failed with -22");
        assert_eq!(journal.calls, ["next", "wait"]);
    }

    /// Scenario: reading the current entry fails after `sd_journal_next`
    /// successfully advances.
    /// Guarantees: current-entry decoding errors are propagated rather than
    /// converted into an idle result that would silently drop the entry.
    #[test]
    fn follow_propagates_current_entry_error() {
        let mut journal = FakeJournal::new(
            vec![FakeEntry {
                cursor: "matching",
                matches: true,
            }],
            None,
        );
        journal.current_error = Some("decode failed".to_owned());

        let error = follow(&mut journal, Duration::from_millis(1))
            .expect_err("current-entry error should propagate");

        assert_eq!(error, "decode failed");
        assert_eq!(journal.calls, ["next"]);
    }

    /// Scenario: a fresh `start_at: end` start where `sd_journal_previous`
    /// fails after the tail seek.
    /// Guarantees: the negative return is surfaced as a
    /// `("sd_journal_previous", rc)` error rather than being treated as an empty
    /// journal.
    #[test]
    fn end_propagates_previous_error() {
        let mut seek = RecordingSeek {
            previous_rc: -5,
            ..Default::default()
        };
        let err = position_for_fresh_start(&mut seek, StartAt::End).unwrap_err();
        assert_eq!(err, ("sd_journal_previous", -5));
        assert_eq!(seek.calls, ["seek_tail", "previous"]);
    }

    /// Scenario: a fresh `start_at: end` start where `sd_journal_seek_tail`
    /// fails.
    /// Guarantees: the error is propagated immediately and no `previous` step is
    /// attempted after a failed tail seek.
    #[test]
    fn end_propagates_seek_tail_error_without_stepping() {
        let mut seek = RecordingSeek {
            seek_tail_rc: -1,
            ..Default::default()
        };
        let err = position_for_fresh_start(&mut seek, StartAt::End).unwrap_err();
        assert_eq!(err, ("sd_journal_seek_tail", -1));
        assert_eq!(seek.calls, ["seek_tail"]);
    }

    /// Scenario: a fresh `start_at: end` start on an empty journal where the
    /// recovery `sd_journal_seek_head` fails.
    /// Guarantees: the failure is surfaced as a `seek_head` error, distinct from
    /// `seek_tail`/`previous` failures.
    #[test]
    fn end_propagates_seek_head_recovery_error() {
        // On the empty path (previous() == 0) a failing recovery seek_head()
        // surfaces as a seek_head error, distinct from seek_tail/previous.
        let mut seek = RecordingSeek {
            previous_rc: 0,
            seek_head_rc: -22,
            ..Default::default()
        };
        let err = position_for_fresh_start(&mut seek, StartAt::End).unwrap_err();
        assert_eq!(err, ("sd_journal_seek_head", -22));
        assert_eq!(seek.calls, ["seek_tail", "previous", "seek_head"]);
    }

    /// Scenario: a fresh `start_at: beginning` start where
    /// `sd_journal_seek_head` fails.
    /// Guarantees: the negative return is surfaced as a
    /// `("sd_journal_seek_head", rc)` error.
    #[test]
    fn beginning_propagates_seek_head_error() {
        let mut seek = RecordingSeek {
            seek_head_rc: -1,
            ..Default::default()
        };
        let err = position_for_fresh_start(&mut seek, StartAt::Beginning).unwrap_err();
        assert_eq!(err, ("sd_journal_seek_head", -1));
        assert_eq!(seek.calls, ["seek_head"]);
    }
}

#[cfg(target_os = "linux")]
mod imp {
    #![allow(unsafe_code)]

    use crate::receivers::journald_receiver::arrow_records_encoder::JournalEntry;
    use crate::receivers::journald_receiver::config::{ExtractionConfig, RuntimeConfig};
    use crate::receivers::journald_receiver::decode;

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
    type PreviousFn = unsafe extern "C" fn(*mut SdJournal) -> c_int;
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
        previous: PreviousFn,
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
                previous: sym!("sd_journal_previous", PreviousFn),
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
        fresh_start_position: Option<super::FreshStartPosition>,
    }

    impl SdJournalReader {
        pub(crate) fn open(
            config: &RuntimeConfig,
            checkpoint: Option<&str>,
        ) -> Result<Self, JournalError> {
            Self::open_inner(config, checkpoint, true)
        }

        pub(crate) fn open_for_rewind(
            config: &RuntimeConfig,
            checkpoint: Option<&str>,
        ) -> Result<Self, JournalError> {
            Self::open_inner(config, checkpoint, false)
        }

        fn open_inner(
            config: &RuntimeConfig,
            checkpoint: Option<&str>,
            run_preflight: bool,
        ) -> Result<Self, JournalError> {
            if run_preflight {
                preflight_journal_access(&config.journal.root_path)?;
            }
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
                fresh_start_position: None,
            };
            let data_threshold = extraction_data_threshold(&reader.extraction);
            check(
                unsafe { (reader.lib.set_data_threshold)(reader.journal.as_ptr(), data_threshold) },
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
            if let Some(cursor) = checkpoint {
                // Verify checkpoint existence against the unfiltered journal. If filters are
                // applied first, a normal unit/identifier/priority config change can hide the
                // checkpoint entry and make a valid cursor look stale. The verify step leaves the
                // read head on the committed entry; after installing matches, the worker's first
                // next() advances from that committed position to the first newly visible entry.
                self.verify_checkpoint_cursor(cursor)?;
                self.configure_matches(config)?;
                return Ok(());
            }

            self.configure_matches(config)?;

            // Position the read head for a fresh (no-checkpoint) start. For
            // `StartAt::End` this anchors past existing entries (and unparks an
            // empty journal from the tail) so the worker's next()/wait() loop
            // follows only newly appended records; see `position_for_fresh_start`.
            let position = super::position_for_fresh_start(self, config.start_at)
                .map_err(|(operation, rc)| JournalError::SystemdCall { operation, rc })?;
            self.fresh_start_position = Some(position);
            Ok(())
        }

        pub(crate) fn took_end_head_recovery(&self) -> bool {
            self.fresh_start_position == Some(super::FreshStartPosition::EndHeadRecovery)
        }

        fn configure_matches(&mut self, config: &RuntimeConfig) -> Result<(), JournalError> {
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
            Ok(())
        }

        fn verify_checkpoint_cursor(&mut self, cursor: &str) -> Result<(), JournalError> {
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
            Ok(())
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
            super::follow_next_entry(self, wait_timeout, Self::current_entry, |operation, rc| {
                JournalError::SystemdCall { operation, rc }
            })
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

            // Decode the entry's fields incrementally. `EnumerateData::next_field`
            // hands out each `sd_journal_enumerate_data` slice borrowed only for
            // the duration of the `feed` call (those buffers are invalidated by
            // the next enumerate call), and `FieldDecoder` copies every kept
            // field out before the next slice is fetched -- so the MESSAGE payload
            // is stored once, never cloned a second time for the log body.
            unsafe { (self.lib.restart_data)(self.journal.as_ptr()) };
            let reader: &SdJournalReader = self;
            let mut enumerate = EnumerateData::new(reader);
            let mut decoder = decode::FieldDecoder::new(&reader.extraction);
            while let Some(bytes) = enumerate.next_field() {
                decoder.feed(bytes);
            }
            if let Some(rc) = enumerate.error {
                return Err(JournalError::SystemdCall {
                    operation: "sd_journal_enumerate_data",
                    rc,
                });
            }
            let decoded = decoder.finish();

            Ok(JournalEntry {
                cursor,
                message_body_index: decoded.message_body_index,
                realtime_unix_nano: realtime_usec.saturating_mul(1000),
                fields: decoded.fields,
                dropped_fields: decoded.dropped_fields,
            })
        }
    }

    /// A lending reader over the current entry's raw `name=value` field slices,
    /// wrapping `sd_journal_enumerate_data`.
    ///
    /// Each slice returned by [`EnumerateData::next_field`] borrows journald-owned
    /// memory that is only valid until the *next* `sd_journal_enumerate_data`
    /// call. That contract is modelled precisely by borrowing `&mut self` for the
    /// returned slice's lifetime: the borrow checker forbids fetching the next
    /// field (or otherwise touching `self`) while a slice is still held, so a
    /// slice can never be aliased past the call that invalidates it. This is
    /// deliberately NOT a `std::iter::Iterator` -- a non-lending `Item` would let
    /// safe code `collect()` the slices or hold two at once, an immediate
    /// use-after-free.
    struct EnumerateData<'j> {
        reader: &'j SdJournalReader,
        /// First negative `sd_journal_enumerate_data` return code, if any.
        /// Enumeration ends on error; the caller inspects this afterward and maps
        /// it to a `JournalError`.
        error: Option<c_int>,
    }

    impl<'j> EnumerateData<'j> {
        fn new(reader: &'j SdJournalReader) -> Self {
            Self {
                reader,
                error: None,
            }
        }

        /// Fetch the next raw `name=value` field, or `None` at end-of-entry or on
        /// error (recorded in `self.error`). The returned slice borrows `self`,
        /// so it must be consumed before the next call.
        fn next_field(&mut self) -> Option<&[u8]> {
            if self.error.is_some() {
                return None;
            }
            let mut data: *const c_void = std::ptr::null();
            let mut len: size_t = 0;
            let rc = unsafe {
                (self.reader.lib.enumerate_data)(self.reader.journal.as_ptr(), &mut data, &mut len)
            };
            if rc < 0 {
                self.error = Some(rc);
                return None;
            }
            if rc == 0 {
                return None;
            }
            // SAFETY: on a positive return, `data`/`len` describe a buffer owned by
            // the journal, valid until the next `sd_journal_enumerate_data` call.
            // The returned slice borrows `&mut self`, so the borrow checker
            // prevents another `next_field` call (which would invalidate it) while
            // it is still held -- matching the FFI's documented lifetime exactly.
            Some(unsafe { std::slice::from_raw_parts(data.cast::<u8>(), len) })
        }
    }

    fn duration_to_usec(duration: Duration) -> u64 {
        if duration.is_zero() {
            return 0;
        }
        // `u64::MAX` is libsystemd's "wait indefinitely" sentinel, so a
        // saturating conversion must stop one microsecond below it.
        let usec = duration.as_micros().min((u64::MAX - 1) as u128) as u64;
        usec.max(1)
    }

    fn extraction_data_threshold(extraction: &ExtractionConfig) -> size_t {
        decode::extraction_data_threshold_u64(extraction).min(size_t::MAX as u64) as size_t
    }

    impl super::JournalSeek for SdJournalReader {
        fn seek_head(&mut self) -> c_int {
            unsafe { (self.lib.seek_head)(self.journal.as_ptr()) }
        }
        fn seek_tail(&mut self) -> c_int {
            unsafe { (self.lib.seek_tail)(self.journal.as_ptr()) }
        }
        fn previous(&mut self) -> c_int {
            unsafe { (self.lib.previous)(self.journal.as_ptr()) }
        }
    }

    impl super::JournalFollow for SdJournalReader {
        fn next(&mut self) -> c_int {
            unsafe { (self.lib.next)(self.journal.as_ptr()) }
        }

        fn wait(&mut self, wait_timeout: Duration) -> c_int {
            let timeout = duration_to_usec(wait_timeout);
            unsafe { (self.lib.wait)(self.journal.as_ptr(), timeout) }
        }
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

        check_journal_access_summary(root_path, summary)
    }

    fn check_journal_access_summary(
        root_path: &Path,
        summary: JournalAccessSummary,
    ) -> Result<(), JournalError> {
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
        if summary.visible_directories == 0 {
            return Err(JournalError::JournalDirectoriesMissing {
                root_path: root_path.to_path_buf(),
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

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::os::unix::fs::PermissionsExt;

        /// Scenario: wait durations exercise zero, sub-microsecond, and
        /// saturating conversions at the libsystemd boundary.
        /// Guarantees: finite durations never map to libsystemd's `u64::MAX`
        /// sentinel for an infinite wait.
        #[test]
        fn duration_conversion_avoids_infinite_wait_sentinel() {
            assert_eq!(duration_to_usec(Duration::ZERO), 0);
            assert_eq!(duration_to_usec(Duration::from_nanos(1)), 1);
            assert_eq!(duration_to_usec(Duration::MAX), u64::MAX - 1);
        }

        /// Scenario: preflight runs against a journal whose parent directory is
        /// not traversable (permission bits removed).
        /// Guarantees: it reports a `JournalAccess` error instead of silently
        /// succeeding or misreporting a missing directory.
        #[test]
        fn preflight_reports_access_when_journal_parent_is_not_traversable() {
            let temp = tempfile::tempdir().expect("tempdir");
            let root = temp.path();
            let log_dir = root.join("var/log");
            std::fs::create_dir_all(log_dir.join("journal")).expect("journal dir");
            std::fs::set_permissions(&log_dir, std::fs::Permissions::from_mode(0o000))
                .expect("chmod log dir");
            if std::fs::metadata(log_dir.join("journal")).is_ok() {
                std::fs::set_permissions(&log_dir, std::fs::Permissions::from_mode(0o755))
                    .expect("restore log dir permissions");
                return;
            }

            let result = preflight_journal_access(root);

            std::fs::set_permissions(&log_dir, std::fs::Permissions::from_mode(0o755))
                .expect("restore log dir permissions");
            assert!(matches!(result, Err(JournalError::JournalAccess { .. })));
        }

        /// Scenario: preflight summarizes access for the default root when the
        /// expected journal directories are absent.
        /// Guarantees: it reports `JournalDirectoriesMissing` rather than a
        /// generic access error.
        #[test]
        fn preflight_reports_missing_directories_for_default_root() {
            let result =
                check_journal_access_summary(Path::new("/"), JournalAccessSummary::default());

            assert!(matches!(
                result,
                Err(JournalError::JournalDirectoriesMissing { .. })
            ));
        }
    }
}

#[cfg(target_os = "linux")]
pub(crate) use imp::{JournalError, SdJournalReader};
