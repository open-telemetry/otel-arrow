// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::pipeline::filter::{Composite, FilterPlan};

/// This performs an optimization on the composite [`FilterPlan`] to combine the attribute filters
/// into fewer `Composite<AttributeFilter>`s.
///
/// When `FilterExec` filters by attributes, it first invokes
/// [`AttributeFilterExec`](super::AttributeFilterExec), which first scans the attribute filter
/// batch to create a selection vector, then maps this to a bitmask of parent_ids that passed the
/// predicate. It then passes this back to filter exec, scans an ID column on the parent batch to
/// create a selection vector for the parent. These final selection vectors then get and/or'd
/// together to produce the final selection vector for the root batch
///
/// However, if there are multiple attribute filters being applied, it's faster to just and/or
/// the intermediate parent_id bitmask, `Composite<AttributeFilterExec>` will do. So the goal with
/// this optimization is to produce a plan that will produce this type.
///
pub struct AttrsFilterCombineOptimizerRule {}

impl AttrsFilterCombineOptimizerRule {
    pub fn optimize(input: Composite<FilterPlan>) -> Composite<FilterPlan> {
        match input {
            Composite::And(left, right) => {
                let left = Self::optimize(*left);
                let right = Self::optimize(*right);

                match (left, right) {
                    // look for opportunity to combine attrs filters on siblings
                    (Composite::Base(mut left_plan), Composite::Base(mut right_plan)) => {
                        match (
                            left_plan.attribute_filter.take(),
                            right_plan.attribute_filter.take(),
                        ) {
                            // combine the attr filters if they're both defined
                            (Some(l), Some(r)) => {
                                // only combine the attribute filters if they are for the same
                                // attribute payload. this is needed because the attribute filters
                                // produce id masks that get joined to the parent_id, but different
                                // attribute's parent_ids point to different ID columns on the root
                                if l.attrs_identifier() == r.attrs_identifier() {
                                    right_plan.attribute_filter = Some(Composite::and(l, r));

                                    if left_plan.source_filter.is_none() {
                                        // left_plan plan is now empty, so just return the right
                                        return right_plan.into();
                                    }
                                } else {
                                    // otherwise just reset the filter to the original state
                                    left_plan.attribute_filter = Some(l);
                                    right_plan.attribute_filter = Some(r);
                                }
                            }

                            // replace the original if only one side has attr filter
                            (Some(l), None) => {
                                left_plan.attribute_filter = Some(l);
                            }
                            (None, Some(r)) => {
                                right_plan.attribute_filter = Some(r);
                            }

                            // when no attribute filters, do nothing
                            (None, None) => {}
                        }

                        Composite::and(left_plan, right_plan)
                    }

                    // look for opportunity to hoist the attribute filter plan from the child.
                    //
                    // e.g. if the expression tree looks like this:
                    // And (
                    //   And (<L>, attributes["x"] == "Y"),
                    //   attributes["x2"] == "Y2"
                    // )
                    //
                    // we want to transform it to:
                    // And(
                    //   <L>,
                    //   AND(attributes["X"] == "Y", attributes["x2"] == "Y2")
                    // )
                    //
                    (Composite::And(left_left, left_right), Composite::Base(right_plan)) => {
                        match *left_right {
                            // if the left's right child is the base, hoist it up to be 'and'ed with the right
                            // side of the current and, and then optimize that.
                            Composite::Base(left_right_plan) => Composite::and(
                                *left_left,
                                Self::optimize(Composite::and(left_right_plan, right_plan)),
                            ),

                            // otherwise just return the original
                            left_right => {
                                Composite::and(Composite::and(*left_left, left_right), right_plan)
                            }
                        }
                    }

                    // otherwise just return the originals
                    (l, r) => Composite::and(l, r),
                }
            }

            Composite::Or(left, right) => {
                let left = Self::optimize(*left);
                let right = Self::optimize(*right);

                match (left, right) {
                    (Composite::Base(mut left_plan), Composite::Base(mut right_plan)) => {
                        // we don't want to combine the attributes filters for something like this
                        // (x == y and attributes["x"] == y) or attributes["x2"] == "y2"
                        // and since the source filter always gets `and`'d with the attribute_filter
                        // we just return the original plan here
                        if left_plan.source_filter.is_some() || right_plan.source_filter.is_some() {
                            return Composite::or(left_plan, right_plan);
                        }

                        match (
                            left_plan.attribute_filter.take(),
                            right_plan.attribute_filter.take(),
                        ) {
                            // combine the attribute filters if they're both defined
                            (Some(l), Some(r)) => {
                                // only combine the attribute filters if they are for the same
                                // attribute payload. this is needed because the attribute filters
                                // produce id masks that get joined to the parent_id, but different
                                // attribute's parent_ids point to different ID columns on the root
                                if l.attrs_identifier() == r.attrs_identifier() {
                                    right_plan.attribute_filter = Some(Composite::or(l, r));

                                    // left filter will now be empty because there is no source_filter
                                    // or attribute filter, so we can just return the right side
                                    right_plan.into()
                                } else {
                                    // otherwise just reset the filter to the original state
                                    left_plan.attribute_filter = Some(l);
                                    right_plan.attribute_filter = Some(r);
                                    Composite::or(left_plan, right_plan)
                                }
                            }
                            // replace the original and return one side. If we're in either of the
                            // next two branches, it means the other side was basically empty
                            (Some(l), None) => {
                                left_plan.attribute_filter = Some(l);
                                left_plan.into()
                            }
                            (None, Some(r)) => {
                                right_plan.attribute_filter = Some(r);
                                right_plan.into()
                            }

                            // if we enter into this branch, both sides are empty so just return
                            // one of the empty sides
                            (None, None) => right_plan.into(),
                        }
                    }

                    // look for opportunity to hoist the attribute filter plan from the child.
                    //
                    // e.g. if the expression tree looks like this:
                    // OR (
                    //   OR (<L>, attributes["x"] == "Y"),
                    //   attributes["x2"] == "Y2"
                    // )
                    //
                    // we want to transform it to:
                    // Or(
                    //   <L>,
                    //   Or(attributes["X"] == "Y", attributes["x2"] == "Y2")
                    // )
                    //
                    (Composite::Or(left_left, left_right), Composite::Base(right_plan)) => {
                        match *left_right {
                            // if the left's right child is the base, hoist it up to be 'or'ed with the right
                            // side of the current and, and then optimize that.
                            Composite::Base(left_right_plan) => Composite::or(
                                *left_left,
                                Self::optimize(Composite::or(left_right_plan, right_plan)),
                            ),

                            // otherwise just return the original
                            left_right => {
                                Composite::or(Composite::or(*left_left, left_right), right_plan)
                            }
                        }
                    }

                    // otherwise, just return the originals
                    (l, r) => Composite::or(l, r),
                }
            }

            input => input,
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    use datafusion::logical_expr::{Expr, col, lit};
    use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use pretty_assertions::assert_eq;

    use crate::pipeline::filter::{AttributesFilterPlan, Composite, FilterPlan};
    use crate::pipeline::planner::AttributesIdentifier;

    impl FilterPlan {
        // helper function for creating a FilterPlan for test suite
        fn new<T>(source_filter: Option<Expr>, attribute_filter: Option<T>) -> Self
        where
            T: Into<Composite<AttributesFilterPlan>>,
        {
            Self {
                source_filter,
                attribute_filter: attribute_filter.map(T::into),
            }
        }
    }

    #[test]
    fn test_attr_combine_simple_and_to_base() {
        let input = Composite::and(
            FilterPlan::from(AttributesFilterPlan::new(
                lit("a"),
                AttributesIdentifier::Root,
            )),
            FilterPlan::from(AttributesFilterPlan::new(
                lit("b"),
                AttributesIdentifier::Root,
            )),
        );

        let result = AttrsFilterCombineOptimizerRule::optimize(input);

        let expected = Composite::from(FilterPlan::from(Composite::and(
            AttributesFilterPlan::new(lit("a"), AttributesIdentifier::Root),
            AttributesFilterPlan::new(lit("b"), AttributesIdentifier::Root),
        )));

        assert_eq!(result, expected);
    }

    #[test]
    fn test_attr_combine_does_not_combine_and_for_different_attr_types() {
        let input = Composite::and(
            FilterPlan::from(AttributesFilterPlan::new(
                lit("a"),
                AttributesIdentifier::Root,
            )),
            FilterPlan::from(AttributesFilterPlan::new(
                lit("b"),
                AttributesIdentifier::NonRoot(ArrowPayloadType::ResourceAttrs),
            )),
        );

        let result = AttrsFilterCombineOptimizerRule::optimize(input.clone());
        assert_eq!(result, input)
    }

    #[test]
    fn test_attr_combine_simple_and_preserve_left_source_filter() {
        let input = Composite::and(
            FilterPlan::new(
                Some(col("severity_text").eq(lit("WARN"))),
                Some(AttributesFilterPlan {
                    filter: col("key").eq(lit("attr")).and(col("str").eq(lit("x"))),
                    attrs_identifier: AttributesIdentifier::Root,
                }),
            ),
            FilterPlan::new(
                None,
                Some(AttributesFilterPlan {
                    filter: col("key").eq(lit("attr2")).and(col("str").eq(lit("x"))),
                    attrs_identifier: AttributesIdentifier::Root,
                }),
            ),
        );

        let result = AttrsFilterCombineOptimizerRule::optimize(input);

        let expected = Composite::and(
            FilterPlan::from(col("severity_text").eq(lit("WARN"))),
            FilterPlan::from(Composite::and(
                AttributesFilterPlan::new(
                    col("key").eq(lit("attr")).and(col("str").eq(lit("x"))),
                    AttributesIdentifier::Root,
                ),
                AttributesFilterPlan::new(
                    col("key").eq(lit("attr2")).and(col("str").eq(lit("x"))),
                    AttributesIdentifier::Root,
                ),
            )),
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn test_attr_combine_deeply_nested_ands() {
        let input = Composite::and(
            Composite::and(
                Composite::and(
                    FilterPlan::from(AttributesFilterPlan::new(
                        lit("1"),
                        AttributesIdentifier::Root,
                    )),
                    FilterPlan::from(AttributesFilterPlan::new(
                        lit("2"),
                        AttributesIdentifier::Root,
                    )),
                ),
                Composite::and(
                    FilterPlan::from(AttributesFilterPlan::new(
                        lit("3"),
                        AttributesIdentifier::Root,
                    )),
                    FilterPlan::from(AttributesFilterPlan::new(
                        lit("4"),
                        AttributesIdentifier::Root,
                    )),
                ),
            ),
            FilterPlan::from(AttributesFilterPlan::new(
                lit("5"),
                AttributesIdentifier::Root,
            )),
        );

        let result = AttrsFilterCombineOptimizerRule::optimize(input);

        let expected = Composite::Base(FilterPlan::from(Composite::and(
            Composite::and(
                Composite::and(
                    AttributesFilterPlan::new(lit("1"), AttributesIdentifier::Root),
                    AttributesFilterPlan::new(lit("2"), AttributesIdentifier::Root),
                ),
                Composite::and(
                    AttributesFilterPlan::new(lit("3"), AttributesIdentifier::Root),
                    AttributesFilterPlan::new(lit("4"), AttributesIdentifier::Root),
                ),
            ),
            AttributesFilterPlan::new(lit("5"), AttributesIdentifier::Root),
        )));

        assert_eq!(result, expected);
    }

    #[test]
    fn test_attr_combine_attr_from_and_child_and_preserve_source_filters() {
        let input = Composite::and(
            Composite::and(
                FilterPlan::from(lit("source1")),
                FilterPlan::from(AttributesFilterPlan {
                    filter: lit("attr1"),
                    attrs_identifier: AttributesIdentifier::Root,
                }),
            ),
            FilterPlan::from(AttributesFilterPlan {
                filter: lit("attr2"),
                attrs_identifier: AttributesIdentifier::Root,
            }),
        );

        let result = AttrsFilterCombineOptimizerRule::optimize(input);

        let expected = Composite::and(
            FilterPlan::from(lit("source1")),
            FilterPlan::from(Composite::and(
                AttributesFilterPlan {
                    filter: lit("attr1"),
                    attrs_identifier: AttributesIdentifier::Root,
                },
                AttributesFilterPlan {
                    filter: lit("attr2"),
                    attrs_identifier: AttributesIdentifier::Root,
                },
            )),
        );

        assert_eq!(result, expected)
    }

    #[test]
    fn test_attr_combine_simple_or_to_base() {
        let input = Composite::or(
            FilterPlan::from(AttributesFilterPlan::new(
                lit("a"),
                AttributesIdentifier::Root,
            )),
            FilterPlan::from(AttributesFilterPlan::new(
                lit("b"),
                AttributesIdentifier::Root,
            )),
        );

        let result = AttrsFilterCombineOptimizerRule::optimize(input);

        let expected = Composite::from(FilterPlan::from(Composite::or(
            AttributesFilterPlan::new(lit("a"), AttributesIdentifier::Root),
            AttributesFilterPlan::new(lit("b"), AttributesIdentifier::Root),
        )));

        assert_eq!(result, expected);
    }

    #[test]
    fn test_attr_combine_does_not_combine_or_for_different_attr_types() {
        let input = Composite::or(
            FilterPlan::from(AttributesFilterPlan::new(
                lit("a"),
                AttributesIdentifier::Root,
            )),
            FilterPlan::from(AttributesFilterPlan::new(
                lit("b"),
                AttributesIdentifier::NonRoot(ArrowPayloadType::ResourceAttrs),
            )),
        );

        let result = AttrsFilterCombineOptimizerRule::optimize(input.clone());
        assert_eq!(result, input)
    }

    #[test]
    fn test_attr_combine_or_handles_empty_filter_one_side() {
        // it would be unusual if we were to construct an empty filter, but this test is just
        // here to safeguard that the optimizer rule handles it in a sensible way, which is to
        // eliminate the empty side

        // check left is empty
        let input = Composite::or(
            FilterPlan::new(None, Option::<AttributesFilterPlan>::None),
            FilterPlan::from(AttributesFilterPlan::new(
                lit("a"),
                AttributesIdentifier::Root,
            )),
        );

        let result = AttrsFilterCombineOptimizerRule::optimize(input);
        let expected = Composite::from(FilterPlan::from(AttributesFilterPlan::new(
            lit("a"),
            AttributesIdentifier::Root,
        )));
        assert_eq!(result, expected);

        // check right side is empty
        let input = Composite::or(
            FilterPlan::from(AttributesFilterPlan::new(
                lit("b"),
                AttributesIdentifier::Root,
            )),
            FilterPlan::new(None, Option::<AttributesFilterPlan>::None),
        );
        let result = AttrsFilterCombineOptimizerRule::optimize(input);
        let expected = Composite::from(FilterPlan::from(AttributesFilterPlan::new(
            lit("b"),
            AttributesIdentifier::Root,
        )));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_attr_combine_or_handles_empty_both_sides() {
        // similar to the test above, it would be unusual if we were to construct an empty filter,
        // but this test is just here to safeguard that the optimizer rule handles it in a sensible
        // way, which is to simplify to a simpler empty filter

        // check left is empty
        let input = Composite::or(
            FilterPlan::new(None, Option::<AttributesFilterPlan>::None),
            FilterPlan::new(None, Option::<AttributesFilterPlan>::None),
        );

        let result = AttrsFilterCombineOptimizerRule::optimize(input);
        let expected = Composite::from(FilterPlan::new(None, Option::<AttributesFilterPlan>::None));

        assert_eq!(result, expected)
    }

    #[test]
    fn test_attr_combine_or_does_not_combine_when_source_filter_is_some() {
        let input = Composite::or(
            FilterPlan::new(
                Some(lit("source1")),
                Some(AttributesFilterPlan::new(
                    lit("a1"),
                    AttributesIdentifier::Root,
                )),
            ),
            FilterPlan::from(AttributesFilterPlan::new(
                lit("b"),
                AttributesIdentifier::Root,
            )),
        );

        let result = AttrsFilterCombineOptimizerRule::optimize(input.clone());
        assert_eq!(result, input);
    }

    #[test]
    fn test_attr_combine_or_handles_deeply_nested() {
        let input = Composite::or(
            Composite::or(
                Composite::or(
                    FilterPlan::from(AttributesFilterPlan::new(
                        lit("a"),
                        AttributesIdentifier::Root,
                    )),
                    FilterPlan::from(AttributesFilterPlan::new(
                        lit("b"),
                        AttributesIdentifier::Root,
                    )),
                ),
                Composite::or(
                    FilterPlan::from(AttributesFilterPlan::new(
                        lit("c"),
                        AttributesIdentifier::Root,
                    )),
                    FilterPlan::from(AttributesFilterPlan::new(
                        lit("d"),
                        AttributesIdentifier::Root,
                    )),
                ),
            ),
            Composite::or(
                FilterPlan::from(AttributesFilterPlan::new(
                    lit("e"),
                    AttributesIdentifier::Root,
                )),
                FilterPlan::from(AttributesFilterPlan::new(
                    lit("f"),
                    AttributesIdentifier::Root,
                )),
            ),
        );

        let result = AttrsFilterCombineOptimizerRule::optimize(input);
        let expected = Composite::from(FilterPlan::from(Composite::or(
            Composite::or(
                Composite::or(
                    AttributesFilterPlan::new(lit("a"), AttributesIdentifier::Root),
                    AttributesFilterPlan::new(lit("b"), AttributesIdentifier::Root),
                ),
                Composite::or(
                    AttributesFilterPlan::new(lit("c"), AttributesIdentifier::Root),
                    AttributesFilterPlan::new(lit("d"), AttributesIdentifier::Root),
                ),
            ),
            Composite::or(
                AttributesFilterPlan::new(lit("e"), AttributesIdentifier::Root),
                AttributesFilterPlan::new(lit("f"), AttributesIdentifier::Root),
            ),
        )));

        assert_eq!(result, expected);
    }

    #[test]
    fn test_attr_combine_from_or_child_and_preserve_source_filter() {
        let input = Composite::or(
            Composite::or(
                FilterPlan::from(lit("source1")),
                FilterPlan::from(AttributesFilterPlan::new(
                    lit("attr1"),
                    AttributesIdentifier::Root,
                )),
            ),
            FilterPlan::from(AttributesFilterPlan::new(
                lit("attr2"),
                AttributesIdentifier::Root,
            )),
        );

        let result = AttrsFilterCombineOptimizerRule::optimize(input);

        let expected = Composite::or(
            FilterPlan::from(lit("source1")),
            FilterPlan::from(Composite::or(
                AttributesFilterPlan::new(lit("attr1"), AttributesIdentifier::Root),
                AttributesFilterPlan::new(lit("attr2"), AttributesIdentifier::Root),
            )),
        );

        assert_eq!(result, expected);
    }
}
