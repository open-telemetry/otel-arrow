// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! ETW session management using the `one_collect` library.
//!
//! This module bridges the blocking `one_collect::etw::EtwSession` API with the
//! async OTAP dataflow receiver by:
//!
//! 1. Parsing provider GUIDs from the receiver configuration.
//! 2. Creating an `EtwSession` with the configured providers, levels, and keywords.
//! 3. Running `parse_until` on a dedicated blocking thread.
//! 4. Sending event metadata through a `tokio::sync::mpsc` channel back to the
//!    async receiver loop.
//! 5. Providing a [`SessionHandle`] that the receiver can use to signal shutdown.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use one_collect::etw::{self, EtwSession};
use one_collect::Guid;
use otap_df_engine::error::Error;
use otap_df_telemetry::metrics::MetricSet;

use super::{Config, EtwReceiverMetrics, ProviderConfig, TraceLevel};

/// Channel capacity for ETW events sent from the blocking session thread to the
/// async receiver loop. A bounded channel provides implicit backpressure: if the
/// receiver loop cannot keep up, the channel will fill and the ETW callback will
/// block, which in turn may cause the kernel to drop events (reported via
/// `EventsLost` in trace session statistics).
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

// ── Session handle for shutdown ──────────────────────────────────────────────

/// Handle returned to the receiver for controlling the ETW session lifetime.
///
/// Calling [`stop`](SessionHandle::stop) sets an atomic flag that causes
/// `EtwSession::parse_until` to return, which in turn tears down the trace
/// session and joins the `ProcessTrace` thread.
pub struct SessionHandle {
    stop_flag: Arc<AtomicBool>,
}

impl SessionHandle {
    /// Signal the ETW session to stop consuming events.
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Release);
    }
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

    let val = u128::from_str_radix(&hex, 16).map_err(|e| {
        Error::InternalError {
            message: format!("invalid GUID string '{s}': {e}"),
        }
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

// ── Session startup ──────────────────────────────────────────────────────────

/// Start an ETW trace session on a dedicated thread and return a
/// [`SessionHandle`] for shutdown control plus a channel receiver for incoming
/// event data.
///
/// # Errors
///
/// Returns an error if any configured provider GUID cannot be parsed.
/// Errors from `EtwSession::parse_until` (e.g. insufficient privileges) are
/// logged but do not propagate — the session thread simply exits and the
/// channel closes, which the receiver detects.
pub fn start_etw_session(
    config: &Config,
    _metrics: Rc<RefCell<MetricSet<EtwReceiverMetrics>>>,
) -> Result<(SessionHandle, tokio::sync::mpsc::Receiver<EtwEventData>), Error> {
    let (event_tx, event_rx) = tokio::sync::mpsc::channel(EVENT_CHANNEL_CAPACITY);

    // Resolve all provider GUIDs up-front (before spawning the thread) so
    // that configuration errors are reported synchronously.
    let resolved_providers: Vec<(Guid, u8, Option<u64>)> = config
        .providers
        .iter()
        .map(|p| {
            let guid = resolve_provider_guid(p)?;
            let level = trace_level_to_etw(&p.level);
            Ok((guid, level, p.keywords))
        })
        .collect::<Result<Vec<_>, Error>>()?;

    // Shared stop flag between the receiver and the session thread.
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();

    let session_name = config.session_name.clone();

    // `EtwSession` is !Send (it contains Rc, non-Send closures, etc.), so it
    // must be created and consumed on the **same** thread. We spawn a
    // dedicated OS thread that builds the session, enables providers, and
    // calls `parse_until` (which internally runs `ProcessTrace` — blocking).
    let _join_handle = std::thread::Builder::new()
        .name(format!("etw-session-{session_name}"))
        .spawn(move || {
            // Build the session on this thread.
            let mut session = EtwSession::new();

            for (guid, level, keywords) in &resolved_providers {
                let enabler = session.enable_provider(*guid);
                enabler.ensure_level(*level);
                if let Some(kw) = keywords {
                    enabler.ensure_keyword(*kw);
                }
            }

            // Install a generic "started" callback for observability.
            // Note: otel_info! / tracing macros are not available on the
            // dedicated OS thread. Logging from the ETW session callbacks
            // will be added once a thread-safe telemetry bridge is in place.
            session.add_started_callback(move |_ctx| {
                // Intentionally empty — the session is now running.
            });

            // Register the raw event callback that captures every
            // EVENT_RECORD from ProcessTrace and sends a lightweight
            // snapshot through the channel to the async receiver loop.
            let tx = event_tx;
            session.set_raw_event_callback(move |event| {
                let data = EtwEventData {
                    provider_id: event.EventHeader.ProviderId.to_bytes(),
                    timestamp: event.EventHeader.TimeStamp,
                    process_id: event.EventHeader.ProcessId,
                    thread_id: event.EventHeader.ThreadId,
                    event_id: event.EventHeader.EventDescriptor.Id,
                    opcode: event.EventHeader.EventDescriptor.Opcode,
                    version: event.EventHeader.EventDescriptor.Version,
                    level: event.EventHeader.EventDescriptor.Level,
                    keywords: event.EventHeader.EventDescriptor.Keyword,
                };

                // Use try_send to avoid blocking the ProcessTrace thread.
                // If the channel is full, drop the event (backpressure).
                let _ = tx.try_send(data);
            });

            // `parse_until` spawns a management thread internally and runs the
            // blocking `ProcessTrace` on the current thread's context. It
            // returns when the `until` closure returns `true` and the trace
            // session is torn down.
            //
            // NOTE: The event_tx sender is moved into the `parse_until` scope.
            // When `parse_until` returns, the sender is dropped, closing the
            // channel and signalling the async receiver loop.
            let result = session.parse_until(
                &session_name,
                move || stop_flag_clone.load(Ordering::Acquire),
            );

            if let Err(_e) = result {
                // Error logging from the ETW OS thread is deferred until a
                // thread-safe telemetry bridge is available. The channel
                // closing will signal the async receiver loop that the
                // session has terminated.
            }
        })
        .map_err(|e| Error::InternalError {
            message: format!("failed to spawn ETW session thread: {e}"),
        })?;

    let handle = SessionHandle { stop_flag };

    Ok((handle, event_rx))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_guid_standard_format() {
        let guid = parse_guid("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716").expect("valid GUID");
        assert_eq!(guid.data1, 0x22fb2cd6);
        assert_eq!(guid.data2, 0x0e7b);
        assert_eq!(guid.data3, 0x422b);
        assert_eq!(guid.data4, [0xa0, 0xc7, 0x2f, 0xad, 0x1f, 0xd0, 0xe7, 0x16]);
    }

    #[test]
    fn parse_guid_with_braces() {
        let guid =
            parse_guid("{22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716}").expect("valid GUID");
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
        assert_eq!(trace_level_to_etw(&TraceLevel::Critical), etw::LEVEL_CRITICAL);
        assert_eq!(trace_level_to_etw(&TraceLevel::Error), etw::LEVEL_ERROR);
        assert_eq!(trace_level_to_etw(&TraceLevel::Warning), etw::LEVEL_WARNING);
        assert_eq!(
            trace_level_to_etw(&TraceLevel::Information),
            etw::LEVEL_INFORMATION
        );
        assert_eq!(trace_level_to_etw(&TraceLevel::Verbose), etw::LEVEL_VERBOSE);
    }
}
