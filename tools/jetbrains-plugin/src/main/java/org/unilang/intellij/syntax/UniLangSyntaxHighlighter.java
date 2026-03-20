package org.unilang.intellij.syntax;

import com.intellij.lexer.Lexer;
import com.intellij.openapi.editor.DefaultLanguageHighlighterColors;
import com.intellij.openapi.editor.HighlighterColors;
import com.intellij.openapi.editor.colors.TextAttributesKey;
import com.intellij.openapi.editor.markup.EffectType;
import com.intellij.openapi.editor.markup.TextAttributes;
import com.intellij.openapi.fileTypes.SyntaxHighlighterBase;
import com.intellij.psi.tree.IElementType;
import org.jetbrains.annotations.NotNull;

import java.awt.Color;
import java.awt.Font;

import static com.intellij.openapi.editor.colors.TextAttributesKey.createTextAttributesKey;

public class UniLangSyntaxHighlighter extends SyntaxHighlighterBase {

    // Keyword styles - bold blue/purple
    public static final TextAttributesKey KEYWORD_KEY =
            createTextAttributesKey("UNILANG_KEYWORD", DefaultLanguageHighlighterColors.KEYWORD);
    public static final TextAttributesKey PYTHON_KEYWORD_KEY =
            createTextAttributesKey("UNILANG_PYTHON_KEYWORD", DefaultLanguageHighlighterColors.KEYWORD);
    public static final TextAttributesKey JAVA_KEYWORD_KEY =
            createTextAttributesKey("UNILANG_JAVA_KEYWORD", DefaultLanguageHighlighterColors.KEYWORD);

    // String styles - green
    public static final TextAttributesKey STRING_KEY =
            createTextAttributesKey("UNILANG_STRING", DefaultLanguageHighlighterColors.STRING);
    public static final TextAttributesKey TRIPLE_STRING_KEY =
            createTextAttributesKey("UNILANG_TRIPLE_STRING", DefaultLanguageHighlighterColors.STRING);
    public static final TextAttributesKey FSTRING_KEY =
            createTextAttributesKey("UNILANG_FSTRING", DefaultLanguageHighlighterColors.STRING);

    // Comment styles - gray italic
    public static final TextAttributesKey COMMENT_KEY =
            createTextAttributesKey("UNILANG_COMMENT", DefaultLanguageHighlighterColors.LINE_COMMENT);
    public static final TextAttributesKey BLOCK_COMMENT_KEY =
            createTextAttributesKey("UNILANG_BLOCK_COMMENT", DefaultLanguageHighlighterColors.BLOCK_COMMENT);

    // Number style - cyan
    public static final TextAttributesKey NUMBER_KEY =
            createTextAttributesKey("UNILANG_NUMBER", DefaultLanguageHighlighterColors.NUMBER);

    // Operator
    public static final TextAttributesKey OPERATOR_KEY =
            createTextAttributesKey("UNILANG_OPERATOR", DefaultLanguageHighlighterColors.OPERATION_SIGN);

    // Identifier
    public static final TextAttributesKey IDENTIFIER_KEY =
            createTextAttributesKey("UNILANG_IDENTIFIER", DefaultLanguageHighlighterColors.IDENTIFIER);

    // Brackets
    public static final TextAttributesKey BRACE_KEY =
            createTextAttributesKey("UNILANG_BRACE", DefaultLanguageHighlighterColors.BRACES);
    public static final TextAttributesKey BRACKET_KEY =
            createTextAttributesKey("UNILANG_BRACKET", DefaultLanguageHighlighterColors.BRACKETS);
    public static final TextAttributesKey PAREN_KEY =
            createTextAttributesKey("UNILANG_PAREN", DefaultLanguageHighlighterColors.PARENTHESES);

    // Type names - teal
    public static final TextAttributesKey TYPE_NAME_KEY =
            createTextAttributesKey("UNILANG_TYPE_NAME", DefaultLanguageHighlighterColors.CLASS_NAME);

    // Decorator - yellow
    public static final TextAttributesKey DECORATOR_KEY =
            createTextAttributesKey("UNILANG_DECORATOR", DefaultLanguageHighlighterColors.METADATA);

