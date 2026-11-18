package com.jediterm.terminal.ui;

import java.awt.Rectangle;
import java.awt.im.InputMethodRequests;
import java.awt.font.TextHitInfo;
import java.text.AttributedCharacterIterator;
import javax.swing.JComponent;

/** Geometry-only stand-in for JediTerm; no IDE window or input source. */
public class TerminalPanel extends JComponent {
    private static final long serialVersionUID = 1L;
    public Rectangle rectangle = new Rectangle(-1500, 800, 0, 0);
    public TerminalPanel() { setFont(new java.awt.Font("Monospaced", java.awt.Font.PLAIN, 16)); }
    @Override public boolean isShowing() { return true; }
    @Override public InputMethodRequests getInputMethodRequests() {
        return new InputMethodRequests() {
            public Rectangle getTextLocation(TextHitInfo offset) { return rectangle; }
            public TextHitInfo getLocationOffset(int x, int y) { return null; }
            public int getInsertPositionOffset() { return 0; }
            public AttributedCharacterIterator getCommittedText(int a, int b, AttributedCharacterIterator.Attribute[] c) { throw new AssertionError("Must not read text"); }
            public int getCommittedTextLength() { return 0; }
            public AttributedCharacterIterator cancelLatestCommittedText(AttributedCharacterIterator.Attribute[] a) { throw new AssertionError("Must not edit text"); }
            public AttributedCharacterIterator getSelectedText(AttributedCharacterIterator.Attribute[] a) { throw new AssertionError("Must not read text"); }
        };
    }
}
