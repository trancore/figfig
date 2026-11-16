"""Local popup-service fixtures. Never opens a GUI or changes OS permissions."""
import json
import os
from pathlib import Path
import socket
import subprocess
import threading

ROOT = Path(__file__).resolve().parents[1]
CLI = ROOT / "target/debug/figfig"


class Backend:
    def __init__(self, directory):
        self.directory = directory
        self.socket = socket.socket(socket.AF_UNIX)
        self.socket.bind(str(directory / "app.sock"))
        self.socket.listen()
        self.socket.settimeout(0.1)
        self.closed = threading.Event()
        self.connections = []
        self.workers = []
        self.messages = []
        self.paused = False
        self.settings = {"popupEnabled": True, "historyEnabled": True, "maxCandidates": 8}
        self.thread = threading.Thread(target=self.run, daemon=True)
        self.thread.start()

    def run(self):
        while not self.closed.is_set():
            try:
                conn, _ = self.socket.accept()
            except (socket.timeout, OSError):
                continue
            self.connections.append(conn)
            worker = threading.Thread(target=self.serve, args=(conn,), daemon=True)
            self.workers.append(worker)
            worker.start()

    def send(self, conn, message):
        conn.sendall((json.dumps(message) + "\n").encode())

    def serve(self, conn):
        try:
            self.send(conn, {"type": "preferences", "settings": self.settings, "paused": self.paused})
            with conn.makefile("rb") as stream:
                for line in stream:
                    msg = json.loads(line)
                    self.messages.append(msg)
                    kind = msg["type"]
                    if kind == "status":
                        self.send(conn, {"type": "status", "version": 1, "paused": self.paused, "accessibility": True, "terminal": "Fixture", "error": None})
                    elif kind == "setPaused":
                        self.paused = msg["paused"]
                        self.send(conn, {"type": "ack"})
                    elif kind == "reloadSettings":
                        config = Path(os.environ.get("FIGFIG_CONFIG_DIR", str(self.directory.parent / "config"))) / "settings.json"
                        self.settings = json.loads(config.read_text())
                        self.send(conn, {"type": "ack"})
                    elif kind == "requestAccessibility":
                        self.send(conn, {"type": "accessibility", "granted": True})
                    elif kind == "shutdown":
                        self.send(conn, {"type": "ack"})
                        self.closed.set()
                        break
                    elif kind == "complete":
                        request = msg["request"]
                        buffer = request["buffer"]
                        if buffer.startswith("echo fig"):
                            response = {
                                "sessionId": request["sessionId"], "revision": request["revision"],
                                "replaceStart": 5, "replaceEnd": len(buffer), "selected": 0, "visible": True,
                                "candidates": [{"label": value, "insertText": value, "description": "Fixture", "kind": "file", "appendSpace": True} for value in ["figfig_one", "figfig_two"]],
                            }
                        else:
                            result = subprocess.run([str(CLI), "complete", buffer, request["cwd"]], capture_output=True, text=True, check=True)
                            response = json.loads(result.stdout)
                            response.update(sessionId=request["sessionId"], revision=request["revision"], visible=True)
                        self.send(conn, {"type": "completion", "response": response})
        except (OSError, ValueError, subprocess.SubprocessError):
            pass

    def close(self):
        self.closed.set()
        self.socket.close()
        for conn in self.connections:
            try:
                conn.shutdown(socket.SHUT_RDWR)
            except OSError:
                pass
            conn.close()
        self.thread.join(timeout=2)
        for worker in self.workers:
            worker.join(timeout=2)


def run_helper():
    directory = Path(os.environ["FIGFIG_RUNTIME_DIR"])
    pid = directory / "fixture.pid"
    pid.write_text(str(os.getpid()))
    backend = Backend(directory)
    try:
        backend.closed.wait()
    finally:
        backend.close()
        (directory / "app.sock").unlink(missing_ok=True)
        pid.unlink(missing_ok=True)
