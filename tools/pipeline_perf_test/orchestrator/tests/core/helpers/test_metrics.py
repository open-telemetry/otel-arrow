import pytest
import pandas as pd

from lib.core.helpers.metrics import (
    format_bytes,
    append_string,
    format_metrics_by_ordered_rules,
)


class DummyMetricDataFrame(pd.DataFrame):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)


@pytest.mark.parametrize(
    "input_value, expected_output",
    [
        (0, "0.00 B"),
        (500, "500.00 B"),
        (1023, "1023.00 B"),
        (1024, "1.00 KB"),
        (123456, "120.56 KB"),
        (1048576, "1.00 MB"),
        (1073741824, "1.00 GB"),
        (1099511627776, "1.00 TB"),
        (1125899906842624, "1.00 PB"),
        (float("nan"), ""),
        (-1024, "-1.00 KB"),
        (-123456, "-120.56 KB"),
    ],
)
def test_format_bytes(input_value, expected_output):
    assert format_bytes(input_value) == expected_output


@pytest.mark.parametrize(
    "suffix, input_value, expected_output",
    [
        ("%", 50, "50.00%"),
        (" units", 3.14159, "3.14 units"),
        (" kg", "100", "100 kg"),
        ("!", "", ""),
        ("!", float("nan"), ""),
        ("$", 0, "0.00$"),
        (" USD", "2500", "2500 USD"),
        ("m", -5.5, "-5.50m"),
    ],
)
def test_append_string(suffix, input_value, expected_output):
    formatter = append_string(suffix)
    assert formatter(input_value) == expected_output


@pytest.fixture
def sample_df():
    return pd.DataFrame(
        {
            "metric_name": [
                "size_bytes",
                "size_bytes",
                "count",
                "count",
                "other_metric",
            ],
            "value1": [1024, 2048, 10, 20, 5],
            "value2": [512, 1024, 5, 15, 3],
            "notes": ["note1", "note2", "note3", "note4", "note5"],
        }
    )


def test_format_metrics_by_ordered_rules_basic(sample_df):
    rules = [(r"bytes$", format_bytes), (r"count", append_string(" units"))]
    result = format_metrics_by_ordered_rules(
        sample_df, format_rules=rules, columns=["value1", "value2"]
    )

    # Check bytes formatting applied correctly
    assert result.loc[result.metric_name == "size_bytes", "value1"].iloc[0] == "1.00 KB"
    assert result.loc[result.metric_name == "size_bytes", "value2"].iloc[1] == "1.00 KB"

    # Check append_string formatting applied correctly
    assert result.loc[result.metric_name == "count", "value1"].iloc[0] == "10.00 units"
    assert result.loc[result.metric_name == "count", "value2"].iloc[1] == "15.00 units"

    # Check unchanged for other_metric and non-target columns
    assert result.loc[result.metric_name == "other_metric", "value1"].iloc[0] == 5
    assert result.loc[result.metric_name == "other_metric", "notes"].iloc[0] == "note5"


def test_format_metrics_with_columns_and_exclusions(sample_df):
    rules = [(r"bytes$", format_bytes)]

    # Only format value1 column
    result = format_metrics_by_ordered_rules(
        sample_df, format_rules=rules, columns=["value1"]
    )
    assert result.loc[result.metric_name == "size_bytes", "value1"].iloc[0] == "1.00 KB"
    # value2 should be untouched
    assert result.loc[result.metric_name == "size_bytes", "value2"].iloc[0] == 512

    # Exclude value1 from formatting, so nothing changes there
    result2 = format_metrics_by_ordered_rules(
        sample_df, format_rules=rules, exclude_columns=["value1", "notes"]
    )
    assert result2.loc[result.metric_name == "size_bytes", "value1"].iloc[0] == 1024
    assert (
        result2.loc[result.metric_name == "size_bytes", "value2"].iloc[0] == "512.00 B"
    )


def test_no_metric_col_or_no_rules(sample_df):
    # No metric_col
    result = format_metrics_by_ordered_rules(
        sample_df, metric_col="missing_col", format_rules=[(r".*", lambda x: "X")]
    )
    pd.testing.assert_frame_equal(result, sample_df)

    # No format_rules
    result2 = format_metrics_by_ordered_rules(sample_df, format_rules=None)
    pd.testing.assert_frame_equal(result2, sample_df)


# Tests for concat_metrics_dataframe
