// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use bytes::Bytes;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;

const ONE_MB: usize = 1024 * 1024; // 1 MB
const MAX_GZIP_FLUSH_COUNT: usize = 100;

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

pub enum FlushResult {
    Empty,
    Flush,
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
            remaining_size: ONE_MB,
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
        if data.len() > (ONE_MB - 2) {
            return PushResult::TooLarge;
        }

        if self.total_uncompressed_size == 0 {
            self.batch_id += 1;
            self.buf
                .write_all(b"[")
                .expect("write to memory buffer failed");
            self.total_uncompressed_size += 1;
            self.uncompressed_size += 1;
        } else {
            self.buf
                .write_all(b",")
                .expect("write to memory buffer failed");
            self.total_uncompressed_size += 1;
            self.uncompressed_size += 1;
        }

        let next_size = self.uncompressed_size + data.len() + 1;

        if next_size > self.remaining_size {
            self.buf.flush().expect("flush to memory buffer failed");

            self.flush_count += 1;
            let compressed_size = self.buf.get_ref().len();

            self.remaining_size = ONE_MB.saturating_sub(compressed_size + 1);
            self.uncompressed_size = 0;
        }

        let next_size = self.uncompressed_size + data.len() + 1;

        if next_size > self.remaining_size || self.flush_count >= MAX_GZIP_FLUSH_COUNT {
            let flush_result = self.flush();
            _ = self.push_internal(data);

            match flush_result {
                FlushResult::Empty => PushResult::Ok(self.batch_id),
                FlushResult::Flush => {
                    // this is the new batch id that we are currently building
                    // the pending batch id is available in the pending_batch field
                    PushResult::BatchReady(self.batch_id)
                }
            }
        } else {
            self.buf
                .write_all(data)
                .expect("write to memory buffer failed");
            self.uncompressed_size += data.len();
            self.total_uncompressed_size += data.len();
            self.row_count += 1.0;

            PushResult::Ok(self.batch_id)
        }
    }

    pub fn flush(&mut self) -> FlushResult {
        if self.buf.get_ref().is_empty() {
            return FlushResult::Empty;
        }

        self.buf
            .write_all(b"]")
            .expect("write to memory buffer failed");

        let old_buf = std::mem::replace(&mut self.buf, Self::new_encoder());

        let compressed_data = old_buf.finish().expect("compression failed");
        let row_count = self.row_count;

        // Reset state
        self.remaining_size = ONE_MB;
        self.uncompressed_size = 0;
        self.total_uncompressed_size = 0;
        self.row_count = 0.0;
        self.flush_count = 0;

        self.pending_batch = Some(GzipResult {
            batch_id: self.batch_id,
            compressed_data: Bytes::from(compressed_data),
            row_count,
        });

        // Convert Vec<u8> to Bytes (zero-copy)
        FlushResult::Flush
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

        assert!(decompressed.starts_with('['), "Should start with [");
        assert!(decompressed.ends_with(']'), "Should end with ]");

        decompressed
    }

    // ==================== Construction Tests ====================

    #[test]
    fn test_new_creates_empty_batcher() {
        let batcher = GzipBatcher::new();

        assert!(!batcher.has_pending_data());
        assert!(batcher.pending_batch.is_none());
        assert_eq!(batcher.batch_id, 0);
        assert_eq!(batcher.row_count, 0.0);
    }

    // ==================== has_pending_data Tests ====================

    #[test]
    fn test_has_pending_data_empty() {
        let batcher = GzipBatcher::new();
        assert!(!batcher.has_pending_data());
    }

    #[test]
    fn test_has_pending_data_after_push() {
        let mut batcher = GzipBatcher::new();
        let data = generate_1kb_data();

        let _ = batcher.push(&data);

        assert!(batcher.has_pending_data());
    }

    #[test]
    fn test_has_pending_data_after_flush() {
        let mut batcher = GzipBatcher::new();
        let data = generate_1kb_data();

        let _ = batcher.push(&data);
        let _ = batcher.flush();

        // After flush, the buffer is reset
        assert!(!batcher.has_pending_data());
    }

    // ==================== take_pending_batch Tests ====================

    #[test]
    fn test_take_pending_batch_when_empty() {
        let mut batcher = GzipBatcher::new();

        assert!(batcher.take_pending_batch().is_none());
    }

    #[test]
    fn test_take_pending_batch_after_flush() {
        let mut batcher = GzipBatcher::new();
        let _ = batcher.push(&generate_1kb_data());
        let _ = batcher.flush();

        let batch = batcher.take_pending_batch();
        assert!(batch.is_some());

        // Second take should return None
        assert!(batcher.take_pending_batch().is_none());
    }

    #[test]
    fn test_take_pending_batch_clears_pending() {
        let mut batcher = GzipBatcher::new();
        let _ = batcher.push(&generate_1kb_data());
        let _ = batcher.flush();

        let _ = batcher.take_pending_batch();

        // Can push new data after taking
        match batcher.push(&generate_1kb_data()) {
            PushResult::Ok(_) => {} // Expected
            other => panic!("Expected Ok, got {:?}", std::mem::discriminant(&other)),
        }
    }

    // ==================== Push Tests ====================

    #[test]
    fn test_push_single_entry() {
        let mut batcher = GzipBatcher::new();
        let data = generate_1kb_data();

        match batcher.push(&data) {
            PushResult::Ok(batch_id) => {
                assert_eq!(batch_id, 1, "First push should be batch 1");
            }
            _ => panic!("Single 1KB push should return Ok"),
        }
    }

    #[test]
    fn test_push_too_large_entry() {
        let mut batcher = GzipBatcher::new();
        let large_data = vec![b'x'; ONE_MB]; // Exactly 1MB - too large

        match batcher.push(&large_data) {
            PushResult::TooLarge => {} // Expected
            _ => panic!("1MB entry should return TooLarge"),
        }
    }

    #[test]
    fn test_push_just_under_limit() {
        let mut batcher = GzipBatcher::new();
        // ONE_MB - 2 is the max allowed (for '[' and ']')
        let data = vec![b'x'; ONE_MB - 3];

        match batcher.push(&data) {
            PushResult::Ok(_) => {} // Expected - just under limit
            PushResult::TooLarge => panic!("Data just under limit should be accepted"),
            PushResult::BatchReady(_) => {} // Also acceptable if it triggers a batch
        }
    }

    #[test]
    fn test_push_returns_batch_ready_when_pending_exists() {
        let mut batcher = GzipBatcher::new();

        // Push until batch is ready
        loop {
            match batcher.push(&generate_1kb_data()) {
                PushResult::Ok(_) => continue,
                PushResult::BatchReady(_) => break,
                PushResult::TooLarge => panic!("Unexpected TooLarge"),
            }
        }

        // Don't take the pending batch, try to push multiple times
        // All should return BatchReady with the same batch_id
        let expected_batch_id = batcher.batch_id;
        for _ in 0..10 {
            match batcher.push(&generate_1kb_data()) {
                PushResult::BatchReady(id) => {
                    assert_eq!(id, expected_batch_id, "Should return same batch_id");
                }
                _ => panic!("Push with pending batch should return BatchReady"),
            }
        }

        // Verify the pending batch is still there and unchanged
        let batch = batcher.take_pending_batch().unwrap();
        assert!(batch.row_count > 0.0);

        // Now we can push again
        match batcher.push(&generate_1kb_data()) {
            PushResult::Ok(_) => {} // Expected
            _ => panic!("Should be able to push after taking pending batch"),
        }
    }

    #[test]
    fn test_push_batch_id_increments() {
        let mut batcher = GzipBatcher::new();
        let mut batch_ids = Vec::new();

        // Collect 3 batch IDs
        for _ in 0..3 {
            loop {
                match batcher.push(&generate_1kb_data()) {
                    PushResult::Ok(_) => continue,
                    PushResult::BatchReady(batch_id) => {
                        let result = batcher.take_pending_batch().unwrap();
                        batch_ids.push(result.batch_id);
                        assert_eq!(
                            batch_id,
                            result.batch_id + 1,
                            "Batch ID should increment when a batch is ready"
                        );
                        break;
                    }
                    PushResult::TooLarge => panic!("Unexpected"),
                }
            }
        }

        assert_eq!(batch_ids, vec![1, 2, 3], "Batch IDs should increment");
    }

    // ==================== Flush Tests ====================

    #[test]
    fn test_flush_empty_batcher() {
        let mut batcher = GzipBatcher::new();

        match batcher.flush() {
            FlushResult::Empty => {} // Expected
            FlushResult::Flush => panic!("Empty batcher should return Empty"),
        }
    }

    #[test]
    fn test_flush_with_data() {
        let mut batcher = GzipBatcher::new();

        for _ in 0..5 {
            let _ = batcher.push(&generate_1kb_data());
        }

        match batcher.flush() {
            FlushResult::Flush => {
                let batch = batcher.take_pending_batch().unwrap();
                assert_eq!(batch.row_count, 5.0);
                assert!(batch.batch_id > 0);
                _ = decompress_and_validate(&batch.compressed_data);
            }
            FlushResult::Empty => panic!("Batcher with data should return Flush"),
        }
    }

    #[test]
    fn test_flush_resets_state() {
        let mut batcher = GzipBatcher::new();

        for _ in 0..5 {
            let _ = batcher.push(&generate_1kb_data());
        }

        let _ = batcher.flush();
        let _ = batcher.take_pending_batch();

        // State should be reset
        assert!(!batcher.has_pending_data());
        assert_eq!(batcher.row_count, 0.0);
        assert_eq!(batcher.flush_count, 0);
    }

    #[test]
    fn test_flush_multiple_times() {
        let mut batcher = GzipBatcher::new();

        // First batch
        for _ in 0..3 {
            let _ = batcher.push(&generate_1kb_data());
        }
        let _ = batcher.flush();
        let batch1 = batcher.take_pending_batch().unwrap();

        // Second batch
        for _ in 0..7 {
            let _ = batcher.push(&generate_1kb_data());
        }
        let _ = batcher.flush();
        let batch2 = batcher.take_pending_batch().unwrap();

        assert_eq!(batch1.row_count, 3.0);
        assert_eq!(batch2.row_count, 7.0);
        assert!(batch2.batch_id > batch1.batch_id);
    }

    // ==================== Compression Tests ====================

    #[test]
    fn test_output_is_valid_gzip() {
        let mut batcher = GzipBatcher::new();

        for _ in 0..10 {
            let _ = batcher.push(&generate_1kb_data());
        }
        let _ = batcher.flush();
        let batch = batcher.take_pending_batch().unwrap();

        // Decompress and verify JSON structure
        let decompressed = decompress_and_validate(&batch.compressed_data);

        // Should be valid JSON array
        let parsed: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&decompressed);
        assert!(parsed.is_ok(), "Should be valid JSON array");
        assert_eq!(parsed.unwrap().len(), 10, "Should have 10 entries");
    }

    #[test]
    fn test_compression_ratio() {
        let mut batcher = GzipBatcher::new();
        let mut uncompressed_size = 0;

        loop {
            let data = generate_1kb_data();
            uncompressed_size += data.len();

            match batcher.push(&data) {
                PushResult::Ok(_) => continue,
                PushResult::BatchReady(_) => {
                    let batch = batcher.take_pending_batch().unwrap();
                    let ratio = batch.compressed_data.len() as f64 / uncompressed_size as f64;

                    // Random data typically compresses to ~50-80%
                    assert!(ratio < 1.0, "Compressed should be smaller than original");
                    println!("Compression ratio: {:.2}%", ratio * 100.0);
                    break;
                }
                PushResult::TooLarge => panic!("Unexpected"),
            }
        }
    }

    // ==================== Full Batch Tests ====================

    #[test]
    fn test_push_until_full() {
        let mut batcher = GzipBatcher::new();
        let mut push_count = 0;

        loop {
            match batcher.push(&generate_1kb_data()) {
                PushResult::Ok(_) => push_count += 1,
                PushResult::BatchReady(_) => {
                    push_count += 1;
                    let batch = batcher.take_pending_batch().unwrap();

                    assert!(!batch.compressed_data.is_empty());
                    assert!(batch.compressed_data.len() <= ONE_MB + 1024);
                    assert!(batch.row_count > 0.0);

                    _ = decompress_and_validate(&batch.compressed_data);
                    break;
                }
                PushResult::TooLarge => panic!("Unexpected"),
            }

            assert!(push_count < 2000, "Safety limit exceeded");
        }

        println!("Pushed {} entries before batch was full", push_count);
    }

    #[test]
    fn test_push_until_full_twice() {
        let mut batcher = GzipBatcher::new();
        let mut batches = Vec::new();

        while batches.len() < 2 {
            match batcher.push(&generate_1kb_data()) {
                PushResult::Ok(_) => {}
                PushResult::BatchReady(_) => {
                    let batch = batcher.take_pending_batch().unwrap();
                    batches.push(batch);
                }
                PushResult::TooLarge => panic!("Unexpected"),
            }
        }

        assert_eq!(batches.len(), 2);
        assert!(batches[1].batch_id > batches[0].batch_id);
    }

    // ==================== Row Count Tests ====================

    #[test]
    fn test_row_count_accuracy() {
        let mut batcher = GzipBatcher::new();

        for _ in 0..42 {
            let _ = batcher.push(&generate_1kb_data());
        }

        let _ = batcher.flush();
        let batch = batcher.take_pending_batch().unwrap();

        assert_eq!(batch.row_count, 42.0);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_empty_data_push() {
        let mut batcher = GzipBatcher::new();

        // Empty data should still work (creates empty JSON object in array)
        match batcher.push(&[]) {
            PushResult::Ok(batch_id) => assert_eq!(batch_id, 1),
            _ => panic!("Empty push should return Ok"),
        }

        assert_eq!(batcher.row_count, 1.0);
    }

    #[test]
    fn test_bytes_clone_is_cheap() {
        let mut batcher = GzipBatcher::new();

        loop {
            match batcher.push(&generate_1kb_data()) {
                PushResult::Ok(_) => continue,
                PushResult::BatchReady(_) => {
                    let batch = batcher.take_pending_batch().unwrap();

                    let start = std::time::Instant::now();
                    for _ in 0..1000 {
                        let _ = batch.compressed_data.clone();
                    }
                    let elapsed = start.elapsed();

                    // 1000 clones should take < 1ms (Bytes uses Arc internally)
                    assert!(elapsed.as_millis() < 10, "Bytes clone should be O(1)");
                    break;
                }
                PushResult::TooLarge => panic!("Unexpected"),
            }
        }
    }

    #[test]
    fn test_interleaved_push_and_take() {
        let mut batcher = GzipBatcher::new();

        // Push some data
        for _ in 0..5 {
            let _ = batcher.push(&generate_1kb_data());
        }

        // Flush and take
        let _ = batcher.flush();
        let batch1 = batcher.take_pending_batch().unwrap();

        // Push more data
        for _ in 0..3 {
            let _ = batcher.push(&generate_1kb_data());
        }

        // Flush and take again
        let _ = batcher.flush();
        let batch2 = batcher.take_pending_batch().unwrap();

        assert_eq!(batch1.row_count, 5.0);
        assert_eq!(batch2.row_count, 3.0);
        assert_eq!(batch2.batch_id, batch1.batch_id + 1);
    }
}
