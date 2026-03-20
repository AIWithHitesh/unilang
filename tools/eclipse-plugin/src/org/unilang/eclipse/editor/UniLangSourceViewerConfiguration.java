package org.unilang.eclipse.editor;

import org.eclipse.jface.text.IDocument;
import org.eclipse.jface.text.contentassist.ContentAssistant;
import org.eclipse.jface.text.contentassist.IContentAssistant;
import org.eclipse.jface.text.presentation.IPresentationReconciler;
import org.eclipse.jface.text.presentation.PresentationReconciler;
import org.eclipse.jface.text.rules.DefaultDamagerRepairer;
import org.eclipse.jface.text.source.ISourceViewer;
import org.eclipse.ui.editors.text.TextSourceViewerConfiguration;
import org.unilang.eclipse.completion.UniLangCompletionProcessor;
import org.unilang.eclipse.syntax.UniLangCodeScanner;
import org.unilang.eclipse.syntax.UniLangColorManager;
import org.unilang.eclipse.syntax.UniLangPartitionScanner;

/**
 * Source viewer configuration for the UniLang editor.
 * Configures syntax highlighting via a presentation reconciler
 * and content assist for auto-completion.
 */
public class UniLangSourceViewerConfiguration extends TextSourceViewerConfiguration {

    private final UniLangColorManager colorManager;
    private UniLangCodeScanner codeScanner;

    public UniLangSourceViewerConfiguration(UniLangColorManager colorManager) {
        this.colorManager = colorManager;
    }

    @Override
    public String[] getConfiguredContentTypes(ISourceViewer sourceViewer) {
        return new String[] {
            IDocument.DEFAULT_CONTENT_TYPE,
            UniLangPartitionScanner.UNILANG_COMMENT,
            UniLangPartitionScanner.UNILANG_BLOCK_COMMENT,
            UniLangPartitionScanner.UNILANG_STRING,
            UniLangPartitionScanner.UNILANG_TRIPLE_STRING,
        };
    }

    @Override
    public String getConfiguredDocumentPartitioning(ISourceViewer sourceViewer) {
        return UniLangPartitionScanner.UNILANG_PARTITIONING;
    }

    private UniLangCodeScanner getCodeScanner() {
        if (codeScanner == null) {
            codeScanner = new UniLangCodeScanner(colorManager);
        }
        return codeScanner;
    }

    @Override
    public IPresentationReconciler getPresentationReconciler(ISourceViewer sourceViewer) {
        PresentationReconciler reconciler = new PresentationReconciler();
        reconciler.setDocumentPartitioning(getConfiguredDocumentPartitioning(sourceViewer));

        // Default content type (code)
        DefaultDamagerRepairer dr = new DefaultDamagerRepairer(getCodeScanner());
        reconciler.setDamager(dr, IDocument.DEFAULT_CONTENT_TYPE);
        reconciler.setRepairer(dr, IDocument.DEFAULT_CONTENT_TYPE);

        // Single-line comments
        DefaultDamagerRepairer commentDR = new DefaultDamagerRepairer(
                new SingleTokenScanner(colorManager.getColor(UniLangColorManager.COMMENT)));
        reconciler.setDamager(commentDR, UniLangPartitionScanner.UNILANG_COMMENT);
        reconciler.setRepairer(commentDR, UniLangPartitionScanner.UNILANG_COMMENT);

        // Block comments
        DefaultDamagerRepairer blockCommentDR = new DefaultDamagerRepairer(
                new SingleTokenScanner(colorManager.getColor(UniLangColorManager.COMMENT)));
        reconciler.setDamager(blockCommentDR, UniLangPartitionScanner.UNILANG_BLOCK_COMMENT);
        reconciler.setRepairer(blockCommentDR, UniLangPartitionScanner.UNILANG_BLOCK_COMMENT);

        // Strings
        DefaultDamagerRepairer stringDR = new DefaultDamagerRepairer(
                new SingleTokenScanner(colorManager.getColor(UniLangColorManager.STRING)));
        reconciler.setDamager(stringDR, UniLangPartitionScanner.UNILANG_STRING);
        reconciler.setRepairer(stringDR, UniLangPartitionScanner.UNILANG_STRING);

        // Triple-quoted strings
        DefaultDamagerRepairer tripleStringDR = new DefaultDamagerRepairer(
                new SingleTokenScanner(colorManager.getColor(UniLangColorManager.STRING)));
        reconciler.setDamager(tripleStringDR, UniLangPartitionScanner.UNILANG_TRIPLE_STRING);
        reconciler.setRepairer(tripleStringDR, UniLangPartitionScanner.UNILANG_TRIPLE_STRING);

        return reconciler;
    }

    @Override
    public IContentAssistant getContentAssistant(ISourceViewer sourceViewer) {
        ContentAssistant assistant = new ContentAssistant();
        assistant.setContentAssistProcessor(
                new UniLangCompletionProcessor(), IDocument.DEFAULT_CONTENT_TYPE);
        assistant.enableAutoActivation(true);
        assistant.setAutoActivationDelay(200);
        assistant.setProposalPopupOrientation(IContentAssistant.PROPOSAL_OVERLAY);
        assistant.setContextInformationPopupOrientation(IContentAssistant.CONTEXT_INFO_ABOVE);
        return assistant;
    }
}
