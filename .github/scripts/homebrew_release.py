import os
import sys
import shutil
import hashlib
import subprocess
from pathlib import Path
from urllib.request import urlopen
import re

def run(cmd, cwd=None):
    r = subprocess.run(cmd, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    if r.returncode != 0:
        print(r.stdout)
        sys.exit(r.returncode)
    return r.stdout

def run_rc(cmd, cwd=None):
    r = subprocess.run(cmd, cwd=cwd)
    return r.returncode

def sha256_file(p):
    h = hashlib.sha256()
    with open(p, 'rb') as f:
        for b in iter(lambda: f.read(1024 * 1024), b''):
            h.update(b)
    return h.hexdigest()

def download(url, dest):
    with urlopen(url) as resp, open(dest, 'wb') as out:
        shutil.copyfileobj(resp, out)

def verify_checksums(repo, version, artifacts_dir):
    archs = [
        ("macos-x86_64", Path(artifacts_dir) / "qui-macos-x86_64" / "qui-macos-x86_64.tar.gz.sha256"),
        ("macos-aarch64", Path(artifacts_dir) / "qui-macos-aarch64" / "qui-macos-aarch64.tar.gz.sha256"),
        ("linux-x86_64", Path(artifacts_dir) / "qui-linux-x86_64" / "qui-linux-x86_64.tar.gz.sha256"),
    ]
    for a, exp_path in archs:
        url = f"https://github.com/{repo}/releases/download/v{version}/qui-{a}.tar.gz"
        tmp = Path("asset.tar.gz")
        download(url, tmp)
        calc = sha256_file(tmp)
        exp = Path(exp_path).read_text().strip()
        if not calc or len(calc) != 64:
            print("Invalid calc checksum")
            sys.exit(1)
        if calc != exp:
            print(f"Checksum mismatch for {a}")
            sys.exit(1)
        print(f"Checksum OK for {a}")
        tmp.unlink(missing_ok=True)

def write_formula(repo, version, artifacts_dir, formula_path):
    mx = Path(artifacts_dir) / "qui-macos-x86_64" / "qui-macos-x86_64.tar.gz.sha256"
    ma = Path(artifacts_dir) / "qui-macos-aarch64" / "qui-macos-aarch64.tar.gz.sha256"
    lx = Path(artifacts_dir) / "qui-linux-x86_64" / "qui-linux-x86_64.tar.gz.sha256"
    mxt = mx.read_text().strip()
    mat = ma.read_text().strip()
    lxt = lx.read_text().strip()
    for s in (mxt, mat, lxt):
        if len(s) != 64:
            print("Invalid checksum length")
            sys.exit(1)
    d = Path(formula_path).parent
    d.mkdir(parents=True, exist_ok=True)
    formula = f"""class Qui < Formula
  desc "Kubernetes Terminal UI - An intuitive TUI for managing Kubernetes clusters"
  homepage "https://github.com/{repo}"
  version "{version}"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/{repo}/releases/download/v{version}/qui-macos-x86_64.tar.gz"
      sha256 "{mxt}"
    elsif Hardware::CPU.arm?
      url "https://github.com/{repo}/releases/download/v{version}/qui-macos-aarch64.tar.gz"
      sha256 "{mat}"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/{repo}/releases/download/v{version}/qui-linux-x86_64.tar.gz"
      sha256 "{lxt}"
    end
  end

  def install
    bin.install "qui"
  end

  test do
    assert_predicate bin/"qui", :exist?
    assert_predicate bin/"qui", :executable?
  end
end
"""
    Path(formula_path).write_text(formula)
    rb = shutil.which("ruby")
    if rb:
        run([rb, "-c", str(formula_path)])
    if f"version \"{version}\"" not in Path(formula_path).read_text():
        print("Version not set in formula")
        sys.exit(1)
    print("Updated Formula/qui.rb with version and checksums")

def update_tap(tap_path, version):
    src = Path("Formula/qui.rb")
    dst = Path(tap_path) / "Formula" / "qui.rb"
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(src, dst)
    run(["git", "-c", "user.email=github-actions[bot]@users.noreply.github.com", "-c", "user.name=github-actions[bot]", "add", "Formula/qui.rb"], cwd=tap_path)
    r = subprocess.run(["git", "diff", "--cached", "--quiet"], cwd=tap_path)
    if r.returncode != 0:
        run(["git", "commit", "-m", f"Update QUI formula to v{version}"], cwd=tap_path)
        if run_rc(["git", "push", "origin", "HEAD:main"], cwd=tap_path) != 0:
            run(["git", "push", "origin", "HEAD:master"], cwd=tap_path)
    else:
        print("No tap changes")

def commit_local(version):
    run(["git", "-c", "user.email=github-actions[bot]@users.noreply.github.com", "-c", "user.name=github-actions[bot]", "add", "Formula/qui.rb"]) 
    r = subprocess.run(["git", "diff", "--cached", "--quiet"]) 
    if r.returncode != 0:
        run(["git", "commit", "-m", f"Sync local Formula/qui.rb to v{version}"]) 
        if run_rc(["git", "push", "origin", "HEAD:main"]) != 0:
            run(["git", "push", "origin", "HEAD:master"]) 
    else:
        print("No local changes")

def main():
    if len(sys.argv) < 2:
        print("usage: homebrew_release.py <verify-checksums|update-formula|detect-version> [args]")
        sys.exit(2)
    cmd = sys.argv[1]
    version = os.environ.get("VERSION")
    repo = os.environ.get("REPO")
    artifacts = os.environ.get("ARTIFACTS", "artifacts")
    if cmd != "detect-version":
        if not version or not repo:
            print("VERSION and REPO must be set")
            sys.exit(2)
    if cmd == "verify-checksums":
        verify_checksums(repo, version, artifacts)
        return
    if cmd == "update-formula":
        tap_path = "tap"
        if len(sys.argv) > 2:
            for i in range(2, len(sys.argv)):
                if sys.argv[i] == "--tap-path" and i + 1 < len(sys.argv):
                    tap_path = sys.argv[i + 1]
        write_formula(repo, version, artifacts, "Formula/qui.rb")
        update_tap(tap_path, version)
        commit_local(version)
        return
    if cmd == "detect-version":
        ref = os.environ.get("GITHUB_REF", "")
        tag = ""
        if ref.startswith("refs/tags/"):
            tag = ref.split("/")[-1]
            if tag.startswith("v"):
                tag = tag[1:]
        cargo = Path("Cargo.toml").read_text()
        m = re.search(r'^version\s*=\s*"([^"]+)"', cargo, re.MULTILINE)
        if not m:
            print("version not found in Cargo.toml")
            sys.exit(1)
        current = m.group(1)
        if tag and tag != current:
            new = re.sub(r'^version\s*=\s*"([^"]+)"', f'version = "{tag}"', cargo, flags=re.MULTILINE)
            Path("Cargo.toml").write_text(new)
            run(["git", "add", "Cargo.toml"]) 
            run(["git", "-c", "user.email=github-actions[bot]@users.noreply.github.com", "-c", "user.name=github-actions[bot]", "commit", "-m", f"chore(release): bump version to {tag}"]) 
            if run_rc(["git", "push", "origin", "HEAD:main"]) != 0:
                run(["git", "push", "origin", "HEAD:master"]) 
            version_out = tag
        else:
            version_out = current
        out = os.environ.get("GITHUB_OUTPUT")
        if not out:
            print("GITHUB_OUTPUT not set")
            sys.exit(2)
        Path(out).write_text(f"version={version_out}\n")
        return
    print("unknown command")
    sys.exit(2)

if __name__ == "__main__":
    main()
