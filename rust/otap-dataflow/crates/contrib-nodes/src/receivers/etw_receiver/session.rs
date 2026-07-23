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
//! Each field's bytes are interpreted into a typed [`EtwAttributeValue`]
//! (see [`interpret_field_value`]) and stored in a [`DecodedField`], which is
//! sent across the channel alongside the event header metadata.
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
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc as std_mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use one_collect::Guid;
use one_collect::etw::tdh::TdhDecoder;
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

/// Typed value of a single TDH-decoded ETW field.
///
/// The decoder interprets each field's raw bytes **once** (on the
/// `ProcessTrace` thread) into one of these variants, instead of deferring
/// interpretation to the encoder via a `(type_name, len)` string match.  This
/// gives compile-time exhaustiveness at every consumer match site (adding a
/// variant is a compile error rather than a silent fall-through) and avoids
/// the redundant `type_name: String` allocation plus consumer-side byte
/// re-parsing.  Modeled on the Linux `user_events_receiver`'s
/// `DecodedAttrValue`.
#[derive(Debug, Clone, PartialEq)]
pub enum EtwAttributeValue {
    /// UTF-8 / UTF-16-decoded string value.
    Str(String),
    /// Signed/unsigned integer widened to `i64`.
    Int(i64),
    /// Floating-point value widened to `f64`.
    Double(f64),
    /// Boolean value.  Note: the `one_collect` TDH decoder maps a Win32
    /// `BOOL` (`TDH_INTYPE_BOOLEAN`, TraceLogging `Bool32`) to a 4-byte
    /// `"u32"` and a 1-byte boolean (`TDH_INTYPE_UINT8` + `OutType::Boolean`)
    /// to `"u8"`.  Both currently surface as [`Int`](Self::Int); this variant
    /// is reserved for a future path that emits a distinct boolean type name.
    Bool(bool),
    /// Genuinely unsupported / opaque field bytes.  The encoder renders these
    /// as a hex string.  Empty for zero-length or undecodable fields.
    Bytes(Vec<u8>),
}

/// A single decoded field from a TDH-decoded TraceLogging event.
///
/// During the `ProcessTrace` callback the raw `EVENT_RECORD` is still valid,
/// so we interpret each field's bytes into an owned [`EtwAttributeValue`]
/// before sending the event across the channel.
#[derive(Debug, Clone, PartialEq)]
pub struct DecodedField {
    /// Field name (e.g. `"ProcessId"`, or `"Parent.ChildField"` for nested structs).
    pub name: String,
    /// Typed field value, interpreted from the raw payload bytes by the decoder.
    pub value: EtwAttributeValue,
}

/// A GUID in canonical (big-endian display) byte order.
///
/// The 16 bytes are stored in the exact order they appear in the standard
/// `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` string form: the first three groups
/// (`Data1`/`Data2`/`Data3`) are big-endian and the trailing eight bytes
/// (`Data4`) are kept as-is.
///
/// Windows stores `Data1`/`Data2`/`Data3` little-endian in memory, so the byte
/// swap into display order is applied **once**, here at the session boundary
/// (see [`CanonicalGuid::from_guid_parts`] and [`From<Guid>`]). Downstream
/// encoders therefore only perform hex/dash formatting and never need to know
/// the source byte order, so a value that is already canonical cannot be
/// silently byte-swapped a second time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CanonicalGuid(pub [u8; 16]);

impl CanonicalGuid {
    /// Assemble canonical bytes from the standard GUID struct fields
    /// (`Data1: u32`, `Data2: u16`, `Data3: u16`, `Data4: [u8; 8]`),
    /// byte-swapping the three numeric fields into big-endian display order.
    ///
    /// Both [`one_collect::Guid`] and the Windows `EVENT_RECORD` activity GUID
    /// share this field layout, so this is the single conversion point for the
    /// provider and activity IDs alike.
    fn from_guid_parts(data1: u32, data2: u16, data3: u16, data4: [u8; 8]) -> Self {
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&data1.to_be_bytes());
        bytes[4..6].copy_from_slice(&data2.to_be_bytes());
        bytes[6..8].copy_from_slice(&data3.to_be_bytes());
        bytes[8..16].copy_from_slice(&data4);
        Self(bytes)
    }

    /// Whether this is the all-zero GUID (e.g. no activity ID was set).
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 16]
    }
}

impl From<Guid> for CanonicalGuid {
    fn from(g: Guid) -> Self {
        Self::from_guid_parts(g.data1, g.data2, g.data3, g.data4)
    }
}

