// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/*!
ID planning for concatenating multiple OTAP batches.

When unrelated OTAP batches are concatenated, their ID / PARENT_ID columns
have to be rewritten so IDs from different inputs do not collide. This module
works out those rewrites **without modifying the inputs**. It produces, for
every input and payload, an [`InputPlan`] containing:

- an [`IdRemap`] for each ID column (`id`, `resource.id`, `scope.id`,
  `parent_id`), which the concatenate column writers apply while copying, and
- a [`Selection`] of rows that survive. Child rows whose `parent_id` has no
  matching parent (a referential integrity violation) are dropped while
  writing rather than by filtering the batch up front.

# Reindexing strategies

There are two strategies. The first is a naive offset where every ID in an
input is shifted by a fixed amount so that its range sits after the previous
input's. For example, if we have batches with IDs [1, 2] and [1, 2, 3], we can
move the second batch out of the way by adding 2 to all of its IDs. This is
represented as [`IdRemap::Offset`] and is applied as a single wrapping add
during the copy.

The problem with a naive offset is that if the second batch has holes then we
"use up" more IDs than we need. For example, if the second batch is [1, 3]
then we still have to add 2 to every ID to move it out of the range of batch 1.
The next batch then has to start after ID 5, and ID 4 is wasted because it was
never used.

The second strategy, "compaction", avoids this. It sorts the ID values, groups
them into contiguous runs with no holes, and remaps each run individually,
which gives a perfectly compact reindexing. For example, for [1, 3] we would
move 1 up by 2 and 3 up by 1 to get [3, 4]. The compacted values are
materialized in source order as [`IdRemap::Replace`].

# Integrity violations

The second major problem with naive offsets is the potential for integrity
violations. Suppose we have corresponding `id` and `parent_id` pairs like this:

id: [1, 2]  parent_id: [1, 3]

`parent_id` has a referential integrity violation. We compute the mappings and
the next offset from the `id` column only, so 3 dangles into the range that
the next OTAP batch will use. We could accidentally associate the row that has
`parent_id = 3` with a record batch that is not in this OTAP batch.

If the violation is in the middle of the range, however, it is not a problem.
Take this example:

id: [1, 3] parent_id: [2]

In this case the dangling `parent_id` is inside the ID range reserved for this
OTAP batch, so it cannot attach to any other input's data.

# Isolation guarantee

A corrupt input may only affect its own rows. Every surviving ID of an input
maps into a range reserved exclusively for that input. Dangling references
that fall inside the reserved range are kept (garbage in, garbage out).
Dangling references that would escape it are redacted. When a row is
redacted, its own children are not re-examined. Those grandchildren still
point inside their input's reserved range, so they are harmless orphans.

# Approach

We prefer a naive offset whenever possible, and compact only when an offset
would create junk data or when we need the extra space.

We first compute the min and max of every column to find which strategies
are eligible. If a child's `parent_id` range is not inside the parent's
corresponding `id` range, we have to compact. If it is, we may choose either
to compact or to apply an offset.

For a naive offset on a primary ID column, the number of IDs we "use up" is
the span of the column (max - min + 1). For compaction we use the length of
the column, because it is a primary ID column and its values must be unique.

For non-primary ID columns (resource ID and scope ID) we only know an upper
bound on how many IDs we will use, which is max - min + 1. We don't know how
many unique IDs the column actually contains unless we compact and find out.

Once we have that upper bound for every OTAP batch under its best eligible
strategy, we check whether the total overflows the limit. If it does, we
compute how many IDs we need to "save".

The only way to save ID space is to compact and hope we end up with fewer IDs
than the upper bound. We keep choosing compaction until we have saved enough,
and then use the best available strategy for the remaining record batches.

# Null IDs

Null slots in ID columns are excluded from statistics and compaction and stay
null in the output. Their underlying values are never interpreted as IDs, and
offsets are applied with wrapping arithmetic so an arbitrary value in a null
slot cannot overflow.

# TODO

- TODO(D4): Make compaction faster. Today it sorts the valid values (or their
  indices), builds range mappings, applies them in sorted order, and unsorts.
  Compaction always maps a parent id `v` to `offset + rank(v)`, where
  `rank(v)` is the number of distinct valid parent ids below `v`, so it can be
  computed with a dense table over `[min, max]` (min/max are already known
  from the stats pass):
  1. Mark present parent ids in a bitmap indexed by `v - min`.
  2. Prefix-assign `table[v - min] = offset + rank` over the span; the final
     counter is the next offset.
  3. Parent replacement: `out[i] = table[values[i] - min]` (0 for nulls).
  4. Child replacement and violations in one pass: a child value is valid iff
     it is within `[min, max]` and present; valid values map through the
     table.

  This is O(n + span) with no sort. Use it when the span is bounded (always
  for u16; for u32 when `span <= max(4 * len, 65536)`) and keep the current
  sort path as the fallback for sparse u32 ids. The table and bitmap can be
  reused across batches of a relation. Compaction only happens for
  malformed input or when the id budget overflows, so this is low priority.
- TODO(root-decode-noop): `remove_transport_optimized_encodings` rebuilds the
  schema and record batch for root tables even when every column is already
  plain. It should return the batch untouched in that case.
- TODO(split-decode): `split` sorts and cuts batches by their ID columns
  before transport-optimized encodings are removed (the batch processor with
  `max_size` set, fed by an OTAP receiver). Sorting delta-encoded IDs
  corrupts them. Encodings should be removed before splitting.
*/

use std::ops::Range;

use arrow::array::{
    Array, ArrowNativeTypeOp, ArrowPrimitiveType, AsArray, BooleanBufferBuilder, PrimitiveArray,
    RecordBatch,
};
use arrow::buffer::{BooleanBuffer, ScalarBuffer};
use arrow::datatypes::{
    ArrowDictionaryKeyType, ArrowNativeType, DataType, UInt8Type, UInt16Type, UInt32Type,
};

use crate::error::{Error, Result};
use crate::otap::OtapBatchStore;
use crate::otap::transform::concatenate::plan::{AnyRemap, IdCol, IdRemap, InputPlan, Selection};
use crate::otap::transform::transport_optimize::{
    RESOURCE_ID_COL_PATH, SCOPE_ID_COL_PATH, remove_transport_optimized_encodings,
};
use crate::otap::transform::util::{extract_id_column, payload_to_idx};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts::{ID, PARENT_ID};

use super::util::{IdColumnType, PrimaryIdInfo, payload_relations};

/// Per-payload, per-input plans. `plans[payload_idx][input_idx]`.
pub(crate) type IdPlan<const N: usize> = [Vec<InputPlan>; N];

/// Remove transport optimized encodings from every batch in place.
pub(crate) fn remove_transport_encodings<S: OtapBatchStore, const N: usize>(
    batches: &mut [[Option<RecordBatch>; N]],
) -> Result<()> {
    for &payload_type in S::allowed_payload_types() {
        let idx = payload_to_idx(payload_type);
        for group in batches.iter_mut() {
            if let Some(rb) = group[idx].as_mut() {
                *rb = remove_transport_optimized_encodings(payload_type, rb)?;
            }
        }
    }
    Ok(())
}

/// Plan the ID rewrites needed to concatenate `batches` without collisions.
///
/// Transport optimized encodings must already have been removed. The inputs
/// are not modified.
pub(crate) fn plan_ids<S: OtapBatchStore, const N: usize>(
    batches: &[[Option<RecordBatch>; N]],
) -> Result<IdPlan<N>> {
    let mut plans: IdPlan<N> = std::array::from_fn(|_| vec![InputPlan::default(); batches.len()]);

    if batches.len() <= 1 {
        return Ok(plans);
    }

    for &payload_type in S::allowed_payload_types() {
        let info = payload_relations(payload_type);

        if let Some(ref primary_id_info) = info.primary_id {
            check_primary_id_for_overflow(batches, payload_type, primary_id_info)?;
        }

        for relation in info.relations {
            let is_primary = Some(relation.key_col) == info.primary_id.as_ref().map(|id| id.name);
            let ctx = RelationCtx {
                parent_payload_type: payload_type,
                child_payload_types: relation.child_types,
                id_column_path: relation.key_col,
                id_col: id_col_for_path(relation.key_col),
                is_primary,
                size: relation.size,
            };
            match relation.size {
                IdColumnType::U16 => plan_relation::<UInt16Type, N>(batches, &mut plans, &ctx)?,
                IdColumnType::U32 => plan_relation::<UInt32Type, N>(batches, &mut plans, &ctx)?,
            }
        }
    }

    Ok(plans)
}

fn id_col_for_path(path: &str) -> IdCol {
    match path {
        RESOURCE_ID_COL_PATH => IdCol::ResourceId,
        SCOPE_ID_COL_PATH => IdCol::ScopeId,
        ID => IdCol::Id,
        _ => unreachable!("unexpected id column path {path}"),
    }
}

struct RelationCtx<'a> {
    parent_payload_type: ArrowPayloadType,
    child_payload_types: &'a [ArrowPayloadType],
    id_column_path: &'a str,
    id_col: IdCol,
    is_primary: bool,
    size: IdColumnType,
}

/// Trait tying the ID native types to their [AnyRemap] variant.
trait IdType: ArrowPrimitiveType<Native: Ord + ArrowNativeTypeOp> {
    fn wrap(remap: IdRemap<Self::Native>) -> AnyRemap;
}

impl IdType for UInt16Type {
    fn wrap(remap: IdRemap<u16>) -> AnyRemap {
        AnyRemap::U16(remap)
    }
}

impl IdType for UInt32Type {
    fn wrap(remap: IdRemap<u32>) -> AnyRemap {
        AnyRemap::U32(remap)
    }
}

// For a given primary id column, determine the count of Ids that exist across
// every record batch and determine if it will fit in the type for that column.
fn check_primary_id_for_overflow<const N: usize>(
    batches: &[[Option<RecordBatch>; N]],
    payload_type: ArrowPayloadType,
    id_info: &PrimaryIdInfo,
) -> Result<()> {
    let idx = payload_to_idx(payload_type);
    let mut count: u64 = 0;
    for rb in batches.iter().filter_map(|b| b[idx].as_ref()) {
        let Ok(id_col) = extract_id_column(rb, id_info.name) else {
            continue;
        };
        count += id_col.len() as u64;
    }

    // TODO: Consider supporting u16::MAX + 1. The offset math is done in the
    // native type, which overflows right at the top. The only consequence is
    // that the max batch size is 1 less.
    if count > id_info.size.max() {
        return Err(Error::TooManyItems {
            payload_type,
            count: count as usize,
            max: id_info.size.max(),
            message: "Too many items to reindex".to_string(),
        });
    }

    Ok(())
}

