// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use bytes::Bytes;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;

use super::error::Error;

/// Hard limit for gzip-compressed payload accepted by the Azure Monitor Gateway.
const COMPRESSED_LIMIT: usize = 1024 * 1024;

/// Conservative headroom subtracted from COMPRESSED_LIMIT to account for gzip
/// framing overhead (sync flush markers, deflate end-of-stream, gzip trailer).
const GZIP_OVERHEAD: usize = 4 * 1024;

/// Working target for compressed output size. The batcher aims to keep the
/// final gzip payload under this value so it stays safely below COMPRESSED_LIMIT.
const TARGET_COMPRESSED_LIMIT: usize = COMPRESSED_LIMIT - GZIP_OVERHEAD;

/// Hard limit for uncompressed (decompressed) payload size. The Azure Monitor
/// Logs Ingestion API rejects payloads whose decompressed JSON exceeds 50 MiB.
/// The batcher finalizes the batch when the cumulative uncompressed input
/// approaches this limit.
const UNCOMPRESSED_LIMIT: usize = 50 * 1024 * 1024;

/// Maximum number of gzip sync flushes allowed per batch. Each flush adds a
/// small amount of framing overhead. This cap prevents excessive overhead
/// accumulation with highly compressible data. Extremely unlikely to reach
/// this limit due to the uncompressed size limit.
const MAX_GZIP_FLUSH_COUNT: usize = 128;

/// Accumulates JSON entries into gzip-compressed batches that stay under a size limit.
pub struct GzipBatcher {
    buf: GzEncoder<Vec<u8>>,
    compression: Compression,
    remaining_size: usize,
    uncompressed_size: usize,
    total_uncompressed_size: usize,
    row_count: u64,
    flush_count: usize,
    batch_id: u64,
    pending_batch: Option<GzipResult>,
}

/// Result of pushing an entry into the batcher.
pub enum PushResult {
    /// Entry accepted into the current batch (returns batch id).
    Ok(u64),
    /// Entry exceeds the maximum allowed size.
    TooLarge,
    /// A batch is ready to be taken (returns the new batch id).
    BatchReady(u64),
}

/// Result of finalizing the current batch.
pub enum FinalizeResult {
    /// No data was present to finalize.
    Empty,
    /// Batch finalized successfully.
    Ok,
}

/// A completed gzip-compressed batch.
pub struct GzipResult {
    /// Unique identifier for this batch.
    pub batch_id: u64,
    /// The gzip-compressed payload.
    pub compressed_data: Bytes,
    /// Number of entries in this batch.
    pub row_count: u64,
    /// Number of gzip sync flushes performed while building this batch.
    pub flush_count: usize,
    /// Total uncompressed size of the JSON payload in bytes (including structural bytes).
    pub uncompressed_size: usize,
}

impl GzipBatcher {
    /// Create a new batcher with the given gzip compression level (0-9).
    #[must_use]
    pub fn new(compression_level: u32) -> Self {
        let compression = Compression::new(compression_level);
        Self {
            buf: Self::new_encoder(compression),
            compression,
            remaining_size: TARGET_COMPRESSED_LIMIT,
            uncompressed_size: 0,
            total_uncompressed_size: 0,
            row_count: 0,
            flush_count: 0,
            batch_id: 0,
            pending_batch: None,
        }
    }

    fn new_encoder(compression: Compression) -> GzEncoder<Vec<u8>> {
        GzEncoder::new(Vec::with_capacity(TARGET_COMPRESSED_LIMIT), compression)
    }

    /// Returns `true` if the encoder buffer contains uncommitted data.
    #[inline]
    pub fn has_pending_data(&self) -> bool {
        !self.buf.get_ref().is_empty()
    }

    /// Push an entry into the batcher. Returns the push result.
    #[inline]
    pub fn push(&mut self, data: Bytes) -> Result<PushResult, Error> {
        if self.pending_batch.is_some() {
            return Ok(PushResult::BatchReady(self.batch_id));
        }

        self.push_internal(data)
    }

    fn push_internal(&mut self, data: Bytes) -> Result<PushResult, Error> {
        // Account for structural JSON bytes: '[' or ',' prefix + ']' for finalization.
        // Reject entries that can't possibly fit in a single batch.
        if data.len() + 2 > TARGET_COMPRESSED_LIMIT {
            return Ok(PushResult::TooLarge);
        }

        let is_first_entry = self.row_count == 0;

        if is_first_entry {
            self.batch_id += 1;
            self.buf.write_all(b"[").map_err(Error::BatchPushFailed)?;
            self.total_uncompressed_size += 1; // '['
        }

        // Include structural overhead: ',' for non-first entries, ']' for finalization.
        let structural_overhead = if is_first_entry { 0 } else { 1 }; // ','
        let finalize_overhead = 1; // ']'
        let entry_cost = structural_overhead + data.len() + finalize_overhead;
        let next_size = self.uncompressed_size + entry_cost;
        let must_flush = next_size > self.remaining_size;

        if must_flush {
            self.buf.flush().map_err(Error::BatchPushFailed)?;

            self.flush_count += 1;
            let compressed_size = self.buf.get_ref().len();

            self.remaining_size = TARGET_COMPRESSED_LIMIT.saturating_sub(compressed_size);
            self.uncompressed_size = 0;
        }

        // Recompute after flush: uncompressed_size was reset so
        // next_size must be recalculated with current state.
        let next_size = self.uncompressed_size + entry_cost;
        let must_finalize = next_size > self.remaining_size
            || self.total_uncompressed_size + entry_cost >= UNCOMPRESSED_LIMIT
            || self.flush_count >= MAX_GZIP_FLUSH_COUNT;

        if must_finalize {
            let finalize_result = self.finalize()?;
            // We attempt to push the data to the next batch.
            // If this fails, we propagate the error.
            // Note: If finalize succeeded, we have a pending batch ready.
            // The recursive push will start a new batch (id+1).
            let _ = self.push_internal(data)?;

            match finalize_result {
                FinalizeResult::Empty => Ok(PushResult::Ok(self.batch_id)),
                FinalizeResult::Ok => {
                    // this is the new batch id that we are currently building
                    // the pending batch id is available in the pending_batch field
                    Ok(PushResult::BatchReady(self.batch_id))
                }
            }
        } else {
            if !is_first_entry {
                self.buf.write_all(b",").map_err(Error::BatchPushFailed)?;
                self.uncompressed_size += 1;
            }
            self.buf.write_all(&data).map_err(Error::BatchPushFailed)?;
            self.uncompressed_size += data.len();
            self.total_uncompressed_size += structural_overhead + data.len();
            self.row_count += 1;

            Ok(PushResult::Ok(self.batch_id))
        }
    }