/// Lightweight snapshot of an ETW event captured in the `ProcessTrace` callback.
///
/// Because the `EVENT_RECORD` pointer is only valid for the duration of the
/// callback, we copy the fields we need into this owned struct before sending
/// it across the channel to the async world.
#[derive(Debug, Clone)]
pub struct EtwEventData {
    /// Provider GUID that produced the event, in canonical byte order.
    pub provider_id: CanonicalGuid,
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
    /// Activity ID from the event header for correlating related events, in
    /// canonical byte order.
    ///
    /// All zeros when the provider does not set an activity ID.
    pub activity_id: CanonicalGuid,
    /// TDH-decoded event payload fields.
    ///
    /// Populated for TraceLogging / TraceLoggingDynamic events whose schema
    /// can be discovered via TDH.  Empty for manifest-based events (which
    /// will be supported in a future extension) or when decoding fails.
    pub decoded_fields: Vec<DecodedField>,
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

/// Number of 100-nanosecond ticks between the Windows `FILETIME` epoch
/// (1601-01-01 UTC) and the Unix epoch (1970-01-01 UTC).
const FILETIME_TICKS_TO_UNIX_EPOCH: i64 = 116_444_736_000_000_000;

/// Interpret a TDH-decoded field's raw bytes as a typed [`EtwAttributeValue`].
///
/// The `type_name` strings come from `one_collect`'s TDH decoder
/// (`intype_to_field_info`) and follow the same naming conventions as the
/// user_events tracefs decoder.  Doing this interpretation here, next to the
/// decoder, keeps TDH type knowledge in one place and lets the encoder
/// collapse to an exhaustive match over [`EtwAttributeValue`] with no silent
/// `(type_name, len)` fall-throughs.
///
/// Type-name reference (from `one_collect::etw::tdh::intype_to_field_info`):
/// `s8`/`s16`/`s32`/`s64` (signed, with `HEXINT*` folded into `s32`/`s64`),
/// `u8`/`u16`/`u32`/`u64` (unsigned, with a Win32 `BOOL` / TraceLogging
/// `Bool32` (`TDH_INTYPE_BOOLEAN`) mapped to a 4-byte `u32`, and a 1-byte
/// boolean (`TDH_INTYPE_UINT8` + `OutType::Boolean`) mapped to `u8`),
/// `float`/`double`, `string`/`wstring`/`counted_string`/`counted_wstring`
/// (text), `pointer` (4 or 8 bytes), `filetime` (8 bytes), `guid` (16),
/// `systemtime` (16), `binary` (SID / opaque), and `unsupported`.
///
/// Genuinely opaque types (`guid`, `systemtime`, `binary`, `unsupported`) and
/// any length mismatch fall back to [`EtwAttributeValue::Bytes`], which the
/// encoder renders as a hex string.  Numeric conversions use the host byte
/// order, matching the live same-host capture model.
fn interpret_field_value(type_name: &str, data: &[u8]) -> EtwAttributeValue {
    match (type_name, data.len()) {
        // Signed integers (HEXINT32/64 are surfaced by one_collect as s32/s64).
        ("s8", 1) => EtwAttributeValue::Int(i64::from(data[0] as i8)),
        ("s16" | "short", 2) => EtwAttributeValue::Int(i64::from(i16::from_ne_bytes(
            data.try_into().expect("matched len==2"),
        ))),
        ("s32" | "int", 4) => EtwAttributeValue::Int(i64::from(i32::from_ne_bytes(
            data.try_into().expect("matched len==4"),
        ))),
        ("s64" | "long", 8) => {
            EtwAttributeValue::Int(i64::from_ne_bytes(data.try_into().expect("matched len==8")))
        }

        // Unsigned integers.  Note: one_collect maps a 1-byte boolean
        // (TDH_INTYPE_UINT8 + OutType::Boolean) to "u8", so 1-byte boolean
        // fields arrive here as a 0/1 integer.  A Win32 BOOL / TraceLogging
        // Bool32 (TDH_INTYPE_BOOLEAN) is a 4-byte value and arrives as "u32"
        // (see the "u32" arm below).
        ("u8", 1) => EtwAttributeValue::Int(i64::from(data[0])),
        ("u16" | "unsigned short", 2) => EtwAttributeValue::Int(i64::from(u16::from_ne_bytes(
            data.try_into().expect("matched len==2"),
        ))),
        ("u32" | "unsigned int", 4) => EtwAttributeValue::Int(i64::from(u32::from_ne_bytes(
            data.try_into().expect("matched len==4"),
        ))),
        ("u64" | "unsigned long", 8) => {
            // u64 may overflow i64; saturate to i64::MAX for observability.
            let v = u64::from_ne_bytes(data.try_into().expect("matched len==8"));
            EtwAttributeValue::Int(v.min(i64::MAX as u64) as i64)
        }

        // Explicit boolean spellings, kept for forward-compatibility in case a
        // future decoder emits a distinct "bool"/"boolean" type name.  With the
        // current one_collect decoder these are unreachable: a 1-byte boolean
        // surfaces as "u8" and a Win32 BOOL / TraceLogging Bool32 surfaces as
        // "u32".
        ("bool" | "boolean", 1) => EtwAttributeValue::Bool(data[0] != 0),
        ("bool" | "boolean", 4) => EtwAttributeValue::Bool(
            u32::from_ne_bytes(data.try_into().expect("matched len==4")) != 0,
        ),

        // Pointer (4 bytes on 32-bit payloads, 8 on 64-bit).  Surface as an
        // unsigned integer (saturating to i64::MAX) rather than opaque bytes.
        ("pointer", 4) => EtwAttributeValue::Int(i64::from(u32::from_ne_bytes(
            data.try_into().expect("matched len==4"),
        ))),
        ("pointer", 8) => {
            let v = u64::from_ne_bytes(data.try_into().expect("matched len==8"));
            EtwAttributeValue::Int(v.min(i64::MAX as u64) as i64)
        }

        // FILETIME: 8-byte count of 100-ns ticks since 1601-01-01 UTC.
        // Convert to Unix-epoch nanoseconds so it is a usable timestamp
        // instead of an opaque hex blob.
        ("filetime", 8) => {
            let ticks = i64::from_ne_bytes(data.try_into().expect("matched len==8"));
            let unix_ns = ticks
                .saturating_sub(FILETIME_TICKS_TO_UNIX_EPOCH)
                .saturating_mul(100);
            EtwAttributeValue::Int(unix_ns)
        }

        // Floating point
        ("float", 4) => EtwAttributeValue::Double(f64::from(f32::from_ne_bytes(
            data.try_into().expect("matched len==4"),
        ))),
        ("double", 8) => {
            EtwAttributeValue::Double(f64::from_ne_bytes(data.try_into().expect("matched len==8")))
        }

        // ANSI/UTF-8 strings (null-terminated or not) and counted ANSI/UTF-8
        // strings (TDH_INTYPE_COUNTEDANSISTRING, in_type 301).  For the
        // counted form the u16 byte-count prefix has already been consumed by
        // the framework's StaticLenPrefixArray, so `data` is just the content
        // bytes in both cases.
        ("string" | "counted_string", _) => EtwAttributeValue::Str(decode_ansi(data)),
        // Counted UTF-16 strings (TDH_INTYPE_COUNTEDSTRING, in_type 300).
        ("counted_wstring", _) if data.len() >= 2 => EtwAttributeValue::Str(decode_utf16le(data)),
        ("counted_wstring", _) => EtwAttributeValue::Str(String::new()),
        // UTF-16LE strings, trim null terminator.
        ("wstring", _) if data.len() >= 2 => EtwAttributeValue::Str(decode_utf16le(data)),

        // Opaque fixed/variable-length types that have no scalar
        // representation: GUID, SYSTEMTIME, SID/BINARY, and the decoder's
        // "unsupported" sentinel.  Preserved as raw bytes (hex downstream)
        // rather than dropped.  Listed explicitly so the intent is documented
        // and the catch-all below only ever sees truly unknown names.
        ("guid" | "systemtime" | "binary" | "unsupported", _) => {
            EtwAttributeValue::Bytes(data.to_vec())
        }

        // Empty payloads carry no value.
        _ if data.is_empty() => EtwAttributeValue::Str(String::new()),
        // Unknown type name or length mismatch: preserve the raw bytes so the
        // encoder can surface them (as a hex string) rather than dropping them.
        _ => EtwAttributeValue::Bytes(data.to_vec()),
    }
}

/// Decode an ANSI/UTF-8 byte slice into a `String`, stopping at the first NUL
/// byte and substituting U+FFFD for invalid UTF-8 sequences.
///
/// The NUL is trimmed from the byte slice *before* the lossy UTF-8 conversion
/// so the invalid-input path allocates only once (`into_owned`) instead of
/// twice (a `from_utf8_lossy` `String` followed by a `to_owned` of the trimmed
/// slice).  The valid-ASCII path is unchanged at a single allocation.
fn decode_ansi(data: &[u8]) -> String {
    let trimmed = data.split(|&b| b == 0).next().unwrap_or(data);
    String::from_utf8_lossy(trimmed).into_owned()
}

/// Decode a UTF-16LE byte slice into a `String`, stopping at the first NUL
/// code unit and substituting U+FFFD for invalid surrogate pairs.
///
/// This runs on the `ProcessTrace` hot path, so it is tuned for the common
/// case: most ETW string fields (paths, identifiers, English log lines) are
/// pure ASCII, where every high byte is zero.  An initial scan detects that
/// case and copies the low bytes directly, skipping the surrogate-decode
/// state machine and pre-sizing the output to avoid reallocation.
fn decode_utf16le(data: &[u8]) -> String {
    // Round down to whole 16-bit code units; ignore a trailing odd byte.
    let len = data.len() & !1;
    let bytes = &data[..len];

    // ASCII fast path: find the first NUL or first non-ASCII code unit.
    let ascii_end = bytes
        .chunks_exact(2)
        .position(|c| c[0] == 0 || c[1] != 0)
        .map(|i| i * 2)
        .unwrap_or(len);

    if ascii_end == len {
        // Entirely ASCII up to the end (or a terminating NUL): copy the low
        // bytes directly, no surrogate logic needed.
        let mut out = String::with_capacity(ascii_end / 2);
        for chunk in bytes[..ascii_end].chunks_exact(2) {
            out.push(chunk[0] as char);
        }
        return out;
    }

    // Mixed / non-ASCII: full UTF-16 decode, stopping at the first NUL and
    // substituting U+FFFD for invalid surrogate pairs.
    let mut out = String::with_capacity(len / 2);
    let u16_iter = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|&c| c != 0);
    out.extend(char::decode_utf16(u16_iter).map(|r| r.unwrap_or('\u{FFFD}')));
    out
}

