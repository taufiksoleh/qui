class Qui < Formula
  desc "Kubernetes Terminal UI - An intuitive TUI for managing Kubernetes clusters"
  homepage "https://github.com/taufiksoleh/qui"
  version "0.0.11"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/taufiksoleh/qui/releases/download/v0.0.11/qui-macos-x86_64.tar.gz"
      sha256 "8200bcb74940befab496381a5aba05652e4f97813b94d5b5bfe214e1ce39c232"
    elsif Hardware::CPU.arm?
      url "https://github.com/taufiksoleh/qui/releases/download/v0.0.11/qui-macos-aarch64.tar.gz"
      sha256 "7b0f395ddb576055fced3e73df6dc3079838de1a3e84b9364e3b38b9696ad889"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/taufiksoleh/qui/releases/download/v0.0.11/qui-linux-x86_64.tar.gz"
      sha256 "758d932f9d3732cdcb6b76ea678877f85b503788c74d86b736cc317f78e63f65"
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
