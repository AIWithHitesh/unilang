// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

use crate::span::{LineCol, SourceId, Span};

/// Represents a loaded source file.
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub id: SourceId,
    pub name: String,
    pub content: String,
    /// Byte offsets of each line start (cached on creation).
    line_starts: Vec<u32>,
}

impl SourceFile {
    pub fn new(id: SourceId, name: String, content: String) -> Self {
        let line_starts = std::iter::once(0)
            .chain(content.match_indices('\n').map(|(i, _)| (i + 1) as u32))
            .collect();
        Self {
            id,
            name,
            content,
            line_starts,
        }
    }

    /// Resolve a byte offset to a 1-indexed line and column.
    pub fn line_col(&self, offset: u32) -> LineCol {
        let offset = offset as usize;
        let line_idx = self
            .line_starts
            .partition_point(|&start| (start as usize) <= offset)
            .saturating_sub(1);
        let line_start = self.line_starts[line_idx] as usize;
        let col = offset - line_start;
        LineCol {
            line: (line_idx + 1) as u32,
            col: (col + 1) as u32,
        }
    }

    /// Get the source text for a given span.
    pub fn slice(&self, span: Span) -> &str {
        &self.content[span.start as usize..span.end as usize]
    }

    /// Total number of lines.
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Get the text of a specific line (1-indexed).
    pub fn line_text(&self, line: u32) -> &str {
        let idx = (line - 1) as usize;
        let start = self.line_starts[idx] as usize;
        let end = self
            .line_starts
            .get(idx + 1)
            .map(|&s| s as usize)
            .unwrap_or(self.content.len());
        self.content[start..end]
            .trim_end_matches('\n')
            .trim_end_matches('\r')
    }
}

/// Manages multiple source files in a compilation session.
#[derive(Debug, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn add(&mut self, name: String, content: String) -> SourceId {
        let id = SourceId(self.files.len() as u32);
        self.files.push(SourceFile::new(id, name, content));
        id
    }

    pub fn get(&self, id: SourceId) -> &SourceFile {
        &self.files[id.0 as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_col() {
        let src = SourceFile::new(SourceId(0), "test".into(), "hello\nworld\nfoo".into());
        assert_eq!(src.line_col(0), LineCol { line: 1, col: 1 });
        assert_eq!(src.line_col(5), LineCol { line: 1, col: 6 });
        assert_eq!(src.line_col(6), LineCol { line: 2, col: 1 });
        assert_eq!(src.line_col(11), LineCol { line: 2, col: 6 });
        assert_eq!(src.line_col(12), LineCol { line: 3, col: 1 });
    }

    #[test]
    fn test_line_text() {
        let src = SourceFile::new(SourceId(0), "test".into(), "hello\nworld\nfoo".into());
        assert_eq!(src.line_text(1), "hello");
        assert_eq!(src.line_text(2), "world");
        assert_eq!(src.line_text(3), "foo");
    }
}
