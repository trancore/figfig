"""Exercise release preparation across real Git tags, archives, and formula generation."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/prepare-release.py"


class ReleasePreparation(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="figfig-release-test-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.env = {**os.environ, "GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_NOSYSTEM": "1"}
        self.git("init", "--initial-branch=main")
        self.write("package.json", json.dumps({"version": "0.1.0"}))
        self.write("Cargo.toml", '[workspace.package]\nversion = "0.1.0"\n')
        self.write("src-tauri/tauri.conf.json", json.dumps({"version": "0.1.0"}))
        self.write("Cargo.lock", '\n'.join(
            f'[[package]]\nname = "{name}"\nversion = "0.1.0"\n'
            for name in ["figfig-cli", "figfig-core", "figfig-desktop"]
        ))
        self.commit()
        self.git("tag", "-a", "v0.1.0", "-m", "test release")

    def git(self, *args):
        return subprocess.check_output([
            "git", "-c", "user.name=Release Test", "-c", "user.email=release@example.invalid",
            "-c", "commit.gpgsign=false", "-c", "tag.gpgsign=false", *args,
        ], cwd=self.root, env=self.env, text=True, stderr=subprocess.PIPE).strip()

    def write(self, name, value):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(value)

    def commit(self):
        self.git("add", ".")
        self.git("commit", "-m", "fixture")

    def prepare(self, tag="v0.1.0"):
        return subprocess.run([
            sys.executable, str(SCRIPT), "--repository", "trancore/figfig",
            "--tag", tag, "--output", str(self.root / "output"),
        ], cwd=self.root, env=self.env, text=True, capture_output=True)

    def test_archive_checksum_and_formula_describe_the_tagged_commit(self):
        # Local edits and untracked files must not enter a published archive.
        self.write("package.json", json.dumps({"version": "9.9.9"}))
        self.write("untracked.txt", "not part of the release")
        result = self.prepare()
        self.assertEqual(result.returncode, 0, result.stderr)
        output = self.root / "output"
        archive = output / "figfig-0.1.0-source.tar.gz"
        original = archive.read_bytes()
        checksum = hashlib.sha256(original).hexdigest()
        self.assertEqual((output / "SHA256SUMS").read_text(), f"{checksum}  {archive.name}\n")
        formula = (output / "Formula/figfig.rb").read_text()
        self.assertIn(f'sha256 "{checksum}"', formula)
        self.assertIn("https://github.com/trancore/figfig/releases/download/v0.1.0/" + archive.name, formula)
        with tarfile.open(archive) as source:
            self.assertNotIn("figfig-0.1.0/untracked.txt", source.getnames())
            package = json.load(source.extractfile("figfig-0.1.0/package.json"))
            self.assertEqual(package["version"], "0.1.0")
        parsed = subprocess.run(["ruby", "-c"], input=formula, text=True, capture_output=True)
        self.assertEqual(parsed.returncode, 0, parsed.stderr)
        self.assertEqual(self.prepare().returncode, 0)
        self.assertEqual(archive.read_bytes(), original, "Retry must produce the same source checksum")

    def test_rejects_version_mismatch_before_writing_artifacts(self):
        self.git("tag", "v0.2.0")
        result = self.prepare("v0.2.0")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("expected 0.2.0, found 0.1.0", result.stderr)
        self.assertFalse((self.root / "output").exists())

    def test_rejects_stale_cargo_lock_version(self):
        self.write("Cargo.lock", '[[package]]\nname = "figfig-cli"\nversion = "0.0.9"\n')
        self.commit()
        self.git("tag", "-f", "v0.1.0")
        result = self.prepare()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Cargo.lock:figfig-cli", result.stderr)
        self.assertFalse((self.root / "output").exists())

    def test_rejects_missing_tag_or_different_checked_out_commit(self):
        self.assertNotEqual(self.prepare("v8.0.0").returncode, 0)
        self.write("new.txt", "next commit")
        self.commit()
        result = self.prepare()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Check out the requested tag", result.stderr)
        self.assertFalse((self.root / "output").exists())

    def test_rejects_a_branch_name_as_a_release_tag(self):
        result = self.prepare("main")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("tag must be a version", result.stderr)
        self.assertFalse((self.root / "output").exists())
