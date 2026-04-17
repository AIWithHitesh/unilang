// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Hover information for UniLang tokens.
//!
//! When the user hovers over a keyword or identifier, this module
//! provides contextual documentation drawn from both the Python
//! and Java sides of UniLang.

use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};

/// Return hover info for the word at the given position, if any.
///
/// Resolution order:
/// 1. Built-in keywords (Python/Java/UniLang)
/// 2. Standard-library built-in functions
/// 3. Variable / function / class declaration scan (simple text search)
pub fn get_hover_info(source: &str, position: Position) -> Option<Hover> {
    let word = extract_word_at_position(source, position)?;

    // 1. Keyword lookup.
    if let Some(info) = keyword_description(&word) {
        return Some(make_hover(info.to_string()));
    }

    // 2. Stdlib function lookup.
    if let Some(info) = stdlib_description(&word) {
        return Some(make_hover(info.to_string()));
    }

    // 3. Variable / function / class declaration scan.
    if let Some(info) = find_declaration_hover(source, &word) {
        return Some(make_hover(info));
    }

    None
}

fn make_hover(value: String) -> Hover {
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value,
        }),
        range: None,
    }
}

/// Scan the source for a declaration of `word` and return a short Markdown
/// description if found.  Matches `val/var/let <word> =`, `def <word>`, and
/// `class <word>`.
fn find_declaration_hover(source: &str, word: &str) -> Option<String> {
    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        let line_no = line_idx + 1; // 1-based for display

        // `def <word>` or `def <word>(`
        if trimmed.starts_with("def ") {
            let rest = trimmed[4..].trim_start();
            if rest.starts_with(word)
                && rest[word.len()..].starts_with(|c: char| c == '(' || c == ':' || c == ' ')
            {
                return Some(format!(
                    "**{}** — function declared on line {}\n\n```\n{}\n```",
                    word, line_no, trimmed
                ));
            }
        }

        // `class <word>` or `class <word>(` or `class <word>:`
        if trimmed.starts_with("class ") {
            let rest = trimmed[6..].trim_start();
            if rest.starts_with(word)
                && rest[word.len()..].starts_with(|c: char| {
                    c == '(' || c == ':' || c == ' ' || c == '{'
                })
            {
                return Some(format!(
                    "**{}** — class declared on line {}\n\n```\n{}\n```",
                    word, line_no, trimmed
                ));
            }
        }

        // `val <word> =`, `var <word> =`, `let <word> =`
        for prefix in &["val ", "var ", "let "] {
            if trimmed.starts_with(prefix) {
                let rest = trimmed[prefix.len()..].trim_start();
                if rest.starts_with(word)
                    && rest[word.len()..].trim_start().starts_with('=')
                {
                    return Some(format!(
                        "**{}** — declared on line {}\n\n```\n{}\n```",
                        word, line_no, trimmed
                    ));
                }
            }
        }
    }
    None
}

/// Extract the word under the cursor at the given LSP position.
pub(crate) fn extract_word_at_position(source: &str, position: Position) -> Option<String> {
    let line_str = source.lines().nth(position.line as usize)?;
    let col = position.character as usize;

    if col > line_str.len() {
        return None;
    }

    // Walk backward from the cursor to find the word start.
    let before = &line_str[..col];
    let word_start = before
        .rfind(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);

    // Walk forward from the cursor to find the word end.
    let after = &line_str[col..];
    let word_end_offset = after
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(after.len());

    let word = &line_str[word_start..col + word_end_offset];
    if word.is_empty() {
        None
    } else {
        Some(word.to_string())
    }
}

