package org.unilang.eclipse.editor;

import org.eclipse.ui.editors.text.TextEditor;
import org.unilang.eclipse.syntax.UniLangColorManager;

/**
 * The UniLang text editor. Extends TextEditor (which itself extends
 * AbstractDecoratedTextEditor) to provide syntax highlighting,
 * content assist, and proper document partitioning for .uniL files.
 */
public class UniLangEditor extends TextEditor {

    private final UniLangColorManager colorManager;

    public UniLangEditor() {
        super();
        colorManager = new UniLangColorManager();
        setSourceViewerConfiguration(new UniLangSourceViewerConfiguration(colorManager));
        setDocumentProvider(new UniLangDocumentProvider());
    }

    @Override
    public void dispose() {
        colorManager.dispose();
        super.dispose();
    }

    @Override
    protected boolean isLineNumberRulerVisible() {
        return true;
    }

    @Override
    protected boolean isOverviewRulerVisible() {
        return true;
    }
}
