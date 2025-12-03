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
                    // look for opportunity to combine attrs filters
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
            FilterPlan {
                source_filter: Some(col("severity_text").eq(lit("WARN"))),
                attribute_filter: Some(
                    AttributesFilterPlan {
                        filter: col("key").eq(lit("attr")).and(col("str").eq(lit("x"))),
                        attrs_identifier: AttributesIdentifier::Root,
                    }
                    .into(),
                ),
            },
            FilterPlan {
                source_filter: None,
                attribute_filter: Some(
                    AttributesFilterPlan {
                        filter: col("key").eq(lit("attr2")).and(col("str").eq(lit("x"))),
                        attrs_identifier: AttributesIdentifier::Root,
                    }
                    .into(),
                ),
            },
        );

        let result = AttrsFilterCombineOptimizerRule::optimize(input);

        let expected = Composite::and(
            FilterPlan {
                source_filter: Some(col("severity_text").eq(lit("WARN"))),
                attribute_filter: None,
            },
            FilterPlan {
                source_filter: None,
                attribute_filter: Some(Composite::and(
                    AttributesFilterPlan {
                        filter: col("key").eq(lit("attr")).and(col("str").eq(lit("x"))),
                        attrs_identifier: AttributesIdentifier::Root,
                    },
                    AttributesFilterPlan {
                        filter: col("key").eq(lit("attr2")).and(col("str").eq(lit("x"))),
                        attrs_identifier: AttributesIdentifier::Root,
                    },
                )),
            },
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn test_attr_compile_deeply_nested_ands() {
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
}
