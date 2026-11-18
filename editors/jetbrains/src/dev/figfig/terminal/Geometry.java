package dev.figfig.terminal;

import java.awt.Component;
import java.awt.FontMetrics;
import java.awt.Rectangle;
import java.awt.im.InputMethodRequests;
import java.util.Map;
import java.util.WeakHashMap;

/** Called on the EDT. Never obtains terminal text or handles a key event. */
public final class Geometry {
    private final Map<Component, Long> focuses = new WeakHashMap<>();
    private long sequence;
    private final Map<Component, Boolean> composing = new WeakHashMap<>();

    public record Position(double x, double y, double height, long focus) {}

    public Position capture(Component component) {
        if (component == null || !component.isShowing() || !isTerminal(component) || composing.containsKey(component)) return null;
        if (component instanceof javax.swing.text.JTextComponent text && text.getSelectionStart() != text.getSelectionEnd()) return null;
        InputMethodRequests input = component.getInputMethodRequests();
        if (input == null) return null;
        Rectangle rect = input.getTextLocation(null);
        if (rect == null) return null;
        double height = rect.height;
        double y = rect.y;
        if (height == 0) {
            // JediTerm returns the point below the current row, not a caret box.
            if (component.getFont() == null) return null;
            FontMetrics metrics = component.getFontMetrics(component.getFont());
            height = metrics.getHeight();
            y -= height;
        }
        if (!Double.isFinite(rect.x) || !Double.isFinite(y) || height < 1 || height > 200) return null;
        long focus = focuses.computeIfAbsent(component, ignored -> ++sequence);
        return new Position(rect.x, y, height, focus);
    }

    public void setComposing(Component component, boolean active) {
        if (active) composing.put(component, true);
        else composing.remove(component);
    }

    static boolean isTerminal(Component component) {
        boolean terminal = false;
        for (Component parent = component; parent != null; parent = parent.getParent()) {
            for (Class<?> type = parent.getClass(); type != null; type = type.getSuperclass()) {
                String name = type.getName();
                if (name.contains("Search") || name.contains("Find")) return false;
                if (name.equals("com.jediterm.terminal.ui.TerminalPanel") ||
                    name.equals("com.intellij.terminal.frontend.view.impl.TerminalViewImpl$TerminalPanel") ||
                    name.equals("org.jetbrains.plugins.terminal.block.SimpleTerminalView$SimpleTerminalPanel")) {
                    terminal = true;
                }
            }
        }
        // Reject search fields even when they are descendants of a terminal panel.
        return terminal && (!(component instanceof javax.swing.text.JTextComponent) ||
            component.getClass().getName().equals("com.intellij.openapi.editor.impl.EditorComponentImpl"));
    }
}
