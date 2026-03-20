// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Auto-completion for UniLang keywords, types, and common patterns.
//!
//! Provides completion items categorized by origin (Python, Java,
//! UniLang-specific) with snippet support for common constructs.

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat, Position};

/// Return completion items relevant to the current cursor position.
///
/// Currently provides all keywords and common snippets regardless of
/// context. A future enhancement could filter based on the token
/// preceding the cursor.
pub fn get_completions(source: &str, position: Position) -> Vec<CompletionItem> {
    let prefix = extract_prefix(source, position);
    let mut items = Vec::new();

    // Keyword completions.
    for entry in KEYWORD_COMPLETIONS {
        if prefix.is_empty() || entry.label.starts_with(&prefix) {
            items.push(CompletionItem {
                label: entry.label.to_string(),
                kind: Some(entry.kind),
                detail: Some(entry.detail.to_string()),
                insert_text: entry.insert_text.map(|s| s.to_string()),
                insert_text_format: entry.insert_text.map(|_| InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }
    }

    // Snippet completions.
    for entry in SNIPPET_COMPLETIONS {
        if prefix.is_empty() || entry.label.starts_with(&prefix) {
            items.push(CompletionItem {
                label: entry.label.to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some(entry.detail.to_string()),
                insert_text: Some(entry.insert_text.to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }
    }

    items
}

/// Extract the partial word the user has typed so far at the cursor position.
fn extract_prefix(source: &str, position: Position) -> String {
    let Some(line) = source.lines().nth(position.line as usize) else {
        return String::new();
    };

    let col = (position.character as usize).min(line.len());
    let before = &line[..col];

    let start = before
        .rfind(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);

    before[start..].to_string()
}

struct KeywordEntry {
    label: &'static str,
    kind: CompletionItemKind,
    detail: &'static str,
    /// Optional snippet insert text (with `$1`, `$2` placeholders).
    insert_text: Option<&'static str>,
}

struct SnippetEntry {
    label: &'static str,
    detail: &'static str,
    insert_text: &'static str,
}

static KEYWORD_COMPLETIONS: &[KeywordEntry] = &[
    // ── Shared Keywords ──────────────────────────────────
    KeywordEntry {
        label: "class",
        kind: CompletionItemKind::KEYWORD,
        detail: "Declare a class (shared)",
        insert_text: Some("class ${1:ClassName}"),
    },
    KeywordEntry {
        label: "if",
        kind: CompletionItemKind::KEYWORD,
        detail: "Conditional branch (shared)",
        insert_text: Some("if ${1:condition}"),
    },
    KeywordEntry {
        label: "else",
        kind: CompletionItemKind::KEYWORD,
        detail: "Else branch (shared)",
        insert_text: None,
    },
    KeywordEntry {
        label: "for",
        kind: CompletionItemKind::KEYWORD,
        detail: "Loop (shared)",
        insert_text: Some("for ${1:item} in ${2:iterable}"),
    },
    KeywordEntry {
        label: "while",
        kind: CompletionItemKind::KEYWORD,
        detail: "While loop (shared)",
        insert_text: Some("while ${1:condition}"),
    },
    KeywordEntry {
        label: "return",
        kind: CompletionItemKind::KEYWORD,
        detail: "Return value (shared)",
        insert_text: Some("return ${1}"),
    },
    KeywordEntry {
        label: "import",
        kind: CompletionItemKind::KEYWORD,
        detail: "Import module (shared)",
        insert_text: Some("import ${1:module}"),
    },
    KeywordEntry {
        label: "try",
        kind: CompletionItemKind::KEYWORD,
        detail: "Exception handling (shared)",
        insert_text: None,
    },
    KeywordEntry {
        label: "finally",
        kind: CompletionItemKind::KEYWORD,
        detail: "Finally block (shared)",
        insert_text: None,
    },
    KeywordEntry {
        label: "continue",
        kind: CompletionItemKind::KEYWORD,
        detail: "Continue loop (shared)",
        insert_text: None,
    },
    KeywordEntry {
        label: "break",
        kind: CompletionItemKind::KEYWORD,
        detail: "Break loop (shared)",
        insert_text: None,
    },
    KeywordEntry {
        label: "assert",
        kind: CompletionItemKind::KEYWORD,
        detail: "Assert condition (shared)",
        insert_text: Some("assert ${1:condition}"),
    },
    // ── Python-origin Keywords ───────────────────────────
    KeywordEntry {
        label: "def",
        kind: CompletionItemKind::KEYWORD,
        detail: "Define function (Python)",
        insert_text: Some("def ${1:name}(${2:params})"),
    },
    KeywordEntry {
        label: "elif",
        kind: CompletionItemKind::KEYWORD,
        detail: "Else-if branch (Python)",
        insert_text: Some("elif ${1:condition}"),
    },
    KeywordEntry {
        label: "except",
        kind: CompletionItemKind::KEYWORD,
        detail: "Catch exception (Python)",
        insert_text: Some("except ${1:ExceptionType} as ${2:e}"),
    },
    KeywordEntry {
        label: "raise",
        kind: CompletionItemKind::KEYWORD,
        detail: "Raise exception (Python)",
        insert_text: Some("raise ${1:Exception}(${2:message})"),
    },
    KeywordEntry {
        label: "pass",
        kind: CompletionItemKind::KEYWORD,
        detail: "No-op (Python)",
        insert_text: None,
    },
    KeywordEntry {
        label: "yield",
        kind: CompletionItemKind::KEYWORD,
        detail: "Yield value (Python)",
        insert_text: Some("yield ${1}"),
    },
    KeywordEntry {
        label: "async",
        kind: CompletionItemKind::KEYWORD,
        detail: "Async function (Python)",
        insert_text: None,
    },
    KeywordEntry {
        label: "await",
        kind: CompletionItemKind::KEYWORD,
        detail: "Await expression (Python)",
        insert_text: Some("await ${1:expr}"),
    },
    KeywordEntry {
        label: "with",
        kind: CompletionItemKind::KEYWORD,
        detail: "Context manager (Python)",
        insert_text: Some("with ${1:expr} as ${2:name}"),
    },
    KeywordEntry {
        label: "as",
        kind: CompletionItemKind::KEYWORD,
        detail: "Alias (Python)",
        insert_text: None,
    },
    KeywordEntry {
        label: "from",
        kind: CompletionItemKind::KEYWORD,
        detail: "Import from (Python)",
        insert_text: Some("from ${1:module} import ${2:name}"),
    },
    KeywordEntry {
        label: "in",
        kind: CompletionItemKind::KEYWORD,
        detail: "Membership/iteration (Python)",
        insert_text: None,
    },
    KeywordEntry {
        label: "is",
        kind: CompletionItemKind::KEYWORD,
        detail: "Identity test (Python)",
        insert_text: None,
    },
    KeywordEntry {
        label: "not",
        kind: CompletionItemKind::KEYWORD,
        detail: "Logical not (Python)",
        insert_text: None,
    },
    KeywordEntry {
        label: "and",
        kind: CompletionItemKind::KEYWORD,
        detail: "Logical and (Python)",
        insert_text: None,
    },
    KeywordEntry {
        label: "or",
        kind: CompletionItemKind::KEYWORD,
        detail: "Logical or (Python)",
        insert_text: None,
    },
    KeywordEntry {
        label: "lambda",
        kind: CompletionItemKind::KEYWORD,
        detail: "Anonymous function (Python)",
        insert_text: Some("lambda ${1:args}: ${2:expr}"),
    },
    KeywordEntry {
        label: "nonlocal",
        kind: CompletionItemKind::KEYWORD,
        detail: "Nonlocal variable (Python)",
        insert_text: Some("nonlocal ${1:var}"),
    },
    KeywordEntry {
        label: "global",
        kind: CompletionItemKind::KEYWORD,
        detail: "Global variable (Python)",
        insert_text: Some("global ${1:var}"),
    },
    KeywordEntry {
        label: "del",
        kind: CompletionItemKind::KEYWORD,
        detail: "Delete variable (Python)",
        insert_text: Some("del ${1:var}"),
    },
    KeywordEntry {
        label: "match",
        kind: CompletionItemKind::KEYWORD,
        detail: "Pattern matching (Python)",
        insert_text: Some("match ${1:value}"),
    },
    KeywordEntry {
        label: "case",
        kind: CompletionItemKind::KEYWORD,
        detail: "Match/switch case",
        insert_text: Some("case ${1:pattern}"),
    },
    // ── Java-origin Keywords ─────────────────────────────
    KeywordEntry {
        label: "public",
        kind: CompletionItemKind::KEYWORD,
        detail: "Public access (Java)",
        insert_text: None,
    },
    KeywordEntry {
        label: "private",
        kind: CompletionItemKind::KEYWORD,
        detail: "Private access (Java)",
        insert_text: None,
    },
    KeywordEntry {
        label: "protected",
        kind: CompletionItemKind::KEYWORD,
        detail: "Protected access (Java)",
        insert_text: None,
    },
    KeywordEntry {
        label: "static",
        kind: CompletionItemKind::KEYWORD,
        detail: "Static member (Java)",
        insert_text: None,
    },
    KeywordEntry {
        label: "final",
        kind: CompletionItemKind::KEYWORD,
        detail: "Final/constant (Java)",
        insert_text: None,
    },
    KeywordEntry {
        label: "abstract",
        kind: CompletionItemKind::KEYWORD,
        detail: "Abstract class/method (Java)",
        insert_text: None,
    },
    KeywordEntry {
        label: "interface",
        kind: CompletionItemKind::KEYWORD,
        detail: "Interface type (Java)",
        insert_text: Some("interface ${1:Name}"),
    },
    KeywordEntry {
        label: "enum",
        kind: CompletionItemKind::KEYWORD,
        detail: "Enum type (Java)",
        insert_text: Some("enum ${1:Name}"),
    },
    KeywordEntry {
        label: "extends",
        kind: CompletionItemKind::KEYWORD,
        detail: "Inherit class (Java)",
        insert_text: Some("extends ${1:ClassName}"),
    },
    KeywordEntry {
        label: "implements",
        kind: CompletionItemKind::KEYWORD,
        detail: "Implement interface (Java)",
        insert_text: Some("implements ${1:InterfaceName}"),
    },
    KeywordEntry {
        label: "new",
        kind: CompletionItemKind::KEYWORD,
        detail: "Create instance (Java)",
        insert_text: Some("new ${1:ClassName}(${2})"),
    },
    KeywordEntry {
        label: "this",
        kind: CompletionItemKind::KEYWORD,
        detail: "Current instance (Java)",
        insert_text: None,
    },
    KeywordEntry {
        label: "super",
        kind: CompletionItemKind::KEYWORD,
        detail: "Parent class (Java)",
        insert_text: None,
    },
    KeywordEntry {
        label: "void",
        kind: CompletionItemKind::KEYWORD,
        detail: "No return type (Java)",
        insert_text: None,
    },
    KeywordEntry {
        label: "throws",
        kind: CompletionItemKind::KEYWORD,
        detail: "Exception declaration (Java)",
        insert_text: Some("throws ${1:Exception}"),
    },
    KeywordEntry {
        label: "throw",
        kind: CompletionItemKind::KEYWORD,
        detail: "Throw exception (Java)",
        insert_text: Some("throw ${1:exception}"),
    },
    KeywordEntry {
        label: "catch",
        kind: CompletionItemKind::KEYWORD,
        detail: "Catch exception (Java)",
        insert_text: Some("catch (${1:Exception} ${2:e})"),
    },
    KeywordEntry {
        label: "synchronized",
        kind: CompletionItemKind::KEYWORD,
        detail: "Thread-safe block (Java)",
        insert_text: Some("synchronized (${1:lock})"),
    },
    KeywordEntry {
        label: "volatile",
        kind: CompletionItemKind::KEYWORD,
        detail: "Volatile field (Java)",
        insert_text: None,
    },
    KeywordEntry {
        label: "transient",
        kind: CompletionItemKind::KEYWORD,
        detail: "Non-serialized field (Java)",
        insert_text: None,
    },
    KeywordEntry {
        label: "native",
        kind: CompletionItemKind::KEYWORD,
        detail: "Native method (Java)",
        insert_text: None,
    },
    KeywordEntry {
        label: "instanceof",
        kind: CompletionItemKind::KEYWORD,
        detail: "Type check (Java)",
        insert_text: Some("instanceof ${1:Type}"),
    },
    KeywordEntry {
        label: "switch",
        kind: CompletionItemKind::KEYWORD,
        detail: "Switch statement (Java)",
        insert_text: Some("switch (${1:value})"),
    },
    KeywordEntry {
        label: "default",
        kind: CompletionItemKind::KEYWORD,
        detail: "Default case (Java)",
        insert_text: None,
    },
    KeywordEntry {
        label: "do",
        kind: CompletionItemKind::KEYWORD,
        detail: "Do-while loop (Java)",
        insert_text: None,
    },
    // ── UniLang-specific Keywords ────────────────────────
    KeywordEntry {
        label: "bridge",
        kind: CompletionItemKind::KEYWORD,
        detail: "Interop bridge (UniLang)",
        insert_text: Some("bridge ${1:name}"),
    },
    KeywordEntry {
        label: "vm",
        kind: CompletionItemKind::KEYWORD,
        detail: "VM target (UniLang)",
        insert_text: Some("vm ${1:target}"),
    },
    KeywordEntry {
        label: "interop",
        kind: CompletionItemKind::KEYWORD,
        detail: "Interop declaration (UniLang)",
        insert_text: None,
    },
    KeywordEntry {
        label: "val",
        kind: CompletionItemKind::KEYWORD,
        detail: "Immutable variable (UniLang)",
        insert_text: Some("val ${1:name} = ${2:value}"),
    },
    KeywordEntry {
        label: "var",
        kind: CompletionItemKind::KEYWORD,
        detail: "Mutable variable (UniLang)",
        insert_text: Some("var ${1:name} = ${2:value}"),
    },
    KeywordEntry {
        label: "const",
        kind: CompletionItemKind::KEYWORD,
        detail: "Compile-time constant (UniLang)",
        insert_text: Some("const ${1:NAME} = ${2:value}"),
    },
    // ── Types ────────────────────────────────────────────
    KeywordEntry {
        label: "int",
        kind: CompletionItemKind::TYPE_PARAMETER,
        detail: "Integer type",
        insert_text: None,
    },
    KeywordEntry {
        label: "float",
        kind: CompletionItemKind::TYPE_PARAMETER,
        detail: "Float type",
        insert_text: None,
    },
    KeywordEntry {
        label: "str",
        kind: CompletionItemKind::TYPE_PARAMETER,
        detail: "String type",
        insert_text: None,
    },
    KeywordEntry {
        label: "bool",
        kind: CompletionItemKind::TYPE_PARAMETER,
        detail: "Boolean type",
        insert_text: None,
    },
    KeywordEntry {
        label: "list",
        kind: CompletionItemKind::TYPE_PARAMETER,
        detail: "List type (Python-style)",
        insert_text: None,
    },
    KeywordEntry {
        label: "dict",
        kind: CompletionItemKind::TYPE_PARAMETER,
        detail: "Dictionary type (Python-style)",
        insert_text: None,
    },
    KeywordEntry {
        label: "set",
        kind: CompletionItemKind::TYPE_PARAMETER,
        detail: "Set type (Python-style)",
        insert_text: None,
    },
    KeywordEntry {
        label: "tuple",
        kind: CompletionItemKind::TYPE_PARAMETER,
        detail: "Tuple type (Python-style)",
        insert_text: None,
    },
    KeywordEntry {
        label: "String",
        kind: CompletionItemKind::TYPE_PARAMETER,
        detail: "String type (Java-style)",
        insert_text: None,
    },
    KeywordEntry {
        label: "List",
        kind: CompletionItemKind::TYPE_PARAMETER,
        detail: "List interface (Java-style)",
        insert_text: None,
    },
    KeywordEntry {
        label: "Map",
        kind: CompletionItemKind::TYPE_PARAMETER,
        detail: "Map interface (Java-style)",
        insert_text: None,
    },
    KeywordEntry {
        label: "Set",
        kind: CompletionItemKind::TYPE_PARAMETER,
        detail: "Set interface (Java-style)",
        insert_text: None,
    },
    KeywordEntry {
        label: "Object",
        kind: CompletionItemKind::TYPE_PARAMETER,
        detail: "Base object type",
        insert_text: None,
    },
    KeywordEntry {
        label: "Array",
        kind: CompletionItemKind::TYPE_PARAMETER,
        detail: "Array type",
        insert_text: None,
    },
    // ── Built-in Functions ───────────────────────────────
    KeywordEntry {
        label: "print",
        kind: CompletionItemKind::FUNCTION,
        detail: "Print to stdout",
        insert_text: Some("print(${1})"),
    },
    KeywordEntry {
        label: "len",
        kind: CompletionItemKind::FUNCTION,
        detail: "Get length/size",
        insert_text: Some("len(${1})"),
    },
    KeywordEntry {
        label: "range",
        kind: CompletionItemKind::FUNCTION,
        detail: "Generate integer range",
        insert_text: Some("range(${1:stop})"),
    },
    KeywordEntry {
        label: "type",
        kind: CompletionItemKind::FUNCTION,
        detail: "Get type of object",
        insert_text: Some("type(${1})"),
    },
    KeywordEntry {
        label: "isinstance",
        kind: CompletionItemKind::FUNCTION,
        detail: "Check instance type",
        insert_text: Some("isinstance(${1:obj}, ${2:Type})"),
    },
    KeywordEntry {
        label: "input",
        kind: CompletionItemKind::FUNCTION,
        detail: "Read user input",
        insert_text: Some("input(${1:prompt})"),
    },
    KeywordEntry {
        label: "open",
        kind: CompletionItemKind::FUNCTION,
        detail: "Open a file",
        insert_text: Some("open(${1:path}, ${2:mode})"),
    },
    KeywordEntry {
        label: "enumerate",
        kind: CompletionItemKind::FUNCTION,
        detail: "Enumerate iterable",
        insert_text: Some("enumerate(${1:iterable})"),
    },
    KeywordEntry {
        label: "zip",
        kind: CompletionItemKind::FUNCTION,
        detail: "Zip iterables together",
        insert_text: Some("zip(${1:a}, ${2:b})"),
    },
    KeywordEntry {
        label: "map",
        kind: CompletionItemKind::FUNCTION,
        detail: "Apply function to iterable",
        insert_text: Some("map(${1:func}, ${2:iterable})"),
    },
    KeywordEntry {
        label: "filter",
        kind: CompletionItemKind::FUNCTION,
        detail: "Filter iterable",
        insert_text: Some("filter(${1:func}, ${2:iterable})"),
    },
];

static SNIPPET_COMPLETIONS: &[SnippetEntry] = &[
    SnippetEntry {
        label: "def (Python function)",
        detail: "Define a Python-style function",
        insert_text: "def ${1:name}(${2:params}):\n    ${3:pass}",
    },
    SnippetEntry {
        label: "class (Python)",
        detail: "Define a Python-style class",
        insert_text: "class ${1:Name}:\n    def __init__(self${2:, params}):\n        ${3:pass}",
    },
    SnippetEntry {
        label: "class (Java)",
        detail: "Define a Java-style class",
        insert_text: "class ${1:Name} {\n    ${2}\n}",
    },
    SnippetEntry {
        label: "for-in (Python)",
        detail: "Python-style for loop",
        insert_text: "for ${1:item} in ${2:iterable}:\n    ${3}",
    },
    SnippetEntry {
        label: "for (Java)",
        detail: "Java-style for loop",
        insert_text: "for (${1:int i = 0}; ${2:i < n}; ${3:i++}) {\n    ${4}\n}",
    },
    SnippetEntry {
        label: "if-else (Python)",
        detail: "Python-style if-else",
        insert_text: "if ${1:condition}:\n    ${2}\nelse:\n    ${3}",
    },
    SnippetEntry {
        label: "if-else (Java)",
        detail: "Java-style if-else",
        insert_text: "if (${1:condition}) {\n    ${2}\n} else {\n    ${3}\n}",
    },
    SnippetEntry {
        label: "try-except (Python)",
        detail: "Python-style exception handling",
        insert_text: "try:\n    ${1}\nexcept ${2:Exception} as ${3:e}:\n    ${4}",
    },
    SnippetEntry {
        label: "try-catch (Java)",
        detail: "Java-style exception handling",
        insert_text: "try {\n    ${1}\n} catch (${2:Exception} ${3:e}) {\n    ${4}\n}",
    },
    SnippetEntry {
        label: "while (Python)",
        detail: "Python-style while loop",
        insert_text: "while ${1:condition}:\n    ${2}",
    },
    SnippetEntry {
        label: "while (Java)",
        detail: "Java-style while loop",
        insert_text: "while (${1:condition}) {\n    ${2}\n}",
    },
    SnippetEntry {
        label: "match",
        detail: "Pattern matching block",
        insert_text: "match ${1:value}:\n    case ${2:pattern}:\n        ${3}",
    },
    SnippetEntry {
        label: "with",
        detail: "Context manager",
        insert_text: "with ${1:expr} as ${2:name}:\n    ${3}",
    },
    SnippetEntry {
        label: "async def",
        detail: "Async function definition",
        insert_text: "async def ${1:name}(${2:params}):\n    ${3}",
    },
    SnippetEntry {
        label: "interface",
        detail: "Interface declaration",
        insert_text: "interface ${1:Name} {\n    ${2}\n}",
    },
    SnippetEntry {
        label: "enum",
        detail: "Enum declaration",
        insert_text: "enum ${1:Name} {\n    ${2}\n}",
    },
    SnippetEntry {
        label: "main",
        detail: "Main entry point",
        insert_text: "def main():\n    ${1}\n\nif __name__ == \"__main__\":\n    main()",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completions_with_prefix() {
        let items = get_completions("de", Position::new(0, 2));
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"def"));
        assert!(labels.contains(&"default"));
        assert!(labels.contains(&"del"));
        assert!(!labels.contains(&"class"));
    }

    #[test]
    fn test_completions_empty_prefix_returns_all() {
        let items = get_completions("", Position::new(0, 0));
        // Should contain at minimum all keywords.
        assert!(items.len() > 50);
    }

    #[test]
    fn test_extract_prefix() {
        assert_eq!(extract_prefix("def foo", Position::new(0, 3)), "def");
        assert_eq!(extract_prefix("  imp", Position::new(0, 5)), "imp");
    }
}
