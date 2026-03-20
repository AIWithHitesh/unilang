package org.unilang.intellij;

import com.intellij.lang.Language;

public class UniLangLanguage extends Language {

    public static final UniLangLanguage INSTANCE = new UniLangLanguage();

    private UniLangLanguage() {
        super("UniLang");
    }
}
