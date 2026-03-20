package org.unilang.intellij.parser;

import com.intellij.extapi.psi.PsiFileBase;
import com.intellij.openapi.fileTypes.FileType;
import com.intellij.psi.FileViewProvider;
import org.jetbrains.annotations.NotNull;
import org.unilang.intellij.UniLangFileType;
import org.unilang.intellij.UniLangLanguage;

public class UniLangPsiFile extends PsiFileBase {

    public UniLangPsiFile(@NotNull FileViewProvider viewProvider) {
        super(viewProvider, UniLangLanguage.INSTANCE);
    }

    @NotNull
    @Override
    public FileType getFileType() {
        return UniLangFileType.INSTANCE;
    }

    @Override
    public String toString() {
        return "UniLang File";
    }
}
