"""Real zsh + Rust bridge integration, isolated from the user's shell and desktop."""
import fcntl
import os
from pathlib import Path
import pty
import select
import struct
import subprocess
import tempfile
import termios
import time
import unittest

ROOT = Path(__file__).resolve().parents[1]
CLI = ROOT / "target/debug/figfig"


from helpers import Backend


class Shell:
    def __init__(self, directory, *, zed=False):
        self.directory = directory
        self.probe = directory / "probe"
        self.master, slave = pty.openpty()
        env = {**os.environ, "ZDOTDIR": str(directory), "FIGFIG_RUNTIME_DIR": str(directory / "runtime"), "FIGFIG_CONFIG_DIR": str(directory / "config"), "TERM": "xterm-256color", "LC_ALL": "en_US.UTF-8", "TEST_PROBE": str(self.probe)}
        env.pop("SSH_CONNECTION", None)
        env.pop("TMUX", None)
        env.pop("TERM_PROGRAM", None)
        if zed:
            env["TERM_PROGRAM"] = "zed"
            fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 18, 162, 1458, 342))
        (directory / ".zshrc").write_text(f"""
PROMPT='FIGFIG_PROMPT> '
RPROMPT=''
HISTFILE=/dev/null
HISTSIZE=100
bindkey -e
eval \"$({CLI} init zsh)\"
print -s -- 'echo history_completed'
_test_probe() {{
  print -r -- \"$BUFFER\" \"$_figfig_visible\" > \"$TEST_PROBE\"
  print -r -- \"fds=$_figfig_read_fd,$_figfig_write_fd rev=$_figfig_revision count=${{#_figfig_insert}} monitor=$options[monitor]\" >> \"$TEST_PROBE\"
  print -r -- '__PROBE_READY__' > /dev/tty
}}
zle -N _test_probe
bindkey '^G' _test_probe
""")
        self.process = subprocess.Popen(["/bin/zsh", "-d", "-i"], stdin=slave, stdout=slave, stderr=slave, cwd=directory, env=env, start_new_session=True, preexec_fn=lambda: fcntl.ioctl(0, termios.TIOCSCTTY, 0))
        os.close(slave)
        self.output = b""
        try:
            self.startup_output = self.wait_output(b"FIGFIG_PROMPT> ")
        except BaseException:
            self.close()
            raise

    def send(self, data):
        os.write(self.master, data.encode() if isinstance(data, str) else data)

    def wait_output(self, token, timeout=8):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if token in self.output:
                before, self.output = self.output.split(token, 1)
                return before
            readable, _, _ = select.select([self.master], [], [], max(0, deadline - time.monotonic()))
            if readable:
                try:
                    chunk = os.read(self.master, 65536)
                except OSError:
                    break
                if not chunk:
                    break
                self.output += chunk
        raise AssertionError(f"Terminal did not produce {token!r}: {self.output[-3000:]!r}")

    def state(self):
        self.send(b"\x07")
        self.wait_output(b"__PROBE_READY__")
        text = self.probe.read_text().splitlines()[0]
        buffer, visible = text.rsplit(" ", 1)
        return buffer, visible

    def wait_state(self, predicate, timeout=8):
        deadline = time.monotonic() + timeout
        state = None
        while time.monotonic() < deadline:
            state = self.state()
            if predicate(*state):
                return state
        raise AssertionError(f"Shell state did not match: {state!r}; probe={self.probe.read_text()!r}; messages={self.backend.messages[-2:]!r}; tty={self.output[-3000:]!r}")

    def close(self):
        os.close(self.master)
        if self.process.poll() is None:
            self.process.kill()
        self.process.wait(timeout=5)


