"""Build against an installed JetBrains IDE; no downloaded SDK or global install."""
import argparse
from pathlib import Path
import subprocess
import tempfile
from zipfile import ZipFile, ZIP_DEFLATED

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ide", type=Path, default=Path("/Applications/IntelliJ IDEA.app"))
    args = parser.parse_args()
    contents = args.ide / "Contents"
    compiler = contents / "jbr/Contents/Home/bin/javac"
    jars = sorted((contents / "lib").glob("*.jar"))
    if not compiler.is_file() or not jars:
        parser.error("--ide must point to a JetBrains macOS app with its bundled JDK")
    output = ROOT / "target/editors/figfig-terminal.zip"
    output.parent.mkdir(parents=True, exist_ok=True)
    source = ROOT / "editors/jetbrains"
    with tempfile.TemporaryDirectory(prefix="figfig-jetbrains-") as temporary:
        classes = Path(temporary)
        subprocess.run([str(compiler), "--release", "17", "-Xlint:all", "-classpath", ":".join(map(str, jars)),
                        "-d", str(classes), *map(str, sorted((source / "src").rglob("*.java")))], check=True)
        jar = classes / "figfig-terminal.jar"
        with ZipFile(jar, "w", ZIP_DEFLATED) as archive:
            for path in sorted(classes.rglob("*.class")):
                archive.write(path, path.relative_to(classes))
            archive.write(source / "resources/META-INF/plugin.xml", "META-INF/plugin.xml")
        with ZipFile(output, "w", ZIP_DEFLATED) as archive:
            archive.write(jar, "figfig-terminal/lib/figfig-terminal.jar")
    print(output)


if __name__ == "__main__":
    main()
