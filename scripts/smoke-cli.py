"""Exercise the real packaged background helper with disposable CLI configuration."""
import json
import os
from pathlib import Path
import select
import socket
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
CLI = ROOT / "target/release/figfig"
APP = ROOT / "target/release/bundle/macos/figfig.app"


def main():
    assert CLI.is_file() and APP.is_dir(), "Build first: pnpm run build:release"
    with tempfile.TemporaryDirectory(prefix="figfig-smoke-", dir="/private/tmp") as directory:
        base = Path(directory)
        env = {**os.environ, "FIGFIG_RUNTIME_DIR": str(base / "runtime"), "FIGFIG_CONFIG_DIR": str(base / "config"), "FIGFIG_APP": str(APP), "FIGFIG_TERMINAL_PID": "0"}

        def command(*args):
            result = subprocess.run([str(CLI), *args], env=env, capture_output=True, text=True, timeout=10)
            assert result.returncode == 0, f"{args}: {result.stderr}"
            return result.stdout

        bridge = None
        try:
            command("config", "set", "history", "true")
            assert not (base / "runtime/app.sock").exists(), "Editing settings started the renderer"
            bridge = subprocess.Popen([str(CLI), "bridge", "native-smoke"], env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)

            def preference(expected):
                ready, _, _ = select.select([bridge.stdout], [], [], 8)
                assert ready, f"No preference event {expected!r}"
                actual = bridge.stdout.readline()
                assert actual == expected, (actual, expected)

            preference(b"P 1\n")
            print("PASS: CLI bridge automatically started the packaged popup helper")
            command("config", "set", "history", "false")
            preference(b"P 0\n")
            assert json.loads(command("config"))["historyEnabled"] is False
            command("config", "set", "history", "true")
            preference(b"P 1\n")
            print("PASS: CLI configuration reaches the running shell bridge")
            command("pause")
            preference(b"P 0\n")
            command("resume")
            preference(b"P 1\n")
            print("PASS: CLI pause/resume reaches the running shell bridge")
            doctor = subprocess.run([str(CLI), "doctor"], env=env, capture_output=True, text=True, timeout=5)
            assert "Protocol: 1" in doctor.stdout, doctor.stderr
            print("PASS: CLI doctor reaches the native helper")
            with socket.socket(socket.AF_UNIX) as client:
                client.connect(str(base / "runtime/app.sock"))
                with client.makefile("rwb", buffering=0) as stream:
                    assert json.loads(stream.readline())["type"] == "preferences"
                    message = {"type": "setupCursor", "bundle_path": "/invalid.app", "enabled": False}
                    stream.write((json.dumps(message) + "\n").encode())
                    result = json.loads(stream.readline())
                    assert result == {"type": "error", "message": "Invalid cursor tracker bundle location"}, result
            print("PASS: native cursor setup rejects invalid bundle paths without registering an input method")
            fields = ["complete", "1", "6", str(ROOT), "git ch", "0"]
            bridge.stdin.write(("\0".join(fields) + "\0").encode())
            bridge.stdin.flush()
            ready, _, _ = select.select([bridge.stdout], [], [], 8)
            assert ready, "No native completion response"
            response = bridge.stdout.readline().decode().split()
            assert response[:5] == ["R", "1", "4", "6", "0"], response
            assert "checkout" in response[6::2], response
            print("PASS: native completion worker returns candidates through the CLI bridge")
            command("stop")
            bridge.wait(timeout=5)
            assert bridge.returncode == 0
            print("PASS: CLI stop disconnects the bridge cleanly")
        finally:
            subprocess.run([str(CLI), "stop"], env=env, capture_output=True, timeout=5)
            if bridge is not None:
                if bridge.poll() is None:
                    bridge.kill()
                bridge.wait(timeout=5)
                for pipe in (bridge.stdin, bridge.stdout, bridge.stderr):
                    pipe.close()


if __name__ == "__main__":
    main()
