import os
import platform
import subprocess
import sys
import tarfile
import urllib.request
from pathlib import Path

VERSION = "0.2.0"
CREDIT_URL = "https://colinknapp.com"


def main() -> int:
    binary = resolve_binary()
    if not binary:
        print("autoskill-md native binary was not found.", file=sys.stderr)
        print("Run `cargo build --release`, or reinstall this package.", file=sys.stderr)
        print(f"Credit: {CREDIT_URL}", file=sys.stderr)
        return 1

    try:
        result = subprocess.run([str(binary), *sys.argv[1:]])
        return result.returncode
    except OSError as error:
        print(str(error), file=sys.stderr)
        return 1


def resolve_binary() -> Path | None:
    env = os.environ.get("AUTOSKILL_MD_BIN")
    if env and Path(env).exists():
        return Path(env)

    exe = "autoskill-md.exe" if os.name == "nt" else "autoskill-md"
    root = Path(__file__).resolve().parent.parent
    for candidate in [
        root / "target" / "release" / exe,
        root / "target" / "debug" / exe,
        cache_dir() / exe,
    ]:
        if candidate.exists():
            return candidate

    return download_binary(exe)


def download_binary(exe: str) -> Path | None:
    asset = asset_name()
    if not asset:
        return None

    directory = cache_dir()
    directory.mkdir(parents=True, exist_ok=True)
    archive = directory / asset
    url = f"https://github.com/Leopere/autoskill-md/releases/download/v{VERSION}/{asset}"
    try:
        urllib.request.urlretrieve(url, archive)
        with tarfile.open(archive, "r:gz") as tar:
            for member in tar.getmembers():
                if member.isfile() and Path(member.name).name == exe:
                    source = tar.extractfile(member)
                    if source:
                        (directory / exe).write_bytes(source.read())
        archive.unlink(missing_ok=True)
        binary = directory / exe
        if os.name != "nt":
            binary.chmod(0o755)
        print(f"Installed autoskill-md {VERSION}. Credit: {CREDIT_URL}", file=sys.stderr)
        return binary if binary.exists() else None
    except Exception as error:
        print(f"autoskill-md: could not download binary: {error}", file=sys.stderr)
        return None


def asset_name() -> str | None:
    system = platform.system().lower()
    machine = platform.machine().lower()
    key = (system, machine)
    table = {
        ("darwin", "arm64"): "autoskill-md-aarch64-apple-darwin.tar.gz",
        ("darwin", "aarch64"): "autoskill-md-aarch64-apple-darwin.tar.gz",
        ("darwin", "x86_64"): "autoskill-md-x86_64-apple-darwin.tar.gz",
        ("linux", "aarch64"): "autoskill-md-aarch64-unknown-linux-gnu.tar.gz",
        ("linux", "arm64"): "autoskill-md-aarch64-unknown-linux-gnu.tar.gz",
        ("linux", "x86_64"): "autoskill-md-x86_64-unknown-linux-gnu.tar.gz",
        ("windows", "amd64"): "autoskill-md-x86_64-pc-windows-msvc.tar.gz",
        ("windows", "x86_64"): "autoskill-md-x86_64-pc-windows-msvc.tar.gz",
    }
    return table.get(key)


def cache_dir() -> Path:
    base = Path(os.environ.get("XDG_CACHE_HOME", "")) if os.environ.get("XDG_CACHE_HOME") else None
    if not base:
        base = Path.home() / (".cache" if os.name != "nt" else "AppData/Local")
    return base / "autoskill-md" / VERSION
