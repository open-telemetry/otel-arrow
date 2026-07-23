// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Task for managing the component inventory baseline (RFC 0001).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Component {
    pub id: String,
    pub category: String,
    pub description: Option<String>,
    pub file: String,
    pub line: usize,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BaselineComponent {
    pub id: String,
    pub category: String,
    pub description: Option<String>,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug)]
struct MissingAnnotation {
    file: String,
    line: usize,
    slice: String,
}

pub fn run(args: &[String]) -> anyhow::Result<()> {
    let mut check_path: Option<PathBuf> = None;
    let mut update_baseline = false;
    let mut format = "table".to_string();

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--check" => {
                if let Some(val) = iter.next() {
                    check_path = Some(PathBuf::from(val));
                } else {
                    check_path = Some(PathBuf::from("components-baseline.json"));
                }
            }
            "--update-baseline" => {
                update_baseline = true;
            }
            "--format" => {
                if let Some(val) = iter.next() {
                    format = val.clone();
                } else {
                    anyhow::bail!("Missing value for --format option");
                }
            }
            other => {
                if other.starts_with("--") {
                    anyhow::bail!("Unknown argument: {other}");
                } else if check_path.is_none() && !update_baseline {
                    // Positional arg interpreted as baseline check path
                    check_path = Some(PathBuf::from(other));
                }
            }
        }
    }

    // Default baseline location
    let default_baseline = PathBuf::from("components-baseline.json");

    let base_dir = std::env::current_dir()?;
    let (components, missing) = scan_workspace(&base_dir)?;

    if update_baseline {
        let path = check_path.unwrap_or(default_baseline);
        save_baseline(&path, &components)?;
        println!(
            "✨ Updated component inventory baseline saved to: {}",
            path.display()
        );
        return Ok(());
    }

    if let Some(path) = check_path {
        run_check(&path, &components, &missing)?;
    } else {
        match format.as_str() {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&components)?);
            }
            "yaml" => {
                print_yaml(&components);
            }
            _ => {
                print_table(&components);
            }
        }
    }

    Ok(())
}

fn scan_workspace(base_dir: &Path) -> anyhow::Result<(Vec<Component>, Vec<MissingAnnotation>)> {
    let mut components = Vec::new();
    let mut missing = Vec::new();

    // Walk crates/ directory recursively
    let crates_dir = base_dir.join("crates");
    if crates_dir.exists() {
        visit_dirs(&crates_dir, base_dir, &mut components, &mut missing)?;
    }

    // Sort components by ID for determinism
    components.sort_by(|a, b| a.id.cmp(&b.id));

    Ok((components, missing))
}

fn visit_dirs(
    dir: &Path,
    base_dir: &Path,
    components: &mut Vec<Component>,
    missing: &mut Vec<MissingAnnotation>,
) -> anyhow::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                // Ignore build/test/utility/macro directories
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name != "target"
                    && name != ".git"
                    && name != "tests"
                    && name != "benches"
                    && name != "examples"
                    && name != "engine-macros"
                    && name != "telemetry-macros"
                    && name != "validation"
                    && name != "otap-test-net"
                    && name != "otap-test-tls-certs"
                    && name != "quiver-e2e"
                {
                    visit_dirs(&path, base_dir, components, missing)?;
                }
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let file_components = scan_file(&path, base_dir)?;
                components.extend(file_components);

                let file_missing = find_missing_annotations(&path, base_dir)?;
                missing.extend(file_missing);
            }
        }
    }
    Ok(())
}

