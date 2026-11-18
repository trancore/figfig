"""Native cursor tracker geometry and IPC, without installing an input method."""
from pathlib import Path
import os
import json
import select
import subprocess
import sys
import tempfile
import time
import unittest

from helpers import Backend

ROOT = Path(__file__).resolve().parents[1]


@unittest.skipUnless(sys.platform == "darwin", "macOS InputMethodKit")
class CursorIntegration(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.build = tempfile.TemporaryDirectory(prefix="figfig-cursor-build-", dir="/private/tmp")
        cls.addClassCleanup(cls.build.cleanup)
        cls.executable = Path(cls.build.name) / "cursor-test"
        subprocess.run([
            "xcrun", "clang", "-fobjc-arc", "-Wall", "-Wextra", "-Wno-unused-parameter",
            "-DFIGFIG_HOSTS_JSON=" + json.dumps((ROOT / "assets/terminal-hosts.json").read_text()),
            "-framework", "ApplicationServices",
            "-framework", "AppKit", "-framework", "Carbon", "-framework", "InputMethodKit",
            str(ROOT / "tests/fixtures/cursor_test.m"), "-o", str(cls.executable),
        ], check=True, capture_output=True, text=True)
        cls.host = Path(cls.build.name) / "Zed Preview.app/Contents/MacOS/zed"
        cls.host.parent.mkdir(parents=True)
        subprocess.run(["xcrun", "clang", str(ROOT / "tests/fixtures/terminal-host.c"), "-o", str(cls.host)], check=True, capture_output=True, text=True)

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="figfig-cursor-", dir="/private/tmp")
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.backend = Backend(self.directory)
        self.addCleanup(self.backend.close)
        self.socket = self.directory / "app.sock"
        self.socket.chmod(0o600)

    def test_valid_geometry_is_sent_to_the_private_socket(self):
        subprocess.run([str(self.executable), str(self.socket)], check=True, timeout=5)
        deadline = time.monotonic() + 2
        while not self.backend.messages and time.monotonic() < deadline:
            self.backend.closed.wait(0.01)
        self.assertEqual(self.backend.messages, [{"type": "cursorPosition", "position": {
            "requestId": "fixture", "terminalPid": 42, "x": -1500, "y": 800, "height": 20, "focus": 123,
        }}])

    def check_bridge_host(self, host, helpers=()):
        cli = ROOT / "target/debug/figfig"
        env = {**os.environ, "FIGFIG_RUNTIME_DIR": str(self.directory)}
        env.pop("FIGFIG_TERMINAL_PID", None)
        process = subprocess.Popen([str(host), *map(str, helpers), str(cli), "bridge", "host-ancestry"], env=env,
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            ready, _, _ = select.select([process.stdout], [], [], 5)
            self.assertTrue(ready, "Bridge did not connect")
            self.assertEqual(process.stdout.readline(), b"P 1\n")
            frame = ["complete", "1", "8", str(self.directory), "echo fig", "0"]
            process.stdin.write(("\0".join(frame) + "\0").encode())
            process.stdin.flush()
            ready, _, _ = select.select([process.stdout], [], [], 5)
            self.assertTrue(ready, "Bridge did not return candidates")
            self.assertTrue(process.stdout.readline().startswith(b"R 1 "))
            request = next(message["request"] for message in self.backend.messages if message["type"] == "complete")
            self.assertEqual(request["terminalPid"], process.pid)
        finally:
            process.stdin.close()
            try:
                process.wait(timeout=3)
            finally:
                if process.poll() is None:
                    process.kill()
                    process.wait(timeout=3)
                process.stdout.close()
                process.stderr.close()

    def test_bridge_identifies_editor_hosts_through_pty_helpers(self):
        binary = self.host.read_bytes()
        for bundle, executable, wrappers in [
            ("Zed Preview.app", "zed", []),
            ("Visual Studio Code.app", "Electron", ["Code Helper", "pty-host"]),
            ("Cursor.app", "Cursor", ["Cursor Helper", "pty-host"]),
            ("IntelliJ IDEA.app", "idea", ["pty4j-unix-spawn-helper"]),
        ]:
            with self.subTest(bundle=bundle):
                host = Path(self.build.name) / bundle / "Contents/MacOS" / executable
                host.parent.mkdir(parents=True, exist_ok=True)
                host.write_bytes(binary)
                host.chmod(0o755)
                helpers = []
                for wrapper in wrappers:
                    path = Path(self.build.name) / wrapper
                    path.write_bytes(binary)
                    path.chmod(0o755)
                    helpers.append(path)
                self.check_bridge_host(host, helpers)
                self.backend.messages.clear()

    def test_insecure_socket_or_runtime_does_not_receive_geometry(self):
        for directory_mode, socket_mode in [(0o755, 0o600), (0o700, 0o666)]:
            with self.subTest(directory_mode=directory_mode, socket_mode=socket_mode):
                self.directory.chmod(directory_mode)
                self.socket.chmod(socket_mode)
                subprocess.run([str(self.executable), str(self.socket)], check=True, timeout=5)
                self.assertFalse(self.backend.connections)
                self.assertFalse(self.backend.messages)
        self.directory.chmod(0o700)