/// Decode an event's fields into owned [`DecodedField`]s.
///
/// Uses [`one_collect::event::EventFormat::fields_with_data`], which walks the
/// payload exactly once (carrying a running offset) and yields each field
/// paired with its bytes. Variable-length fields (strings / counted arrays)
/// are therefore scanned a single time, making extraction an O(n) pass rather
/// than the O(n^2) cost of resolving each field independently. The single pass
/// also needs no per-schema reader cache: there are no boxed closures to build
/// or reuse.
///
/// Each field's bytes are interpreted into a typed [`EtwAttributeValue`]
/// straight from the borrowed slice, with no intermediate copy. Numeric fields
/// allocate nothing; string and bytes fields allocate only their owned value.
///
/// # Safety
///
/// Called during the `ProcessTrace` callback while the `EVENT_RECORD` (and its
/// `UserData`) is still valid. `fields_with_data` reads only within the payload
/// slice and yields an empty slice for any field (and all following) whose
/// length can't be resolved. It cannot panic here because TDH emits only
/// fixed / string / counted-array layouts, never the `__rel_loc`/`__data_loc`
/// types that hit `todo!()` in `get_data_with_offset_direct`. So no
/// `catch_unwind` is needed in this `extern "system"` callback.
fn extract_decoded_fields(
    format: &one_collect::event::EventFormat,
    event_data: &[u8],
) -> Vec<DecodedField> {
    format
        .fields_with_data(event_data)
        .map(|(field, bytes)| DecodedField {
            name: field.name.clone(),
            value: interpret_field_value(&field.type_name, bytes),
        })
        .collect()
}

