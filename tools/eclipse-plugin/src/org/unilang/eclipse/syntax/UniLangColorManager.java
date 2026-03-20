package org.unilang.eclipse.syntax;

import java.util.HashMap;
import java.util.Map;

import org.eclipse.swt.graphics.Color;
import org.eclipse.swt.graphics.RGB;
import org.eclipse.swt.widgets.Display;

/**
 * Manages colors used for syntax highlighting in the UniLang editor.
 * Colors are lazily created and cached; call {@link #dispose()} when
 * the editor is closed to free SWT resources.
 */
public class UniLangColorManager {

    // Color constant keys
    public static final RGB KEYWORD = new RGB(100, 100, 200);
    public static final RGB PYTHON_KEYWORD = new RGB(86, 156, 214);    // Blue
    public static final RGB JAVA_KEYWORD = new RGB(180, 120, 220);     // Purple
    public static final RGB STRING = new RGB(106, 171, 115);           // Green
    public static final RGB COMMENT = new RGB(128, 128, 128);          // Gray
    public static final RGB NUMBER = new RGB(100, 200, 200);           // Cyan
    public static final RGB TYPE = new RGB(78, 201, 176);              // Teal
    public static final RGB DECORATOR = new RGB(220, 200, 100);        // Yellow
    public static final RGB DEFAULT = new RGB(212, 212, 212);          // Light gray (for dark themes)

    private final Map<RGB, Color> colorTable = new HashMap<>();

    /**
     * Returns the {@link Color} for the given RGB value.
     * Colors are cached and reused across calls.
     *
     * @param rgb the RGB color value
     * @return the SWT Color instance
     */
    public Color getColor(RGB rgb) {
        Color color = colorTable.get(rgb);
        if (color == null) {
            color = new Color(Display.getCurrent(), rgb);
            colorTable.put(rgb, color);
        }
        return color;
    }

    /**
     * Disposes all cached SWT Color resources.
     * Must be called when the editor is closed.
     */
    public void dispose() {
        for (Color color : colorTable.values()) {
            color.dispose();
        }
        colorTable.clear();
    }
}
