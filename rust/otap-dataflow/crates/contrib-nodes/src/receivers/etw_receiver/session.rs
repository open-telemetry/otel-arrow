// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! ETW session management using the `one_collect` library.
//!
//! ## One session per `session_name` with round-robin fan-out
//!
//! Windows allows only **one** real-time ETW trace session per session name.
//! The OTAP engine, however, may create multiple receiver replicas (one per
//! allocated core).  To reconcile these two models this module maintains one
//! session per `session_name` and pre-creates N consumer channels (one per
//! core).  Each factory call pops one receiver from the pool.
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
//! ## TDH Decoding
//!
//! For TraceLogging and TraceLoggingDynamic events the callback uses
//! [`one_collect::etw::tdh::TdhDecoder`] to discover the event schema at
//! runtime via the Windows TDH APIs.  The decoder maintains a schema cache
//! so that repeated events with the same layout avoid kernel transitions.
//! Decoded field data is copied into [`DecodedField`] structs and sent
//! across the channel alongside the event header metadata.
//!
//! ## Lifecycle
//!
//! The session lives until the process exits.  Dropping individual receivers
//! only closes their channel; the `ProcessTrace` thread continues delivering
//! events to the remaining senders.  When **all** senders have been dropped
//! (i.e. no receivers remain) the callback becomes a no-op.

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use one_collect::Guid;
use one_collect::etw::tdh::{TdhDecodeError, TdhDecoder};
use one_collect::etw::{self, EtwSession};
use otap_df_engine::error::Error;
use otap_df_telemetry::{otel_error, otel_info, otel_warn};
use tokio::sync::mpsc;

use super::{Config, ProviderConfig, TraceLevel};

// ── QPC → Unix epoch conversion ──────────────────────────────────────────────

/// Reference point captured once at session start to convert QPC ticks to
/// Unix epoch nanoseconds.  All three values are sampled on the session
/// thread before `parse_until` enters the `ProcessTrace` loop.
#[derive(Debug, Clone, Copy)]
struct QpcReference {
    /// QPC tick value at reference time.
    qpc_at_ref: u64,
    /// QPC frequency (ticks per second).
    qpc_frequency: u64,
    /// Unix epoch nanoseconds at reference time.
    unix_ns_at_ref: i64,
}

impl QpcReference {
    /// Capture a QPC reference point using Win32 APIs.
    ///
    /// # Safety
    ///
    /// Calls `QueryPerformanceCounter` and `QueryPerformanceFrequency`,
    /// which are always safe to call on Windows.
    #[allow(unsafe_code)]
    fn capture() -> Self {
        // Use windows-sys types for QPC
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn QueryPerformanceCounter(lp: *mut i64) -> i32;
            fn QueryPerformanceFrequency(lp: *mut i64) -> i32;
        }

        let mut qpc: i64 = 0;
        let mut freq: i64 = 0;

        // SAFETY: These Win32 APIs are always safe to call; they write to
        // valid stack-allocated i64 pointers.
        unsafe {
            let _ = QueryPerformanceCounter(&mut qpc);
            let _ = QueryPerformanceFrequency(&mut freq);
        }

        let unix_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;

        Self {
            qpc_at_ref: qpc as u64,
            qpc_frequency: freq.max(1) as u64,
            unix_ns_at_ref: unix_ns,
        }
    }

    /// Convert a QPC tick value to Unix epoch nanoseconds.
    fn qpc_to_unix_ns(self, qpc_ticks: u64) -> i64 {
        // delta_ticks can be negative if the event was captured slightly
        // before our reference point (race between QPC and wall clock).
        let delta_ticks = qpc_ticks as i128 - self.qpc_at_ref as i128;
        let delta_ns = delta_ticks * 1_000_000_000 / self.qpc_frequency as i128;
        self.unix_ns_at_ref.saturating_add(delta_ns as i64)
    }
}

