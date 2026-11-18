package dev.figfig.terminal;

import com.jediterm.terminal.ui.TerminalPanel;
import java.awt.Rectangle;
import javax.swing.JPanel;
import javax.swing.JTextField;

public final class AdapterTest {
    public static void main(String[] args) throws Exception {
        Geometry geometry = new Geometry();
        TerminalPanel panel = new TerminalPanel();
        var position = geometry.capture(panel);
        if (position == null || position.x() != -1500 || position.y() + position.height() != 800 || position.height() <= 0) throw new AssertionError("JediTerm point normalization");
        if (geometry.capture(panel).focus() != position.focus()) throw new AssertionError("Unstable focus");
        if (geometry.capture(new TerminalPanel()).focus() == position.focus()) throw new AssertionError("Pane focus collision");
        panel.rectangle = new Rectangle(100, 200, 1, 24);
        var box = geometry.capture(panel);
        if (box.y() != 200 || box.height() != 24) throw new AssertionError("Caret box changed");
        geometry.setComposing(panel, true);
        if (geometry.capture(panel) != null) throw new AssertionError("IME composition was not suppressed");
        geometry.setComposing(panel, false);
        if (geometry.capture(panel) == null) throw new AssertionError("IME did not resume");
        JPanel editor = new JPanel();
        panel.add(editor);
        JTextField search = new JTextField();
        panel.add(search);
        if (Geometry.isTerminal(new JPanel()) || Geometry.isTerminal(search)) throw new AssertionError("Editor/search accepted");
        if (args.length == 0) return;
        CursorTracker tracker = new CursorTracker(() -> new Geometry.Position(-1500, 780, 20, 123));
        System.out.println("READY");
        System.out.flush();
        System.in.read();
        tracker.dispose();
    }
}
