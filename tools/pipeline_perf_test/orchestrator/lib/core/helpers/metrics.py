import pandas as pd
from typing import (
    Iterable,
    Optional,
    Callable,
    Union,
    Sequence,
    TYPE_CHECKING,
    List,
    Tuple,
)

if TYPE_CHECKING:
    from ..telemetry.metric import MetricDataFrame


def format_bytes(num: float) -> str:
    """
    Convert a numeric byte value to a human-readable string.
    E.g. 123456 -> "120.56 KB"
    """
    if pd.isna(num):
        return ""
    for unit in ["B", "KB", "MB", "GB", "TB"]:
        if abs(num) < 1024:
            return f"{num:.2f} {unit}"
        num /= 1024
    return f"{num:.2f} PB"


def append_string(suffix: str) -> Callable[[str | float], str]:
    """
    Returns a formatter function that appends the given suffix to a value.

    The returned function:
    - Returns an empty string if the input value is NaN or an empty string.
    - If the input is a string, appends the suffix directly.
    - If the input is a float or numeric, formats it to two decimal places before appending the suffix.

    Parameters:
    - suffix: The string to append to the formatted value.

    Returns:
    - A function that formats input values by appending the suffix.
    """

    def formatter(val: str | float) -> str:
        # Return empty string for NaN or empty input
        if pd.isna(val) or val == "":
            return ""
        # Append suffix directly if val is string, else format float with 2 decimals before appending
        return f"{val}{suffix}" if isinstance(val, str) else f"{val:.2f}{suffix}"

    return formatter


def format_metrics_by_ordered_rules(
    df: pd.DataFrame,
    metric_col: str = "metric_name",
    format_rules: List[Tuple[str, Callable[[float | str], str]]] = None,
    columns: Iterable[str] = None,
    exclude_columns: Iterable[str] = None,
) -> pd.DataFrame:
    """
    Apply formatting functions to DataFrame columns based on ordered (pattern, function) rules.

    Parameters:
    - df: the input DataFrame
    - metric_col: column holding metric names (default: 'metric_name')
    - format_rules: ordered list of (regex pattern, function)
    - columns: which columns to apply formatting to (default: all except metric_col)
    - exclude_columns: columns to exclude from formatting

    Returns:
    - A new DataFrame with formatted strings
    """
    df = df.copy()
    if metric_col not in df.columns or format_rules is None:
        return df

    if columns is None:
        columns = [col for col in df.columns if col != metric_col]

    if exclude_columns is not None:
        columns = [col for col in columns if col not in exclude_columns]

    # Allow string assignment
    for col in columns:
        df[col] = df[col].astype("object")

    for pattern, func in format_rules:
        metric_mask = df[metric_col].str.contains(pattern, regex=True)
        for col in columns:
            df.loc[metric_mask, col] = df.loc[metric_mask, col].apply(func)

    return df


def concat_metrics_df(
    dfs: Sequence[Union[pd.DataFrame, "MetricDataFrame"]], **kwargs
) -> "MetricDataFrame":
    """
    Concatenate a sequence of MetricDataFrame or DataFrame objects and return a MetricDataFrame.

    Parameters:
    - dfs: A sequence of DataFrames (preferably MetricDataFrame)
    - kwargs: Additional keyword arguments passed to pd.concat()

    Returns:
    - MetricDataFrame: A new MetricDataFrame containing the concatenated result
    """
    from ..telemetry.metric import MetricDataFrame

    concatenated = pd.concat(dfs, **kwargs)
    return MetricDataFrame(concatenated)


def delta(x: pd.DataFrame):
    """
    Calculate the difference between the last and first rows of a DataFrame.

    Parameters:
    - x: A pandas DataFrame with at least one row.

    Returns:
    - A pandas Series representing the element-wise difference between the last and first rows,
      or None if the DataFrame has fewer than 2 rows.
    """
    # Check if DataFrame has at least two rows
    if len(x) >= 2:
        # Subtract the first row from the last row
        return x.iloc[-1] - x.iloc[0]
    else:
        # Not enough rows to compute delta; return None
        return None


