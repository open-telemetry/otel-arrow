// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use bytes::Bytes;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;

const ONE_MB: usize = 1024 * 1024; // 1 MB
const MAX_GZIP_FLUSH_COUNT: usize = 100;
const GZIP_SAFETY_MARGIN: usize = 30; // Safety margin in bytes

pub struct GzipBatcher {
    buf: GzEncoder<Vec<u8>>,
    remaining_size: usize,
    uncompressed_size: usize,
    total_uncompressed_size: usize,
    row_count: f64,
    flush_count: usize,
    batch_id: u64,
    pending_batch: Option<GzipResult>,
}

pub enum PushResult {
    Ok(u64),
    TooLarge,
    BatchReady(u64),
}

pub enum FinalizeResult {
    Empty,
    Ok,
}

pub struct GzipResult {
    pub batch_id: u64,
    pub compressed_data: Bytes,
    pub row_count: f64,
}

// TODO: Remove print_stdout after logging is set up
#[allow(clippy::print_stdout)]
impl GzipBatcher {
    pub fn new() -> Self {
        Self {
            buf: Self::new_encoder(),
            // Use the constant here
            remaining_size: ONE_MB - GZIP_SAFETY_MARGIN,
            uncompressed_size: 0,
            total_uncompressed_size: 0,
            row_count: 0.0,
            flush_count: 0,
            batch_id: 0,
            pending_batch: None,
        }
    }

    fn new_encoder() -> GzEncoder<Vec<u8>> {
        GzEncoder::new(Vec::with_capacity(ONE_MB), Compression::default())
    }

    #[inline]
    pub fn has_pending_data(&self) -> bool {
        !self.buf.get_ref().is_empty()
    }

    #[inline]
    pub fn push(&mut self, data: &[u8]) -> PushResult {
        if self.pending_batch.is_some() {
            return PushResult::BatchReady(self.batch_id);
        }

        self.push_internal(data)
    }

    fn push_internal(&mut self, data: &[u8]) -> PushResult {
        // This limits uncompressed data size to a maximum of 1MB
        // Is this a good compromise for code simplicity vs efficiency?
        // This algorithm is still very good up to 100KB per entry, which
        // can be considered quite abnormal for log entries.
        if data.len() > (ONE_MB - GZIP_SAFETY_MARGIN) {
            return PushResult::TooLarge;
        }

        let is_first_entry = self.uncompressed_size == 0;

        if is_first_entry {
            self.batch_id += 1;
            self.buf
                .write_all(b"[")
                .expect("write to memory buffer failed");
        }

        // Update calculation to use the constant
        let next_size = self.uncompressed_size + data.len();
        let must_flush = next_size > self.remaining_size;

        if must_flush {
            self.buf.flush().expect("flush to memory buffer failed");

            self.flush_count += 1;
            let compressed_size = self.buf.get_ref().len();

            // Use the constant here
            self.remaining_size = ONE_MB.saturating_sub(compressed_size + GZIP_SAFETY_MARGIN);
            self.uncompressed_size = 0;
        }

        let next_size = self.uncompressed_size + data.len();
        let must_finalize =
            next_size > self.remaining_size || self.flush_count >= MAX_GZIP_FLUSH_COUNT;

        if must_finalize {
            let flush_result = self.finalize();
            _ = self.push_internal(data);

            match flush_result {
                FinalizeResult::Empty => PushResult::Ok(self.batch_id),
                FinalizeResult::Ok => {
                    // this is the new batch id that we are currently building
                    // the pending batch id is available in the pending_batch field
                    PushResult::BatchReady(self.batch_id)
                }
            }
        } else {
            if !is_first_entry {
                self.buf
                    .write_all(b",")
                    .expect("write to memory buffer failed");
                self.total_uncompressed_size += 1;
                self.uncompressed_size += 1;
            }
            self.buf
                .write_all(data)
                .expect("write to memory buffer failed");
            self.uncompressed_size += data.len();
            self.total_uncompressed_size += data.len();
            self.row_count += 1.0;

            PushResult::Ok(self.batch_id)
        }
    }