/// Plan the remaps for one ID column and all of its child `parent_id`
/// columns, preferring a naive offset whenever possible.
fn plan_relation<T: IdType, const N: usize>(
    batches: &[[Option<RecordBatch>; N]],
    plans: &mut IdPlan<N>,
    ctx: &RelationCtx<'_>,
) -> Result<()> {
    let stats = gather_column_stats::<T, N>(batches, ctx)?;

    // Figure out an upper bound on how many ids we will use with the optimal
    // available strategy for each input. When this exceeds the column-type
    // limit we must compact enough inputs to fit.
    let limit: u64 = ctx.size.max();
    let total_ids_needed: u64 = stats
        .iter()
        .filter_map(|s| s.as_ref())
        .map(|s| s.max_ids_needed as u64)
        .sum();
    let need_to_save: u64 = total_ids_needed.saturating_sub(limit);

    let parent_idx = payload_to_idx(ctx.parent_payload_type);
    let mut offset = T::Native::usize_as(0);
    let mut current_saved: u64 = 0;

    for (i, stat) in stats.iter().enumerate() {
        let Some(stat) = stat else {
            continue;
        };

        let must_compact = stat.strategy == ReindexStrategy::CompactOnly
            || (need_to_save > 0 && current_saved < need_to_save);

        let parent_rb = batches[i][parent_idx]
            .as_ref()
            .expect("batch must exist for non-None stat");
        let id_col = extract_id_column(parent_rb, ctx.id_column_path)?;

        if must_compact {
            let (replace, mappings, new_offset) = compact_parent::<T>(id_col.as_ref(), offset)?;

            // ids_consumed <= max_ids_needed always
            let ids_consumed = new_offset.as_usize() - offset.as_usize();
            current_saved += stat.max_ids_needed as u64 - ids_consumed as u64;
            offset = new_offset;

            plans[parent_idx][i].remaps[ctx.id_col as usize] =
                Some(T::wrap(IdRemap::Replace(replace)));

            for &child_payload_type in ctx.child_payload_types {
                let child_idx = payload_to_idx(child_payload_type);
                let Some(child_rb) = batches[i][child_idx].as_ref() else {
                    continue;
                };
                let child_col = extract_id_column(child_rb, PARENT_ID)?;
                let (replace, selection) = compact_child::<T>(child_col.as_ref(), &mappings)?;
                let plan = &mut plans[child_idx][i];
                plan.remaps[IdCol::ParentId as usize] = Some(T::wrap(IdRemap::Replace(replace)));
                plan.selection = selection;
            }
        } else {
            let delta = offset.sub_wrapping(stat.min);
            let remap = T::wrap(IdRemap::Offset(delta));
            for &child_payload_type in ctx.child_payload_types {
                let child_idx = payload_to_idx(child_payload_type);
                if batches[i][child_idx].is_some() {
                    plans[child_idx][i].remaps[IdCol::ParentId as usize] = Some(remap.clone());
                }
            }
            plans[parent_idx][i].remaps[ctx.id_col as usize] = Some(remap);

            let span = stat.max.as_usize() - stat.min.as_usize() + 1;
            offset = offset.add_wrapping(T::Native::usize_as(span));
        }
    }

    Ok(())
}

/// Returns (min, max) over the valid values of an ID column, resolving
/// dictionary encoding through its values. `None` if empty or all null.
///
/// Computed in a single pass over the values rather than separate min and
/// max passes.
fn id_column_min_max<T: IdType>(col: &dyn Array) -> Result<Option<(T::Native, T::Native)>> {
    let array = materialize_id_values::<T>(col)?;
    let values = array.values();

    let fold = |acc: Option<(T::Native, T::Native)>, v: T::Native| match acc {
        None => Some((v, v)),
        Some((lo, hi)) => Some((lo.min(v), hi.max(v))),
    };

    let result = match array.nulls() {
        Some(nulls) if nulls.null_count() == array.len() => None,
        Some(nulls) if nulls.null_count() > 0 => {
            nulls.valid_indices().map(|i| values[i]).fold(None, fold)
        }
        _ => {
            let (&first, rest) = match values.split_first() {
                Some(split) => split,
                None => return Ok(None),
            };
            Some(
                rest.iter()
                    .fold((first, first), |(lo, hi), &v| (lo.min(v), hi.max(v))),
            )
        }
    };

    Ok(result)
}

/// Returns the primitive array holding the ID values of `array`. For
/// dictionary arrays this is the VALUES array, not the per-row values.
fn materialize_id_values<T: ArrowPrimitiveType>(array: &dyn Array) -> Result<&PrimitiveArray<T>> {
    let id_arr = match array.data_type() {
        data_type if data_type == &T::DATA_TYPE => array.as_primitive::<T>(),
        DataType::Dictionary(key_type, value_type) if value_type.as_ref() == &T::DATA_TYPE => {
            match key_type.as_ref() {
                DataType::UInt8 => array.as_dictionary::<UInt8Type>().values().as_primitive(),
                DataType::UInt16 => array.as_dictionary::<UInt16Type>().values().as_primitive(),
                _ => {
                    return Err(Error::UnsupportedDictionaryKeyType {
                        expect_oneof: vec![DataType::UInt8, DataType::UInt16],
                        actual: key_type.as_ref().clone(),
                    });
                }
            }
        }
        _ => {
            return Err(Error::ColumnDataTypeMismatch {
                name: ID.to_string(),
                expect: T::DATA_TYPE,
                actual: array.data_type().clone(),
            });
        }
    };

    Ok(id_arr)
}

/// Compact a parent ID column starting at `offset`.
///
/// Returns the replacement values in source order (null slots hold 0), the
/// range mappings needed to remap children, and the next free offset.
fn compact_parent<T: IdType>(
    col: &dyn Array,
    offset: T::Native,
) -> Result<(
    ScalarBuffer<T::Native>,
    Vec<IdMapping<T::Native>>,
    T::Native,
)> {
    let array = materialize_id_values::<T>(col)?;
    let values = array.values();
    let mut out = vec![T::Native::default(); values.len()];

    // Positions of valid (non-null) ids.
    let valid: Vec<u32> = match array.nulls() {
        Some(nulls) if nulls.null_count() > 0 => nulls.valid_indices().map(|i| i as u32).collect(),
        _ => (0..values.len() as u32).collect(),
    };

    let mut sorted_positions = valid;
    let already_sorted = sorted_positions
        .windows(2)
        .all(|w| values[w[0] as usize] <= values[w[1] as usize]);
    if !already_sorted {
        sorted_positions.sort_unstable_by_key(|&i| values[i as usize]);
    }

    let mut sorted: Vec<T::Native> = sorted_positions
        .iter()
        .map(|&i| values[i as usize])
        .collect();
    let (mappings, new_offset) = create_mappings::<T>(&sorted, offset);
    let violations = apply_mappings::<T>(&mut sorted, &mappings);
    debug_assert!(violations.is_none(), "parent ids always map to themselves");

    for (&pos, &v) in sorted_positions.iter().zip(&sorted) {
        out[pos as usize] = v;
    }

    Ok((ScalarBuffer::from(out), mappings, new_offset))
}

/// Remap a child `parent_id` column using the parent's mappings.
///
/// Returns the replacement values in source order (indexed like the values
/// array for dictionary columns) and the selection of rows whose parent
/// exists. Violating rows are excluded from the selection.
fn compact_child<T: IdType>(
    col: &dyn Array,
    mappings: &[IdMapping<T::Native>],
) -> Result<(ScalarBuffer<T::Native>, Selection)> {
    let values = materialize_id_values::<T>(col)?.values();

    let sort_indices = sort_vec_to_indices(values);
    let mut sorted = vec![T::Native::default(); values.len()];
    take_vec(values, &mut sorted, &sort_indices);
    let violations = apply_mappings::<T>(&mut sorted, mappings);

    let mut out = vec![T::Native::default(); values.len()];
    untake_vec(&sorted, &mut out, &sort_indices);

    let selection = match violations {
        None => Selection::All,
        Some(violations) => {
            // Mark violating value positions (source order).
            let mut value_ok = BooleanBufferBuilder::new(values.len());
            value_ok.append_n(values.len(), true);
            for range in violations {
                for &src in &sort_indices[range] {
                    value_ok.set_bit(src as usize, false);
                }
            }
            let value_ok = value_ok.finish();

            let row_ok = match col.data_type() {
                DataType::Dictionary(key_type, _) => match key_type.as_ref() {
                    DataType::UInt8 => rows_ok_from_keys::<UInt8Type>(col, &value_ok),
                    DataType::UInt16 => rows_ok_from_keys::<UInt16Type>(col, &value_ok),
                    k => {
                        return Err(Error::UnsupportedDictionaryKeyType {
                            expect_oneof: vec![DataType::UInt8, DataType::UInt16],
                            actual: k.clone(),
                        });
                    }
                },
                _ => value_ok,
            };
            selection_from_bitmap(&row_ok)
        }
    };

    Ok((ScalarBuffer::from(out), selection))
}

/// Map a per-value validity bitmap to a per-row bitmap through dictionary
/// keys.
fn rows_ok_from_keys<K: ArrowDictionaryKeyType>(
    col: &dyn Array,
    value_ok: &BooleanBuffer,
) -> BooleanBuffer {
    let dict = col.as_dictionary::<K>();
    let keys = dict.keys().values();
    let values_len = value_ok.len();
    BooleanBuffer::collect_bool(keys.len(), |i| {
        let k = keys[i].as_usize();
        k < values_len && value_ok.value(k)
    })
}

/// Convert a keep-bitmap into a [Selection].
fn selection_from_bitmap(keep: &BooleanBuffer) -> Selection {
    if keep.count_set_bits() == keep.len() {
        return Selection::All;
    }
    Selection::Ranges(keep.set_slices().map(|(s, e)| s..e).collect())
}

/// Sort a slice and return the resulting sort indices.
fn sort_vec_to_indices<T: Ord>(values: &[T]) -> Vec<u32> {
    let mut indices: Vec<u32> = (0u32..values.len() as u32).collect();
    indices.sort_unstable_by_key(|&i| &values[i as usize]);
    indices
}

/// Whether a batch can use the fast offset path or must compact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReindexStrategy {
    /// Must use the slow path (sort + compact).
    CompactOnly,
    /// May use the fast path (uniform offset) if headroom allows.
    Any,
}

/// Per-batch statistics collected in the first pass of planning.
#[derive(Debug, Clone)]
struct ColumnStats<T> {
    min: T,
    max: T,
    /// Upper bound on IDs consumed by this batch under its best strategy.
    max_ids_needed: usize,
    strategy: ReindexStrategy,
}

/// Returns one entry per input. Entry `i` is `None` when input `i` has no
/// parent payload, no ID column, or an empty/all-null ID column.
fn gather_column_stats<T: IdType, const N: usize>(
    batches: &[[Option<RecordBatch>; N]],
    ctx: &RelationCtx<'_>,
) -> Result<Vec<Option<ColumnStats<T::Native>>>> {
    let parent_idx = payload_to_idx(ctx.parent_payload_type);
    let mut stats = Vec::with_capacity(batches.len());

    for group in batches {
        let Some(parent_rb) = &group[parent_idx] else {
            stats.push(None);
            continue;
        };
        let Ok(id_col) = extract_id_column(parent_rb, ctx.id_column_path) else {
            stats.push(None);
            continue;
        };
        let Some((lo, hi)) = id_column_min_max::<T>(id_col.as_ref())? else {
            stats.push(None);
            continue;
        };

        let len = id_col.len() - id_col.null_count();
        let span = hi.as_usize() - lo.as_usize() + 1;

        let mut children_ok = true;
        for &ct in ctx.child_payload_types {
            let Some(child_rb) = &group[payload_to_idx(ct)] else {
                continue;
            };
            let Ok(child_col) = extract_id_column(child_rb, PARENT_ID) else {
                continue;
            };
            if let Some((cmin, cmax)) = id_column_min_max::<T>(child_col.as_ref())?
                && (cmin < lo || cmax > hi)
            {
                children_ok = false;
                break;
            }
        }

        // The number of ids used under each strategy. With an offset, the
        // span is consumed. When compacting a primary column, exactly `len`
        // ids are consumed since values are unique. For non-primary columns
        // duplicates are possible so the span remains the upper bound.
        let (strategy, max_ids_needed) = match (ctx.is_primary, children_ok) {
            (_, true) => (ReindexStrategy::Any, span),
            (true, false) => (ReindexStrategy::CompactOnly, len),
            (false, false) => (ReindexStrategy::CompactOnly, span),
        };

        stats.push(Some(ColumnStats {
            min: lo,
            max: hi,
            max_ids_needed,
            strategy,
        }));
    }

    Ok(stats)
}

/// Sign of an offset operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sign {
    Positive,
    Negative,
}

/// Represents a contiguous range of IDs with an offset to apply.
#[derive(Debug, Clone)]
struct IdMapping<T> {
    start_id: T,
    end_id: T,
    offset: T,
    sign: Sign,
}

