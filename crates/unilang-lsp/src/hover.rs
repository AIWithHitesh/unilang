// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

//! Hover information for UniLang tokens.
//!
//! When the user hovers over a keyword or identifier, this module
//! provides contextual documentation drawn from both the Python
//! and Java sides of UniLang.

use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};

/// Return hover info for the word at the given position, if any.
pub fn get_hover_info(source: &str, position: Position) -> Option<Hover> {
    let word = extract_word_at_position(source, position)?;
    let info = keyword_description(&word)?;

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: info.to_string(),
        }),
        range: None,
    })
}

/// Extract the word under the cursor at the given LSP position.
fn extract_word_at_position(source: &str, position: Position) -> Option<String> {
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
        "assert" => Some("**assert** (shared)\n\nAssert that a condition is true; raises an error otherwise."),

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
        "int" => Some("**int**\n\nInteger numeric type."),
        "float" => Some("**float**\n\nFloating-point numeric type."),
        "str" => Some("**str**\n\nString type (text sequence)."),
        "bool" => Some("**bool**\n\nBoolean type (`true`/`false`)."),
        "list" => Some("**list**\n\nOrdered, mutable collection type."),
        "dict" => Some("**dict**\n\nKey-value mapping type (dictionary/hashmap)."),
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
