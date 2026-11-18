package dev.figfig.terminal;

import com.google.gson.Gson;
import com.google.gson.JsonObject;
import com.intellij.openapi.Disposable;
import java.awt.KeyboardFocusManager;
import java.awt.Window;
import java.awt.AWTEvent;
import java.awt.Component;
import java.awt.Toolkit;
import java.awt.event.AWTEventListener;
import java.awt.event.InputMethodEvent;
import java.io.IOException;
import java.net.StandardProtocolFamily;
import java.net.UnixDomainSocketAddress;
import java.nio.ByteBuffer;
import java.nio.channels.ServerSocketChannel;
import java.nio.channels.SocketChannel;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.LinkOption;
import java.nio.file.Path;
import java.nio.file.attribute.PosixFilePermissions;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.function.Supplier;
import javax.swing.SwingUtilities;

/** Private, bounded local IPC; all UI geometry is captured on the EDT. */
public final class CursorTracker implements Disposable {
    private static final Gson JSON = new Gson();
    private final Geometry geometry = new Geometry();
    private volatile ServerSocketChannel server;
    private volatile boolean disposed;
    private Path endpoint;
    private Object endpointKey;
    private final Supplier<Geometry.Position> capture;
    private final AWTEventListener composition = event -> {
        if (event instanceof InputMethodEvent input && input.getID() == InputMethodEvent.INPUT_METHOD_TEXT_CHANGED
                && input.getSource() instanceof Component component) {
            var text = input.getText();
            // Inspect only composition length, never characters or keystrokes.
            geometry.setComposing(component, text != null &&
                    text.getEndIndex() - text.getBeginIndex() > input.getCommittedCharacterCount());
        }
    };

    public CursorTracker() {
        this(null);
    }

    CursorTracker(Supplier<Geometry.Position> capture) {
        this.capture = capture;
        if (!System.getProperty("os.name").startsWith("Mac")) return;
        Toolkit.getDefaultToolkit().addAWTEventListener(composition, AWTEvent.INPUT_METHOD_EVENT_MASK);
        Thread worker = new Thread(this::serve, "figfig-terminal-cursor");
        worker.setDaemon(true);
        worker.start();
    }

