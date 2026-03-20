package org.unilang.intellij.syntax;

import com.intellij.psi.tree.IElementType;
import com.intellij.psi.tree.TokenSet;
import org.unilang.intellij.UniLangLanguage;

public final class UniLangTokenTypes {

    // Keywords
    public static final IElementType KEYWORD = new IElementType("KEYWORD", UniLangLanguage.INSTANCE);
    public static final IElementType PYTHON_KEYWORD = new IElementType("PYTHON_KEYWORD", UniLangLanguage.INSTANCE);
    public static final IElementType JAVA_KEYWORD = new IElementType("JAVA_KEYWORD", UniLangLanguage.INSTANCE);

    // Literals
    public static final IElementType STRING = new IElementType("STRING", UniLangLanguage.INSTANCE);
    public static final IElementType TRIPLE_STRING = new IElementType("TRIPLE_STRING", UniLangLanguage.INSTANCE);
    public static final IElementType FSTRING = new IElementType("FSTRING", UniLangLanguage.INSTANCE);
    public static final IElementType NUMBER = new IElementType("NUMBER", UniLangLanguage.INSTANCE);

    // Comments
    public static final IElementType COMMENT = new IElementType("COMMENT", UniLangLanguage.INSTANCE);
    public static final IElementType BLOCK_COMMENT = new IElementType("BLOCK_COMMENT", UniLangLanguage.INSTANCE);

    // Operators and delimiters
    public static final IElementType OPERATOR = new IElementType("OPERATOR", UniLangLanguage.INSTANCE);
    public static final IElementType IDENTIFIER = new IElementType("IDENTIFIER", UniLangLanguage.INSTANCE);
    public static final IElementType BRACE = new IElementType("BRACE", UniLangLanguage.INSTANCE);
    public static final IElementType BRACKET = new IElementType("BRACKET", UniLangLanguage.INSTANCE);
    public static final IElementType PAREN = new IElementType("PAREN", UniLangLanguage.INSTANCE);

    // Special
    public static final IElementType DECORATOR = new IElementType("DECORATOR", UniLangLanguage.INSTANCE);
    public static final IElementType TYPE_NAME = new IElementType("TYPE_NAME", UniLangLanguage.INSTANCE);
    public static final IElementType WHITESPACE = new IElementType("WHITESPACE", UniLangLanguage.INSTANCE);
    public static final IElementType NEWLINE = new IElementType("NEWLINE", UniLangLanguage.INSTANCE);
    public static final IElementType BAD_CHARACTER = new IElementType("BAD_CHARACTER", UniLangLanguage.INSTANCE);

    // Token sets
    public static final TokenSet COMMENTS = TokenSet.create(COMMENT, BLOCK_COMMENT);
    public static final TokenSet WHITESPACES = TokenSet.create(WHITESPACE, NEWLINE);
    public static final TokenSet STRINGS = TokenSet.create(STRING, TRIPLE_STRING, FSTRING);

    private UniLangTokenTypes() {
        // Utility class
    }
}