// ── Per-session telemetry bridge ─────────────────────────────────────────────

/// Counters written by the `!Send` `ProcessTrace` callback and read by the
/// async per-core receivers.
///
/// One instance exists per `session_name`, shared by `Arc`. The producer (the
/// blocking `ProcessTrace` OS thread) cannot touch the async `MetricSet`, so it
/// only ever `fetch_add`s into these atomics. The async receiver side
/// `swap(0)`s the running totals on each `CollectTelemetry` tick (and before
/// any terminal snapshot) and folds the delta into the `MetricSet`.
///
/// `Relaxed` ordering is sufficient: each field is an independent running
/// total with no happens-before relationship to other state.
#[derive(Debug, Default)]
pub(super) struct SessionWideMetrics {
    /// Every event the `ProcessTrace` callback observed from the trace
    /// session, counted *before* any per-core channel send is attempted.
    /// Published as `received_events_total` in the metric set.
    ///
    /// This is the producer-side ingress denominator. The slow-worker drop
    /// rate is computable as `dropped_slow_worker / total`. See the
    /// counter-algebra note on `EtwReceiverMetrics` for the exact relationships.
    pub total: AtomicU64,
    /// Events dropped because a per-core channel was full (internal backpressure).
    /// Published as `received_events_dropped_slow_worker` in the metric set.
    pub dropped_slow_worker: AtomicU64,
    /// Events whose TDH decode failed (`received_events_invalid`).
    pub decode_failed: AtomicU64,
    /// Kernel-side ETW events lost (buffer overrun) before `one_collect` ever
    /// saw them, from `TraceStats::events_lost`. A background poller converts
    /// the cumulative session counter into per-interval deltas and
    /// `fetch_add`s them here. Published as `received_events_lost_kernel`.
    /// Distinct from `dropped_slow_worker`, which is our own downstream loss.
    pub kernel_events_lost: AtomicU64,
    /// Real-time delivery buffers lost (consumer too slow to drain the ETW
    /// real-time buffers), from `TraceStats::real_time_buffers_lost`.
    /// Published as `kernel_real_time_buffers_lost`.
    pub kernel_real_time_buffers_lost: AtomicU64,
    /// Log buffers that could not be flushed, from
    /// `TraceStats::log_buffers_lost`. Published as `kernel_log_buffers_lost`.
    pub kernel_log_buffers_lost: AtomicU64,
    /// Total ETW buffers written by the session, from
    /// `TraceStats::buffers_written`. A throughput/health denominator rather
    /// than a loss signal. Published as `kernel_buffers_written`.
    pub kernel_buffers_written: AtomicU64,
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
    /// Shared atomic counters bridging the `!Send` `ProcessTrace` callback to
    /// the async receivers. Cloned to every per-core subscriber.
    telemetry: Arc<SessionWideMetrics>,
}

/// Snapshot of ETW cumulative trace counters returned by `query_stats`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct TraceStatsSnapshot {
    events_lost: u64,
    real_time_buffers_lost: u64,
    log_buffers_lost: u64,
    buffers_written: u64,
}

/// Per-poller baseline state used to compute monotonic deltas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct PollerBaselines {
    last: TraceStatsSnapshot,
    query_failed_logged: bool,
}

impl PollerBaselines {
    /// Mark a query failure and report whether the caller should emit a warn.
    fn on_query_failed(&mut self) -> bool {
        if self.query_failed_logged {
            return false;
        }
        self.query_failed_logged = true;
        true
    }

    /// Mark a successful query and report whether the caller should emit
    /// a single recovery info log.
    fn on_query_recovered(&mut self) -> bool {
        if !self.query_failed_logged {
            return false;
        }
        self.query_failed_logged = false;
        true
    }
}

/// Convert cumulative ETW stats into per-interval deltas and publish to the
/// shared session atomics.
fn publish_trace_stats_delta(
    telemetry: &SessionWideMetrics,
    baselines: &mut PollerBaselines,
    stats: TraceStatsSnapshot,
) {
    /// Compute the delta against the stored baseline, advance the baseline to
    /// the new cumulative value, and add the delta to the shared atomic.
    fn accumulate(atomic: &AtomicU64, baseline: &mut u64, current: u64) {
        let delta = current.saturating_sub(*baseline);
        *baseline = current;
        if delta > 0 {
            let _ = atomic.fetch_add(delta, Ordering::Relaxed);
        }
    }

    accumulate(
        &telemetry.kernel_events_lost,
        &mut baselines.last.events_lost,
        stats.events_lost,
    );
    accumulate(
        &telemetry.kernel_real_time_buffers_lost,
        &mut baselines.last.real_time_buffers_lost,
        stats.real_time_buffers_lost,
    );
    accumulate(
        &telemetry.kernel_log_buffers_lost,
        &mut baselines.last.log_buffers_lost,
        stats.log_buffers_lost,
    );
    accumulate(
        &telemetry.kernel_buffers_written,
        &mut baselines.last.buffers_written,
        stats.buffers_written,
    );
}

