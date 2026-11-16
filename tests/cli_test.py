"""CLI lifecycle, persistent settings, and safe zsh setup, using disposable homes."""
import json
import os
from pathlib import Path
import select
import signal
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
CLI = ROOT / "target/debug/figfig"


class CliIntegration(unittest.TestCase):
    def setUp(self):
        self.assertTrue(CLI.exists(), "Build first: cargo build -p figfig-cli")
        self.temp = tempfile.TemporaryDirectory(prefix="figfig-cli-", dir="/private/tmp")
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.runtime = self.directory / "runtime"
        self.config = self.directory / "config"
        self.app = self.directory / "fixture.app"
        executable = self.app / "Contents/MacOS/figfig-desktop"
        executable.parent.mkdir(parents=True)
        executable.write_text(f"#!{sys.executable}\nimport sys\nsys.path.insert(0, {str(ROOT / 'tests')!r})\nfrom helpers import run_helper\nrun_helper()\n")
        executable.chmod(0o755)
        self.env = {**os.environ, "HOME": str(self.directory), "ZDOTDIR": str(self.directory), "FIGFIG_RUNTIME_DIR": str(self.runtime), "FIGFIG_CONFIG_DIR": str(self.config), "FIGFIG_APP": str(self.app), "FIGFIG_TERMINAL_PID": "0"}
        self.addCleanup(self.stop_helper)

    def call(self, *args, ok=True):
        result = subprocess.run([str(CLI), *args], env=self.env, text=True, capture_output=True, timeout=10)
        if ok:
            self.assertEqual(result.returncode, 0, result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout)
        return result

    def stop_helper(self):
        if (self.runtime / "app.sock").exists():
            subprocess.run([str(CLI), "stop"], env=self.env, capture_output=True, timeout=5)
        pid = self.runtime / "fixture.pid"
        if pid.exists():
            try:
                os.kill(int(pid.read_text()), signal.SIGTERM)
            except (OSError, ValueError):
                pass

    def test_setup_preserves_existing_configuration_and_is_idempotent(self):
        rc = self.directory / ".zshrc"
        original = "# user settings\nexport USER_SETTING=preserved"
        rc.write_text(original)
        self.call("setup")
        content = rc.read_text()
        self.assertTrue(content.startswith(original + "\n"))
        self.assertIn("figfig init zsh", content)
        self.assertEqual((self.directory / ".zshrc.figfig-backup").read_text(), original)
        self.call("setup")
        self.assertEqual(rc.read_text(), content)
        self.assertFalse((self.runtime / "app.sock").exists())

    def test_settings_can_be_changed_without_starting_a_helper(self):
        self.call("config", "set", "popup", "false")
        self.call("config", "set", "max-candidates", "6")
        self.assertEqual(self.call("config", "get", "popup").stdout.strip(), "false")
        settings = json.loads(self.call("config").stdout)
        self.assertEqual(settings, {"popupEnabled": False, "historyEnabled": True, "maxCandidates": 6})
        self.assertEqual(self.call("settings").stdout, self.call("config").stdout)
        self.assertFalse((self.runtime / "app.sock").exists())

    def test_invalid_settings_do_not_overwrite_saved_values(self):
        self.call("config", "set", "history", "false")
        before = (self.config / "settings.json").read_bytes()
        for key, value in [("max-candidates", "2"), ("max-candidates", "13"), ("popup", "yes"), ("unknown", "true")]:
            with self.subTest(key=key, value=value):
                self.call("config", "set", key, value, ok=False)
                self.assertEqual((self.config / "settings.json").read_bytes(), before)

    def test_start_pause_resume_config_and_stop_are_controlled_from_cli(self):
        self.call("start")
        self.assertTrue((self.runtime / "app.sock").exists())
        self.call("start")
        self.call("pause")
        self.assertIn("Paused: true", self.call("doctor").stdout)
        self.call("resume")
        self.assertIn("Paused: false", self.call("doctor").stdout)
        self.call("config", "set", "history", "false")
        self.assertIn("already granted", self.call("permissions").stdout)
        self.call("stop")

    def test_bridge_automatically_starts_the_background_helper(self):
        process = subprocess.Popen([str(CLI), "bridge", "auto-start"], env=self.env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            ready, _, _ = select.select([process.stdout], [], [], 8)
            self.assertTrue(ready, "Bridge did not start its helper")
            self.assertEqual(process.stdout.readline(), b"P 1\n")
            self.assertTrue((self.runtime / "app.sock").exists())
        finally:
            process.kill()
            process.wait(timeout=3)
            for pipe in [process.stdin, process.stdout, process.stderr]:
                pipe.close()
