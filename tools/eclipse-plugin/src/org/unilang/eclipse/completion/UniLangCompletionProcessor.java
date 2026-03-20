package org.unilang.eclipse.completion;

import java.util.ArrayList;
import java.util.List;

import org.eclipse.jface.text.ITextViewer;
import org.eclipse.jface.text.contentassist.CompletionProposal;
import org.eclipse.jface.text.contentassist.ICompletionProposal;
import org.eclipse.jface.text.contentassist.IContentAssistProcessor;
import org.eclipse.jface.text.contentassist.IContextInformation;
import org.eclipse.jface.text.contentassist.IContextInformationValidator;

/**
 * Content assist processor for UniLang.
 * Provides auto-completion proposals for Python keywords, Java keywords,
 * type names, and common UniLang patterns.
 */
public class UniLangCompletionProcessor implements IContentAssistProcessor {

    /** All completable keywords and identifiers. */
    private static final String[] PROPOSALS = {
        // Python keywords
        "and", "as", "assert", "async", "await",
        "break", "class", "continue", "def", "del",
        "elif", "else", "except", "finally", "for",
        "from", "global", "if", "import", "in",
        "is", "lambda", "nonlocal", "not", "or",
        "pass", "raise", "return", "try", "while",
        "with", "yield", "None", "True", "False",
        "self", "print", "range", "len",
        // Java keywords
        "abstract", "boolean", "byte", "case", "catch",
        "char", "const", "default", "do", "double",
        "enum", "extends", "final", "float", "goto",
        "implements", "instanceof", "int", "interface", "long",
        "native", "new", "package", "private", "protected",
        "public", "short", "static", "strictfp", "super",
        "switch", "synchronized", "this", "throw", "throws",
        "transient", "void", "volatile",
        // Type names
        "String", "Integer", "Float", "Double", "Boolean",
        "List", "Map", "Set", "Optional", "Object",
        "Array", "HashMap", "ArrayList", "HashSet",
        "Tuple", "Dict", "Callable", "Iterator",
        // Common patterns
        "def __init__(self):",
        "public static void main(String[] args)",
        "class ${name}:",
        "if __name__ == \"__main__\":",
        "try:\n    \nexcept Exception as e:\n    ",
        "for i in range():",
        "while True:",
        "import java.util.*",
        "@Override",
        "@staticmethod",
        "@classmethod",
        "@property",
    };

    @Override
    public ICompletionProposal[] computeCompletionProposals(ITextViewer viewer, int offset) {
        String text = viewer.getDocument().get();
        String prefix = extractPrefix(text, offset);
        List<ICompletionProposal> proposals = new ArrayList<>();

        for (String proposal : PROPOSALS) {
            if (prefix.isEmpty() || proposal.toLowerCase().startsWith(prefix.toLowerCase())) {
                proposals.add(new CompletionProposal(
                        proposal,
                        offset - prefix.length(),
                        prefix.length(),
                        proposal.length(),
                        null,
                        proposal,
                        null,
                        getDescriptionFor(proposal)));
            }
        }

        return proposals.toArray(new ICompletionProposal[0]);
    }

    /**
     * Extracts the word prefix at the given offset by scanning backwards
     * for identifier characters.
     */
    private String extractPrefix(String text, int offset) {
        int start = offset;
        while (start > 0 && Character.isJavaIdentifierPart(text.charAt(start - 1))) {
            start--;
        }
        return text.substring(start, offset);
    }

    /**
     * Returns a brief description string for common proposals.
     */
    private String getDescriptionFor(String proposal) {
        if (isPythonKeyword(proposal)) {
            return "Python keyword";
        } else if (isJavaKeyword(proposal)) {
            return "Java keyword";
        } else if (Character.isUpperCase(proposal.charAt(0)) && proposal.indexOf(' ') < 0) {
            return "Type name";
        } else if (proposal.startsWith("@")) {
            return "Decorator / Annotation";
        } else {
            return "UniLang pattern";
        }
    }

    private boolean isPythonKeyword(String word) {
        String[] pyKw = {
            "and", "as", "assert", "async", "await", "break", "class", "continue",
            "def", "del", "elif", "else", "except", "finally", "for", "from",
            "global", "if", "import", "in", "is", "lambda", "nonlocal", "not",
            "or", "pass", "raise", "return", "try", "while", "with", "yield",
            "None", "True", "False", "self", "print", "range", "len",
        };
        for (String k : pyKw) {
            if (k.equals(word)) return true;
        }
        return false;
    }

    private boolean isJavaKeyword(String word) {
        String[] javaKw = {
            "abstract", "boolean", "byte", "case", "catch", "char", "const",
            "default", "do", "double", "enum", "extends", "final", "float",
            "goto", "implements", "instanceof", "int", "interface", "long",
            "native", "new", "package", "private", "protected", "public",
            "short", "static", "strictfp", "super", "switch", "synchronized",
            "this", "throw", "throws", "transient", "void", "volatile",
        };
        for (String k : javaKw) {
            if (k.equals(word)) return true;
        }
        return false;
    }

    @Override
    public IContextInformation[] computeContextInformation(ITextViewer viewer, int offset) {
        return null;
    }

    @Override
    public char[] getCompletionProposalAutoActivationCharacters() {
        return new char[] { '.' };
    }

    @Override
    public char[] getContextInformationAutoActivationCharacters() {
        return null;
    }

    @Override
    public String getErrorMessage() {
        return null;
    }

    @Override
    public IContextInformationValidator getContextInformationValidator() {
        return null;
    }
}
