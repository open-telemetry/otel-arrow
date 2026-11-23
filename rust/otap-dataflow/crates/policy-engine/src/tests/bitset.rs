// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{Array, BooleanArray};
use arrow::buffer::NullBuffer;

use crate::EvaluationScratch;
use crate::bitset::{BitsetBufferPool, DenseBitset};
use crate::model::{
    Action, AttributePath, LogAdd, LogFieldSelector, LogMatcher, LogRecordField, LogTargetSelector,
    LogTransformPolicy, Policy, Predicate, ScalarValue,
};

use super::helpers::{build_custom_test_batch, compile_engine};

/// Scenario: Exercise `DenseBitset` across 64-bit word boundaries (`0..500`), mismatched word lengths, trailing zero words, Arrow `BooleanBuffer` conversion, and `BitsetBufferPool` recycling inside `EvaluationScratch` across filter and transform stages.
/// Guarantees: Bitset set operations and `PartialEq` remain exact across mismatched backing lengths, `to_boolean_buffer` matches `BooleanBuffer::collect_bool` bit-for-bit for every batch size, and active predicate backing buffers are recycled into `EvaluationScratch` for reuse across subsequent batches.
#[test]
fn test_dense_bitset_ops_arrow_buffer_and_scratch_pool_reuse() {
    use arrow::buffer::BooleanBuffer;

    // 1. Empty, full, word-boundary inserts, iteration, and logical equality ignoring trailing zeroes.
    assert!(DenseBitset::new().is_empty());
    assert!(DenseBitset::full(0).is_empty());
    let f64 = DenseBitset::full(64);
    assert_eq!(f64.len(), 64);
    assert!(f64.contains(0));
    assert!(f64.contains(63));
    assert!(!f64.contains(64));

    let f65 = DenseBitset::full(65);
    assert_eq!(f65.len(), 65);
    assert!(f65.contains(64));
    assert!(!f65.contains(65));

    let indices = [0u32, 1, 63, 64, 65, 127, 128, 191, 192, 500];
    let mut a = DenseBitset::new();
    for &idx in indices.iter().rev() {
        a.insert(idx);
    }
    assert_eq!(a.len(), indices.len() as u64);
    assert_eq!(a.iter().collect::<Vec<_>>(), indices);
    for &idx in &indices {
        assert!(a.contains(idx));
    }
    assert!(!a.contains(2));
    assert!(!a.contains(501));
    assert!(!a.contains(10_000));

    // Mismatched word lengths: `b` has 2 words, `a` has 8 words.
    let b = DenseBitset::from_indices([1, 63, 64, 100]);
    assert_eq!(a.intersection_len(&b), 3);
    assert_eq!(b.intersection_len(&a), 3);
    assert_eq!(a.and(&b).iter().collect::<Vec<_>>(), vec![1, 63, 64]);
    assert_eq!(b.and(&a).iter().collect::<Vec<_>>(), vec![1, 63, 64]);
    assert_eq!(
        a.sub(&b).iter().collect::<Vec<_>>(),
        vec![0, 65, 127, 128, 191, 192, 500]
    );
    assert_eq!(b.sub(&a).iter().collect::<Vec<_>>(), vec![100]);
    assert_eq!(
        a.or(&b).iter().collect::<Vec<_>>(),
        vec![0, 1, 63, 64, 65, 100, 127, 128, 191, 192, 500]
    );

    // Subtracting a bitset from itself leaves trailing zero words that compare equal to Empty.
    let mut cleared = a.clone();
    cleared.sub_assign(&a);
    assert!(cleared.is_empty());
    assert_eq!(cleared, DenseBitset::new());
    assert_eq!(DenseBitset::new(), cleared);

    // In-place and_assign with empty clears cleanly; or_assign grows cleanly.
    let mut grow = DenseBitset::new();
    grow.or_assign(&b);
    assert_eq!(grow, b);
    grow.and_assign(&DenseBitset::new());
    assert!(grow.is_empty());
    assert_eq!(grow, DenseBitset::new());

    // 2. to_boolean_buffer bit-for-bit parity with BooleanBuffer::collect_bool.
    for num_rows in [0usize, 1, 7, 63, 64, 65, 130] {
        let mut bs = DenseBitset::new();
        for r in (0..num_rows).step_by(3) {
            bs.insert(r as u32);
        }
        let fast = bs.to_boolean_buffer(num_rows);
        let expected = BooleanBuffer::collect_bool(num_rows, |i| bs.contains(i as u32));
        assert_eq!(
            fast, expected,
            "BooleanBuffer mismatch at num_rows={num_rows}"
        );
    }

    // 3. BitsetBufferPool recycles active buffers in EvaluationScratch across batches and stages.
    let mut pool = BitsetBufferPool::default();
    let mut bs1 = DenseBitset::new();
    bs1.insert_with_pool(5, 2, &mut pool);
    bs1.insert_with_pool(200, 2, &mut pool); // grows past initial min_words=2
    assert_eq!(bs1.iter().collect::<Vec<_>>(), vec![5, 200]);
    let mut vec_bs = vec![bs1, DenseBitset::new()];
    pool.recycle_vec(&mut vec_bs);
    assert_eq!(pool.free_len(), 1); // Only the allocated bitset was recycled

    let batch = build_custom_test_batch(
        vec![Some("drop_me"), Some("keep_me")],
        vec![5, 9],
        vec!["DEBUG", "INFO"],
        vec![[1u8; 16]; 2],
        vec![[1u8; 8]; 2],
        vec![],
        vec![],
    );
    let eng = compile_engine(&[
        Policy::log_filter(
            "drop_dbg",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                predicate: Predicate::Equals(ScalarValue::String("DEBUG".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_transform(
            "tag_info",
            LogTransformPolicy {
                matches: vec![LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Equals(ScalarValue::String("INFO".into())),
                    negate: false,
                }],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("ok")),
                    value: ScalarValue::Bool(true),
                    upsert: true,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
    ]);
    let mut scratch = EvaluationScratch::new();
    assert_eq!(scratch.pooled_bitset_buffers(), 0);
    let _ = eng.evaluate(&batch, &mut scratch, None).unwrap();
    let pooled_after_1 = scratch.pooled_bitset_buffers();
    assert!(
        pooled_after_1 >= 1,
        "expected recycled bitset buffers in EvaluationScratch"
    );
    let _ = eng.evaluate(&batch, &mut scratch, None).unwrap();
    assert_eq!(
        scratch.pooled_bitset_buffers(),
        pooled_after_1,
        "steady-state pool size should stay stable across identical batches"
    );
}

/// Scenario: Convert Arrow `BooleanArray`s (including non-zero slice offsets) and `NullBuffer`s
/// into `DenseBitset`s with and without masks and `nulls_match`.
/// Guarantees: Exact bit-for-bit parity across word boundaries (`0..130`), zero leaked tail bits,
/// and bounded `BitsetBufferPool` buffer reuse.
#[test]
fn test_dense_bitset_arrow_interop_sliced_nulls_and_validity() {
    let mut pool = BitsetBufferPool::default();

    for len in [0usize, 1, 5, 63, 64, 65, 130] {
        let base_vals: Vec<Option<bool>> = (0..len + 70)
            .map(|i| match i % 5 {
                0 => None,
                1 | 3 => Some(true),
                _ => Some(false),
            })
            .collect();
        let full_arr = BooleanArray::from(base_vals);

        for offset in [0usize, 3, 65] {
            let sliced = full_arr.slice(offset, len);
            let mask = DenseBitset::from_indices((0..len as u32).filter(|i| i % 2 == 0));

            for nulls_match in [false, true] {
                // Unmasked
                let bs = DenseBitset::from_boolean_array(&sliced, nulls_match, None, &mut pool);
                let expected: Vec<u32> = (0..len)
                    .filter(|&i| {
                        if sliced.is_valid(i) {
                            sliced.value(i)
                        } else {
                            nulls_match
                        }
                    })
                    .map(|i| i as u32)
                    .collect();
                assert_eq!(
                    bs.iter().collect::<Vec<_>>(),
                    expected,
                    "len={len}, offset={offset}, nulls_match={nulls_match}"
                );
                pool.recycle(bs);

                // Masked
                let bs_m =
                    DenseBitset::from_boolean_array(&sliced, nulls_match, Some(&mask), &mut pool);
                let expected_m: Vec<u32> = expected
                    .iter()
                    .copied()
                    .filter(|&i| mask.contains(i))
                    .collect();
                assert_eq!(
                    bs_m.iter().collect::<Vec<_>>(),
                    expected_m,
                    "masked len={len}, offset={offset}, nulls_match={nulls_match}"
                );
                pool.recycle(bs_m);
            }

            // Negated (valid false only)
            let mut neg_bs = DenseBitset::new();
            neg_bs.or_negated_boolean_array_with_pool(&sliced, Some(&mask), &mut pool);
            let expected_neg: Vec<u32> = (0..len)
                .filter(|&i| sliced.is_valid(i) && !sliced.value(i) && mask.contains(i as u32))
                .map(|i| i as u32)
                .collect();
            assert_eq!(neg_bs.iter().collect::<Vec<_>>(), expected_neg);
            pool.recycle(neg_bs);

            // Validity buffers intersection
            let parent_nulls = NullBuffer::from((0..len).map(|i| i % 3 != 0).collect::<Vec<_>>());
            let val_bs = DenseBitset::from_validity_buffers(
                len,
                sliced.nulls(),
                Some(&parent_nulls),
                Some(&mask),
                &mut pool,
            );
            let expected_val: Vec<u32> = (0..len)
                .filter(|&i| {
                    sliced.is_valid(i) && parent_nulls.is_valid(i) && mask.contains(i as u32)
                })
                .map(|i| i as u32)
                .collect();
            assert_eq!(val_bs.iter().collect::<Vec<_>>(), expected_val);
            pool.recycle(val_bs);
        }
    }

    assert!(
        pool.free_len() <= 2,
        "pool leaked extra buffers: {}",
        pool.free_len()
    );
}

/// Scenario: Recycle more than `MAX_POOLED_BUFFERS` small bitsets and oversized bitsets exceeding `MAX_POOLED_BUFFER_WORDS`, and run pooled `clone_with_pool`, `and_with_pool`, and `sub_with_pool`.
/// Guarantees: `BitsetBufferPool` caps retained buffer count (`<= 256`) and drops oversized buffers (`> 1024` words), while pooled set operations produce exact results.
#[test]
fn test_bitset_buffer_pool_enforces_buffer_and_word_caps() {
    let mut pool = BitsetBufferPool::default();

    // Oversized buffer (> 1024 words = > 65,536 bits) must be dropped instead of retained.
    let huge = DenseBitset::from_indices([70_000u32]);
    pool.recycle(huge);
    assert_eq!(pool.free_len(), 0);

    // Recycling 300 normal buffers caps retained count at 256.
    for i in 0..300u32 {
        let bs = DenseBitset::from_indices([i % 64]);
        pool.recycle(bs);
    }
    assert_eq!(pool.free_len(), 256);

    // Pooled clone, intersection, and difference match unpooled equivalents.
    let a = DenseBitset::from_indices([1, 3, 5, 65, 130]);
    let b = DenseBitset::from_indices([3, 5, 7, 130]);
    let cloned = a.clone_with_pool(&mut pool);
    assert_eq!(cloned, a);
    let inter = a.and_with_pool(&b, &mut pool);
    assert_eq!(inter, DenseBitset::from_indices([3, 5, 130]));
    let diff = a.sub_with_pool(&b, &mut pool);
    assert_eq!(diff, DenseBitset::from_indices([1, 65]));
    pool.recycle(cloned);
    pool.recycle(inter);
    pool.recycle(diff);
    assert_eq!(pool.free_len(), 256);
}
