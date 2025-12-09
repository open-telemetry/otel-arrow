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

// TODO: actual logging instead of print statements
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

    pub fn has_pending_data(&self) -> bool {
        !self.buf.get_ref().is_empty()
    }

    pub fn push(&mut self, data: &[u8]) -> PushResult {
        if self.pending_batch.is_some() {
            // this is the new batch id that we are currently building
            // the pending batch id is available in the pending_batch field
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
        // let compressed_size = compressed_data.len();
        // let uncompressed_size = self.total_uncompressed_size;
        let row_count = self.row_count;

        // // Calculate compression ratio
        // let compression_ratio = if uncompressed_size > 0 {
        //     (compressed_size as f64 / uncompressed_size as f64) * 100.0
        // } else {
        //     0.0
        // };

        // Get human-readable timestamp
        // let now = std::time::SystemTime::now();
        // let datetime = chrono::DateTime::<chrono::Utc>::from(now);
        // let timestamp = datetime.format("%Y-%m-%d %H:%M:%S UTC");

        // let avg_row_size = uncompressed_size as f64 / row_count;

        // println!(
        //     "[{}] Flushed batch: flush count: {}, {} rows, {} bytes -> {} bytes (compression ratio: {:.2}%), avg row size: {:.2} bytes",
        //     timestamp, self.flush_count, row_count, uncompressed_size, compressed_size, compression_ratio, avg_row_size
        // );

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

    pub fn take_pending_batch(&mut self) -> Option<GzipResult> {
        self.pending_batch.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    fn generate_1kb_data() -> Vec<u8> {
        let mut rng = rand::rng();

        // Randomize the ID and timestamp
        let id = rng.random_range(10000..99999);
        let timestamp = rng.random_range(1600000000..1700000000);
        let levels = ["INFO", "WARN", "ERROR", "DEBUG"];
        let level = levels[rng.random_range(0..levels.len())];

        let base_content = format!(
            r#"{{"id":{},"timestamp":{},"level":"{}","message":""#,
            id, timestamp, level
        );
        let base_len = base_content.len();

        // Calculate padding needed to reach exactly 1024 bytes
        let target_size = 1024;
        let closing = r#""}"#;
        let closing_len = closing.len();

        let padding_needed = target_size - base_len - closing_len;

        // Generate random padding with mixed characters
        let padding: String = (0..padding_needed)
            .map(|_| match rng.random_range(0..3) {
                0 => rng.random_range(b'a'..=b'z') as char,
                1 => rng.random_range(b'A'..=b'Z') as char,
                _ => rng.random_range(b'0'..=b'9') as char,
            })
            .collect();

        let full_content = format!("{}{}{}", base_content, padding, closing);
        let data = full_content.into_bytes();

        assert_eq!(data.len(), 1024, "Generated data should be exactly 1KB");
        data
    }

    #[test]
    fn test_push_until_full() {
        let mut batcher = GzipBatcher::new();
        let mut push_count = 0;
        let mut total_uncompressed_sent = 0;

        // Keep pushing 1KB chunks until we get a BatchReady result
        loop {
            let data = generate_1kb_data();
            let data_len = data.len();

            match batcher.push(&data) {
                PushResult::Ok(_) => {
                    push_count += 1;
                    total_uncompressed_sent += data_len;
                    // Continue pushing
                }
                PushResult::BatchReady(batch_id) => {
                    push_count += 1;
                    total_uncompressed_sent += data_len;

                    // Get the pending batch
                    let gzip_result = batcher
                        .take_pending_batch()
                        .expect("BatchReady should have pending batch");

                    // Verify we got compressed data back
                    assert!(
                        !gzip_result.compressed_data.is_empty(),
                        "Compressed data should not be empty"
                    );

                    // Verify batch_id is set
                    assert!(gzip_result.batch_id > 0, "Batch ID should be set");

                    // Verify the compressed data is valid gzip
                    // Try to decompress it
                    use flate2::read::GzDecoder;
                    use std::io::Read;
                    let mut decoder = GzDecoder::new(&gzip_result.compressed_data[..]);
                    let mut decompressed = String::new();
                    let _ = decoder
                        .read_to_string(&mut decompressed)
                        .expect("Should be valid gzip");

                    // Verify it starts with [ and ends with ]
                    assert!(
                        decompressed.starts_with('['),
                        "Decompressed data should start with ["
                    );
                    assert!(
                        decompressed.ends_with(']'),
                        "Decompressed data should end with ]"
                    );

                    println!(
                        "Pushed {} 1KB entries before getting BatchReady result",
                        push_count
                    );
                    println!(
                        "Total uncompressed data sent: {} bytes",
                        total_uncompressed_sent
                    );
                    println!(
                        "Compressed data size: {} bytes",
                        gzip_result.compressed_data.len()
                    );
                    println!("Batch ID: {}", gzip_result.batch_id);
                    println!("Row count: {}", gzip_result.row_count);
                    println!(
                        "Compression ratio: {:.2}%",
                        (gzip_result.compressed_data.len() as f64 / total_uncompressed_sent as f64)
                            * 100.0
                    );

                    break;
                }
                PushResult::TooLarge => {
                    panic!("Data size is too large!");
                }
            }

            // Safety check to prevent infinite loop in case of bugs
            if push_count > 2000 {
                panic!("Too many pushes without getting BatchReady result");
            }
        }

        // Verify we pushed a reasonable amount of data
        assert!(push_count > 0, "Should have pushed at least one entry");
        assert!(total_uncompressed_sent > 0, "Should have sent some data");
    }

    #[test]
    fn test_push_until_full_twice() {
        let mut batcher = GzipBatcher::new();
        let mut full_count = 0;
        let mut total_push_count = 0;
        let mut batch_sizes = Vec::new();
        let mut batch_ids = Vec::new();

        // Keep pushing until we get 2 BatchReady results
        while full_count < 2 {
            let data = generate_1kb_data();

            match batcher.push(&data) {
                PushResult::Ok(_) => {
                    total_push_count += 1;
                }
                PushResult::BatchReady(batch_id) => {
                    total_push_count += 1;
                    full_count += 1;

                    let gzip_result = batcher
                        .take_pending_batch()
                        .expect("BatchReady should have pending batch");

                    // Verify compressed data is valid
                    assert!(
                        !gzip_result.compressed_data.is_empty(),
                        "Batch {} should not be empty",
                        full_count
                    );

                    // Verify batch ID increments
                    batch_ids.push(gzip_result.batch_id);
                    if batch_ids.len() > 1 {
                        assert!(batch_ids[1] > batch_ids[0], "Batch IDs should increment");
                    }

                    // Verify row count
                    assert!(gzip_result.row_count > 0.0, "Row count should be positive");

                    // Decompress and validate
                    use flate2::read::GzDecoder;
                    use std::io::Read;
                    let mut decoder = GzDecoder::new(&gzip_result.compressed_data[..]);
                    let mut decompressed = String::new();
                    _ = decoder
                        .read_to_string(&mut decompressed)
                        .unwrap_or_else(|_| panic!("Batch {} should be valid gzip", full_count));

                    // Check JSON array structure
                    assert!(
                        decompressed.starts_with('['),
                        "Batch {} should start with [",
                        full_count
                    );
                    assert!(
                        decompressed.ends_with(']'),
                        "Batch {} should end with ]",
                        full_count
                    );

                    batch_sizes.push(gzip_result.compressed_data.len());

                    println!(
                        "Batch {}: compressed size = {} bytes, batch_id = {}, row_count = {}",
                        full_count,
                        gzip_result.compressed_data.len(),
                        gzip_result.batch_id,
                        gzip_result.row_count
                    );
                }
                PushResult::TooLarge => {
                    panic!("Unexpected TooLarge result");
                }
            }

            // Safety limit
            if total_push_count > 4000 {
                panic!("Too many pushes without getting 2 BatchReady results");
            }
        }

        // Verify we got exactly 2 batches
        assert_eq!(full_count, 2, "Should have filled exactly 2 batches");
        assert_eq!(batch_sizes.len(), 2, "Should have 2 batch sizes recorded");

        // Both batches should be reasonably sized (under 1MB)
        for (i, size) in batch_sizes.iter().enumerate() {
            assert!(
                *size <= ONE_MB + 1024,
                "Batch {} size {} should be under limit",
                i + 1,
                size
            );
            assert!(*size > 0, "Batch {} should have non-zero size", i + 1);
        }

        println!(
            "Successfully filled 2 batches with {} total pushes",
            total_push_count
        );
        println!("Batch sizes: {:?}", batch_sizes);
        println!("Batch IDs: {:?}", batch_ids);
    }

    #[test]
    fn test_push_single_large_entry() {
        let mut batcher = GzipBatcher::new();

        // Create a very large entry that should trigger TooLarge
        let large_data = vec![b'x'; 2 * 1024 * 1024]; // 2MB

        match batcher.push(&large_data) {
            PushResult::BatchReady(batch_id) => {
                panic!("Large entry should not be accepted, got BatchReady instead");
            }
            PushResult::Ok(_) => {
                panic!("Large entry should trigger TooLarge result");
            }
            PushResult::TooLarge => {
                // Expected result - large entry correctly identified
            }
        }
    }

    #[test]
    fn test_bytes_cloning_is_cheap() {
        let mut batcher = GzipBatcher::new();

        // Push until we get a full batch
        loop {
            let data = generate_1kb_data();

            match batcher.push(&data) {
                PushResult::Ok(_) => continue,
                PushResult::BatchReady(batch_id) => {
                    let gzip_result = batcher.take_pending_batch().unwrap();

                    // Test that cloning Bytes is cheap
                    let start = std::time::Instant::now();
                    let _clone1 = gzip_result.compressed_data.clone();
                    let _clone2 = gzip_result.compressed_data.clone();
                    let _clone3 = gzip_result.compressed_data.clone();
                    let elapsed = start.elapsed();

                    // Cloning should be very fast (microseconds at most)
                    assert!(
                        elapsed.as_micros() < 100,
                        "Cloning Bytes should be very fast"
                    );

                    // All clones point to the same data
                    assert_eq!(gzip_result.compressed_data.len(), _clone1.len());
                    assert_eq!(gzip_result.compressed_data.len(), _clone2.len());
                    assert_eq!(gzip_result.compressed_data.len(), _clone3.len());

                    // Verify the GzipResult fields are present
                    assert!(gzip_result.batch_id > 0, "Batch ID should be set");
                    assert!(gzip_result.row_count > 0.0, "Row count should be positive");

                    break;
                }
                PushResult::TooLarge => panic!("Unexpected TooLarge"),
            }
        }
    }

    #[test]
    fn test_flush_empty() {
        let mut batcher = GzipBatcher::new();

        // Flush without pushing anything
        match batcher.flush() {
            FlushResult::Empty => {
                // Expected result
            }
            FlushResult::Flush => {
                panic!("Flush on empty batcher should return Empty");
            }
        }
    }

    #[test]
    fn test_flush_with_data() {
        let mut batcher = GzipBatcher::new();

        // Push some data but not enough to trigger BatchReady
        for _ in 0..5 {
            let data = generate_1kb_data();
            match batcher.push(&data) {
                PushResult::Ok(_) => continue,
                PushResult::BatchReady(batch_id) => panic!("Should not be full after 5 pushes"),
                PushResult::TooLarge => panic!("Unexpected TooLarge"),
            }
        }

        // Now flush
        match batcher.flush() {
            FlushResult::Empty => {
                panic!("Flush with data should return Flush variant");
            }
            FlushResult::Flush => {
                let gzip_result = batcher
                    .take_pending_batch()
                    .expect("Flush should have pending batch");

                // Verify we got valid compressed data
                assert!(!gzip_result.compressed_data.is_empty(), "Should have data");
                assert!(gzip_result.batch_id > 0, "Batch ID should be set");
                assert_eq!(gzip_result.row_count, 5.0, "Should have 5 rows");

                // Decompress and verify
                use flate2::read::GzDecoder;
                use std::io::Read;
                let mut decoder = GzDecoder::new(&gzip_result.compressed_data[..]);
                let mut decompressed = String::new();
                _ = decoder
                    .read_to_string(&mut decompressed)
                    .expect("Should decompress");

                assert!(decompressed.starts_with('['), "Should start with [");
                assert!(decompressed.ends_with(']'), "Should end with ]");
            }
        }
    }
}
