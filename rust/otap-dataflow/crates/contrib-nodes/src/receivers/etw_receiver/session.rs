// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! ETW session management using the `one_collect` library.
//!
//! ## Singleton session with round-robin fan-out
//!
//! Windows allows only **one** real-time ETW trace session per session name.
//! The OTAP engine, however, may create multiple receiver replicas (one per
//! allocated core).  To reconcile these two models this module maintains a
//! **process-global singleton session** and pre-creates N consumer channels
//! (one per core).  Each factory call pops one receiver from the pool.
//!
//! ```text
//! ProcessTrace OS thread  (lazily spawned on first factory call)
//! callback: txs[next].try_send(data); next = (next + 1) % N
//!       |          |          |
//!     tx[0]      tx[1]      tx[2]
//!       v          v          v
//!      mpsc       mpsc       mpsc
//!       v          v          v
//!  +--------+ +--------+ +--------+
//!  | core 0 | | core 1 | | core 2 |
//!  | rx[0]  | | rx[1]  | | rx[2]  |
//!  +--------+ +--------+ +--------+
//! ```
//!
//! ## Lifecycle
//!
//! The session lives until the process exits.  Dropping individual receivers
//! only closes their channel; the `ProcessTrace` thread continues delivering
//! events to the remaining senders.  When **all** senders have been dropped
//! (i.e. no receivers remain) the callback becomes a no-op.

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use one_collect::etw::{self, EVENT_RECORD, EtwSession};
use one_collect::Guid;
use otap_df_engine::error::Error;
use tokio::sync::mpsc;

use super::{Config, ProviderConfig, TraceLevel};

/// Channel capacity for ETW events sent from the blocking session thread to
/// each per-core async receiver loop.  A bounded channel provides implicit
/// backpressure: when a core's channel is full the round-robin callback skips
/// that core for the current event (the event is dropped for that core only).
const EVENT_CHANNEL_CAPACITY: usize = 4096;

// ── Event data transferred across the channel ────────────────────────────────

/// Lightweight snapshot of an ETW event captured in the `ProcessTrace` callback.
///
/// Because the `EVENT_RECORD` pointer is only valid for the duration of the
/// callback, we copy the fields we need into this owned struct before sending
/// it across the channel to the async world.
#[derive(Debug, Clone)]
pub struct EtwEventData {
    /// Provider GUID that produced the event.
    pub provider_id: [u8; 16],
    /// ETW event timestamp (QPC ticks from `EVENT_HEADER.TimeStamp`).
    pub timestamp: u64,
    /// Process ID from the event header.
    pub process_id: u32,
    /// Thread ID from the event header.
    pub thread_id: u32,
    /// Event ID from the event descriptor.
    pub event_id: u16,
    /// Opcode from the event descriptor.
    pub opcode: u8,
    /// Version from the event descriptor.
    pub version: u8,
    /// ETW level from the event descriptor.
    pub level: u8,
    /// Keywords from the event descriptor.
    pub keywords: u64,
}

// ── GUID parsing ─────────────────────────────────────────────────────────────

/// Parse a GUID string in the standard `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`
/// format (with or without surrounding braces/hyphens) into a
/// [`one_collect::Guid`].
fn parse_guid(s: &str) -> Result<Guid, Error> {
    // Strip optional braces.
    let s = s.trim().trim_start_matches('{').trim_end_matches('}');

    // Collect only hex digits.
    let hex: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();

    if hex.len() != 32 {
        return Err(Error::InternalError {
            message: format!(
                "invalid GUID string '{s}': expected 32 hex digits, got {}",
                hex.len()
            ),
        });
    }

    let val = u128::from_str_radix(&hex, 16).map_err(|e| Error::InternalError {
        message: format!("invalid GUID string '{s}': {e}"),
    })?;

    Ok(Guid::from_u128(val))
}

/// Map the receiver's [`TraceLevel`] to the `one_collect` ETW level constant.
const fn trace_level_to_etw(level: &TraceLevel) -> u8 {
    match level {
        TraceLevel::Critical => etw::LEVEL_CRITICAL,
        TraceLevel::Error => etw::LEVEL_ERROR,
        TraceLevel::Warning => etw::LEVEL_WARNING,
        TraceLevel::Information => etw::LEVEL_INFORMATION,
        TraceLevel::Verbose => etw::LEVEL_VERBOSE,
    }
}

