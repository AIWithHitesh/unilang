// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

/// Tracks indentation levels and emits synthetic INDENT/DEDENT tokens.
///
/// Key behaviors:
/// - Maintains a stack of indentation levels (starts with [0]).
/// - When `brace_depth > 0`, indentation tracking is suspended.
/// - Indentation changes are detected at the start of each non-blank line.
#[derive(Debug)]
pub struct IndentTracker {
    /// Stack of indentation levels in spaces. Always has at least one entry (0).
    stack: Vec<u32>,
    /// Current nesting depth of `{ }`. When > 0, indent/dedent emission is suppressed.
    brace_depth: u32,
    /// Number of DEDENT tokens pending emission.
    pending_dedents: u32,
    /// Whether a pending INDENT token should be emitted.
    pending_indent: bool,
}

impl Default for IndentTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl IndentTracker {
    pub fn new() -> Self {
        Self {
            stack: vec![0],
            brace_depth: 0,
            pending_dedents: 0,
            pending_indent: false,
        }
    }

    /// Current indentation level (top of stack).
    pub fn current_level(&self) -> u32 {
        *self.stack.last().unwrap()
    }

    /// Whether indentation tracking is active (not inside braces).
    pub fn is_active(&self) -> bool {
        self.brace_depth == 0
    }

    /// Notify the tracker that a `{` was encountered.
    pub fn open_brace(&mut self) {
        self.brace_depth += 1;
    }

    /// Notify the tracker that a `}` was encountered.
    pub fn close_brace(&mut self) {
        self.brace_depth = self.brace_depth.saturating_sub(1);
    }

    /// Process indentation at the start of a new line.
    /// Returns the number of DEDENT tokens to emit (may be 0),
    /// and whether an INDENT token should be emitted.
    ///
    /// `indent_level` is the number of spaces at the start of the line.
    pub fn process_line_indent(&mut self, indent_level: u32) -> IndentChange {
        if !self.is_active() {
            return IndentChange::None;
        }

        let current = self.current_level();

        match indent_level.cmp(&current) {
            std::cmp::Ordering::Greater => {
                self.stack.push(indent_level);
                IndentChange::Indent
            }
            std::cmp::Ordering::Less => {
                let mut dedent_count = 0u32;
                while self.stack.len() > 1 && *self.stack.last().unwrap() > indent_level {
                    self.stack.pop();
                    dedent_count += 1;
                }
                // If the final level doesn't match exactly, it's an indentation error.
                // We still emit the dedents for error recovery.
                if *self.stack.last().unwrap() != indent_level
                    && indent_level > *self.stack.last().unwrap()
                {
                    self.stack.push(indent_level);
                }
                IndentChange::Dedent(dedent_count)
            }
            std::cmp::Ordering::Equal => IndentChange::None,
        }
    }

    /// Flush remaining dedents at end of file.
    /// Returns the number of DEDENT tokens to emit.
    pub fn flush_eof(&mut self) -> u32 {
        let count = (self.stack.len() as u32).saturating_sub(1);
        self.stack.truncate(1);
        count
    }

    /// Number of DEDENT tokens waiting to be emitted.
    pub fn pending_dedents(&self) -> u32 {
        self.pending_dedents
    }

    /// Set pending dedents (used by lexer to queue).
    pub fn set_pending_dedents(&mut self, n: u32) {
        self.pending_dedents = n;
    }

    /// Take one pending dedent (returns true if there was one).
    pub fn take_pending_dedent(&mut self) -> bool {
        if self.pending_dedents > 0 {
            self.pending_dedents -= 1;
            true
        } else {
            false
        }
    }

    pub fn set_pending_indent(&mut self, v: bool) {
        self.pending_indent = v;
    }

    pub fn take_pending_indent(&mut self) -> bool {
        if self.pending_indent {
            self.pending_indent = false;
            true
        } else {
            false
        }
    }
}

/// The result of processing indentation at a line start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentChange {
    /// No change in indentation.
    None,
    /// Indentation increased by one level.
    Indent,
    /// Indentation decreased by `n` levels.
    Dedent(u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_indent_dedent() {
        let mut tracker = IndentTracker::new();

        assert_eq!(tracker.process_line_indent(0), IndentChange::None);
        assert_eq!(tracker.process_line_indent(4), IndentChange::Indent);
        assert_eq!(tracker.process_line_indent(4), IndentChange::None);
        assert_eq!(tracker.process_line_indent(8), IndentChange::Indent);
        assert_eq!(tracker.process_line_indent(4), IndentChange::Dedent(1));
        assert_eq!(tracker.process_line_indent(0), IndentChange::Dedent(1));
    }

    #[test]
    fn test_multi_dedent() {
        let mut tracker = IndentTracker::new();

        tracker.process_line_indent(4);
        tracker.process_line_indent(8);
        tracker.process_line_indent(12);
        assert_eq!(tracker.process_line_indent(0), IndentChange::Dedent(3));
    }

    #[test]
    fn test_brace_suppresses_indent() {
        let mut tracker = IndentTracker::new();

        tracker.open_brace();
        assert_eq!(tracker.process_line_indent(8), IndentChange::None);
        assert_eq!(tracker.process_line_indent(0), IndentChange::None);
        tracker.close_brace();
        // Back to normal
        assert_eq!(tracker.process_line_indent(4), IndentChange::Indent);
    }

    #[test]
    fn test_eof_flush() {
        let mut tracker = IndentTracker::new();

        tracker.process_line_indent(4);
        tracker.process_line_indent(8);
        assert_eq!(tracker.flush_eof(), 2);
    }
}
