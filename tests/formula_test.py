import importlib.util
from pathlib import Path
import subprocess
import unittest

spec = importlib.util.spec_from_file_location("generate_formula", Path(__file__).resolve().parents[1] / "scripts/generate-formula.py")
generator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(generator)


class FormulaGeneration(unittest.TestCase):
    def test_generates_a_parseable_versioned_formula(self):
        formula = generator.generate("example/figfig", "0.1.0", "a" * 64)
        result = subprocess.run(["ruby", "-c"], input=formula, text=True, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("releases/download/v0.1.0/figfig-0.1.0-source.tar.gz", formula)
        self.assertIn('sha256 "' + "a" * 64 + '"', formula)
        self.assertIn('libexec.install "target/release/bundle/macos/figfig.app"', formula)
        self.assertIn('depends_on "pnpm@12" => :build', formula)
        self.assertIn('Formula["pnpm@12"].opt_bin', formula)
        self.assertIn("figfig setup", formula)
        self.assertIn("figfig permissions", formula)

    def test_rejects_repository_and_version_code_injection(self):
        with self.assertRaises(ValueError):
            generator.generate('owner/repo"; system "command"', "0.1.0", "a" * 64)
        with self.assertRaises(ValueError):
            generator.generate("owner/repo", "main", "a" * 64)
