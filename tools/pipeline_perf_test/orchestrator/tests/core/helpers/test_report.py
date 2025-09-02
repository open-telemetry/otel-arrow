import pytest
import pandas as pd
import numpy as np

from lib.core.helpers.report import group_by_populated_columns, slugify


def test_basic_string():
    assert slugify("Hello World") == "hello_world"


def test_special_characters():
    assert slugify("Python@3.9! is #awesome") == "python39_is_awesome"


def test_multiple_spaces_and_dashes():
    assert slugify("hello    world---again") == "hello_world_again"


def test_leading_trailing_spaces():
    assert slugify("  Hello World  ") == "hello_world"


def test_uppercase_conversion():
    assert slugify("THIS IS A TEST") == "this_is_a_test"


def test_underscore_preservation():
    assert slugify("already_slugified_text") == "already_slugified_text"


def test_max_length_truncation():
    long_string = "This is a very long string that should be truncated properly"
    result = slugify(long_string, max_length=20)
    assert len(result) <= 20
    assert result.startswith("this_is_a")


def test_only_special_characters():
    assert slugify("!!!@@@###") == ""


def test_empty_string():
    assert slugify("") == ""


def test_custom_max_length_exact():
    input_str = "test_max_length_here"
    assert slugify(input_str, max_length=len(input_str)) == "test_max_length_here"


def test_trailing_underscores_removed_after_truncation():
    # This test checks that underscores at the edge are stripped *before* truncation
    input_str = "  A--B--C--D--E--F--G--H--I--J--K--L--M--N--O--P--Q--R--S--T--U--V--W--X--Y--Z  "
    result = slugify(input_str, max_length=10)
    assert not result.startswith("_") and not result.endswith("_")
    assert len(result) <= 10


def test_all_columns_populated():
    df = pd.DataFrame({"max": [1, 2], "mean": [3, 4], "min": [5, 6], "delta": [7, 8]})
    columns = ["max", "mean", "min", "delta"]
    result = group_by_populated_columns(df, columns)
    pd.testing.assert_frame_equal(
        result.reset_index(drop=True), df.reset_index(drop=True)
    )


def test_some_columns_empty_or_nan():
    df = pd.DataFrame(
        {
            "max": [1, np.nan, 3, ""],
            "mean": ["", 2, 3, 4],
            "min": [np.nan, "", 3, 4],
            "delta": [1, 2, "", np.nan],
        }
    )
    columns = ["max", "mean", "min", "delta"]
    result = group_by_populated_columns(df, columns)

    def signature(row):
        return tuple(
            col for col in columns if pd.notna(row[col]) and str(row[col]).strip() != ""
        )

    df["_sig"] = df.apply(signature, axis=1)
    expected_order = df.sort_values("_sig").index.tolist()
    result_indices = result.index.tolist()
    assert result_indices == expected_order
    df.drop(columns="_sig", inplace=True)


def test_columns_with_whitespace_strings():
    df = pd.DataFrame(
        {
            "max": [" ", "x", ""],
            "mean": ["", " ", "y"],
            "min": [np.nan, " ", "z"],
            "delta": [None, "a", " "],
        }
    )
    columns = ["max", "mean", "min", "delta"]
    result = group_by_populated_columns(df, columns)

    def signature(row):
        return tuple(
            col for col in columns if pd.notna(row[col]) and str(row[col]).strip() != ""
        )

    df["_sig"] = df.apply(signature, axis=1)
    expected_order = df.sort_values("_sig").index.tolist()
    result_indices = result.index.tolist()
    assert result_indices == expected_order
    df.drop(columns="_sig", inplace=True)


def test_empty_dataframe():
    df = pd.DataFrame(columns=["max", "mean", "min", "delta"])
    columns = ["max", "mean", "min", "delta"]
    result = group_by_populated_columns(df, columns)
    assert result.empty
    assert list(result.columns) == ["max", "mean", "min", "delta"]


def test_subset_of_columns():
    df = pd.DataFrame(
        {
            "max": [1, 2, None],
            "mean": [None, 2, 3],
            "min": [1, None, 3],
            "delta": [None, None, None],
        }
    )
    columns = ["max", "mean"]
    result = group_by_populated_columns(df, columns)

    def signature(row):
        return tuple(
            col for col in columns if pd.notna(row[col]) and str(row[col]).strip() != ""
        )

    df["_sig"] = df.apply(signature, axis=1)
    expected_order = df.sort_values("_sig").index.tolist()
    result_indices = result.index.tolist()
    assert result_indices == expected_order
    df.drop(columns="_sig", inplace=True)