    // Bad character - red wavy underline
    public static final TextAttributesKey BAD_CHARACTER_KEY =
            createTextAttributesKey("UNILANG_BAD_CHARACTER", HighlighterColors.BAD_CHARACTER);

    private static final TextAttributesKey[] KEYWORD_KEYS = {KEYWORD_KEY};
    private static final TextAttributesKey[] PYTHON_KEYWORD_KEYS = {PYTHON_KEYWORD_KEY};
    private static final TextAttributesKey[] JAVA_KEYWORD_KEYS = {JAVA_KEYWORD_KEY};
    private static final TextAttributesKey[] STRING_KEYS = {STRING_KEY};
    private static final TextAttributesKey[] TRIPLE_STRING_KEYS = {TRIPLE_STRING_KEY};
    private static final TextAttributesKey[] FSTRING_KEYS = {FSTRING_KEY};
    private static final TextAttributesKey[] COMMENT_KEYS = {COMMENT_KEY};
    private static final TextAttributesKey[] BLOCK_COMMENT_KEYS = {BLOCK_COMMENT_KEY};
    private static final TextAttributesKey[] NUMBER_KEYS = {NUMBER_KEY};
    private static final TextAttributesKey[] OPERATOR_KEYS = {OPERATOR_KEY};
    private static final TextAttributesKey[] IDENTIFIER_KEYS = {IDENTIFIER_KEY};
    private static final TextAttributesKey[] BRACE_KEYS = {BRACE_KEY};
    private static final TextAttributesKey[] BRACKET_KEYS = {BRACKET_KEY};
    private static final TextAttributesKey[] PAREN_KEYS = {PAREN_KEY};
    private static final TextAttributesKey[] TYPE_NAME_KEYS = {TYPE_NAME_KEY};
    private static final TextAttributesKey[] DECORATOR_KEYS = {DECORATOR_KEY};
    private static final TextAttributesKey[] BAD_CHARACTER_KEYS = {BAD_CHARACTER_KEY};
    private static final TextAttributesKey[] EMPTY_KEYS = {};

    @NotNull
    @Override
    public Lexer getHighlightingLexer() {
        return new UniLangLexer();
    }

    @NotNull
    @Override
    public TextAttributesKey @NotNull [] getTokenHighlights(IElementType tokenType) {
        if (tokenType.equals(UniLangTokenTypes.KEYWORD)) return KEYWORD_KEYS;
        if (tokenType.equals(UniLangTokenTypes.PYTHON_KEYWORD)) return PYTHON_KEYWORD_KEYS;
        if (tokenType.equals(UniLangTokenTypes.JAVA_KEYWORD)) return JAVA_KEYWORD_KEYS;
        if (tokenType.equals(UniLangTokenTypes.STRING)) return STRING_KEYS;
        if (tokenType.equals(UniLangTokenTypes.TRIPLE_STRING)) return TRIPLE_STRING_KEYS;
        if (tokenType.equals(UniLangTokenTypes.FSTRING)) return FSTRING_KEYS;
        if (tokenType.equals(UniLangTokenTypes.COMMENT)) return COMMENT_KEYS;
        if (tokenType.equals(UniLangTokenTypes.BLOCK_COMMENT)) return BLOCK_COMMENT_KEYS;
        if (tokenType.equals(UniLangTokenTypes.NUMBER)) return NUMBER_KEYS;
        if (tokenType.equals(UniLangTokenTypes.OPERATOR)) return OPERATOR_KEYS;
        if (tokenType.equals(UniLangTokenTypes.IDENTIFIER)) return IDENTIFIER_KEYS;
        if (tokenType.equals(UniLangTokenTypes.BRACE)) return BRACE_KEYS;
        if (tokenType.equals(UniLangTokenTypes.BRACKET)) return BRACKET_KEYS;
        if (tokenType.equals(UniLangTokenTypes.PAREN)) return PAREN_KEYS;
        if (tokenType.equals(UniLangTokenTypes.TYPE_NAME)) return TYPE_NAME_KEYS;
        if (tokenType.equals(UniLangTokenTypes.DECORATOR)) return DECORATOR_KEYS;
        if (tokenType.equals(UniLangTokenTypes.BAD_CHARACTER)) return BAD_CHARACTER_KEYS;
        return EMPTY_KEYS;
    }
}
