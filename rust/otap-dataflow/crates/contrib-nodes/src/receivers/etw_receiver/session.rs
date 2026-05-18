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
//! ## Integration with `one_collect`
//!
//! Instead of using the low-level `set_raw_event_callback` (which bypasses
//! `one_collect`'s event routing), we register a catch-all `Event` per
//! provider via [`EtwSession::add_event`].  This fires for every event from
//! a given provider GUID, regardless of event ID.  Header metadata (PID,
//! TID, timestamp, etc.) is read from the session's [`AncillaryData`] which
//! `one_collect` populates before each dispatch.
//!
//! ## Lifecycle
//!
//! The session lives until the process exits.  Dropping individual receivers
//! only closes their channel; the `ProcessTrace` thread continues delivering
//! events to the remaining senders.  When **all** senders have been dropped
//! (i.e. no receivers remain) the callback becomes a no-op.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Mutex;

use one_collect::etw::{self, EtwSession};
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
/// format into a [`one_collect::Guid`].
///
/// Only hex digits and hyphens in the canonical positions are accepted.
fn parse_guid(s: &str) -> Result<Guid, Error> {
    let s = s.trim();

    // Validate the exact format: 8-4-4-4-12 hex digits separated by hyphens.
    let parts: Vec<&str> = s.split('-').collect();
    let expected_lengths = [8, 4, 4, 4, 12];

    if parts.len() != 5
        || !parts
            .iter()
            .zip(expected_lengths.iter())
            .all(|(part, &len)| part.len() == len && part.chars().all(|c| c.is_ascii_hexdigit()))
    {
        return Err(Error::ConfigError(Box::new(
            otap_df_config::error::Error::InvalidUserConfig {
                error: format!(
                    "invalid GUID '{s}': expected format xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
                ),
            },
        )));
    }

    // Concatenate hex parts and parse.
    let hex: String = parts.concat();
    let val = u128::from_str_radix(&hex, 16).map_err(|e| {
        Error::ConfigError(Box::new(
            otap_df_config::error::Error::InvalidUserConfig {
                error: format!("invalid GUID '{s}': {e}"),
            },
        ))
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
///
/// # Panics
///
/// Panics (debug builds only) if both `name` and `guid` are set, or if
/// neither is set.  These cases are prevented by [`Config::validate`],
/// which must be called before this function.
fn resolve_provider_guid(cfg: &ProviderConfig) -> Result<Guid, Error> {
    debug_assert!(
        cfg.name.is_some() != cfg.guid.is_some(),
        "Config::validate must be called before resolve_provider_guid; \
         expected exactly one of 'name' or 'guid', got name={:?}, guid={:?}",
        cfg.name,
        cfg.guid
    );

    if let Some(guid_str) = &cfg.guid {
        return parse_guid(guid_str);
    }

    if let Some(name) = &cfg.name {
        // TODO: Implement provider name → GUID resolution via
        // TdhEnumerateProviders or registry lookup.
        return Err(Error::ConfigError(Box::new(
            otap_df_config::error::Error::InvalidUserConfig {
                error: format!(
                    "provider name resolution is not yet implemented; \
                     please specify a GUID instead of name '{name}'. \
                     You can find a provider's GUID via `logman query providers \"{name}\"`"
                ),
            },
        )));
    }

    unreachable!("validated upstream: provider must specify either 'name' or 'guid'")
}

// ── Singleton session state ──────────────────────────────────────────────────

/// Process-global session state.  Initialized on the first call to
/// [`subscribe`]; subsequent calls pop one receiver from the pool.
///
/// We use `Mutex<Option<Vec<…>>>` rather than `OnceLock` / `LazyLock` because:
/// - Initialization is fallible (GUID parsing, thread spawn).
/// - We need post-init mutation (`Vec::pop`).
static SESSION: Mutex<Option<Vec<mpsc::Receiver<EtwEventData>>>> = Mutex::new(None);

/// Spawn the ETW session and block on `parse_until`.
///
/// This function:
/// 1. Creates an `EtwSession`.
/// 2. Enables each configured provider.
/// 3. Registers a **provider-wide event** (catch-all) per provider that uses
///    `AncillaryData` to extract header fields and round-robins the resulting
///    `EtwEventData` across the N senders.
/// 4. Calls `parse_until` which blocks until the process exits.
fn spawn_etw_session(
    config: &Config,
    txs: Vec<mpsc::Sender<EtwEventData>>,
) -> Result<(), Error> {
    // Resolve all provider GUIDs up-front so configuration errors are
    // reported synchronously (before the session thread is spawned).
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

    // Detach the session thread; it runs for the lifetime of the process.
    let _ = std::thread::Builder::new()
        .name(format!("etw-session-{session_name}"))
        .spawn(move || {
            let mut session = EtwSession::new();

            // Enable each configured provider.
            for (guid, level, keywords) in &resolved_providers {
                let enabler = session.enable_provider(*guid);
                enabler.ensure_level(*level);
                if let Some(kw) = keywords {
                    enabler.ensure_keyword(*kw);
                }
            }

            // Obtain the ancillary data handle.  `one_collect` populates this
            // with the current EVENT_RECORD's header fields (PID, TID,
            // timestamp, provider GUID, etc.) before dispatching each event.
            let ancillary = session.ancillary_data();

            // Shared round-robin counter and sender list.  All provider
            // callbacks run on the single `ProcessTrace` thread, so `Cell`
            // is safe — no atomics or locking needed.  Sharing the counter
            // ensures uniform core distribution even at startup when
            // multiple providers would otherwise all start at index 0.
            let next: Rc<Cell<usize>> = Rc::new(Cell::new(0));
            let txs: Rc<Vec<mpsc::Sender<EtwEventData>>> = Rc::new(txs);

            // Register a provider-wide event for each configured provider.
            // A "wide event" fires for ALL event IDs from the provider,
            // unlike `add_event` which only fires for a specific event ID.
            for (guid, level, keywords) in &resolved_providers {
                let mut wide_event = one_collect::event::Event::new(0, "otap_wide".to_string());
                {
                    let ext = wide_event.extension_mut();
                    *ext.provider_mut() = *guid;
                    *ext.level_mut() = *level;
                    *ext.keyword_mut() = keywords.unwrap_or(0);
                }

                let ancillary = ancillary.clone();
                let next = Rc::clone(&next);
                let txs = Rc::clone(&txs);

                wide_event.add_callback(move |_event_data| {
                    // Read header metadata from AncillaryData (populated
                    // by one_collect before each dispatch).
                    let anc = ancillary.borrow();

                    // Build EtwEventData from AncillaryData.
                    // PID, TID, timestamp, provider, and opcode are
                    // available directly; event_id/version/level/keywords
                    // come from the full_data bytes passed via EventData.
                    let data = EtwEventData {
                        provider_id: anc.provider().to_bytes(),
                        timestamp: anc.time(),
                        process_id: anc.pid(),
                        thread_id: anc.tid(),
                        // TODO: populate event_id/opcode/level/keywords/version
                        // once WindowsEventExtension exposes EVENT_DESCRIPTOR.
                        event_id: 0,
                        opcode: 0,
                        version: 0,
                        level: 0,
                        keywords: 0,
                    };

                    // Drop the borrow before sending.
                    drop(anc);

                    let i = next.get();
                    next.set(i.wrapping_add(1));

                    // Best-effort send; if this core's channel is full,
                    // drop the event for that core only.
                    let _ = txs[i % txs.len()].try_send(data);

                    Ok(())
                });

                session.add_event(wide_event, None);
            }

            // `parse_until` blocks on `ProcessTrace`.  We never signal stop,
            // so the session runs until the process exits.
            let _result = session.parse_until(&session_name, || false);

            // The session thread exits only on unrecoverable ETW errors or
            // process shutdown.  When it exits, all senders are dropped,
            // closing the channels and signalling the async receiver loops.
        })
        .map_err(|e| Error::InternalError {
            message: format!("failed to spawn ETW session thread: {e}"),
        })?;

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
pub(super) fn subscribe(
    config: &Config,
    num_cores: usize,
) -> Result<mpsc::Receiver<EtwEventData>, Error> {
    let mut guard = SESSION.lock().map_err(|e| Error::InternalError {
        message: format!("ETW session lock poisoned: {e}"),
    })?;

    if guard.is_none() {
        // First call — initialize the session.
        let (txs, rxs): (Vec<_>, Vec<_>) = (0..num_cores)
            .map(|_| mpsc::channel(EVENT_CHANNEL_CAPACITY))
            .unzip();

        spawn_etw_session(config, txs)?;

        *guard = Some(rxs);
    }

    let pool = guard.as_mut().expect("session initialized above");
    pool.pop().ok_or_else(|| Error::InternalError {
        message: "ETW consumer pool exhausted; engine requested more receivers than num_cores"
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
    fn parse_guid_rejects_braces() {
        let result = parse_guid("{22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716}");
        assert!(result.is_err());
    }

    #[test]
    fn parse_guid_uppercase() {
        let guid = parse_guid("22FB2CD6-0E7B-422B-A0C7-2FAD1FD0E716").expect("valid GUID");
        assert_eq!(guid.data1, 0x22fb2cd6);
        assert_eq!(guid.data2, 0x0e7b);
        assert_eq!(guid.data3, 0x422b);
        assert_eq!(guid.data4, [0xa0, 0xc7, 0x2f, 0xad, 0x1f, 0xd0, 0xe7, 0x16]);
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