/// Return a Markdown description for a UniLang standard-library built-in function.
fn stdlib_description(word: &str) -> Option<&'static str> {
    match word {
        "print" => Some("**print(value, ...)** — Print values to stdout. Accepts any number of arguments.\n\n```\nprint(\"Hello\", name)\n```"),
        "println" => Some("**println(value, ...)** — Print values to stdout with a newline."),
        "len" => Some("**len(collection)** — Return the number of items in a collection (list, dict, string).\n\n```\nlen([1, 2, 3])  # → 3\n```"),
        "range" => Some("**range(stop)** / **range(start, stop)** / **range(start, stop, step)** — Generate a sequence of integers.\n\n```\nfor i in range(10):\n    print(i)\n```"),
        "type" => Some("**type(value)** — Return the type name of a value as a string."),
        "abs" => Some("**abs(x)** — Return the absolute value of x."),
        "max" => Some("**max(a, b, ...)** — Return the largest of the given values."),
        "min" => Some("**min(a, b, ...)** — Return the smallest of the given values."),
        "sum" => Some("**sum(iterable)** — Return the sum of all elements."),
        "sorted" => Some("**sorted(iterable)** — Return a new sorted list from the iterable."),
        "reversed" => Some("**reversed(iterable)** — Return a reversed iterator."),
        "enumerate" => Some("**enumerate(iterable, start=0)** — Return (index, value) pairs."),
        "zip" => Some("**zip(iter1, iter2, ...)** — Pair elements from multiple iterables."),
        "map" => Some("**map(func, iterable)** — Apply a function to each element."),
        "filter" => Some("**filter(func, iterable)** — Filter elements by a predicate."),
        "any" => Some("**any(iterable)** — Return true if any element is truthy."),
        "all" => Some("**all(iterable)** — Return true if all elements are truthy."),
        "round" => Some("**round(x, ndigits=0)** — Round x to ndigits decimal places."),
        "append" => Some("**append(list, value)** — Append value to a list (mutates in place)."),
        "push" => Some("**push(list, value)** — Alias for append."),
        "pop" => Some("**pop(list)** — Remove and return the last element."),
        "keys" => Some("**keys(dict)** — Return all keys of a dictionary."),
        "values" => Some("**values(dict)** — Return all values of a dictionary."),
        "items" => Some("**items(dict)** — Return all key-value pairs of a dictionary."),
        "contains" => Some("**contains(collection, value)** — Return true if collection contains value."),
        "split" => Some("**split(string, sep?)** — Split string by separator."),
        "join" => Some("**join(sep, list)** — Join list items with separator."),
        "strip" => Some("**strip(string)** — Remove leading/trailing whitespace."),
        "upper" => Some("**upper(string)** — Convert string to uppercase."),
        "lower" => Some("**lower(string)** — Convert string to lowercase."),
        "replace" => Some("**replace(string, old, new)** — Replace occurrences of old with new."),
        "starts_with" => Some("**starts_with(string, prefix)** — Check if string starts with prefix."),
        "ends_with" => Some("**ends_with(string, suffix)** — Check if string ends with suffix."),
        "format" => Some("**format(template, ...)** — Format a string with placeholders."),
        "open" => Some("**open(path, mode?)** — Open a file for reading or writing."),
        "read_file" => Some("**read_file(path)** — Read the entire content of a file as a string."),
        "write_file" => Some("**write_file(path, content)** — Write content to a file."),
        "sleep" => Some("**sleep(seconds)** — Pause execution for the given number of seconds."),
        "now" => Some("**now()** — Return the current Unix timestamp in seconds."),
        "random" => Some("**random()** — Return a random float between 0.0 and 1.0."),
        "random_int" => Some("**random_int(min, max)** — Return a random integer between min and max."),
        "exit" => Some("**exit(code?)** — Exit the program with the given exit code (default 0)."),
        "sqrt" => Some("**sqrt(x)** — Return the square root of x."),
        "pow" => Some("**pow(base, exp)** — Return base raised to exp."),
        "floor" => Some("**floor(x)** — Return the floor of x."),
        "ceil" => Some("**ceil(x)** — Return the ceiling of x."),
        "pi" => Some("**pi** — Mathematical constant π ≈ 3.14159."),
        "e" => Some("**e** — Mathematical constant e ≈ 2.71828."),
        "json_encode" => Some("**json_encode(value)** — Serialize a value to a JSON string."),
        "json_decode" => Some("**json_decode(string)** — Parse a JSON string into a UniLang value."),
        "http_get" => Some("**http_get(url, headers?)** — Perform an HTTP GET request. Returns a Dict with `status`, `body`, `headers`."),
        "http_post" => Some("**http_post(url, body, headers?)** — Perform an HTTP POST request."),
        "http_put" => Some("**http_put(url, body, headers?)** — Perform an HTTP PUT request."),
        "http_delete" => Some("**http_delete(url, headers?)** — Perform an HTTP DELETE request."),
        "env_get" => Some("**env_get(name)** — Get an environment variable value."),
        "env_set" => Some("**env_set(name, value)** — Set an environment variable."),
        "input" => Some("**input(prompt?)** — Read a line from stdin."),
        "hash" => Some("**hash(value)** — Return a hash code for a value."),
        _ => None,
    }
}

