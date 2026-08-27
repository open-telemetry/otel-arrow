// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Cosmetic formatting for generated pipeline YAML.
//!
//! `serde_yaml` emits correct but dense YAML: block sequences sit at the same indentation as the
//! key that owns them, and nothing separates one node from the next. Hand-written pipeline configs
//! in this repository indent sequence items under their key and leave a blank line between nodes,
//! which is what a reviewer diffing generated output against a hand-written sample expects to see.
//!
//! This module only ever adds leading spaces and blank lines. It never rewrites the content of a
//! line, so quoting and escaping stay exactly as `serde_yaml` produced them. As a backstop,
//! [`prettify`] re-parses its own output and returns the input unchanged if the round-trip is not
//! value-identical, so a formatting bug can never alter the configuration the engine reads.

/// Spaces added per enclosing block sequence.
const SEQUENCE_INDENT: usize = 2;

/// Orders node entries so a generated pipeline reads in data-flow order.
///
/// Nodes live in a `HashMap`, so `serde_yaml` emits them in an arbitrary order that varies between
/// runs -- a batch node can precede the receiver that feeds it, and two runs of the same input
/// produce different files. Sorting by role and then by name makes the output stable and readable.
///
/// YAML mappings are unordered, so this changes presentation only; the parsed value is unaffected.
pub(crate) fn sort_nodes(value: &mut serde_yaml::Value) {
    let Some(groups) = value.get_mut("groups").and_then(|g| g.as_mapping_mut()) else {
        return;
    };

    for (_, group) in groups.iter_mut() {
        let Some(pipelines) = group.get_mut("pipelines").and_then(|p| p.as_mapping_mut()) else {
            continue;
        };
        for (_, pipeline) in pipelines.iter_mut() {
            let Some(nodes) = pipeline.get_mut("nodes").and_then(|n| n.as_mapping_mut()) else {
                continue;
            };

            let mut entries: Vec<_> = std::mem::take(nodes).into_iter().collect();
            entries.sort_by_cached_key(|(name, node)| {
                let urn = node
                    .get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or_default();
                (role_rank(urn), name.as_str().unwrap_or_default().to_owned())
            });
            for (name, node) in entries {
                let _ = nodes.insert(name, node);
            }
        }
    }
}

/// Position of a node URN in data-flow order.
fn role_rank(urn: &str) -> u8 {
    match () {
        () if urn.contains(":receiver:") => 0,
        () if urn.contains(":processor:fanout") => 1,
        () if urn.contains(":processor:filter") => 2,
        () if urn.contains(":processor:batch") => 3,
        () if urn.contains(":exporter:") => 5,
        () => 4,
    }
}

/// Re-indents block sequences and separates node entries with blank lines.
///
/// Returns `yaml` unchanged if the transformation would alter the parsed value, or if the document
/// contains a block scalar, whose interior lines are content rather than structure.
#[must_use]
pub(crate) fn prettify(yaml: &str) -> String {
    // A block scalar's body is data; shifting it would change the value.
    if yaml.contains(": |") || yaml.contains(": >") {
        return yaml.to_owned();
    }

    let styled = restyle(yaml);

    match (
        serde_yaml::from_str::<serde_yaml::Value>(yaml),
        serde_yaml::from_str::<serde_yaml::Value>(&styled),
    ) {
        (Ok(before), Ok(after)) if before == after => styled,
        _ => yaml.to_owned(),
    }
}