def aggregate(
    df: "MetricDataFrame",
    by: Optional[list[str]] = None,
    agg_func: Union[str, Callable, list[Union[str, Callable]]] = "mean",
    agg_collapsed_metric_name: Optional[str] = None,
) -> "MetricDataFrame":
    from ..telemetry.metric import MetricDataFrame

    def prepare_dataframe(df: pd.DataFrame) -> pd.DataFrame:
        def safe_numeric(val):
            return val if isinstance(val, (int, float)) else None

        df = df.copy()
        df["value"] = df["value"].apply(safe_numeric)
        return df

    def extract_nested_group_keys(df: pd.DataFrame, by: list[str]):
        extracted_keys = {}
        actual_by = []
        if by:
            for key in by:
                if "." in key:
                    base, subkey = key.split(".", 1)
                    new_col = f"{base}.{subkey}"
                    df[new_col] = df[base].apply(
                        lambda d: d.get(subkey) if isinstance(d, dict) else None
                    )
                    actual_by.append(new_col)
                    extracted_keys.setdefault(base, {})[subkey] = new_col
                else:
                    actual_by.append(key)

        return df, actual_by, extracted_keys

    def aggregate_values(df: pd.DataFrame, by: list[str], agg_func):
        if by:
            grouped = df.groupby(by, dropna=False)["value"].agg(agg_func).reset_index()
            latest_ts = (
                df.groupby(by, dropna=False)["timestamp"]
                .max()
                .reset_index(name="latest_timestamp")
            )
        else:
            grouped = pd.DataFrame({"value": [df["value"].agg(agg_func)]})
            latest_ts = pd.DataFrame({"latest_timestamp": [df["timestamp"].max()]})
        return grouped, latest_ts

    def extract_attributes(row, extracted_keys, key_name):
        return (
            {k: row[v] for k, v in extracted_keys.get(key_name, {}).items()}
            if key_name in extracted_keys
            else {}
        )

    def create_output_row(
        row,
        col_name: str,
        extracted_keys: dict,
        agg_collapsed_metric_name: Optional[str],
        single_func: bool = False,
    ) -> dict:
        def unpack_attrs(attr_type: str) -> dict:
            return extract_attributes(row, extracted_keys, attr_type)

        new_row = {
            "value": row[col_name] if not single_func else row["value"],
            "metric_type": "aggregated",
            "metric_name": f"{col_name}({row.get('metric_name', agg_collapsed_metric_name)})",
            "timestamp": row.get("latest_timestamp", pd.NaT),
            "metric_attributes": unpack_attrs("metric_attributes"),
            "resource_attributes": unpack_attrs("resource_attributes"),
            "scope_attributes": unpack_attrs("scope_attributes"),
        }
        return new_row

    def generate_output_rows(
        grouped: pd.DataFrame,
        agg_func,
        extracted_keys: dict,
        agg_collapsed_metric_name: Optional[str],
    ) -> list[dict]:
        result_rows = []

        if isinstance(agg_func, list):
            for func in agg_func:
                col_name = func.__name__ if callable(func) else func
                for _, row in grouped.iterrows():
                    result_rows.append(
                        create_output_row(
                            row, col_name, extracted_keys, agg_collapsed_metric_name
                        )
                    )
        else:
            col_name = agg_func.__name__ if callable(agg_func) else agg_func
            for _, row in grouped.iterrows():
                result_rows.append(
                    create_output_row(
                        row,
                        col_name,
                        extracted_keys,
                        agg_collapsed_metric_name,
                        single_func=True,
                    )
                )

        return result_rows

    df = prepare_dataframe(df)
    df, actual_by, extracted_keys = extract_nested_group_keys(df, by)
    grouped, latest_ts = aggregate_values(df, actual_by, agg_func)

    if actual_by:
        grouped = grouped.merge(latest_ts, on=actual_by, how="left")
    else:
        grouped["latest_timestamp"] = latest_ts["latest_timestamp"].iloc[0]

    result_rows = generate_output_rows(
        grouped, agg_func, extracted_keys, agg_collapsed_metric_name
    )

    return MetricDataFrame(result_rows)


def compute_over_time(
    df: "MetricDataFrame",
    by: list[str],
    compute_fn: Callable[[pd.DataFrame], pd.DataFrame],
    name_fn: Callable[[str], str],
) -> "MetricDataFrame":
    from ..telemetry.metric import MetricDataFrame

    df = df.copy()
    df = df.sort_values("timestamp")
    df["value"] = pd.to_numeric(df["value"], errors="coerce")
    df = df.dropna(subset=["value", "timestamp"])

    # Handle nested dict keys (dot notation)
    group_keys = []
    for key in by:
        if "." in key:
            base, subkey = key.split(".", 1)
            col = f"{base}.{subkey}"
            df[col] = df[base].apply(
                lambda d: d.get(subkey) if isinstance(d, dict) else None
            )
            group_keys.append(col)
        else:
            group_keys.append(key)

    grouped = df.groupby(group_keys, dropna=False)
    result_rows = []

    for group, group_df in grouped:
        group_df = group_df.sort_values("timestamp")
        group_df = compute_fn(group_df)  # apply transformation function

        for _, row in group_df.iterrows():
            if pd.isna(row["value"]):
                continue

            new_row = {
                "value": row["value"],
                "metric_type": "aggregated",
                "timestamp": row["timestamp"],
            }

            keys = group if isinstance(group, tuple) else [group]
            for key, val in zip(group_keys, keys):
                if "." in key:
                    base, subkey = key.split(".", 1)
                    new_row.setdefault(base, {})[subkey] = val
                else:
                    new_row[key] = val

            metric_name = row["metric_name"]
            new_row["metric_name"] = name_fn(metric_name)

            result_rows.append(new_row)

    return MetricDataFrame(result_rows)


