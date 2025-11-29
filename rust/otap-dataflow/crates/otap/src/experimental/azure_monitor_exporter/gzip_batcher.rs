use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;

const ONE_MB: usize = 1024 * 1024; // 1 MB

pub struct GzipBatcher {
    buf: GzEncoder<Vec<u8>>,
    remaining_size: usize,
    current_uncompressed_size: usize,
    total_uncompressed_size: usize,
    record_count: usize,
}

pub enum PushResult {
    Ok,
    TooLarge,
    Full(Vec<u8>),
}

pub enum FlushResult {
    Empty,
    Flush(Vec<u8>),
}

// TODO: actual logging instead of print statements
#[allow(clippy::print_stdout)]
impl GzipBatcher {
    pub fn new() -> Self {
        Self {
            buf: GzEncoder::new(Vec::default(), Compression::default()),
            remaining_size: ONE_MB,
            current_uncompressed_size: 0,
            total_uncompressed_size: 0,
            record_count: 0,
        }
    }

    pub fn push(&mut self, data: &[u8]) -> PushResult {
        // This limits uncompressed data size to a maximum of 1MB
        // Is this a good compromise for code simplicity vs efficiency?
        // This algorithm is still very good up to 100KB per entry, which
        // can be considered quite abnormal for log entries.
        if data.len() > (ONE_MB - 2) {
            return PushResult::TooLarge;
        }

        if self.total_uncompressed_size == 0 {
            self.buf.write_all(b"[").expect("write to memory buffer failed");
            self.total_uncompressed_size += 1;
            self.current_uncompressed_size += 1;
        } else {
            self.buf.write_all(b",").expect("write to memory buffer failed");
            self.total_uncompressed_size += 1;
            self.current_uncompressed_size += 1;
        }

        let next_size = self.current_uncompressed_size + data.len() + 1;

        if next_size > self.remaining_size {
            self.buf.flush().expect("flush to memory buffer failed");
            let compressed_size = self.buf.get_ref().len();

            self.remaining_size = ONE_MB.saturating_sub(compressed_size + 1);
            self.current_uncompressed_size = 0;
        }

        let next_size = self.current_uncompressed_size + data.len() + 1;

        if next_size > self.remaining_size {
            let flush_result = self.flush();
            _ = self.push(data);

            match flush_result {
                FlushResult::Empty => PushResult::Ok,
                FlushResult::Flush(compressed_data) => PushResult::Full(compressed_data),
            }
        } else {
            self.buf.write_all(data).expect("write to memory buffer failed");
            self.current_uncompressed_size += data.len();
            self.total_uncompressed_size += data.len();
            self.record_count += 1;

            PushResult::Ok
        }
    }

    pub fn flush(&mut self) -> FlushResult {
        if self.buf.get_ref().is_empty() {
            return FlushResult::Empty;
        }

        self.buf.write_all(b"]").expect("write to memory buffer failed");

        let old_buf = std::mem::replace(
            &mut self.buf,
            GzEncoder::new(Vec::default(), Compression::default()),
        );

        let compressed_data = old_buf.finish().expect("compression failed");
        let compressed_size = compressed_data.len();
        let uncompressed_size = self.total_uncompressed_size;
        let records = self.record_count;

        // Calculate compression ratio
        let compression_ratio = if uncompressed_size > 0 {
            (compressed_size as f64 / uncompressed_size as f64) * 100.0
        } else {
            0.0
        };

        // Get human-readable timestamp
        let now = std::time::SystemTime::now();
        let datetime = chrono::DateTime::<chrono::Utc>::from(now);
        let timestamp = datetime.format("%Y-%m-%d %H:%M:%S UTC");

        println!(
            "[{}] Flushed batch: {} records, {} bytes -> {} bytes (compression ratio: {:.2}%)",
            timestamp, records, uncompressed_size, compressed_size, compression_ratio
        );

        // Reset state
        self.remaining_size = ONE_MB;
        self.current_uncompressed_size = 0;
        self.total_uncompressed_size = 0;
        self.record_count = 0;

        FlushResult::Flush(compressed_data)
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

        // Keep pushing 1KB chunks until we get a Full result
        loop {
            let data = generate_1kb_data();
            let data_len = data.len();

            match batcher.push(&data) {
                PushResult::Ok => {
                    push_count += 1;
                    total_uncompressed_sent += data_len;
                    // Continue pushing
                }
                PushResult::Full(compressed_data) => {
                    push_count += 1;
                    total_uncompressed_sent += data_len;

                    // Verify we got compressed data back
                    assert!(
                        !compressed_data.is_empty(),
                        "Compressed data should not be empty"
                    );

                    // Verify the compressed data is valid gzip
                    // Try to decompress it
                    use flate2::read::GzDecoder;
                    use std::io::Read;
                    let mut decoder = GzDecoder::new(&compressed_data[..]);
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
                        "Pushed {} 1KB entries before getting Full result",
                        push_count
                    );
                    println!(
                        "Total uncompressed data sent: {} bytes",
                        total_uncompressed_sent
                    );
                    println!("Compressed data size: {} bytes", compressed_data.len());
                    println!(
                        "Compression ratio: {:.2}%",
                        (compressed_data.len() as f64 / total_uncompressed_sent as f64) * 100.0
                    );

                    break;
                }
                PushResult::TooLarge => {
                    panic!("Data size is too large!");
                }
            }

            // Safety check to prevent infinite loop in case of bugs
            if push_count > 2000 {
                panic!("Too many pushes without getting Full result");
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

        // Keep pushing until we get 2 Full results
        while full_count < 2 {
            let data = generate_1kb_data();

            match batcher.push(&data) {
                PushResult::Ok => {
                    total_push_count += 1;
                }
                PushResult::Full(compressed_data) => {
                    total_push_count += 1;
                    full_count += 1;

                    // Verify compressed data is valid
                    assert!(
                        !compressed_data.is_empty(),
                        "Batch {} should not be empty",
                        full_count
                    );

                    // Decompress and validate
                    use flate2::read::GzDecoder;
                    use std::io::Read;
                    let mut decoder = GzDecoder::new(&compressed_data[..]);
                    let mut decompressed = String::new();
                    let _ = decoder
                        .read_to_string(&mut decompressed)
                        .expect(&format!("Batch {} should be valid gzip", full_count));

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

                    batch_sizes.push(compressed_data.len());

                    println!(
                        "Batch {}: compressed size = {} bytes",
                        full_count,
                        compressed_data.len()
                    );
                }
                PushResult::TooLarge => {
                    panic!("Unexpected TooLarge result");
                }
            }

            // Safety limit
            if total_push_count > 4000 {
                panic!("Too many pushes without getting 2 Full results");
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
    }

    #[test]
    fn test_push_single_large_entry() {
        let mut batcher = GzipBatcher::new();

        // Create a very large entry that should trigger Full immediately
        let large_data = vec![b'x'; 2 * 1024 * 1024]; // 2MB

        match batcher.push(&large_data) {
            PushResult::Full(_) => {
                panic!("Large entry should not be accepted, got Full instead");
            }
            PushResult::Ok => {
                panic!("Large entry should trigger Full result");
            }
            PushResult::TooLarge => {
                assert!(true, "Correctly identified too large entry");
            }
        }
    }
}