/// Chunks sorted IDs into consecutive ranges and creates mappings that make
/// them sequential starting from `offset`.
///
/// Returns (mappings, next_offset).
fn create_mappings<T: IdType>(
    sorted_ids: &[T::Native],
    offset: T::Native,
) -> (Vec<IdMapping<T::Native>>, T::Native) {
    let mut mappings = Vec::new();
    let mut current_offset = offset;
    let one = T::Native::usize_as(1);

    for chunk in sorted_ids.chunk_by(|a, b| *b == a.add_wrapping(one) || *b == *a) {
        let start_id = chunk[0];
        let end_id = chunk[chunk.len() - 1];

        let (offset, sign) = if start_id <= current_offset {
            (current_offset.sub_wrapping(start_id), Sign::Positive)
        } else {
            (start_id.sub_wrapping(current_offset), Sign::Negative)
        };

        mappings.push(IdMapping {
            start_id,
            end_id,
            offset,
            sign,
        });

        let new_end = match sign {
            Sign::Positive => end_id.add_wrapping(offset),
            Sign::Negative => end_id.sub_wrapping(offset),
        };
        current_offset = new_end.add_wrapping(one);
    }

    (mappings, current_offset)
}

/// Applies mappings to a sorted ID buffer that were produced from the
/// corresponding parent ID column.
///
/// Returns ranges of sorted positions whose IDs are not covered by any
/// mapping (referential integrity violations), or `None` if there are none.
#[must_use]
fn apply_mappings<T: IdType>(
    sorted_ids: &mut [T::Native],
    mappings: &[IdMapping<T::Native>],
) -> Option<Vec<Range<usize>>> {
    let mut violations = Vec::new();
    let mut remaining_slice = &mut sorted_ids[..];
    let mut idx = 0;
    for mapping in mappings.iter() {
        if remaining_slice.is_empty() {
            break;
        }

        // Elements before the current mapping were never part of the parent.
        let map_start_idx = remaining_slice
            .iter()
            .position(|id| *id >= mapping.start_id)
            .unwrap_or(remaining_slice.len());
        if map_start_idx != 0 {
            violations.push(idx..idx + map_start_idx);
        }
        idx += map_start_idx;

        remaining_slice = &mut remaining_slice[map_start_idx..];
        let end_idx = remaining_slice
            .iter()
            .position(|id| *id > mapping.end_id)
            .unwrap_or(remaining_slice.len());

        let slice_to_map = &mut remaining_slice[0..end_idx];
        idx += slice_to_map.len();

        match mapping.sign {
            Sign::Positive => slice_to_map
                .iter_mut()
                .for_each(|id| *id = id.add_wrapping(mapping.offset)),
            Sign::Negative => slice_to_map
                .iter_mut()
                .for_each(|id| *id = id.sub_wrapping(mapping.offset)),
        }
        remaining_slice = &mut remaining_slice[end_idx..];
    }

    // Elements after all mappings were never part of the parent either.
    if !remaining_slice.is_empty() {
        violations.push(idx..idx + remaining_slice.len());
    }

    (!violations.is_empty()).then_some(violations)
}

/// Copies `src[indices[i]]` to `dst[i]` for all i.
fn take_vec<T: Copy>(src: &[T], dst: &mut [T], indices: &[u32]) {
    assert_eq!(src.len(), dst.len());
    assert_eq!(src.len(), indices.len());
    for (d, &i) in dst.iter_mut().zip(indices) {
        *d = src[i as usize];
    }
}

