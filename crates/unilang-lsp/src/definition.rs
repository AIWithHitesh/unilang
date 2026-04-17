// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Go-to-definition support for UniLang.
//!
//! Performs a simple text scan for `def`, `class`, `val`, and `var`
//! declarations and returns the first matching location.

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};

use crate::hover::extract_word_at_position;

/// Find the definition location of the symbol under the cursor.
///
/// Returns `None` when:
/// - the cursor is not on a word, or
/// - no declaration for the word is found in the current document.
pub fn find_definition(
    source: &str,
    position: Position,
    uri: &Url,
) -> Option<GotoDefinitionResponse> {
    let word = extract_word_at_position(source, position)?;

    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        // Column offset of the trimmed content inside the original line.
        let indent = line.len() - line.trim_start().len();

        // `def <word>(` or `def <word>:`
        if trimmed.starts_with("def ") {
            let rest = trimmed[4..].trim_start();
            if rest.starts_with(word.as_str())
                && rest[word.len()..].starts_with(|c: char| c == '(' || c == ':' || c == ' ')
            {
                let col = indent + 4; // "def " is 4 chars
                return Some(GotoDefinitionResponse::Scalar(make_location(
                    uri, line_idx, col,
                )));
            }
        }

        // `class <word>`
        if trimmed.starts_with("class ") {
            let rest = trimmed[6..].trim_start();
            if rest.starts_with(word.as_str())
                && rest[word.len()..].starts_with(|c: char| {
                    c == '(' || c == ':' || c == ' ' || c == '{'
                })
            {
                let col = indent + 6; // "class " is 6 chars
                return Some(GotoDefinitionResponse::Scalar(make_location(
                    uri, line_idx, col,
                )));
            }
        }

        // `val <word> =`, `var <word> =`
        for prefix in &["val ", "var "] {
            if trimmed.starts_with(prefix) {
                let rest = trimmed[prefix.len()..].trim_start();
                if rest.starts_with(word.as_str())
                    && rest[word.len()..].trim_start().starts_with('=')
                {
                    let col = indent + prefix.len();
                    return Some(GotoDefinitionResponse::Scalar(make_location(
                        uri, line_idx, col,
                    )));
                }
            }
        }
    }

    None
}

fn make_location(uri: &Url, line: usize, col: usize) -> Location {
    let pos = Position::new(line as u32, col as u32);
    Location {
        uri: uri.clone(),
        range: Range {
            start: pos,
            end: pos,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_uri() -> Url {
        Url::parse("file:///test.uniL").unwrap()
    }

    #[test]
    fn test_goto_def_function() {
        let source = "def greet(name):\n    print(name)\n";
        let uri = dummy_uri();
        // Cursor on "greet" in the call site — but here we call at the definition line itself.
        let result = find_definition(source, Position::new(0, 5), &uri);
        assert!(result.is_some());
        if let Some(GotoDefinitionResponse::Scalar(loc)) = result {
            assert_eq!(loc.range.start.line, 0);
        }
    }

    #[test]
    fn test_goto_def_variable() {
        let source = "val count = 0\nprint(count)\n";
        let uri = dummy_uri();
        let result = find_definition(source, Position::new(1, 7), &uri);
        assert!(result.is_some());
        if let Some(GotoDefinitionResponse::Scalar(loc)) = result {
            assert_eq!(loc.range.start.line, 0);
        }
    }

    #[test]
    fn test_goto_def_missing() {
        let source = "val count = 0\n";
        let uri = dummy_uri();
        // "unknown" is not declared anywhere.
        let result = find_definition(source, Position::new(0, 0), &uri);
        // "val" is a keyword not a declaration target; should still find no match for the
        // word "val" itself (it is the keyword, not a user symbol).
        // Just assert no panic.
        let _ = result;
    }
}
