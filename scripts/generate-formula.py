"""Generate a versioned, checksummed third-party Homebrew formula."""
import argparse
import re
from pathlib import Path


def generate(repository, version, sha256, source_url=None):
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
        raise ValueError("repository must be OWNER/REPO")
    if not re.fullmatch(r"\d+\.\d+\.\d+(?:-[A-Za-z0-9.-]+)?", version):
        raise ValueError("version must be a semantic version without v")
    if not re.fullmatch(r"[a-fA-F0-9]{64}", sha256):
        raise ValueError("sha256 must contain 64 hexadecimal characters")
    url = source_url or f"https://github.com/{repository}/releases/download/v{version}/figfig-{version}-source.tar.gz"
    if not re.fullmatch(r"(?:https://|file:///)[A-Za-z0-9_./%:+~-]+", url):
        raise ValueError("source URL must be a literal HTTPS or file URL")
    # Homebrew infers the version from our release filenames. Retain an explicit
    # version for local overrides such as file:///tmp/source.tar.gz.
    version_line = "" if source_url is None or url.endswith(f"/figfig-{version}-source.tar.gz") else f'  version "{version}"\n'
    return f'''class Figfig < Formula
  desc "Local autocomplete and inline suggestions for your existing terminal"
  homepage "https://github.com/{repository}"
  url "{url}"
{version_line}  sha256 "{sha256.lower()}"
  license "MIT"

  depends_on "node" => :build
  depends_on "pnpm@12" => :build
  depends_on "rust" => :build
  depends_on macos: :ventura

  def install
    ENV["MACOSX_DEPLOYMENT_TARGET"] = "13.0"
    ENV["PNPM_HOME"] = (buildpath/".pnpm").to_s
    ENV["XDG_DATA_HOME"] = (buildpath/".local/share").to_s
    ENV["XDG_CACHE_HOME"] = (buildpath/".cache").to_s
    ENV.prepend_path "PATH", formula_opt_bin("pnpm@12")
    system "pnpm", "install", "--frozen-lockfile", "--ignore-scripts", "--store-dir", buildpath/".pnpm-store"
    system "pnpm", "run", "build:release"
    bin.install "target/release/figfig"
    libexec.install "target/release/bundle/macos/figfig.app"
  end

  def caveats
    <<~EOS
      Set up zsh integration:
        figfig setup
      Allow terminal cursor positioning once:
        figfig permissions
      Open a new terminal. The popup helper starts automatically in the background.
      Configure suggestions from the command line:
        figfig config

      Enable figfig in System Settings > Privacy & Security > Accessibility.
      The private popup helper is built locally with an ad-hoc signature. It is not notarized.
      After upgrades, macOS may ask you to grant Accessibility again.
    EOS
  end

  test do
    assert_path_exists libexec/"figfig.app/Contents/MacOS/figfig-desktop"
    system "/usr/bin/codesign", "--verify", "--deep", "--strict", libexec/"figfig.app"
    assert_match version.to_s, shell_output("#{{bin}}/figfig --version")
    result = JSON.parse(shell_output("#{{bin}}/figfig complete 'git ch' '#{{testpath}}'"))
    assert result.fetch("candidates").any? {{ |candidate| candidate.fetch("label") == "checkout" }}
    assert_match "_figfig_receive", shell_output("#{{bin}}/figfig init zsh")
  end
end
'''


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--sha256", required=True)
    parser.add_argument("--source-url", help="Override for local source installation testing")
    parser.add_argument("--output", type=Path, default=Path("Formula/figfig.rb"))
    args = parser.parse_args()
    formula = generate(args.repository, args.version, args.sha256, args.source_url)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(formula)
    print(args.output)