    pub fn finalize(&mut self) -> FinalizeResult {
        if self.buf.get_ref().is_empty() {
            return FinalizeResult::Empty;
        }

        self.buf
            .write_all(b"]")
            .expect("write to memory buffer failed");

        let old_buf = std::mem::replace(&mut self.buf, Self::new_encoder());

        let compressed_data = old_buf.finish().expect("compression failed");
        let row_count = self.row_count;

        // Reset state
        self.remaining_size = ONE_MB - GZIP_SAFETY_MARGIN;
        self.uncompressed_size = 0;
        self.total_uncompressed_size = 0;
        self.row_count = 0.0;
        self.flush_count = 0;

        self.pending_batch = Some(GzipResult {
            batch_id: self.batch_id,
            compressed_data: Bytes::from(compressed_data),
            row_count,
        });

        FinalizeResult::Ok
    }

    #[inline]
    pub fn take_pending_batch(&mut self) -> Option<GzipResult> {
        self.pending_batch.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::GzDecoder;
    use rand::Rng;
    use std::io::Read;

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
        let batcher = GzipBatcher::new();
        assert!(!batcher.has_pending_data());
        assert!(batcher.pending_batch.is_none());
    }

    #[test]
    fn test_has_pending_data_lifecycle() {
        let mut batcher = GzipBatcher::new();
        assert!(!batcher.has_pending_data());

        let _ = batcher.push(&generate_1kb_data());
        assert!(batcher.has_pending_data());

        let _ = batcher.finalize();
        assert!(!batcher.has_pending_data());
    }

    #[test]
    fn test_take_pending_batch_lifecycle() {
        let mut batcher = GzipBatcher::new();
        assert!(batcher.take_pending_batch().is_none());

        let _ = batcher.push(&generate_1kb_data());
        let _ = batcher.finalize();

        let batch = batcher.take_pending_batch();
        assert!(batch.is_some());
        assert!(batcher.take_pending_batch().is_none());
    }

    // ==================== Push Logic Tests ====================

    #[test]
    fn test_push_single_entry() {
        let mut batcher = GzipBatcher::new();
        match batcher.push(&generate_1kb_data()) {
            PushResult::Ok(id) => assert_eq!(id, 1),
            _ => panic!("Should be Ok"),
        }
    }

    #[test]
    fn test_push_too_large_entry() {
        let mut batcher = GzipBatcher::new();
        let large_data = vec![b'x'; ONE_MB];
        match batcher.push(&large_data) {
            PushResult::TooLarge => {} // Expected
            _ => panic!("Should be TooLarge"),
        }
    }

    #[test]
    fn test_push_just_under_limit() {
        let mut batcher = GzipBatcher::new();
        let data = vec![b'x'; ONE_MB - GZIP_SAFETY_MARGIN]; // Max allowed (minus safety margin overhead)
        match batcher.push(&data) {
            PushResult::Ok(_) | PushResult::BatchReady(_) => {} // Expected
            PushResult::TooLarge => panic!("Should fit"),
        }
    }

    #[test]
    fn test_push_returns_batch_ready_when_pending_exists() {
        let mut batcher = GzipBatcher::new();

        // Force a pending batch
        loop {
            if let PushResult::BatchReady(_) = batcher.push(&generate_1kb_data()) {
                break;
            }
        }

        // Subsequent pushes should return BatchReady
        match batcher.push(&generate_1kb_data()) {
            PushResult::BatchReady(_) => {}
            _ => panic!("Should return BatchReady"),
        }
    }

