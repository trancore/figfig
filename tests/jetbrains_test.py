"""JetBrains adapter IPC against the real plugin and a geometry-only UI fixture."""
import json
import os
from pathlib import Path
import select
import socket
import subprocess
import tempfile
import time
import unittest
from helpers import Backend

ROOT = Path(__file__).resolve().parents[1]
IDE = Path(os.environ.get("FIGFIG_JETBRAINS_IDE", "/Applications/IntelliJ IDEA.app")) / "Contents"


@unittest.skipUnless((IDE / "jbr/Contents/Home/bin/javac").is_file(), "JetBrains IDE SDK (FIGFIG_JETBRAINS_IDE)")
class JetBrainsIntegration(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.build = tempfile.TemporaryDirectory(prefix="figfig-jb-build-", dir="/private/tmp")
        cls.addClassCleanup(cls.build.cleanup)
        cls.classes = Path(cls.build.name)
        cls.classpath = ":".join(map(str, [cls.classes, *sorted((IDE / "lib").glob("*.jar"))]))
        sources = [*sorted((ROOT / "editors/jetbrains/src").rglob("*.java")),
                   *sorted((ROOT / "tests/fixtures/jetbrains").rglob("*.java"))]
        subprocess.run([str(IDE / "jbr/Contents/Home/bin/javac"), "--release", "17", "-classpath", cls.classpath,
                        "-d", str(cls.classes), *map(str, sources)], check=True, capture_output=True, text=True)

    def test_geometry_and_private_socket_roundtrip(self):
        with tempfile.TemporaryDirectory(prefix="figfig-jb-", dir="/private/tmp") as directory:
            base = Path(directory)
            backend = Backend(base)
            self.addCleanup(backend.close)
            (base / "app.sock").chmod(0o600)
            process = subprocess.Popen([str(IDE / "jbr/Contents/Home/bin/java"), "-Djava.awt.headless=true", "-cp", self.classpath,
                                        "dev.figfig.terminal.AdapterTest", "serve"], env={**os.environ, "FIGFIG_RUNTIME_DIR": directory},
                                       stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            try:
                ready, _, _ = select.select([process.stdout], [], [], 10)
                self.assertTrue(ready, "Plugin fixture did not start")
                self.assertEqual(process.stdout.readline(), b"READY\n")
                endpoint = base / f"jb-{process.pid}.sock"
                deadline = time.monotonic() + 3
                while (not endpoint.exists() or endpoint.stat().st_mode & 0o777 != 0o600) and time.monotonic() < deadline:
                    backend.closed.wait(0.01)
                self.assertTrue(endpoint.exists(), "Plugin did not bind its private socket")
                self.assertEqual(endpoint.stat().st_mode & 0o777, 0o600)
                def send(request):
                    with socket.socket(socket.AF_UNIX) as client:
                        client.settimeout(2)
                        client.connect(str(endpoint))
                        client.sendall(request)
                        self.assertEqual(client.recv(1), b"", "Adapter did not finish the request")

                send(b"invalid\n")
                send(json.dumps({"requestId": "wrong", "terminalPid": process.pid + 1}).encode() + b"\n")
                (base / "app.sock").chmod(0o666)
                send(json.dumps({"requestId": "insecure", "terminalPid": process.pid}).encode() + b"\n")
                self.assertFalse(backend.messages)
                self.assertFalse(backend.connections)
                (base / "app.sock").chmod(0o600)
                send(json.dumps({"requestId": "fixture", "terminalPid": process.pid}).encode() + b"\n")
                deadline = time.monotonic() + 3
                while not backend.messages and time.monotonic() < deadline:
                    backend.closed.wait(0.01)
                self.assertEqual(backend.messages, [{"type": "cursorPosition", "position": {
                    "requestId": "fixture", "terminalPid": process.pid, "x": -1500.0, "y": 780.0,
                    "height": 20.0, "focus": 123, "coordinateSystem": "accessibility",
                }}])
                process.stdin.write(b"\n")
                process.stdin.flush()
                process.wait(timeout=5)
                self.assertEqual(process.returncode, 0, process.stderr.read().decode())
                self.assertFalse(endpoint.exists(), "Plugin left its socket behind")
            finally:
                if process.poll() is None:
                    process.kill()
                process.wait(timeout=5)
                for pipe in (process.stdin, process.stdout, process.stderr):
                    pipe.close()
