package org.unilang.eclipse.syntax;

import java.util.ArrayList;
import java.util.List;

import org.eclipse.jface.text.TextAttribute;
import org.eclipse.jface.text.rules.IRule;
import org.eclipse.jface.text.rules.IToken;
import org.eclipse.jface.text.rules.IWordDetector;
import org.eclipse.jface.text.rules.NumberRule;
import org.eclipse.jface.text.rules.RuleBasedScanner;
import org.eclipse.jface.text.rules.SingleLineRule;
import org.eclipse.jface.text.rules.Token;
import org.eclipse.jface.text.rules.WordRule;
import org.eclipse.swt.SWT;

/**
 * Token scanner for UniLang code regions (the default partition).
 * Provides highlighting rules for:
 * <ul>
 *   <li>Python keywords (blue)</li>
 *   <li>Java keywords (purple)</li>
 *   <li>Type names (teal)</li>
 *   <li>Numeric literals (cyan)</li>
 *   <li>String literals (green)</li>
 *   <li>Decorators (yellow)</li>
 * </ul>
 */
public class UniLangCodeScanner extends RuleBasedScanner {

    /** Python-specific keywords. */
    private static final String[] PYTHON_KEYWORDS = {
        "and", "as", "assert", "async", "await",
        "break", "class", "continue", "def", "del",
        "elif", "else", "except", "finally", "for",
        "from", "global", "if", "import", "in",
        "is", "lambda", "nonlocal", "not", "or",
        "pass", "raise", "return", "try", "while",
        "with", "yield", "None", "True", "False",
        "self", "print", "range", "len", "list",
        "dict", "set", "tuple",
    };

    /** Java-specific keywords. */
    private static final String[] JAVA_KEYWORDS = {
        "abstract", "boolean", "byte", "case", "catch",
        "char", "const", "default", "do", "double",
        "enum", "extends", "final", "float", "goto",
        "implements", "instanceof", "int", "interface", "long",
        "native", "new", "package", "private", "protected",
        "public", "short", "static", "strictfp", "super",
        "switch", "synchronized", "this", "throw", "throws",
        "transient", "void", "volatile",
    };

    /** Type names commonly used in UniLang. */
    private static final String[] TYPE_NAMES = {
        "String", "Integer", "Float", "Double", "Boolean",
        "List", "Map", "Set", "Optional", "Object",
        "Array", "HashMap", "ArrayList", "HashSet",
        "Tuple", "Dict", "Callable", "Iterator",
        "int", "float", "str", "bool", "bytes",
    };

    public UniLangCodeScanner(UniLangColorManager colorManager) {
        IToken pythonKeywordToken = new Token(
                new TextAttribute(colorManager.getColor(UniLangColorManager.PYTHON_KEYWORD), null, SWT.BOLD));
        IToken javaKeywordToken = new Token(
                new TextAttribute(colorManager.getColor(UniLangColorManager.JAVA_KEYWORD), null, SWT.BOLD));
        IToken typeToken = new Token(
                new TextAttribute(colorManager.getColor(UniLangColorManager.TYPE), null, SWT.NONE));
        IToken numberToken = new Token(
                new TextAttribute(colorManager.getColor(UniLangColorManager.NUMBER)));
        IToken stringToken = new Token(
                new TextAttribute(colorManager.getColor(UniLangColorManager.STRING)));
        IToken decoratorToken = new Token(
                new TextAttribute(colorManager.getColor(UniLangColorManager.DECORATOR), null, SWT.ITALIC));
        IToken defaultToken = new Token(
                new TextAttribute(colorManager.getColor(UniLangColorManager.DEFAULT)));

        setDefaultReturnToken(defaultToken);

        List<IRule> rules = new ArrayList<>();

        // Strings (in case they appear inside the code partition)
        rules.add(new SingleLineRule("\"", "\"", stringToken, '\\'));
        rules.add(new SingleLineRule("'", "'", stringToken, '\\'));

        // Decorator pattern: @word
        rules.add(new SingleLineRule("@", " ", decoratorToken, (char) 0, true));

        // Numbers
        rules.add(new NumberRule(numberToken));

        // Keywords and types via WordRule
        WordRule wordRule = new WordRule(new UniLangWordDetector(), defaultToken);

        for (String kw : PYTHON_KEYWORDS) {
            wordRule.addWord(kw, pythonKeywordToken);
        }
        for (String kw : JAVA_KEYWORDS) {
            wordRule.addWord(kw, javaKeywordToken);
        }
        for (String type : TYPE_NAMES) {
            wordRule.addWord(type, typeToken);
        }

        rules.add(wordRule);

        setRules(rules.toArray(new IRule[0]));
    }

    /**
     * Word detector for UniLang identifiers.
     */
    private static class UniLangWordDetector implements IWordDetector {
        @Override
        public boolean isWordStart(char c) {
            return Character.isJavaIdentifierStart(c);
        }

        @Override
        public boolean isWordPart(char c) {
            return Character.isJavaIdentifierPart(c);
        }
    }
}