def compute_rate_over_time(df: "MetricDataFrame", by: list[str]) -> "MetricDataFrame":
    def _rate_compute_fn(df: pd.DataFrame) -> pd.DataFrame:
        df["prev_value"] = df["value"].shift(1)
        df["prev_timestamp"] = df["timestamp"].shift(1)
        df["delta_seconds"] = (
            df["timestamp"] - df["prev_timestamp"]
        ).dt.total_seconds()
        df["delta_value"] = df["value"] - df["prev_value"]
        df["value"] = df.apply(
            lambda row: (
                row["delta_value"] / row["delta_seconds"]
                if row["delta_seconds"] > 0
                else None
            ),
            axis=1,
        )
        return df

    return compute_over_time(
        df, by, compute_fn=_rate_compute_fn, name_fn=lambda name: f"rate({name})"
    )


def compute_delta_over_time(df: "MetricDataFrame", by: list[str]) -> "MetricDataFrame":
    def _delta_compute_fn(df: pd.DataFrame) -> pd.DataFrame:
        df["prev_value"] = df["value"].shift(1)
        df["value"] = df["value"] - df["prev_value"]
        return df

    return compute_over_time(
        df, by, compute_fn=_delta_compute_fn, name_fn=lambda name: f"delta({name})"
    )


def split_raw_metrics_by_group(
    df: "MetricDataFrame",
    group_key: str = "metric_attributes.container_name",
) -> dict[str, pd.DataFrame]:
    """
    Takes a raw MetricDataFrame and returns a dict of DataFrames per group/component.

    Each DataFrame contains timestamp, metric_name, and value columns.

    Args:
        df (MetricDataFrame): Raw metric dataframe.
        group_key (str): Dot-separated key to group by (e.g. "metric_attributes.component_name").

    Returns:
        dict of group/component name -> simple DataFrame
    """
    df = df.copy()
    # Step 1: Extract group key
    base, subkey = group_key.split(".", 1)
    df["_group"] = df[base].apply(
        lambda d: d.get(subkey) if isinstance(d, dict) else None
    )

    # Step 2: Filter columns
    slim_df = df[["timestamp", "metric_name", "value", "_group"]]

    # Step 43: Split into group-specific tables
    results = {
        group: group_df.drop(columns="_group").reset_index(drop=True)
        for group, group_df in slim_df.groupby("_group")
    }

    return results


def pivot_aggregated_metrics(
    df: "MetricDataFrame",
    group_key: str = "metric_attributes.container_name",
) -> dict[str, pd.DataFrame]:
    """
    Takes an aggregated MetricDataFrame and returns a dict of formatted tables
    per component (e.g. container_name).

    Args:
        df (MetricDataFrame): Aggregated metric dataframe.
        group_key (str): Dot-separated key to group by (e.g. "metric_attributes.container_name").

    Returns:
        dict of container/component name -> formatted pivot DataFrame
    """
    df = df.copy()
    # Step 1: Extract group key
    base, subkey = group_key.split(".", 1)
    df["_group"] = df[base].apply(
        lambda d: d.get(subkey) if isinstance(d, dict) else None
    )

    # Step 2: Extract agg + raw metric name
    metric_info = df["metric_name"].str.extract(r"(?P<agg>\w+)\((?P<metric>.*)\)")
    df["agg"] = metric_info["agg"]
    df["metric"] = metric_info["metric"]

    # Step 3: Pivot per group_key
    results = {}
    for group, sub_df in df.groupby("_group"):
        pivot = sub_df.pivot(index="metric", columns="agg", values="value")
        pivot = pivot.reset_index()
        pivot.rename(columns={"metric": "metric_name"}, inplace=True)
        results[group] = pivot

    return results
