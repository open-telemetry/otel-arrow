"""
Utility functions for data processing and reporting.

Includes:
- `slugify`: Converts strings into filesystem-safe slugs.
- `group_by_populated_columns`: Groups and sorts DataFrame rows by which specified columns are populated.
"""

import re

import pandas as pd


def slugify(value: str, max_length=50) -> str:
    """
    Converts a string to a filesystem-safe slug:
    - Lowercase
    - Alphanumerics and underscores
    - Truncated to max_length
    """
    value = value.lower()
    value = re.sub(r"[^\w\s-]", "", value)  # Remove special chars
    value = re.sub(r"[\s\-]+", "_", value)  # Replace spaces/dashes with underscore
    value = value[:max_length]
    return value.strip("_")


def group_by_populated_columns(df: pd.DataFrame, columns: list[str]) -> pd.DataFrame:
    """
    Groups a DataFrame by which of the specified columns are populated (non-empty and non-NaN),
    and returns the DataFrame sorted by this group signature.

    Parameters:
    - df: The input DataFrame
    - columns: A list of columns to check for non-empty values (e.g., ["max", "mean", "min", "delta"])

    Returns:
    - A sorted copy of the DataFrame with similar rows grouped together
    """

    def populated_signature(row):
        return tuple(
            col for col in columns if pd.notna(row[col]) and str(row[col]).strip() != ""
        )

    df = df.copy()
    df["_populated_signature"] = df.apply(populated_signature, axis=1)
    df = df.sort_values("_populated_signature").drop(columns="_populated_signature")
    return df
