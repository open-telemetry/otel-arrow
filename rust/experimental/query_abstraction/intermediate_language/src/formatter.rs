use std::fmt::{Debug, Formatter, Result};

/// Helper functions for formatting tree-like structures in Debug implementations
pub struct TreeFormatter;

impl TreeFormatter {
    /// Format an element as a tree node with the given label and children
    pub fn write_node(
        f: &mut Formatter<'_>,
        label: &str,
        indent: &str,
        children: &[(&str, &dyn Debug)],
    ) -> Result {
        writeln!(f, "{}", label)?;
        
        let last_idx = children.len().saturating_sub(1);
        for (i, (child_label, child)) in children.iter().enumerate() {
            if i == last_idx {
                writeln!(f, "{}└── {}: {:?}", indent, child_label, child)?;
            } else {
                writeln!(f, "{}├── {}: {:?}", indent, child_label, child)?;
            }
        }
        
        Ok(())
    }
    
    /// Format an item with custom indentation
    pub fn write_with_indent<T: Debug>(
        f: &mut Formatter<'_>,
        item: &T, 
        prefix: &str,
        indent: &str,
    ) -> Result {
        write!(f, "{}{}", prefix, indent)?;
        item.fmt(f)
    }
    
    /// Format a tree branch with a label
    pub fn write_branch(
        f: &mut Formatter<'_>,
        label: &str,
        indent: &str,
        is_last: bool,
    ) -> Result {
        let branch = if is_last { "└── " } else { "├── " };
        writeln!(f, "{}{}{}", indent, branch, label)
    }
    
    /// Format a nested debug item with appropriate indentation
    pub fn write_nested_item<T: Debug + ?Sized>(
        f: &mut Formatter<'_>,
        item: &T,
        indent: &str,
        is_last: bool,
    ) -> Result {
        let branch = if is_last { "└── " } else { "├── " };
        write!(f, "{}{}", indent, branch)?;
        item.fmt(f)
    }
    
    /// Get the appropriate continuation indent based on whether an item is the last one
    pub fn continuation_indent(indent: &str, is_last: bool) -> String {
        if is_last {
            format!("{}    ", indent)
        } else {
            format!("{}│   ", indent)
        }
    }
}