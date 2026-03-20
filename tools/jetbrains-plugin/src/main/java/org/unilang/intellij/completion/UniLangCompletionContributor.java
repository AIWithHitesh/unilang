package org.unilang.intellij.completion;

import com.intellij.codeInsight.completion.*;
import com.intellij.codeInsight.lookup.LookupElementBuilder;
import com.intellij.patterns.PlatformPatterns;
import com.intellij.util.ProcessingContext;
import org.jetbrains.annotations.NotNull;
import org.unilang.intellij.UniLangIcons;
import org.unilang.intellij.UniLangLanguage;

public class UniLangCompletionContributor extends CompletionContributor {

    private static final String[] PYTHON_KEYWORDS = {
            "def", "class", "if", "elif", "else", "for", "while", "return",
            "import", "from", "as", "with", "try", "except", "finally",
            "raise", "pass", "break", "continue", "and", "or", "not",
            "in", "is", "lambda", "yield", "global", "nonlocal", "assert",
            "del", "True", "False", "None", "async", "await", "self",
            "print", "range", "len", "isinstance", "type"
    };

    private static final String[] JAVA_KEYWORDS = {
            "public", "private", "protected", "static", "final", "abstract",
            "class", "interface", "extends", "implements", "new", "this",
            "super", "void", "return", "if", "else", "for", "while", "do",
            "switch", "case", "default", "break", "continue", "try", "catch",
            "finally", "throw", "throws", "import", "package", "instanceof",
            "synchronized", "volatile", "transient", "native", "enum",
            "var", "record", "sealed", "permits"
    };

    private static final String[] BUILTIN_TYPES = {
            "int", "float", "double", "long", "short", "byte", "char",
            "boolean", "String", "List", "Dict", "Set", "Tuple", "Optional",
            "Map", "Array", "Vector", "HashMap", "ArrayList", "Integer",
            "Float", "Double", "Long", "Boolean", "Object", "str", "bool",
            "list", "dict", "set", "tuple"
    };

    private static final String[] COMMON_PATTERNS = {
            "def __init__(self):",
            "def __str__(self):",
            "def __repr__(self):",
            "if __name__ == \"__main__\":",
            "public static void main(String[] args) {",
            "class ${name}:",
            "class ${name} {",
            "for i in range(",
            "for (int i = 0; i < ",
            "try:\n    \nexcept Exception as e:",
            "try {\n    \n} catch (Exception e) {",
            "import java.util.*",
            "from typing import"
    };

    public UniLangCompletionContributor() {
        extend(CompletionType.BASIC,
                PlatformPatterns.psiElement().withLanguage(UniLangLanguage.INSTANCE),
                new CompletionProvider<>() {
                    @Override
                    protected void addCompletions(@NotNull CompletionParameters parameters,
                                                  @NotNull ProcessingContext context,
                                                  @NotNull CompletionResultSet resultSet) {
                        // Python keywords
                        for (String keyword : PYTHON_KEYWORDS) {
                            resultSet.addElement(
                                    LookupElementBuilder.create(keyword)
                                            .withIcon(UniLangIcons.FILE)
                                            .withTypeText("Python keyword")
                                            .bold()
                            );
                        }

                        // Java keywords
                        for (String keyword : JAVA_KEYWORDS) {
                            resultSet.addElement(
                                    LookupElementBuilder.create(keyword)
                                            .withIcon(UniLangIcons.FILE)
                                            .withTypeText("Java keyword")
                                            .bold()
                            );
                        }

                        // Built-in types
                        for (String type : BUILTIN_TYPES) {
                            resultSet.addElement(
                                    LookupElementBuilder.create(type)
                                            .withIcon(UniLangIcons.FILE)
                                            .withTypeText("Built-in type")
                            );
                        }

                        // Common patterns
                        for (String pattern : COMMON_PATTERNS) {
                            String display = pattern.length() > 40
                                    ? pattern.substring(0, 40) + "..."
                                    : pattern;
                            resultSet.addElement(
                                    LookupElementBuilder.create(pattern)
                                            .withPresentableText(display)
                                            .withIcon(UniLangIcons.FILE)
                                            .withTypeText("Pattern")
                            );
                        }
                    }
                }
        );
    }
}
