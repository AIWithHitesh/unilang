package org.unilang.eclipse.editor;

import org.eclipse.jface.text.TextAttribute;
import org.eclipse.jface.text.rules.IRule;
import org.eclipse.jface.text.rules.RuleBasedScanner;
import org.eclipse.jface.text.rules.Token;
import org.eclipse.swt.graphics.Color;

/**
 * A simple scanner that returns a single token for the entire partition.
 * Used for comment and string partitions where the whole region shares one style.
 */
public class SingleTokenScanner extends RuleBasedScanner {

    public SingleTokenScanner(Color color) {
        setDefaultReturnToken(new Token(new TextAttribute(color)));
        setRules(new IRule[0]);
    }
}