    #[test]
    fn test_push_batch_id_increments() {
        let mut batcher = GzipBatcher::new();
        let mut last_id = 0;

        for _ in 0..3 {
            loop {
                match batcher.push(&generate_1kb_data()) {
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
        let mut batcher = GzipBatcher::new();
        match batcher.finalize() {
            FinalizeResult::Empty => {}
            _ => panic!("Should be Empty"),
        }
    }

    #[test]
    fn test_flush_with_data() {
        let mut batcher = GzipBatcher::new();
        let _ = batcher.push(&generate_1kb_data());

        match batcher.finalize() {
            FinalizeResult::Ok => {
                let batch = batcher.take_pending_batch().unwrap();
                assert!(batch.row_count > 0.0);
                assert!(!batch.compressed_data.is_empty());
            }
            _ => panic!("Should be Ok"),
        }
    }

    #[test]
    fn test_flush_multiple_times() {
        let mut batcher = GzipBatcher::new();

        // Batch 1
        let _ = batcher.push(&generate_1kb_data());
        let _ = batcher.finalize();
        let b1 = batcher.take_pending_batch().unwrap();

        // Batch 2
        let _ = batcher.push(&generate_1kb_data());
        let _ = batcher.finalize();
        let b2 = batcher.take_pending_batch().unwrap();

        assert!(b2.batch_id > b1.batch_id);
    }

    // ==================== Integration & Format Tests ====================

    #[test]
    fn test_output_is_valid_gzip_json_array() {
        let mut batcher = GzipBatcher::new();
        for _ in 0..10 {
            let _ = batcher.push(&generate_1kb_data());
        }
        let _ = batcher.finalize();

        let batch = batcher.take_pending_batch().unwrap();
        let decompressed = decompress_and_validate(&batch.compressed_data);

        let parsed: Vec<serde_json::Value> = serde_json::from_str(&decompressed).unwrap();
        assert_eq!(parsed.len(), 10);
    }

    #[test]
    fn test_row_count_accuracy() {
        let mut batcher = GzipBatcher::new();
        for _ in 0..42 {
            let _ = batcher.push(&generate_1kb_data());
        }
        let _ = batcher.finalize();
        assert_eq!(batcher.take_pending_batch().unwrap().row_count, 42.0);
    }

    #[test]
    fn test_interleaved_push_and_take() {
        let mut batcher = GzipBatcher::new();

        let _ = batcher.push(&generate_1kb_data());
        let _ = batcher.finalize();
        let _ = batcher.take_pending_batch();

        let _ = batcher.push(&generate_1kb_data());
        let _ = batcher.finalize();
        let b2 = batcher.take_pending_batch().unwrap();

        assert_eq!(b2.row_count, 1.0);
    }

    // ==================== Comma Handling Regression Tests ====================

    #[test]
    fn test_no_leading_comma_after_bracket() {
        let mut batcher = GzipBatcher::new();
        let _ = batcher.push(b"1");
        let _ = batcher.push(b"2");
        let _ = batcher.finalize();

        let json = decompress_and_validate(&batcher.take_pending_batch().unwrap().compressed_data);
        assert_eq!(json, "[1,2]");
    }

    #[test]
    fn test_no_trailing_comma_before_bracket() {
        let mut batcher = GzipBatcher::new();
        let _ = batcher.push(b"1");
        let _ = batcher.finalize();

        let json = decompress_and_validate(&batcher.take_pending_batch().unwrap().compressed_data);
        assert_eq!(json, "[1]");
    }

    #[test]
    fn test_format_valid_after_auto_finalize() {
        let mut batcher = GzipBatcher::new();

        // Fill until split
        loop {
            if let PushResult::BatchReady(_) = batcher.push(&generate_1kb_data()) {
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
        let mut batcher = GzipBatcher::new();

        // Fill first batch and discard
        loop {
            if let PushResult::BatchReady(_) = batcher.push(&generate_1kb_data()) {
                break;
            }
        }
        let _ = batcher.take_pending_batch();

        // Second batch
        // Note: This batch will start with the "spillover" entry that triggered the previous BatchReady.
        // We append more data to it.
        let _ = batcher.push(b"1");
        let _ = batcher.push(b"2");
        let _ = batcher.finalize();

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

    #[test]
    fn test_exact_1mb_limit_enforcement() {
        let mut batcher = GzipBatcher::new();
        let mut rng = rand::rng();

        // We use uncompressible data (random bytes) to ensure the compressed size
        // grows predictably and we can hit the limit accurately.
        // We use small chunks (10 bytes) to fill the remaining space granularly.
        loop {
            let chunk: Vec<u8> = (0..10).map(|_| rng.random()).collect();
            match batcher.push(&chunk) {
                PushResult::Ok(_) => continue,
                PushResult::BatchReady(_) => break,
                PushResult::TooLarge => panic!("Should not happen with small chunks"),
            }
        }

        let batch = batcher.take_pending_batch().unwrap();
        let size = batch.compressed_data.len();

        println!("Final batch size: {} bytes", size);

        // 1. Must not exceed 1MB
        assert!(size <= ONE_MB, "Batch size {} exceeds 1MB limit", size);

        // 2. Must be close to the limit minus safety margin.
        // Since we have a 100 byte safety margin, the batch will stop filling
        // when it hits roughly (1MB - 100).
        // We allow another 100 bytes of "slop" for the granularity of the last chunk/flush.
        let expected_min = ONE_MB - GZIP_SAFETY_MARGIN - 100;
        assert!(
            size >= expected_min,
            "Batch size {} is below expected minimum {}",
            size,
            expected_min
        );
    }
}