    private void serve() {
        try {
            Path home = Path.of(System.getProperty("user.home"));
            int uid = ((Number) Files.getAttribute(home, "unix:uid")).intValue();
            String override = System.getenv("FIGFIG_RUNTIME_DIR");
            Path runtime = override == null ? Path.of(System.getProperty("java.io.tmpdir"), "figfig-" + uid) : Path.of(override);
            if (!Files.exists(runtime, LinkOption.NOFOLLOW_LINKS)) {
                try {
                    Files.createDirectory(runtime, PosixFilePermissions.asFileAttribute(PosixFilePermissions.fromString("rwx------")));
                } catch (java.nio.file.FileAlreadyExistsException ignored) {
                    // A shell/helper may create the directory concurrently.
                }
            }
            if (!privatePath(runtime, uid, 0040000)) return;
            endpoint = runtime.resolve("jb-" + ProcessHandle.current().pid() + ".sock");
            // Never replace an unrelated file, symlink, or existing listener.
            if (Files.exists(endpoint, LinkOption.NOFOLLOW_LINKS)) return;
            try (ServerSocketChannel listener = ServerSocketChannel.open(StandardProtocolFamily.UNIX)) {
                server = listener;
                if (disposed) return;
                listener.bind(UnixDomainSocketAddress.of(endpoint));
                Files.setPosixFilePermissions(endpoint, PosixFilePermissions.fromString("rw-------"));
                endpointKey = Files.readAttributes(endpoint, java.nio.file.attribute.BasicFileAttributes.class).fileKey();
                while (!disposed) {
                    try (SocketChannel client = listener.accept()) {
                        client.configureBlocking(false);
                        String line = readLine(client);
                        JsonObject request = JSON.fromJson(line, JsonObject.class);
                        long pid = request.get("terminalPid").getAsLong();
                        String id = request.get("requestId").getAsString();
                        if (pid != ProcessHandle.current().pid() || id.length() > 256) continue;
                        CompletableFuture<Geometry.Position> result = new CompletableFuture<>();
                        SwingUtilities.invokeLater(() -> {
                            try {
                                if (capture != null) result.complete(capture.get());
                                else {
                                    KeyboardFocusManager manager = KeyboardFocusManager.getCurrentKeyboardFocusManager();
                                    Window window = manager.getFocusedWindow();
                                    result.complete(window != null && window.isActive() ? geometry.capture(manager.getFocusOwner()) : null);
                                }
                            } catch (RuntimeException error) { result.complete(null); }
                        });
                        Geometry.Position position = result.get(200, TimeUnit.MILLISECONDS);
                        if (position == null || disposed || !privatePath(runtime, uid, 0040000)) continue;
                        Path socket = runtime.resolve("app.sock");
                        if (!privatePath(socket, uid, 0140000)) continue;
                        JsonObject value = new JsonObject();
                        value.addProperty("requestId", id);
                        value.addProperty("terminalPid", pid);
                        value.addProperty("x", position.x());
                        value.addProperty("y", position.y());
                        value.addProperty("height", position.height());
                        value.addProperty("focus", position.focus());
                        value.addProperty("coordinateSystem", "accessibility");
                        JsonObject message = new JsonObject();
                        message.addProperty("type", "cursorPosition");
                        message.add("position", value);
                        try (SocketChannel output = SocketChannel.open(StandardProtocolFamily.UNIX)) {
                            output.configureBlocking(false);
                            output.connect(UnixDomainSocketAddress.of(socket));
                            long deadline = System.nanoTime() + TimeUnit.MILLISECONDS.toNanos(250);
                            while (!output.finishConnect()) await(deadline);
                            readLine(output); // Consume the initial preferences.
                            ByteBuffer bytes = StandardCharsets.UTF_8.encode(JSON.toJson(message) + "\n");
                            while (bytes.hasRemaining()) {
                                output.write(bytes);
                                if (bytes.hasRemaining()) await(deadline);
                            }
                        }
                    } catch (Exception ignored) {
                        // Missing focus, a busy EDT, malformed frames, or a stopped helper hide suggestions.
                    }
                }
            }
        } catch (IOException ignored) {
            // The CLI diagnoses a missing adapter; never prevent IDE startup.
        } finally {
            cleanup();
        }
    }

    static boolean privatePath(Path path, int uid, int kind) throws IOException {
        int mode = ((Number) Files.getAttribute(path, "unix:mode", LinkOption.NOFOLLOW_LINKS)).intValue();
        int owner = ((Number) Files.getAttribute(path, "unix:uid", LinkOption.NOFOLLOW_LINKS)).intValue();
        return owner == uid && (mode & 077) == 0 && (mode & 0170000) == kind;
    }

    private static String readLine(SocketChannel channel) throws Exception {
        ByteBuffer bytes = ByteBuffer.allocate(8192);
        long deadline = System.nanoTime() + TimeUnit.MILLISECONDS.toNanos(250);
        while (bytes.hasRemaining()) {
            int count = channel.read(bytes);
            if (count < 0) throw new IOException("Incomplete frame");
            for (int i = bytes.position() - count; i < bytes.position(); i++) {
                if (bytes.get(i) == '\n') {
                    return new String(bytes.array(), 0, i, StandardCharsets.UTF_8);
                }
            }
            await(deadline);
        }
        throw new IOException("Frame too large");
    }

    private static void await(long deadline) throws Exception {
        if (System.nanoTime() >= deadline) throw new IOException("IPC timeout");
        Thread.sleep(2);
    }

    private void cleanup() {
        try {
            if (endpointKey != null && endpointKey.equals(Files.readAttributes(endpoint,
                    java.nio.file.attribute.BasicFileAttributes.class, LinkOption.NOFOLLOW_LINKS).fileKey())) {
                Files.delete(endpoint);
            }
        } catch (IOException ignored) { }
    }

    @Override public void dispose() {
        disposed = true;
        Toolkit.getDefaultToolkit().removeAWTEventListener(composition);
        ServerSocketChannel listener = server;
        if (listener != null) try { listener.close(); } catch (IOException ignored) { }
    }
}