/// Channel capacity for ETW events sent from the blocking session thread to
/// each per-core async receiver loop.  A bounded channel provides implicit
/// backpressure: when the target core's channel is full the event is dropped
/// from the pipeline entirely.  The round-robin index still advances, so the
/// next event continues to the following core (no retry on another core).
const EVENT_CHANNEL_CAPACITY: usize = 4096;

// ── Event data transferred across the channel ────────────────────────────────

/// A single decoded field from a TDH-decoded TraceLogging event.
///
/// During the `ProcessTrace` callback the raw `EVENT_RECORD` is still valid,
/// so we copy each field's bytes into an owned `Vec<u8>` before sending the
/// event across the channel.
#[derive(Debug, Clone)]
pub struct DecodedField {
    /// Field name (e.g. `"ProcessId"`, or `"Parent.ChildField"` for nested structs).
    pub name: String,
    /// Type name matching one_collect conventions (e.g. `"u32"`, `"string"`, `"wstring"`).
    pub type_name: String,
    /// Raw field data bytes copied from the event payload.
    /// Empty for unsupported or zero-length fields.
    pub data: Vec<u8>,
}

/// Lightweight snapshot of an ETW event captured in the `ProcessTrace` callback.
///
/// Because the `EVENT_RECORD` pointer is only valid for the duration of the
/// callback, we copy the fields we need into this owned struct before sending
/// it across the channel to the async world.
#[derive(Debug, Clone)]
pub struct EtwEventData {
    /// Provider GUID that produced the event (16 raw bytes).
    pub provider_id: [u8; 16],
    /// ETW event timestamp converted to Unix epoch nanoseconds.
    ///
    /// Derived from `EVENT_HEADER.TimeStamp` (QPC ticks) using a reference
    /// point captured at session start via `QueryPerformanceCounter` and
    /// `SystemTime::now()`.
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
    /// TraceLogging event name discovered via TDH (e.g. `"AppStarted"`).
    ///
    /// Empty for manifest-based events or when TDH decoding fails.
    pub event_name: String,
    /// Activity ID from the event header for correlating related events.
    ///
    /// All zeros when the provider does not set an activity ID.
    pub activity_id: [u8; 16],
    /// TDH-decoded event payload fields.
    ///
    /// Populated for TraceLogging / TraceLoggingDynamic events whose schema
    /// can be discovered via TDH.  Empty for manifest-based events (which
    /// will be supported in a future extension) or when decoding fails.
    pub decoded_fields: Vec<DecodedField>,
    /// Raw copy of the event's `UserData` payload.
    ///
    /// Provided for consumers that need access to the full uninterpreted
    /// event bytes (e.g. for binary-level forwarding or custom decoding).
    #[expect(
        dead_code,
        reason = "retained for future binary-level forwarding or manifest decoding"
    )]
    pub user_data: Vec<u8>,
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
        Error::ConfigError(Box::new(otap_df_config::error::Error::InvalidUserConfig {
            error: format!("invalid GUID '{s}': {e}"),
        }))
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

// ── TDH field extraction ─────────────────────────────────────────────────────

/// Extract decoded fields from a TDH-decoded event.
///
/// For each field in the event format, this function uses the format's
/// `try_get_field_data_closure` to correctly resolve dynamic offsets
/// (e.g. for null-terminated strings that shift subsequent field positions)
/// and copies the field data into an owned [`DecodedField`].
///
/// # Safety
///
/// This function is called during the `ProcessTrace` callback while
/// the `EVENT_RECORD` (and its `UserData`) is still valid.
fn extract_decoded_fields(
    format: &one_collect::event::EventFormat,
    event_data: &[u8],
) -> Vec<DecodedField> {
    let mut fields = Vec::with_capacity(format.fields().len());

    for field in format.fields() {
        // `try_get_field_data_closure` may panic with `todo!()` for
        // unsupported LocationType variants (e.g. DynAbsolute).
        // Since we're called from an `extern "system"` ETW callback
        // that cannot unwind, we must catch any panic here to prevent
        // the process from aborting.
        let data = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if let Some(mut data_fn) = format.try_get_field_data_closure(&field.name) {
                data_fn(event_data).to_vec()
            } else {
                Vec::new()
            }
        }))
        .unwrap_or_default();

        fields.push(DecodedField {
            name: field.name.clone(),
            type_name: field.type_name.clone(),
            data,
        });
    }

    fields
}