    /// Finalize the current batch, making it available via [`take_pending_batch`](Self::take_pending_batch).
    pub fn finalize(&mut self) -> Result<FinalizeResult, Error> {
        if self.buf.get_ref().is_empty() {
            return Ok(FinalizeResult::Empty);
        }

        self.buf
            .write_all(b"]")
            .map_err(Error::BatchFinalizeFailed)?;

        let old_buf = std::mem::replace(&mut self.buf, Self::new_encoder(self.compression));

        let compressed_data = old_buf.finish().map_err(Error::BatchFinalizeFailed)?;

        let row_count = self.row_count;
        let flush_count = self.flush_count;
        let uncompressed_size = self.total_uncompressed_size + 1; // +1 for ']'

        // Reset state
        self.remaining_size = TARGET_COMPRESSED_LIMIT;
        self.uncompressed_size = 0;
        self.total_uncompressed_size = 0;
        self.row_count = 0;
        self.flush_count = 0;

        self.pending_batch = Some(GzipResult {
            batch_id: self.batch_id,
            compressed_data: Bytes::from(compressed_data),
            row_count,
            flush_count,
            uncompressed_size,
        });

        Ok(FinalizeResult::Ok)
    }

    /// Take the pending completed batch, if any.
    #[inline]
    pub fn take_pending_batch(&mut self) -> Option<GzipResult> {
        self.pending_batch.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::GzDecoder;
    use rand::RngExt;
    use std::io::Read;

    const ONE_MB: usize = 1024 * 1024;

    // ==================== Test Helpers ====================

    fn generate_data(size: usize) -> Vec<u8> {
        let mut rng = rand::rng();
        let id = rng.random_range(10000..99999);
        let timestamp = rng.random_range(1600000000..1700000000);

        let base = format!(r#"{{"id":{},"ts":{},"msg":""#, id, timestamp);
        let closing = r#""}"#;

        let padding_needed = size.saturating_sub(base.len() + closing.len());
        let padding: String = (0..padding_needed)
            .map(|_| rng.random_range(b'a'..=b'z') as char)
            .collect();

        format!("{}{}{}", base, padding, closing).into_bytes()
    }

    fn generate_1kb_data() -> Vec<u8> {
        generate_data(1024)
    }

    fn decompress_and_validate(data: &Bytes) -> String {
        let mut decoder = GzDecoder::new(&data[..]);
        let mut decompressed = String::new();
        _ = decoder
            .read_to_string(&mut decompressed)
            .expect("Should decompress");

        let trimmed = decompressed.trim();
        assert!(trimmed.starts_with('['), "Should start with [");
        assert!(trimmed.ends_with(']'), "Should end with ]");

        // Remove all whitespace to check for structural issues like [, or ,]
        let no_whitespace: String = decompressed
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();

        // Ensure no invalid comma placement (ignoring whitespace)
        assert!(
            !no_whitespace.contains("[,") && !no_whitespace.contains(",]"),
            "Invalid comma placement found in JSON: {}",
            decompressed
        );

        decompressed
    }

    // ==================== Construction & State Tests ====================

    #[test]
    fn test_new_creates_empty_batcher() {
        let batcher = GzipBatcher::new(1);
        assert!(!batcher.has_pending_data());
        assert!(batcher.pending_batch.is_none());
    }

    #[test]
    fn test_has_pending_data_lifecycle() {
        let mut batcher = GzipBatcher::new(1);
        assert!(!batcher.has_pending_data());

        let _ = batcher.push(generate_1kb_data().into()).unwrap();
        assert!(batcher.has_pending_data());

        let _ = batcher.finalize().unwrap();
        assert!(!batcher.has_pending_data());
    }

    #[test]
    fn test_take_pending_batch_lifecycle() {
        let mut batcher = GzipBatcher::new(1);
        assert!(batcher.take_pending_batch().is_none());

        let _ = batcher.push(generate_1kb_data().into()).unwrap();
        let _ = batcher.finalize().unwrap();

        let batch = batcher.take_pending_batch();
        assert!(batch.is_some());
        assert!(batcher.take_pending_batch().is_none());
    }

    // ==================== Push Logic Tests ====================

    #[test]
    fn test_push_single_entry() {
        let mut batcher = GzipBatcher::new(1);
        match batcher.push(generate_1kb_data().into()).unwrap() {
            PushResult::Ok(id) => assert_eq!(id, 1),
            _ => panic!("Should be Ok"),
        }
    }

    #[test]
    fn test_push_too_large_entry() {
        let mut batcher = GzipBatcher::new(1);
        let large_data = vec![b'x'; ONE_MB];
        match batcher.push(large_data.into()).unwrap() {
            PushResult::TooLarge => {} // Expected
            _ => panic!("Should be TooLarge"),
        }
    }

    #[test]
    fn test_push_just_under_limit() {
        let mut batcher = GzipBatcher::new(1);
        // Max allowed: TARGET_COMPRESSED_LIMIT - 2 (for '[' and ']')
        let data = vec![b'x'; TARGET_COMPRESSED_LIMIT - 2];
        match batcher.push(data.into()).unwrap() {
            PushResult::Ok(_) | PushResult::BatchReady(_) => {} // Expected
            PushResult::TooLarge => panic!("Should fit"),
        }
    }

    #[test]
    fn test_push_returns_batch_ready_when_pending_exists() {
        let mut batcher = GzipBatcher::new(1);

        // Force a pending batch
        loop {
            if let PushResult::BatchReady(_) = batcher.push(generate_1kb_data().into()).unwrap() {
                break;
            }
        }

        // Subsequent pushes should return BatchReady
        match batcher.push(generate_1kb_data().into()).unwrap() {
            PushResult::BatchReady(_) => {}
            _ => panic!("Should return BatchReady"),
        }
    }

    #[test]
    fn test_push_batch_id_increments() {
        let mut batcher = GzipBatcher::new(1);
        let mut last_id = 0;

        for _ in 0..3 {
            loop {
                match batcher.push(generate_1kb_data().into()).unwrap() {
                    PushResult::Ok(_) => continue,
                    PushResult::BatchReady(id) => {
                        assert!(id > last_id);
                        last_id = id;
                        let _ = batcher.take_pending_batch();
                        break;
                    }
                    _ => panic!("Unexpected"),
                }
            }
        }
    }

    // ==================== Flush & Finalize Tests ====================

    #[test]
    fn test_flush_empty_batcher() {
        let mut batcher = GzipBatcher::new(1);
        match batcher.finalize().unwrap() {
            FinalizeResult::Empty => {}
            _ => panic!("Should be Empty"),
        }
    }

    #[test]
    fn test_flush_with_data() {
        let mut batcher = GzipBatcher::new(1);
        let _ = batcher.push(generate_1kb_data().into()).unwrap();

        match batcher.finalize().unwrap() {
            FinalizeResult::Ok => {
                let batch = batcher.take_pending_batch().unwrap();
                assert!(batch.row_count > 0);
                assert!(!batch.compressed_data.is_empty());
            }
            _ => panic!("Should be Ok"),
        }
    }

    #[test]
    fn test_flush_multiple_times() {
        let mut batcher = GzipBatcher::new(1);

        // Batch 1
        let _ = batcher.push(generate_1kb_data().into()).unwrap();
        let _ = batcher.finalize().unwrap();
        let b1 = batcher.take_pending_batch().unwrap();

        // Batch 2
        let _ = batcher.push(generate_1kb_data().into()).unwrap();
        let _ = batcher.finalize().unwrap();
        let b2 = batcher.take_pending_batch().unwrap();

        assert!(b2.batch_id > b1.batch_id);
    }

    // ==================== Integration & Format Tests ====================

    #[test]
    fn test_output_is_valid_gzip_json_array() {
        let mut batcher = GzipBatcher::new(1);
        for _ in 0..10 {
            let _ = batcher.push(generate_1kb_data().into()).unwrap();
        }
        let _ = batcher.finalize().unwrap();

        let batch = batcher.take_pending_batch().unwrap();
        let decompressed = decompress_and_validate(&batch.compressed_data);

        let parsed: Vec<serde_json::Value> = serde_json::from_str(&decompressed).unwrap();
        assert_eq!(parsed.len(), 10);
    }

    #[test]
    fn test_row_count_accuracy() {
        let mut batcher = GzipBatcher::new(1);
        for _ in 0..42 {
            let _ = batcher.push(generate_1kb_data().into()).unwrap();
        }
        let _ = batcher.finalize().unwrap();
        assert_eq!(batcher.take_pending_batch().unwrap().row_count, 42);
    }

    #[test]
    fn test_interleaved_push_and_take() {
        let mut batcher = GzipBatcher::new(1);

        let _ = batcher.push(generate_1kb_data().into()).unwrap();
        let _ = batcher.finalize().unwrap();
        let _ = batcher.take_pending_batch();

        let _ = batcher.push(generate_1kb_data().into()).unwrap();
        let _ = batcher.finalize().unwrap();
        let b2 = batcher.take_pending_batch().unwrap();

        assert_eq!(b2.row_count, 1);
    }

    // ==================== Comma Handling Regression Tests ====================

    #[test]
    fn test_no_leading_comma_after_bracket() {
        let mut batcher = GzipBatcher::new(1);
        let _ = batcher.push(Bytes::from_static(b"1")).unwrap();
        let _ = batcher.push(Bytes::from_static(b"2")).unwrap();
        let _ = batcher.finalize().unwrap();

        let json = decompress_and_validate(&batcher.take_pending_batch().unwrap().compressed_data);
        assert_eq!(json, "[1,2]");
    }

    #[test]
    fn test_no_trailing_comma_before_bracket() {
        let mut batcher = GzipBatcher::new(1);
        let _ = batcher.push(Bytes::from_static(b"1")).unwrap();
        let _ = batcher.finalize().unwrap();

        let json = decompress_and_validate(&batcher.take_pending_batch().unwrap().compressed_data);
        assert_eq!(json, "[1]");
    }

    #[test]
    fn test_format_valid_after_auto_finalize() {
        let mut batcher = GzipBatcher::new(1);

        // Fill until split
        loop {
            if let PushResult::BatchReady(_) = batcher.push(generate_1kb_data().into()).unwrap() {
                break;
            }
        }

        let batch = batcher.take_pending_batch().unwrap();
        let json = decompress_and_validate(&batch.compressed_data);

        assert!(!json.contains("[,"));
        assert!(!json.contains(",]"));
        assert!(serde_json::from_str::<Vec<serde_json::Value>>(&json).is_ok());
    }

    #[test]
    fn test_format_valid_for_second_batch() {
        let mut batcher = GzipBatcher::new(1);

        // Fill first batch and discard
        loop {
            if let PushResult::BatchReady(_) = batcher.push(generate_1kb_data().into()).unwrap() {
                break;
            }
        }
        let _ = batcher.take_pending_batch();

        // Second batch
        // Note: This batch will start with the "spillover" entry that triggered the previous BatchReady.
        // We append more data to it.
        let _ = batcher.push(Bytes::from_static(b"1")).unwrap();
        let _ = batcher.push(Bytes::from_static(b"2")).unwrap();
        let _ = batcher.finalize().unwrap();

        // decompress_and_validate checks for [, and ,] and [] wrapping
        let json = decompress_and_validate(&batcher.take_pending_batch().unwrap().compressed_data);

        // If it deserializes successfully, the format is valid.
        let parsed: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&json);
        assert!(
            parsed.is_ok(),
            "Second batch must be valid JSON. Got error: {:?}. Content: {}",
            parsed.err(),
            json
        );

        // We can also verify it contains at least the elements we explicitly added
        let array = parsed.unwrap();
        assert!(array.len() >= 2);
        assert_eq!(array[array.len() - 2], serde_json::json!(1));
        assert_eq!(array[array.len() - 1], serde_json::json!(2));
    }

    // ==================== Size Limit Tests ====================

    struct BatchStats {
        size: usize,
        flush_count: usize,
        row_count: u64,
        decompressed_size: usize,
    }

    /// Helper: fill a batcher until BatchReady, return batch stats.
    fn fill_to_batch_ready(compression_level: u32, gen_chunk: &dyn Fn() -> Vec<u8>) -> BatchStats {
        let mut batcher = GzipBatcher::new(compression_level);
        loop {
            let chunk = gen_chunk();
            match batcher.push(chunk.into()).unwrap() {
                PushResult::Ok(_) => continue,
                PushResult::BatchReady(_) => break,
                PushResult::TooLarge => panic!("Should not happen with small chunks"),
            }
        }
        let batch = batcher.take_pending_batch().unwrap();
        let mut decoder = GzDecoder::new(&batch.compressed_data[..]);
        let mut decompressed = Vec::new();
        _ = decoder
            .read_to_end(&mut decompressed)
            .expect("gzip decode should succeed in test helper");
        BatchStats {
            size: batch.compressed_data.len(),
            flush_count: batch.flush_count,
            row_count: batch.row_count,
            decompressed_size: decompressed.len(),
        }
    }

    /// Waste threshold is computed relative to the entry size being tested:
    /// the maximum waste is one full entry that didn't fit.
    fn assert_batch_utilization(stats: &BatchStats, entry_size: usize, label: &str) {
        // Hard limit: must never exceed ONE_MB.
        assert!(
            stats.size <= ONE_MB,
            "{label}: batch size {} exceeds hard limit (ONE_MB = {ONE_MB})",
            stats.size
        );
        // Utilization: should be close to TARGET_COMPRESSED_LIMIT.
        let target_limit_f64 = TARGET_COMPRESSED_LIMIT as f64;
        let utilization = stats.size as f64 / target_limit_f64 * 100.0;
        let waste = 100.0 - utilization;
        let max_waste = entry_size as f64 / target_limit_f64 * 100.0;
        eprintln!(
            "{label}: size={} utilization={utilization:.2}% waste={waste:.2}% max_waste={max_waste:.2}% flush_count={}",
            stats.size, stats.flush_count
        );
        assert!(
            waste <= max_waste,
            "{label}: batch size {} ({utilization:.1}% utilization, \
             {waste:.1}% waste) exceeds {max_waste:.1}% waste threshold for {entry_size}B entries",
            stats.size
        );
        assert!(
            stats.flush_count <= MAX_GZIP_FLUSH_COUNT,
            "{label}: flush count {} exceeds limit {MAX_GZIP_FLUSH_COUNT}",
            stats.flush_count
        );
    }

    /// JSON with random hex payload.
    fn generate_hex_json(size: usize) -> Vec<u8> {
        let mut rng = rand::rng();
        let hex = b"0123456789abcdef";
        let base = r#"{"v":""#;
        let closing = r#""}"#;
        let val_len = size.saturating_sub(base.len() + closing.len());
        let val: String = (0..val_len)
            .map(|_| hex[rng.random_range(0..16usize)] as char)
            .collect();
        format!("{base}{val}{closing}").into_bytes()
    }

    /// Random printable ASCII payload JSON.
    fn generate_random_ascii_json(size: usize) -> Vec<u8> {
        let mut rng = rand::rng();
        let chars = b" !#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
        let base = r#"{"v":""#;
        let closing = r#""}"#;
        let val_len = size.saturating_sub(base.len() + closing.len());
        let val: String = (0..val_len)
            .map(|_| chars[rng.random_range(0..chars.len())] as char)
            .collect();
        format!("{base}{val}{closing}").into_bytes()
    }

    /// Generates a JSON entry that is mostly identical (very compressible) with
    /// a small varying tail. Used to exercise the flush cap with high compression ratios.
    fn generate_highly_compressible_json(
        total_size: usize,
        sequence: usize,
        varying_tail_len: usize,
    ) -> Vec<u8> {
        let prefix = r#"{"svc":"load-generator","ver":"1.0.0","iid":"instance-001","sev":"INFO","sevn":9,"tid":1,"tname":"main","msg":""#;
        let closing = r#""}"#;
        let body_len = total_size.saturating_sub(prefix.len() + closing.len());
        let fill_len = body_len.saturating_sub(varying_tail_len);
        let fill = "a".repeat(fill_len);
        let tail: String = format!("{:0>width$}", sequence, width = varying_tail_len)
            .chars()
            .take(varying_tail_len)
            .collect();
        format!("{prefix}{fill}{tail}{closing}").into_bytes()
    }

    /// Generates JSON that mimics what CEF syslog data looks like after the
    /// Azure Monitor transformer. The CEF template is ~900 bytes with only
    /// 4 numeric fields randomized per message, plus fresh timestamps.
    fn generate_cef_syslog_style_json(msg_size: usize, sequence: usize) -> Vec<u8> {
        let cn1 = 10000000 + (sequence * 7 + 13) % 90000000;
        let cn2 = 1000000000u64 + (sequence as u64 * 31 + 17) % 9000000000u64;
        let spt = 10000 + (sequence * 3 + 5) % 55535;
        let dvcpid = 10000 + (sequence * 11 + 7) % 90000;

        let cef_body = format!(
            "CEF:0|PaloAltoNetworks|PAN-OS|9.1.8|SSH2 Login Attempt(31914)|\
             SSH2 Login Attempt(31914)|1|act=alert \
             actionflags=0x2000000000000000 app=ssh cat=any cn1={cn1} \
             cn2={cn2} cnt=1 cs1=THREAT cs2=vulnerability cs3=Tap_Allow \
             cs5= cs6= destinationTranslatedAddress=0.0.0.0 \
             destinationTranslatedPort=0 deviceExternalId=0120010106097 \
             deviceInboundInterface=ethernet1/3 \
             deviceOutboundInterface=ethernet1/3 dntdom=Tap domeid=vsys1 \
             dpt=22 dst=172.21.166.15 dstloc=172.16.0.0-172.31.255.255 \
             duid= duser= dvchost=PA-820 dvcpid={dvcpid} \
             end=Jun 23 2021 20:36:07 GMT fileHash= fileId=0 filePath= \
             fileType= flags=0x80002000 fname= logset=InfoCIC-LogForwarding \
             msg=informational outcome=client-to-server proto=tcp \
             request=\\\"\\\" requestClientApplication= requestContext= \
             requestMethod= rt=Jun 23 2021 20:36:07 GMT sntdom=Tap \
             sourceTranslatedAddress=0.0.0.0 sourceTranslatedPort=0 \
             spt={spt} src=172.21.76.92 \
             srcloc=172.16.0.0-172.31.255.255 suid= suser= \
             PanOSThreatCategory=brute-force PanOSParentSessionID=0 \
             PanOSParentStartTime= PanOSContentVer=AppThreat-8348-6427 \
             PanOSTunnelID=0 PanOSTunnelType=N/A"
        );

        let padding_needed = msg_size.saturating_sub(cef_body.len());
        let padding = " ".repeat(padding_needed);

        let second = sequence % 60;
        let minute = (sequence / 60) % 60;
        let nanos = ((sequence as u32).wrapping_mul(1_234_567)) % 1_000_000_000;
        let observed_nanos = nanos.wrapping_add(111_111_111) % 1_000_000_000;

        format!(
            concat!(
                r#"{{"ServiceName":"syslog-collector","ServiceVersion":"1.0.0","#,
                r#""ServiceInstanceId":"instance-001","#,
                r#""Message":"{}{}","#,
                r#""SeverityText":"notice","SeverityNumber":5,"#,
                r#""TimeGenerated":"2026-04-03T12:{:02}:{:02}.{:09}+00:00","#,
                r#""ObservedTime":"2026-04-03T12:{:02}:{:02}.{:09}+00:00"}}"#
            ),
            cef_body, padding, minute, second, nanos, minute, second, observed_nanos,
        )
        .into_bytes()
    }

    /// Verify that decompressed batch size never exceeds UNCOMPRESSED_LIMIT
    /// using identical (maximally compressible) JSON entries.
    #[test]
    fn test_uncompressed_limit_enforced_with_identical_entries() {
        // Identical entries compress extremely well — without the uncompressed
        // limit the batcher would pack hundreds of MiBs into a tiny gzip payload.
        let entries: &[&[u8]] = &[br#"{"a":1}"#, br#"{"b":2}"#, br#"{"c":3}"#];
        let mut idx = 0usize;

        let mut batcher = GzipBatcher::new(6);
        loop {
            match batcher
                .push(Bytes::from_static(entries[idx % entries.len()]))
                .unwrap()
            {
                PushResult::Ok(_) => {
                    idx += 1;
                    continue;
                }
                PushResult::BatchReady(_) => break,
                PushResult::TooLarge => panic!("Should not happen"),
            }
        }

        let batch = batcher.take_pending_batch().unwrap();
        let mut decoder = GzDecoder::new(&batch.compressed_data[..]);
        let mut decompressed = Vec::new();
        _ = decoder.read_to_end(&mut decompressed).unwrap();

        eprintln!(
            "identical_entries: compressed={} decompressed={} rows={} flushes={}",
            batch.compressed_data.len(),
            decompressed.len(),
            batch.row_count,
            batch.flush_count
        );

        assert!(
            decompressed.len() <= UNCOMPRESSED_LIMIT,
            "decompressed size {} exceeds UNCOMPRESSED_LIMIT {}",
            decompressed.len(),
            UNCOMPRESSED_LIMIT
        );
        assert!(
            batch.compressed_data.len() <= COMPRESSED_LIMIT,
            "compressed size {} exceeds COMPRESSED_LIMIT {}",
            batch.compressed_data.len(),
            COMPRESSED_LIMIT
        );
        // With the 50 MiB uncompressed limit, flush counts should stay well under 100
        // even for maximally compressible data.
        assert!(
            batch.flush_count < 100,
            "flush count {} should be under 100 with UNCOMPRESSED_LIMIT in effect",
            batch.flush_count
        );
    }

    /// Verify that decompressed batch size never exceeds UNCOMPRESSED_LIMIT
    /// using highly compressible entries with tiny varying tails.
    #[test]
    fn test_uncompressed_limit_enforced_with_compressible_entries() {
        let counter = std::cell::Cell::new(0usize);
        let mut batcher = GzipBatcher::new(6);
        loop {
            let seq = counter.get();
            counter.set(seq + 1);
            let entry = generate_highly_compressible_json(200, seq, 4);
            match batcher.push(entry.into()).unwrap() {
                PushResult::Ok(_) => continue,
                PushResult::BatchReady(_) => break,
                PushResult::TooLarge => panic!("Should not happen"),
            }
        }

        let batch = batcher.take_pending_batch().unwrap();
        let mut decoder = GzDecoder::new(&batch.compressed_data[..]);
        let mut decompressed = Vec::new();
        _ = decoder.read_to_end(&mut decompressed).unwrap();

        eprintln!(
            "compressible_entries: compressed={} decompressed={} rows={} flushes={}",
            batch.compressed_data.len(),
            decompressed.len(),
            batch.row_count,
            batch.flush_count
        );

        assert!(
            decompressed.len() <= UNCOMPRESSED_LIMIT,
            "decompressed size {} exceeds UNCOMPRESSED_LIMIT {}",
            decompressed.len(),
            UNCOMPRESSED_LIMIT
        );
        assert!(
            batch.compressed_data.len() <= COMPRESSED_LIMIT,
            "compressed size {} exceeds COMPRESSED_LIMIT {}",
            batch.compressed_data.len(),
            COMPRESSED_LIMIT
        );
    }

    /// Measure batch dimensions for CEF syslog-style payloads and verify
    /// compressed output stays under 1 MiB.
    #[test]
    fn test_cef_syslog_batch_dimensions() {
        eprintln!(
            "{:<45} {:>10} {:>10} {:>8} {:>8} {:>8}",
            "label", "compressed", "decompress", "rows", "ratio", "flushes"
        );

        for level in [1u32, 6] {
            for msg_size in [800usize, 1000, 1200] {
                let counter = std::cell::Cell::new(0usize);
                let stats = fill_to_batch_ready(level, &|| {
                    let seq = counter.get();
                    counter.set(seq + 1);
                    generate_cef_syslog_style_json(msg_size, seq)
                });

                let ratio = stats.decompressed_size as f64 / stats.size as f64;
                let label = format!("cef_syslog/{msg_size}B/lvl{level}");
                eprintln!(
                    "{label:<45} {:<10} {:<10} {:<8} {:<8.1} {:<8}",
                    stats.size, stats.decompressed_size, stats.row_count, ratio, stats.flush_count
                );

                assert!(
                    stats.size <= ONE_MB,
                    "{label}: compressed batch {} exceeds 1 MiB",
                    stats.size
                );
                assert!(
                    stats.decompressed_size <= UNCOMPRESSED_LIMIT,
                    "{label}: decompressed batch {} exceeds 50 MiB limit ({UNCOMPRESSED_LIMIT})",
                    stats.decompressed_size
                );
            }
        }
    }

    #[test]
    fn test_batch_utilization_json_log_data() {
        for level in [1u32, 6, 9] {
            for entry_size in [256, 1024, 2048, 16384, 65536] {
                let stats = fill_to_batch_ready(level, &|| generate_data(entry_size));
                assert_batch_utilization(
                    &stats,
                    entry_size,
                    &format!("json_log/{entry_size}B/level_{level}"),
                );
            }
        }
    }

    /// Hex-payload JSON: minimal object with random hex value.
    #[test]
    fn test_batch_utilization_hex_json_data() {
        for level in [1u32, 6, 9] {
            for entry_size in [10, 256, 1024, 16384, 65536] {
                let stats = fill_to_batch_ready(level, &|| generate_hex_json(entry_size));
                assert_batch_utilization(
                    &stats,
                    entry_size,
                    &format!("hex_json/{entry_size}B/level_{level}"),
                );
            }
        }
    }

    /// Smallest valid JSON entries (`1`), exercises structural byte accounting
    /// with a high ratio of commas to data.
    #[test]
    fn test_batch_utilization_tiny_json_entries() {
        for level in [1u32, 6, 9] {
            let stats = fill_to_batch_ready(level, &|| {
                let mut rng = rand::rng();
                vec![rng.random_range(b'0'..=b'9')]
            });
            assert_batch_utilization(&stats, 1, &format!("tiny_json/level_{level}"));
        }
    }

    /// Varying JSON entry sizes within a single batch.
    /// Mixes tiny (1-10B), medium (256-1KB), and large (8-16KB) entries
    /// to exercise size accounting across different granularities.
    #[test]
    fn test_batch_utilization_mixed_json_sizes() {
        let sizes = [1, 5, 10, 50, 256, 512, 1024, 4096, 8192, 16384];
        for level in [1u32, 6, 9] {
            let counter = std::cell::Cell::new(0usize);
            let stats = fill_to_batch_ready(level, &|| {
                let i = counter.get();
                counter.set(i + 1);
                generate_data(sizes[i % sizes.len()])
            });
            assert_batch_utilization(&stats, 16384, &format!("mixed_json/level_{level}"));
        }
    }

    /// Minimal JSON with random hex payload.
    #[test]
    fn test_1mb_limit_with_hex_json_payload() {
        let hex = b"0123456789abcdef";
        let mut rng = rand::rng();
        for _ in 0..5 {
            let mut batcher = GzipBatcher::new(1);
            loop {
                // Minimal JSON: {"v":"<random hex>"}
                let val: String = (0..200)
                    .map(|_| hex[rng.random_range(0..16usize)] as char)
                    .collect();
                let entry = format!(r#"{{"v":"{val}"}}"#).into_bytes();
                match batcher.push(entry.into()).unwrap() {
                    PushResult::Ok(_) => continue,
                    PushResult::BatchReady(_) => break,
                    PushResult::TooLarge => panic!("Should not happen"),
                }
            }

            let batch = batcher.take_pending_batch().unwrap();
            assert!(
                batch.compressed_data.len() <= ONE_MB,
                "Batch size {} exceeds 1MB limit with hex-payload JSON",
                batch.compressed_data.len()
            );
        }
    }

    /// Random bytes encoded as hex, wrapped in minimal JSON.
    fn generate_random_bytes_json(size: usize) -> Vec<u8> {
        use rand::RngExt;
        let mut rng = rand::rng();
        let base = r#"{"d":""#;
        let closing = r#""}"#;
        let val_len = size.saturating_sub(base.len() + closing.len());
        // Hex-encode random bytes to keep valid JSON while remaining hard to compress.
        let raw_bytes: Vec<u8> = (0..val_len / 2).map(|_| rng.random::<u8>()).collect();
        let hex_val: String = raw_bytes.iter().map(|b| format!("{b:02x}")).collect();
        let val = &hex_val[..val_len.min(hex_val.len())];
        format!("{base}{val}{closing}").into_bytes()
    }

    /// Replay seed 89: verifies the batch stays under 1 MiB with
    /// highly incompressible data near the limit boundary. This is
    /// also a reproduction test for the gzip framing overhead issue
    /// that was observed in rare cases in early testing when the
    /// headroom for gzip overhead was set to 30 bytes.
    #[test]
    fn test_replay_seed_89() {
        use rand::SeedableRng;

        let mut rng = rand::rngs::SmallRng::seed_from_u64(89);
        let mode = rng.random_range(0..4usize);
        let mut batcher = GzipBatcher::new(6);

        let mut entry_index = 0u64;
        let mut entry_sizes: Vec<usize> = Vec::new();

        loop {
            let entry_size = match mode {
                0 => rng.random_range(64..512usize),
                1 => rng.random_range(512..4096usize),
                2 => rng.random_range(4096..16384usize),
                _ => rng.random_range(16384..32768usize),
            };

            let entry: Vec<u8> = match mode {
                0 => generate_hex_json(entry_size),
                1 => generate_random_ascii_json(entry_size),
                2 => generate_random_bytes_json(entry_size),
                _ => generate_hex_json(entry_size),
            };

            let actual_len = entry.len();
            entry_sizes.push(actual_len);

            match batcher.push(entry.into()).unwrap() {
                PushResult::Ok(_) => {
                    entry_index += 1;
                    continue;
                }
                PushResult::BatchReady(_) => {
                    eprintln!("--- Replay of seed=89, mode={mode} ---");
                    eprintln!("BatchReady triggered at entry_index={entry_index} (0-based)");
                    eprintln!("Entry that triggered finalize: size={actual_len} bytes");
                    eprintln!("Total entries in batch (before spillover): {entry_index}");

                    let start = entry_sizes.len().saturating_sub(5);
                    eprintln!("Last entry sizes: {:?}", &entry_sizes[start..]);

                    let min = entry_sizes.iter().min().unwrap();
                    let max = entry_sizes.iter().max().unwrap();
                    let sum: usize = entry_sizes.iter().sum();
                    let avg = sum / entry_sizes.len();
                    eprintln!(
                        "Entry size stats: min={min} max={max} avg={avg} total_uncompressed={sum}"
                    );

                    let batch = batcher.take_pending_batch().unwrap();
                    eprintln!(
                        "Batch compressed size: {} bytes ({:.2}% of ONE_MB)",
                        batch.compressed_data.len(),
                        batch.compressed_data.len() as f64 / ONE_MB as f64 * 100.0
                    );
                    eprintln!(
                        "Batch rows: {}, flushes: {}",
                        batch.row_count, batch.flush_count
                    );
                    eprintln!(
                        "Gap to ONE_MB: {} bytes",
                        ONE_MB.saturating_sub(batch.compressed_data.len())
                    );
                    assert!(
                        batch.compressed_data.len() <= ONE_MB,
                        "Batch {} exceeds 1 MiB",
                        batch.compressed_data.len()
                    );
                    break;
                }
                PushResult::TooLarge => panic!("Unexpected TooLarge"),
            }
        }
    }

    /// Verify structural bytes ('[', ',', ']') are correctly accounted for
    /// by checking that a single entry produces valid JSON with no overflow.
    #[test]
    fn test_structural_bytes_accounting() {
        let mut batcher = GzipBatcher::new(6);
        // Push data that's just under the limit minus structural overhead
        let data = vec![b'a'; TARGET_COMPRESSED_LIMIT - 3]; // -3 for '[', ']', and slack
        match batcher.push(data.into()).unwrap() {
            PushResult::Ok(_) => {}
            other => panic!("Expected Ok, got {:?}", std::mem::discriminant(&other)),
        }
        let _ = batcher.finalize().unwrap();
        let batch = batcher.take_pending_batch().unwrap();
        assert!(
            batch.compressed_data.len() <= ONE_MB,
            "Single large entry batch {} exceeds 1MB",
            batch.compressed_data.len()
        );
    }

    /// Verify the TooLarge check accounts for structural bytes.
    #[test]
    fn test_too_large_includes_structural_overhead() {
        let mut batcher = GzipBatcher::new(1);
        // Exactly TARGET_COMPRESSED_LIMIT - 1: too large because +2 for structural bytes
        let data = vec![b'x'; TARGET_COMPRESSED_LIMIT - 1];
        match batcher.push(data.into()).unwrap() {
            PushResult::TooLarge => {} // Expected: data.len() + 2 > TARGET_COMPRESSED_LIMIT
            _ => panic!("Should be TooLarge"),
        }
    }

    // ==================== Edge Case Tests ====================

    /// Verify JSON validity across flush boundaries: commas must separate
    /// entries even when a sync flush occurs between them.
    #[test]
    fn test_comma_present_after_flush_boundary() {
        let mut batcher = GzipBatcher::new(6);

        // Fill until we trigger at least one flush, then finalize.
        loop {
            match batcher.push(generate_1kb_data().into()).unwrap() {
                PushResult::Ok(_) => continue,
                PushResult::BatchReady(_) => break,
                PushResult::TooLarge => panic!("Should not happen"),
            }
        }

        let batch = batcher.take_pending_batch().unwrap();
        assert!(batch.flush_count > 0, "Test requires at least one flush");

        // Decompress and verify it's a valid JSON array with commas between entries.
        let json = decompress_and_validate(&batch.compressed_data);
        let parsed: Vec<serde_json::Value> =
            serde_json::from_str(&json).expect("Must be valid JSON with commas between entries");
        assert!(parsed.len() > 1, "Must have multiple entries");
    }

    /// Verify batches never exceed the ONE_MB hard limit across all
    /// compression levels.
    #[test]
    fn test_hard_limit_enforced_across_levels() {
        for level in [1u32, 6, 9] {
            let mut batcher = GzipBatcher::new(level);
            loop {
                match batcher.push(generate_1kb_data().into()).unwrap() {
                    PushResult::Ok(_) => continue,
                    PushResult::BatchReady(_) => break,
                    PushResult::TooLarge => panic!("Should not happen"),
                }
            }
            let batch = batcher.take_pending_batch().unwrap();
            assert!(
                batch.compressed_data.len() <= ONE_MB,
                "level {level}: batch {} exceeds hard limit",
                batch.compressed_data.len()
            );
        }
    }

    /// Verify each batch starts with '[' and produces valid JSON across
    /// multiple consecutive batches.
    #[test]
    fn test_is_first_entry_correct_across_batches() {
        let mut batcher = GzipBatcher::new(1);

        // Fill first batch
        loop {
            if let PushResult::BatchReady(_) = batcher.push(generate_1kb_data().into()).unwrap() {
                break;
            }
        }

        // Validate first batch
        let b1 = batcher.take_pending_batch().unwrap();
        let json1 = decompress_and_validate(&b1.compressed_data);
        assert!(json1.starts_with('['));
        assert!(
            serde_json::from_str::<Vec<serde_json::Value>>(&json1).is_ok(),
            "First batch must be valid JSON"
        );

        // The spillover entry already started batch 2. Add more and finalize.
        let _ = batcher.push(Bytes::from_static(b"1")).unwrap();
        let _ = batcher.finalize().unwrap();

        let b2 = batcher.take_pending_batch().unwrap();
        let json2 = decompress_and_validate(&b2.compressed_data);
        assert!(json2.starts_with('['));
        assert!(
            !json2.starts_with("[,"),
            "Second batch must not start with '[,'"
        );
        assert!(
            serde_json::from_str::<Vec<serde_json::Value>>(&json2).is_ok(),
            "Second batch must be valid JSON"
        );
    }
}
