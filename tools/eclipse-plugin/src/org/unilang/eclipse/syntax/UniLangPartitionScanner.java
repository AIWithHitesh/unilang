package org.unilang.eclipse.syntax;

import org.eclipse.jface.text.rules.EndOfLineRule;
import org.eclipse.jface.text.rules.IPredicateRule;
import org.eclipse.jface.text.rules.IToken;
import org.eclipse.jface.text.rules.MultiLineRule;
import org.eclipse.jface.text.rules.RuleBasedPartitionScanner;
import org.eclipse.jface.text.rules.SingleLineRule;
import org.eclipse.jface.text.rules.Token;

/**
 * Partition scanner for UniLang source files.
 * Splits the document into distinct regions (partitions):
 * <ul>
 *   <li>Code (default partition)</li>
 *   <li>Single-line comments: {@code //} and {@code #}</li>
 *   <li>Block comments: {@code /* ... * /}</li>
 *   <li>Strings: single and double quoted</li>
 *   <li>Triple-quoted strings: Python-style {@code """..."""} and {@code '''...'''}</li>
 * </ul>
 */
public class UniLangPartitionScanner extends RuleBasedPartitionScanner {

    /** Partitioning ID used to connect the partitioner to documents. */
    public static final String UNILANG_PARTITIONING = "org.unilang.eclipse.partitioning";

    /** Partition type for single-line comments (// and #). */
    public static final String UNILANG_COMMENT = "__unilang_comment";

    /** Partition type for block comments. */
    public static final String UNILANG_BLOCK_COMMENT = "__unilang_block_comment";

    /** Partition type for regular strings. */
    public static final String UNILANG_STRING = "__unilang_string";

    /** Partition type for triple-quoted strings. */
    public static final String UNILANG_TRIPLE_STRING = "__unilang_triple_string";

    public UniLangPartitionScanner() {
        IToken commentToken = new Token(UNILANG_COMMENT);
        IToken blockCommentToken = new Token(UNILANG_BLOCK_COMMENT);
        IToken stringToken = new Token(UNILANG_STRING);
        IToken tripleStringToken = new Token(UNILANG_TRIPLE_STRING);

        IPredicateRule[] rules = new IPredicateRule[] {
            // Triple-quoted strings must come before single-line strings
            new MultiLineRule("\"\"\"", "\"\"\"", tripleStringToken, '\\'),
            new MultiLineRule("'''", "'''", tripleStringToken, '\\'),

            // Block comments
            new MultiLineRule("/*", "*/", blockCommentToken, (char) 0, true),

            // Single-line comments
            new EndOfLineRule("//", commentToken),
            new EndOfLineRule("#", commentToken),

            // Regular strings
            new SingleLineRule("\"", "\"", stringToken, '\\'),
            new SingleLineRule("'", "'", stringToken, '\\'),
        };

        setPredicateRules(rules);
    }
}
