package org.unilang.intellij.syntax;

import com.intellij.lexer.LexerBase;
import com.intellij.psi.tree.IElementType;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.util.Set;

public class UniLangLexer extends LexerBase {

    private CharSequence buffer;
    private int bufferEnd;
    private int tokenStart;
    private int tokenEnd;
    private IElementType tokenType;

    private static final Set<String> PYTHON_KEYWORDS = Set.of(
            "def", "class", "if", "elif", "else", "for", "while", "return",
            "import", "from", "as", "with", "try", "except", "finally",
            "raise", "pass", "break", "continue", "and", "or", "not",
            "in", "is", "lambda", "yield", "global", "nonlocal", "assert",
            "del", "True", "False", "None", "async", "await", "self"
    );

    private static final Set<String> JAVA_KEYWORDS = Set.of(
            "public", "private", "protected", "static", "final", "abstract",
            "class", "interface", "extends", "implements", "new", "this",
            "super", "void", "return", "if", "else", "for", "while", "do",
            "switch", "case", "default", "break", "continue", "try", "catch",
            "finally", "throw", "throws", "import", "package", "instanceof",
            "synchronized", "volatile", "transient", "native", "enum",
            "var", "record", "sealed", "permits", "yield"
    );

    private static final Set<String> SHARED_KEYWORDS = Set.of(
            "class", "if", "else", "for", "while", "return", "break",
            "continue", "try", "finally", "import", "yield"
    );

    private static final Set<String> TYPE_NAMES = Set.of(
            "int", "float", "double", "long", "short", "byte", "char",
            "boolean", "String", "List", "Dict", "Set", "Tuple", "Optional",
            "Map", "Array", "Vector", "HashMap", "ArrayList", "Integer",
            "Float", "Double", "Long", "Boolean", "Object", "str", "bool",
            "list", "dict", "set", "tuple", "type"
    );

    @Override
    public void start(@NotNull CharSequence buffer, int startOffset, int endOffset, int initialState) {
        this.buffer = buffer;
        this.bufferEnd = endOffset;
        this.tokenStart = startOffset;
        this.tokenEnd = startOffset;
        this.tokenType = null;
        advance();
    }

    @Override
    public int getState() {
        return 0;
    }

    @Nullable
    @Override
    public IElementType getTokenType() {
        return tokenType;
    }

    @Override
    public int getTokenStart() {
        return tokenStart;
    }

    @Override
    public int getTokenEnd() {
        return tokenEnd;
    }