fn run_trace_stats_poller_loop<F>(
    handle_slot: Arc<AtomicU64>,
    telemetry: Arc<SessionWideMetrics>,
    poll_stop: Arc<AtomicBool>,
    poll_interval: Duration,
    poll_session_name: &str,
    mut query_stats: F,
) where
    F: FnMut(u64) -> Result<TraceStatsSnapshot, String>,
{
    let mut baselines = PollerBaselines::default();

    while !poll_stop.load(Ordering::Relaxed) {
        std::thread::sleep(poll_interval);
        if poll_stop.load(Ordering::Relaxed) {
            break;
        }

        let handle = handle_slot.load(Ordering::SeqCst);
        if handle == 0 {
            continue; // session not started yet
        }

        match query_stats(handle) {
            Ok(stats) => {
                if baselines.on_query_recovered() {
                    otel_info!(
                        "etw.query_stats.recovered",
                        session_name = poll_session_name,
                        handle = handle,
                        message = "ETW trace-stats polling recovered",
                    );
                }

                publish_trace_stats_delta(&telemetry, &mut baselines, stats);
            }
            Err(e) => {
                if baselines.on_query_failed() {
                    otel_warn!(
                        "etw.query_stats.failed",
                        session_name = poll_session_name,
                        handle = handle,
                        error = %e,
                        message = "Failed to query ETW trace stats; kernel loss metrics will stall until polling recovers",
                    );
                }
            }
        }
    }
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
fn spawn_etw_session(
    config: &Config,
    txs: Vec<mpsc::Sender<EtwEventData>>,
    telemetry: Arc<SessionWideMetrics>,
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
    let (ready_tx, ready_rx) = std_mpsc::sync_channel(1);

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
            // callbacks run on the single `ProcessTrace` thread, so `Cell` is
            // safe - no atomics or locking needed.  Sharing the counter ensures
            // uniform core distribution even at startup when multiple providers
            // would otherwise all start at index 0.  Drop accounting lives in
            // the shared `SessionWideMetrics` atomics so the async side can
            // surface it as a metric.
            let next: Rc<Cell<usize>> = Rc::new(Cell::new(0));
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
                let closed_logged = Rc::clone(&closed_logged);
                let txs = Rc::clone(&txs);
                let decoder = Rc::clone(&decoder);
                let telemetry = Arc::clone(&telemetry);

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

                    // All EVENT_RECORD-derived fields are read in a single
                    // `if let Some(record)` below; `anc.record()` returns the
                    // same `Option<&EVENT_RECORD>` each call, so folding the
                    // reads together avoids redundant lookups and the tuple
                    // shuffle.  When the record is absent every field keeps its
                    // default value declared here.
                    let mut level = 0u8;
                    let mut keywords = 0u64;
                    let mut activity_id = CanonicalGuid::default();
                    let mut event_name = String::new();
                    let mut decoded_fields = Vec::new();

                    if let Some(record) = anc.record() {
                        level = record.EventHeader.EventDescriptor.Level;
                        keywords = record.EventHeader.EventDescriptor.Keyword;

                        // Extract the Activity ID from the EVENT_RECORD header
                        // and convert it into canonical (big-endian display)
                        // byte order once, here at the session boundary. The
                        // GUID fields {data1: u32, data2: u16, data3: u16,
                        // data4: [u8;8]} are stored little-endian in memory;
                        // `from_guid_parts` byte-swaps the first three so the
                        // encoder only needs to render hex/dashes.
                        let g = &record.EventHeader.ActivityId;
                        activity_id =
                            CanonicalGuid::from_guid_parts(g.data1, g.data2, g.data3, g.data4);

                        // TDH decode: attempt to decode TraceLogging event
                        // schema.  A failure (NotFound for manifest-based
                        // events, or other decode errors) leaves the empty
                        // defaults in place and is counted via
                        // `received_events_invalid` - the event is still
                        // forwarded with empty `decoded_fields`.  Future work
                        // will add manifest decoding with a
                        // (Provider, Id, Version) cache key.
                        match decoder.borrow_mut().decode(record) {
                            Ok(result) => {
                                event_name = result.event_name.unwrap_or("").to_owned();
                                decoded_fields = extract_decoded_fields(
                                    result.event_data.format(),
                                    result.event_data.event_data(),
                                );
                            }
                            Err(_) => {
                                let _ = telemetry.decode_failed.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }

                    // Build EtwEventData with all available metadata.
                    // Convert QPC ticks to Unix epoch nanoseconds.
                    let qpc_ticks = anc.time();
                    let unix_ns = qpc_ref.qpc_to_unix_ns(qpc_ticks);
                    let data = EtwEventData {
                        provider_id: CanonicalGuid::from(anc.provider()),
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
                    };

                    // Drop the borrow before sending.
                    drop(anc);

                    let i = next.get();
                    next.set(i.wrapping_add(1));

                    // Count every event the session produced, *before* the
                    // send is attempted. This is the producer-side ingress
                    // denominator (`received_events_total` in the metric set).
                    // The slow-worker drop rate is therefore
                    // `dropped_slow_worker / total`.
                    let _ = telemetry.total.fetch_add(1, Ordering::Relaxed);

                    // Best-effort send; if this core's channel is full the
                    // event is dropped from the pipeline entirely (each event
                    // is assigned to exactly one core via round-robin).
                    match txs[i % txs.len()].try_send(data) {
                        Ok(()) => {}
                        Err(mpsc::error::TrySendError::Full(_)) => {
                            // Bump the shared atomic so the async receiver can
                            // surface it as `received_events_dropped_slow_worker`
                            // on the next `CollectTelemetry`.
                            let _ = telemetry
                                .dropped_slow_worker
                                .fetch_add(1, Ordering::Relaxed);
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

            // Capture the live ETW session handle so a background poller can
            // query kernel-side loss counters while `parse_until` blocks. The
            // handle is only valid for the running session; 0 means the
            // session has not started yet.
            //
            // The started_callback fires when the ETW session is ACTUALLY
            // started (inside parse_until), not when the thread is initialized.
            // This is where we signal readiness to the caller: the session is
            // now live, the handle is valid, and event delivery is beginning.
            let handle_slot = Arc::new(AtomicU64::new(0));
            {
                let handle_slot = handle_slot.clone();
                let ready_tx_for_started = ready_tx.clone();
                session.add_started_callback(move |ctx| {
                    handle_slot.store(ctx.handle(), Ordering::SeqCst);

                    // Signal to the caller that the ETW session is actually
                    // ready: the handle is now populated, the poller can
                    // query stats, and event callbacks are active.
                    let _ = ready_tx_for_started.send(Ok(()));
                });
            }

            // Background poller: `query_stats` returns cumulative, monotonic
            // counters for the running session. We convert each into a
            // per-interval delta and `fetch_add` into the shared atomics, which
            // the async side then claims via the existing `swap(0)` model. The
            // poller lives for the duration of `parse_until` and is stopped and
            // joined immediately after it returns.
            let poll_stop = Arc::new(AtomicBool::new(false));
            let poller = {
                let handle_slot = handle_slot.clone();
                let telemetry = Arc::clone(&telemetry);
                let poll_stop = poll_stop.clone();
                let poll_session_name = session_name.clone();
                match std::thread::Builder::new()
                    .name("etw-drop-poller".into())
                    .spawn(move || {
                        run_trace_stats_poller_loop(
                            handle_slot,
                            telemetry,
                            poll_stop,
                            Duration::from_secs(1),
                            poll_session_name.as_str(),
                            |handle| {
                                // `one_collect` exposes the native ETW counters
                                // as `u32`; widen to `u64` here at the consumer
                                // boundary so the shared atomics can accumulate
                                // deltas without overflow.
                                etw::query_stats(handle)
                                    .map(|stats| TraceStatsSnapshot {
                                        events_lost: u64::from(stats.events_lost),
                                        real_time_buffers_lost: u64::from(
                                            stats.real_time_buffers_lost,
                                        ),
                                        log_buffers_lost: u64::from(stats.log_buffers_lost),
                                        buffers_written: u64::from(stats.buffers_written),
                                    })
                                    .map_err(|e| e.to_string())
                            },
                        );
                    }) {
                    Ok(handle) => Some(handle),
                    Err(e) => {
                        let _ = ready_tx
                            .send(Err(format!("failed to spawn ETW trace-stats poller: {e}")));
                        return;
                    }
                }
            };

            // `parse_until` blocks on `ProcessTrace`, which also triggers the
            // started_callback where we signal readiness. We never signal stop,
            // so the session runs until the process exits.
            // TODO: Surface startup failures via a oneshot readiness channel
            // once the one-collect API stabilizes (TDH decoding work).
            let result = session.parse_until(&session_name, || false);

            // Session ended: stop the poller and join it so the thread is torn
            // down within one poll interval.
            poll_stop.store(true, Ordering::Relaxed);
            if let Some(poller) = poller {
                let _ = poller.join();
            }

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

    ready_rx
        .recv()
        .map_err(|e| Error::InternalError {
            message: format!("ETW session startup signal failed: {e}"),
        })?
        .map_err(|message| Error::InternalError { message })?;

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
) -> Result<(mpsc::Receiver<EtwEventData>, Arc<SessionWideMetrics>), Error> {
    let mut guard = SESSIONS.lock().map_err(|e| Error::InternalError {
        message: format!("ETW sessions lock poisoned: {e}"),
    })?;

    let sessions = guard.get_or_insert_with(HashMap::new);

    let entry = match sessions.entry(config.session_name.clone()) {
        Entry::Vacant(v) => {
            // First call for this session_name: initialize the session.
            let (txs, rxs): (Vec<_>, Vec<_>) = (0..num_cores)
                .map(|_| mpsc::channel(EVENT_CHANNEL_CAPACITY))
                .unzip();

            // Shared telemetry bridge for this session_name, cloned into the
            // ProcessTrace callback and into every per-core subscriber.
            let telemetry = Arc::new(SessionWideMetrics::default());

            spawn_etw_session(config, txs, Arc::clone(&telemetry))?;

            v.insert(SessionEntry {
                config: config.clone(),
                pool: rxs,
                telemetry,
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

    let telemetry = Arc::clone(&entry.telemetry);
    let rx = entry.pool.pop().ok_or_else(|| {
        Error::ConfigError(Box::new(otap_df_config::error::Error::InvalidUserConfig {
            error: format!(
                "ETW session_name '{}' is already in use; \
                     each receiver:etw node must specify a distinct session_name",
                config.session_name,
            ),
        }))
    })?;

    Ok((rx, telemetry))
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
            let _ = sessions.insert(
                name.to_string(),
                SessionEntry {
                    config,
                    pool,
                    telemetry: Arc::new(SessionWideMetrics::default()),
                },
            );
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

        // Third pop, pool exhausted, should return InvalidUserConfig.
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
        // same session_name; this should be rejected.
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

    // ── Field value interpretation ───────────────────

    #[test]
    fn interpret_signed_integers() {
        assert_eq!(
            interpret_field_value("s8", &[0xFF]),
            EtwAttributeValue::Int(-1)
        );
        assert_eq!(
            interpret_field_value("s16", &(-2i16).to_ne_bytes()),
            EtwAttributeValue::Int(-2)
        );
        assert_eq!(
            interpret_field_value("int", &(-3i32).to_ne_bytes()),
            EtwAttributeValue::Int(-3)
        );
        assert_eq!(
            interpret_field_value("long", &(-4i64).to_ne_bytes()),
            EtwAttributeValue::Int(-4)
        );
    }

    #[test]
    fn interpret_unsigned_integers() {
        assert_eq!(
            interpret_field_value("u8", &[200]),
            EtwAttributeValue::Int(200)
        );
        assert_eq!(
            interpret_field_value("unsigned short", &40000u16.to_ne_bytes()),
            EtwAttributeValue::Int(40000)
        );
        assert_eq!(
            interpret_field_value("u32", &1234u32.to_ne_bytes()),
            EtwAttributeValue::Int(1234)
        );
        // u64 above i64::MAX saturates to i64::MAX.
        assert_eq!(
            interpret_field_value("unsigned long", &u64::MAX.to_ne_bytes()),
            EtwAttributeValue::Int(i64::MAX)
        );
    }

    #[test]
    fn interpret_boolean() {
        assert_eq!(
            interpret_field_value("boolean", &[0]),
            EtwAttributeValue::Bool(false)
        );
        assert_eq!(
            interpret_field_value("boolean", &[1]),
            EtwAttributeValue::Bool(true)
        );
        // 4-byte Win32 BOOL form.
        assert_eq!(
            interpret_field_value("bool", &0u32.to_ne_bytes()),
            EtwAttributeValue::Bool(false)
        );
        assert_eq!(
            interpret_field_value("bool", &1u32.to_ne_bytes()),
            EtwAttributeValue::Bool(true)
        );
    }

    #[test]
    fn interpret_floating_point() {
        assert_eq!(
            interpret_field_value("float", &1.5f32.to_ne_bytes()),
            EtwAttributeValue::Double(1.5)
        );
        assert_eq!(
            interpret_field_value("double", &2.5f64.to_ne_bytes()),
            EtwAttributeValue::Double(2.5)
        );
    }

    #[test]
    fn interpret_strings() {
        assert_eq!(
            interpret_field_value("string", b"hello\0"),
            EtwAttributeValue::Str("hello".to_string())
        );
        assert_eq!(
            interpret_field_value("counted_string", b"world"),
            EtwAttributeValue::Str("world".to_string())
        );
        // UTF-16LE "Hi" with NUL terminator.
        let wide: Vec<u8> = "Hi\0".encode_utf16().flat_map(u16::to_ne_bytes).collect();
        assert_eq!(
            interpret_field_value("wstring", &wide),
            EtwAttributeValue::Str("Hi".to_string())
        );
    }

    #[test]
    fn interpret_pointer() {
        // 32-bit pointer.
        assert_eq!(
            interpret_field_value("pointer", &0x1234_5678u32.to_ne_bytes()),
            EtwAttributeValue::Int(0x1234_5678)
        );
        // 64-bit pointer above i64::MAX saturates to i64::MAX.
        assert_eq!(
            interpret_field_value("pointer", &u64::MAX.to_ne_bytes()),
            EtwAttributeValue::Int(i64::MAX)
        );
    }

    #[test]
    fn interpret_filetime_converts_to_unix_nanos() {
        // The FILETIME epoch tick count itself maps to Unix epoch (0 ns).
        assert_eq!(
            interpret_field_value("filetime", &FILETIME_TICKS_TO_UNIX_EPOCH.to_ne_bytes()),
            EtwAttributeValue::Int(0)
        );
        // One second (10,000,000 ticks) past the Unix epoch → 1e9 ns.
        let one_sec_after = FILETIME_TICKS_TO_UNIX_EPOCH + 10_000_000;
        assert_eq!(
            interpret_field_value("filetime", &one_sec_after.to_ne_bytes()),
            EtwAttributeValue::Int(1_000_000_000)
        );
    }

    #[test]
    fn interpret_opaque_types_fall_back_to_bytes() {
        // GUID, systemtime, binary, and the decoder's "unsupported" sentinel
        // are preserved as raw bytes (rendered as hex downstream) rather than
        // silently dropped.
        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        for type_name in ["guid", "systemtime", "binary", "unsupported"] {
            assert_eq!(
                interpret_field_value(type_name, &data),
                EtwAttributeValue::Bytes(data.clone()),
                "type_name={type_name} should fall back to Bytes"
            );
        }
    }

    #[test]
    fn interpret_empty_payload_is_empty_string() {
        assert_eq!(
            interpret_field_value("u32", &[]),
            EtwAttributeValue::Str(String::new())
        );
    }

    #[test]
    fn interpret_length_mismatch_falls_back_to_bytes() {
        // A "u32" with the wrong length must not panic; it falls through to
        // the opaque Bytes path instead of the fixed-width integer arm.
        let data = vec![1, 2, 3];
        assert_eq!(
            interpret_field_value("u32", &data),
            EtwAttributeValue::Bytes(data)
        );
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

    // ── Single-pass field extraction ─────────────────

    /// Builds an all-fixed-size `u32` schema with the given field names.
    fn u32_schema(names: &[&str]) -> one_collect::event::EventFormat {
        use one_collect::event::{EventField, EventFormat, LocationType};
        let mut format = EventFormat::new();
        for (i, name) in names.iter().enumerate() {
            format.add_field(EventField::new(
                (*name).to_string(),
                "u32".to_string(),
                LocationType::Static,
                i * 4,
                4,
            ));
        }
        format
    }

    #[test]
    fn extract_decodes_all_fixed_fields_in_order() {
        // A single forward pass yields every field, in schema order, with its
        // interpreted value.
        let format = u32_schema(&["a", "b"]);
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_ne_bytes());
        data.extend_from_slice(&2u32.to_ne_bytes());

        let fields = extract_decoded_fields(&format, &data);

        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name, "a");
        assert_eq!(fields[0].value, EtwAttributeValue::Int(1));
        assert_eq!(fields[1].name, "b");
        assert_eq!(fields[1].value, EtwAttributeValue::Int(2));
    }

    #[test]
    fn extract_is_deterministic_across_calls() {
        // The extraction owns no per-schema state, so repeated calls on the
        // same schema and payload produce identical output.
        let format = u32_schema(&["x"]);
        let data = 9u32.to_ne_bytes();

        let first = extract_decoded_fields(&format, &data);
        let second = extract_decoded_fields(&format, &data);

        assert_eq!(first, second);
        assert_eq!(first[0].value, EtwAttributeValue::Int(9));
    }

    #[test]
    fn extract_handles_variable_length_prefix() {
        // A leading NUL-terminated string shifts the fixed field that follows;
        // the single pass carries the running offset so the trailing u64 is
        // read from the correct position.
        use one_collect::event::{EventField, EventFormat, LocationType};

        let mut format = EventFormat::new();
        format.add_field(EventField::new(
            "s".to_string(),
            "string".to_string(),
            LocationType::StaticString,
            0,
            0,
        ));
        format.add_field(EventField::new(
            "n".to_string(),
            "u64".to_string(),
            LocationType::Static,
            0,
            8,
        ));

        let mut data = Vec::new();
        data.extend_from_slice(b"hello\0");
        data.extend_from_slice(&123u64.to_ne_bytes());

        let fields = extract_decoded_fields(&format, &data);

        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name, "s");
        assert_eq!(fields[0].value, EtwAttributeValue::Str("hello".to_string()));
        assert_eq!(fields[1].name, "n");
        assert_eq!(fields[1].value, EtwAttributeValue::Int(123));
    }

    // ── Trace-stats poller edge cases ──────────────

    #[test]
    fn trace_stats_first_poll_after_start_publishes_full_sample() {
        let telemetry = SessionWideMetrics::default();
        let mut baselines = PollerBaselines::default();

        publish_trace_stats_delta(
            &telemetry,
            &mut baselines,
            TraceStatsSnapshot {
                events_lost: 7,
                real_time_buffers_lost: 3,
                log_buffers_lost: 2,
                buffers_written: 11,
            },
        );

        assert_eq!(telemetry.kernel_events_lost.load(Ordering::Relaxed), 7);
        assert_eq!(
            telemetry
                .kernel_real_time_buffers_lost
                .load(Ordering::Relaxed),
            3
        );
        assert_eq!(telemetry.kernel_log_buffers_lost.load(Ordering::Relaxed), 2);
        assert_eq!(telemetry.kernel_buffers_written.load(Ordering::Relaxed), 11);
    }

    #[test]
    fn trace_stats_query_failure_then_recovery_is_one_shot() {
        let mut baselines = PollerBaselines::default();

        // First failure should log.
        assert!(baselines.on_query_failed());
        // Repeated failures should be suppressed.
        assert!(!baselines.on_query_failed());
        // First success after failure should log recovery.
        assert!(baselines.on_query_recovered());
        // Repeated successes should be suppressed.
        assert!(!baselines.on_query_recovered());
    }

    #[test]
    fn trace_stats_poller_loop_stops_and_joins() {
        let handle_slot = Arc::new(AtomicU64::new(77));
        let telemetry = Arc::new(SessionWideMetrics::default());
        let poll_stop = Arc::new(AtomicBool::new(false));
        let query_calls = Arc::new(AtomicU64::new(0));

        let query_calls_clone = Arc::clone(&query_calls);
        let poller = std::thread::spawn({
            let handle_slot = Arc::clone(&handle_slot);
            let telemetry = Arc::clone(&telemetry);
            let poll_stop = Arc::clone(&poll_stop);
            move || {
                run_trace_stats_poller_loop(
                    handle_slot,
                    telemetry,
                    poll_stop,
                    Duration::from_millis(5),
                    "test-session",
                    |handle| {
                        let _ = query_calls_clone.fetch_add(1, Ordering::Relaxed);
                        Ok(TraceStatsSnapshot {
                            events_lost: handle,
                            real_time_buffers_lost: 0,
                            log_buffers_lost: 0,
                            buffers_written: 0,
                        })
                    },
                )
            }
        });

        std::thread::sleep(Duration::from_millis(25));
        poll_stop.store(true, Ordering::Relaxed);

        poller.join().expect("poller thread should join cleanly");
        assert!(
            query_calls.load(Ordering::Relaxed) > 0,
            "poller should have executed at least one query before stop",
        );
    }
}
