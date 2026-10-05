"""Read saved Prometheus evidence and require finite, nonnegative counters."""

import json
import math
import re


def read_prometheus(path):
    samples = []
    for line in path.read_text().splitlines():
        if not line or line.startswith("#"):
            continue
        match = re.fullmatch(r"([\w:]+)(?:\{(.*)\})?\s+(\S+)(?:\s+\S+)?", line)
        if match is None:
            raise ValueError(f"Invalid Prometheus sample in {path}: {line}")
        labels = {
            key: json.loads(f'"{value}"')
            for key, value in re.findall(
                r'(\w+)="((?:[^"\\]|\\.)*)"', match[2] or ""
            )
        }
        samples.append((match[1], labels, float(match[3])))
    return samples


def counter(samples, name, **labels):
    values = [value for metric, tags, value in samples
              if metric == name
              and all(tags.get(key) == expected for key, expected in labels.items())]
    if not values or any(not math.isfinite(value) or value < 0 for value in values):
        raise ValueError(f"Missing or invalid counter {name} {labels}")
    return sum(values)