fn scan_file(file_path: &Path, base_dir: &Path) -> anyhow::Result<Vec<Component>> {
    let content = std::fs::read_to_string(file_path)?;
    let relative_path = file_path
        .strip_prefix(base_dir)
        .unwrap_or(file_path)
        .to_string_lossy()
        .into_owned();

    let mut components = Vec::new();
    let chars: Vec<char> = content.chars().collect();
    let mut idx = 0;

    let get_line_num =
        |idx: usize| -> usize { chars[..idx].iter().filter(|&&c| c == '\n').count() + 1 };

    while idx < chars.len() {
        // Skip single line comments
        if chars[idx..].starts_with(&['/', '/']) {
            while idx < chars.len() && chars[idx] != '\n' {
                idx += 1;
            }
            continue;
        }
        // Skip multi line comments
        if chars[idx..].starts_with(&['/', '*']) {
            idx += 2;
            while idx < chars.len() && !chars[idx..].starts_with(&['*', '/']) {
                idx += 1;
            }
            if idx < chars.len() {
                idx += 2; // skip "*/"
            }
            continue;
        }

        if chars[idx..].starts_with(&['#', '[']) {
            let mut path_chars = Vec::new();
            let mut check_idx = idx + 2;
            while check_idx < chars.len() {
                let c = chars[check_idx];
                if c == '(' || c == ']' {
                    break;
                }
                if !c.is_whitespace() {
                    path_chars.push(c);
                }
                check_idx += 1;
            }
            let macro_path: String = path_chars.into_iter().collect();
            let starts_with_macro = macro_path == "component_inventory"
                || macro_path == "otap_df_engine::component_inventory";
            if starts_with_macro {
                let macro_start_line = get_line_num(idx);
                let mut p_idx = idx + 2 + macro_path.len();
                while p_idx < chars.len() && chars[p_idx] != '(' {
                    p_idx += 1;
                }
                if p_idx < chars.len() {
                    let mut depth = 1;
                    let mut inside_idx = p_idx + 1;
                    let mut macro_args = String::new();
                    while inside_idx < chars.len() && depth > 0 {
                        let c = chars[inside_idx];
                        if c == '(' {
                            depth += 1;
                        } else if c == ')' {
                            depth -= 1;
                        }
                        if depth > 0 {
                            macro_args.push(c);
                        }
                        inside_idx += 1;
                    }

                    let id = extract_param_string(&macro_args, "id");
                    let category = extract_param_ident(&macro_args, "category").unwrap_or_default();
                    let description = extract_param_string(&macro_args, "description");

                    let mut attributes = BTreeMap::new();
                    if let Some(attr_start) = macro_args.find("attributes") {
                        if let Some(open_p) = macro_args[attr_start..].find('(') {
                            let abs_open = attr_start + open_p;
                            let mut attr_depth = 1;
                            let mut attr_args = String::new();
                            let mut attr_idx = abs_open + 1;
                            let macro_args_chars: Vec<char> = macro_args.chars().collect();
                            while attr_idx < macro_args_chars.len() && attr_depth > 0 {
                                let c = macro_args_chars[attr_idx];
                                if c == '(' {
                                    attr_depth += 1;
                                } else if c == ')' {
                                    attr_depth -= 1;
                                }
                                if attr_depth > 0 {
                                    attr_args.push(c);
                                }
                                attr_idx += 1;
                            }
                            attributes = parse_attributes(&attr_args);
                        }
                    }

                    let resolved_id = match id {
                        Some(val) => val,
                        None => {
                            let post_macro_str: String =
                                chars[inside_idx..].iter().take(2000).collect();
                            if let Some(name_pos) = post_macro_str.find("name") {
                                let mut s = &post_macro_str[name_pos + 4..];
                                s = s.trim_start_matches(|c: char| c.is_whitespace() || c == ':');
                                let const_name: String = s
                                    .chars()
                                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                                    .collect();
                                if !const_name.is_empty() {
                                    let mut resolved = None;
                                    let content_str: String = chars.iter().collect();
                                    for line in content_str.lines() {
                                        if line.trim().starts_with("//") {
                                            continue;
                                        }
                                        if (line.contains("const") || line.contains("static"))
                                            && line.contains(&const_name)
                                        {
                                            if let Some(eq_pos) = line.find('=') {
                                                if let Some(val) =
                                                    extract_string_literal(&line[eq_pos..])
                                                {
                                                    resolved = Some(val);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    resolved.unwrap_or_else(|| {
                                        format!("urn:derived:unknown:{}", const_name)
                                    })
                                } else {
                                    "urn:derived:unknown:name_field_not_parsed".to_string()
                                }
                            } else {
                                "urn:derived:unknown:no_name_field".to_string()
                            }
                        }
                    };

                    components.push(Component {
                        id: resolved_id,
                        category,
                        description,
                        file: relative_path.clone(),
                        line: macro_start_line,
                        attributes,
                    });

                    idx = inside_idx;
                    continue;
                }
            }
        }
        idx += 1;
    }

    Ok(components)
}

fn find_missing_annotations(
    file_path: &Path,
    base_dir: &Path,
) -> anyhow::Result<Vec<MissingAnnotation>> {
    let content = std::fs::read_to_string(file_path)?;
    let relative_path = file_path
        .strip_prefix(base_dir)
        .unwrap_or(file_path)
        .to_string_lossy()
        .into_owned();
    let lines: Vec<&str> = content.lines().collect();

    let mut missing = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("//") {
            continue;
        }
        if line.contains("#[distributed_slice(OTAP_")
            || line.contains("#[distributed_slice(otap_df_otap::OTAP_")
        {
            if let Some(start_idx) = line.find("OTAP_") {
                let slice_name: String = line[start_idx..]
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();

                let start_check = i.saturating_sub(15);
                let mut found_annotation = false;
                for check_line in &lines[start_check..i] {
                    if check_line.contains("component_inventory") {
                        found_annotation = true;
                        break;
                    }
                }

                if !found_annotation {
                    missing.push(MissingAnnotation {
                        file: relative_path.clone(),
                        line: i + 1,
                        slice: slice_name,
                    });
                }
            }
        }
    }

    Ok(missing)
}

fn extract_param_string(args: &str, key: &str) -> Option<String> {
    let parts: Vec<&str> = args.split(',').collect();
    for part in parts {
        let part = part.trim();
        if part.starts_with(key) {
            if let Some(eq_idx) = part.find('=') {
                let val_part = part[eq_idx + 1..].trim();
                return extract_string_literal(val_part);
            }
        }
    }
    None
}

fn extract_param_ident(args: &str, key: &str) -> Option<String> {
    let parts: Vec<&str> = args.split(',').collect();
    for part in parts {
        let part = part.trim();
        if part.starts_with(key) {
            if let Some(eq_idx) = part.find('=') {
                let val_part = part[eq_idx + 1..].trim();
                let ident: String = val_part
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !ident.is_empty() {
                    return Some(ident);
                }
            }
        }
    }
    None
}

fn extract_string_literal(input: &str) -> Option<String> {
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' {
            let mut s = String::new();
            while let Some(&nc) = chars.peek() {
                if nc == '"' {
                    chars.next();
                    return Some(s);
                } else if nc == '\\' {
                    chars.next();
                    if let Some(escaped) = chars.next() {
                        match escaped {
                            'n' => s.push('\n'),
                            'r' => s.push('\r'),
                            't' => s.push('\t'),
                            _ => s.push(escaped),
                        }
                    }
                } else {
                    s.push(chars.next().unwrap());
                }
            }
        }
    }
    None
}

fn parse_attributes(input: &str) -> BTreeMap<String, String> {
    let mut attrs = BTreeMap::new();
    let mut s = input;
    while let Some(eq_idx) = s.find('=') {
        let key_part = s[..eq_idx].trim();
        let key = key_part.trim_matches('"').trim_matches('\'').to_string();
        s = &s[eq_idx + 1..];
        if let Some(start_quote) = s.find('"') {
            s = &s[start_quote + 1..];
            let mut val = String::new();
            let mut escaped = false;
            let mut chars = s.chars().peekable();
            let mut consumed = 0;
            for c in &mut chars {
                consumed += c.len_utf8();
                if escaped {
                    match c {
                        'n' => val.push('\n'),
                        'r' => val.push('\r'),
                        't' => val.push('\t'),
                        _ => val.push(c),
                    }
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    break;
                } else {
                    val.push(c);
                }
            }
            attrs.insert(key, val);
            s = &s[consumed..];
            if let Some(comma_idx) = s.find(',') {
                s = &s[comma_idx + 1..];
            } else {
                break;
            }
        } else {
            break;
        }
    }
    attrs
}

fn save_baseline(path: &Path, components: &[Component]) -> anyhow::Result<()> {
    let baseline: Vec<BaselineComponent> = components
        .iter()
        .map(|c| BaselineComponent {
            id: c.id.clone(),
            category: c.category.clone(),
            description: c.description.clone(),
            attributes: c.attributes.clone(),
        })
        .collect();

    let file = std::fs::File::create(path)?;
    serde_json::to_writer_pretty(file, &baseline)?;
    Ok(())
}

fn load_baseline(path: &Path) -> anyhow::Result<Vec<BaselineComponent>> {
    let file = std::fs::File::open(path)?;
    let baseline: Vec<BaselineComponent> = serde_json::from_reader(file)?;
    Ok(baseline)
}

fn run_check(
    path: &Path,
    code_components: &[Component],
    missing_annotations: &[MissingAnnotation],
) -> anyhow::Result<()> {
    println!(
        "🚀 Verifying component inventory against baseline: {}...\n",
        path.display()
    );

    let baseline = load_baseline(path).unwrap_or_else(|_| {
        println!("⚠️ Warning: Baseline file not found or invalid. Treating baseline as empty.");
        Vec::new()
    });

    let mut baseline_map: BTreeMap<String, &BaselineComponent> = BTreeMap::new();
    for item in &baseline {
        baseline_map.insert(item.id.clone(), item);
    }

    let mut code_map: BTreeMap<String, &Component> = BTreeMap::new();
    for item in code_components {
        code_map.insert(item.id.clone(), item);
    }

    let mut new_components = Vec::new();
    let mut modified_components = Vec::new();
    let mut removed_components = Vec::new();

    for code_c in code_components {
        if let Some(base_c) = baseline_map.get(&code_c.id) {
            let mut diffs = Vec::new();
            if base_c.category != code_c.category {
                diffs.push(format!(
                    "Category differs: baseline '{}', code '{}'",
                    base_c.category, code_c.category
                ));
            }
            if base_c.description != code_c.description {
                diffs.push(format!(
                    "Description differs:\n  baseline: {:?}\n  code:     {:?}",
                    base_c.description, code_c.description
                ));
            }
            if base_c.attributes != code_c.attributes {
                diffs.push(format!(
                    "Attributes differ:\n  baseline: {:?}\n  code:     {:?}",
                    base_c.attributes, code_c.attributes
                ));
            }
            if !diffs.is_empty() {
                modified_components.push((code_c, diffs));
            }
        } else {
            new_components.push(code_c);
        }
    }

    for base_c in &baseline {
        if !code_map.contains_key(&base_c.id) {
            removed_components.push(base_c);
        }
    }

    let mut failed = false;

    if !new_components.is_empty() {
        failed = true;
        println!("🆕 NEW (annotated in code, not in baseline):");
        for c in &new_components {
            println!("  + {}  ({}:{})", c.id, c.file, c.line);
        }
        println!();
    }

    if !modified_components.is_empty() {
        failed = true;
        println!("🔄 MODIFIED (properties differ from baseline):");
        for (c, diffs) in &modified_components {
            println!("  * {}  ({}:{})", c.id, c.file, c.line);
            for diff in diffs {
                println!("    - {}", diff);
            }
        }
        println!();
    }

    if !removed_components.is_empty() {
        failed = true;
        println!("❌ REMOVED (in baseline, no annotation found in code):");
        for c in &removed_components {
            println!("  - {}", c.id);
        }
        println!();
    }

    if !missing_annotations.is_empty() {
        failed = true;
        println!("⚠️ MISSING annotation (factory static without #[component_inventory]):");
        for m in missing_annotations {
            println!("  ! {}:{}", m.file, m.line);
            println!(
                "    {} distributed_slice has no component_inventory annotation",
                m.slice
            );
        }
        println!();
    }

    if failed {
        println!("❌ STATUS: FAIL");
        std::process::exit(1);
    } else {
        println!("✅ STATUS: PASS (Component inventory perfectly matches baseline)");
    }

    Ok(())
}

fn print_table(components: &[Component]) {
    if components.is_empty() {
        println!("(No components registered)");
        return;
    }

    let mut max_id = 2;
    let mut max_cat = 8;
    let mut max_desc = 11;

    for c in components {
        max_id = max_id.max(c.id.len());
        max_cat = max_cat.max(c.category.len());
        max_desc = max_desc.max(c.description.as_deref().unwrap_or("").len());
    }

    max_desc = max_desc.min(80);

    let hr_line = format!(
        "┌─{}─┬─{}─┬─{}─┐",
        "─".repeat(max_id),
        "─".repeat(max_cat),
        "─".repeat(max_desc)
    );
    let separator = format!(
        "├─{}─┼─{}─┼─{}─┤",
        "─".repeat(max_id),
        "─".repeat(max_cat),
        "─".repeat(max_desc)
    );
    let footer_line = format!(
        "└─{}─┴─{}─┴─{}─┘",
        "─".repeat(max_id),
        "─".repeat(max_cat),
        "─".repeat(max_desc)
    );

    println!("{hr_line}");
    println!(
        "│ {:<id_w$} │ {:<cat_id$} │ {:<desc_w$} │",
        "ID",
        "Category",
        "Description",
        id_w = max_id,
        cat_id = max_cat,
        desc_w = max_desc
    );
    println!("{separator}");

    for c in components {
        let desc = c.description.as_deref().unwrap_or("");
        let formatted_desc = if desc.len() > max_desc {
            format!("{}...", &desc[..max_desc - 3])
        } else {
            desc.to_string()
        };

        println!(
            "│ {:<id_w$} │ {:<cat_id$} │ {:<desc_w$} │",
            c.id,
            c.category,
            formatted_desc,
            id_w = max_id,
            cat_id = max_cat,
            desc_w = max_desc
        );
    }
    println!("{footer_line}");
}

fn print_yaml(components: &[Component]) {
    for c in components {
        println!("- id: {}", c.id);
        println!("  category: {}", c.category);
        if let Some(desc) = &c.description {
            println!("  description: {:?}", desc);
        } else {
            println!("  description: null");
        }
        println!("  file: {}", c.file);
        println!("  line: {}", c.line);
        if c.attributes.is_empty() {
            println!("  attributes: {{}}");
        } else {
            println!("  attributes:");
            for (k, v) in &c.attributes {
                println!("    {}: {:?}", k, v);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Scenario: Extracting a string literal with escaped characters from a string
    // Guarantees: Escaped characters are correctly parsed and unescaped, returning the expected inner string
    #[test]
    fn test_extract_string_literal() {
        let input = r#" "hello \"world\"\nnext line" "#;
        let result = extract_string_literal(input);
        assert_eq!(result, Some("hello \"world\"\nnext line".to_string()));
    }

    // Scenario: Parsing key-value attributes inside the attributes(...) macro parameters
    // Guarantees: Values with quotes, commas, and spaces are correctly split and mapped to their respective keys
    #[test]
    fn test_parse_attributes() {
        let input = r#"port = "4317", protocol = "gRPC (HTTP/2)", auth = "mTLS (opt-in)""#;
        let attrs = parse_attributes(input);
        assert_eq!(attrs.len(), 3);
        assert_eq!(attrs.get("port"), Some(&"4317".to_string()));
        assert_eq!(attrs.get("protocol"), Some(&"gRPC (HTTP/2)".to_string()));
        assert_eq!(attrs.get("auth"), Some(&"mTLS (opt-in)".to_string()));
    }

    // Scenario: Extracting named macro arguments like id or category from macro tokens
    // Guarantees: Correctly parses identifier values and quoted string values respectively
    #[test]
    fn test_extract_param_string_and_ident() {
        let input = r#"id = "urn:otel:receiver:otlp", category = Receiver, description = "OTLP gRPC receiver""#;
        assert_eq!(
            extract_param_string(input, "id"),
            Some("urn:otel:receiver:otlp".to_string())
        );
        assert_eq!(
            extract_param_ident(input, "category"),
            Some("Receiver".to_string())
        );
        assert_eq!(
            extract_param_string(input, "description"),
            Some("OTLP gRPC receiver".to_string())
        );
    }
}
