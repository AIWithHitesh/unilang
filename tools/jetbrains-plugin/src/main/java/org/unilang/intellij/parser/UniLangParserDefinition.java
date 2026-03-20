package org.unilang.intellij.parser;

import com.intellij.extapi.psi.ASTWrapperPsiElement;
import com.intellij.lang.ASTNode;
import com.intellij.lang.ParserDefinition;
import com.intellij.lang.PsiParser;
import com.intellij.lexer.Lexer;
import com.intellij.openapi.project.Project;
import com.intellij.psi.FileViewProvider;
import com.intellij.psi.PsiElement;
import com.intellij.psi.PsiFile;
import com.intellij.psi.tree.IFileElementType;
import com.intellij.psi.tree.TokenSet;
import org.jetbrains.annotations.NotNull;
import org.unilang.intellij.UniLangLanguage;
import org.unilang.intellij.syntax.UniLangLexer;
import org.unilang.intellij.syntax.UniLangTokenTypes;

public class UniLangParserDefinition implements ParserDefinition {

    public static final IFileElementType FILE = new IFileElementType(UniLangLanguage.INSTANCE);

    @NotNull
    @Override
    public Lexer createLexer(Project project) {
        return new UniLangLexer();
    }

    @NotNull
    @Override
    public PsiParser createParser(Project project) {
        return (root, builder) -> {
            var marker = builder.mark();
            while (!builder.eof()) {
                builder.advanceLexer();
            }
            marker.done(root);
            return builder.getTreeBuilt();
        };
    }

    @NotNull
    @Override
    public IFileElementType getFileNodeType() {
        return FILE;
    }

    @NotNull
    @Override
    public TokenSet getCommentTokens() {
        return UniLangTokenTypes.COMMENTS;
    }

    @NotNull
    @Override
    public TokenSet getStringLiteralElements() {
        return UniLangTokenTypes.STRINGS;
    }

    @NotNull
    @Override
    public TokenSet getWhitespaceTokens() {
        return UniLangTokenTypes.WHITESPACES;
    }

    @NotNull
    @Override
    public PsiElement createElement(ASTNode node) {
        return new ASTWrapperPsiElement(node);
    }

    @NotNull
    @Override
    public PsiFile createFile(@NotNull FileViewProvider viewProvider) {
        return new UniLangPsiFile(viewProvider);
    }
}
