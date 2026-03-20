package org.unilang.intellij.annotator;

import com.intellij.lang.annotation.AnnotationHolder;
import com.intellij.lang.annotation.Annotator;
import com.intellij.lang.annotation.HighlightSeverity;
import com.intellij.openapi.editor.colors.TextAttributesKey;
import com.intellij.openapi.util.TextRange;
import com.intellij.psi.PsiElement;
import com.intellij.psi.PsiFile;
import org.jetbrains.annotations.NotNull;

import java.util.ArrayDeque;
import java.util.Deque;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class UniLangAnnotator implements Annotator {

    private static final Pattern TODO_PATTERN = Pattern.compile("\\b(TODO|FIXME|HACK|XXX)\\b");
    private static final Pattern BAD_ESCAPE_PATTERN = Pattern.compile("\\\\[^\\\\nrtbf'\"0xuUN\n\r]");

    @Override
    public void annotate(@NotNull PsiElement element, @NotNull AnnotationHolder holder) {
        // Only process the file-level element to avoid duplicate annotations
        if (!(element instanceof PsiFile)) {
            return;
        }

        String text = element.getText();
        int length = text.length();

        checkUnmatchedBrackets(text, length, element, holder);
        checkTodoComments(text, element, holder);
        checkBadEscapes(text, length, element, holder);
    }

    private void checkUnmatchedBrackets(String text, int length, PsiElement element, AnnotationHolder holder) {
        Deque<Integer> parenStack = new ArrayDeque<>();
        Deque<Integer> bracketStack = new ArrayDeque<>();
        Deque<Integer> braceStack = new ArrayDeque<>();

        boolean inString = false;
        boolean inLineComment = false;
        boolean inBlockComment = false;
        char stringChar = 0;

        for (int i = 0; i < length; i++) {
            char c = text.charAt(i);

            // Handle newlines
            if (c == '\n') {
                inLineComment = false;
                continue;
            }

            // Skip line comments
            if (inLineComment) continue;

            // Handle block comments
            if (inBlockComment) {
                if (c == '*' && i + 1 < length && text.charAt(i + 1) == '/') {
                    inBlockComment = false;
                    i++;
                }
                continue;
            }

            // Handle strings
            if (inString) {
                if (c == '\\') {
                    i++; // skip escaped char
                    continue;
                }
                if (c == stringChar) {
                    inString = false;
                }
                continue;
            }

            // Check for comment starts
            if (c == '#') {
                inLineComment = true;
                continue;
            }
            if (c == '/' && i + 1 < length) {
                if (text.charAt(i + 1) == '/') {
                    inLineComment = true;
                    continue;
                }
                if (text.charAt(i + 1) == '*') {
                    inBlockComment = true;
                    i++;
                    continue;
                }
            }

            // Check for string starts
            if (c == '"' || c == '\'') {
                inString = true;
                stringChar = c;
                continue;
            }

            // Track brackets
            switch (c) {
                case '(' -> parenStack.push(i);
                case ')' -> {
                    if (parenStack.isEmpty()) {
                        annotateError(holder, element, i, i + 1, "Unmatched closing parenthesis ')'");
                    } else {
                        parenStack.pop();
                    }
                }
                case '[' -> bracketStack.push(i);
                case ']' -> {
                    if (bracketStack.isEmpty()) {
                        annotateError(holder, element, i, i + 1, "Unmatched closing bracket ']'");
                    } else {
                        bracketStack.pop();
                    }
                }
                case '{' -> braceStack.push(i);
                case '}' -> {
                    if (braceStack.isEmpty()) {
                        annotateError(holder, element, i, i + 1, "Unmatched closing brace '}'");
                    } else {
                        braceStack.pop();
                    }
                }
            }
        }

        // Report unmatched opening brackets
        for (int pos : parenStack) {
            annotateError(holder, element, pos, pos + 1, "Unmatched opening parenthesis '('");
        }
        for (int pos : bracketStack) {
            annotateError(holder, element, pos, pos + 1, "Unmatched opening bracket '['");
        }
        for (int pos : braceStack) {
            annotateError(holder, element, pos, pos + 1, "Unmatched opening brace '{'");
        }
    }

    private void checkTodoComments(String text, PsiElement element, AnnotationHolder holder) {
        int offset = element.getTextRange().getStartOffset();
        String[] lines = text.split("\n", -1);
        int pos = 0;

        for (String line : lines) {
            String trimmed = line.trim();
            boolean isComment = trimmed.startsWith("#") || trimmed.startsWith("//");

            // Also check for block comment lines
            if (isComment || trimmed.startsWith("*") || trimmed.startsWith("/*")) {
                Matcher matcher = TODO_PATTERN.matcher(line);
                while (matcher.find()) {
                    int start = offset + pos + matcher.start();
                    int end = offset + pos + matcher.end();
                    holder.newAnnotation(HighlightSeverity.INFORMATION,
                                    matcher.group(1) + " comment")
                            .range(new TextRange(start, end))
                            .create();
                }
            }
            pos += line.length() + 1; // +1 for the newline
        }
    }

    private void checkBadEscapes(String text, int length, PsiElement element, AnnotationHolder holder) {
        int offset = element.getTextRange().getStartOffset();
        boolean inString = false;
        boolean isRawString = false;
        char stringChar = 0;

        for (int i = 0; i < length; i++) {
            char c = text.charAt(i);

            if (!inString) {
                // Check for raw string prefix
                if ((c == 'r' || c == 'R') && i + 1 < length
                        && (text.charAt(i + 1) == '"' || text.charAt(i + 1) == '\'')) {
                    isRawString = true;
                    i++;
                    inString = true;
                    stringChar = text.charAt(i);
                    continue;
                }
                // Check for f-string prefix
                if ((c == 'f' || c == 'F') && i + 1 < length
                        && (text.charAt(i + 1) == '"' || text.charAt(i + 1) == '\'')) {
                    isRawString = false;
                    i++;
                    inString = true;
                    stringChar = text.charAt(i);
                    continue;
                }
                if (c == '"' || c == '\'') {
                    inString = true;
                    isRawString = false;
                    stringChar = c;
                    continue;
                }
                // Skip comments
                if (c == '#') break;
                if (c == '/' && i + 1 < length && text.charAt(i + 1) == '/') break;
            } else {
                if (c == '\\' && !isRawString && i + 1 < length) {
                    char next = text.charAt(i + 1);
                    if (!isValidEscape(next)) {
                        int start = offset + i;
                        int end = offset + i + 2;
                        holder.newAnnotation(HighlightSeverity.WARNING,
                                        "Unknown escape sequence '\\" + next + "'")
                                .range(new TextRange(start, end))
                                .create();
                    }
                    i++; // skip the escaped character
                    continue;
                }
                if (c == stringChar) {
                    inString = false;
                    isRawString = false;
                }
                if (c == '\n') {
                    inString = false;
                    isRawString = false;
                }
            }
        }
    }

    private boolean isValidEscape(char c) {
        return c == '\\' || c == '\'' || c == '"' || c == 'n' || c == 'r'
                || c == 't' || c == 'b' || c == 'f' || c == '0'
                || c == 'x' || c == 'u' || c == 'U' || c == 'N'
                || c == '\n' || c == '\r' || c == 'a' || c == 'v'
                || c == '{' || c == '}';
    }

    private void annotateError(AnnotationHolder holder, PsiElement element, int start, int end, String message) {
        int offset = element.getTextRange().getStartOffset();
        holder.newAnnotation(HighlightSeverity.ERROR, message)
                .range(new TextRange(offset + start, offset + end))
                .create();
    }
}
