// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Simple in-process formatter for UniLang source code.
//!
//! Applies the same rules as `unilang fmt`:
//!   - Replace leading tabs with 4 spaces
//!   - Trim trailing whitespace
//!   - Collapse 3+ consecutive blank lines to 2
//!   - Ensure file ends with a single newline

use tower_lsp::lsp_types::{Position, Range, TextEdit};

/// Format `source` and return a single `TextEdit` that replaces the entire
/// document with the formatted content.
///
/// Returns an empty `Vec` if the source is already correctly formatted.
pub fn format_document(source: &str) -> Vec<TextEdit> {
    let formatted = apply_formatting(source);

    // Only emit an edit when there is actually a change.
    if formatted == source {
        return Vec::new();
    }

    // Compute the end position of the original source so we can replace it all.
    let end = source_end_position(source);

    vec![TextEdit {
        range: Range {
            start: Position::new(0, 0),
            end,
        },
        new_text: formatted,
    }]
}

/// Apply formatting rules to `source` and return the result.
fn apply_formatting(source: &str) -> String {
    // Split into lines, preserving whether the source had a trailing newline.
    let lines: Vec<&str> = source.split('\n').collect();

    // Strip the implicit empty string that `split` appends for a trailing '\n'.
    let line_count = if source.ends_with('\n') && lines.last() == Some(&"") {
        lines.len() - 1
    } else {
        lines.len()
    };

    let mut out: Vec<String> = Vec::with_capacity(line_count);
    let mut consecutive_blank: u32 = 0;

    for raw in &lines[..line_count] {
        // 1. Replace leading tabs with 4 spaces (one tab = one indent level).
        let mut line = String::new();
        let mut chars = raw.chars().peekable();
        while let Some(&c) = chars.peek() {
            if c == '\t' {
                line.push_str("    ");
                chars.next();
            } else {
                break;
            }
        }
        for c in chars {
            line.push(c);
        }

        // 2. Trim trailing whitespace.
        let line = line.trim_end().to_string();

        // 3. Collapse 3+ consecutive blank lines to 2.
        if line.is_empty() {
            consecutive_blank += 1;
            if consecutive_blank > 2 {
                continue; // drop this blank line
            }
        } else {
            consecutive_blank = 0;
        }

        out.push(line);
    }

    // 4. Ensure the file ends with exactly one newline.
    // Join lines with '\n' and append final '\n'.
    let mut result = out.join("\n");
    result.push('\n');
    result
}

/// Return the LSP `Position` of the very end of `source`.
fn source_end_position(source: &str) -> Position {
    let mut line = 0u32;
    let mut col = 0u32;
    for ch in source.chars() {
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    Position::new(line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_replacement() {
        let src = "\tdef foo():\n\t\tpass\n";
        let result = apply_formatting(src);
        assert!(result.starts_with("    def foo():"));
        assert!(result.contains("        pass"));
    }

    #[test]
    fn test_trailing_whitespace() {
        let src = "val x = 1   \nval y = 2\n";
        let result = apply_formatting(src);
        assert_eq!(result, "val x = 1\nval y = 2\n");
    }

    #[test]
    fn test_collapse_blank_lines() {
        let src = "a\n\n\n\n\nb\n";
        let result = apply_formatting(src);
        // Should have at most 2 consecutive blank lines.
        assert!(!result.contains("\n\n\n\n"));
        assert!(result.contains("a"));
        assert!(result.contains("b"));
    }

    #[test]
    fn test_ensures_trailing_newline() {
        let src = "val x = 1";
        let result = apply_formatting(src);
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_no_edit_when_clean() {
        let src = "val x = 1\n";
        let edits = format_document(src);
        assert!(edits.is_empty());
    }

    #[test]
    fn test_format_document_returns_edit() {
        let src = "\tval x = 1  \n";
        let edits = format_document(src);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].new_text, "    val x = 1\n");
    }
}