/// Return a Markdown description for a UniLang keyword or type.
fn keyword_description(word: &str) -> Option<&'static str> {
    match word {
        // ── Shared Keywords ──────────────────────────────────
        "class" => Some("**class** (shared)\n\nDeclare a class. Works in both Python-style and Java-style blocks.\n\n```\nclass MyClass:\n    pass\n\nclass MyClass {\n}\n```"),
        "if" => Some("**if** (shared)\n\nConditional branch. Supports both colon-block and brace-block syntax.\n\n```\nif condition:\n    ...\n\nif (condition) {\n    ...\n}\n```"),
        "else" => Some("**else** (shared)\n\nAlternative branch for `if` statements."),
        "for" => Some("**for** (shared)\n\nIteration construct. Supports Python-style `for x in iterable:` and Java-style `for (init; cond; step) {}`.\n\n```\nfor item in collection:\n    ...\n\nfor (int i = 0; i < n; i++) {\n    ...\n}\n```"),
        "while" => Some("**while** (shared)\n\nLoop while a condition is true."),
        "return" => Some("**return** (shared)\n\nReturn a value from a function or method."),
        "import" => Some("**import** (shared)\n\nImport a module or package.\n\n```\nimport math\nimport java.util.List\nfrom os import path\n```"),
        "try" => Some("**try** (shared)\n\nStart an exception/error handling block."),
        "finally" => Some("**finally** (shared)\n\nBlock that always executes after `try`/`except`/`catch`."),
        "continue" => Some("**continue** (shared)\n\nSkip to the next iteration of the enclosing loop."),
        "break" => Some("**break** (shared)\n\nExit the enclosing loop."),
        "assert" => Some("**assert(condition, message?)** — Assert a condition is true; error if false. Also a shared keyword."),

        // ── Python-origin Keywords ───────────────────────────
        "def" => Some("**def** (Python-origin)\n\nDefine a function using Python-style syntax.\n\n```\ndef greet(name: str) -> str:\n    return f\"Hello, {name}\"\n```"),
        "elif" => Some("**elif** (Python-origin)\n\nElse-if branch in a conditional chain.\n\n```\nif x > 0:\n    ...\nelif x == 0:\n    ...\nelse:\n    ...\n```"),
        "except" => Some("**except** (Python-origin)\n\nCatch an exception in a `try` block (Python-style).\n\n```\ntry:\n    ...\nexcept ValueError as e:\n    ...\n```"),
        "raise" => Some("**raise** (Python-origin)\n\nRaise an exception.\n\n```\nraise ValueError(\"invalid input\")\n```"),
        "pass" => Some("**pass** (Python-origin)\n\nNo-op placeholder statement."),
        "yield" => Some("**yield** (Python-origin)\n\nYield a value from a generator function."),
        "async" => Some("**async** (Python-origin)\n\nDeclare an asynchronous function.\n\n```\nasync def fetch(url):\n    ...\n```"),
        "await" => Some("**await** (Python-origin)\n\nAwait the result of an asynchronous expression."),
        "with" => Some("**with** (Python-origin)\n\nContext manager statement.\n\n```\nwith open(\"file.txt\") as f:\n    ...\n```"),
        "as" => Some("**as** (Python-origin)\n\nAlias in imports or exception handlers."),
        "from" => Some("**from** (Python-origin)\n\nImport specific names from a module.\n\n```\nfrom math import sqrt\n```"),
        "in" => Some("**in** (Python-origin)\n\nMembership test or loop iteration keyword."),
        "is" => Some("**is** (Python-origin)\n\nIdentity comparison operator."),
        "not" => Some("**not** (Python-origin)\n\nLogical negation operator."),
        "and" => Some("**and** (Python-origin)\n\nLogical AND operator."),
        "or" => Some("**or** (Python-origin)\n\nLogical OR operator."),
        "lambda" => Some("**lambda** (Python-origin)\n\nAnonymous inline function.\n\n```\nsquare = lambda x: x ** 2\n```"),
        "nonlocal" => Some("**nonlocal** (Python-origin)\n\nDeclare a variable as belonging to an enclosing scope."),
        "global" => Some("**global** (Python-origin)\n\nDeclare a variable as global."),
        "del" => Some("**del** (Python-origin)\n\nDelete a variable or collection element."),
        "match" => Some("**match** (Python-origin)\n\nStructural pattern matching.\n\n```\nmatch command:\n    case \"quit\":\n        ...\n    case \"help\":\n        ...\n```"),
        "case" => Some("**case** (shared)\n\nA branch in a `match`/`switch` statement."),

        // ── Java-origin Keywords ─────────────────────────────
        "public" => Some("**public** (Java-origin)\n\nAccess modifier: visible to all classes."),
        "private" => Some("**private** (Java-origin)\n\nAccess modifier: visible only within the declaring class."),
        "protected" => Some("**protected** (Java-origin)\n\nAccess modifier: visible within the package and subclasses."),
        "static" => Some("**static** (Java-origin)\n\nDeclare a class-level (static) member."),
        "final" => Some("**final** (Java-origin)\n\nDeclare a constant or prevent overriding/inheritance."),
        "abstract" => Some("**abstract** (Java-origin)\n\nDeclare an abstract class or method (no implementation)."),
        "interface" => Some("**interface** (Java-origin)\n\nDeclare an interface type.\n\n```\ninterface Drawable {\n    def draw(self):\n        pass\n}\n```"),
        "enum" => Some("**enum** (Java-origin)\n\nDeclare an enumeration type."),
        "extends" => Some("**extends** (Java-origin)\n\nInherit from a parent class.\n\n```\nclass Dog extends Animal {\n}\n```"),
        "implements" => Some("**implements** (Java-origin)\n\nImplement one or more interfaces."),
        "new" => Some("**new** (Java-origin)\n\nCreate a new object instance.\n\n```\nval list = new ArrayList()\n```"),
        "this" => Some("**this** (Java-origin)\n\nReference to the current object instance."),
        "super" => Some("**super** (Java-origin)\n\nReference to the parent class."),
        "void" => Some("**void** (Java-origin)\n\nReturn type indicating no value is returned."),
        "throws" => Some("**throws** (Java-origin)\n\nDeclare exceptions a method may throw."),
        "throw" => Some("**throw** (Java-origin)\n\nThrow an exception (Java-style `raise`)."),
        "catch" => Some("**catch** (Java-origin)\n\nCatch an exception in a `try` block (Java-style).\n\n```\ntry {\n    ...\n} catch (Exception e) {\n    ...\n}\n```"),
        "synchronized" => Some("**synchronized** (Java-origin)\n\nDeclare a synchronized block or method for thread safety."),
        "volatile" => Some("**volatile** (Java-origin)\n\nMark a field as volatile (visible across threads)."),
        "transient" => Some("**transient** (Java-origin)\n\nMark a field to be excluded from serialization."),
        "native" => Some("**native** (Java-origin)\n\nDeclare a method implemented in native code."),
        "instanceof" => Some("**instanceof** (Java-origin)\n\nTest whether an object is an instance of a type.\n\n```\nif (obj instanceof String) { ... }\n```"),
        "switch" => Some("**switch** (Java-origin)\n\nMulti-way branch statement.\n\n```\nswitch (value) {\n    case 1: ...\n    default: ...\n}\n```"),
        "default" => Some("**default** (Java-origin)\n\nDefault branch in a `switch` statement."),
        "do" => Some("**do** (Java-origin)\n\nStart a `do-while` loop.\n\n```\ndo {\n    ...\n} while (condition);\n```"),

        // ── UniLang-specific Keywords ────────────────────────
        "bridge" => Some("**bridge** (UniLang)\n\nDeclare a bridge between Python and Java interop boundaries."),
        "vm" => Some("**vm** (UniLang)\n\nSpecify a target virtual machine context."),
        "interop" => Some("**interop** (UniLang)\n\nDeclare cross-language interoperability."),
        "val" => Some("**val** (UniLang)\n\nDeclare an immutable variable.\n\n```\nval name = \"UniLang\"\n```"),
        "var" => Some("**var** (UniLang)\n\nDeclare a mutable variable.\n\n```\nvar count = 0\ncount = count + 1\n```"),
        "const" => Some("**const** (UniLang)\n\nDeclare a compile-time constant."),

        // ── Literals / Built-in values ───────────────────────
        "True" | "true" => Some("**True** / **true**\n\nBoolean literal representing truth."),
        "False" | "false" => Some("**False** / **false**\n\nBoolean literal representing falsehood."),
        "None" => Some("**None** (Python-origin)\n\nRepresents the absence of a value (Python-style null)."),
        "null" => Some("**null** (Java-origin)\n\nRepresents the absence of a value (Java-style null)."),

        // ── Built-in types ───────────────────────────────────
        "int" => Some("**int(value)** — Convert a value to an integer. Also the integer numeric type."),
        "float" => Some("**float(value)** — Convert a value to a floating-point number. Also the float numeric type."),
        "str" => Some("**str(value)** — Convert a value to its string representation. Also the string type (text sequence)."),
        "bool" => Some("**bool(value)** — Convert a value to a boolean. Also the boolean type (`true`/`false`)."),
        "list" => Some("**list(iterable?)** — Create a new list, optionally from an iterable. Also the ordered mutable collection type."),
        "dict" => Some("**dict()** — Create a new empty dictionary. Also the key-value mapping type."),
        "set" => Some("**set**\n\nUnordered collection of unique elements."),
        "tuple" => Some("**tuple**\n\nOrdered, immutable collection type."),
        "String" => Some("**String**\n\nJava-style string object type."),
        "List" => Some("**List**\n\nJava-style generic list interface."),
        "Map" => Some("**Map**\n\nJava-style generic map interface."),
        "Set" => Some("**Set**\n\nJava-style generic set interface."),
        "Object" => Some("**Object**\n\nBase type for all objects."),
        "Array" => Some("**Array**\n\nFixed-size array type."),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_word_at_position() {
        let source = "def hello_world():";
        let word = extract_word_at_position(source, Position::new(0, 5));
        assert_eq!(word, Some("hello_world".to_string()));
    }

    #[test]
    fn test_hover_keyword() {
        let source = "def greet(name):";
        let hover = get_hover_info(source, Position::new(0, 1));
        assert!(hover.is_some());
    }

    #[test]
    fn test_hover_unknown_word() {
        let source = "xyzzy = 42";
        let hover = get_hover_info(source, Position::new(0, 2));
        assert!(hover.is_none());
    }
}
