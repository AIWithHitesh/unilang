package org.unilang.intellij;

import com.intellij.openapi.fileTypes.LanguageFileType;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import javax.swing.Icon;

public class UniLangFileType extends LanguageFileType {

    public static final UniLangFileType INSTANCE = new UniLangFileType();

    private UniLangFileType() {
        super(UniLangLanguage.INSTANCE);
    }

    @NotNull
    @Override
    public String getName() {
        return "UniLang";
    }

    @NotNull
    @Override
    public String getDescription() {
        return "UniLang source file";
    }

    @NotNull
    @Override
    public String getDefaultExtension() {
        return "uniL";
    }

    @Nullable
    @Override
    public Icon getIcon() {
        return UniLangIcons.FILE;
    }
}