class ShellIntegration(unittest.TestCase):
    def setUp(self):
        self.assertTrue(CLI.exists(), "Build CLI first: cargo build -p figfig-cli")
        self.temp = tempfile.TemporaryDirectory(prefix="figfig-test-", dir="/private/tmp")
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        (self.directory / "runtime").mkdir(mode=0o700)
        self.backend = Backend(self.directory / "runtime")
        self.addCleanup(self.backend.close)
        self.shell = Shell(self.directory)
        self.shell.backend = self.backend
        self.addCleanup(self.shell.close)

    def test_startup_hides_only_the_helper_job_notification(self):
        # line-init runs after the prompt; wait for a widget to ensure it finished.
        self.shell.send(b"\x07")
        output = self.shell.startup_output + self.shell.wait_output(b"__PROBE_READY__")
        self.assertNotRegex(output, rb"\[\d+\]\s+\d+")
        self.assertIn("monitor=on", self.shell.probe.read_text())

        self.shell.send("/bin/sleep 0.01 & wait $!\r")
        output = self.shell.wait_output(b"FIGFIG_PROMPT> ")
        self.assertRegex(output, rb"\[\d+\]\s+\d+")

    def test_enter_inserts_then_next_enter_executes(self):
        self.shell.send("echo fig")
        self.shell.wait_state(lambda text, visible: text == "echo fig" and visible == "1")
        self.shell.send("\r")
        self.shell.wait_state(lambda text, visible: text == "echo figfig_one " and visible == "0")
        self.shell.send("\r")
        output = self.shell.wait_output(b"FIGFIG_PROMPT> ")
        self.assertIn(b"\r\nfigfig_one\r\n", output)

    def test_arrows_choose_the_second_candidate_and_tab_inserts(self):
        self.shell.send("echo fig")
        self.shell.wait_state(lambda _, visible: visible == "1")
        self.shell.send(b"\x1b[B\t")
        self.shell.wait_state(lambda text, visible: text == "echo figfig_two " and visible == "0")

    def test_clearing_input_hides_popup_until_typing_resumes(self):
        command = "echo fig"
        self.shell.send(command)
        self.shell.wait_state(lambda text, visible: (text, visible) == (command, "1"))
        self.assertFalse(any(message["type"] == "hide" for message in self.backend.messages))

        for method, keys in (("Backspace", b"\x7f" * len(command)), ("Ctrl-U", b"\x15")):
            with self.subTest(method=method):
                request = self.backend.messages[-1]["request"]
                start = len(self.backend.messages)
                self.shell.send(keys)
                self.shell.wait_state(lambda text, visible: (text, visible) == ("", "0"))
                self.assertEqual(self.shell.state(), ("", "0"))

                # Receiving the next completion also waits for earlier messages on this socket.
                self.shell.send(command)
                self.shell.wait_state(lambda text, visible: (text, visible) == (command, "1"))
                messages = self.backend.messages[start:]
                hides = [message for message in messages if message["type"] == "hide"]
                self.assertEqual(len(hides), 1, messages)
                self.assertEqual(hides[0]["session_id"], request["sessionId"])
                self.assertLess(request["revision"], hides[0]["revision"])
                self.assertLess(hides[0]["revision"], messages[-1]["request"]["revision"])

    def test_zed_reports_the_cursor_cell_and_preserves_input_during_the_query(self):
        shell = Shell(self.directory, zed=True)
        shell.backend = self.backend
        self.addCleanup(shell.close)

        shell.send("echo fig")
        shell.wait_output(b"\x1b[5n\x1b[6n")
        shell.send(b"f\x1b[0n\x1b[3;24R")
        shell.wait_output(b"\x1b[5n\x1b[6n")
        shell.send(b"\x1b[0n\x1b[3;25R")
        shell.wait_state(lambda text, visible: text == "echo figf" and visible == "1")

        request = next(
            message["request"] for message in reversed(self.backend.messages)
            if message["type"] == "complete"
        )
        self.assertEqual(request["buffer"], "echo figf")
        self.assertEqual(request["terminalSize"]["cursor"], {
            "row": 3, "column": 25, "rows": 18, "columns": 162,
        })

    def test_zed_discards_a_cursor_report_after_the_query_timed_out(self):
        shell = Shell(self.directory, zed=True)
        shell.backend = self.backend
        self.addCleanup(shell.close)

        shell.send("echo fig")
        shell.wait_output(b"\x1b[5n\x1b[6n")
        shell.wait_state(lambda text, visible: text == "echo fig" and visible == "1")
        request = next(
            message["request"] for message in reversed(self.backend.messages)
            if message["type"] == "complete"
        )
        self.assertNotIn("cursor", request["terminalSize"])

        shell.send(b"\x1b[0n\x1b[3;24R\t")
        shell.wait_state(lambda text, visible: text == "echo figfig_one " and visible == "0")

    def test_bridge_exchanges_a_completion(self):
        env = {**os.environ, "FIGFIG_RUNTIME_DIR": str(self.directory / "runtime"), "FIGFIG_TERMINAL_PID": "0"}
        with subprocess.Popen([str(CLI), "bridge", "bridge-test"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env) as process:
            try:
                frame = ["complete", "1", "8", str(self.directory), "echo fig", "0"]
                process.stdin.write(("\0".join(frame) + "\0").encode())
                process.stdin.flush()
                ready, _, _ = select.select([process.stdout], [], [], 5)
                self.assertTrue(ready, f"Bridge did not reply: {self.backend.messages!r}")
                response = process.stdout.readline()
                if response.startswith(b"P "):
                    response = process.stdout.readline()
                self.assertTrue(response.startswith(b"R 1 5 8 1 2 "), response)
            finally:
                process.kill()
                process.wait(timeout=3)

    def test_unicode_path_is_quoted_as_a_single_argument(self):
        (self.directory / "日本 語.txt").write_text("fixture")
        self.shell.send("cat 日本")
        self.shell.wait_state(lambda _, visible: visible == "1")
        self.shell.send("\t")
        self.shell.wait_state(lambda text, _: text == "cat '日本 語.txt' ")

    def test_right_arrow_accepts_local_history_prediction(self):
        self.shell.send("echo hist")
        self.shell.wait_state(lambda text, _: text == "echo hist")
        self.shell.send(b"\x1b[C")
        self.shell.wait_state(lambda text, _: text == "echo history_completed")

    def test_disconnect_restores_ordinary_enter(self):
        self.shell.send("echo fig")
        self.shell.wait_state(lambda _, visible: visible == "1")
        self.backend.close()
        self.shell.wait_state(lambda _, visible: visible == "0")
        self.shell.send("\r")
        output = self.shell.wait_output(b"FIGFIG_PROMPT> ")
        self.assertIn(b"\r\nfig\r\n", output)


if __name__ == "__main__":
    unittest.main()