/// Applies the indentation and blank-line rules.
fn restyle(yaml: &str) -> String {
    // Each entry is one enclosing block sequence: the indentation `serde_yaml` gave its items, and
    // the total shift applied to lines inside it.
    let mut sequences: Vec<(usize, usize)> = Vec::new();
    let mut out = String::with_capacity(yaml.len() + yaml.len() / 8);

    // Indentation of the `nodes:` key whose children get blank lines between them, and whether one
    // child has already been seen.
    let mut nodes_indent: Option<usize> = None;
    let mut seen_first_node = false;

    for line in yaml.lines() {
        if line.trim().is_empty() {
            out.push('\n');
            continue;
        }

        let indent = line.len() - line.trim_start().len();
        let body = &line[indent..];
        let is_item = body.starts_with("- ") || body == "-";

        // A line at or left of a sequence's own indentation ends it, unless it is another item of
        // that same sequence. Continuation lines sit further right and keep it open.
        while let Some(&(seq_indent, _)) = sequences.last() {
            if seq_indent > indent || (seq_indent == indent && !is_item) {
                let _ = sequences.pop();
            } else {
                break;
            }
        }

        if is_item && sequences.last().map(|&(i, _)| i) != Some(indent) {
            let parent_shift = sequences.last().map_or(0, |&(_, shift)| shift);
            sequences.push((indent, parent_shift + SEQUENCE_INDENT));
        }

        let shift = sequences.last().map_or(0, |&(_, shift)| shift);

        // Blank line between sibling node entries, matching hand-written configs.
        if let Some(parent) = nodes_indent {
            if indent <= parent {
                // Left the `nodes:` block entirely.
                nodes_indent = None;
            } else if indent == parent + 2 && sequences.is_empty() {
                if seen_first_node {
                    out.push('\n');
                }
                seen_first_node = true;
            }
        }
        if body == "nodes:" {
            nodes_indent = Some(indent);
            seen_first_node = false;
        }

        for _ in 0..shift {
            out.push(' ');
        }
        out.push_str(line);
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a block sequence owned by a mapping key is formatted.
    /// Guarantees: items are indented under their key rather than left at the key's own column,
    /// which is how the hand-written sample configs in this repository are written.
    #[test]
    fn sequence_items_are_indented_under_their_key() {
        let input = "outputs:\n- default\n";

        let out = prettify(input);

        assert_eq!(out, "outputs:\n  - default\n");
    }

    /// Scenario: a sequence of mappings, and a sequence nested inside one of those mappings.
    /// Guarantees: every level is shifted by its own depth, so continuation lines stay aligned with
    /// the item that owns them and the nested sequence indents again.
    #[test]
    fn nested_sequences_indent_once_per_level() {
        let input = "connections:\n- from: a\n  to:\n  - b\n";

        let out = prettify(input);

        assert_eq!(out, "connections:\n  - from: a\n    to:\n      - b\n");
    }

    /// Scenario: several sibling entries under a `nodes:` mapping.
    /// Guarantees: a blank line separates them, and none is added before the first, so the block
    /// reads as discrete nodes rather than one wall of keys.
    #[test]
    fn node_entries_are_separated_by_blank_lines() {
        let input = "nodes:\n  a:\n    type: x\n  b:\n    type: y\n";

        let out = prettify(input);

        assert_eq!(out, "nodes:\n  a:\n    type: x\n\n  b:\n    type: y\n");
    }

    /// Scenario: a document containing a block scalar, whose body lines are data.
    /// Guarantees: it is returned untouched, because shifting those lines would change the string
    /// they encode.
    #[test]
    fn block_scalars_are_left_alone() {
        let input = "note: |\n  first\n  second\n";

        assert_eq!(prettify(input), input);
    }

    /// Scenario: any document is formatted.
    /// Guarantees: formatting is value-preserving. This is the property that lets the style pass
    /// run on real output at all -- a cosmetic bug must never change what the engine reads.
    #[test]
    fn formatting_preserves_the_parsed_value() {
        let input = "nodes:\n  a:\n    type: x\n    outputs:\n    - default\nconnections:\n- from: a\n  to:\n  - b\n";

        let out = prettify(input);

        let before: serde_yaml::Value = serde_yaml::from_str(input).expect("input parses");
        let after: serde_yaml::Value = serde_yaml::from_str(&out).expect("output parses");
        assert_eq!(before, after);
    }
}
