// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::ops::Add;

/// A delta decoder that accumulates values for the same key and resets when the key changes.
///
/// This decoder is used in OpenTelemetry Arrow format processing where delta-encoded values
/// need to be accumulated within the same logical group (identified by a key) but reset
/// when moving to a different group.
///
/// # Type Parameters
///
/// * `K` - The key type that must implement `Eq` and `Clone` for comparison and storage
/// * `V` - The value type that must implement `Add` and `Copy` for accumulation
///
/// # Examples
///
/// ```
/// use otel_arrow_rust::otlp::traces::delta_decoder::DeltaDecoder;
///
/// let mut decoder = DeltaDecoder::new();
///
/// // First value for key "span1"
/// assert_eq!(decoder.decode(&"span1".to_string(), 10), 10);
///
/// // Second value for same key - accumulates
/// assert_eq!(decoder.decode(&"span1".to_string(), 5), 15);
///
/// // Different key - resets accumulation
/// assert_eq!(decoder.decode(&"span2".to_string(), 20), 20);
/// ```
#[derive(Default)]
pub struct DeltaDecoder<K, V> {
    /// Stores the previous key-value pair for comparison and accumulation
    prev_key_value: Option<(K, V)>,
}

impl<K, V> DeltaDecoder<K, V>
where
    K: Eq + Clone,
    V: Add<Output = V> + Copy,
{
    /// Creates a new `DeltaDecoder` instance.
    pub fn new() -> Self {
        Self {
            prev_key_value: None,
        }
    }

    /// Decodes a delta value by accumulating it with previous values for the same key.
    ///
    /// This method implements the core delta decoding logic:
    /// - If this is the first call or the key is different from the previous key,
    ///   the delta value is returned as-is and becomes the new baseline
    /// - If the key matches the previous key, the delta value is added to the
    ///   accumulated value and the new total is returned
    ///
    /// # Parameters
    ///
    /// * `key` - A reference to the key that identifies the logical group
    /// * `delta_value` - The delta value to be decoded/accumulated
    ///
    /// # Returns
    ///
    /// The accumulated value for the given key. For the first occurrence of a key,
    /// this will be the delta value itself. For subsequent occurrences of the same
    /// key, this will be the sum of all delta values seen for that key.
    ///
    /// # Examples
    ///
    /// ```
    /// use otel_arrow_rust::otlp::traces::delta_decoder::DeltaDecoder;
    ///
    /// let mut decoder = DeltaDecoder::new();
    ///
    /// // First decode - returns delta value as baseline
    /// assert_eq!(decoder.decode(&"key1", 100), 100);
    ///
    /// // Same key - accumulates delta
    /// assert_eq!(decoder.decode(&"key1", 50), 150);
    ///
    /// // Different key - resets and starts new accumulation
    /// assert_eq!(decoder.decode(&"key2", 200), 200);
    ///
    /// // Back to first key - resets (doesn't remember previous accumulation)
    /// assert_eq!(decoder.decode(&"key1", 75), 75);
    /// ```
    pub fn decode(&mut self, key: &K, delta_value: V) -> V {
        match &mut self.prev_key_value {
            None => {
                // First decode - store the key and return the initial value
                self.prev_key_value = Some((key.clone(), delta_value));
                delta_value
            }
            Some((prev_key, prev_accumulated_value)) => {
                if prev_key == key {
                    // Same key - accumulate the delta with previous value, no need to clone
                    *prev_accumulated_value = *prev_accumulated_value + delta_value;
                    *prev_accumulated_value
                } else {
                    // Different key - reset accumulation and start fresh
                    *prev_key = key.clone();
                    *prev_accumulated_value = delta_value;
                    delta_value
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_decoder_new() {
        let decoder: DeltaDecoder<String, u32> = DeltaDecoder::new();
        assert!(decoder.prev_key_value.is_none());
    }

    #[test]
    fn test_delta_decoder_default() {
        let decoder: DeltaDecoder<String, u32> = DeltaDecoder::default();
        assert!(decoder.prev_key_value.is_none());
    }

    #[test]
    fn test_delta_decoder_first_decode() {
        let mut decoder = DeltaDecoder::new();
        let key = "test_key".to_string();
        let value = 10u32;

        let result = decoder.decode(&key, value);

        assert_eq!(result, 10);
        assert!(decoder.prev_key_value.is_some());
        let (stored_key, stored_value) = decoder.prev_key_value.as_ref().unwrap();
        assert_eq!(stored_key, &key);
        assert_eq!(*stored_value, 10);
    }

    #[test]
    fn test_delta_decoder_same_key_accumulation() {
        let mut decoder = DeltaDecoder::new();
        let key = "test_key".to_string();

        // First decode
        let result1 = decoder.decode(&key, 10u32);
        assert_eq!(result1, 10);

        // Second decode with same key - should accumulate
        let result2 = decoder.decode(&key, 5u32);
        assert_eq!(result2, 15);

        // Third decode with same key - should continue accumulating
        let result3 = decoder.decode(&key, 3u32);
        assert_eq!(result3, 18);

        // Verify internal state
        let (stored_key, stored_value) = decoder.prev_key_value.as_ref().unwrap();
        assert_eq!(stored_key, &key);
        assert_eq!(*stored_value, 18);
    }

    #[test]
    fn test_delta_decoder_different_key_reset() {
        let mut decoder = DeltaDecoder::new();
        let key1 = "key1".to_string();
        let key2 = "key2".to_string();

        // First decode with key1
        let result1 = decoder.decode(&key1, 10u32);
        assert_eq!(result1, 10);

        // Second decode with key1 - should accumulate
        let result2 = decoder.decode(&key1, 5u32);
        assert_eq!(result2, 15);

        // Third decode with key2 - should reset and not accumulate
        let result3 = decoder.decode(&key2, 7u32);
        assert_eq!(result3, 7);

        // Verify internal state has key2
        let (stored_key, stored_value) = decoder.prev_key_value.as_ref().unwrap();
        assert_eq!(stored_key, &key2);
        assert_eq!(*stored_value, 7);

        // Fourth decode with key2 - should accumulate from key2's base
        let result4 = decoder.decode(&key2, 3u32);
        assert_eq!(result4, 10);
    }

    #[test]
    fn test_delta_decoder_alternating_keys() {
        let mut decoder = DeltaDecoder::new();
        let key1 = "key1".to_string();
        let key2 = "key2".to_string();

        // Alternating between keys should reset each time
        let result1 = decoder.decode(&key1, 10u32);
        assert_eq!(result1, 10);

        let result2 = decoder.decode(&key2, 20u32);
        assert_eq!(result2, 20);

        let result3 = decoder.decode(&key1, 5u32);
        assert_eq!(result3, 5); // Reset, not accumulated

        let result4 = decoder.decode(&key2, 3u32);
        assert_eq!(result4, 3); // Reset, not accumulated
    }

    #[test]
    fn test_delta_decoder_with_zero_values() {
        let mut decoder = DeltaDecoder::new();
        let key = "test_key".to_string();

        // First decode with zero
        let result1 = decoder.decode(&key, 0u32);
        assert_eq!(result1, 0);

        // Second decode with zero - should still work
        let result2 = decoder.decode(&key, 0u32);
        assert_eq!(result2, 0);

        // Third decode with non-zero
        let result3 = decoder.decode(&key, 5u32);
        assert_eq!(result3, 5);
    }

    #[test]
    fn test_delta_decoder_with_different_types() {
        // Test with i32
        let mut decoder_i32 = DeltaDecoder::new();
        let key = "test".to_string();

        let result1 = decoder_i32.decode(&key, -5i32);
        assert_eq!(result1, -5);

        let result2 = decoder_i32.decode(&key, 10i32);
        assert_eq!(result2, 5);

        // Test with u64
        let mut decoder_u64 = DeltaDecoder::new();
        let result3 = decoder_u64.decode(&key, 1000u64);
        assert_eq!(result3, 1000);

        let result4 = decoder_u64.decode(&key, 500u64);
        assert_eq!(result4, 1500);
    }

    #[test]
    fn test_delta_decoder_with_numeric_keys() {
        let mut decoder = DeltaDecoder::new();

        // Test with numeric keys
        let result1 = decoder.decode(&1u32, 10u32);
        assert_eq!(result1, 10);

        let result2 = decoder.decode(&1u32, 5u32);
        assert_eq!(result2, 15);

        let result3 = decoder.decode(&2u32, 20u32);
        assert_eq!(result3, 20);

        let result4 = decoder.decode(&1u32, 3u32);
        assert_eq!(result4, 3); // Reset because key changed
    }

    #[test]
    fn test_delta_decoder_large_values() {
        let mut decoder = DeltaDecoder::new();
        let key = "large_values".to_string();

        let result1 = decoder.decode(&key, u32::MAX - 100);
        assert_eq!(result1, u32::MAX - 100);

        // This would overflow in real usage, but for testing the logic
        let result2 = decoder.decode(&key, 50u32);
        assert_eq!(result2, (u32::MAX - 100).wrapping_add(50));
    }

    #[test]
    fn test_delta_decoder_empty_string_key() {
        let mut decoder = DeltaDecoder::new();
        let empty_key = "".to_string();
        let non_empty_key = "non_empty".to_string();

        let result1 = decoder.decode(&empty_key, 10u32);
        assert_eq!(result1, 10);

        let result2 = decoder.decode(&empty_key, 5u32);
        assert_eq!(result2, 15);

        let result3 = decoder.decode(&non_empty_key, 20u32);
        assert_eq!(result3, 20);

        let result4 = decoder.decode(&empty_key, 3u32);
        assert_eq!(result4, 3); // Reset because key changed back to empty
    }

    #[test]
    fn test_delta_decoder_single_character_keys() {
        let mut decoder = DeltaDecoder::new();
        let key_a = "a".to_string();
        let key_b = "b".to_string();

        let result1 = decoder.decode(&key_a, 1u32);
        assert_eq!(result1, 1);

        let result2 = decoder.decode(&key_a, 1u32);
        assert_eq!(result2, 2);

        let result3 = decoder.decode(&key_b, 10u32);
        assert_eq!(result3, 10);

        let result4 = decoder.decode(&key_a, 1u32);
        assert_eq!(result4, 1); // Reset
    }

    #[test]
    fn test_delta_decoder_sequence() {
        let mut decoder = DeltaDecoder::new();
        let key = "sequence".to_string();

        // Test a longer sequence of accumulations
        let values = vec![1, 2, 3, 4, 5];
        let mut expected = 0;

        for value in values {
            expected += value;
            let result = decoder.decode(&key, value);
            assert_eq!(result, expected);
        }

        // Final state should be sum of all values
        let (stored_key, stored_value) = decoder.prev_key_value.as_ref().unwrap();
        assert_eq!(stored_key, &key);
        assert_eq!(*stored_value, 15);
    }
}