/// Resolve a [`ProviderConfig`] to a [`Guid`].
///
/// If the provider specifies a `guid` string it is parsed directly.
/// If it specifies a `name`, provider-name-to-GUID resolution is not yet
/// implemented and an error is returned with guidance.
fn resolve_provider_guid(cfg: &ProviderConfig) -> Result<Guid, Error> {
    if let Some(guid_str) = &cfg.guid {
        return parse_guid(guid_str);
    }

    if let Some(name) = &cfg.name {
        // TODO: Implement provider name → GUID resolution via
        // TdhEnumerateProviders or registry lookup.
        return Err(Error::InternalError {
            message: format!(
                "provider name resolution is not yet implemented; \
                 please specify a GUID instead of name '{name}'. \
                 You can find a provider's GUID via `logman query providers \"{name}\"`"
            ),
        });
    }

    Err(Error::InternalError {
        message: "provider must specify either 'name' or 'guid'".to_string(),
    })
}

// ── Singleton session state ──────────────────────────────────────────────────

/// Process-global session state.  Initialised on the first call to
/// [`take_consumer`]; subsequent calls pop one receiver from the pool.
///
/// We use `Mutex<Option<Vec<…>>>` rather than `OnceLock` / `LazyLock` because:
/// - Initialisation is fallible (GUID parsing, thread spawn).
/// - We need post-init mutation (`Vec::pop`).
static SESSION: Mutex<Option<Vec<mpsc::Receiver<EtwEventData>>>> = Mutex::new(None);

/// Build the event data snapshot from a raw [`EVENT_RECORD`].
///
/// # Safety
///
/// The caller must guarantee the `EVENT_RECORD` pointer inside the reference
/// is valid.  This is always the case when called from the `ProcessTrace`
/// callback.
fn build_event_data(event: &EVENT_RECORD) -> EtwEventData {
    EtwEventData {
        provider_id: event.EventHeader.ProviderId.to_bytes(),
        timestamp: event.EventHeader.TimeStamp,
        process_id: event.EventHeader.ProcessId,
        thread_id: event.EventHeader.ThreadId,
        event_id: event.EventHeader.EventDescriptor.Id,
        opcode: event.EventHeader.EventDescriptor.Opcode,
        version: event.EventHeader.EventDescriptor.Version,
        level: event.EventHeader.EventDescriptor.Level,
        keywords: event.EventHeader.EventDescriptor.Keyword,
    }
}

