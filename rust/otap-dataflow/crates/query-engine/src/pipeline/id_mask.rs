// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared `IdMask` type for representing sets of selected IDs.
//!
//! `IdMask` is used across the filter and expression evaluation systems to represent which
//! parent IDs have been selected by some predicate. It supports efficient boolean combination
//! (AND, OR, NOT) via bitmap algebra without materializing intermediate arrays.

use otap_df_pdata::otap::filter::{IdBitmap, IdBitmapPool};

/// This represents which IDs have been selected by some filter operation.
///
/// For example it can be used as the return type from filtering attributes to represent
/// values from the parent_id column matched some filter that was applied.
#[derive(Debug, PartialEq)]
pub(crate) enum IdMask {
    // All IDs are selected
    All,

    /// None of the IDs are selected
    None,

    /// Some of the IDs are selected
    Some(IdBitmap),

    /// Some of the IDs are not selected
    NotSome(IdBitmap),
}

impl IdMask {
    #[allow(dead_code)] // used by the execute_as_id_mask path in bitmap.rs
    pub(crate) fn contains(&self, id: u32) -> bool {
        match self {
            Self::All => true,
            Self::None => false,
            Self::Some(bitmap) => bitmap.contains(id),
            Self::NotSome(bitmap) => !bitmap.contains(id),
        }
    }

    /// Returns the owned bitmap (if any) to the pool for reuse.
    pub(crate) fn release_to(self, pool: &mut IdBitmapPool) {
        match self {
            Self::Some(bm) | Self::NotSome(bm) => pool.release(bm),
            Self::All | Self::None => {}
        }
    }

    /// Combines two masks with OR logic, returning spare bitmaps to the pool.
    pub(crate) fn combine_or(self, rhs: Self, pool: &mut IdBitmapPool) -> Self {
        match (self, rhs) {
            (Self::All, other) | (other, Self::All) => {
                other.release_to(pool);
                Self::All
            }
            (Self::None, other) | (other, Self::None) => other,

            (Self::Some(mut lhs), Self::Some(rhs)) => {
                lhs.union_with(&rhs);
                pool.release(rhs);
                Self::Some(lhs)
            }

            (Self::Some(lhs), Self::NotSome(mut rhs))
            | (Self::NotSome(mut rhs), Self::Some(lhs)) => {
                // Some(lhs) | NotSome(rhs) = Some(lhs) | !Some(rhs)
                // = everything except what's in rhs but not in lhs
                // = NotSome(rhs - lhs)
                rhs.difference_with(&lhs);
                pool.release(lhs);
                if rhs.is_empty() {
                    pool.release(rhs);
                    Self::All
                } else {
                    Self::NotSome(rhs)
                }
            }

            (Self::NotSome(mut lhs), Self::NotSome(rhs)) => {
                // NotSome(lhs) | NotSome(rhs) = !lhs | !rhs = !(lhs & rhs)
                lhs.intersect_with(&rhs);
                pool.release(rhs);
                Self::NotSome(lhs)
            }
        }
    }

    /// Combines two masks with AND logic, returning spare bitmaps to the pool.
    pub(crate) fn combine_and(self, rhs: Self, pool: &mut IdBitmapPool) -> Self {
        match (self, rhs) {
            (Self::None, other) | (other, Self::None) => {
                other.release_to(pool);
                Self::None
            }
            (Self::All, other) | (other, Self::All) => other,

            (Self::Some(mut lhs), Self::Some(rhs)) => {
                // Some(lhs) & Some(rhs) = intersection
                lhs.intersect_with(&rhs);
                pool.release(rhs);
                Self::Some(lhs)
            }

            (Self::Some(mut lhs), Self::NotSome(rhs))
            | (Self::NotSome(rhs), Self::Some(mut lhs)) => {
                // Some(lhs) & NotSome(rhs) = Some(lhs) & !Some(rhs)
                // = lhs minus rhs
                lhs.difference_with(&rhs);
                pool.release(rhs);
                if lhs.is_empty() {
                    pool.release(lhs);
                    Self::None
                } else {
                    Self::Some(lhs)
                }
            }

            (Self::NotSome(mut lhs), Self::NotSome(rhs)) => {
                // NotSome(lhs) & NotSome(rhs) = !lhs & !rhs = !(lhs | rhs)
                lhs.union_with(&rhs);
                pool.release(rhs);
                Self::NotSome(lhs)
            }
        }
    }
}
