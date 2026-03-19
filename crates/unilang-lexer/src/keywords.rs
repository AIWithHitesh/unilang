// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.

use crate::token::TokenKind;

/// Look up a keyword from an identifier string.
/// Returns `None` if the string is not a keyword.
pub fn lookup_keyword(ident: &str) -> Option<TokenKind> {
    match ident {
        // Shared keywords
        "class" => Some(TokenKind::KwClass),
        "if" => Some(TokenKind::KwIf),
        "else" => Some(TokenKind::KwElse),
        "for" => Some(TokenKind::KwFor),
        "while" => Some(TokenKind::KwWhile),
        "return" => Some(TokenKind::KwReturn),
        "import" => Some(TokenKind::KwImport),
        "try" => Some(TokenKind::KwTry),
        "finally" => Some(TokenKind::KwFinally),
        "continue" => Some(TokenKind::KwContinue),
        "break" => Some(TokenKind::KwBreak),
        "assert" => Some(TokenKind::KwAssert),

        // Python-origin keywords
        "def" => Some(TokenKind::KwDef),
        "elif" => Some(TokenKind::KwElif),
        "except" => Some(TokenKind::KwExcept),
        "raise" => Some(TokenKind::KwRaise),
        "pass" => Some(TokenKind::KwPass),
        "yield" => Some(TokenKind::KwYield),
        "async" => Some(TokenKind::KwAsync),
        "await" => Some(TokenKind::KwAwait),
        "with" => Some(TokenKind::KwWith),
        "as" => Some(TokenKind::KwAs),
        "from" => Some(TokenKind::KwFrom),
        "in" => Some(TokenKind::KwIn),
        "is" => Some(TokenKind::KwIs),
        "not" => Some(TokenKind::KwNot),
        "and" => Some(TokenKind::KwAnd),
        "or" => Some(TokenKind::KwOr),
        "lambda" => Some(TokenKind::KwLambda),
        "nonlocal" => Some(TokenKind::KwNonlocal),
        "global" => Some(TokenKind::KwGlobal),
        "del" => Some(TokenKind::KwDel),
        "match" => Some(TokenKind::KwMatch),
        "case" => Some(TokenKind::KwCase),

        // Java-origin keywords
        "public" => Some(TokenKind::KwPublic),
        "private" => Some(TokenKind::KwPrivate),
        "protected" => Some(TokenKind::KwProtected),
        "static" => Some(TokenKind::KwStatic),
        "final" => Some(TokenKind::KwFinal),
        "abstract" => Some(TokenKind::KwAbstract),
        "interface" => Some(TokenKind::KwInterface),
        "enum" => Some(TokenKind::KwEnum),
        "extends" => Some(TokenKind::KwExtends),
        "implements" => Some(TokenKind::KwImplements),
        "new" => Some(TokenKind::KwNew),
        "this" => Some(TokenKind::KwThis),
        "super" => Some(TokenKind::KwSuper),
        "void" => Some(TokenKind::KwVoid),
        "throws" => Some(TokenKind::KwThrows),
        "throw" => Some(TokenKind::KwThrow),
        "catch" => Some(TokenKind::KwCatch),
        "synchronized" => Some(TokenKind::KwSynchronized),
        "volatile" => Some(TokenKind::KwVolatile),
        "transient" => Some(TokenKind::KwTransient),
        "native" => Some(TokenKind::KwNative),
        "instanceof" => Some(TokenKind::KwInstanceof),
        "switch" => Some(TokenKind::KwSwitch),
        "default" => Some(TokenKind::KwDefault),
        "do" => Some(TokenKind::KwDo),

        // Boolean and null literals (treated as keywords)
        "True" => Some(TokenKind::BoolTrue),
        "true" => Some(TokenKind::BoolTrue),
        "False" => Some(TokenKind::BoolFalse),
        "false" => Some(TokenKind::BoolFalse),
        "None" => Some(TokenKind::NullNone),
        "null" => Some(TokenKind::NullNull),

        // UniLang-specific keywords
        "bridge" => Some(TokenKind::KwBridge),
        "vm" => Some(TokenKind::KwVm),
        "interop" => Some(TokenKind::KwInterop),
        "val" => Some(TokenKind::KwVal),
        "var" => Some(TokenKind::KwVar),
        "const" => Some(TokenKind::KwConst),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_keywords() {
        assert_eq!(lookup_keyword("class"), Some(TokenKind::KwClass));
        assert_eq!(lookup_keyword("if"), Some(TokenKind::KwIf));
        assert_eq!(lookup_keyword("return"), Some(TokenKind::KwReturn));
    }

    #[test]
    fn test_python_keywords() {
        assert_eq!(lookup_keyword("def"), Some(TokenKind::KwDef));
        assert_eq!(lookup_keyword("elif"), Some(TokenKind::KwElif));
        assert_eq!(lookup_keyword("lambda"), Some(TokenKind::KwLambda));
    }

    #[test]
    fn test_java_keywords() {
        assert_eq!(lookup_keyword("public"), Some(TokenKind::KwPublic));
        assert_eq!(
            lookup_keyword("synchronized"),
            Some(TokenKind::KwSynchronized)
        );
        assert_eq!(lookup_keyword("instanceof"), Some(TokenKind::KwInstanceof));
    }

    #[test]
    fn test_bool_null() {
        assert_eq!(lookup_keyword("True"), Some(TokenKind::BoolTrue));
        assert_eq!(lookup_keyword("true"), Some(TokenKind::BoolTrue));
        assert_eq!(lookup_keyword("False"), Some(TokenKind::BoolFalse));
        assert_eq!(lookup_keyword("None"), Some(TokenKind::NullNone));
        assert_eq!(lookup_keyword("null"), Some(TokenKind::NullNull));
    }

    #[test]
    fn test_not_keyword() {
        assert_eq!(lookup_keyword("foo"), None);
        assert_eq!(lookup_keyword("myVar"), None);
        assert_eq!(lookup_keyword(""), None);
    }
}
