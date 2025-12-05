class Qui < Formula
  desc "Kubernetes Terminal UI - An intuitive TUI for managing Kubernetes clusters"
  homepage "https://github.com/taufiksoleh/qui"
  version "0.0.12"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/taufiksoleh/qui/releases/download/v0.0.12/qui-macos-x86_64.tar.gz"
      sha256 "58e92f9efc9ade057079e51854052d5e6af5d396b8a98d7dfec3a8e31fbc5ae3"
    elsif Hardware::CPU.arm?
      url "https://github.com/taufiksoleh/qui/releases/download/v0.0.12/qui-macos-aarch64.tar.gz"
      sha256 "5aadd2e93382021283432c79fa05012285c52287c9e29c7cc4b6aaee6ed88b2b"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/taufiksoleh/qui/releases/download/v0.0.12/qui-linux-x86_64.tar.gz"
      sha256 "c02927345921076b1d6ab732d9ce5ffe48ddd97cf23841fa33b37435e54aedcd"
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
