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

                                // left_plan plan is now empty, so just return the right
                                if left_plan.source_filter.is_none() {
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

        let expected = Composite::Base(FilterPlan {
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
    fn test_attr_combine_simple_and_preserve_left_root_filter() {
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
}
