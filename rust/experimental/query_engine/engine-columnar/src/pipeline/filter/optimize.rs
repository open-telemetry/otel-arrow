// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::pipeline::filter::{Composite, FilterPlan};

///
pub struct AttrsFilterCombineOptimizerRule {}

impl AttrsFilterCombineOptimizerRule {
    fn optimize(input: Composite<FilterPlan>) -> Composite<FilterPlan> {
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
                                right_plan.attribute_filter = Some(Composite::and(l, r));

                                if left_plan.source_filter.is_none() {
                                    // left_plan plan is now empty, so just return the right
                                    return right_plan.into();
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
                                right_plan.attribute_filter = Some(Composite::or(l, r));

                                // left filter will now be empty because there is no source_filter
                                // or attribute filter, so we can just return the right side
                                right_plan.into()
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

    use datafusion::logical_expr::{col, lit};
    use pretty_assertions::assert_eq;

    use crate::pipeline::filter::{AttributesFilterPlan, Composite, FilterPlan};
    use crate::pipeline::planner::AttributesIdentifier;

    // TODO -- need to add checks to ensure we don't combine any attrs filters that aren't
    // for the same payload type

    // TODO use constructors for AttributesFilterPlan

    #[test]
    fn test_attr_combine_simple_and_to_base() {
        let input = Composite::and(
            FilterPlan::from(AttributesFilterPlan {
                filter: col("key").eq(lit("attr")).and(col("str").eq(lit("x"))),
                attrs_identifier: AttributesIdentifier::Root,
            }),
            FilterPlan::from(AttributesFilterPlan {
                filter: col("key").eq(lit("attr2")).and(col("str").eq(lit("x"))),
                attrs_identifier: AttributesIdentifier::Root,
            }),
        );

        let result = AttrsFilterCombineOptimizerRule::optimize(input);

        let expected = Composite::from(FilterPlan {
            source_filter: None,
            attribute_filter: Some(Composite::And(
                Box::new(
                    AttributesFilterPlan {
                        filter: col("key").eq(lit("attr")).and(col("str").eq(lit("x"))),
                        attrs_identifier: AttributesIdentifier::Root,
                    }
                    .into(),
                ),
                Box::new(
                    AttributesFilterPlan {
                        filter: col("key").eq(lit("attr2")).and(col("str").eq(lit("x"))),
                        attrs_identifier: AttributesIdentifier::Root,
                    }
                    .into(),
                ),
            )),
        });

        assert_eq!(result, expected);
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
                AttributesFilterPlan {
                    filter: col("key").eq(lit("attr")).and(col("str").eq(lit("x"))),
                    attrs_identifier: AttributesIdentifier::Root,
                },
                AttributesFilterPlan {
                    filter: col("key").eq(lit("attr2")).and(col("str").eq(lit("x"))),
                    attrs_identifier: AttributesIdentifier::Root,
                },
            )),
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn test_attr_combine_deeply_nested_ands() {
        let input = Composite::and(
            Composite::and(
                Composite::and(
                    FilterPlan::from(AttributesFilterPlan {
                        filter: lit("1"),
                        attrs_identifier: AttributesIdentifier::Root,
                    }),
                    FilterPlan::from(AttributesFilterPlan {
                        filter: lit("2"),
                        attrs_identifier: AttributesIdentifier::Root,
                    }),
                ),
                Composite::and(
                    FilterPlan::from(AttributesFilterPlan {
                        filter: lit("3"),
                        attrs_identifier: AttributesIdentifier::Root,
                    }),
                    FilterPlan::from(AttributesFilterPlan {
                        filter: lit("4"),
                        attrs_identifier: AttributesIdentifier::Root,
                    }),
                ),
            ),
            FilterPlan::from(AttributesFilterPlan {
                filter: lit("5"),
                attrs_identifier: AttributesIdentifier::Root,
            }),
        );

        let result = AttrsFilterCombineOptimizerRule::optimize(input);

        let expected = Composite::Base(FilterPlan::from(Composite::and(
            Composite::and(
                Composite::and(
                    AttributesFilterPlan {
                        filter: lit("1"),
                        attrs_identifier: AttributesIdentifier::Root,
                    },
                    AttributesFilterPlan {
                        filter: lit("2"),
                        attrs_identifier: AttributesIdentifier::Root,
                    },
                ),
                Composite::and(
                    AttributesFilterPlan {
                        filter: lit("3"),
                        attrs_identifier: AttributesIdentifier::Root,
                    },
                    AttributesFilterPlan {
                        filter: lit("4"),
                        attrs_identifier: AttributesIdentifier::Root,
                    },
                ),
            ),
            AttributesFilterPlan {
                filter: lit("5"),
                attrs_identifier: AttributesIdentifier::Root,
            },
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