    @Override
    public void advance() {
        tokenStart = tokenEnd;
        if (tokenStart >= bufferEnd) {
            tokenType = null;
            return;
        }

        char c = buffer.charAt(tokenStart);

        // Whitespace
        if (c == ' ' || c == '\t') {
            tokenEnd = tokenStart + 1;
            while (tokenEnd < bufferEnd && (buffer.charAt(tokenEnd) == ' ' || buffer.charAt(tokenEnd) == '\t')) {
                tokenEnd++;
            }
            tokenType = UniLangTokenTypes.WHITESPACE;
            return;
        }

        // Newline
        if (c == '\n' || c == '\r') {
            tokenEnd = tokenStart + 1;
            if (c == '\r' && tokenEnd < bufferEnd && buffer.charAt(tokenEnd) == '\n') {
                tokenEnd++;
            }
            tokenType = UniLangTokenTypes.NEWLINE;
            return;
        }

        // Line comments: // or #
        if (c == '#' || (c == '/' && tokenStart + 1 < bufferEnd && buffer.charAt(tokenStart + 1) == '/')) {
            tokenEnd = tokenStart;
            while (tokenEnd < bufferEnd && buffer.charAt(tokenEnd) != '\n') {
                tokenEnd++;
            }
            tokenType = UniLangTokenTypes.COMMENT;
            return;
        }

        // Block comments: /* ... */
        if (c == '/' && tokenStart + 1 < bufferEnd && buffer.charAt(tokenStart + 1) == '*') {
            tokenEnd = tokenStart + 2;
            while (tokenEnd < bufferEnd - 1) {
                if (buffer.charAt(tokenEnd) == '*' && buffer.charAt(tokenEnd + 1) == '/') {
                    tokenEnd += 2;
                    tokenType = UniLangTokenTypes.BLOCK_COMMENT;
                    return;
                }
                tokenEnd++;
            }
            tokenEnd = bufferEnd;
            tokenType = UniLangTokenTypes.BLOCK_COMMENT;
            return;
        }

        // Decorator: @identifier
        if (c == '@' && tokenStart + 1 < bufferEnd && Character.isLetter(buffer.charAt(tokenStart + 1))) {
            tokenEnd = tokenStart + 1;
            while (tokenEnd < bufferEnd && (Character.isLetterOrDigit(buffer.charAt(tokenEnd)) || buffer.charAt(tokenEnd) == '.')) {
                tokenEnd++;
            }
            tokenType = UniLangTokenTypes.DECORATOR;
            return;
        }

        // F-strings: f"..." or f'...'
        if ((c == 'f' || c == 'F') && tokenStart + 1 < bufferEnd) {
            char next = buffer.charAt(tokenStart + 1);
            if (next == '"' || next == '\'') {
                tokenEnd = tokenStart + 2;
                tokenEnd = scanString(next, tokenEnd);
                tokenType = UniLangTokenTypes.FSTRING;
                return;
            }
        }

        // Raw strings: r"..." or r'...'
        if ((c == 'r' || c == 'R') && tokenStart + 1 < bufferEnd) {
            char next = buffer.charAt(tokenStart + 1);
            if (next == '"' || next == '\'') {
                tokenEnd = tokenStart + 2;
                tokenEnd = scanString(next, tokenEnd);
                tokenType = UniLangTokenTypes.STRING;
                return;
            }
        }

        // Triple-quoted strings: \"\"\" or '''
        if ((c == '"' || c == '\'') && tokenStart + 2 < bufferEnd
                && buffer.charAt(tokenStart + 1) == c && buffer.charAt(tokenStart + 2) == c) {
            tokenEnd = tokenStart + 3;
            while (tokenEnd + 2 < bufferEnd) {
                if (buffer.charAt(tokenEnd) == c && buffer.charAt(tokenEnd + 1) == c && buffer.charAt(tokenEnd + 2) == c) {
                    tokenEnd += 3;
                    tokenType = UniLangTokenTypes.TRIPLE_STRING;
                    return;
                }
                if (buffer.charAt(tokenEnd) == '\\') {
                    tokenEnd++;
                }
                tokenEnd++;
            }
            tokenEnd = bufferEnd;
            tokenType = UniLangTokenTypes.TRIPLE_STRING;
            return;
        }

        // Regular strings: "..." or '...'
        if (c == '"' || c == '\'') {
            tokenEnd = tokenStart + 1;
            tokenEnd = scanString(c, tokenEnd);
            tokenType = UniLangTokenTypes.STRING;
            return;
        }

        // Numbers
        if (Character.isDigit(c) || (c == '.' && tokenStart + 1 < bufferEnd && Character.isDigit(buffer.charAt(tokenStart + 1)))) {
            tokenEnd = tokenStart;
            // Hex
            if (c == '0' && tokenEnd + 1 < bufferEnd && (buffer.charAt(tokenEnd + 1) == 'x' || buffer.charAt(tokenEnd + 1) == 'X')) {
                tokenEnd += 2;
                while (tokenEnd < bufferEnd && isHexDigit(buffer.charAt(tokenEnd))) {
                    tokenEnd++;
                }
            } else {
                // Decimal / float
                while (tokenEnd < bufferEnd && Character.isDigit(buffer.charAt(tokenEnd))) {
                    tokenEnd++;
                }
                if (tokenEnd < bufferEnd && buffer.charAt(tokenEnd) == '.') {
                    tokenEnd++;
                    while (tokenEnd < bufferEnd && Character.isDigit(buffer.charAt(tokenEnd))) {
                        tokenEnd++;
                    }
                }
                // Exponent
                if (tokenEnd < bufferEnd && (buffer.charAt(tokenEnd) == 'e' || buffer.charAt(tokenEnd) == 'E')) {
                    tokenEnd++;
                    if (tokenEnd < bufferEnd && (buffer.charAt(tokenEnd) == '+' || buffer.charAt(tokenEnd) == '-')) {
                        tokenEnd++;
                    }
                    while (tokenEnd < bufferEnd && Character.isDigit(buffer.charAt(tokenEnd))) {
                        tokenEnd++;
                    }
                }
                // Type suffix
                if (tokenEnd < bufferEnd && (buffer.charAt(tokenEnd) == 'L' || buffer.charAt(tokenEnd) == 'l'
                        || buffer.charAt(tokenEnd) == 'f' || buffer.charAt(tokenEnd) == 'd'
                        || buffer.charAt(tokenEnd) == 'D' || buffer.charAt(tokenEnd) == 'F')) {
                    tokenEnd++;
                }
            }
            tokenType = UniLangTokenTypes.NUMBER;
            return;
        }

        // Identifiers and keywords
        if (Character.isLetter(c) || c == '_') {
            tokenEnd = tokenStart;
            while (tokenEnd < bufferEnd && (Character.isLetterOrDigit(buffer.charAt(tokenEnd)) || buffer.charAt(tokenEnd) == '_')) {
                tokenEnd++;
            }
            String word = buffer.subSequence(tokenStart, tokenEnd).toString();

            if (TYPE_NAMES.contains(word)) {
                tokenType = UniLangTokenTypes.TYPE_NAME;
            } else if (SHARED_KEYWORDS.contains(word)) {
                tokenType = UniLangTokenTypes.KEYWORD;
            } else if (PYTHON_KEYWORDS.contains(word)) {
                tokenType = UniLangTokenTypes.PYTHON_KEYWORD;
            } else if (JAVA_KEYWORDS.contains(word)) {
                tokenType = UniLangTokenTypes.JAVA_KEYWORD;
            } else {
                tokenType = UniLangTokenTypes.IDENTIFIER;
            }
            return;
        }

        // Brackets
        if (c == '{' || c == '}') {
            tokenEnd = tokenStart + 1;
            tokenType = UniLangTokenTypes.BRACE;
            return;
        }
        if (c == '[' || c == ']') {
            tokenEnd = tokenStart + 1;
            tokenType = UniLangTokenTypes.BRACKET;
            return;
        }
        if (c == '(' || c == ')') {
            tokenEnd = tokenStart + 1;
            tokenType = UniLangTokenTypes.PAREN;
            return;
        }

        // Multi-char operators
        if (tokenStart + 1 < bufferEnd) {
            String twoChar = buffer.subSequence(tokenStart, tokenStart + 2).toString();
            if (Set.of("==", "!=", "<=", ">=", "&&", "||", "->", "=>", "+=", "-=",
                    "*=", "/=", "%=", "**", "//", "<<", ">>", "::", "..").contains(twoChar)) {
                tokenEnd = tokenStart + 2;
                tokenType = UniLangTokenTypes.OPERATOR;
                return;
            }
        }

        // Single-char operators
        if ("+-*/%=<>!&|^~?:;,.".indexOf(c) >= 0) {
            tokenEnd = tokenStart + 1;
            tokenType = UniLangTokenTypes.OPERATOR;
            return;
        }

        // Bad character
        tokenEnd = tokenStart + 1;
        tokenType = UniLangTokenTypes.BAD_CHARACTER;
    }

    @NotNull
    @Override
    public CharSequence getBufferSequence() {
        return buffer;
    }

    @Override
    public int getBufferEnd() {
        return bufferEnd;
    }

    private int scanString(char quote, int pos) {
        while (pos < bufferEnd) {
            char ch = buffer.charAt(pos);
            if (ch == '\\') {
                pos += 2;
                continue;
            }
            if (ch == quote) {
                return pos + 1;
            }
            if (ch == '\n') {
                return pos;
            }
            pos++;
        }
        return bufferEnd;
    }

    private static boolean isHexDigit(char c) {
        return (c >= '0' && c <= '9') || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F');
    }
}