/// Spawn the ETW session thread with N senders for round-robin fan-out.
///
/// The thread creates the `EtwSession`, enables the configured providers,
/// installs a raw event callback that round-robins across `txs`, and calls
/// `parse_until` (blocking).  The session lives until the process exits.
fn spawn_etw_thread(
    config: &Config,
    txs: Vec<mpsc::Sender<EtwEventData>>,
) -> Result<(), Error> {
    // Resolve all provider GUIDs up-front so configuration errors are
    // reported synchronously (before the thread is spawned).
    let resolved_providers: Vec<(Guid, u8, Option<u64>)> = config
        .providers
        .iter()
        .map(|p| {
            let guid = resolve_provider_guid(p)?;
            let level = trace_level_to_etw(&p.level);
            Ok((guid, level, p.keywords))
        })
        .collect::<Result<Vec<_>, Error>>()?;

    let session_name = config.session_name.clone();

    let _join_handle = std::thread::Builder::new()
        .name(format!("etw-session-{session_name}"))
        .spawn(move || {
            let mut session = EtwSession::new();

            for (guid, level, keywords) in &resolved_providers {
                let enabler = session.enable_provider(*guid);
                enabler.ensure_level(*level);
                if let Some(kw) = keywords {
                    enabler.ensure_keyword(*kw);
                }
            }

            // Round-robin index, atomically updated from the callback.
            // Using `AtomicUsize` even though the callback runs on a single
            // thread because the closure is `Fn` (not `FnMut`); interior
            // mutability via atomics avoids requiring a `Mutex` in the
            // hot path.
            let next = AtomicUsize::new(0);
            let num_txs = txs.len();

            session.set_raw_event_callback(move |event| {
                let data = build_event_data(event);
                let idx = next.fetch_add(1, Ordering::Relaxed) % num_txs;
                // Best-effort send; if this core's channel is full, drop the
                // event for that core only.  Other cores are unaffected.
                let _ = txs[idx].try_send(data);
            });

            // `parse_until` blocks on `ProcessTrace`.  We never signal stop,
            // so the session runs until the process exits (all senders are
            // dropped when the thread exits / process terminates).
            let _result = session.parse_until(&session_name, || false);

            // The session thread exits only on unrecoverable ETW errors or
            // process shutdown.  When it exits, all senders are dropped,
            // closing the channels and signalling the async receiver loops.
        })
        .map_err(|e| Error::InternalError {
            message: format!("failed to spawn ETW session thread: {e}"),
        })?;

    // The JoinHandle is intentionally leaked — the session thread runs for
    // the lifetime of the process.
    drop(_join_handle);

    Ok(())
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Acquire one consumer channel from the process-global ETW session.
///
/// On the **first** call, this function:
/// 1. Creates `num_cores` bounded MPSC channels.
/// 2. Spawns the ETW session thread with round-robin fan-out across all
///    senders.
/// 3. Stores the receivers in a process-global pool.
///
/// On each call (including the first) it pops one receiver from the pool and
/// returns it.  The engine calls the receiver factory once per allocated core,
/// so exactly `num_cores` calls are expected.
///
/// # Errors
///
/// Returns an error if:
/// - Provider GUID parsing fails (first call only).
/// - The ETW session thread cannot be spawned (first call only).
/// - All consumers have already been handed out (more calls than `num_cores`).
/// - The session lock is poisoned (indicates a prior panic).
pub fn take_consumer(
    config: &Config,
    num_cores: usize,
) -> Result<mpsc::Receiver<EtwEventData>, Error> {
    let mut guard = SESSION.lock().map_err(|e| Error::InternalError {
        message: format!("ETW session lock poisoned: {e}"),
    })?;

    if guard.is_none() {
        // First call — initialise the session.
        let (txs, rxs): (Vec<_>, Vec<_>) = (0..num_cores)
            .map(|_| mpsc::channel(EVENT_CHANNEL_CAPACITY))
            .unzip();

        spawn_etw_thread(config, txs)?;

        *guard = Some(rxs);
    }

    guard
        .as_mut()
        .and_then(|rxs| rxs.pop())
        .ok_or_else(|| Error::InternalError {
            message: "all ETW consumer channels have been handed out; \
                      num_cores mismatch or duplicate factory calls"
                .to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_guid_standard_format() {
        let guid = parse_guid("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716").expect("valid GUID");
        assert_eq!(guid.data1, 0x22fb2cd6);
        assert_eq!(guid.data2, 0x0e7b);
        assert_eq!(guid.data3, 0x422b);
        assert_eq!(
            guid.data4,
            [0xa0, 0xc7, 0x2f, 0xad, 0x1f, 0xd0, 0xe7, 0x16]
        );
    }

    #[test]
    fn parse_guid_with_braces() {
        let guid = parse_guid("{22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716}").expect("valid GUID");
        assert_eq!(guid.data1, 0x22fb2cd6);
    }

    #[test]
    fn parse_guid_uppercase() {
        let guid = parse_guid("22FB2CD6-0E7B-422B-A0C7-2FAD1FD0E716").expect("valid GUID");
        assert_eq!(guid.data1, 0x22fb2cd6);
    }

    #[test]
    fn parse_guid_invalid_length() {
        let result = parse_guid("22fb2cd6-0e7b");
        assert!(result.is_err());
    }

    #[test]
    fn trace_level_mapping() {
        assert_eq!(
            trace_level_to_etw(&TraceLevel::Critical),
            etw::LEVEL_CRITICAL
        );
        assert_eq!(trace_level_to_etw(&TraceLevel::Error), etw::LEVEL_ERROR);
        assert_eq!(
            trace_level_to_etw(&TraceLevel::Warning),
            etw::LEVEL_WARNING
        );
        assert_eq!(
            trace_level_to_etw(&TraceLevel::Information),
            etw::LEVEL_INFORMATION
        );
        assert_eq!(
            trace_level_to_etw(&TraceLevel::Verbose),
            etw::LEVEL_VERBOSE
        );
    }
}