// ── Per-session state ────────────────────────────────────────────────────────

/// State for a single ETW session keyed by `session_name`.
struct SessionEntry {
    /// The config used to create this session.  Subsequent `subscribe()`
    /// calls for the same `session_name` must present an identical config;
    /// a mismatch indicates a misconfigured pipeline (two different
    /// `receiver:etw` nodes accidentally sharing a `session_name`).
    config: Config,
    /// Pre-allocated consumer channels, one per core.  Popped one at a time
    /// as each per-core receiver factory call arrives.
    pool: Vec<mpsc::Receiver<EtwEventData>>,
}

/// Process-global session registry.  Keyed by `session_name` so that:
///
/// - Two `receiver:etw` nodes with **different** `session_name`s each get
///   their own independent kernel session.
/// - Two nodes with the **same** `session_name` produce a clear
///   `InvalidUserConfig` error instead of silently sharing or failing with
///   a misleading "pool exhausted" message.
///
/// We use `Mutex<HashMap<…>>` rather than `OnceLock` / `LazyLock` because:
/// - Initialization is fallible (GUID parsing, thread spawn).
/// - We need post-init mutation (`Vec::pop`).
static SESSIONS: Mutex<Option<HashMap<String, SessionEntry>>> = Mutex::new(None);

/// Spawn the ETW session and block on `parse_until`.
///
/// This function:
/// 1. Creates an `EtwSession`.
/// 2. Enables each configured provider.
/// 3. Registers a **provider-wide event** (catch-all) per provider that uses
///    `AncillaryData` to extract header fields and round-robins the resulting
///    `EtwEventData` across the N senders.
/// 4. Creates a shared [`TdhDecoder`] for runtime schema discovery of
///    TraceLogging events.
/// 5. Calls `parse_until` which blocks until the process exits.
#[allow(unsafe_code)]
fn spawn_etw_session(config: &Config, txs: Vec<mpsc::Sender<EtwEventData>>) -> Result<(), Error> {
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

            // Shared round-robin counter, drop counter, and sender list.
            // All provider callbacks run on the single `ProcessTrace` thread,
            // so `Cell` is safe — no atomics or locking needed.  Sharing the
            // counter ensures uniform core distribution even at startup when
            // multiple providers would otherwise all start at index 0.
            let next: Rc<Cell<usize>> = Rc::new(Cell::new(0));
            let dropped: Rc<Cell<u64>> = Rc::new(Cell::new(0));
            let closed_logged: Rc<Vec<Cell<bool>>> =
                Rc::new((0..txs.len()).map(|_| Cell::new(false)).collect());
            let txs: Rc<Vec<mpsc::Sender<EtwEventData>>> = Rc::new(txs);

            // TDH decoder shared across all provider callbacks.
            // All callbacks run on the single ProcessTrace thread, so
            // Rc<RefCell<>> is safe (no cross-thread access).
            let decoder: Rc<RefCell<TdhDecoder>> = Rc::new(RefCell::new(TdhDecoder::new()));

            // Capture QPC reference point for timestamp conversion.
            // This is done on the session thread just before parse_until
            // enters the ProcessTrace loop.
            let qpc_ref = QpcReference::capture();

            // Register a provider-wide event for each configured provider.
            // A "wide event" fires for ALL event IDs from the provider,
            // unlike `add_event` which only fires for a specific event ID.
            for (guid, level, keywords) in &resolved_providers {
                let mut wide_event = one_collect::event::Event::new(0, "otap_wide".to_string());
                // Mark as a wildcard event so the callback fires for ALL
                // event IDs from this provider, not just event ID 0.
                wide_event.set_id_wild_card_flag();
                {
                    let ext = wide_event.extension_mut();
                    *ext.provider_mut() = *guid;
                    *ext.level_mut() = *level;
                    *ext.keyword_mut() = keywords.unwrap_or(0);
                }

                let ancillary = ancillary.clone();
                let next = Rc::clone(&next);
                let dropped = Rc::clone(&dropped);
                let closed_logged = Rc::clone(&closed_logged);
                let txs = Rc::clone(&txs);
                let decoder = Rc::clone(&decoder);

                wide_event.add_callback(move |_event_data| {
                    // Read header metadata from AncillaryData (populated
                    // by one_collect before each dispatch).
                    let anc = ancillary.borrow();

                    // Extract event descriptor fields from the raw EVENT_RECORD.
                    // AncillaryData exposes id/opcode/version directly; for
                    // level and keywords we read from the EVENT_RECORD pointer.
                    let event_id = anc.id();
                    let opcode = anc.op_code();
                    let version = anc.version();

                    let (level, keywords) = match anc.record() {
                        Some(record) => (
                            record.EventHeader.EventDescriptor.Level,
                            record.EventHeader.EventDescriptor.Keyword,
                        ),
                        None => (0, 0),
                    };

                    // TDH decode: attempt to decode TraceLogging event schema.
                    // For manifest-based events (NotFound) we proceed with
                    // empty decoded_fields — future work will add manifest
                    // decoding with a (Provider, Id, Version) cache key.
                    let (decoded_fields, event_name, user_data) = if let Some(record) = anc.record()
                    {
                        let ud = if record.UserData.is_null() || record.UserDataLength == 0 {
                            Vec::new()
                        } else {
                            unsafe {
                                std::slice::from_raw_parts(
                                    record.UserData as *const u8,
                                    record.UserDataLength as usize,
                                )
                            }
                            .to_vec()
                        };

                        let (fields, name) = match decoder.borrow_mut().decode(record) {
                            Ok(result) => {
                                let name = result
                                    .event_data
                                    .format()
                                    .fields()
                                    .first()
                                    .map(|_| {
                                        // Use the TDH event name from the decoder's
                                        // schema cache (populated during decode).
                                        String::new()
                                    })
                                    .unwrap_or_default();
                                let fields = extract_decoded_fields(
                                    result.event_data.format(),
                                    result.event_data.event_data(),
                                );
                                (fields, name)
                            }
                            Err(TdhDecodeError::NotFound) => (Vec::new(), String::new()),
                            Err(_e) => (Vec::new(), String::new()),
                        };
                        // Retrieve the event name from the decoder's cache
                        // (available after decode, even on cache hit).
                        let tdh_name = decoder.borrow().event_name(record).unwrap_or("").to_owned();
                        let _ = name; // shadowed by tdh_name
                        (fields, tdh_name, ud)
                    } else {
                        (Vec::new(), String::new(), Vec::new())
                    };

                    // Extract Activity ID from the EVENT_RECORD header.
                    // The GUID is {data1: u32, data2: u16, data3: u16, data4: [u8;8]}
                    // which we flatten to 16 bytes in standard GUID byte order.
                    let activity_id = anc
                        .record()
                        .map(|r| {
                            let g = &r.EventHeader.ActivityId;
                            let mut bytes = [0u8; 16];
                            bytes[0..4].copy_from_slice(&g.data1.to_ne_bytes());
                            bytes[4..6].copy_from_slice(&g.data2.to_ne_bytes());
                            bytes[6..8].copy_from_slice(&g.data3.to_ne_bytes());
                            bytes[8..16].copy_from_slice(&g.data4);
                            bytes
                        })
                        .unwrap_or([0u8; 16]);

                    // Build EtwEventData with all available metadata.
                    // Convert QPC ticks to Unix epoch nanoseconds.
                    let qpc_ticks = anc.time();
                    let unix_ns = qpc_ref.qpc_to_unix_ns(qpc_ticks);
                    let data = EtwEventData {
                        provider_id: anc.provider().to_bytes(),
                        timestamp: unix_ns as u64,
                        process_id: anc.pid(),
                        thread_id: anc.tid(),
                        event_id,
                        opcode,
                        version,
                        level,
                        keywords,
                        event_name,
                        activity_id,
                        decoded_fields,
                        user_data,
                    };

                    // Drop the borrow before sending.
                    drop(anc);

                    let i = next.get();
                    next.set(i.wrapping_add(1));

                    // Best-effort send; if this core's channel is full the
                    // event is dropped from the pipeline entirely (each event
                    // is assigned to exactly one core via round-robin).
                    match txs[i % txs.len()].try_send(data) {
                        Ok(()) => {}
                        Err(mpsc::error::TrySendError::Full(_)) => {
                            let count = dropped.get() + 1;
                            dropped.set(count);
                            // TODO: Report dropped events as a metric counter
                            // instead of a log line.  MetricSet is not directly
                            // usable here because this callback runs on the
                            // blocking ProcessTrace OS thread (!Send context).
                            // Consider an AtomicU64 that the async receiver
                            // side periodically reads and reports via MetricSet.
                            //
                            // Rate-limited log: first drop, then every 10,000th.
                            if count == 1 || count.is_multiple_of(10_000) {
                                otel_warn!(
                                    "etw.event.dropped",
                                    total_dropped = count,
                                    core = i % txs.len(),
                                );
                            }
                        }
                        Err(mpsc::error::TrySendError::Closed(_)) => {
                            // The receiver for this core has been dropped
                            // (shutdown).  Log once per core so we can
                            // distinguish an early single-core failure from
                            // a normal full-shutdown sequence.
                            let core = i % txs.len();
                            if !closed_logged[core].get() {
                                closed_logged[core].set(true);
                                otel_info!("etw.event.channel_closed", core = core);
                            }
                        }
                    }

                    Ok(())
                });

                session.add_event(wide_event, None);
            }

            // `parse_until` blocks on `ProcessTrace`.  We never signal stop,
            // so the session runs until the process exits.
            // TODO: Surface startup failures via a oneshot readiness channel
            // once the one-collect API stabilizes (TDH decoding work).
            let result = session.parse_until(&session_name, || false);
            if let Err(ref e) = result {
                otel_error!(
                    "etw.parse_until.failed",
                    session_name = session_name.as_str(),
                    error = %e,
                );
            }

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

/// Acquire one consumer channel from the ETW session for the given
/// `session_name`.
///
/// On the **first** call for a given `session_name`, this function:
/// 1. Creates `num_cores` bounded MPSC channels.
/// 2. Spawns the ETW session thread with round-robin fan-out across all
///    senders.
/// 3. Stores the receivers in the session registry.
///
/// On each call (including the first) it pops one receiver from the pool and
/// returns it.  The engine calls the receiver factory once per allocated core,
/// so exactly `num_cores` calls are expected per `session_name`.
///
/// # Errors
///
/// Returns an error if:
/// - Provider GUID parsing fails (first call for this `session_name` only).
/// - The ETW session thread cannot be spawned (first call only).
/// - The `session_name` is already in use by another `receiver:etw` node
///   with a **different** provider configuration (config mismatch).
/// - The `session_name` pool is exhausted (all consumers for that session
///   have already been handed out).
/// - The session lock is poisoned (indicates a prior panic).
pub(super) fn subscribe(
    config: &Config,
    num_cores: usize,
) -> Result<mpsc::Receiver<EtwEventData>, Error> {
    let mut guard = SESSIONS.lock().map_err(|e| Error::InternalError {
        message: format!("ETW sessions lock poisoned: {e}"),
    })?;

    let sessions = guard.get_or_insert_with(HashMap::new);

    let entry = match sessions.entry(config.session_name.clone()) {
        Entry::Vacant(v) => {
            // First call for this session_name — initialize the session.
            let (txs, rxs): (Vec<_>, Vec<_>) = (0..num_cores)
                .map(|_| mpsc::channel(EVENT_CHANNEL_CAPACITY))
                .unzip();

            spawn_etw_session(config, txs)?;

            v.insert(SessionEntry {
                config: config.clone(),
                pool: rxs,
            })
        }
        Entry::Occupied(o) => {
            let existing = o.into_mut();
            // Guard against two different receiver:etw nodes accidentally
            // sharing the same session_name with different provider configs.
            // Without this check, node B would silently consume channels
            // from node A's session and receive the wrong events.
            if existing.config != *config {
                return Err(Error::ConfigError(Box::new(
                    otap_df_config::error::Error::InvalidUserConfig {
                        error: format!(
                            "ETW session_name '{}' is already in use with a different \
                             provider configuration; each receiver:etw node must use a \
                             distinct session_name or an identical config",
                            config.session_name,
                        ),
                    },
                )));
            }
            existing
        }
    };

    entry.pool.pop().ok_or_else(|| {
        Error::ConfigError(Box::new(otap_df_config::error::Error::InvalidUserConfig {
            error: format!(
                "ETW session_name '{}' is already in use; \
                     each receiver:etw node must specify a distinct session_name",
                config.session_name,
            ),
        }))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: insert a pre-built `SessionEntry` into the global registry
    /// for testing.  Returns a guard-like struct that removes the entry on
    /// drop to avoid leaking test state between tests.
    struct TestSession {
        name: String,
    }

    impl TestSession {
        fn insert(name: &str, pool: Vec<mpsc::Receiver<EtwEventData>>) -> Self {
            Self::insert_with_config(name, pool, test_config(name))
        }

        fn insert_with_config(
            name: &str,
            pool: Vec<mpsc::Receiver<EtwEventData>>,
            config: Config,
        ) -> Self {
            let mut guard = SESSIONS.lock().expect("lock not poisoned");
            let sessions = guard.get_or_insert_with(HashMap::new);
            let _ = sessions.insert(name.to_string(), SessionEntry { config, pool });
            Self {
                name: name.to_string(),
            }
        }
    }

    impl Drop for TestSession {
        fn drop(&mut self) {
            if let Ok(mut guard) = SESSIONS.lock() {
                if let Some(sessions) = guard.as_mut() {
                    let _ = sessions.remove(&self.name);
                }
            }
        }
    }

    fn test_config(session_name: &str) -> Config {
        Config {
            session_name: session_name.to_string(),
            providers: vec![ProviderConfig {
                name: None,
                guid: Some("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716".to_string()),
                level: TraceLevel::default(),
                keywords: None,
            }],
            batching: None,
        }
    }

    // ── Session registry ─────────────────────────────

    #[test]
    fn subscribe_rejects_exhausted_session_name() {
        // Pre-populate the registry with an empty pool to simulate
        // a session whose channels have all been handed out.
        let _guard = TestSession::insert("test-exhausted", vec![]);

        let config = test_config("test-exhausted");
        let err = subscribe(&config, 1).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("already in use"),
            "expected 'already in use' error, got: {msg}"
        );
        assert!(
            msg.contains("test-exhausted"),
            "error should mention the session name, got: {msg}"
        );
    }

    #[test]
    fn subscribe_pops_from_existing_pool() {
        // Pre-populate with a pool of 2 receivers.
        let (_tx1, rx1) = mpsc::channel::<EtwEventData>(1);
        let (_tx2, rx2) = mpsc::channel::<EtwEventData>(1);
        let _guard = TestSession::insert("test-pool-pop", vec![rx1, rx2]);

        let config = test_config("test-pool-pop");

        // First pop should succeed.
        let result1 = subscribe(&config, 2);
        assert!(result1.is_ok(), "first subscribe should succeed");

        // Second pop should succeed.
        let result2 = subscribe(&config, 2);
        assert!(result2.is_ok(), "second subscribe should succeed");

        // Third pop — pool exhausted — should return InvalidUserConfig.
        let err = subscribe(&config, 2).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("already in use"),
            "expected 'already in use' error on exhausted pool, got: {msg}"
        );
    }

    #[test]
    fn subscribe_different_session_names_are_independent() {
        // Pre-populate two different session names, each with their own pool.
        let (_tx_a, rx_a) = mpsc::channel::<EtwEventData>(1);
        let (_tx_b, rx_b) = mpsc::channel::<EtwEventData>(1);
        let _guard_a = TestSession::insert("test-session-a", vec![rx_a]);
        let _guard_b = TestSession::insert("test-session-b", vec![rx_b]);

        let config_a = test_config("test-session-a");
        let config_b = test_config("test-session-b");

        // Both should succeed independently.
        let result_a = subscribe(&config_a, 1);
        assert!(result_a.is_ok(), "session-a subscribe should succeed");

        let result_b = subscribe(&config_b, 1);
        assert!(result_b.is_ok(), "session-b subscribe should succeed");

        // Exhausting one doesn't affect the other (both are now empty,
        // so both should fail independently with their own session name).
        let err_a = subscribe(&config_a, 1).unwrap_err();
        assert!(
            err_a.to_string().contains("test-session-a"),
            "error should mention session-a"
        );

        let err_b = subscribe(&config_b, 1).unwrap_err();
        assert!(
            err_b.to_string().contains("test-session-b"),
            "error should mention session-b"
        );
    }

    #[test]
    fn subscribe_rejects_config_mismatch() {
        // Pre-populate with a session that has one provider config.
        let (_tx, rx) = mpsc::channel::<EtwEventData>(1);
        let original_config = test_config("test-mismatch");
        let _guard = TestSession::insert_with_config("test-mismatch", vec![rx], original_config);

        // Attempt to subscribe with a different provider config but the
        // same session_name — this should be rejected.
        let different_config = Config {
            session_name: "test-mismatch".to_string(),
            providers: vec![ProviderConfig {
                name: None,
                guid: Some("a0c1853b-5c40-4b15-8766-3cf1c58f985a".to_string()),
                level: TraceLevel::Verbose,
                keywords: None,
            }],
            batching: None,
        };

        let err = subscribe(&different_config, 1).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("different provider configuration"),
            "expected config mismatch error, got: {msg}"
        );
        assert!(
            msg.contains("test-mismatch"),
            "error should mention the session name, got: {msg}"
        );
    }

    // ── GUID parsing ─────────────────────────────────

    #[test]
    fn parse_guid_standard_format() {
        let guid = parse_guid("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716").expect("valid GUID");
        assert_eq!(guid.data1, 0x22fb2cd6);
        assert_eq!(guid.data2, 0x0e7b);
        assert_eq!(guid.data3, 0x422b);
        assert_eq!(guid.data4, [0xa0, 0xc7, 0x2f, 0xad, 0x1f, 0xd0, 0xe7, 0x16]);
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

    // ── Trace level mapping ──────────────────────────

    #[test]
    fn trace_level_mapping() {
        assert_eq!(
            trace_level_to_etw(&TraceLevel::Critical),
            etw::LEVEL_CRITICAL
        );
        assert_eq!(trace_level_to_etw(&TraceLevel::Error), etw::LEVEL_ERROR);
        assert_eq!(trace_level_to_etw(&TraceLevel::Warning), etw::LEVEL_WARNING);
        assert_eq!(
            trace_level_to_etw(&TraceLevel::Information),
            etw::LEVEL_INFORMATION
        );
        assert_eq!(trace_level_to_etw(&TraceLevel::Verbose), etw::LEVEL_VERBOSE);
    }
}
