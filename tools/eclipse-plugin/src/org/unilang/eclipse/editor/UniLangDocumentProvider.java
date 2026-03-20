package org.unilang.eclipse.editor;

import org.eclipse.core.filebuffers.IDocumentSetupParticipant;
import org.eclipse.jface.text.IDocument;
import org.eclipse.jface.text.IDocumentPartitioner;
import org.eclipse.jface.text.rules.FastPartitioner;
import org.eclipse.ui.editors.text.FileDocumentProvider;
import org.unilang.eclipse.syntax.UniLangPartitionScanner;

/**
 * Document provider for UniLang files.
 * Sets up document partitioning so the editor can distinguish between
 * code, comments, and string regions.
 */
public class UniLangDocumentProvider extends FileDocumentProvider
        implements IDocumentSetupParticipant {

    @Override
    protected IDocument createDocument(Object element) throws org.eclipse.core.runtime.CoreException {
        IDocument document = super.createDocument(element);
        if (document != null) {
            setupDocument(document);
        }
        return document;
    }

    /**
     * Sets up partitioning on the given document.
     * Called both when the editor creates documents and when used as a
     * document setup participant via the extension point.
     */
    @Override
    public void setup(IDocument document) {
        setupDocument(document);
    }

    private void setupDocument(IDocument document) {
        IDocumentPartitioner partitioner = new FastPartitioner(
                new UniLangPartitionScanner(),
                new String[] {
                    UniLangPartitionScanner.UNILANG_COMMENT,
                    UniLangPartitionScanner.UNILANG_BLOCK_COMMENT,
                    UniLangPartitionScanner.UNILANG_STRING,
                    UniLangPartitionScanner.UNILANG_TRIPLE_STRING,
                });
        partitioner.connect(document);
        document.setDocumentPartitioner(UniLangPartitionScanner.UNILANG_PARTITIONING, partitioner);
    }
}