/// Copies `src[i]` to `dst[indices[i]]` for all i.
fn untake_vec<T: Copy>(src: &[T], dst: &mut [T], indices: &[u32]) {
    assert_eq!(src.len(), dst.len());
    assert_eq!(src.len(), indices.len());
    for (s, &i) in src.iter().zip(indices) {
        dst[i as usize] = *s;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    use arrow::array::RecordBatch;

    use crate::error::Error;
    use crate::otap::transform::concatenate::{ConcatOptions, concatenate, reindex_in_place};
    use crate::otap::transform::testing::{assert_no_id_overlaps, extract_relation_fingerprints};
    use crate::otap::transform::transport_optimize::apply_transport_optimized_encodings;
    use crate::otap::transform::util::{IdColumnType, payload_relations, payload_to_idx};
    use crate::otap::{Logs, Metrics, OtapArrowRecords, OtapBatchStore, Traces};
    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use crate::record_batch;
    use crate::testing::equiv::assert_equivalent;
    use crate::testing::round_trip::otap_to_otlp;
    use crate::{logs, metrics, traces};

    const HALF_U16: u16 = (u16::MAX / 2) + 1;

    fn reindex_logs(batches: &mut [[Option<RecordBatch>; Logs::COUNT]]) -> Result<()> {
        reindex_in_place::<Logs, { Logs::COUNT }>(batches)
    }

    fn reindex_traces(batches: &mut [[Option<RecordBatch>; Traces::COUNT]]) -> Result<()> {
        reindex_in_place::<Traces, { Traces::COUNT }>(batches)
    }

    // ---- Logs tests ----

    #[test]
    fn test_logs_mismatched_id_types() {
        // Note: We may not have to support u32 here for logs. If this starts
        // failing then the right thing to do might be to remove this test.
        let mut batches = vec![
            logs!((Logs, ("id", UInt16, vec![0, 1]))).into_batches(),
            logs!((Logs, ("id", UInt32, vec![0, 1]))).into_batches(),
        ];
        let result = reindex_logs(&mut batches);
        assert!(matches!(result, Err(Error::ColumnDataTypeMismatch { .. })));
    }

    #[test]
    fn test_logs_greater_than_u16_max() {
        let ids = (0..HALF_U16).collect::<Vec<_>>();
        let ids2 = (HALF_U16..u16::MAX).collect::<Vec<_>>();
        let ids3 = vec![u16::MAX];

        let mut batches = vec![
            logs!((Logs, ("id", UInt16, ids))).into_batches(),
            logs!((Logs, ("id", UInt16, ids2))).into_batches(),
            logs!((Logs, ("id", UInt16, ids3))).into_batches(),
        ];
        let result = reindex_logs(&mut batches);
        assert!(matches!(result, Err(Error::TooManyItems { .. })));
    }

    #[test]
    fn test_logs_u16_max_items() {
        let ids = (0..HALF_U16).collect::<Vec<_>>();
        let ids2 = (HALF_U16..u16::MAX).collect::<Vec<_>>();

        let batches = vec![
            logs!((Logs, ("id", UInt16, ids))),
            logs!((Logs, ("id", UInt16, ids2))),
        ];
        test_reindex_logs(&batches);
    }

    #[test]
    #[rustfmt::skip]
    fn test_logs_referential_integrity_violations() {
        // Referential integrity violations occur when a child has parent_ids that
        // don't exist in the parent's id column. These orphaned rows are removed
        // during reindexing.
        //
        // Three different violation positions:
        // - At the start (id 0 not in parent [1, 2, 4])
        // - In the middle (id 3 not in parent [1, 2, 4])
        // - At the end (id 5 not in parent [1, 2, 4])
        //
        // Valid children: ids 1, 2, 4 -> 3 valid rows out of 7.
        let parent_ids = vec![1u16, 2, 4];
        let child_ids  = vec![1u16, 0, 2, 2, 3, 3, 4, 5, 4];

        let parent_set: HashSet<u16> = parent_ids.iter().copied().collect();
        let expected_child_count = child_ids.iter().filter(|id| parent_set.contains(id)).count();

        let mut batches = vec![
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids.clone()))
            ).into_batches(),
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids.clone()))
            ).into_batches(),
        ];

        reindex_logs(&mut batches).unwrap();
        assert_no_id_overlaps::<Logs, { Logs::COUNT }>(&batches);

        // Verify orphaned rows were removed
        let child_idx = payload_to_idx(ArrowPayloadType::LogAttrs);
        for group in &batches {
            let child_batch = group[child_idx].as_ref().unwrap();
            assert_eq!(child_batch.num_rows(), expected_child_count);
        }
    }

    #[test]
    #[rustfmt::skip]
    fn test_logs_complex() {
        // Overlapping ranges, not in order, many to many child relations
        let parent_ids   = vec![0, 5, 3, 10, 7, 11];
        let parent_ids_2 = vec![2, 8, 5, 9, 11, 0];
        let child_ids    = vec![0, 10, 10, 10, 7, 7, 3, 11, 3, 11, 3, 11];
        let child_ids_2  = vec![2, 8, 8, 5, 9, 9, 0, 11, 11];

        // Log Attrs
        test_reindex_logs(&[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids_2.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids_2.clone()))
            ),
        ]);

        // ScopeAttrs
        test_reindex_logs(&[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("scope.id", UInt16, parent_ids.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids_2.clone()), ("scope.id", UInt16, parent_ids_2.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids_2.clone()))
            ),
        ]);

        // ResourceAttrs
        test_reindex_logs(&[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("resource.id", UInt16, parent_ids.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids_2.clone()), ("resource.id", UInt16, parent_ids_2.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids_2.clone()))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_logs_range_gaps() {
        // IDs with gaps between consecutive ranges
        let parent_ids   = vec![0, 1, 10, 11, 15, 16];
        let parent_ids_2 = vec![5, 6, 12, 13, 20, 21];
        let child_ids    = vec![0, 1, 10, 11, 15, 16];
        let child_ids_2  = vec![5, 6, 12, 13, 20, 21];

        // Log Attrs
        test_reindex_logs(&[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids_2.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids_2.clone()))
            ),
        ]);

        // ScopeAttrs
        test_reindex_logs(&[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("scope.id", UInt16, parent_ids.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids_2.clone()), ("scope.id", UInt16, parent_ids_2.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids_2.clone()))
            ),
        ]);

        // ResourceAttrs
        test_reindex_logs(&[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("resource.id", UInt16, parent_ids.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids_2.clone()), ("resource.id", UInt16, parent_ids_2.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids_2.clone()))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_logs_reindex_overlapping() {
        // Both batches use the same IDs, so reindexing must remap to avoid overlap
        let parent_ids   = vec![1, 0];
        let child_ids   = vec![0, 0, 1, 1];

        // LogAttrs
        test_reindex_logs(&[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]);

        // ScopeAttrs
        test_reindex_logs(&[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("scope.id", UInt16, parent_ids.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("scope.id", UInt16, parent_ids.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]);

        // ResourceAttrs
        test_reindex_logs(&[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("resource.id", UInt16, parent_ids.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("resource.id", UInt16, parent_ids.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_logs_reindex_noop() {
        // IDs are not overlapping at all
        let parent_ids   = vec![0, 2, 1, 3];
        let parent_ids_2 = vec![4, 6, 5, 7];

        let child_ids   = vec![1, 2, 2, 0, 3];
        let child_ids_2 = vec![6, 6, 5, 5, 7, 4];

        // LogAttrs
        test_reindex_logs(&[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids_2.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids_2.clone()))
            ),
        ]);

        // ScopeAttrs
        test_reindex_logs(&[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("scope.id", UInt16, parent_ids.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids_2.clone()), ("scope.id", UInt16, parent_ids_2.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids_2.clone()))
            ),
        ]);

        // ResourceAttrs
        test_reindex_logs(&[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("resource.id", UInt16, parent_ids.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids_2.clone()), ("resource.id", UInt16, parent_ids_2.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids_2.clone()))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_logs_greedy_all_offset() {
        // Checking scenario where we skip all compaction because 
        // all batches have gaps (span > len) but sum(span) fits
        // in U16 range. Every batch qualifies for the offset fast-path.
        //
        // Per batch: span=6, len=3, sum(span)=18, need_to_save < 0.

        // id
        test_reindex_logs(& [
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 2, 5])),
                (LogAttrs, ("parent_id", UInt16, vec![0u16, 2, 5, 2]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![10u16, 13, 15])),
                (LogAttrs, ("parent_id", UInt16, vec![10u16, 13, 15]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![20u16, 22, 25])),
                (LogAttrs, ("parent_id", UInt16, vec![20u16, 22, 25, 25]))
            ),
        ]);

        // resource.id
        test_reindex_logs(& [
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3, 4]),
                       ("resource.id", UInt16, vec![0u16, 0, 2, 5, 5])),
                (ResourceAttrs, ("parent_id", UInt16, vec![0u16, 2, 5]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3]),
                       ("resource.id", UInt16, vec![10u16, 10, 13, 15])),
                (ResourceAttrs, ("parent_id", UInt16, vec![10u16, 13, 15]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3]),
                       ("resource.id", UInt16, vec![20u16, 22, 25, 25])),
                (ResourceAttrs, ("parent_id", UInt16, vec![20u16, 22, 25]))
            ),
        ]);

        // scope.id
        test_reindex_logs(& [
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3, 4]),
                       ("scope.id", UInt16, vec![0u16, 0, 2, 5, 5])),
                (ScopeAttrs, ("parent_id", UInt16, vec![0u16, 2, 5]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3]),
                       ("scope.id", UInt16, vec![10u16, 10, 13, 15])),
                (ScopeAttrs, ("parent_id", UInt16, vec![10u16, 13, 15]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3]),
                       ("scope.id", UInt16, vec![20u16, 22, 25, 25])),
                (ScopeAttrs, ("parent_id", UInt16, vec![20u16, 22, 25]))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_logs_greedy_some_compact() {
        // Checking that we can handle partial compaction where some 
        // batches need to compact, but not all.
        //
        // Batch 0: span=40001, len=3
        // Batch 1: span=30001, len=3
        // Batch 2: span=6,     len=3
        // total_span=70008, need_to_save=4473. Batch 0 compacts (saves 39998).

        // id
        test_reindex_logs(& [
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 10000, 40000])),
                (LogAttrs, ("parent_id", UInt16, vec![0u16, 10000, 40000]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 10000, 30000])),
                (LogAttrs, ("parent_id", UInt16, vec![0u16, 10000, 30000]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 3, 5])),
                (LogAttrs, ("parent_id", UInt16, vec![0u16, 3, 5]))
            ),
        ]);

        // resource.id
        test_reindex_logs(& [
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3, 4]),
                       ("resource.id", UInt16, vec![0u16, 0, 10000, 40000, 40000])),
                (ResourceAttrs, ("parent_id", UInt16, vec![0u16, 10000, 40000]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3]),
                       ("resource.id", UInt16, vec![0u16, 0, 10000, 30000])),
                (ResourceAttrs, ("parent_id", UInt16, vec![0u16, 10000, 30000]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3]),
                       ("resource.id", UInt16, vec![0u16, 3, 5, 5])),
                (ResourceAttrs, ("parent_id", UInt16, vec![0u16, 3, 5]))
            ),
        ]);

        // scope.id
        test_reindex_logs(& [
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3, 4]),
                       ("scope.id", UInt16, vec![0u16, 0, 10000, 40000, 40000])),
                (ScopeAttrs, ("parent_id", UInt16, vec![0u16, 10000, 40000]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3]),
                       ("scope.id", UInt16, vec![0u16, 0, 10000, 30000])),
                (ScopeAttrs, ("parent_id", UInt16, vec![0u16, 10000, 30000]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3]),
                       ("scope.id", UInt16, vec![0u16, 3, 5, 5])),
                (ScopeAttrs, ("parent_id", UInt16, vec![0u16, 3, 5]))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_logs_greedy_all_compact() {
        // Checking that we can perfectly compact to the max u16
        // range when every batch needs compaction.
        //
        // Batch 0: ids = 0,2,4,...,65534 (32768 items, span=65535)
        // Batch 1: ids = 0,2,4,...,65532 (32767 items, span=65533)
        // total_span=131068, need_to_save=65533. Both compact.

        let even_ids_0: Vec<u16> = (0..32768).map(|i| i * 2).collect();
        let even_ids_1: Vec<u16> = (0..32767).map(|i| i * 2).collect();

        // id
        test_reindex_logs(& [
            logs!(
                (Logs, ("id", UInt16, even_ids_0.clone())),
                (LogAttrs, ("parent_id", UInt16, vec![0u16, 2, 4]))
            ),
            logs!(
                (Logs, ("id", UInt16, even_ids_1.clone())),
                (LogAttrs, ("parent_id", UInt16, vec![0u16, 2, 4]))
            ),
        ]);

        // resource.id
        let contiguous_ids_0: Vec<u16> = (0..32768).collect();
        let contiguous_ids_1: Vec<u16> = (0..32767).collect();

        test_reindex_logs(& [
            logs!(
                (Logs, ("id", UInt16, contiguous_ids_0.clone()),
                       ("resource.id", UInt16, even_ids_0.clone())),
                (ResourceAttrs, ("parent_id", UInt16, vec![0u16, 2, 4]))
            ),
            logs!(
                (Logs, ("id", UInt16, contiguous_ids_1.clone()),
                       ("resource.id", UInt16, even_ids_1.clone())),
                (ResourceAttrs, ("parent_id", UInt16, vec![0u16, 2, 4]))
            ),
        ]);

        // scope.id
        test_reindex_logs(& [
            logs!(
                (Logs, ("id", UInt16, contiguous_ids_0.clone()),
                       ("scope.id", UInt16, even_ids_0.clone())),
                (ScopeAttrs, ("parent_id", UInt16, vec![0u16, 2, 4]))
            ),
            logs!(
                (Logs, ("id", UInt16, contiguous_ids_1.clone()),
                       ("scope.id", UInt16, even_ids_1.clone())),
                (ScopeAttrs, ("parent_id", UInt16, vec![0u16, 2, 4]))
            ),
        ]);
    }

    #[test]
    fn test_logs_greedy_violation_no_overflow() {
        // Exactly 65535 total logs with referential integrity violations in
        // child tables. Each child table has more rows than its parent and
        // includes parent_ids outside the parent's [min, max] range.
        //
        // The children_in_parent_range check detects these violations via
        // min/max statistics and forces CompactOnly strategy. Without this,
        // the offset fast-path would overflow u16 when adding the offset to
        // out-of-range parent_ids (e.g. batch 1 offset=32768, child
        // parent_id=u16::MAX -> 32768 + 65535 overflows).

        let ids_0: Vec<u16> = (0..32768).collect();
        let ids_1: Vec<u16> = (0..32767).collect();

        // let mut child_pids_0: Vec<u16> = (0..32768).collect();
        let mut child_pids_0: Vec<u16> = (0..65535).collect();
        child_pids_0.push(u16::MAX);
        // let mut child_pids_1: Vec<u16> = (0..32767).collect();
        let mut child_pids_1: Vec<u16> = (0..65535).collect();
        child_pids_1.push(u16::MAX);

        // id
        test_reindex_logs(&[
            logs!(
                (Logs, ("id", UInt16, ids_0.clone())),
                (LogAttrs, ("parent_id", UInt16, child_pids_0.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, ids_1.clone())),
                (LogAttrs, ("parent_id", UInt16, child_pids_1.clone()))
            ),
        ]);

        test_reindex_logs(&[
            logs!(
                (
                    Logs,
                    ("id", UInt16, ids_0.clone()),
                    ("resource.id", UInt16, ids_0.clone())
                ),
                (ResourceAttrs, ("parent_id", UInt16, child_pids_0.clone()))
            ),
            logs!(
                (
                    Logs,
                    ("id", UInt16, ids_1.clone()),
                    ("resource.id", UInt16, ids_1.clone())
                ),
                (ResourceAttrs, ("parent_id", UInt16, child_pids_1.clone()))
            ),
        ]);

        // scope.id
        test_reindex_logs(&[
            logs!(
                (
                    Logs,
                    ("id", UInt16, ids_0.clone()),
                    ("scope.id", UInt16, ids_0.clone())
                ),
                (ScopeAttrs, ("parent_id", UInt16, child_pids_0.clone()))
            ),
            logs!(
                (
                    Logs,
                    ("id", UInt16, ids_1.clone()),
                    ("scope.id", UInt16, ids_1.clone())
                ),
                (ScopeAttrs, ("parent_id", UInt16, child_pids_1.clone()))
            ),
        ]);
    }

    // ---- Traces tests ----

    #[test]
    #[rustfmt::skip]
    fn test_traces_reindex_attrs() {
        // Test resource, scope, and span attrs reindexing with overlapping IDs
        let parent_ids = vec![1u16, 0];
        let child_ids  = vec![0u16, 0, 1, 1];

        // SpanAttrs
        test_reindex_traces(&[
            traces!(
                (Spans, ("id", UInt16, parent_ids.clone())),
                (SpanAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            traces!(
                (Spans, ("id", UInt16, parent_ids.clone())),
                (SpanAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]);

        // ScopeAttrs
        test_reindex_traces(&[
            traces!(
                (Spans, ("id", UInt16, parent_ids.clone()), ("scope.id", UInt16, parent_ids.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            traces!(
                (Spans, ("id", UInt16, parent_ids.clone()), ("scope.id", UInt16, parent_ids.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]);

        // ResourceAttrs
        test_reindex_traces(&[
            traces!(
                (Spans, ("id", UInt16, parent_ids.clone()), ("resource.id", UInt16, parent_ids.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            traces!(
                (Spans, ("id", UInt16, parent_ids.clone()), ("resource.id", UInt16, parent_ids.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]);
    }

    #[test]
    fn test_traces_greater_than_u16_max() {
        let ids = (0..HALF_U16).collect::<Vec<_>>();
        let ids2 = (HALF_U16..u16::MAX).collect::<Vec<_>>();
        let ids3 = vec![u16::MAX];

        let mut batches = vec![
            traces!((Spans, ("id", UInt16, ids))).into_batches(),
            traces!((Spans, ("id", UInt16, ids2))).into_batches(),
            traces!((Spans, ("id", UInt16, ids3))).into_batches(),
        ];
        let result = reindex_traces(&mut batches);
        assert!(matches!(result, Err(Error::TooManyItems { .. })));
    }

    #[test]
    fn test_traces_empty_batches() {
        let mut batches: Vec<[Option<RecordBatch>; Traces::COUNT]> = vec![];
        reindex_traces(&mut batches).unwrap();

        let batches = vec![traces!((Spans, ("id", UInt16, vec![0u16, 1])))];
        test_reindex_traces(&batches);
    }

    #[test]
    #[rustfmt::skip]
    fn test_traces_span_events_and_links() {
        // Test new two level relationships:
        //
        // Spans -> SpanEvents (parent_id UInt16, id UInt32)
        // Spans -> SpanLinks (parent_id UInt16, id UInt32)

        let span_ids        = vec![0u16, 1, 2];
        let event_pids      = vec![0u16, 0, 1, 2, 2];
        let link_pids       = vec![1u16, 2];
        let event_ids       = vec![0u32, 1, 2, 3, 4];
        let link_ids        = vec![0u32, 1];

        let span_ids_2      = vec![1u16, 3, 4];
        let event_pids_2    = vec![1u16, 3, 3, 4];
        let event_ids_2     = vec![0u32, 1, 2, 3];
        let link_pids_2     = vec![3u16, 4, 4];
        let link_ids_2      = vec![0u32, 1, 2];

        test_reindex_traces(&[
            traces!(
                (Spans, ("id", UInt16, span_ids)),
                (SpanEvents, ("id", UInt32, event_ids), ("parent_id", UInt16, event_pids)),
                (SpanLinks, ("id", UInt32, link_ids), ("parent_id", UInt16, link_pids))
            ),
            traces!(
                (Spans, ("id", UInt16, span_ids_2)),
                (SpanEvents, ("id", UInt32, event_ids_2), ("parent_id", UInt16, event_pids_2)),
                (SpanLinks, ("id", UInt32, link_ids_2), ("parent_id", UInt16, link_pids_2))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_traces_span_events_with_attrs() {
        // Three-level relationship: Spans -> SpanEvents -> SpanEventAttrs
        // SpanEvents.id is UInt32, SpanEventAttrs.parent_id is UInt32
        //
        // UInt32 id columns are always plain (not dictionary encoded).
        // UInt32 parent_id columns may be dictionary encoded.
        // We test encoding variants for parent_id columns:
        // 1. Plain UInt32 parent_ids
        // 2. Dict<UInt8, UInt32> parent_ids
        // 3. Dict<UInt16, UInt32> parent_ids
        // 4. Mixed encodings across parent_id columns and batches
        let span_ids     = vec![0u16, 1];
        let span_ids_2   = vec![2u16, 3];
        let event_pids   = vec![0u16, 0, 1];
        let event_pids_2 = vec![2u16, 3, 3];

        // Plain UInt32 parent_ids
        test_reindex_traces(&[
            traces!(
                (Spans, ("id", UInt16, span_ids.clone())),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, event_pids.clone())),
                (SpanEventAttrs, ("parent_id", UInt32, vec![0u32, 1, 1, 2]))
            ),
            traces!(
                (Spans, ("id", UInt16, span_ids_2.clone())),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, event_pids_2.clone())),
                (SpanEventAttrs, ("parent_id", UInt32, vec![0u32, 2, 2]))
            ),
        ]);

        // Dict<UInt8, UInt32> parent_ids
        test_reindex_traces(&[
            traces!(
                (Spans, ("id", UInt16, span_ids.clone())),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, event_pids.clone())),
                (SpanEventAttrs, ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 1, 2], vec![0u32, 1, 2])))
            ),
            traces!(
                (Spans, ("id", UInt16, span_ids_2.clone())),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, event_pids_2.clone())),
                (SpanEventAttrs, ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 1], vec![0u32, 2])))
            ),
        ]);

        // Dict<UInt16, UInt32> parent_ids
        test_reindex_traces(&[
            traces!(
                (Spans, ("id", UInt16, span_ids.clone())),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, event_pids.clone())),
                (SpanEventAttrs, ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 1, 2], vec![0u32, 1, 2])))
            ),
            traces!(
                (Spans, ("id", UInt16, span_ids_2.clone())),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, event_pids_2.clone())),
                (SpanEventAttrs, ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 1], vec![0u32, 2])))
            ),
        ]);

        // Mixed: Dict<UInt16, UInt32> event attr parent_ids in first batch,
        // Dict<UInt8, UInt32> event attr parent_ids in second batch
        test_reindex_traces(&[
            traces!(
                (Spans, ("id", UInt16, span_ids.clone())),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, event_pids.clone())),
                (SpanEventAttrs, ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 1, 2], vec![0u32, 1, 2])))
            ),
            traces!(
                (Spans, ("id", UInt16, span_ids_2.clone())),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, event_pids_2.clone())),
                (SpanEventAttrs, ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 1], vec![0u32, 2])))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_traces_complex() {
        // Complex case with spans, events, event attrs, links, and span attrs all at once
        let span_ids          = vec![0u16, 5, 3, 10];
        let span_ids_2        = vec![2u16, 7, 4, 12];
        let event_pids        = vec![0u16, 0, 3, 10, 10];
        let event_pids_2      = vec![2u16, 7, 7, 12];
        let event_ids         = vec![0u32, 1, 2, 3, 4];
        let event_ids_2       = vec![0u32, 1, 2, 3];
        let event_attr_pids   = vec![0u32, 1, 2, 3, 4, 4];
        let event_attr_pids_2 = vec![0u32, 1, 2, 3];
        let link_pids         = vec![5u16, 3];
        let link_pids_2       = vec![7u16, 4, 12];
        let link_ids          = vec![0u32, 1];
        let link_ids_2        = vec![0u32, 1, 2];
        let span_attr_pids    = vec![0u16, 5, 3, 10, 10];
        let span_attr_pids_2  = vec![2u16, 7, 12];

        test_reindex_traces(&[
            traces!(
                (Spans, ("id", UInt16, span_ids.clone())),
                (SpanEvents, ("id", UInt32, event_ids.clone()), ("parent_id", UInt16, event_pids.clone())),
                (SpanEventAttrs, ("parent_id", UInt32, event_attr_pids.clone())),
                (SpanLinks, ("id", UInt32, link_ids.clone()), ("parent_id", UInt16, link_pids.clone())),
                (SpanAttrs, ("parent_id", UInt16, span_attr_pids.clone()))
            ),
            traces!(
                (Spans, ("id", UInt16, span_ids_2.clone())),
                (SpanEvents, ("id", UInt32, event_ids_2.clone()), ("parent_id", UInt16, event_pids_2.clone())),
                (SpanEventAttrs, ("parent_id", UInt32, event_attr_pids_2.clone())),
                (SpanLinks, ("id", UInt32, link_ids_2.clone()), ("parent_id", UInt16, link_pids_2.clone())),
                (SpanAttrs, ("parent_id", UInt16, span_attr_pids_2.clone()))
            ),
        ]);
    }

    // ---- Metrics tests ----

    #[test]
    #[rustfmt::skip]
    fn test_metrics_reindex_attrs() {
        // Test resource, scope, and metric attrs reindexing with overlapping IDs
        let parent_ids = vec![1u16, 0];
        let child_ids  = vec![0u16, 0, 1, 1];

        // MetricAttrs
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, parent_ids.clone())),
                (MetricAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, parent_ids.clone())),
                (MetricAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]);

        // ScopeAttrs
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, parent_ids.clone()), ("scope.id", UInt16, parent_ids.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, parent_ids.clone()), ("scope.id", UInt16, parent_ids.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]);

        // ResourceAttrs
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, parent_ids.clone()), ("resource.id", UInt16, parent_ids.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, parent_ids.clone()), ("resource.id", UInt16, parent_ids.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_metrics_data_points() {
        // Test Metrics.id -> DataPoints.parent_id for each data point type
        // with overlapping IDs across batches
        let metric_ids    = vec![0u16, 1, 2];
        let metric_ids_2  = vec![1u16, 3, 4];
        let dp_pids       = vec![0u16, 0, 1, 2, 2];
        let dp_pids_2     = vec![1u16, 3, 3, 4];
        let dp_ids        = vec![0u32, 1, 2, 3, 4];
        let dp_ids_2      = vec![0u32, 1, 2, 3];

        // NumberDataPoints
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (NumberDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (NumberDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone()))
            ),
        ]);

        // SummaryDataPoints
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (SummaryDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (SummaryDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone()))
            ),
        ]);

        // HistogramDataPoints
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (HistogramDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (HistogramDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone()))
            ),
        ]);

        // ExpHistogramDataPoints
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (ExpHistogramDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (ExpHistogramDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone()))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_metrics_dp_with_attrs() {
        // Three-level relationship: Metrics -> DataPoints -> DpAttrs
        let metric_ids     = vec![0u16, 1];
        let metric_ids_2   = vec![2u16, 3];
        let dp_pids        = vec![0u16, 0, 1];
        let dp_pids_2      = vec![2u16, 3, 3];
        let dp_ids         = vec![0u32, 1, 2];
        let dp_ids_2       = vec![0u32, 1, 2];
        let dp_attr_pids   = vec![0u32, 1, 1, 2];
        let dp_attr_pids_2 = vec![0u32, 2, 2];

        // NumberDataPoints -> NumberDpAttrs
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (NumberDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (NumberDpAttrs, ("parent_id", UInt32, dp_attr_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (NumberDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (NumberDpAttrs, ("parent_id", UInt32, dp_attr_pids_2.clone()))
            ),
        ]);

        // SummaryDataPoints -> SummaryDpAttrs
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (SummaryDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (SummaryDpAttrs, ("parent_id", UInt32, dp_attr_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (SummaryDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (SummaryDpAttrs, ("parent_id", UInt32, dp_attr_pids_2.clone()))
            ),
        ]);

        // HistogramDataPoints -> HistogramDpAttrs
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (HistogramDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (HistogramDpAttrs, ("parent_id", UInt32, dp_attr_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (HistogramDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (HistogramDpAttrs, ("parent_id", UInt32, dp_attr_pids_2.clone()))
            ),
        ]);

        // ExpHistogramDataPoints -> ExpHistogramDpAttrs
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (ExpHistogramDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (ExpHistogramDpAttrs, ("parent_id", UInt32, dp_attr_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (ExpHistogramDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (ExpHistogramDpAttrs, ("parent_id", UInt32, dp_attr_pids_2.clone()))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_metrics_dp_with_exemplars() {
        // Three-level relationship: Metrics -> DataPoints -> Exemplars
        // (Summary does not have exemplars)
        let metric_ids      = vec![0u16, 1];
        let metric_ids_2    = vec![2u16, 3];
        let dp_pids         = vec![0u16, 0, 1];
        let dp_pids_2       = vec![2u16, 3, 3];
        let dp_ids          = vec![0u32, 1, 2];
        let dp_ids_2        = vec![0u32, 1, 2];
        let exemplar_pids   = vec![0u32, 1, 1, 2];
        let exemplar_pids_2 = vec![0u32, 2, 2];
        let exemplar_ids    = vec![0u32, 1, 2, 3];
        let exemplar_ids_2  = vec![0u32, 1, 2];

        // NumberDataPoints -> NumberDpExemplars
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (NumberDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (NumberDpExemplars, ("id", UInt32, exemplar_ids.clone()), ("parent_id", UInt32, exemplar_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (NumberDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (NumberDpExemplars, ("id", UInt32, exemplar_ids_2.clone()), ("parent_id", UInt32, exemplar_pids_2.clone()))
            ),
        ]);

        // HistogramDataPoints -> HistogramDpExemplars
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (HistogramDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (HistogramDpExemplars, ("id", UInt32, exemplar_ids.clone()), ("parent_id", UInt32, exemplar_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (HistogramDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (HistogramDpExemplars, ("id", UInt32, exemplar_ids_2.clone()), ("parent_id", UInt32, exemplar_pids_2.clone()))
            ),
        ]);

        // ExpHistogramDataPoints -> ExpHistogramDpExemplars
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (ExpHistogramDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (ExpHistogramDpExemplars, ("id", UInt32, exemplar_ids.clone()), ("parent_id", UInt32, exemplar_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (ExpHistogramDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (ExpHistogramDpExemplars, ("id", UInt32, exemplar_ids_2.clone()), ("parent_id", UInt32, exemplar_pids_2.clone()))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_metrics_exemplar_attrs() {
        // Four-level relationship: Metrics -> DataPoints -> Exemplars -> ExemplarAttrs
        let metric_ids          = vec![0u16, 1];
        let metric_ids_2        = vec![2u16, 3];
        let dp_pids             = vec![0u16, 0, 1];
        let dp_pids_2           = vec![2u16, 3, 3];
        let dp_ids              = vec![0u32, 1, 2];
        let dp_ids_2            = vec![0u32, 1, 2];
        let exemplar_pids       = vec![0u32, 1, 2];
        let exemplar_pids_2     = vec![0u32, 1, 2];
        let exemplar_ids        = vec![0u32, 1, 2];
        let exemplar_ids_2      = vec![0u32, 1, 2];
        let exemplar_attr_pids  = vec![0u32, 1, 1, 2];
        let exemplar_attr_pids_2 = vec![0u32, 2, 2];

        // NumberDataPoints -> NumberDpExemplars -> NumberDpExemplarAttrs
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (NumberDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (NumberDpExemplars, ("id", UInt32, exemplar_ids.clone()), ("parent_id", UInt32, exemplar_pids.clone())),
                (NumberDpExemplarAttrs, ("parent_id", UInt32, exemplar_attr_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (NumberDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (NumberDpExemplars, ("id", UInt32, exemplar_ids_2.clone()), ("parent_id", UInt32, exemplar_pids_2.clone())),
                (NumberDpExemplarAttrs, ("parent_id", UInt32, exemplar_attr_pids_2.clone()))
            ),
        ]);

        // HistogramDataPoints -> HistogramDpExemplars -> HistogramDpExemplarAttrs
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (HistogramDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (HistogramDpExemplars, ("id", UInt32, exemplar_ids.clone()), ("parent_id", UInt32, exemplar_pids.clone())),
                (HistogramDpExemplarAttrs, ("parent_id", UInt32, exemplar_attr_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (HistogramDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (HistogramDpExemplars, ("id", UInt32, exemplar_ids_2.clone()), ("parent_id", UInt32, exemplar_pids_2.clone())),
                (HistogramDpExemplarAttrs, ("parent_id", UInt32, exemplar_attr_pids_2.clone()))
            ),
        ]);

        // ExpHistogramDataPoints -> ExpHistogramDpExemplars -> ExpHistogramDpExemplarAttrs
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (ExpHistogramDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (ExpHistogramDpExemplars, ("id", UInt32, exemplar_ids.clone()), ("parent_id", UInt32, exemplar_pids.clone())),
                (ExpHistogramDpExemplarAttrs, ("parent_id", UInt32, exemplar_attr_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (ExpHistogramDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (ExpHistogramDpExemplars, ("id", UInt32, exemplar_ids_2.clone()), ("parent_id", UInt32, exemplar_pids_2.clone())),
                (ExpHistogramDpExemplarAttrs, ("parent_id", UInt32, exemplar_attr_pids_2.clone()))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_metrics_number_dp_chain_with_dicts() {
        // Four-level chain: Metrics -> NumberDataPoints -> NumberDpExemplars -> NumberDpExemplarAttrs
        //
        // UInt32 id columns are always plain (not dictionary encoded).
        // UInt32 parent_id columns may be dictionary encoded.
        // Tests dictionary encoding variants for parent_id columns:
        // 1. Dict<UInt8, UInt32> parent_ids
        // 2. Dict<UInt16, UInt32> parent_ids
        // 3. Mixed encodings across parent_id columns and batches
        let metric_ids           = vec![0u16, 1];
        let metric_ids_2         = vec![2u16, 3];
        let dp_pids              = vec![0u16, 0, 1];
        let dp_pids_2            = vec![2u16, 3, 3];

        // Dict<UInt8, UInt32> for all UInt32 parent_id columns
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (NumberDataPoints, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, dp_pids.clone())),
                (NumberDpExemplars, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 2], vec![0u32, 1, 2]))),
                (NumberDpExemplarAttrs, ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 1, 2], vec![0u32, 1, 2])))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (NumberDataPoints, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, dp_pids_2.clone())),
                (NumberDpExemplars, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 2], vec![0u32, 1, 2]))),
                (NumberDpExemplarAttrs, ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 1], vec![0u32, 2])))
            ),
        ]);

        // Dict<UInt16, UInt32> for all UInt32 parent_id columns
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (NumberDataPoints, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, dp_pids.clone())),
                (NumberDpExemplars, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 2], vec![0u32, 1, 2]))),
                (NumberDpExemplarAttrs, ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 1, 2], vec![0u32, 1, 2])))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (NumberDataPoints, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, dp_pids_2.clone())),
                (NumberDpExemplars, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 2], vec![0u32, 1, 2]))),
                (NumberDpExemplarAttrs, ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 1], vec![0u32, 2])))
            ),
        ]);

        // Mixed: Dict<UInt16, UInt32> exemplar parent_ids and exemplar attr parent_ids in first batch,
        // Dict<UInt8, UInt32> exemplar attr parent_ids in second batch
        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (NumberDataPoints, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, dp_pids.clone())),
                (NumberDpExemplars, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 2], vec![0u32, 1, 2]))),
                (NumberDpExemplarAttrs, ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 1, 2], vec![0u32, 1, 2])))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (NumberDataPoints, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, dp_pids_2.clone())),
                (NumberDpExemplars, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt32, vec![0u32, 1, 2])),
                (NumberDpExemplarAttrs, ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 1], vec![0u32, 2])))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_metrics_complex() {
        // Complex case: multiple DP types + attrs + exemplars + exemplar attrs
        let metric_ids            = vec![0u16, 5, 3, 10];
        let metric_ids_2          = vec![2u16, 7, 4, 12];

        // NumberDataPoints chain
        let num_dp_pids           = vec![0u16, 0, 3];
        let num_dp_pids_2         = vec![2u16, 7, 7];
        let num_dp_ids            = vec![0u32, 1, 2];
        let num_dp_ids_2          = vec![0u32, 1, 2];
        let num_dp_attr_pids      = vec![0u32, 1, 2];
        let num_dp_attr_pids_2    = vec![0u32, 1, 2];
        let num_exemplar_pids     = vec![0u32, 1];
        let num_exemplar_pids_2   = vec![0u32, 2];
        let num_exemplar_ids      = vec![0u32, 1];
        let num_exemplar_ids_2    = vec![0u32, 1];
        let num_ex_attr_pids      = vec![0u32, 1];
        let num_ex_attr_pids_2    = vec![0u32, 1];

        // HistogramDataPoints chain
        let hist_dp_pids          = vec![5u16, 10, 10];
        let hist_dp_pids_2        = vec![4u16, 12, 12];
        let hist_dp_ids           = vec![0u32, 1, 2];
        let hist_dp_ids_2         = vec![0u32, 1, 2];
        let hist_exemplar_pids    = vec![0u32, 2];
        let hist_exemplar_pids_2  = vec![1u32, 2];
        let hist_exemplar_ids     = vec![0u32, 1];
        let hist_exemplar_ids_2   = vec![0u32, 1];

        // MetricAttrs
        let metric_attr_pids      = vec![0u16, 5, 3, 10];
        let metric_attr_pids_2    = vec![2u16, 7, 12];

        test_reindex_metrics(&[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (NumberDataPoints, ("id", UInt32, num_dp_ids.clone()), ("parent_id", UInt16, num_dp_pids.clone())),
                (NumberDpAttrs, ("parent_id", UInt32, num_dp_attr_pids.clone())),
                (NumberDpExemplars, ("id", UInt32, num_exemplar_ids.clone()), ("parent_id", UInt32, num_exemplar_pids.clone())),
                (NumberDpExemplarAttrs, ("parent_id", UInt32, num_ex_attr_pids.clone())),
                (HistogramDataPoints, ("id", UInt32, hist_dp_ids.clone()), ("parent_id", UInt16, hist_dp_pids.clone())),
                (HistogramDpExemplars, ("id", UInt32, hist_exemplar_ids.clone()), ("parent_id", UInt32, hist_exemplar_pids.clone())),
                (MetricAttrs, ("parent_id", UInt16, metric_attr_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (NumberDataPoints, ("id", UInt32, num_dp_ids_2.clone()), ("parent_id", UInt16, num_dp_pids_2.clone())),
                (NumberDpAttrs, ("parent_id", UInt32, num_dp_attr_pids_2.clone())),
                (NumberDpExemplars, ("id", UInt32, num_exemplar_ids_2.clone()), ("parent_id", UInt32, num_exemplar_pids_2.clone())),
                (NumberDpExemplarAttrs, ("parent_id", UInt32, num_ex_attr_pids_2.clone())),
                (HistogramDataPoints, ("id", UInt32, hist_dp_ids_2.clone()), ("parent_id", UInt16, hist_dp_pids_2.clone())),
                (HistogramDpExemplars, ("id", UInt32, hist_exemplar_ids_2.clone()), ("parent_id", UInt32, hist_exemplar_pids_2.clone())),
                (MetricAttrs, ("parent_id", UInt16, metric_attr_pids_2.clone()))
            ),
        ]);
    }

    // ---- Transport optimized encoding tests ----

    #[test]
    #[rustfmt::skip]
    fn test_logs_transport_optimized() {
        let batches = vec![
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3])),
                (LogAttrs, ("parent_id", UInt16, vec![1u16, 2, 2, 0, 3, 3, 2, 2]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3])),
                (LogAttrs, ("parent_id", UInt16, vec![0u16, 1, 3, 3, 2, 2, 1, 1, 2, 2, 0, 0]))
            ),
        ];
        test_reindex_transport_optimized_logs(& batches);
    }

    #[test]
    #[rustfmt::skip]
    fn test_traces_transport_optimized() {
        let batches = vec![
            traces!(
                (Spans, ("id", UInt16, vec![0u16, 1, 2])),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, vec![0u16, 0, 1])),
                (SpanEventAttrs, ("parent_id", UInt32, vec![0u32, 1, 1, 2])),
                (SpanLinks, ("id", UInt32, vec![0u32, 1]), ("parent_id", UInt16, vec![1u16, 2])),
                (SpanAttrs, ("parent_id", UInt16, vec![0u16, 1, 2]))
            ),
            traces!(
                (Spans, ("id", UInt16, vec![0u16, 1, 2])),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, vec![1u16, 2, 2])),
                (SpanEventAttrs, ("parent_id", UInt32, vec![0u32, 2, 2])),
                (SpanLinks, ("id", UInt32, vec![0u32, 1]), ("parent_id", UInt16, vec![0u16, 1])),
                (SpanAttrs, ("parent_id", UInt16, vec![0u16, 1, 2]))
            ),
        ];
        test_reindex_transport_optimized_traces(& batches);
    }

    #[test]
    #[rustfmt::skip]
    fn test_metrics_transport_optimized() {
        let batches = vec![
            metrics!(
                (UnivariateMetrics, ("id", UInt16, vec![0u16, 1])),
                (NumberDataPoints, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, vec![0u16, 0, 1])),
                (NumberDpAttrs, ("parent_id", UInt32, vec![0u32, 1, 2])),
                (NumberDpExemplars, ("id", UInt32, vec![0u32, 1]), ("parent_id", UInt32, vec![0u32, 2])),
                (NumberDpExemplarAttrs, ("parent_id", UInt32, vec![0u32, 1])),
                (MetricAttrs, ("parent_id", UInt16, vec![0u16, 0, 1]))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, vec![0u16, 1])),
                (NumberDataPoints, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, vec![0u16, 1, 1])),
                (NumberDpAttrs, ("parent_id", UInt32, vec![0u32, 1])),
                (NumberDpExemplars, ("id", UInt32, vec![0u32, 1]), ("parent_id", UInt32, vec![1u32, 2])),
                (NumberDpExemplarAttrs, ("parent_id", UInt32, vec![0u32, 1])),
                (MetricAttrs, ("parent_id", UInt16, vec![0u16, 1]))
            ),
        ];
        test_reindex_transport_optimized_metrics(& batches);
    }

    // ---- Primary ID bounds tests ----

    #[test]
    fn test_logs_u16_primary_id_bounds() {
        test_u16_primary_id_bounds::<Logs, { Logs::COUNT }>();
    }

    #[test]
    fn test_traces_u16_primary_id_bounds() {
        test_u16_primary_id_bounds::<Traces, { Traces::COUNT }>();
    }

    #[test]
    fn test_metrics_u16_primary_id_bounds() {
        test_u16_primary_id_bounds::<Metrics, { Metrics::COUNT }>();
    }

    /// Tests the overflow bounds for every U16 primary id column in a batch store.
    /// Currently we're not testing u32 because it's a lot of memory to do that.
    ///
    /// For each U16 payload type, verifies that u16::MAX total rows succeeds
    /// and u16::MAX + 1 fails with TooManyItems.
    fn test_u16_primary_id_bounds<S: OtapBatchStore, const N: usize>() {
        let reindex_fn = |b: &mut [[Option<RecordBatch>; N]]| plan_ids::<S, N>(b).map(|_| ());
        for &payload_type in S::allowed_payload_types() {
            let info = payload_relations(payload_type);
            let Some(id_info) = info.primary_id else {
                continue;
            };
            if matches!(id_info.size, IdColumnType::U32) {
                continue;
            }

            // Exactly u16::MAX rows split across two batches should succeed
            let half = (u16::MAX / 2) as usize;
            let other_half = u16::MAX as usize - half;
            let idx = payload_to_idx(payload_type);

            let mut ok_batches: Vec<[Option<RecordBatch>; N]> =
                vec![std::array::from_fn(|_| None), std::array::from_fn(|_| None)];
            ok_batches[0][idx] = Some(make_u16_id_batch::<S>(payload_type, half));
            ok_batches[1][idx] = Some(make_u16_id_batch::<S>(payload_type, other_half));
            reindex_fn(&mut ok_batches).unwrap_or_else(|e| {
                panic!(
                    "{:?}: u16::MAX rows should succeed but got: {}",
                    payload_type, e
                )
            });

            // u16::MAX + 1 rows should fail
            let mut fail_batches: Vec<[Option<RecordBatch>; N]> =
                vec![std::array::from_fn(|_| None), std::array::from_fn(|_| None)];
            fail_batches[0][idx] = Some(make_u16_id_batch::<S>(payload_type, half));
            fail_batches[1][idx] = Some(make_u16_id_batch::<S>(payload_type, other_half + 1));
            assert!(
                matches!(
                    reindex_fn(&mut fail_batches),
                    Err(Error::TooManyItems { .. })
                ),
                "{:?}: u16::MAX + 1 rows should fail with TooManyItems",
                payload_type,
            );
        }
    }

    /// Creates a minimal batch for a U16 primary id payload type containing an
    /// "id" column with `count` rows. Non-root types also get a "parent_id"
    /// column whose type is determined by looking up the parent's primary id
    /// size via `find_parent_id_size`.
    fn make_u16_id_batch<S: OtapBatchStore>(
        payload_type: ArrowPayloadType,
        count: usize,
    ) -> RecordBatch {
        let ids: Vec<u16> = (0..count as u16).collect();
        let parent_id_size = find_parent_id_size::<S>(payload_type);
        let batch = match parent_id_size {
            None => record_batch!(("id", UInt16, ids)).unwrap(),
            Some(IdColumnType::U16) => {
                let pids = vec![0u16; count];
                record_batch!(("id", UInt16, ids), ("parent_id", UInt16, pids)).unwrap()
            }
            Some(IdColumnType::U32) => {
                let pids = vec![0u32; count];
                record_batch!(("id", UInt16, ids), ("parent_id", UInt32, pids)).unwrap()
            }
        };
        crate::otap::testing::mark_id_columns_plain(batch)
    }

    #[test]
    fn test_logs_wrong_id_size() {
        let mut batches = vec![
            logs!((Logs, ("id", UInt32, vec![0u32, 1]))).into_batches(),
            logs!((Logs, ("id", UInt32, vec![0u32, 1]))).into_batches(),
        ];
        let result = reindex_logs(&mut batches);
        assert!(
            matches!(result, Err(Error::ColumnDataTypeMismatch { .. })),
            "expected ColumnDataTypeMismatch, got: {:?}",
            result,
        );
    }

    #[test]
    fn test_traces_span_events_wrong_id_size() {
        let mut batches = vec![
            traces!(
                (Spans, ("id", UInt16, vec![0u16, 1])),
                (
                    SpanEvents,
                    ("id", UInt16, vec![0u16, 1]),
                    ("parent_id", UInt16, vec![0u16, 1])
                )
            )
            .into_batches(),
            traces!(
                (Spans, ("id", UInt16, vec![0u16, 1])),
                (
                    SpanEvents,
                    ("id", UInt16, vec![0u16, 1]),
                    ("parent_id", UInt16, vec![0u16, 1])
                )
            )
            .into_batches(),
        ];
        let result = reindex_traces(&mut batches);
        assert!(
            matches!(result, Err(Error::ColumnDataTypeMismatch { .. })),
            "expected ColumnDataTypeMismatch, got: {:?}",
            result,
        );
    }

    // ---- Min/max tests ----

    /// Scenario: single-pass min/max over native and dictionary ID columns that
    /// are empty, all null, null-free, or mixed with nulls whose underlying
    /// slot values lie outside the valid range.
    /// Guarantees: returns `None` for empty/all-null columns, otherwise the
    /// min and max of the valid values only (null slots are ignored), and for
    /// dictionaries the min/max of the values array.
    #[test]
    fn test_id_column_min_max() {
        use arrow::array::{DictionaryArray, UInt8Array, UInt16Array, UInt32Array};
        use arrow::buffer::{NullBuffer, ScalarBuffer};
        use std::sync::Arc;

        let empty = UInt16Array::from(Vec::<u16>::new());
        assert_eq!(id_column_min_max::<UInt16Type>(&empty).unwrap(), None);

        let all_null = UInt16Array::from(vec![None, None]);
        assert_eq!(id_column_min_max::<UInt16Type>(&all_null).unwrap(), None);

        let plain = UInt16Array::from(vec![7u16, 3, 9, 4]);
        assert_eq!(
            id_column_min_max::<UInt16Type>(&plain).unwrap(),
            Some((3, 9))
        );

        // Null slots hold 0 and u16::MAX, which must not affect the result.
        let mixed = UInt16Array::new(
            ScalarBuffer::from(vec![0u16, 5, u16::MAX, 8]),
            Some(NullBuffer::from(vec![false, true, false, true])),
        );
        assert_eq!(
            id_column_min_max::<UInt16Type>(&mixed).unwrap(),
            Some((5, 8))
        );

        // Sliced so the null buffer has a non-zero offset.
        let sliced = mixed.slice(1, 3);
        assert_eq!(
            id_column_min_max::<UInt16Type>(&sliced).unwrap(),
            Some((5, 8))
        );

        let dict = DictionaryArray::<UInt8Type>::new(
            UInt8Array::from(vec![0u8, 1]),
            Arc::new(UInt32Array::from(vec![40u32, 2, 17])),
        );
        assert_eq!(
            id_column_min_max::<UInt32Type>(&dict).unwrap(),
            Some((2, 40))
        );

        // Lengths spanning full lane chunks plus remainders, with extremes in
        // the chunked body and in the remainder.
        for len in [1usize, 31, 32, 33, 64, 100, 1000] {
            let mut v: Vec<u32> = (0..len as u32).map(|i| 500 + (i * 7) % 97).collect();
            v[len / 2] = 3;
            v[len - 1] = 9000;
            let expected = (*v.iter().min().unwrap(), *v.iter().max().unwrap());
            let arr = UInt32Array::from(v);
            assert_eq!(
                id_column_min_max::<UInt32Type>(&arr).unwrap(),
                Some(expected),
                "len {len}"
            );
        }
    }

    // ---- Null ID tests ----

    /// Build a Logs store whose root `id` (and optionally `resource.id`) columns
    /// contain nulls. The value stored under each null slot is 0, mirroring what
    /// Arrow builders produce.
    fn logs_with_null_ids(
        ids: Vec<Option<u16>>,
        resource_ids: Option<Vec<Option<u16>>>,
        attr_parent_ids: Vec<u16>,
        resource_attr_parent_ids: Vec<u16>,
    ) -> Logs {
        use arrow::array::{ArrayRef, UInt16Array};
        use arrow::datatypes::{Field, Schema};
        use std::sync::Arc;

        let mut fields = vec![Field::new("id", DataType::UInt16, true)];
        let mut cols: Vec<ArrayRef> = vec![Arc::new(UInt16Array::from(ids))];
        if let Some(rids) = resource_ids {
            fields.push(Field::new("resource.id", DataType::UInt16, true));
            cols.push(Arc::new(UInt16Array::from(rids)));
        }
        let root = RecordBatch::try_new(Arc::new(Schema::new(fields)), cols).unwrap();

        let mut inputs = vec![
            (ArrowPayloadType::Logs, root),
            (
                ArrowPayloadType::LogAttrs,
                record_batch!(("parent_id", UInt16, attr_parent_ids)).unwrap(),
            ),
        ];
        if !resource_attr_parent_ids.is_empty() {
            inputs.push((
                ArrowPayloadType::ResourceAttrs,
                record_batch!(("parent_id", UInt16, resource_attr_parent_ids)).unwrap(),
            ));
        }
        crate::otap::testing::make_test_batch::<Logs, { Logs::COUNT }>(inputs)
    }

    fn column_at(rb: &RecordBatch, path: &str) -> arrow::array::ArrayRef {
        extract_id_column(rb, path).unwrap()
    }

    /// Scenario: the second input's root `id` column has a null slot (holding
    /// 0) and a minimum id above the running offset, so the offset delta is
    /// negative. The old reindex subtracted the offset from the null slot's 0
    /// and underflowed (panic in debug builds).
    /// Guarantees: fused reindex + concatenate does not panic on null ID
    /// slots, the null stays null at the same output row, valid ids remain
    /// unique, and the data is OTLP-equivalent to the inputs.
    #[test]
    fn test_null_id_offset_underflow() {
        let a = logs_with_null_ids(vec![Some(0), Some(1)], None, vec![0, 1], vec![]);
        let b = logs_with_null_ids(vec![Some(5), None, Some(6)], None, vec![5, 6], vec![]);

        let expected: Vec<_> = [a.clone(), b.clone()]
            .into_iter()
            .map(|s| otap_to_otlp(&s.into()))
            .collect();
        let mut batches = vec![a.into_batches(), b.into_batches()];
        let out = concatenate::<{ Logs::COUNT }>(&mut batches, ConcatOptions::reindex()).unwrap();

        let root = out[payload_to_idx(ArrowPayloadType::Logs)]
            .as_ref()
            .unwrap();
        let ids = column_at(root, ID);
        assert_eq!(ids.null_count(), 1);
        assert!(ids.is_null(3));
        let ids = ids.as_primitive::<UInt16Type>();
        let valid: Vec<u16> = ids.iter().flatten().collect();
        let unique: HashSet<u16> = valid.iter().copied().collect();
        assert_eq!(valid.len(), unique.len());

        let otlp = otap_to_otlp(&batches_to_otap::<Logs, { Logs::COUNT }>(&out));
        assert_equivalent(&expected, &[otlp]);
    }

    /// Scenario: null root `id` slots on both the offset path and the
    /// compaction path (forced by a child parent_id outside the parent range).
    /// Guarantees: the null bitmap of the id column is preserved in the output
    /// on both paths (the old reindex dropped it), and orphaned children that
    /// would escape the input's id range are redacted.
    #[test]
    fn test_null_id_bitmap_preserved() {
        // Offset path.
        let a = logs_with_null_ids(vec![None, Some(1), Some(2)], None, vec![1, 2], vec![]);
        let b = logs_with_null_ids(vec![Some(0), None, Some(1)], None, vec![0, 1], vec![]);
        let mut batches = vec![a.into_batches(), b.into_batches()];
        let out = concatenate::<{ Logs::COUNT }>(&mut batches, ConcatOptions::reindex()).unwrap();
        let root = out[payload_to_idx(ArrowPayloadType::Logs)]
            .as_ref()
            .unwrap();
        let ids = column_at(root, ID);
        assert_eq!(ids.null_count(), 2);
        assert!(ids.is_null(0) && ids.is_null(4));

        // Compaction path: LogAttrs parent_id 9 is outside [1, 2].
        let a = logs_with_null_ids(vec![None, Some(1), Some(2)], None, vec![1, 2, 9], vec![]);
        let b = logs_with_null_ids(vec![Some(0), None, Some(1)], None, vec![0, 1], vec![]);
        let mut batches = vec![a.into_batches(), b.into_batches()];
        let out = concatenate::<{ Logs::COUNT }>(&mut batches, ConcatOptions::reindex()).unwrap();
        let root = out[payload_to_idx(ArrowPayloadType::Logs)]
            .as_ref()
            .unwrap();
        let ids = column_at(root, ID);
        assert_eq!(ids.null_count(), 2);
        assert!(ids.is_null(0) && ids.is_null(4));
        let valid: Vec<u16> = ids.as_primitive::<UInt16Type>().iter().flatten().collect();
        assert_eq!(valid.len(), valid.iter().collect::<HashSet<_>>().len());

        let attrs = out[payload_to_idx(ArrowPayloadType::LogAttrs)]
            .as_ref()
            .unwrap();
        assert_eq!(attrs.num_rows(), 4, "orphaned parent_id 9 must be redacted");
        let pids: HashSet<u16> = column_at(attrs, PARENT_ID)
            .as_primitive::<UInt16Type>()
            .values()
            .iter()
            .copied()
            .collect();
        let idset: HashSet<u16> = valid.into_iter().collect();
        assert!(pids.is_subset(&idset));
    }

    /// Scenario: the nullable `resource.id` struct child has nulls on both the
    /// offset path and the compaction path (forced by an out-of-range
    /// ResourceAttrs parent_id), with the null-slot value below the offset.
    /// Guarantees: no panic, the `resource.id` null bitmap is preserved, and
    /// valid resource ids from different inputs do not collide.
    #[test]
    fn test_null_resource_id() {
        for resource_attr_pids in [vec![3u16, 4], vec![3u16, 4, 20]] {
            let a = logs_with_null_ids(
                vec![Some(0), Some(1)],
                Some(vec![Some(0), Some(1)]),
                vec![0],
                vec![0, 1],
            );
            let b = logs_with_null_ids(
                vec![Some(0), Some(1), Some(2)],
                Some(vec![Some(3), None, Some(4)]),
                vec![0],
                resource_attr_pids.clone(),
            );
            let mut batches = vec![a.into_batches(), b.into_batches()];
            let out =
                concatenate::<{ Logs::COUNT }>(&mut batches, ConcatOptions::reindex()).unwrap();
            let root = out[payload_to_idx(ArrowPayloadType::Logs)]
                .as_ref()
                .unwrap();
            let rids = column_at(root, "resource.id");
            assert_eq!(rids.null_count(), 1, "{resource_attr_pids:?}");
            assert!(rids.is_null(3));
            let rids = rids.as_primitive::<UInt16Type>();
            let first: HashSet<u16> = rids.slice(0, 2).iter().flatten().collect();
            let second: HashSet<u16> = rids.slice(2, 3).iter().flatten().collect();
            assert!(first.is_disjoint(&second));
        }
    }

    /// Scenario: a dictionary-encoded u32 parent_id column (SpanEventAttrs)
    /// references a SpanEvents id that does not exist and lies outside the
    /// parent range, forcing compaction.
    /// Guarantees: exactly the violating rows are redacted and the surviving
    /// rows keep their relation to their parent.
    #[test]
    #[rustfmt::skip]
    fn test_dict_parent_id_redaction() {
        let make = |attr_keys: Vec<u8>| traces!(
            (Spans, ("id", UInt16, vec![0u16])),
            (SpanEvents,
                ("id", UInt32, vec![0u32, 1]),
                ("parent_id", UInt16, vec![0u16, 0])),
            (SpanEventAttrs,
                ("parent_id", (UInt8, UInt32), (attr_keys, vec![0u32, 1, 7])))
        );
        let a = make(vec![0, 1]);
        let b = make(vec![0, 2, 1, 2]);
        let mut batches = vec![a.into_batches(), b.into_batches()];
        let out = concatenate::<{ Traces::COUNT }>(&mut batches, ConcatOptions::reindex()).unwrap();

        let attrs = out[payload_to_idx(ArrowPayloadType::SpanEventAttrs)].as_ref().unwrap();
        assert_eq!(attrs.num_rows(), 4, "two rows referencing id 7 are redacted");
        let events = out[payload_to_idx(ArrowPayloadType::SpanEvents)].as_ref().unwrap();
        let event_ids: HashSet<u32> =
            column_at(events, ID).as_primitive::<UInt32Type>().values().iter().copied().collect();
        assert_eq!(event_ids.len(), 4);
        let pids = crate::otap::transform::testing::collect_row_ids(
            column_at(attrs, PARENT_ID).as_ref(),
        );
        assert!(pids.iter().all(|p| event_ids.contains(p)));
    }

    // ---- Test helpers ----

    /// Converts a raw batch array back into an `OtapArrowRecords` via the store type `S`.
    fn batches_to_otap<S, const N: usize>(b: &[Option<RecordBatch>; N]) -> OtapArrowRecords
    where
        S: OtapBatchStore + Into<OtapArrowRecords>,
    {
        let mut store = S::new();
        store.batches_mut().clone_from_slice(b);
        store.into()
    }

    /// Validates reindexing for any signal type:
    /// 1. Converts input to OTLP (before reindex)
    /// 2. Snapshots parent -> child relation fingerprints (before reindex)
    /// 3. Reindexes the batches
    /// 4. Asserts no ID overlaps across batch groups
    /// 5. Asserts relation fingerprints are unchanged
    /// 6. Converts output to OTLP (after reindex)
    /// 7. Asserts the OTLP data is equivalent
    fn test_reindex_stores<S, const N: usize>(stores: &[S])
    where
        S: OtapBatchStore<BatchArray = [Option<RecordBatch>; N]> + Into<OtapArrowRecords> + Clone,
    {
        let mut batches: Vec<[Option<RecordBatch>; N]> =
            stores.iter().cloned().map(|s| s.into_batches()).collect();

        let before_otlp: Vec<_> = batches
            .iter()
            .map(|b| otap_to_otlp(&batches_to_otap::<S, N>(b)))
            .collect();
        let before_relations = extract_relation_fingerprints::<S, N>(&batches);

        let original = batches.clone();
        reindex_in_place::<S, N>(&mut batches).unwrap();
        assert_no_id_overlaps::<S, N>(&batches);

        let after_relations = extract_relation_fingerprints::<S, N>(&batches);
        assert_eq!(
            before_relations, after_relations,
            "Parent-child relations changed after reindexing"
        );

        let after_otlp: Vec<_> = batches
            .iter()
            .map(|b| otap_to_otlp(&batches_to_otap::<S, N>(b)))
            .collect();

        assert_equivalent(&before_otlp, &after_otlp);
        assert_fused_concatenate_equivalent::<S, N>(original, &before_otlp);
    }

    /// Runs the fused reindex + concatenate over `batches` and asserts the
    /// single output is equivalent to the inputs.
    fn assert_fused_concatenate_equivalent<S, const N: usize>(
        mut batches: Vec<[Option<RecordBatch>; N]>,
        expected: &[crate::proto::OtlpProtoMessage],
    ) where
        S: OtapBatchStore<BatchArray = [Option<RecordBatch>; N]> + Into<OtapArrowRecords>,
    {
        if batches.iter().all(|b| b.iter().all(Option::is_none)) {
            return;
        }
        let combined = concatenate::<N>(&mut batches, ConcatOptions::reindex()).unwrap();
        let combined_otlp = otap_to_otlp(&batches_to_otap::<S, N>(&combined));
        assert_equivalent(expected, &[combined_otlp]);
    }

    /// Reindexes transport-optimized batches and verifies OTLP equivalence.
    ///
    /// Fingerprinting doesn't work on transport-optimized batches (IDs are
    /// delta-encoded), so we only verify OTLP equivalence before and after.
    fn test_reindex_transport_optimized_stores<S, const N: usize>(stores: &[S])
    where
        S: OtapBatchStore<BatchArray = [Option<RecordBatch>; N]> + Into<OtapArrowRecords> + Clone,
    {
        let mut batches: Vec<[Option<RecordBatch>; N]> =
            stores.iter().cloned().map(|s| s.into_batches()).collect();

        apply_transport_encodings::<S, N>(&mut batches);

        let before_otlp: Vec<_> = batches
            .iter()
            .map(|b| otap_to_otlp(&batches_to_otap::<S, N>(b)))
            .collect();

        let original = batches.clone();
        reindex_in_place::<S, N>(&mut batches).unwrap();

        let after_otlp: Vec<_> = batches
            .iter()
            .map(|b| otap_to_otlp(&batches_to_otap::<S, N>(b)))
            .collect();
        assert_equivalent(&before_otlp, &after_otlp);
        assert_fused_concatenate_equivalent::<S, N>(original, &before_otlp);
    }

    fn test_reindex_logs(stores: &[Logs]) {
        test_reindex_stores::<Logs, { Logs::COUNT }>(stores);
    }

    fn test_reindex_traces(stores: &[Traces]) {
        test_reindex_stores::<Traces, { Traces::COUNT }>(stores);
    }

    fn test_reindex_metrics(stores: &[Metrics]) {
        test_reindex_stores::<Metrics, { Metrics::COUNT }>(stores);
    }

    fn test_reindex_transport_optimized_logs(stores: &[Logs]) {
        test_reindex_transport_optimized_stores::<Logs, { Logs::COUNT }>(stores);
    }

    fn test_reindex_transport_optimized_traces(stores: &[Traces]) {
        test_reindex_transport_optimized_stores::<Traces, { Traces::COUNT }>(stores);
    }

    fn test_reindex_transport_optimized_metrics(stores: &[Metrics]) {
        test_reindex_transport_optimized_stores::<Metrics, { Metrics::COUNT }>(stores);
    }

    #[test]
    #[rustfmt::skip]
    fn test_reindex_dict_parent_id_values_longer_than_keys() {
        // 2 rows, but the dictionary values array has 4 entries.
        // keys=[0,1], values=[0,1,2,3]
        let batch_a = traces!(
            (Spans,
                ("id", UInt16, vec![0u16, 1])),
            (SpanEvents,
                ("id", UInt32, vec![0u32, 1]),
                ("parent_id", UInt16, vec![0u16, 1])),
            (SpanEventAttrs,
                ("parent_id", (UInt8, UInt32), (vec![0u8, 1], vec![0u32, 1, 2, 3])))
        ).into_batches();

        reindex_traces(&mut [batch_a]).unwrap();
    }

    /// Regression test: non-primary ID columns (resource.id) with duplicates must
    /// use the slow path even when span == len.
    ///
    /// resource.id = [0, 1, 1, 4, 4] has span=5, len=5, which looks contiguous.
    /// But only 3 unique values exist (0, 1, 4). The fast path would advance the
    /// offset by 5 (the span) instead of 3 (the unique count), causing the second
    /// batch's resource.id values to collide with the first batch after remapping.
    /// The is_primary check forces the slow path for non-primary columns.
    #[test]
    #[rustfmt::skip]
    fn test_logs_many_to_many_resource_id_with_gaps_and_duplicates() {
        // resource.id has duplicates and gaps -- span == len but NOT contiguous.
        // 5 log rows, 3 unique resources.
        let log_ids_1     = vec![0u16, 1, 2, 3, 4];
        let resource_ids_1 = vec![0u16, 1, 1, 4, 4];

        // Second batch: 2 log rows, 2 unique resources.
        let log_ids_2      = vec![0u16, 1];
        let resource_ids_2 = vec![0u16, 1];

        test_reindex_logs(& [
            logs!(
                (Logs,
                    ("id", UInt16, log_ids_1),
                    ("resource.id", UInt16, resource_ids_1)),
                (ResourceAttrs,
                    ("parent_id", UInt16, vec![0u16, 1, 4]))
            ),
            logs!(
                (Logs,
                    ("id", UInt16, log_ids_2),
                    ("resource.id", UInt16, resource_ids_2)),
                (ResourceAttrs,
                    ("parent_id", UInt16, vec![0u16, 1]))
            ),
        ]);
    }

    /// Applies transport optimized encodings to all payload types in each batch group.
    fn apply_transport_encodings<S: OtapBatchStore, const N: usize>(
        batches: &mut [[Option<RecordBatch>; N]],
    ) {
        for group in batches.iter_mut() {
            for &payload_type in S::allowed_payload_types() {
                let idx = payload_to_idx(payload_type);
                if let Some(rb) = group[idx].take() {
                    let (encoded, _) =
                        apply_transport_optimized_encodings(&payload_type, &rb).unwrap();
                    group[idx] = Some(encoded);
                }
            }
        }
    }

    /// Searches all allowed payload types in a batch store to find if
    /// `child_type` appears as a child in any relation.  If found, returns the
    /// parent's primary id column size (which determines the parent_id column
    /// type).  Returns `None` for root types that have no parent.
    fn find_parent_id_size<S: OtapBatchStore>(
        child_type: ArrowPayloadType,
    ) -> Option<IdColumnType> {
        for &pt in S::allowed_payload_types() {
            let info = payload_relations(pt);
            let Some(primary_id) = info.primary_id else {
                continue;
            };
            for relation in info.relations {
                if relation.child_types.contains(&child_type) {
                    return Some(primary_id.size);
                }
            }
        }
        None
    }
}
