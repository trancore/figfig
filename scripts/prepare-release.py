"""Prepare a tagged source archive, checksum, and Homebrew formula (no publishing)."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tomllib


def git(*args):
    return subprocess.check_output(["git", *args], text=True).strip()


def prepare(repository, tag, output):
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
        raise ValueError("repository must be OWNER/REPO")
    if not re.fullmatch(r"v\d+\.\d+\.\d+(?:-[A-Za-z0-9.-]+)?", tag):
        raise ValueError("tag must be a version such as v0.1.0")
    commit = git("rev-parse", "--verify", f"refs/tags/{tag}^{{commit}}")
    if git("rev-parse", "HEAD") != commit:
        raise ValueError("Check out the requested tag before preparing a release")
    version = tag[1:]
    versions = {
        "package.json": json.loads(git("show", f"{commit}:package.json"))["version"],
        "Cargo.toml": tomllib.loads(git("show", f"{commit}:Cargo.toml"))["workspace"]["package"]["version"],
        "src-tauri/tauri.conf.json": json.loads(git("show", f"{commit}:src-tauri/tauri.conf.json"))["version"],
    }
    lock = tomllib.loads(git("show", f"{commit}:Cargo.lock"))
    for package in lock["package"]:
        if package["name"] in {"figfig-cli", "figfig-core", "figfig-desktop"}:
            versions[f"Cargo.lock:{package['name']}"] = package["version"]
    for name, value in versions.items():
        if value != version:
            raise ValueError(f"{name}: expected {version}, found {value}")

    output = Path(output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    archive = output / f"figfig-{version}-source.tar.gz"
    subprocess.run([
        "git", "archive", "--format=tar.gz", f"--prefix=figfig-{version}/",
        f"--output={archive}", commit,
    ], check=True)
    checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
    (output / "SHA256SUMS").write_text(f"{checksum}  {archive.name}\n")
    subprocess.run([
        sys.executable, str(Path(__file__).with_name("generate-formula.py")),
        "--repository", repository, "--version", version, "--sha256", checksum,
        "--output", str(output / "Formula/figfig.rb"),
    ], check=True)
    (output / "release-notes.md").write_text(
        f"Source build of figfig {version} from `{commit}`.\n\n"
        "The Homebrew formula is attached as `figfig.rb`. When this release is published as latest, "
        "GitHub Actions verifies the public source archive and updates `trancore/homebrew-tap`.\n\n"
        "Once the tap is updated:\n\n"
        "```sh\nbrew install trancore/tap/figfig\nfigfig setup\nfigfig permissions\n```\n\n"
        "macOS 13+, zsh 5.9+, Terminal.app or iTerm2. Installation builds locally "
        "with an ad-hoc signature; the helper is not notarized.\n"
    )
    print(f"Prepared {tag} ({commit}) in {output}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--output", type=Path, default=Path("release"))
    args = parser.parse_args()
    try:
        prepare(args.repository, args.tag, args.output)
    except (ValueError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"Release preparation failed: {error}\n")
